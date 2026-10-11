#!/usr/bin/env python3
"""The unstoppable-passer test against Syzygy truth: the engine's rule and the
square rule one tempo stricter, offline, checked against the engine row by row.

Rarog's evaluator (`eval_unstoppable_passers`, `src/eval/passers.rs`) gives a
bonus for each passed pawn with an empty path to promotion when the defender
has no piece other than pawns and its king is too far from the promotion
square: with `steps` the pawn's moves to promote (one fewer from its second
rank) and `dist` the king's distance to the promotion square, the engine
counts the pawn when `dist > steps` with the defender to move and
`dist > steps - 1` with the pawn's side to move. The rule of the square,
counting the king's capture of the new queen, catches the pawn with
`dist <= steps + 1` when the defender moves first and `dist <= steps` when
the pawn moves first, so the engine's rule is one tempo generous in both
orders. `square` is that stricter rule. The engine has played `square`
since C.6.4 (2026-10-11); `engine` names the rule it played before, which
RAR-E40 read. `verify --rule` says which one the dump must match.

  positions  Syzygy-labelled pawn endings of six men or fewer with at least
             one passer the engine's preconditions admit, both sides to move,
             from endgame_truth.py's generator; `fen;label` (White's result)
  verify     the offline engine rule equals the engine's own trace count
             (`rarog-texel --dump-feature <csv> <out> unstoppable_passer_eg
             --full`, whose coefficient is tapered by phase and is turned
             back into a whole count) on every row; exit 1 otherwise
  read       per rule, each (position, side) where the side has a counted
             passer is an event; its truth is that side's Syzygy result
  corpus     on a `fen;label` file (e.g. hce-v4-tb validation): how often each
             rule fires, where they disagree, and the outcome labels there

Static and zero-game: it ranks a definition, it accepts nothing.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

import chess
import chess.syzygy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from endgame_truth import generate_family, parse_family, wdl_for_white  # noqa: E402

RULES = ("engine", "square")
DEFAULT_FAMILIES = "KP-K,KPP-K,KPPP-K,KP-KP,KPP-KP,KPPP-KP,KPP-KPP"


def passed_pawns(board: chess.Board, color: bool) -> list[int]:
    """The engine's passed pawns: no enemy pawn ahead on its or an adjacent file."""
    enemy = board.pieces(chess.PAWN, not color)
    out = []
    for sq in board.pieces(chess.PAWN, color):
        f, r = chess.square_file(sq), chess.square_rank(sq)
        ahead = range(r + 1, 8) if color == chess.WHITE else range(0, r)
        blocked = any(chess.square(ff, rr) in enemy
                      for ff in (f - 1, f, f + 1) if 0 <= ff < 8 for rr in ahead)
        if not blocked:
            out.append(sq)
    return out


def has_pieces(board: chess.Board, color: bool) -> bool:
    return any(board.pieces(p, color) for p in
               (chess.KNIGHT, chess.BISHOP, chess.ROOK, chess.QUEEN))


def counted(board: chess.Board, color: bool, rule: str) -> int:
    """Passers of `color` the rule counts as unstoppable."""
    them = not color
    if has_pieces(board, them):
        return 0
    king = board.king(them)
    defender_to_move = board.turn == them
    n = 0
    for sq in passed_pawns(board, color):
        f = chess.square_file(sq)
        promo = chess.square(f, 7 if color == chess.WHITE else 0)
        rel = chess.square_rank(sq) if color == chess.WHITE else 7 - chess.square_rank(sq)
        path = chess.SquareSet(chess.between(sq, promo)) | chess.SquareSet([promo])
        if any(board.piece_at(s) for s in path):
            continue
        steps = (7 - rel) - (1 if rel == 1 else 0)
        dist = chess.square_distance(king, promo)
        if rule == "engine":
            threshold = steps - (0 if defender_to_move else 1)
        else:
            threshold = steps + (1 if defender_to_move else 0)
        if dist > threshold:
            n += 1
    return n


def net(board: chess.Board, rule: str) -> int:
    return counted(board, chess.WHITE, rule) - counted(board, chess.BLACK, rule)


def admitted(board: chess.Board) -> bool:
    """A side has a passer with an empty path and the other side has no piece."""
    for color in (chess.WHITE, chess.BLACK):
        if has_pieces(board, not color):
            continue
        for sq in passed_pawns(board, color):
            f = chess.square_file(sq)
            promo = chess.square(f, 7 if color == chess.WHITE else 0)
            path = chess.SquareSet(chess.between(sq, promo)) | chess.SquareSet([promo])
            if not any(board.piece_at(s) for s in path):
                return True
    return False


def label(wdl_white: int) -> str:
    """Cursed results count as draws: the games play the fifty-move rule."""
    return "1" if wdl_white >= 2 else "0" if wdl_white <= -2 else "0.5"


def cmd_positions(args) -> int:
    tb = chess.syzygy.open_tablebase(str(args.syzygy))
    rows, census = [], Counter()
    for name in args.families.split(","):
        strong, weak = parse_family(name)
        for board in generate_family(args.seed, name, strong, weak, args.per_family):
            for turn in (chess.WHITE, chess.BLACK):
                b = board.copy(stack=False)
                b.turn = turn
                if not b.is_valid() or b.is_check() or not any(b.legal_moves):
                    continue
                if not admitted(b):
                    continue
                rows.append(f"{b.fen()};{label(wdl_for_white(tb, b))}")
                census[name] += 1
    args.output.write_text("\n".join(rows) + "\n")
    print(json.dumps({"rows": len(rows), "by_family": census}, indent=2))
    return 0


