#!/usr/bin/env python3
"""
Import Beast/Stockfish-evaluated positions into Rarog Texel CSV format.

Input lines are expected as:
    FEN<TAB>target

Output lines are:
    FEN;target

By default the input target is treated as side-to-move expected score, matching
Stockfish WDL output. It is converted to white-perspective expected score,
which is what rarog-texel trains against. Files are streamed, so the large
Beast dataset does not need to be copied into this repo.
"""

import argparse
import glob
import os
import random
import sys


def iter_input_files(path: str):
    if os.path.isdir(path):
        files = sorted(glob.glob(os.path.join(path, "evaluated_positions_*.txt")))
    else:
        files = [path]
    if not files:
        raise SystemExit(f"No evaluated_positions_*.txt files found under {path}")
    return files


def parse_line(line: str, target_perspective: str):
    line = line.strip()
    if not line:
        return None
    if "\t" not in line:
        return None
    fen, target = line.rsplit("\t", 1)
    try:
        value = float(target)
    except ValueError:
        return None
    if value < 0.0 or value > 1.0:
        return None
    # Basic FEN sanity: keep full 6-field FENs only.
    fields = fen.split()
    if len(fields) != 6:
        return None
    if target_perspective == "stm" and fields[1] == "b":
        value = 1.0 - value
    return fen, f"{value:.6g}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", help="Beast evaluated directory or one evaluated file")
    parser.add_argument("--out-dir", default="tools/texel/data")
    parser.add_argument("--train", default="beast_sf_train.csv")
    parser.add_argument("--holdout", default="beast_sf_holdout.csv")
    parser.add_argument("--max-positions", type=int, default=1_600_000)
    parser.add_argument("--holdout-pct", type=float, default=5.0)
    parser.add_argument("--target-perspective", choices=("stm", "white"), default="stm",
                        help="input target perspective: Stockfish WDL is side-to-move")
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    os.makedirs(args.out_dir, exist_ok=True)
    train_path = os.path.join(args.out_dir, args.train)
    holdout_path = os.path.join(args.out_dir, args.holdout)
    rng = random.Random(args.seed)

    files = iter_input_files(args.source)
    read = kept = skipped = train = holdout = 0

    print(f"Importing from {len(files)} file(s)")
    print(f"Target positions: {args.max_positions:,}, holdout={args.holdout_pct:g}%")
    print(f"Input target perspective: {args.target_perspective}")
    print(f"Train  -> {train_path}")
    print(f"Holdout-> {holdout_path}")

    with open(train_path, "w", encoding="utf-8", newline="\n") as train_out, \
         open(holdout_path, "w", encoding="utf-8", newline="\n") as holdout_out:
        for file_index, path in enumerate(files, start=1):
            print(f"[{file_index}/{len(files)}] {path}")
            with open(path, "r", encoding="utf-8", errors="replace") as source:
                for line in source:
                    read += 1
                    parsed = parse_line(line, args.target_perspective)
                    if parsed is None:
                        skipped += 1
                        continue

                    fen, target = parsed
                    out = holdout_out if rng.random() * 100.0 < args.holdout_pct else train_out
                    out.write(f"{fen};{target}\n")
                    kept += 1
                    if out is holdout_out:
                        holdout += 1
                    else:
                        train += 1

                    if kept % 100_000 == 0:
                        print(f"  kept={kept:,} train={train:,} holdout={holdout:,}")
                    if kept >= args.max_positions:
                        break
            if kept >= args.max_positions:
                break

    print()
    print("Summary:")
    print(f"  Lines read : {read:,}")
    print(f"  Kept       : {kept:,}")
    print(f"  Train      : {train:,}")
    print(f"  Holdout    : {holdout:,}")
    print(f"  Skipped    : {skipped:,}")
    return 0 if kept > 0 else 1


if __name__ == "__main__":
    sys.exit(main())
