#!/usr/bin/env python3
"""Audit datagen game results against tablebase truth (PLAN 4.10.8).

If HCE tuning uses game-RESULT labels from self-play, those labels are sound
only if the games decide won endings correctly. Basilisk measured **19.77% of
tablebase clean wins not won at 8,000 nodes datagen, 13.65% at 25,000**, with
~43% of games reaching an adjudicable clean win -- so about **8.5% of all games
carried a result contradicting tablebase truth**, ONE-DIRECTIONALLY toward
draws, concentrated in rook and pawn families. That teaches the evaluator to
undervalue exactly what wins endgames (BAS-E46).

Rarog's own share is what this measures. It is an input to PLAN 4.13, not a
verdict: the fix is 4.13's to choose, and raising datagen nodes is the WEAK fix
(3.1x compute bought a 31% relative reduction).

Two details the count depends on, both easy to get wrong:

* **Probe only to the man-limit actually present.** A missing table is not a
  draw; probing past the limit silently converts "unknown" into "agrees".
* **EXCLUDE cursed wins.** Syzygy WDL 1 is a win the fifty-move rule has
  already turned into a draw. A game drawing one is not evidence of weak play,
  and counting it inflates the defect.

The first clean win a game reaches is the one that counts. Later positions are
consequences of how that one was played, so counting them all would weight long
technical endings more heavily for no reason.

Layer: this is a CORPUS audit, not one of the four measurement layers -- it
grades labels, not play. See analysis/endgame_measurement_layers.md.

Example:

  python tools/diag/datagen_label_audit.py \\
      --pgn A:/Chess/data/hce-v3-tb.pgn \\
      --syzygy D:/chess/tablebases/syzygy3456 \\
      --workers 12 --output tools/results/label-audit/hce-v3-tb.json
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import chess
import chess.pgn
import chess.syzygy

PIECE_LETTER = {
    chess.QUEEN: "Q", chess.ROOK: "R", chess.BISHOP: "B",
    chess.KNIGHT: "N", chess.PAWN: "P",
}


def family_of(board: chess.Board, strong: chess.Color) -> str:
    """Material key written strong-side first, e.g. `KRP-KR`."""
    def side(color: chess.Color) -> str:
        out = "K"
        for piece in (chess.QUEEN, chess.ROOK, chess.BISHOP,
                      chess.KNIGHT, chess.PAWN):
            out += PIECE_LETTER[piece] * len(board.pieces(piece, color))
        return out
    return f"{side(strong)}-{side(not strong)}"


def result_score(result: str) -> float | None:
    """PGN result to a White-perspective score, or None if unfinished."""
    return {"1-0": 1.0, "0-1": 0.0, "1/2-1/2": 0.5}.get(result)


def audit_game(game, tb: chess.syzygy.Tablebase, max_men: int) -> dict | None:
    """The first clean-win position this game reaches, and whether it was won.

    Returns None when the game never reaches an adjudicable clean win inside
    the man-limit, which is itself a number worth having: it is the denominator
    that says how much of the corpus this defect can touch at all.
    """
    score = result_score(game.headers.get("Result", "*"))
    if score is None:
        return None
    board = game.board()
    for move in game.mainline_moves():
        board.push(move)
        if chess.popcount(board.occupied) > max_men:
            continue
        try:
            wdl = tb.probe_wdl(board)
        except (chess.syzygy.MissingTableError, KeyError, ValueError):
            # A missing table is UNKNOWN, never "agrees". Skipping it here is
            # the difference between measuring the corpus and measuring the
            # tables that happen to be installed.
            continue
        if abs(wdl) != 2:
            # 0 is a draw and 1 or -1 is a cursed win -- already drawn under
            # the fifty-move rule, so a drawn game there is correct play.
            continue
        winner = board.turn if wdl == 2 else not board.turn
        won = (score == 1.0) if winner == chess.WHITE else (score == 0.0)
        return {
            "family": family_of(board, winner),
            "won": won,
            "men": chess.popcount(board.occupied),
            "ply": board.ply(),
        }
    return None


def _worker(task):
    pgn_path, syzygy, max_men, offsets = task
    tb = chess.syzygy.open_tablebase(syzygy)
    out = []
    try:
        with open(pgn_path, encoding="utf-8", errors="replace") as handle:
            for index, offset in offsets:
                handle.seek(offset)
                game = chess.pgn.read_game(handle)
                if game is None:
                    continue
                out.append((index, audit_game(game, tb, max_men)))
    finally:
        tb.close()
    return out


def scan_offsets(path: Path) -> list[int]:
    """Byte offset of each game, by skipping headers and movetext.

    python-chess 1.11 has no `scan_offsets`, so this is `skip_game` in a loop --
    the same thing, and cheap because it never builds a game tree. The offsets
    are what make sharding possible without holding the corpus in memory, and
    what make a sharded run address exactly the games a serial run does.
    """
    out = []
    with open(path, encoding="utf-8", errors="replace") as handle:
        while True:
            offset = handle.tell()
            if not chess.pgn.skip_game(handle):
                break
            out.append(offset)
    return out


def shard(items, workers: int) -> list[list]:
    """Fixed-index round-robin, so parallel output equals serial output."""
    buckets = [[] for _ in range(workers)]
    for i, item in enumerate(items):
        buckets[i % workers].append(item)
    return [b for b in buckets if b]


def summarize(findings: list[dict | None], games: int) -> dict:
    """Aggregate. Both denominators are reported, because they answer
    different questions: `not_won / clean_wins` is how badly the endings are
    played, `not_won / games` is how much of the CORPUS carries a wrong label.
    """
    reached = [f for f in findings if f is not None]
    not_won = [f for f in reached if not f["won"]]
    by_family: dict[str, Counter] = {}
    for f in reached:
        c = by_family.setdefault(f["family"], Counter())
        c["clean_wins"] += 1
        if not f["won"]:
            c["not_won"] += 1
    families = {
        name: {
            "clean_wins": c["clean_wins"],
            "not_won": c["not_won"],
            "share_not_won": round(c["not_won"] / c["clean_wins"], 4),
        }
        for name, c in sorted(by_family.items(),
                              key=lambda kv: -kv[1]["clean_wins"])
    }
    return {
        "games": games,
        "games_reaching_a_clean_win": len(reached),
        "share_of_games_reaching_a_clean_win":
            round(len(reached) / games, 4) if games else None,
        "clean_wins_not_won": len(not_won),
        "share_of_clean_wins_not_won":
            round(len(not_won) / len(reached), 4) if reached else None,
        "share_of_all_games_mislabelled":
            round(len(not_won) / games, 4) if games else None,
        "families": families,
    }


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--pgn", required=True, type=Path)
    ap.add_argument("--syzygy", required=True, type=Path)
    ap.add_argument("--max-men", type=int, default=6,
                    help="probe only to the man-limit actually installed")
    ap.add_argument("--limit", type=int, default=0,
                    help="audit at most this many games (0 = all)")
    ap.add_argument("--workers", type=int, default=1)
    ap.add_argument("--output", type=Path)
    args = ap.parse_args()

    if not args.pgn.is_file():
        ap.error(f"pgn not found: {args.pgn}")
    if not args.syzygy.is_dir():
        ap.error(f"syzygy path is not a directory: {args.syzygy}")
    if args.workers < 1:
        ap.error("workers must be at least 1")
    if not 3 <= args.max_men <= 7:
        ap.error("max-men must be between 3 and 7")

    print("scanning game offsets...", flush=True)
    offsets = list(enumerate(scan_offsets(args.pgn)))
    if args.limit:
        offsets = offsets[:args.limit]
    print(f"{len(offsets)} games", flush=True)
    if not offsets:
        ap.error("no games found")

    results: dict[int, dict | None] = {}
    if args.workers == 1:
        tb = chess.syzygy.open_tablebase(str(args.syzygy))
        try:
            with open(args.pgn, encoding="utf-8", errors="replace") as handle:
                for index, offset in offsets:
                    handle.seek(offset)
                    game = chess.pgn.read_game(handle)
                    if game is None:
                        continue
                    results[index] = audit_game(game, tb, args.max_men)
                    if (index + 1) % 2000 == 0:
                        print(f"{index + 1}/{len(offsets)}", flush=True)
        finally:
            tb.close()
    else:
        shards = shard(offsets, args.workers)
        tasks = [(str(args.pgn), str(args.syzygy.resolve()), args.max_men, s)
                 for s in shards]
        with ProcessPoolExecutor(max_workers=len(shards)) as pool:
            done = 0
            for produced in pool.map(_worker, tasks):
                for index, finding in produced:
                    results[index] = finding
                done += len(produced)
                print(f"{done}/{len(offsets)}", flush=True)

    findings = [results.get(i) for i, _ in offsets]
    report = {
        "schema": "rarog-datagen-label-audit-v1",
        "layer": "corpus_label_audit",
        "layer_note": (
            "grades LABELS against tablebase truth, not play. Cursed wins are "
            "excluded; probing stops at the installed man-limit."
        ),
        "pgn": str(args.pgn.resolve()),
        "syzygy": str(args.syzygy.resolve()),
        "max_men": args.max_men,
        "workers": args.workers,
        **summarize(findings, len(offsets)),
    }

    rendered = json.dumps(report, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8", newline="\n")
        print(f"Report: {args.output.resolve()}")
    def pct(value):
        # An empty numerator prints as empty, not as 0.00%. The first
        # smoke run CRASHED here on a corpus whose games never reached an
        # adjudicable clean win -- `summarize` returned None correctly and
        # its unit test covered that, but nothing covered the printing.
        # Crashing was the better of the two failures available; reporting
        # 0.00% would have read as "no defect" on a corpus with no data.
        return "n/a" if value is None else f"{value:.2%}"

    print(f"\ngames                       {report['games']:,}")
    print(f"reaching a clean win        {report['games_reaching_a_clean_win']:,} "
          f"({pct(report['share_of_games_reaching_a_clean_win'])})")
    print(f"of those, NOT won           {report['clean_wins_not_won']:,} "
          f"({pct(report['share_of_clean_wins_not_won'])})")
    print(f"share of ALL games mislabelled "
          f"{pct(report['share_of_all_games_mislabelled'])}")
    if not report["families"]:
        print(f"\nno family reached an adjudicable clean win inside "
              f"{args.max_men} men")
        return 0
    print("\nworst families by volume:")
    for name, f in list(report["families"].items())[:10]:
        print(f"  {name:<12} {f['not_won']:>6}/{f['clean_wins']:<6} "
              f"{f['share_not_won']:.2%}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
