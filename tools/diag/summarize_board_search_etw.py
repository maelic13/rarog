#!/usr/bin/env python3
"""Recover and summarize PLAN 4.11b.7 ETW profiles with LLVM symbols.

The ETW capture stores executable-relative addresses in xperf's exclusive-hit
table.  This tool resolves those addresses against the exact archived PE/PDB
pair and attributes sampled full-search time to mutually exclusive board-work
regions.  All inline frames participate in attribution so a hot shared helper
such as a sliding attack is charged to its board caller rather than its leaf.

This is also a recovery path for reports produced before the ETW runner kept
the PDB under its original ``rarog.pdb`` name.  The executable and matching PDB
must be beside each other when llvm-symbolizer is invoked.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from collections import Counter
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path
from typing import Iterable


IMAGE_BASE = 0x140000000


class StackReportParser(HTMLParser):
    """Extract tables from xperf's XHTML-like stack report."""

    def __init__(self) -> None:
        super().__init__()
        self.tables: dict[str, list[list[str]]] = {}
        self._heading = ""
        self._in_h2 = False
        self._heading_parts: list[str] = []
        self._table_heading: str | None = None
        self._in_cell = False
        self._cell_parts: list[str] = []
        self._row: list[str] | None = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        del attrs
        if tag == "h2":
            self._in_h2 = True
            self._heading_parts = []
        elif tag == "table":
            self._table_heading = self._heading
            self.tables.setdefault(self._table_heading, [])
        elif tag == "tr" and self._table_heading is not None:
            self._row = []
        elif tag in {"td", "th"} and self._row is not None:
            self._in_cell = True
            self._cell_parts = []

    def handle_endtag(self, tag: str) -> None:
        if tag == "h2":
            self._in_h2 = False
            self._heading = " ".join("".join(self._heading_parts).split())
        elif tag in {"td", "th"} and self._in_cell and self._row is not None:
            self._row.append(" ".join("".join(self._cell_parts).split()))
            self._in_cell = False
        elif tag == "tr" and self._row is not None and self._table_heading is not None:
            self.tables[self._table_heading].append(self._row)
            self._row = None
        elif tag == "table":
            self._table_heading = None

    def handle_data(self, data: str) -> None:
        if self._in_h2:
            self._heading_parts.append(data)
        if self._in_cell:
            self._cell_parts.append(data)


@dataclass(frozen=True)
class ExclusiveHit:
    rva: int
    hits: int


def parse_report(path: Path, module_name: str) -> tuple[int, list[ExclusiveHit]]:
    parser = StackReportParser()
    parser.feed(path.read_text(encoding="utf-8", errors="strict"))

    process_rows = parser.tables.get("Processes and Root functions")
    exclusive_rows = parser.tables.get("Functions by Exclusive Hits")
    if not process_rows or not exclusive_rows:
        raise ValueError(f"{path}: required xperf tables are missing")

    process_matches = [
        row for row in process_rows if row and Path(row[0]).name.casefold() == module_name.casefold()
    ]
    if len(process_matches) != 1 or len(process_matches[0]) < 3:
        raise ValueError(f"{path}: expected exactly one {module_name} process row")
    total_samples = int(process_matches[0][2].replace(",", ""))

    # xperf emits the SAME column header for two very different tables, so the
    # schema must be detected from the DATA, not the header.
    #
    #   per-address  base == limit, size == 0  -- one row per sampled address,
    #                names are `***unknown***`. This is what this tool needs:
    #                it recovers the symbols itself, with the full inline chain,
    #                so a hot inlined helper is charged to its board caller.
    #
    #   per-function base <  limit, size >  0  -- xperf resolved symbols and
    #                aggregated by function. Board work inlined into a large
    #                search function is then charged to that function and never
    #                appears under its own region.
    #
    # Reading a fixed index happened to work on the per-address table because
    # base == limit there. On the per-function table index 5 is `limit`, the
    # byte one past the end of each function, so every lookup resolved into the
    # next function or into padding while still reporting 100% resolved.
    #
    # The per-function table is NOT usable by correcting the column: it read
    # make/unmake at 3.59% where RAR-M33's measured speedup independently
    # requires about 6.3%. Refuse it and say how to regenerate.
    header = next(
        (
            row
            for row in exclusive_rows
            if row and row[0].strip().casefold() in ("function", "function name")
        ),
        None,
    )
    if header is None:
        raise ValueError(f"{path}: exclusive-hits table has no header row")
    columns = {name.strip().casefold(): index for index, name in enumerate(header)}
    count_column = next(
        (columns[name] for name in ("hits", "exclusivehits") if name in columns), 1
    )

    if "address" in columns:
        # Legacy six-column form; the address is explicit.
        address_column = columns["address"]
        size_column = None
    else:
        for required in ("base", "limit"):
            if required not in columns:
                raise ValueError(
                    f"{path}: exclusive-hits header lacks {required!r}; got {header!r}"
                )
        address_column = columns["base"]
        size_column = columns.get("size")

    hits: list[ExclusiveHit] = []
    prefix = f"{module_name}!".casefold()
    aggregated = 0
    for row in exclusive_rows:
        if row is header:
            continue
        if len(row) <= max(count_column, address_column):
            continue
        if not row[0].casefold().startswith(prefix):
            continue
        try:
            count = int(row[count_column].replace(",", ""))
            rva = int(row[address_column], 16)
            if size_column is not None and len(row) > size_column:
                if int(row[size_column], 16) != 0:
                    aggregated += count
        except ValueError as exc:
            raise ValueError(f"{path}: malformed exclusive row {row!r}") from exc
        hits.append(ExclusiveHit(rva=rva, hits=count))

    if aggregated:
        raise ValueError(
            f"{path}: this report is aggregated PER FUNCTION ({aggregated} hits in "
            "rows with a non-zero size), so board work inlined into a search "
            "function is charged to that function and the shares would be wrong. "
            "Regenerate the butterfly with xperf unable to resolve symbols "
            "(empty _NT_SYMBOL_PATH and empty _NT_SYMCACHE_PATH, no PDB beside "
            "the executable) so each sampled address gets its own row."
        )

    if not hits or sum(item.hits for item in hits) >= total_samples:
        if not hits:
            raise ValueError(f"{path}: no exclusive {module_name} rows")
        if sum(item.hits for item in hits) > total_samples:
            raise ValueError(f"{path}: module hits exceed process samples")
    return total_samples, hits


