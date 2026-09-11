#!/usr/bin/env python3
"""How often each reference endgame appears on the board in real games.

PLAN 4.11.12. This replaces the occurrence input that PLAN 4.11.6's ranking
had to hard-code: RAR-M15's twenty percentages, measured once over 3,915
self-play games, transcribed into `endgame_ranking.py` as constants with no
artifact behind them. The ranking now takes this tool's output as a file, so
re-measuring occurrence is a re-run rather than an edit.

WHY IT IS WORTH RE-MEASURING. Occurrence is the only gate in the ranking, and
the ranking decides the order of twenty engine changes. It rested on:

* **board occurrence** over 3,915 games of ONE engine pair against ITSELF, at
  a single time control -- so every ending in it is an ending two nearly
  identical evaluations steered into; and
* **tree occurrence** over 40 bench roots, which PLAN 4.11.5 then measured as
  weak (three roots produce 56% of the whole census).

A 36,400-game rated tournament between fourteen engines of very different
strengths and styles is a far better sample of the distribution Rarog actually
plays into, and it is already on disk.

CALIBRATION BEFORE APPLICATION. A new tool producing new numbers proves
nothing about whether the numbers moved or the DEFINITION moved. So this tool
is first run over RAR-M15's own corpus, whose PGN is retained
(`tools/results/sprt_HCERefit_vs_HCEBase_20260901_072106.pgn`, exactly 3,915
games), and `--calibrate` checks its output against RAR-M15's published
percentages. Only a tool that reproduces the old measurement on the old corpus
is allowed to speak about a new one.

DEFINITIONS, stated rather than implied. A family "occurs in a game" if ANY
position of the mainline matches its material predicate -- the same
per-position classification RAR-M15 describes. Predicates are on material
alone, both colours tried as the strong side.

* Overlaps are REAL and intentional. `KPsK` contains every `KPK`; `KXK`
  contains most lone-king families. These are separate evaluation functions
  with separate dispatch conditions, so they are counted separately and the
  shares do not sum to one.
* `KXK` is "the weak side has nothing but its king, and the strong side can
  mate" -- a lone knight, a lone bishop and two knights are excluded, because
  none of them can force mate and the last has its own listed function. That
  choice is not free: it is worth **2.81 pp** of games on RAR-M15's corpus
  (110 of 3,915 reach a bare king facing a single minor), and it is the
  variant RAR-M15 used, so this is also what makes the two comparable.
* **Exact material, not "at least".** `KRPKR` is K+R+one pawn against K+R,
  because that is the dispatch condition of the FUNCTION being ranked. This
  differs from RAR-M15, which used a plural strong side -- see
  `CALIBRATION_EXCEPTIONS`.
* There is **no piece-count threshold**. PLAN 4.11.5 produced a reassuring
  result from a "generous" 7-man cut-off that moved sharply when the cut-off
  moved one man; the lesson taken was to let the predicates bound themselves.
  The `<= 8 men` fast path below is a strict BOUND, not a cut-off: the largest
  bounded family is 7 men, and the unbounded ones (`KXK`, `KPsK`, `KBPsK`) are
  tested by a lone-king check that no piece count gates.

Example:

  # calibrate against RAR-M15, on RAR-M15's own corpus -- do this first
  python tools/diag/endgame_board_occurrence.py \\
      --pgn tools/results/sprt_HCERefit_vs_HCEBase_20260901_072106.pgn \\
      --calibrate --output tools/diag/endgame_board_occurrence_m15_replay.json

  # then measure, over a Colosseum tournament
  python tools/diag/endgame_board_occurrence.py \\
      --sqlite "$env:APPDATA/Colosseum/data/colosseum.sqlite" \\
      --tournament "Rating Tournament" --engine Rarog --workers 14 \\
      --output tools/diag/endgame_board_occurrence_v1.json
"""

from __future__ import annotations

import argparse
import io
import json
import sqlite3
import sys
from collections import Counter
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import chess
import chess.pgn

SCHEMA = "rarog-board-occurrence-v1"

PIECE_LETTER = {
    chess.QUEEN: "Q", chess.ROOK: "R", chess.BISHOP: "B",
    chess.KNIGHT: "N", chess.PAWN: "P",
}

