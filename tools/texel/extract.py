#!/usr/bin/env python3
"""PGN -> phase-balanced FEN;target data for Rarog Texel tuning.

The default contract is an exact 3,000,000-position training set, split evenly
across five material-phase buckets.  Positions are sampled per phase *inside
each game* before entering fixed-size reservoirs.  This avoids the old failure
mode where a uniform 12-ply/game cap discarded nearly all opening positions and
the phase mix was only discovered after a full extraction.

Examples:
    # Cheap sizing pass; reads only the first 20k games.
    python tools/texel/extract.py tools/texel/data/*.pgn --preflight-games 20000

    # One extraction over any number of archives; produces exactly 3M train
    # rows when every phase quota is available.
    python tools/texel/extract.py tools/texel/data/*.pgn \
        --out-dir tools/texel/data --train train.csv --holdout holdout.csv

Output is FEN;target, with the target from White's perspective in [0,1].
Requires python-chess (``pip install chess``).
"""

from __future__ import annotations

import argparse
import glob
import math
import os
import random
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

try:
    import chess
    import chess.pgn
except ImportError:
    print("ERROR: python-chess not installed. Run: pip install chess", file=sys.stderr)
    sys.exit(1)


RESULT_MAP = {"1-0": 1.0, "0-1": 0.0, "1/2-1/2": 0.5}

# Matches src/eval.rs: N=B=1, R=2, Q=4, capped at 24.
PHASE_W = {chess.KNIGHT: 1, chess.BISHOP: 1, chess.ROOK: 2, chess.QUEEN: 4}
PHASE_BUCKETS = (
    ("opening", 20, 24),
    ("early_mid", 14, 19),
    ("middlegame", 8, 13),
    ("endgame", 3, 7),
    ("deep_endgame", 0, 2),
)

PIECE_VAL = {
    chess.PAWN: 1,
    chess.KNIGHT: 3,
    chess.BISHOP: 3,
    chess.ROOK: 5,
    chess.QUEEN: 9,
    chess.KING: 20,
}

# fastchess comment: "+0.25/12 0.013s" or mate "+M5/12 0.002s".
COMMENT_CP = re.compile(r"^([+-]?)(M?)(\d+(?:\.\d+)?)/")
CP_CLAMP = 2000


def game_phase(board: "chess.Board") -> int:
    return min(
        24,
        sum(
            PHASE_W[pt] * len(board.pieces(pt, color))
            for pt in PHASE_W
            for color in (chess.WHITE, chess.BLACK)
        ),
    )


def phase_bucket(phase: int) -> int:
    for index, (_, lo, hi) in enumerate(PHASE_BUCKETS):
        if lo <= phase <= hi:
            return index
    raise ValueError(f"phase outside 0..24: {phase}")


def fen_key(fen: str) -> str:
    """Position, side, castling and EP; clocks are irrelevant to eval."""
    return " ".join(fen.split()[:4])


def has_winning_capture(board: "chess.Board") -> bool:
    """Cheap SEE>0 proxy used by the existing Rarog pipeline."""
    for move in board.generate_legal_captures():
        victim = board.piece_type_at(move.to_square) or chess.PAWN  # EP
        attacker = board.piece_type_at(move.from_square)
        if PIECE_VAL[victim] > PIECE_VAL[attacker]:
            return True
        if not board.is_attacked_by(not board.turn, move.to_square):
            return True
    return False


def comment_cp_white(comment: str, white_to_move: bool) -> float | None:
    match = COMMENT_CP.match(comment.strip())
    if not match:
        return None
    sign = -1.0 if match.group(1) == "-" else 1.0
    cp = sign * (CP_CLAMP if match.group(2) else float(match.group(3)) * 100.0)
    cp = max(-CP_CLAMP, min(CP_CLAMP, cp))
    return cp if white_to_move else -cp


def sigmoid_cp(cp: float) -> float:
    return 1.0 / (1.0 + 10.0 ** (-cp / 400.0))


@dataclass
class Reservoir:
    """Uniform fixed-size sample from a stream of unknown length."""

    capacity: int
    rng: random.Random

    def __post_init__(self) -> None:
        self.seen = 0
        self.items: list[tuple[str, float, float | None]] = []

    def offer(self, item: tuple[str, float, float | None]) -> None:
        self.seen += 1
        if len(self.items) < self.capacity:
            self.items.append(item)
            return
        pick = self.rng.randrange(self.seen)
        if pick < self.capacity:
            self.items[pick] = item


