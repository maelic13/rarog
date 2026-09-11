#!/usr/bin/env python3
"""PGN -> phase-balanced FEN;target data for Rarog Texel tuning.

The default contract is an exact 3,000,000-position training set plus separate
validation and frozen-test splits, each balanced across five material phases.
Whole supplied starts are assigned by stable hash, so a replay cannot leak
across splits and input order cannot change membership. Positions are sampled
per phase *inside each game* before entering fixed-size reservoirs.

Examples:
    # Cheap sizing pass; reads only the first 20k games.
    python tools/texel/extract.py tools/texel/data/*.pgn --preflight-games 20000

    # One extraction over any number of archives; publishes train, validation,
    # frozen test and a hash-complete manifest when every quota is available.
    python tools/texel/extract.py tools/texel/data/*.pgn \
        --out-dir tools/texel/data/hce-v1

Output is FEN;target, with the target from White's perspective in [0,1].
Requires python-chess (``pip install chess``).
"""

from __future__ import annotations

import argparse
import glob
import hashlib
import json
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
BUCKET_NAMES = tuple(item[0] for item in PHASE_BUCKETS)
SPLITS = ("train", "validation", "test")

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
    """Static-eval identity: retain rule-50 clock, discard fullmove number."""
    return " ".join(fen.split()[:5])


def start_key(game: "chess.pgn.Game") -> str:
    """Every replay of one supplied start belongs to the same data split."""
    return " ".join(game.board().fen().split()[:4])


def start_digest(game: "chess.pgn.Game") -> int:
    return int.from_bytes(hashlib.sha256(start_key(game).encode("utf-8")).digest()[:8], "big")


def split_for_key(key: str, validation_pct: float, test_pct: float) -> str:
    slot = int.from_bytes(hashlib.sha256(key.encode("utf-8")).digest()[:8], "big") % 1_000_000
    test_cut = round(test_pct * 10_000)
    validation_cut = test_cut + round(validation_pct * 10_000)
    if slot < test_cut:
        return "test"
    if slot < validation_cut:
        return "validation"
    return "train"


def split_for(game: "chess.pgn.Game", validation_pct: float, test_pct: float) -> str:
    return split_for_key(start_key(game), validation_pct, test_pct)


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
    audit: dict[str, int | bool] | None = None,
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

    if audit is not None:
        audit["plies"] = len(nodes)
        audit["natural_mate"] = board.is_checkmate()

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


def split_counts(train: int, validation_pct: float, test_pct: float) -> dict[str, int]:
    train_fraction = 1.0 - (validation_pct + test_pct) / 100.0
    return {
        "train": train,
        "validation": round(train * validation_pct / 100.0 / train_fraction),
        "test": round(train * test_pct / 100.0 / train_fraction),
    }


def make_reservoirs(counts: dict[str, int], weights: list[float], seed: int):
    quotas = {split: allocate(counts[split], weights) for split in SPLITS}
    reservoirs = {
        split: [
            Reservoir(
                quota,
                random.Random(seed ^ (split_index + 1) * 0x9E3779B1 ^ (phase + 1) * 0x85EBCA77),
            )
            for phase, quota in enumerate(quotas[split])
        ]
        for split_index, split in enumerate(SPLITS)
    }
    return quotas, reservoirs


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest().upper()


def stage_rows(path: Path, rows: Iterable[tuple[str, float]]) -> Path:
    tmp = path.with_name(path.name + ".tmp")
    with tmp.open("w", encoding="utf-8", newline="\n") as out:
        for fen, target in rows:
            out.write(f"{fen};{fmt_target(target)}\n")
    return tmp


