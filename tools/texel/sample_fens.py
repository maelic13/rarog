#!/usr/bin/env python3
"""
Sample FEN positions into a fastchess EPD opening book.

This is intended for the post-SF-label path: use Beast/database FENs only as
diverse start positions, then let Rarog self-play produce the labels.

Accepted input line shapes:
  FEN
  FEN<TAB>target
  FEN;target

Output line shape:
  <piece-placement> <side> <castling> <ep>

Example:
  python tools/texel/sample_fens.py A:\\Chess\\Beast\\data\\txt\\positions.txt \
    --out tools\\texel\\data\\beast_seed.epd --count 100000
"""

from __future__ import annotations

import argparse
import glob
import os
import random
import sys
from typing import Iterable


def iter_files(sources: list[str]) -> Iterable[str]:
    for source in sources:
        if os.path.isdir(source):
            patterns = [
                os.path.join(source, "evaluated_positions_*.txt"),
                os.path.join(source, "positions*.txt"),
                os.path.join(source, "*.csv"),
            ]
            seen: set[str] = set()
            files: list[str] = []
            for pattern in patterns:
                for path in sorted(glob.glob(pattern)):
                    if path not in seen:
                        seen.add(path)
                        files.append(path)
            if not files:
                raise SystemExit(f"No supported text/csv files found under {source}")
            yield from files
        else:
            if not os.path.isfile(source):
                raise SystemExit(f"Not found: {source}")
            yield source


def parse_line(line: str, target_min: float | None, target_max: float | None) -> str | None:
    line = line.strip()
    if not line:
        return None

    target = None
    if "\t" in line:
        fen, target = line.rsplit("\t", 1)
    elif ";" in line:
        fen, target = line.rsplit(";", 1)
    else:
        fen = line

    if target_min is not None or target_max is not None:
        if target is None:
            return None
        try:
            value = float(target)
        except ValueError:
            return None
        if target_min is not None and value < target_min:
            return None
        if target_max is not None and value > target_max:
            return None

    fields = fen.split()
    if len(fields) < 4:
        return None
    return " ".join(fields[:4])


def quick_piece_count(epd: str) -> int:
    placement = epd.split()[0]
    return sum(1 for ch in placement if ch.isalpha())


def validate_epds(epds: list[str], args) -> list[str]:
    if args.no_validate:
        return epds[: args.count]

    try:
        import chess
    except ImportError:
        print("WARNING: python-chess not installed; writing unvalidated EPD sample.", file=sys.stderr)
        return epds[: args.count]

    out: list[str] = []
    seen: set[str] = set()
    for epd in epds:
        if epd in seen:
            continue
        seen.add(epd)

        pieces = quick_piece_count(epd)
        if pieces < args.min_pieces or pieces > args.max_pieces:
            continue

        try:
            board = chess.Board(epd + " 0 1")
        except ValueError:
            continue

        if not board.is_valid() or board.is_game_over(claim_draw=False):
            continue
        if not args.allow_check and board.is_check():
            continue
        if args.quiet:
            tactical = False
            for move in board.legal_moves:
                if board.is_capture(move) or move.promotion is not None:
                    tactical = True
                    break
            if tactical:
                continue

        out.append(epd)
        if len(out) >= args.count:
            break

    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("sources", nargs="+", help="FEN/txt/csv source files or directories")
    parser.add_argument("--out", default="tools/texel/data/beast_seed.epd")
    parser.add_argument("--count", type=int, default=100_000)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--max-read", type=int, default=0,
                        help="stop after reading this many lines; 0 = no cap")
    parser.add_argument("--target-min", type=float, default=None,
                        help="optional filter when input has target column")
    parser.add_argument("--target-max", type=float, default=None,
                        help="optional filter when input has target column")
    parser.add_argument("--oversample", type=int, default=3,
                        help="sample this many times --count before validation")
    parser.add_argument("--min-pieces", type=int, default=6)
    parser.add_argument("--max-pieces", type=int, default=32)
    parser.add_argument("--allow-check", action="store_true")
    parser.add_argument("--quiet", action="store_true",
                        help="keep only final sampled positions with no legal capture/promotion")
    parser.add_argument("--no-validate", action="store_true",
                        help="skip python-chess validation of the final sample")
    parser.add_argument("--progress-every", type=int, default=5_000_000)
    args = parser.parse_args()

    if args.count <= 0:
        raise SystemExit("--count must be positive")
    if args.oversample <= 0:
        raise SystemExit("--oversample must be positive")
    if args.target_min is not None and args.target_max is not None and args.target_min > args.target_max:
        raise SystemExit("--target-min cannot exceed --target-max")

    rng = random.Random(args.seed)
    reservoir_size = args.count if args.no_validate else args.count * args.oversample
    reservoir: list[str] = []

    read = candidates = 0
    files = list(iter_files(args.sources))
    print(f"Sampling from {len(files)} file(s)")
    print(f"Target output: {args.count:,} EPD positions -> {args.out}")

    for file_index, path in enumerate(files, start=1):
        print(f"[{file_index}/{len(files)}] {path}")
        with open(path, "r", encoding="utf-8", errors="replace") as source:
            for line in source:
                read += 1
                epd = parse_line(line, args.target_min, args.target_max)
                if epd is None:
                    if args.max_read > 0 and read >= args.max_read:
                        break
                    continue

                candidates += 1
                if len(reservoir) < reservoir_size:
                    reservoir.append(epd)
                else:
                    j = rng.randrange(candidates)
                    if j < reservoir_size:
                        reservoir[j] = epd

                if args.progress_every > 0 and read % args.progress_every == 0:
                    print(f"  read={read:,} candidates={candidates:,}")
                if args.max_read > 0 and read >= args.max_read:
                    break
        if args.max_read > 0 and read >= args.max_read:
            break

    rng.shuffle(reservoir)
    selected = validate_epds(reservoir, args)

    out_dir = os.path.dirname(os.path.abspath(args.out))
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)
    with open(args.out, "w", encoding="utf-8", newline="\n") as out:
        for epd in selected:
            out.write(epd + "\n")

    print()
    print("Summary:")
    print(f"  Lines read : {read:,}")
    print(f"  Candidates : {candidates:,}")
    print(f"  Reservoir : {len(reservoir):,}")
    print(f"  Written   : {len(selected):,}")
    if len(selected) < args.count:
        print(f"WARNING: requested {args.count:,}, wrote only {len(selected):,}.")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