def read_rows(path: Path):
    with path.open() as handle:
        for line in handle:
            line = line.strip()
            if line:
                fen, rest = line.rsplit(";", 1)
                yield fen, rest


TOTAL_PHASE = 24


def engine_count(value: str, phase: str) -> int | None:
    """The eg-only term's integer count from the dump's tapered coefficient
    `count · (24 − phase) / 24`, or None when it is not a whole count."""
    weight = (TOTAL_PHASE - int(phase)) / TOTAL_PHASE
    if weight <= 0:
        return None
    count = float(value) / weight
    return round(count) if abs(count - round(count)) < 1e-9 else None


def read_dump(path: Path):
    """(fen, coefficient, phase) from `rarog-texel --dump-feature` rows."""
    with path.open() as handle:
        for line in handle:
            line = line.strip()
            if line:
                fen, value, phase = line.rsplit(";", 2)
                yield fen, value, phase


def cmd_verify(args) -> int:
    checked = mismatched = fired = unreadable = 0
    examples = []
    for (fen_a, _), (fen_b, value, phase) in zip(read_rows(args.positions),
                                                 read_dump(args.dump), strict=True):
        if fen_a.split()[:4] != fen_b.split()[:4]:
            raise SystemExit(f"row {checked + 1}: FENs differ")
        offline = net(chess.Board(fen_a), args.rule)
        checked += 1
        if int(phase) >= TOTAL_PHASE:
            # The eg weight is zero at full phase: the term cannot move the
            # score and its count cannot be read back. A defender without
            # pieces cannot reach full phase without promoted pieces, so the
            # offline count must be zero here; anything else is a mismatch.
            unreadable += 1
            if offline != 0:
                mismatched += 1
                if len(examples) < 10:
                    examples.append((fen_a, "unreadable", offline))
            continue
        engine = engine_count(value, phase)
        fired += bool(engine)
        if engine != offline:
            mismatched += 1
            if len(examples) < 10:
                examples.append((fen_a, engine, offline))
    ok = mismatched == 0 and checked > 0
    print(json.dumps({"rows": checked, "rows_at_full_phase_unreadable": unreadable,
                      "rows_where_engine_fires": fired, "mismatches": mismatched,
                      "examples": examples, "pass": ok}, indent=2))
    return 0 if ok else 1


def cmd_read(args) -> int:
    events = {rule: Counter() for rule in RULES}
    disagree = Counter()
    for fen, lab in read_rows(args.positions):
        board = chess.Board(fen)
        white = float(lab)
        for color in (chess.WHITE, chess.BLACK):
            result = white if color == chess.WHITE else 1.0 - white
            outcome = {1.0: "win", 0.5: "draw", 0.0: "loss"}[result]
            mover = "defender_to_move" if board.turn != color else "pawn_side_to_move"
            fires = {rule: counted(board, color, rule) > 0 for rule in RULES}
            for rule in RULES:
                if fires[rule]:
                    events[rule][outcome] += 1
                    events[rule][f"{mover}:{outcome}"] += 1
            if fires["engine"] != fires["square"]:
                disagree[f"engine_only:{mover}:{outcome}" if fires["engine"]
                         else f"square_only:{mover}:{outcome}"] += 1
    report = {"events": {}, "disagreements": dict(sorted(disagree.items()))}
    for rule, c in events.items():
        n = c["win"] + c["draw"] + c["loss"]
        report["events"][rule] = {
            "fired": n,
            "win": c["win"], "draw": c["draw"], "loss": c["loss"],
            "precision_win": c["win"] / n if n else None,
            "by_move_order": {k: v for k, v in sorted(c.items()) if ":" in k},
        }
    text = json.dumps(report, indent=2)
    print(text)
    if args.output:
        args.output.write_text(text + "\n")
    return 0


def cmd_corpus(args) -> int:
    fires = Counter()
    disagree = Counter()
    rows = 0
    for fen, lab in read_rows(args.csv):
        rows += 1
        board = chess.Board(fen)
        white = float(lab)
        for color in (chess.WHITE, chess.BLACK):
            got = {rule: counted(board, color, rule) > 0 for rule in RULES}
            for rule in RULES:
                fires[rule] += got[rule]
            if got["engine"] != got["square"]:
                result = white if color == chess.WHITE else 1.0 - white
                kind = "engine_only" if got["engine"] else "square_only"
                disagree[f"{kind}:{result}"] += 1
    report = {"rows": rows, "side_events_fired": dict(fires),
              "disagreements_by_side_result": dict(sorted(disagree.items()))}
    text = json.dumps(report, indent=2)
    print(text)
    if args.output:
        args.output.write_text(text + "\n")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("positions")
    p.add_argument("--syzygy", required=True, type=Path)
    p.add_argument("--families", default=DEFAULT_FAMILIES)
    p.add_argument("--per-family", type=int, default=20000)
    p.add_argument("--seed", type=int, default=0x5E9D18)
    p.add_argument("--output", required=True, type=Path)
    p = sub.add_parser("verify")
    p.add_argument("--positions", required=True, type=Path)
    p.add_argument("--dump", required=True, type=Path)
    p.add_argument("--rule", choices=RULES, default="square",
                   help="the rule the dumping engine plays: square since C.6.4, "
                        "engine before it")
    p = sub.add_parser("read")
    p.add_argument("--positions", required=True, type=Path)
    p.add_argument("--output", type=Path)
    p = sub.add_parser("corpus")
    p.add_argument("--csv", required=True, type=Path)
    p.add_argument("--output", type=Path)
    args = ap.parse_args()
    return {"positions": cmd_positions, "verify": cmd_verify,
            "read": cmd_read, "corpus": cmd_corpus}[args.cmd](args)


if __name__ == "__main__":
    sys.exit(main())