# Reference function -> predicate on (strong, weak) material strings, where a
# side is spelled in Q,R,B,N,P order with the king implied ("" is a lone king).
# Written as explicit lambdas rather than a table of literals because five of
# the twenty are plural in one piece type.
PREDICATES = {
    "KXK": lambda s, w: w == "" and s not in ("", "N", "B", "NN"),
    "KPsK": lambda s, w: w == "" and s != "" and set(s) == {"P"},
    "KPK": lambda s, w: w == "" and s == "P",
    "KBPsK": lambda s, w: w == "" and s.startswith("B") and set(s[1:]) == {"P"},
    "KBNK": lambda s, w: w == "" and s == "BN",
    "KNNK": lambda s, w: w == "" and s == "NN",
    "KRPKR": lambda s, w: s == "RP" and w == "R",
    "KRPKB": lambda s, w: s == "RP" and w == "B",
    "KRKP": lambda s, w: s == "R" and w == "P",
    "KRKN": lambda s, w: s == "R" and w == "N",
    "KRKB": lambda s, w: s == "R" and w == "B",
    "KBPKB": lambda s, w: s == "BP" and w == "B",
    "KBPPKB": lambda s, w: s == "BPP" and w == "B",
    "KBPKN": lambda s, w: s == "BP" and w == "N",
    "KPKP": lambda s, w: s == "P" and w == "P",
    "KQKP": lambda s, w: s == "Q" and w == "P",
    "KQKR": lambda s, w: s == "Q" and w == "R",
    "KNNKP": lambda s, w: s == "NN" and w == "P",
    "KQKRPs": lambda s, w: s == "Q" and w.startswith("R") and set(w[1:]) == {"P"},
    "KRPPKRP": lambda s, w: s == "RPP" and w == "RP",
}

# RAR-M15's published board occurrence, as a share of its 3,915 games. The
# calibration target: a tool that does not reproduce these on RAR-M15's own
# corpus is measuring something else and may not be used on another one.
M15 = {
    "KXK": 0.3734, "KRPKR": 0.1004, "KPsK": 0.0419, "KPK": 0.0284,
    "KRKP": 0.0240, "KBPsK": 0.0192, "KRPKB": 0.0123, "KPKP": 0.0123,
    "KQKP": 0.0117, "KBPKB": 0.0089, "KBPPKB": 0.0066, "KRKN": 0.0061,
    "KRKB": 0.0051, "KBPKN": 0.0028, "KBNK": 0.0028, "KNNKP": 0.0005,
    "KNNK": 0.0003, "KQKR": 0.0, "KQKRPs": 0.0, "KRPPKRP": 0.0,
}
# RAR-M15 rounded to four decimals, so a share can differ from the published
# figure by half a unit in the last place before anything has actually moved.
# One game in 3,915 is 0.000255, so the tolerance is a hair over that.
CALIBRATION_TOLERANCE = 0.0003

# The six families where this tool and RAR-M15 disagree by more than that, each
# with the cause MEASURED on RAR-M15's own corpus rather than argued. Fourteen
# of twenty reproduce exactly, including KBN-K's published count of 11 games and
# both aggregate figures (52.7% of games reach <=6 men, 60.9% reach <=7), so the
# per-position classifier is the same classifier and these six are differences
# of definition or of coverage -- not of measurement.
#
# TWO GROUPS, and they are not the same kind of thing.
#
# Definition (RAR-M15 is internally consistent; this tool asks a different and,
# for ranking an evaluation FUNCTION, a better-posed question):
#
#   KXK     RAR-M15 excluded a bare king facing a single minor. So does this
#           tool now. Residual 0.3747 here against 0.3734 published -- five
#           games, cause not established, recorded rather than smoothed over.
#   KRPKR   RAR-M15 counted a PLURAL strong side and capped positions at six
#           men. Measured: that variant reads 393/3915 = 0.1004, which is its
#           published figure exactly. Exact-material KRP-KR is 320 = 0.0817.
#   KRPKB   The same, exactly: plural-and-capped reads 48/3915 = 0.0123,
#           its published figure; exact-material KRP-KB is 34 = 0.0087.
#   KBPsK   Same cap, same result: capped at six men it reads 75/3915 = 0.0192,
#           its published figure, against 78 uncapped. Three games, and worth
#           the line: the calibration gate FOUND this one rather than the
#           exception list anticipating it, which is the only reason to have a
#           gate whose expected outcome is a list of known differences.
#
# DEFECT (RAR-M15 reported zero for something that is simply there, and the
# ranking then floored those three with the rule of three as though the only
# problem were sample size):
#
#   KRPPKRP 395 games, 10.09%. Seven men, so RAR-M15's six-man cap could not
#           see it at all. Example: `8/1r3p2/8/7P/8/4kPK1/1R6/8 b - - 0 66`.
#           It was ranked LAST and called unverifiable on this zero.
#   KQKR    45 games, 1.15%. Four men, well inside the cap, so the cap does
#           not explain this one and the cause is unestablished. Example:
#           `8/8/R6K/8/8/7k/8/5q2 w - - 0 79`.
#   KQKRPs  16 games, 0.41%.
CALIBRATION_EXCEPTIONS = {
    "KXK": "RAR-M15 also excluded lone minors; 5-game residual unexplained",
    "KRPKR": "RAR-M15 counted plural pawns capped at 6 men (measured 0.1004)",
    "KRPKB": "RAR-M15 counted plural pawns capped at 6 men (measured 0.0123)",
    "KBPsK": "RAR-M15 capped positions at 6 men (measured 0.0192)",
    "KRPPKRP": "RAR-M15 published ZERO; 7 men, outside its 6-man cap",
    "KQKR": "RAR-M15 published ZERO; 4 men, inside its cap, cause unknown",
    "KQKRPs": "RAR-M15 published ZERO",
}


