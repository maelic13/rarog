#!/usr/bin/env python3
"""Read a tablebase-enabled Colosseum run: activation, conversion, time.

B.5.2.2's activation read (PLAN B.5.2). For each side of a finished run
directory (`games.pgn`, `games.jsonl`) it reports:

- moves played from a tablebase root (the position before the move is in the
  tables), as a count and a share;
- conversion: every game in which the side to move stood on a clean
  tablebase win (WDL win and distance to zeroing plus the rule-50 counter
  within 100) at any of its moves, and whether that side won it;
- time at tablebase roots: each such move's time (`t=` comment) against the
  optimum Rarog's clock computes for that move (`src/search/time.rs`,
  sudden death with increment), and how many exceeded it by more than the PV
  extension's box plus a millisecond;
- terminations other than normal play, time losses included.

The PV extension's "requires more time" notice is an engine `info string`
and does not reach the PGN; it is not measured here.

  python tools/diag/tb_activation.py --run tools/results/<dir> \\
      --syzygy D:/chess/tablebases/syzygy3456 [--json out.json]
"""

from __future__ import annotations

import argparse
import json
import math
import re
import sys
from collections import defaultdict
from pathlib import Path

import chess
import chess.pgn
import chess.syzygy

TIME = re.compile(r"\bt=(\d+)ms")


def optimum_ms(time_ms: float, inc_ms: float, overhead_ms: float, game_ply: int) -> float:
    """`compute_runtime_limits`' optimum, sudden death with increment."""
    mtg = 50.0
    time_left = max(1.0, time_ms + inc_ms * (mtg - 1) - overhead_ms * (2 + mtg))
    log_t = math.log10(max(time_left / 1000.0, 1e-9))
    opt_const = min(0.0029869 + 0.00033554 * log_t, 0.004905)
    opt = min(0.012112 + max(game_ply + 3.22713, 0.0) ** 0.46866 * opt_const,
              0.19404 * time_ms / time_left)
    optimum = max(opt * time_left, 1.0)
    # The hard ceiling also caps the optimum.
    return min(optimum, max(time_ms - 2 * overhead_ms, 1.0))


def clean_win(tables: chess.syzygy.Tablebase, board: chess.Board) -> bool | None:
    """Whether the side to move has a win the rule-50 counter cannot spoil;
    None when the position is not in the tables."""
    if board.has_castling_rights(chess.WHITE) or board.has_castling_rights(chess.BLACK):
        return None
    try:
        wdl = tables.probe_wdl(board)
        if wdl != 2:
            return False
        dtz = tables.probe_dtz(board)
    except (KeyError, chess.syzygy.MissingTableError):
        return None
    return dtz > 0 and dtz + board.halfmove_clock <= 100


def in_tables(tables: chess.syzygy.Tablebase, board: chess.Board) -> bool:
    if board.has_castling_rights(chess.WHITE) or board.has_castling_rights(chess.BLACK):
        return False
    try:
        tables.probe_wdl(board)
    except (KeyError, chess.syzygy.MissingTableError):
        return False
    return True


def analyse(run: Path, syzygy: str, base_ms: float, inc_ms: float, overhead_ms: float,
            box_ms: float) -> dict:
    tables = chess.syzygy.open_tablebase(syzygy)
    sides: dict[str, dict] = defaultdict(lambda: {
        "moves": 0, "tb_root_moves": 0, "tb_root_ms": [], "tb_root_over_optimum": 0,
        "won_positions_games": 0, "won_positions_converted": 0,
        "unconverted_games": [],
    })
    terminations: dict[str, int] = defaultdict(int)
    games = 0
    with open(run / "games.pgn", encoding="utf-8") as handle:
        while True:
            game = chess.pgn.read_game(handle)
            if game is None:
                break
            games += 1
            names = {chess.WHITE: game.headers.get("White", "white"),
                     chess.BLACK: game.headers.get("Black", "black")}
            terminations[game.headers.get("Termination", "?")] += 1
            result = game.headers.get("Result", "*")
            winner = {"1-0": chess.WHITE, "0-1": chess.BLACK}.get(result)
            board = game.board()
            clock = {chess.WHITE: base_ms, chess.BLACK: base_ms}
            stood_won = {chess.WHITE: False, chess.BLACK: False}
            for node in game.mainline():
                mover = board.turn
                side = sides[names[mover]]
                side["moves"] += 1
                spent = TIME.search(node.comment or "")
                spent_ms = float(spent.group(1)) if spent else 0.0
                if in_tables(tables, board):
                    side["tb_root_moves"] += 1
                    side["tb_root_ms"].append(spent_ms)
                    game_ply = 2 * (board.fullmove_number - 1) + (board.turn == chess.BLACK)
                    limit = optimum_ms(clock[mover], inc_ms, overhead_ms, game_ply)
                    if spent_ms > limit + box_ms + 1.0:
                        side["tb_root_over_optimum"] += 1
                    if clean_win(tables, board):
                        stood_won[mover] = True
                clock[mover] = clock[mover] - spent_ms + inc_ms
                board.push(node.move)
            for colour in (chess.WHITE, chess.BLACK):
                if stood_won[colour]:
                    side = sides[names[colour]]
                    side["won_positions_games"] += 1
                    if winner == colour:
                        side["won_positions_converted"] += 1
                    else:
                        side["unconverted_games"].append(
                            {"game": games, "result": result,
                             "start": game.headers.get("FEN", "startpos")})
    tables.close()
    report = {"run": str(run), "games": games, "terminations": dict(terminations), "sides": {}}
    for name, side in sides.items():
        times = side.pop("tb_root_ms")
        side["tb_root_share"] = side["tb_root_moves"] / side["moves"] if side["moves"] else 0.0
        side["tb_root_ms_mean"] = sum(times) / len(times) if times else 0.0
        side["tb_root_ms_max"] = max(times) if times else 0.0
        report["sides"][name] = side
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    parser.add_argument("--run", required=True, type=Path)
    parser.add_argument("--syzygy", required=True)
    parser.add_argument("--base-ms", type=float, default=3000.0)
    parser.add_argument("--inc-ms", type=float, default=30.0)
    parser.add_argument("--overhead-ms", type=float, default=10.0)
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()
    box_ms = args.overhead_ms / 2.0
    report = analyse(args.run, args.syzygy, args.base_ms, args.inc_ms, args.overhead_ms, box_ms)
    print(f"games {report['games']}  terminations {report['terminations']}")
    for name, side in sorted(report["sides"].items()):
        print(
            f"{name}: moves {side['moves']}, tablebase-root moves {side['tb_root_moves']} "
            f"({100 * side['tb_root_share']:.1f}%), mean {side['tb_root_ms_mean']:.0f} ms, "
            f"max {side['tb_root_ms_max']:.0f} ms, over optimum+box {side['tb_root_over_optimum']}; "
            f"clean-win games {side['won_positions_games']}, converted "
            f"{side['won_positions_converted']}, unconverted {len(side['unconverted_games'])}"
        )
    if args.json:
        args.json.write_text(json.dumps(report, indent=1), encoding="utf-8")
    unconverted = sum(len(side["unconverted_games"]) for side in report["sides"].values())
    return 1 if unconverted else 0


if __name__ == "__main__":
    sys.exit(main())
