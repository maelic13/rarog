#!/usr/bin/env python3
"""
extract.py  —  PGN → FEN;result dataset for Rarog Texel tuning (Phase 3.4)

Usage:
    python tools/texel/extract.py <selfplay.pgn> [options]

Options:
    --out-dir DIR       Output directory (default: same directory as PGN)
    --train  FILENAME   Training set filename  (default: train.csv)
    --holdout FILENAME  Holdout set filename   (default: holdout.csv)
    --holdout-pct N     Percent of games → holdout (default: 5)
    --max-per-game N    Max qualifying plies sampled per game (default: 12)
    --skip-start N      Plies to skip at game start  (default: 16, = 8 full moves)
    --skip-end   N      Plies to skip at game end    (default: 6)
    --seed N            Random seed (default: 42)
    --min-train N       Warn if fewer than N training positions (default: 1500000)
    --balance-phase R   If >0, downsample over-represented phase buckets in TRAIN so
                        none exceeds R x the smallest (e.g. 2.0). Lossy; off by default.
                        Always prints the train/holdout phase mix regardless.

Output format (FEN;result):
    rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1;0.5

    result is from White's perspective: 1.0 win, 0.5 draw, 0.0 loss.

Requires:
    pip install chess
"""

import argparse
import os
import random
import sys

try:
    import chess
    import chess.pgn
except ImportError:
    print("ERROR: python-chess not installed. Run: pip install chess", file=sys.stderr)
    sys.exit(1)


RESULT_MAP = {
    "1-0":     1.0,
    "0-1":     0.0,
    "1/2-1/2": 0.5,
}

# Game-phase weights, matching the engine's PHASE_W (knight/bishop 1, rook 2,
# queen 4; max 24 = full non-pawn material). Buckets match the tuner's
# bucket_of(): opening >= 16, middlegame 6..16, endgame < 6.
PHASE_W = {chess.KNIGHT: 1, chess.BISHOP: 1, chess.ROOK: 2, chess.QUEEN: 4}


def game_phase(board: "chess.Board") -> int:
    return sum(
        PHASE_W[pt] * len(board.pieces(pt, c))
        for pt in PHASE_W
        for c in (chess.WHITE, chess.BLACK)
    )


def phase_bucket(phase: int) -> int:
    return 0 if phase >= 16 else (1 if phase >= 6 else 2)


PHASE_NAMES = ("opening", "middlegame", "endgame")


def fen_key(fen: str) -> str:
    """Return the first 4 FEN fields as a deduplication key (position, side, castling, ep)."""
    return " ".join(fen.split()[:4])


