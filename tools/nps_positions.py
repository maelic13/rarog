#!/usr/bin/env python3
"""Freeze the game-position set of the game-position NPS read (PLAN D.16.1).

    python tools/nps_positions.py <games.pgn> <out.jsonl> [--count 200] [--seed 20261123]

The positions a gate's games thought in, drawn so the set's mix is the games'
mix of thinking time: every move with a comment `s=<score> d=<depth>
t=<ms>ms ... n=<nodes>` is a candidate, weighted by its `t`, at most one a
game, and drawn without replacement (Efraimidis-Spirakis keys from a seeded
generator). A move whose score is a mate or tablebase score (|s| at least
20,000) is left out: its tree is not a game's ordinary tree and can end
before any budget. The position is the one the mover thought in, so every
side to move is the engine's own in a self-play gate.

The node budget is the median of the drawn moves' `n`, rounded to a thousand:
a move's work in those games. The set and a manifest beside it
(`<out>.manifest.txt`: the source's SHA-256, the rule, the seed, the count,
the budget, the phase mix and the set's own SHA-256) are written; the read
refuses a set whose hash does not match its manifest.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import random
import re
import statistics
from pathlib import Path

import chess
import chess.pgn

COMMENT = re.compile(r"s=(-?\d+) d=(\d+) t=(-?\d+)ms\b.*?\bn=(\d+)")
DECISIVE = 20_000
VALUES = {chess.KNIGHT: 3, chess.BISHOP: 3, chess.ROOK: 5, chess.QUEEN: 9}


def non_pawn_material(board: chess.Board) -> int:
    """Both sides' non-pawn material in pawns (N=B=3, R=5, Q=9)."""
    return sum(v * chess.popcount(board.pieces_mask(pt, c)) for pt, v in VALUES.items() for c in chess.COLORS)


def band(npm: int) -> str:
    """RAR-O10's four phase bands."""
    return "45+" if npm >= 45 else "25-44" if npm >= 25 else "10-24" if npm >= 10 else "0-9"


def candidates(pgn_path: Path):
    """Every commented move of every game: (game, ply, fen, npm, t, n)."""
    with pgn_path.open(encoding="utf-8") as handle:
        game_number = 0
        while True:
            game = chess.pgn.read_game(handle)
            if game is None:
                return
            game_number += 1
            board = game.board()
            for ply, node in enumerate(game.mainline()):
                m = COMMENT.search(node.comment or "")
                if m:
                    score, _, t, n = (int(x) for x in m.groups())
                    if abs(score) < DECISIVE and t > 0:
                        yield (game_number, ply, board.fen(), non_pawn_material(board), t, n)
                board.push(node.move)


def draw(rows: list, count: int, seed: int) -> list:
    """`count` moves, weighted by thinking time, at most one a game."""
    rng = random.Random(seed)
    keyed = sorted(((math.log(rng.random()) / row[4], row) for row in rows), reverse=True)
    chosen, games, fens = [], set(), set()
    for _, row in keyed:
        if row[0] in games or row[2] in fens:
            continue
        chosen.append(row)
        games.add(row[0])
        fens.add(row[2])
        if len(chosen) == count:
            break
    if len(chosen) < count:
        raise SystemExit(f"only {len(chosen)} positions from distinct games, {count} asked")
    return sorted(chosen)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("pgn", type=Path)
    parser.add_argument("out", type=Path)
    parser.add_argument("--count", type=int, default=200)
    parser.add_argument("--seed", type=int, default=20261123)
    args = parser.parse_args()

    rows = list(candidates(args.pgn))
    chosen = draw(rows, args.count, args.seed)
    budget = int(round(statistics.median(r[5] for r in chosen), -3))
    lines = [json.dumps({"game": g, "ply": p, "fen": f, "npm": npm, "band": band(npm), "t_ms": t, "n": n})
             for g, p, f, npm, t, n in chosen]
    data = ("\n".join(lines) + "\n").encode("utf-8")
    args.out.write_bytes(data)
    mix = {b: sum(1 for r in chosen if band(r[3]) == b) for b in ("45+", "25-44", "10-24", "0-9")}
    time_mix = {b: round(sum(r[4] for r in rows if band(r[3]) == b) / sum(r[4] for r in rows), 3)
                for b in ("45+", "25-44", "10-24", "0-9")}
    manifest = [
        f"source      : {args.pgn.as_posix()}",
        f"source_sha256: {hashlib.sha256(args.pgn.read_bytes()).hexdigest()}",
        "rule        : moves weighted by thinking time, at most one a game, |score| < 20000, seeded keys",
        f"seed        : {args.seed}",
        f"count       : {len(chosen)} of {len(rows)} candidate moves",
        f"nodes       : {budget}",
        f"bands       : {json.dumps(mix)}",
        f"time_share  : {json.dumps(time_mix)}",
        f"set_sha256  : {hashlib.sha256(data).hexdigest()}",
    ]
    manifest_path = args.out.with_name(args.out.name + ".manifest.txt")
    manifest_path.write_bytes(("\n".join(manifest) + "\n").encode("utf-8"))
    print("\n".join(manifest))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
