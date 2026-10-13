#!/usr/bin/env python3
"""Exact endgame families in played games: how often they occur, whether the
move that entered them changed its mover's Syzygy result, and whether games
ended as the tablebase said at entry.

A family is written strong side first, `KRP-KR`, and matches either colour
assignment. For every game the first position of each family is graded with
Syzygy (WDL from White's side; the cursed values count as draws, since the
games play the fifty-move rule), as is the position before the entering move
when it has six men or fewer. A game is *affected* when the entering move
worsened its mover's result (a win to a draw, a draw to a loss), or when the
game did not end as the tablebase said at entry. The expected value of fixing
every affected game, an upper bound, is affected / games x 0.5 points x 6.95
Elo per 1% of score near 50% (RAR-E38's rule).

  python tools/diag/ending_games.py --pgn <games.pgn> --syzygy <dir> \
      --families KRP-KR,KR-KP,KP-K,KP-KP --output report.json

Static truth on played games; it ranks what a change could be worth and
accepts nothing.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

import chess
import chess.pgn
import chess.syzygy

PIECES = {"Q": chess.QUEEN, "R": chess.ROOK, "B": chess.BISHOP, "N": chess.KNIGHT,
          "P": chess.PAWN}
ELO_PER_PERCENT = 6.95


def signature(spec: str) -> tuple[Counter, Counter]:
    """`KRP-KR` -> (Counter of the strong side's non-king pieces, the weak's)."""
    strong, weak = spec.split("-")
    if not (strong.startswith("K") and weak.startswith("K")):
        raise ValueError(f"family {spec!r} must look like KRP-KR")
    return Counter(strong[1:]), Counter(weak[1:])


def material(board: chess.Board, color: bool) -> Counter:
    c = Counter()
    for symbol, piece in PIECES.items():
        n = len(board.pieces(piece, color))
        if n:
            c[symbol] = n
    return c


def match(board: chess.Board, families: dict) -> tuple[str, bool] | None:
    """(family, strong side's colour) for the first family the board is, else None."""
    white, black = material(board, chess.WHITE), material(board, chess.BLACK)
    for name, (strong, weak) in families.items():
        if white == strong and black == weak:
            return name, chess.WHITE
        if black == strong and white == weak:
            return name, chess.BLACK
    return None


def verdict(wdl: int) -> str:
    return "win" if wdl >= 2 else "loss" if wdl <= -2 else "draw"


def probe_white(tb, board: chess.Board):
    """WDL from White's side, or None beyond the tables."""
    if len(board.piece_map()) > 6 or board.castling_rights:
        return None
    try:
        wdl = tb.probe_wdl(board)
    except (chess.syzygy.MissingTableError, KeyError, ValueError):
        return None
    return wdl if board.turn == chess.WHITE else -wdl


def result_white(result: str) -> str | None:
    return {"1-0": "win", "0-1": "loss", "1/2-1/2": "draw"}.get(result)


RANK = {"loss": 0, "draw": 1, "win": 2}


def flip(v: str) -> str:
    return {"win": "loss", "loss": "win", "draw": "draw"}[v]


def read(pgn_path: Path, tb, families: dict) -> dict:
    games = official = 0
    entries = []
    with pgn_path.open(encoding="utf-8") as handle:
        while True:
            game = chess.pgn.read_game(handle)
            if game is None:
                break
            games += 1
            if game.errors:
                raise SystemExit(f"game {games}: parse errors {game.errors}")
            is_official = game.headers.get("ColosseumSample", "official") == "official"
            official += is_official
            final = result_white(game.headers.get("Result", "*"))
            board = game.board()
            seen = set()
            for ply, node in enumerate(game.mainline(), start=1):
                before = board.copy(stack=False)
                mover = board.turn
                board.push(node.move)
                hit = match(board, families)
                if hit is None or hit[0] in seen:
                    continue
                name, strong = hit
                seen.add(name)
                w_before = probe_white(tb, before)
                w_after = probe_white(tb, board)
                to_mover = (lambda v: v) if mover == chess.WHITE else flip
                entries.append({
                    "game": games, "official": is_official, "family": name, "ply": ply,
                    "fen_before": before.fen(), "move": node.move.uci(),
                    "capture": before.is_capture(node.move),
                    "mover": "strong" if mover == strong else "weak",
                    "mover_before": to_mover(verdict(w_before)) if w_before is not None else None,
                    "mover_after": to_mover(verdict(w_after)) if w_after is not None else None,
                    "white_after": verdict(w_after) if w_after is not None else None,
                    "white_result": final,
                })
    return {"games": games, "official": official, "entries": entries}


def summarise(data: dict, names) -> dict:
    entries = data["entries"]
    worsened = [e for e in entries if e["mover_before"] and e["mover_after"]
                and RANK[e["mover_after"]] < RANK[e["mover_before"]]]
    missed = [e for e in entries if e["white_after"] and e["white_result"]
              and e["white_after"] != e["white_result"]]
    affected = sorted({e["game"] for e in worsened + missed})
    by_family = {}
    for name in names:
        fe = [e for e in entries if e["family"] == name]
        truth = Counter(e["white_after"] for e in fe)
        held = Counter((e["white_after"], e["white_after"] == e["white_result"]) for e in fe)
        by_family[name] = {
            "games": len(fe),
            "share_pct": round(100.0 * len(fe) / data["games"], 3) if data["games"] else None,
            "entered_by": dict(Counter(e["mover"] for e in fe)),
            "entering_captures": sum(e["capture"] for e in fe),
            "truth_at_entry": dict(truth),
            "ended_as_truth": {f"{k[0]}:{'yes' if k[1] else 'no'}": v for k, v in held.items()},
            "worsening_entries": sum(1 for e in worsened if e["family"] == name),
            "affected_games": len({e["game"] for e in worsened + missed if e["family"] == name}),
        }
    return {
        "games": data["games"], "official_games": data["official"],
        "by_family": by_family,
        "affected_games": affected,
        "expected_value_upper_bound_elo": round(
            len(affected) / data["games"] * 0.5 * 100 * ELO_PER_PERCENT, 3) if data["games"] else None,
        "worsening_entries": worsened,
        "endings_not_as_truth": missed,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--pgn", required=True, type=Path)
    ap.add_argument("--syzygy", required=True, type=Path)
    ap.add_argument("--families", required=True)
    ap.add_argument("--output", type=Path)
    args = ap.parse_args()
    names = args.families.split(",")
    families = {name: signature(name) for name in names}
    tb = chess.syzygy.open_tablebase(str(args.syzygy))
    report = summarise(read(args.pgn, tb, families), names)
    text = json.dumps(report, indent=1)
    if args.output:
        args.output.write_text(text + "\n")
    short = {k: v for k, v in report.items() if k not in ("worsening_entries", "endings_not_as_truth")}
    print(json.dumps(short, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