def parser() -> argparse.ArgumentParser:
    command = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    command.add_argument("pgn", nargs="+", help="PGN files, globs, or directories")
    command.add_argument("--out-dir", default="", metavar="DIR")
    command.add_argument("--train", default="train.csv", metavar="FILE")
    command.add_argument("--validation", "--holdout", dest="validation", default="validation.csv", metavar="FILE")
    command.add_argument("--test", default="test.csv", metavar="FILE")
    command.add_argument("--target-train", default=3_000_000, type=int, metavar="N")
    command.add_argument("--phase-weights", default=parse_phase_weights("1,1,1,1,1"), type=parse_phase_weights,
                         metavar="A,B,C,D,E", help="target mix for the five phase buckets (default equal)")
    command.add_argument("--validation-pct", "--holdout-pct", dest="validation_pct",
                         default=5.0, type=float, metavar="N")
    command.add_argument("--test-pct", default=5.0, type=float, metavar="N")
    command.add_argument("--max-per-phase-per-game", default=8, type=int, metavar="N")
    command.add_argument("--max-per-game", default=16, type=int, metavar="N")
    command.add_argument("--skip-start", default=2, type=int, metavar="N")
    command.add_argument("--skip-end", default=6, type=int, metavar="N")
    command.add_argument("--seed", default=42, type=int, metavar="N")
    command.add_argument("--preflight-games", default=0, type=int, metavar="N")
    command.add_argument("--preflight-safety", default=1.25, type=float, metavar="X")
    command.add_argument("--no-quiet-filter", dest="quiet_filter", action="store_false")
    command.add_argument("--blend", default=1.0, type=float, metavar="LAMBDA",
                         help="training target = lambda*WDL + (1-lambda)*sigmoid(search cp)")
    return command


def validate_args(args, command: argparse.ArgumentParser) -> None:
    if args.target_train <= 0 or args.max_per_phase_per_game <= 0 or args.max_per_game <= 0:
        command.error("targets and per-game caps must be positive")
    if args.max_per_game < args.max_per_phase_per_game:
        command.error("max per game cannot be smaller than max per phase")
    if min(args.validation_pct, args.test_pct) < 0 or args.validation_pct + args.test_pct >= 100:
        command.error("validation/test percentages must be non-negative and sum below 100")
    if not 0.0 <= args.blend <= 1.0:
        command.error("--blend must be in [0,1]")
    if args.skip_start < 0 or args.skip_end < 0:
        command.error("skip counts cannot be negative")


