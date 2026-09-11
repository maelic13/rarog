#!/usr/bin/env python3
"""Measure Texel row yield per game CONDITIONED ON THE START POSITION'S PHASE.

The extractor preflight sizes GAMES: it reports rows/game per phase bucket over
whatever mix of starts the book happened to contain, and recommends a game
count. That is the right answer to the wrong question when the book is the
constraint. `extract.py`'s phase is MATERIAL, not ply, so a game started below
phase 20 can never produce an `opening` row -- and no schedule fixes a book
whose starts cannot reach the bucket that binds.

This tool splits a datagen PGN by the phase bucket of each game's own start
position and runs the extractor's preflight on each split separately. The
result is a matrix whose rows are start buckets and whose columns are phase
buckets, which is what you need to choose a book composition. See
`analysis/texel_corpus_book_shape_2026-09-02.md` for the finding it produced.

The rates come from `extract.py --preflight-games`, not from a reimplementation
here, so the quiet filter, the per-phase cap and the global cap are the
extractor's own. Pass through `--max-per-game` and `--skip-start` to measure
what those levers cost.

Usage:

  python tools/diag/book_yield.py <datagen.pgn> [--preflight-games 4000]
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools" / "texel"))

try:
    import chess  # noqa: F401  (extract.py needs it; fail early with its message)
except ImportError:
    print("ERROR: python-chess not installed. Run: pip install chess", file=sys.stderr)
    sys.exit(1)

from extract import PHASE_BUCKETS, PHASE_W  # noqa: E402

BUCKET_NAMES = [name for name, _, _ in PHASE_BUCKETS]

_W: dict[str, int] = {}
for _pt, _w in PHASE_W.items():
    _sym = chess.piece_symbol(_pt)
    _W[_sym] = _w
    _W[_sym.upper()] = _w

_BUCKET_OF = [""] * 25
for _name, _lo, _hi in PHASE_BUCKETS:
    for _p in range(_lo, _hi + 1):
        _BUCKET_OF[_p] = _name

FEN_TAG = re.compile(r'^\[FEN "([^"]+)"')
RATE = re.compile(r"^\s*train\s*/(\w+)\s+rate=\s*([0-9.]+)/game")


def bucket_of_board(board_field: str) -> str:
    return _BUCKET_OF[min(24, sum(_W.get(ch, 0) for ch in board_field))]


def split_by_start(pgn: Path, out_dir: Path) -> dict[str, int]:
    """Write one PGN per start bucket. Returns game counts."""
    handles = {n: (out_dir / f"start_{n}.pgn").open("w", encoding="utf-8", newline="\n")
               for n in BUCKET_NAMES}
    counts = {n: 0 for n in BUCKET_NAMES}
    current: str | None = None
    buf: list[str] = []
    with pgn.open(encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if line.startswith("[Event ") and buf:
                if current:
                    handles[current].writelines(buf)
                    counts[current] += 1
                buf = []
                current = None
            buf.append(line)
            m = FEN_TAG.match(line)
            if m:
                current = bucket_of_board(m.group(1).split(" ", 1)[0])
    if buf and current:
        handles[current].writelines(buf)
        counts[current] += 1
    for fh in handles.values():
        fh.close()
    return counts


def preflight(path: Path, games: int, max_per_game: int, skip_start: int) -> dict[str, float]:
    cmd = [sys.executable, str(REPO / "tools" / "texel" / "extract.py"), str(path),
           "--preflight-games", str(games),
           "--max-per-game", str(max_per_game),
           "--skip-start", str(skip_start)]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    rates = {}
    for line in proc.stdout.splitlines():
        m = RATE.match(line)
        if m:
            rates[m.group(1)] = float(m.group(2))
    # A nonzero exit is EXPECTED here and is the finding, not a failure: the
    # preflight refuses to recommend a game count when some phase had zero
    # yield, which is exactly what every non-opening start bucket does to the
    # opening column. Only a run that produced no rates at all is an error.
    if not rates:
        raise SystemExit(
            f"extract.py preflight produced no rates for {path} "
            f"(exit {proc.returncode}):\n{proc.stdout}\n{proc.stderr}"
        )
    return rates


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("pgn", type=Path)
    ap.add_argument("--preflight-games", type=int, default=4000,
                    help="games per split handed to the extractor preflight")
    ap.add_argument("--max-per-game", type=int, default=16)
    ap.add_argument("--skip-start", type=int, default=2)
    ap.add_argument("--keep-splits", type=Path, default=None,
                    help="write the per-bucket PGNs here instead of a temp dir")
    args = ap.parse_args()

    if not args.pgn.is_file():
        raise SystemExit(f"no such PGN: {args.pgn}")

    tmp = Path(tempfile.mkdtemp(prefix="book_yield_")) if args.keep_splits is None \
        else args.keep_splits
    tmp.mkdir(parents=True, exist_ok=True)
    try:
        counts = split_by_start(args.pgn, tmp)
        total = sum(counts.values())
        if total == 0:
            raise SystemExit(f"{args.pgn}: no games with a [FEN] tag")
        print(f"{args.pgn}  ({total:,} games)")
        print(f"preflight: --max-per-game {args.max_per_game} "
              f"--skip-start {args.skip_start}\n")
        print("start distribution:")
        for n in BUCKET_NAMES:
            print(f"  {n:<14} {counts[n]:>8,}  {100.0 * counts[n] / total:5.1f}%")
        print()

        head = "  ".join(f"{n[:9]:>9}" for n in BUCKET_NAMES)
        print(f"train rows/game    {head}      total")
        print("-" * (19 + len(head) + 11))
        matrix = {}
        for start in BUCKET_NAMES:
            path = tmp / f"start_{start}.pgn"
            if counts[start] == 0:
                print(f"{start:<18} (no games)")
                continue
            rates = preflight(path, min(args.preflight_games, counts[start]),
                              args.max_per_game, args.skip_start)
            matrix[start] = rates
            row = "  ".join(f"{rates.get(n, 0.0):9.4f}" for n in BUCKET_NAMES)
            print(f"{start:<18} {row}  {sum(rates.values()):9.2f}")
        print()
        print("Only a start bucket with a nonzero `opening` entry can feed the")
        print("opening bucket. Choose the book composition from this matrix; see")
        print("analysis/texel_corpus_book_shape_2026-09-02.md.")
    finally:
        if args.keep_splits is None:
            shutil.rmtree(tmp, ignore_errors=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
