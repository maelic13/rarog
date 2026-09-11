#!/usr/bin/env python3
"""Profile board work on frozen full-search cohorts (PLAN 4.11b.7).

The diagnostic build supplies exact recursive-search call/work counters.  Its
existing lifecycle resets after the one top-level legal-generation call; that
known setup call is recorded separately instead of being silently omitted.
A second, ordinary
release binary can be supplied with ``--compare-exe`` to prove that the
instrumentation does not change fixed-node search identity.  The JSON output
retains every per-root dump; stdout is only a compact derived summary.

Examples:
  python tools/diag/board_search_profile.py --exe target/release/rarog.exe
  python tools/diag/board_search_profile.py --exe tools/results/rarog-diag.exe \
      --compare-exe tools/results/rarog-production.exe --nodes 600000 \
      --repeats 3 --output tools/results/board-profile.json
"""

from __future__ import annotations

import argparse
import collections
import dataclasses
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SUITE = Path(__file__).with_name("board_search_profile_v1.epd")
REQUIRED_COHORTS = {
    "opening",
    "middlegame",
    "check-heavy",
    "promotion",
    "sparse-endgame",
}
BOARD_COUNTERS = (
    "board_gen_vec_calls",
    "board_gen_vec_moves",
    "board_gen_full_calls",
    "board_gen_full_moves",
    "board_gen_capture_calls",
    "board_gen_capture_moves",
    "board_gen_staged_capture_calls",
    "board_gen_staged_capture_moves",
    "board_gen_staged_quiet_calls",
    "board_gen_staged_quiet_moves",
    "board_compute_pinned_calls",
    "board_check_info_calls",
    "board_gives_check_fast_calls",
    "board_gives_check_full_calls",
    "board_calculate_checkers_calls",
    "board_see_full_calls",
    "board_see_threshold_calls",
    "board_see_quiet_threshold_calls",
    "board_make_plain_calls",
    "board_make_with_check_calls",
    "board_unmake_calls",
    "board_make_null_calls",
    "board_unmake_null_calls",
    "board_history_pushes",
    "board_history_growths",
)
INFO_FIELD = re.compile(r"\b(depth|seldepth|nodes|nps|time)\s+(\d+)")
SCORE_FIELD = re.compile(r"\bscore\s+(cp|mate)\s+(-?\d+)")


@dataclasses.dataclass(frozen=True)
class ProfileCase:
    name: str
    cohort: str
    fen: str
    source: str


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def load_suite(path: Path) -> list[ProfileCase]:
    cases = []
    for line_number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not raw or raw.startswith("#"):
            continue
        fields = [field.strip() for field in raw.split(";")]
        if len(fields) != 4:
            raise ValueError(f"{path}:{line_number}: expected FEN plus three fields")
        fen = fields[0]
        values = {}
        for field in fields[1:]:
            key, separator, value = field.partition(" ")
            if not separator or not value:
                raise ValueError(f"{path}:{line_number}: malformed field {field!r}")
            if key in values:
                raise ValueError(f"{path}:{line_number}: duplicate field {key}")
            values[key] = value
        if set(values) != {"cohort", "name", "src"}:
            raise ValueError(f"{path}:{line_number}: expected cohort, name and src")
        if len(fen.split()) != 6:
            raise ValueError(f"{path}:{line_number}: incomplete FEN")
        cases.append(ProfileCase(values["name"], values["cohort"], fen, values["src"]))
    if not cases:
        raise ValueError(f"{path}: empty profile")
    names = [case.name for case in cases]
    if len(set(names)) != len(names):
        raise ValueError(f"{path}: case names are not unique")
    cohorts = {case.cohort for case in cases}
    if cohorts != REQUIRED_COHORTS:
        raise ValueError(
            f"{path}: cohorts differ: missing={sorted(REQUIRED_COHORTS - cohorts)}, "
            f"extra={sorted(cohorts - REQUIRED_COHORTS)}"
        )
    counts = collections.Counter(case.cohort for case in cases)
    thin = {cohort: count for cohort, count in counts.items() if count < 4}
    if thin:
        raise ValueError(f"{path}: every cohort needs at least four roots; found {thin}")
    return cases


def wait_for(proc: subprocess.Popen[str], prefix: str, context: str) -> list[str]:
    lines = []
    while True:
        line = proc.stdout.readline()
        if not line:
            raise RuntimeError(f"engine closed before {prefix!r} ({context})")
        line = line.rstrip("\r\n")
        lines.append(line)
        if line.startswith(prefix):
            return lines


def send(proc: subprocess.Popen[str], command: str) -> None:
    proc.stdin.write(command + "\n")
    proc.stdin.flush()


