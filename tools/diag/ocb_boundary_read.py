#!/usr/bin/env python3
"""Who trades down across the opposite-bishop boundary, and how it ends.

RAR-E28's split scales pure opposite-bishop endings of seven men or more hard
and leaves six men or fewer to the older, weaker rule, so one exchange across
the boundary can raise the score. This reads a gate's PGN for that incentive:
for every game, the first move that turns a position with opposite bishops and
seven men or more into a pure opposite-bishop ending of six men or fewer, which
engine made it, and the game's result from that engine's side. A diagnostic;
it decides nothing.

  python tools/diag/ocb_boundary_read.py games.pgn --arms c52ocb,c52head
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import chess  # noqa: E402
import chess.pgn  # noqa: E402

from ocb_scale_screen import NOT_OCB, PURE, classify  # noqa: E402


def men(board: chess.Board) -> int:
    return chess.popcount(board.occupied)


def first_crossing(game: chess.pgn.Game):
    """(ply, mover colour) of the first move from a seven-plus-men opposite-
    bishop position into a pure opposite-bishop ending of six men or fewer,
    or None."""
    board = game.board()
    for ply, move in enumerate(game.mainline_moves()):
        before_ocb = men(board) >= 7 and classify(board.fen())[0] != NOT_OCB
        mover = board.turn
        board.push(move)
        if before_ocb and men(board) <= 6 and classify(board.fen())[0] == PURE:
            return ply, mover
    return None


def score_for(result: str, colour: chess.Color) -> float | None:
    points = {"1-0": 1.0, "0-1": 0.0, "1/2-1/2": 0.5}.get(result)
    if points is None:
        return None
    return points if colour == chess.WHITE else 1.0 - points


def read(pgn_path: Path, arms: list[str]) -> dict:
    out = {"games": 0, "crossings": 0,
           "by_mover": {arm: {"crossings": 0, "points": 0.0, "draws": 0} for arm in arms}}
    with pgn_path.open(encoding="utf-8", errors="replace") as handle:
        while (game := chess.pgn.read_game(handle)) is not None:
            out["games"] += 1
            crossing = first_crossing(game)
            if crossing is None:
                continue
            _ply, mover = crossing
            name = game.headers["White" if mover == chess.WHITE else "Black"]
            if name not in out["by_mover"]:
                continue
            score = score_for(game.headers.get("Result", "*"), mover)
            if score is None:
                continue
            entry = out["by_mover"][name]
            out["crossings"] += 1
            entry["crossings"] += 1
            entry["points"] += score
            entry["draws"] += score == 0.5
    for entry in out["by_mover"].values():
        n = entry["crossings"]
        entry["score_rate"] = round(entry["points"] / n, 4) if n else None
        entry["draw_rate"] = round(entry["draws"] / n, 4) if n else None
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("pgn", type=Path)
    ap.add_argument("--arms", required=True, help="engine names as in the PGN, comma-separated")
    ap.add_argument("--output", type=Path)
    args = ap.parse_args()
    report = read(args.pgn, [a.strip() for a in args.arms.split(",")])
    text = json.dumps(report, indent=2)
    print(text)
    if args.output:
        args.output.write_text(text + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