def allocate(total: int, weights: list[float]) -> list[int]:
    """Largest-remainder allocation whose entries sum exactly to total."""
    if total < 0 or not weights or any(weight <= 0 for weight in weights):
        raise ValueError("total must be non-negative and phase weights positive")
    weight_sum = sum(weights)
    raw = [total * weight / weight_sum for weight in weights]
    out = [math.floor(value) for value in raw]
    for index in sorted(range(len(weights)), key=lambda i: raw[i] - out[i], reverse=True)[: total - sum(out)]:
        out[index] += 1
    return out


def parse_phase_weights(value: str) -> list[float]:
    try:
        weights = [float(part) for part in value.split(",")]
    except ValueError as exc:
        raise argparse.ArgumentTypeError("phase weights must be comma-separated numbers") from exc
    if len(weights) != len(PHASE_BUCKETS) or any(weight <= 0 for weight in weights):
        raise argparse.ArgumentTypeError("phase weights must contain five positive numbers")
    return weights


def iter_pgn_paths(inputs: list[str]) -> list[Path]:
    paths: list[Path] = []
    seen: set[Path] = set()
    for source in inputs:
        matches = sorted(glob.glob(source))
        if not matches and os.path.exists(source):
            matches = [source]
        for match in matches:
            path = Path(match).resolve()
            candidates = sorted(path.glob("*.pgn")) if path.is_dir() else [path]
            for candidate in candidates:
                if candidate not in seen:
                    seen.add(candidate)
                    paths.append(candidate)
    missing = [str(path) for path in paths if not path.is_file()]
    if not paths or missing:
        detail = f": {', '.join(missing)}" if missing else ""
        raise SystemExit(f"No readable PGN inputs found{detail}")
    return paths


def process_game(
    game: "chess.pgn.Game",
    skip_start: int,
    skip_end: int,
    max_per_phase_per_game: int,
    max_per_game: int,
    quiet_filter: bool,
    rng: random.Random,
) -> tuple[list[tuple[str, int, float | None]], int]:
    """Return phase-stratified candidates and the quiet-filter reject count."""
    if game.headers.get("Result", "*") not in RESULT_MAP:
        return [], 0

    board = game.board()
    nodes = list(game.mainline())
    by_phase: list[list[tuple[str, int, float | None]]] = [[] for _ in PHASE_BUCKETS]

    for ply_index, node in enumerate(nodes):
        move = node.move
        if (
            ply_index >= skip_start
            and ply_index < len(nodes) - skip_end
            and not board.is_check()
            and not board.is_capture(move)
            and move.promotion is None
        ):
            bucket = phase_bucket(game_phase(board))
            cp = comment_cp_white(node.comment or "", board.turn == chess.WHITE)
            by_phase[bucket].append((board.fen(), bucket, cp))
        board.push(move)

    # Bound expensive quiet checks per phase, rather than sampling uniformly
    # over the whole game and silently starving opening/deep-endgame rows.
    selected: list[tuple[str, int, float | None]] = []
    quiet_rejected = 0
    for candidates in by_phase:
        check_cap = max_per_phase_per_game * (2 if quiet_filter else 1)
        if len(candidates) > check_cap:
            candidates = rng.sample(candidates, check_cap)
        if quiet_filter:
            kept = []
            for item in candidates:
                if has_winning_capture(chess.Board(item[0])):
                    quiet_rejected += 1
                else:
                    kept.append(item)
            candidates = kept
        if len(candidates) > max_per_phase_per_game:
            candidates = rng.sample(candidates, max_per_phase_per_game)
        selected.extend(candidates)

    if max_per_game > 0 and len(selected) > max_per_game:
        # Compatibility/safety cap. The phase cap above is the primary control.
        selected = rng.sample(selected, max_per_game)
    return selected, quiet_rejected


def fmt_target(target: float) -> str:
    if target in (0.0, 0.5, 1.0):
        return f"{target:g}"
    return f"{target:.6f}".rstrip("0").rstrip(".")