def parse_search(lines: list[str], case: ProfileCase, require_diag: bool) -> dict:
    counters = {}
    info = None
    bestmove = None
    for line in lines:
        if line.startswith("info string diag "):
            parts = line.split()
            if len(parts) != 5:
                raise RuntimeError(f"{case.name}: malformed diagnostic line: {line}")
            name, value = parts[3], int(parts[4])
            if name in counters:
                raise RuntimeError(f"{case.name}: duplicate diagnostic counter {name}")
            counters[name] = value
        elif line.startswith("info ") and " nodes " in f" {line} ":
            info = line
        elif line.startswith("bestmove "):
            bestmove = line.split()[1]
    if info is None or bestmove is None:
        raise RuntimeError(f"{case.name}: missing final info or bestmove")
    if require_diag:
        missing = sorted(set(BOARD_COUNTERS) - set(counters))
        if missing:
            raise RuntimeError(
                f"{case.name}: diagnostic build did not emit board counters: {missing}"
            )
    numeric = {name: int(value) for name, value in INFO_FIELD.findall(info)}
    score_match = SCORE_FIELD.search(info)
    if "nodes" not in numeric or "depth" not in numeric or score_match is None:
        raise RuntimeError(f"{case.name}: incomplete final info: {info}")
    return {
        "name": case.name,
        "cohort": case.cohort,
        "fen": case.fen,
        "source": case.source,
        "depth": numeric["depth"],
        "seldepth": numeric.get("seldepth"),
        "reported_nodes": numeric["nodes"],
        "reported_nps": numeric.get("nps"),
        "reported_time_ms": numeric.get("time"),
        "score_type": score_match.group(1),
        "score": int(score_match.group(2)),
        "bestmove": bestmove,
        "counters": counters,
    }


