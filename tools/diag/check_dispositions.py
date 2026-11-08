"""Fail when a ledger entry and its index row disagree on the disposition.

An entry, `docs/experiments/<ID>.md`, states its disposition on one line,

  - Disposition: <word>[ then `,` `.` `;` `:` or ` — ` and any explanation]

and its row in `docs/EXPERIMENTS.md` starts the Disposition cell with the same
word. The words are DISPOSITIONS below; `open` is a registered experiment that
has not resolved. Entries registered before 2026-11-08 wrote the disposition
as prose in either place, so they are listed in `disposition_exempt.txt`,
which only shrinks: an exempt entry that carries the form on both sides fails
until it is removed from the list. Every other entry needs both, and they
must match.

  python tools/diag/check_dispositions.py         # exit 0 clean, 1 on a problem
"""
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
INDEX = ROOT / "docs" / "EXPERIMENTS.md"
ENTRIES = ROOT / "docs" / "experiments"
EXEMPT = pathlib.Path(__file__).with_name("disposition_exempt.txt")

DISPOSITIONS = (
    "accepted", "retained", "rejected", "refuted", "inconclusive",
    "observation", "no-change", "deferred", "withdrawn", "open",
)
WORD = "|".join(re.escape(word) for word in DISPOSITIONS)
ENTRY_LINE = re.compile(rf"^- Disposition: ({WORD})(?=\s*$|[.,;:]|\s+—)", re.MULTILINE)
INDEX_ROW = re.compile(
    r"^\| \[(RAR-[A-Z]+\d+)\]\(experiments/\1\.md\) \|[^|\n]*\|([^|\n]*)\|\s*$", re.MULTILINE)
INDEX_CELL = re.compile(rf"^({WORD})(?=\s*$|[.,;:]|\s+—)")


def entry_disposition(text: str) -> str | None | list[str]:
    """The entry's disposition word; None without the line; every word when
    the line appears more than once, which is itself a problem."""
    words = ENTRY_LINE.findall(text)
    if not words:
        return None
    return words[0] if len(words) == 1 else words


def index_dispositions(text: str) -> dict[str, list[str | None]]:
    """For each linked ID, the disposition word each of its rows starts with
    (None when a row's cell starts with none)."""
    rows: dict[str, list[str | None]] = {}
    for match in INDEX_ROW.finditer(text):
        cell = INDEX_CELL.match(match.group(2).strip())
        rows.setdefault(match.group(1), []).append(cell.group(1) if cell else None)
    return rows


def load_exempt(text: str) -> set[str]:
    """One ID per line; `#` starts a comment."""
    exempt: set[str] = set()
    for number, line in enumerate(text.splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if not re.fullmatch(r"RAR-[A-Z]+\d+", line):
            raise ValueError(f"{EXEMPT.name}:{number}: expected one ledger ID, got {line!r}")
        if line in exempt:
            raise ValueError(f"{EXEMPT.name}:{number}: {line} listed twice")
        exempt.add(line)
    return exempt


def problems(entries: dict[str, str], index: str, exempt: set[str]) -> list[str]:
    """`entries` maps each ID to its file's text."""
    found = []
    rows = index_dispositions(index)
    for name in sorted(exempt - entries.keys()):
        found.append(f"{name}: exempt but has no entry file")
    for name, text in sorted(entries.items()):
        entry = entry_disposition(text)
        row = rows.get(name, [])
        if isinstance(entry, list):
            found.append(f"{name}: the entry states a disposition {len(entry)} times")
            continue
        if len(row) > 1:
            found.append(f"{name}: {len(row)} index rows")
            continue
        cell = row[0] if row else None
        if name in exempt:
            if entry is not None and cell is not None:
                found.append(f"{name}: carries the form on both sides; remove it from {EXEMPT.name}")
            continue
        if not row:
            found.append(f"{name}: no index row")
        elif cell is None:
            found.append(f"{name}: the index row's disposition starts with none of {', '.join(DISPOSITIONS)}")
        if entry is None:
            found.append(f"{name}: no `- Disposition: <word>` line")
        if entry is not None and cell is not None and entry != cell:
            found.append(f"{name}: the entry says {entry}, the index row {cell}")
    return found


def main() -> int:
    try:
        exempt = load_exempt(EXEMPT.read_text(encoding="utf-8"))
    except ValueError as error:
        print(error)
        return 1
    entries = {path.stem: path.read_text(encoding="utf-8") for path in ENTRIES.glob("RAR-*.md")}
    found = problems(entries, INDEX.read_text(encoding="utf-8"), exempt)
    for line in found:
        print(line)
    print(f"{len(entries)} entries, {len(exempt)} exempt, {len(found)} problems")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