def stage_rows(path: Path, rows: Iterable[tuple[str, float]]) -> Path:
    tmp = path.with_name(path.name + ".tmp")
    with tmp.open("w", encoding="utf-8", newline="\n") as out:
        for fen, target in rows:
            out.write(f"{fen};{fmt_target(target)}\n")
    return tmp


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("pgn", nargs="+", help="PGN files, globs, or directories")
    parser.add_argument("--out-dir", default="", metavar="DIR")
    parser.add_argument("--train", default="train.csv", metavar="FILE")
    parser.add_argument("--holdout", default="holdout.csv", metavar="FILE")
    parser.add_argument("--target-train", default=3_000_000, type=int, metavar="N")
    parser.add_argument("--phase-weights", default=parse_phase_weights("1,1,1,1,1"), type=parse_phase_weights,
                        metavar="A,B,C,D,E", help="target mix for the five phase buckets (default equal)")
    parser.add_argument("--holdout-pct", default=5.0, type=float, metavar="N")
    parser.add_argument("--max-per-phase-per-game", default=8, type=int, metavar="N")
    parser.add_argument("--max-per-game", default=0, type=int, metavar="N",
                        help="optional total cap after phase sampling; 0 = no extra cap")
    parser.add_argument("--skip-start", default=2, type=int, metavar="N",
                        help="plies after the supplied opening FEN to skip (default 2)")
    parser.add_argument("--skip-end", default=6, type=int, metavar="N")
    parser.add_argument("--seed", default=42, type=int, metavar="N")
    parser.add_argument("--preflight-games", default=0, type=int, metavar="N",
                        help="estimate required games from the first N games; write nothing")
    parser.add_argument("--preflight-safety", default=1.25, type=float, metavar="X")
    parser.add_argument("--no-quiet-filter", dest="quiet_filter", action="store_false")
    parser.add_argument("--blend", default=1.0, type=float, metavar="LAMBDA",
                        help="train target = lambda*WDL + (1-lambda)*sigmoid(search cp)")
    args = parser.parse_args()

    if args.target_train <= 0:
        parser.error("--target-train must be positive")
    if not 0.0 <= args.holdout_pct < 100.0:
        parser.error("--holdout-pct must be in [0,100)")
    if not 0.0 <= args.blend <= 1.0:
        parser.error("--blend must be in [0,1]")
    if args.max_per_phase_per_game <= 0:
        parser.error("--max-per-phase-per-game must be positive")
    if args.skip_start < 0 or args.skip_end < 0:
        parser.error("skip counts cannot be negative")

    paths = iter_pgn_paths(args.pgn)
    out_dir = Path(args.out_dir).resolve() if args.out_dir else paths[0].parent
    quotas = allocate(args.target_train, args.phase_weights)
    holdout_total = round(args.target_train * args.holdout_pct / max(100.0 - args.holdout_pct, 1.0))
    holdout_quotas = allocate(holdout_total, args.phase_weights)
    rng = random.Random(args.seed)
    train = [Reservoir(quota, rng) for quota in quotas]
    holdout = [Reservoir(quota, rng) for quota in holdout_quotas]

    print(f"PGN inputs: {len(paths)}")
    for path in paths:
        print(f"  {path}")
    print(f"Target train: {args.target_train:,} | holdout: {holdout_total:,}")
    print("Phase quotas: " + ", ".join(f"{PHASE_BUCKETS[i][0]}={quotas[i]:,}" for i in range(len(quotas))))
    print(f"skip_start={args.skip_start}, skip_end={args.skip_end}, "
          f"max/phase/game={args.max_per_phase_per_game}, "
          f"quiet_filter={'on' if args.quiet_filter else 'OFF'}, blend={args.blend}")

    seen: set[str] = set()
    games_total = games_skipped = raw_candidates = quiet_rejected = missing_evals = 0
    unique_by_phase = [0] * len(PHASE_BUCKETS)
    preflight_base, preflight_extra = divmod(args.preflight_games, len(paths))

    for path_index, path in enumerate(paths):
        path_limit = preflight_base + (1 if path_index < preflight_extra else 0)
        if args.preflight_games and path_limit == 0:
            continue
        path_games = 0
        print(f"Reading {path} ...")
        with path.open(encoding="utf-8", errors="replace") as pgn_file:
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
                path_games += 1
                result_str = game.headers.get("Result", "*")
                candidates, rejected = process_game(
                    game,
                    args.skip_start,
                    args.skip_end,
                    args.max_per_phase_per_game,
                    args.max_per_game,
                    args.quiet_filter,
                    rng,
                )
                quiet_rejected += rejected
                if not candidates:
                    games_skipped += 1
                    if args.preflight_games and path_games >= path_limit:
                        break
                    continue

                raw_candidates += len(candidates)
                result = RESULT_MAP[result_str]
                is_holdout = rng.random() * 100.0 < args.holdout_pct
                reservoirs = holdout if is_holdout else train

                for fen, bucket, cp in candidates:
                    key = fen_key(fen)
                    if key in seen:
                        continue
                    seen.add(key)
                    unique_by_phase[bucket] += 1
                    if not is_holdout and args.blend < 1.0 and cp is not None:
                        target = args.blend * result + (1.0 - args.blend) * sigmoid_cp(cp)
                    else:
                        target = result
                        if not is_holdout and args.blend < 1.0 and cp is None:
                            missing_evals += 1
                    reservoirs[bucket].offer((fen, target, cp))

                if games_total % 10_000 == 0:
                    fill = ", ".join(
                        f"{PHASE_BUCKETS[i][0]}={len(train[i].items):,}/{quotas[i]:,}"
                        for i in range(len(quotas))
                    )
                    print(f"  games={games_total:,} unique={len(seen):,} | {fill}")
                if args.preflight_games and path_games >= path_limit:
                    break

    print("\nSummary:")
    print(f"  Games read       : {games_total:,}")
    print(f"  Games skipped    : {games_skipped:,}")
    print(f"  Raw candidates   : {raw_candidates:,}")
    print(f"  Quiet-rejected   : {quiet_rejected:,}")
    print(f"  Unique positions : {len(seen):,}")
    if args.blend < 1.0:
        print(f"  Missing evals    : {missing_evals:,} (pure-WDL fallback)")

    if args.preflight_games:
        print("\nPreflight estimate (unique sampled positions/game, with safety margin):")
        required = 0
        for i, quota in enumerate(quotas):
            # unique_by_phase includes both splits. Convert to expected train rate.
            train_rate = unique_by_phase[i] / max(games_total, 1) * (1.0 - args.holdout_pct / 100.0)
            games = math.ceil(quota / train_rate * args.preflight_safety) if train_rate else math.inf
            if games != math.inf:
                required = max(required, int(games))
            print(f"  {PHASE_BUCKETS[i][0]:13} rate={train_rate:6.3f}  required={games:,}")
        if required:
            print(f"Recommended minimum: {required:,} independent games, including "
                  f"{args.preflight_safety:g}x safety.")
        return 0

    train_counts = [len(res.items) for res in train]
    holdout_counts = [len(res.items) for res in holdout]
    for i, name in enumerate(name for name, _, _ in PHASE_BUCKETS):
        print(f"  {name:13}: train {train_counts[i]:,}/{quotas[i]:,} "
              f"holdout {holdout_counts[i]:,}/{holdout_quotas[i]:,} "
              f"(eligible train stream {train[i].seen:,})")

    short = [i for i in range(len(quotas)) if train_counts[i] < quotas[i] or holdout_counts[i] < holdout_quotas[i]]
    if short:
        print("\nERROR: dataset quotas were not met; existing outputs were left untouched.", file=sys.stderr)
        for i in short:
            print(f"  {PHASE_BUCKETS[i][0]}: need {quotas[i] - train_counts[i]:,} more train and "
                  f"{holdout_quotas[i] - holdout_counts[i]:,} more holdout rows", file=sys.stderr)
        print("Run --preflight-games 20000 on the intended PGN mix before the full pass, "
              "or add more independent games for the short phase.", file=sys.stderr)
        return 2

    train_rows = [(fen, target) for reservoir in train for fen, target, _ in reservoir.items]
    holdout_rows = [(fen, target) for reservoir in holdout for fen, target, _ in reservoir.items]
    rng.shuffle(train_rows)
    rng.shuffle(holdout_rows)
    out_dir.mkdir(parents=True, exist_ok=True)
    train_path = out_dir / args.train
    holdout_path = out_dir / args.holdout
    print(f"\nWriting {len(train_rows):,} rows -> {train_path}")
    train_tmp = stage_rows(train_path, train_rows)
    print(f"Writing {len(holdout_rows):,} rows -> {holdout_path}")
    holdout_tmp = stage_rows(holdout_path, holdout_rows)
    # Both complete files exist before either published output is replaced.
    os.replace(train_tmp, train_path)
    os.replace(holdout_tmp, holdout_path)
    print("Target met with the requested phase distribution.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