def symbolize(
    symbolizer: Path, executable: Path, rvas: Iterable[int]
) -> dict[int, list[str]]:
    ordered = sorted(set(rvas))
    addresses = [IMAGE_BASE + rva for rva in ordered]
    proc = subprocess.run(
        [
            str(symbolizer),
            f"--obj={executable}",
            "--inlines",
            "--demangle",
            "--output-style=JSON",
        ],
        input="".join(f"0x{address:x}\n" for address in addresses),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"llvm-symbolizer failed ({proc.returncode}): {proc.stderr.strip()}"
        )
    lines = [line for line in proc.stdout.splitlines() if line.strip()]
    if len(lines) != len(ordered):
        raise ValueError(
            f"llvm-symbolizer returned {len(lines)} records for {len(ordered)} addresses"
        )

    resolved: dict[int, list[str]] = {}
    for expected_rva, line in zip(ordered, lines, strict=True):
        record = json.loads(line)
        expected_address = IMAGE_BASE + expected_rva
        if int(record["Address"], 16) != expected_address:
            raise ValueError("llvm-symbolizer changed address order")
        names = [frame["FunctionName"] for frame in record.get("Symbol", [])]
        resolved[expected_rva] = names
    return resolved


def classify(functions: list[str]) -> str:
    """Assign one exclusive sample location using its complete inline context."""
    joined = "\n".join(functions).casefold()

    # More specific consumers precede shared geometry and mutation helpers.
    if "rarog::board::see" in joined or "::see_" in joined:
        return "see"
    if (
        "rarog::board::movegen::" in joined
        or "rarog::board::moves::movelist" in joined
        or any(
            marker in joined
            for marker in (
                "::generate_legal",
                "::legal_move",
                "::pseudo_legal_move",
                "::king_safe_after",
                "::ep_capture_is_legal",
            )
        )
    ):
        return "generation_and_legality"
    if any(
        marker in joined
        for marker in (
            "::make_move",
            "::unmake_move",
            "::make_null",
            "::unmake_null",
            "::remove_piece",
            "::add_piece",
            "::captured_piece",
        )
    ):
        return "make_unmake"
    if any(
        marker in joined
        for marker in (
            "::gives_check",
            "::check_info",
            "::calculate_checkers",
            "::is_attacked",
            "::attackers_to",
        )
    ):
        return "check_queries"
    if "rarog::board::" in joined:
        return "other_board"
    return "other_engine"