def run_engine(
    exe: Path,
    cases: list[ProfileCase],
    nodes: int,
    repeats: int,
    require_diag: bool,
) -> dict:
    exe = exe.resolve()
    if not exe.is_file():
        raise FileNotFoundError(exe)
    env = dict(os.environ)
    env["RAROG_DIAG_SAMPLE_STRIDE"] = "1"
    started = time.time()
    proc = subprocess.Popen(
        [str(exe)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        cwd=str(exe.parent),
        env=env,
    )
    searches = []
    try:
        send(proc, "uci")
        uci_lines = wait_for(proc, "uciok", "UCI handshake")
        options = {
            line[len("option name ") :].split(" type ", 1)[0]
            for line in uci_lines
            if line.startswith("option name ") and " type " in line
        }
        for required in ("Hash", "Threads"):
            if required not in options:
                raise RuntimeError(f"{exe}: required UCI option {required!r} is absent")
        send(proc, "setoption name Hash value 16")
        send(proc, "setoption name Threads value 1")
        for repeat in range(1, repeats + 1):
            for case in cases:
                send(proc, "ucinewgame")
                send(proc, "isready")
                wait_for(proc, "readyok", f"{case.name} preflight")
                send(proc, f"position fen {case.fen}")
                send(proc, f"go nodes {nodes}")
                lines = wait_for(proc, "bestmove ", f"{case.name} search")
                result = parse_search(lines, case, require_diag)
                result["repeat"] = repeat
                searches.append(result)
        send(proc, "quit")
    finally:
        try:
            return_code = proc.wait(timeout=30)
        except subprocess.TimeoutExpired:
            proc.kill()
            return_code = proc.wait()
    if return_code != 0:
        raise RuntimeError(f"{exe}: engine exited with status {return_code}")
    return {
        "exe": str(exe),
        "exe_sha256": sha256(exe),
        "elapsed_seconds": time.time() - started,
        "nodes_requested": nodes,
        "repeats": repeats,
        "diagnostics_required": require_diag,
        "searches": searches,
    }


def identity(result: dict) -> tuple:
    return (
        result["name"],
        result["repeat"],
        result["depth"],
        result["seldepth"],
        result["reported_nodes"],
        result["score_type"],
        result["score"],
        result["bestmove"],
    )


def compare_identity(primary: dict, comparison: dict) -> None:
    left = [identity(row) for row in primary["searches"]]
    right = [identity(row) for row in comparison["searches"]]
    if left != right:
        mismatches = []
        for index, pair in enumerate(zip(left, right)):
            if pair[0] != pair[1]:
                mismatches.append({"index": index, "primary": pair[0], "comparison": pair[1]})
        if len(left) != len(right):
            mismatches.append({"primary_count": len(left), "comparison_count": len(right)})
        raise RuntimeError(
            "instrumentation-off identity failed:\n" + json.dumps(mismatches[:8], indent=2)
        )


def aggregate(searches: list[dict]) -> dict[str, dict]:
    grouped = collections.defaultdict(list)
    for search in searches:
        grouped[search["cohort"]].append(search)
    output = {}
    for cohort, rows in sorted(grouped.items()):
        counters = collections.Counter()
        for row in rows:
            counters.update(row["counters"])
        output[cohort] = {
            "searches": len(rows),
            "root_legal_generation_calls_outside_diag": len(rows),
            "reported_nodes": sum(row["reported_nodes"] for row in rows),
            "reported_time_ms": sum(row["reported_time_ms"] or 0 for row in rows),
            "counters": dict(sorted(counters.items())),
        }
    return output


def ratio(numerator: int, denominator: int, scale: float = 1.0) -> str:
    return "n/a" if not denominator else f"{scale * numerator / denominator:.3f}"


def print_summary(aggregates: dict[str, dict], diagnostics_present: bool) -> None:
    if diagnostics_present:
        print("cohort               nodes      gen/kN  checks/kN    SEE/kN   makes/kN growths")
        print("-" * 86)
    else:
        print("cohort               nodes   engine NPS searches")
        print("-" * 55)
    for cohort, row in aggregates.items():
        counters = collections.Counter(row["counters"])
        nodes = row["reported_nodes"]
        if not diagnostics_present:
            elapsed = row["reported_time_ms"]
            nps = 0 if not elapsed else nodes * 1000 // elapsed
            print(f"{cohort:20s} {nodes:10d} {nps:12d} {row['searches']:8d}")
            continue
        generation = (
            counters["board_gen_vec_calls"]
            + counters["board_gen_full_calls"]
            + counters["board_gen_capture_calls"]
            + counters["board_gen_staged_capture_calls"]
            + counters["board_gen_staged_quiet_calls"]
        )
        checks = (
            counters["board_check_info_calls"]
            + counters["board_gives_check_fast_calls"]
            + counters["board_gives_check_full_calls"]
            + counters["board_calculate_checkers_calls"]
        )
        see = (
            counters["board_see_full_calls"]
            + counters["board_see_threshold_calls"]
            + counters["board_see_quiet_threshold_calls"]
        )
        makes = counters["board_make_plain_calls"] + counters["board_make_with_check_calls"]
        print(
            f"{cohort:20s} {nodes:10d} {ratio(generation, nodes, 1000):>11s}"
            f" {ratio(checks, nodes, 1000):>10s} {ratio(see, nodes, 1000):>9s}"
            f" {ratio(makes, nodes, 1000):>10s} {counters['board_history_growths']:7d}"
        )
    if diagnostics_present:
        print("\nDetailed per-counter totals are retained in JSON; all rates use reported nodes.")
        print("One top-level legal-generation call/search is outside the diag reset boundary and")
        print("is recorded separately; ETW includes its time.")
    else:
        print("\nDiagnostic counters intentionally not requested; ETW supplies timing attribution.")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--compare-exe", type=Path)
    parser.add_argument("--suite", type=Path, default=DEFAULT_SUITE)
    parser.add_argument("--nodes", type=int, default=600_000)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--cohort", choices=sorted(REQUIRED_COHORTS))
    parser.add_argument("--allow-no-diag", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.nodes < 1 or args.repeats < 1:
        parser.error("--nodes and --repeats must be positive")
    suite = args.suite.resolve()
    cases = load_suite(suite)
    if args.cohort:
        cases = [case for case in cases if case.cohort == args.cohort]

    primary = run_engine(args.exe, cases, args.nodes, args.repeats, not args.allow_no_diag)
    comparison = None
    if args.compare_exe:
        comparison = run_engine(args.compare_exe, cases, args.nodes, args.repeats, False)
        compare_identity(primary, comparison)

    aggregates = aggregate(primary["searches"])
    document = {
        "schema": "rarog-board-search-profile-v1",
        "suite": str(suite),
        "suite_sha256": sha256(suite),
        "cohort_filter": args.cohort,
        "primary": primary,
        "comparison": comparison,
        "instrumentation_off_identity": comparison is not None,
        "aggregates": aggregates,
    }
    diagnostics_present = any(
        row["counters"] for row in primary["searches"]
    )
    print_summary(aggregates, diagnostics_present)
    if comparison:
        print(f"instrumentation-off identity: PASS ({len(primary['searches'])} searches)")
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {args.output}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (FileNotFoundError, RuntimeError, ValueError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        sys.exit(1)