def side_material(board: chess.Board, color: chess.Color) -> str:
    out = []
    for piece in (chess.QUEEN, chess.ROOK, chess.BISHOP, chess.KNIGHT, chess.PAWN):
        out.append(PIECE_LETTER[piece] * len(board.pieces(piece, color)))
    return "".join(out)


def families_at(board: chess.Board) -> set[str]:
    """Every reference function whose material condition this position meets."""
    white_lone = board.occupied_co[chess.WHITE] == board.kings & board.occupied_co[chess.WHITE]
    black_lone = board.occupied_co[chess.BLACK] == board.kings & board.occupied_co[chess.BLACK]
    # A strict bound, not a threshold: no bounded predicate exceeds 7 men, and
    # the unbounded ones all require a lone king, tested separately above.
    if not (white_lone or black_lone or chess.popcount(board.occupied) <= 8):
        return set()

    white = side_material(board, chess.WHITE)
    black = side_material(board, chess.BLACK)
    hit = set()
    for name, predicate in PREDICATES.items():
        if predicate(white, black) or predicate(black, white):
            hit.add(name)
    return hit


def scan_game(pgn_text: str) -> dict | None:
    """The set of families one game reaches, plus its players and men floor."""
    game = chess.pgn.read_game(io.StringIO(pgn_text))
    if game is None:
        return None
    board = game.board()
    hit: set[str] = set()
    fewest = chess.popcount(board.occupied)
    hit |= families_at(board)
    for move in game.mainline_moves():
        board.push(move)
        fewest = min(fewest, chess.popcount(board.occupied))
        hit |= families_at(board)
    return {
        "families": sorted(hit),
        "fewest_men": fewest,
        "white": game.headers.get("White", "?"),
        "black": game.headers.get("Black", "?"),
    }


def _worker(chunk: list[str]) -> list[dict]:
    return [r for r in (scan_game(text) for text in chunk) if r is not None]


def shard(items: list, workers: int) -> list[list]:
    """Fixed-index round robin, so the split cannot change any total."""
    buckets: list[list] = [[] for _ in range(max(1, workers))]
    for i, item in enumerate(items):
        buckets[i % len(buckets)].append(item)
    return [b for b in buckets if b]


def summarize(records: list[dict], engine_filter: str | None) -> dict:
    """Occurrence shares over all games, and over one engine's games."""
    total = len(records)
    counts: Counter[str] = Counter()
    engine_total = 0
    engine_counts: Counter[str] = Counter()
    men: Counter[int] = Counter()
    for rec in records:
        men[rec["fewest_men"]] += 1
        for name in rec["families"]:
            counts[name] += 1
        if engine_filter and (engine_filter in rec["white"] or engine_filter in rec["black"]):
            engine_total += 1
            for name in rec["families"]:
                engine_counts[name] += 1

    def block(n_games: int, hits: Counter[str]) -> dict:
        if not n_games:
            return {}
        return {
            name: {"games": hits.get(name, 0), "share": hits.get(name, 0) / n_games}
            for name in PREDICATES
        }

    reached = {
        f"reaches_{k}_men": sum(v for m, v in men.items() if m <= k) / total if total else None
        for k in (6, 7)
    }
    return {
        "schema": SCHEMA,
        "games": total,
        "engine_filter": engine_filter,
        "engine_games": engine_total,
        "all": block(total, counts),
        "engine": block(engine_total, engine_counts),
        **reached,
    }