def mechanisms(functions: list[str]) -> list[str]:
    """Return overlapping leaf-specific mechanisms visible in inline context."""
    joined = "\n".join(functions).casefold()
    markers = {
        "piece_relocation_helpers": ("::remove_piece", "::add_piece", "::move_piece"),
        "compute_pinned": ("::compute_pinned",),
        "check_info": ("::check_info",),
        "gives_check": ("::gives_check",),
        "king_square_lookup": ("::king_sq",),
    }
    return [
        name
        for name, candidates in markers.items()
        if any(candidate in joined for candidate in candidates)
    ]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def summarize_report(
    cohort: str,
    total_samples: int,
    hits: list[ExclusiveHit],
    symbols: dict[int, list[str]],
) -> dict[str, object]:
    categories: Counter[str] = Counter()
    mechanism_counts: Counter[str] = Counter()
    leaf_functions: Counter[str] = Counter()
    resolved_hits = 0
    for item in hits:
        functions = symbols[item.rva]
        if functions and functions[0] != "??":
            resolved_hits += item.hits
            categories[classify(functions)] += item.hits
            for mechanism in mechanisms(functions):
                mechanism_counts[mechanism] += item.hits
            leaf_functions[functions[0]] += item.hits
        else:
            categories["unresolved_engine"] += item.hits

    engine_hits = sum(item.hits for item in hits)
    return {
        "cohort": cohort,
        "process_samples": total_samples,
        "engine_samples": engine_hits,
        "engine_share_percent": round(100.0 * engine_hits / total_samples, 3),
        "resolved_engine_samples": resolved_hits,
        "resolved_engine_percent": round(100.0 * resolved_hits / engine_hits, 3),
        "sample_counts": dict(sorted(categories.items())),
        "sample_shares_percent": {
            name: round(100.0 * count / total_samples, 3)
            for name, count in sorted(categories.items())
        },
        "overlapping_mechanism_counts": dict(sorted(mechanism_counts.items())),
        "overlapping_mechanism_shares_percent": {
            name: round(100.0 * count / total_samples, 3)
            for name, count in sorted(mechanism_counts.items())
        },
        "top_exclusive_functions": [
            {
                "function": name,
                "samples": count,
                "process_share_percent": round(100.0 * count / total_samples, 3),
            }
            for name, count in leaf_functions.most_common(15)
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument(
        "--pdb",
        type=Path,
        help="matching PDB (defaults to the executable stem); keep its embedded/original name beside the PE",
    )
    parser.add_argument("--symbolizer", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("reports", type=Path, nargs="+")
    args = parser.parse_args()

    pdb = args.pdb or args.exe.with_suffix(".pdb")
    if not args.exe.is_file() or not pdb.is_file() or not args.symbolizer.is_file():
        parser.error("--exe, --pdb and --symbolizer must name files")

    parsed: list[tuple[str, int, list[ExclusiveHit]]] = []
    all_rvas: set[int] = set()
    module_name = args.exe.name
    for report in args.reports:
        suffix = "-butterfly"
        cohort = report.stem.removesuffix(suffix)
        total, hits = parse_report(report, module_name)
        parsed.append((cohort, total, hits))
        all_rvas.update(item.rva for item in hits)

    symbols = symbolize(args.symbolizer, args.exe, all_rvas)
    reports = [summarize_report(*item, symbols) for item in parsed]

    process_total = sum(int(report["process_samples"]) for report in reports)
    aggregate_shares: Counter[str] = Counter()
    aggregate_mechanisms: Counter[str] = Counter()
    for report in reports:
        aggregate_shares.update(report["sample_counts"])
        aggregate_mechanisms.update(report["overlapping_mechanism_counts"])

    result = {
        "schema": "rarog-board-search-etw-summary-v1",
        "classification": {
            "basis": "exclusive CPU samples attributed with complete LLVM inline context",
            "priority": [
                "see",
                "generation_and_legality",
                "make_unmake",
                "check_queries",
                "other_board",
                "other_engine",
            ],
            "history_allocation": "not inferred from samples; use exact growth counters",
        },
        "image_base": f"0x{IMAGE_BASE:x}",
        "executable": str(args.exe.resolve()),
        "executable_sha256": sha256(args.exe),
        "pdb": str(pdb.resolve()),
        "pdb_sha256": sha256(pdb),
        "process_samples": process_total,
        "weighted_sample_shares_percent": {
            name: round(100.0 * count / process_total, 3)
            for name, count in sorted(aggregate_shares.items())
        },
        "weighted_overlapping_mechanism_shares_percent": {
            name: round(100.0 * count / process_total, 3)
            for name, count in sorted(aggregate_mechanisms.items())
        },
        "reports": reports,
    }
    rendered = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