def main() -> int:
    command = parser()
    args = command.parse_args()
    validate_args(args, command)
    paths = iter_pgn_paths(args.pgn)
    out_dir = Path(args.out_dir).resolve() if args.out_dir else paths[0].parent
    counts = split_counts(args.target_train, args.validation_pct, args.test_pct)
    quotas, reservoirs = make_reservoirs(counts, args.phase_weights, args.seed)
    unique = {split: [0] * len(PHASE_BUCKETS) for split in SPLITS}
    games_by_split = {split: 0 for split in SPLITS}
    seen_positions: set[str] = set()
    seen_starts: set[str] = set()
    independent = recorded = skipped = raw = quiet_rejected = parse_errors = missing_evals = 0

    for path in paths:
        print(f"Reading {path} ...")
        with path.open(encoding="utf-8", errors="replace") as stream:
            while not args.preflight_games or independent < args.preflight_games:
                try:
                    game = chess.pgn.read_game(stream)
                except Exception as exc:
                    print(f"WARNING: parse error, skipping game: {exc}", file=sys.stderr)
                    parse_errors += 1
                    continue
                if game is None:
                    break
                recorded += 1
                opening = start_key(game)
                if opening in seen_starts:
                    continue
                seen_starts.add(opening)
                independent += 1
                split = split_for(game, args.validation_pct, args.test_pct)
                games_by_split[split] += 1
                candidates, rejected = process_game(
                    game, args.skip_start, args.skip_end, args.max_per_phase_per_game,
                    args.max_per_game, args.quiet_filter,
                    random.Random(args.seed ^ start_digest(game)),
                )
                quiet_rejected += rejected
                if not candidates:
                    skipped += 1
                    continue
                raw += len(candidates)
                result = RESULT_MAP[game.headers["Result"]]
                for fen, bucket, cp in candidates:
                    key = fen_key(fen)
                    if key in seen_positions:
                        continue
                    seen_positions.add(key)
                    unique[split][bucket] += 1
                    target = result
                    if split == "train" and args.blend < 1.0:
                        if cp is None:
                            missing_evals += 1
                        else:
                            target = args.blend * result + (1.0 - args.blend) * sigmoid_cp(cp)
                    reservoirs[split][bucket].offer((fen, target, cp))
        if args.preflight_games and independent >= args.preflight_games:
            break

    print(f"Independent starts={independent:,} recorded_games={recorded:,} "
          f"paired_replays={recorded-independent:,} skipped={skipped:,} parse_errors={parse_errors:,} "
          f"raw={raw:,} unique={len(seen_positions):,} quiet_rejected={quiet_rejected:,}")
    if args.blend < 1.0:
        print(f"Missing training evals={missing_evals:,} (pure-WDL fallback)")

    if args.preflight_games:
        required = 0
        incomplete = False
        print("Preflight by split/phase (safety included):")
        for split in SPLITS:
            for phase, name in enumerate(BUCKET_NAMES):
                rate = unique[split][phase] / max(independent, 1)
                estimate = math.ceil(quotas[split][phase] / rate * args.preflight_safety) if rate else math.inf
                if estimate == math.inf:
                    incomplete = True
                else:
                    required = max(required, estimate)
                print(f"  {split:10}/{name:13} rate={rate:7.4f}/game required={estimate:,}")
        if incomplete:
            print("No recommendation: at least one split/phase had zero pilot yield.", file=sys.stderr)
            return 2
        print(f"Recommended total independent games: {required:,}")
        return 0

    short = []
    for split in SPLITS:
        for phase, name in enumerate(BUCKET_NAMES):
            have = len(reservoirs[split][phase].items)
            want = quotas[split][phase]
            print(f"  {split:10}/{name:13}: {have:,}/{want:,} eligible={reservoirs[split][phase].seen:,}")
            if have < want:
                short.append((split, name))
    if short:
        print("ERROR: exact quotas not met; existing outputs unchanged.", file=sys.stderr)
        return 2

    names = {"train": args.train, "validation": args.validation, "test": args.test}
    targets = {split: out_dir / names[split] for split in SPLITS}
    manifest_path = out_dir / "manifest.json"
    for target in (*targets.values(), manifest_path):
        if target.exists():
            raise FileExistsError(f"refusing to overwrite frozen dataset artifact: {target}")
    out_dir.mkdir(parents=True, exist_ok=True)
    staged: list[tuple[Path, Path]] = []
    output_hashes = {}
    shuffle = random.Random(args.seed)
    for split in SPLITS:
        rows = [(fen, target) for phase in reservoirs[split]
                for fen, target, _cp in phase.items]
        shuffle.shuffle(rows)
        temporary = stage_rows(targets[split], rows)
        staged.append((temporary, targets[split]))
        output_hashes[split] = sha256_file(temporary)

    manifest = {
        "schema": "rarog-hce-wdl-v2",
        "inputs": [{"path": str(path), "bytes": path.stat().st_size,
                    "sha256": sha256_file(path)} for path in paths],
        "seed": args.seed,
        "independent_starts": independent,
        "recorded_games": recorded,
        "paired_replays_discarded": recorded - independent,
        "skipped_games": skipped,
        "parse_errors": parse_errors,
        "games_by_split": games_by_split,
        "rows": counts,
        "phase_quotas": {split: dict(zip(BUCKET_NAMES, quotas[split])) for split in SPLITS},
        "output_sha256": output_hashes,
        "dedup_fields": 5,
        "filters": {"quiet": args.quiet_filter, "skip_start": args.skip_start,
                    "skip_end": args.skip_end,
                    "max_per_phase_per_game": args.max_per_phase_per_game,
                    "max_per_game": args.max_per_game},
        "label": "white-perspective self-play WDL",
        "train_blend": args.blend,
    }
    manifest_tmp = manifest_path.with_name(manifest_path.name + ".tmp")
    manifest_tmp.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    for temporary, target in staged:
        os.replace(temporary, target)
    os.replace(manifest_tmp, manifest_path)
    print(f"Published {sum(counts.values()):,} rows under {out_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
