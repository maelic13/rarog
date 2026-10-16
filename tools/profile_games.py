"""Replay real game positions through an engine: the game-play workload for a profile.

`bench 13` searches 40 fixed positions from an empty hash. A game searches every
ply of one line with the hash carried forward, at a few hundred thousand nodes a
move. This script plays that second shape so a profiler can see it: it picks
games from a PGN by seed and, for every ply of each, sends the position and
`go nodes N`, with `ucinewgame` between games.

The workload is deterministic for a given engine, PGN, seed and node limit
(one thread, fixed nodes), so two profiles of one binary see the same searches.

    python tools/profile_games.py --engine tools/profile/rarog.exe \
        --pgn tools/results/rar-m72/games.pgn --games 6 --nodes 150000
"""

import argparse
import random
import sys
import time
from pathlib import Path

import chess
import chess.engine
import chess.pgn


def load_games(pgn_path: Path) -> list[chess.pgn.Game]:
    games = []
    with pgn_path.open(encoding="utf-8") as handle:
        while (game := chess.pgn.read_game(handle)) is not None:
            if game.errors:
                raise SystemExit(f"{pgn_path}: game {len(games) + 1} did not parse: {game.errors[0]}")
            games.append(game)
    return games


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--engine", type=Path, required=True)
    parser.add_argument("--pgn", type=Path, required=True)
    parser.add_argument("--games", type=int, default=6)
    parser.add_argument("--seed", type=int, default=12)
    parser.add_argument("--nodes", type=int, default=150_000)
    parser.add_argument("--hash", type=int, default=64)
    args = parser.parse_args()

    if not args.engine.is_file():
        raise SystemExit(f"engine not found: {args.engine}")
    games = load_games(args.pgn)
    if len(games) < args.games:
        raise SystemExit(f"{args.pgn} holds {len(games)} games, fewer than the {args.games} asked for")
    picked = random.Random(args.seed).sample(range(len(games)), args.games)

    searches = 0
    nodes = 0
    started = time.perf_counter()
    engine = chess.engine.SimpleEngine.popen_uci(str(args.engine))
    try:
        engine.configure({"Hash": args.hash, "Threads": 1})
        for index in picked:
            game = games[index]
            board = game.board()
            # A fresh object per game makes python-chess send `ucinewgame`.
            game_id = object()
            for move in game.mainline_moves():
                result = engine.play(board, chess.engine.Limit(nodes=args.nodes), game=game_id,
                                     info=chess.engine.INFO_BASIC)
                if result.move is None:
                    raise SystemExit(f"game {index + 1}: no move at ply {board.ply()}")
                searches += 1
                nodes += result.info.get("nodes", 0)
                board.push(move)
    finally:
        engine.quit()

    elapsed = time.perf_counter() - started
    if searches == 0:
        raise SystemExit("no position was searched")
    print(f"games {args.games} (indices {sorted(i + 1 for i in picked)}), searches {searches}, "
          f"nodes {nodes:,}, {elapsed:.1f} s, {nodes / elapsed / 1e6:.2f} MNPS wall")
    return 0


if __name__ == "__main__":
    sys.exit(main())