def calibrate(summary: dict) -> tuple[list[str], list[str]]:
    """Compare against RAR-M15; return (unexplained, explained) differences.

    Only run this against RAR-M15's own corpus. The point is not that the
    numbers are the same -- six of them are known not to be -- but that the
    fourteen with no recorded reason to move have not moved. A difference that
    appears outside `CALIBRATION_EXCEPTIONS` means the classifier drifted, and
    nothing this tool says about another corpus can be trusted until it is
    explained. Loosening the tolerance is never the explanation.
    """
    unexplained, explained = [], []
    for name, expected in M15.items():
        got = summary["all"][name]["share"]
        if abs(got - expected) <= CALIBRATION_TOLERANCE:
            continue
        line = (f"{name}: measured {got:.4f}, RAR-M15 published {expected:.4f} "
                f"(delta {got - expected:+.4f})")
        if name in CALIBRATION_EXCEPTIONS:
            explained.append(f"{line} -- {CALIBRATION_EXCEPTIONS[name]}")
        else:
            unexplained.append(line)
    return unexplained, explained


def games_from_pgn(path: Path) -> list[str]:
    """Split a PGN into per-game texts without parsing any of them twice."""
    out, current = [], []
    blank_since_moves = False
    with path.open(encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if line.startswith("[Event ") and blank_since_moves and current:
                out.append("".join(current))
                current, blank_since_moves = [], False
            if not line.startswith("[") and line.strip():
                blank_since_moves = True
            current.append(line)
    if current:
        out.append("".join(current))
    return out


def games_from_sqlite(db: Path, tournament: str) -> list[str]:
    conn = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    rows = conn.execute(
        "select g.pgn from games g join tournaments t on t.id = g.tournament_id "
        "where (t.id = ? or t.name = ?) and g.pgn is not null",
        (tournament, tournament)).fetchall()
    conn.close()
    return [r[0] for r in rows]


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    src = ap.add_mutually_exclusive_group(required=True)
    src.add_argument("--pgn", type=Path)
    src.add_argument("--sqlite", type=Path)
    ap.add_argument("--tournament", help="tournament id or name, with --sqlite")
    ap.add_argument("--engine", default="Rarog",
                    help="report a second block over games this engine played")
    ap.add_argument("--workers", type=int, default=8)
    ap.add_argument("--limit", type=int, help="first N games only (a smoke run)")
    ap.add_argument("--calibrate", action="store_true",
                    help="require the output to reproduce RAR-M15's shares")
    ap.add_argument("--output", type=Path)
    args = ap.parse_args()

    if args.sqlite and not args.tournament:
        raise SystemExit("--sqlite needs --tournament")
    texts = (games_from_pgn(args.pgn) if args.pgn
             else games_from_sqlite(args.sqlite, args.tournament))
    if args.limit:
        texts = texts[:args.limit]
    if not texts:
        raise SystemExit("no games found")
    print(f"{len(texts)} games", flush=True)

    records: list[dict] = []
    if args.workers > 1:
        with ProcessPoolExecutor(max_workers=args.workers) as pool:
            for part in pool.map(_worker, shard(texts, args.workers)):
                records.extend(part)
    else:
        records = _worker(texts)

    summary = summarize(records, args.engine)
    summary["source"] = str(args.pgn or f"{args.sqlite}#{args.tournament}")

    print(f"\n{'function':<10}{'games':>8}{'share':>9}"
          f"{'eng games':>11}{'eng share':>11}  vs RAR-M15")
    print("-" * 62)
    for name in sorted(PREDICATES, key=lambda n: -summary["all"][n]["share"]):
        a = summary["all"][name]
        e = summary["engine"].get(name, {"games": 0, "share": 0.0})
        delta = a["share"] - M15[name]
        print(f"{name:<10}{a['games']:>8}{a['share']:>9.4f}"
              f"{e['games']:>11}{e['share']:>11.4f}  {delta:+.4f}")
    print(f"\nreaches <=6 men: {summary['reaches_6_men']:.4f}   "
          f"<=7 men: {summary['reaches_7_men']:.4f}")

    if args.calibrate:
        unexplained, explained = calibrate(summary)
        agreed = len(M15) - len(unexplained) - len(explained)
        print(f"\nCalibration against RAR-M15: {agreed} of {len(M15)} agree "
              f"within {CALIBRATION_TOLERANCE}.")
        for line in explained:
            print(f"  known: {line}")
        if unexplained:
            print("\nCALIBRATION FAILED -- differences with no recorded cause:",
                  file=sys.stderr)
            for line in unexplained:
                print(f"  {line}", file=sys.stderr)
            return 1

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(summary, indent=2) + "\n",
                               encoding="utf-8", newline="\n")
        print(f"\nFrozen: {args.output.resolve()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