def process_game(game, skip_start: int, skip_end: int,
                 max_per_game: int, rng: random.Random):
    """
    Extract qualifying (fen, result_float) pairs from a single game.

    Returns a list of (fen_string, float) or [] if the game should be skipped.
    """
    result_str = game.headers.get("Result", "*")
    if result_str not in RESULT_MAP:
        return []
    label = RESULT_MAP[result_str]

    board = game.board()
    moves = list(game.mainline_moves())
    n = len(moves)

    candidates = []
    for ply_idx, move in enumerate(moves):
        # Skip first skip_start plies
        if ply_idx < skip_start:
            board.push(move)
            continue
        # Skip last skip_end plies
        if ply_idx >= n - skip_end:
            board.push(move)
            continue
        # Skip positions in check
        if board.is_check():
            board.push(move)
            continue
        # Skip if played move is a capture or promotion (cheapness filter)
        if board.is_capture(move) or move.promotion is not None:
            board.push(move)
            continue

        candidates.append((board.fen(), phase_bucket(game_phase(board))))
        board.push(move)

    # Sample at most max_per_game positions (decorrelation)
    if len(candidates) > max_per_game:
        candidates = rng.sample(candidates, max_per_game)

    return [(fen, label, bucket) for fen, bucket in candidates]


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("pgn", help="Input PGN file path")
    parser.add_argument("--out-dir",      default="",          metavar="DIR")
    parser.add_argument("--train",        default="train.csv",    metavar="FILENAME")
    parser.add_argument("--holdout",      default="holdout.csv",  metavar="FILENAME")
    parser.add_argument("--holdout-pct",  default=5,   type=int, metavar="N")
    parser.add_argument("--max-per-game", default=12,  type=int, metavar="N")
    parser.add_argument("--skip-start",   default=16,  type=int, metavar="N",
                        help="Plies to skip at game start (default 16 = 8 full moves)")
    parser.add_argument("--skip-end",     default=6,   type=int, metavar="N",
                        help="Plies to skip at game end (default 6)")
    parser.add_argument("--seed",         default=42,  type=int, metavar="N")
    parser.add_argument("--min-train",    default=1_500_000, type=int, metavar="N")
    parser.add_argument("--balance-phase", default=0.0, type=float, metavar="R",
                        help="If >0, downsample over-represented phase buckets in TRAIN so "
                             "none exceeds R x the smallest bucket (e.g. 2.0). Lossy; off by "
                             "default. Holdout is never rebalanced.")
    args = parser.parse_args()

    if not os.path.isfile(args.pgn):
        print(f"ERROR: PGN file not found: {args.pgn}", file=sys.stderr)
        sys.exit(1)

    out_dir = args.out_dir if args.out_dir else os.path.dirname(os.path.abspath(args.pgn))
    os.makedirs(out_dir, exist_ok=True)

    train_path   = os.path.join(out_dir, args.train)
    holdout_path = os.path.join(out_dir, args.holdout)

    rng = random.Random(args.seed)
    holdout_threshold = args.holdout_pct / 100.0

    seen: set[str] = set()
    train_positions   = []
    holdout_positions = []

    games_total    = 0
    games_skipped  = 0
    raw_candidates = 0

    print(f"Reading PGN: {args.pgn}")
    print(f"  skip_start={args.skip_start} plies, skip_end={args.skip_end} plies, "
          f"max_per_game={args.max_per_game}, holdout={args.holdout_pct}%")

    with open(args.pgn, encoding="utf-8", errors="replace") as pgn_file:
        while True:
            try:
                game = chess.pgn.read_game(pgn_file)
            except Exception as exc:
                print(f"  WARNING: parse error, skipping game: {exc}", file=sys.stderr)
                games_skipped += 1
                continue

            if game is None:
                break

            games_total += 1
            if games_total % 10_000 == 0:
                print(f"  {games_total:,} games processed, "
                      f"train={len(train_positions):,}, holdout={len(holdout_positions):,}, "
                      f"unique positions so far={len(seen):,}")

            pairs = process_game(game, args.skip_start, args.skip_end,
                                 args.max_per_game, rng)
            if not pairs:
                games_skipped += 1
                continue

            raw_candidates += len(pairs)

            # Split by game (not by position) to avoid train/holdout leakage
            is_holdout = rng.random() < holdout_threshold
            target = holdout_positions if is_holdout else train_positions

            for fen, label, bucket in pairs:
                key = fen_key(fen)
                if key in seen:
                    continue
                seen.add(key)
                target.append((fen, label, bucket))

    def phase_counts(positions):
        c = [0, 0, 0]
        for _, _, b in positions:
            c[b] += 1
        return c

    def fmt_phase(positions):
        c = phase_counts(positions)
        tot = max(sum(c), 1)
        return ", ".join(f"{PHASE_NAMES[i]} {c[i]:,} ({100*c[i]/tot:.1f}%)" for i in range(3))

    # Optional phase rebalancing of TRAIN (holdout left untouched so it stays a
    # faithful sample of the played distribution).
    if args.balance_phase > 0:
        counts = phase_counts(train_positions)
        present = [n for n in counts if n > 0]
        if present:
            cap = int(args.balance_phase * min(present))
            by_bucket = ([], [], [])
            for item in train_positions:
                by_bucket[item[2]].append(item)
            balanced = []
            for b in range(3):
                bucket_items = by_bucket[b]
                if len(bucket_items) > cap:
                    bucket_items = rng.sample(bucket_items, cap)
                balanced.extend(bucket_items)
            rng.shuffle(balanced)
            print(f"\nPhase balance (cap = {args.balance_phase} x smallest = {cap:,}):")
            print(f"  before: {fmt_phase(train_positions)}")
            train_positions = balanced
            print(f"  after : {fmt_phase(train_positions)}")

    print(f"\nSummary:")
    print(f"  Games read       : {games_total:,}")
    print(f"  Games skipped    : {games_skipped:,}")
    print(f"  Raw candidates   : {raw_candidates:,}")
    print(f"  Unique positions : {len(seen):,}")
    print(f"  Train positions  : {len(train_positions):,}")
    print(f"  Holdout positions: {len(holdout_positions):,}")
    print(f"  Train phase mix  : {fmt_phase(train_positions)}")
    print(f"  Holdout phase mix: {fmt_phase(holdout_positions)}")

    # Write train
    print(f"\nWriting {train_path} ...")
    with open(train_path, "w", encoding="utf-8") as f:
        for fen, label, _ in train_positions:
            # Format label: 1.0 → "1", 0.5 → "0.5", 0.0 → "0"
            if label == 1.0:
                s = "1"
            elif label == 0.0:
                s = "0"
            else:
                s = "0.5"
            f.write(f"{fen};{s}\n")

    # Write holdout
    print(f"Writing {holdout_path} ...")
    with open(holdout_path, "w", encoding="utf-8") as f:
        for fen, label, _ in holdout_positions:
            if label == 1.0:
                s = "1"
            elif label == 0.0:
                s = "0"
            else:
                s = "0.5"
            f.write(f"{fen};{s}\n")

    print(f"\nDone.")
    if len(train_positions) < args.min_train:
        print(f"\nWARNING: only {len(train_positions):,} training positions "
              f"(target >= {args.min_train:,}).")
        print("  Generate more games with datagen.ps1 (try more -Rounds or different -Nodes).")
        sys.exit(2)
    else:
        print(f"Target met: {len(train_positions):,} >= {args.min_train:,} training positions.")


if __name__ == "__main__":
    main()
