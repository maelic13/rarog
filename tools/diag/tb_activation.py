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
  optimum Rarog's clock computes for that move (`budget` and
  `compute_runtime_limits` in `src/search/time.rs`, sudden death with
  increment; the budget's constants are read from that file), and how many
  exceeded it by more than the PV extension's box plus a millisecond;
- terminations other than normal play, time losses included.

The PV extension's "requires more time" notice is an engine `info string`
and does not reach the PGN; it is not measured here.

  python tools/diag/tb_activation.py --run tools/results/<dir> \\
      --syzygy D:/chess/tablebases/syzygy3456 [--threads N] [--json out.json]
"""

from __future__ import annotations

import argparse
import functools
import hashlib
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

TIME_RS = Path(__file__).resolve().parents[2] / "src" / "search" / "time.rs"
CLOCK_CONSTANTS = ("TIME_OVERHEAD_MS", "SOFT_SCALE_LATE", "SOFT_SCALE_EARLY_CUT",
                   "SOFT_SCALE_RATE", "HARD_SCALE", "INCREMENT_SHARE")
# Rarog's reserve, literals inside `compute_runtime_limits`: the hard bound
# stays `2 * overhead` under the clock, 30 ms more at several threads. The
# fixture's small clocks and four-thread cases check them.
OVERHEAD_RESERVE = 2.0
SMP_RESERVE_MS = 30.0


@functools.lru_cache(maxsize=None)
def clock_constants(source: Path = TIME_RS) -> dict[str, float]:
    """`budget`'s constants, read from the clock's source, each declared once."""
    text = source.read_text(encoding="utf-8")
    values = {}
    for name in CLOCK_CONSTANTS:
        found = re.findall(rf"^const {name}: f64 = ([0-9.]+);", text, re.M)
        if len(found) != 1:
            raise SystemExit(f"{source}: expected one `const {name}: f64`, found {len(found)}")
        values[name] = float(found[0])
    return values


def clock_source_extracts(source: Path = TIME_RS) -> dict[str, str]:
    """SHA-256 of the two pieces of `src/search/time.rs` the optimum is
    transcribed from: `budget`, and the reserve in `compute_runtime_limits`
    (line endings ignored), so a change to either shows."""
    text = source.read_text(encoding="utf-8").replace("\r\n", "\n")
    pieces = {
        "budget": (r"^fn budget\(.*?^}\n", re.M | re.S),
        "reserve": (r"^ *let smp_reserve = .*?optimum_ms = soft\.min\(maximum_ms\);\n", re.M | re.S),
    }
    out = {}
    for name, (pattern, flags) in pieces.items():
        found = re.findall(pattern, text, flags)
        if len(found) != 1:
            raise SystemExit(f"{source}: expected one `{name}` extract, found {len(found)}")
        out[f"{name}_sha256"] = hashlib.sha256(found[0].encode("utf-8")).hexdigest()
    return out


def optimum_ms(time_ms: float, inc_ms: float, overhead_ms: float, game_ply: int,
               threads: int = 1) -> float:
    """`compute_runtime_limits`' optimum, sudden death with increment:
    `budget`'s soft bound, held under its hard bound and Rarog's reserve."""
    c = clock_constants()
    main = max(time_ms - overhead_ms, 0.0)
    fullmove = game_ply // 2 + 1
    soft_scale = c["SOFT_SCALE_LATE"] - c["SOFT_SCALE_EARLY_CUT"] * math.exp(-c["SOFT_SCALE_RATE"] * fullmove)
    soft = min(soft_scale * main + c["INCREMENT_SHARE"] * inc_ms, main)
    hard = min(c["HARD_SCALE"] * main + c["INCREMENT_SHARE"] * inc_ms, main)
    soft = max(soft - c["TIME_OVERHEAD_MS"], 1.0)
    hard = max(hard - c["TIME_OVERHEAD_MS"], 1.0)
    reserve = OVERHEAD_RESERVE * overhead_ms + (SMP_RESERVE_MS if threads > 1 else 0.0)
    maximum = min(hard, max(time_ms - reserve, 1.0))
    return min(soft, maximum)


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
            box_ms: float, threads: int = 1) -> dict:
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
                    limit = optimum_ms(clock[mover], inc_ms, overhead_ms, game_ply, threads)
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
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()
    box_ms = args.overhead_ms / 2.0
    report = analyse(args.run, args.syzygy, args.base_ms, args.inc_ms, args.overhead_ms, box_ms,
                     args.threads)
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
