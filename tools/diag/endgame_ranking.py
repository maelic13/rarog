#!/usr/bin/env python3
"""Rank the twenty reference functions on corrected evidence (PLAN 4.11.6).

The pre-correction order was board occurrence times a conversion number the
RAR-E14 instrument defect depressed. All three inputs have since been
re-measured, and **they disagree with each other sharply** -- which is the point
of ranking on all of them rather than on whichever one was to hand.

RULES, fixed before the output was looked at:

1. **Rank by DEFECT SHAPE, not by the donor's taxonomy.** A family whose
   drawn-share bias is high is a SCALE problem whatever Stockfish called its
   function; one whose conversion deficit is high is a VERDICT problem. A family
   can be both, and is then ranked on the larger of its two normalised defects.
   4.9a.7 nearly declared a working scale change null by reading it on
   conversion; 4.11.4 found three families that would have been called healthy
   the same way.
2. **Occurrence gates, it does not score.** It cannot rescue a family with no
   defect and it cannot condemn one with a large defect -- it decides how much a
   given defect is worth fixing. Board occurrence is the primary gate because
   it is far better sampled than tree occurrence (40 bench positions, PLAN
   4.11.5); tree occurrence is carried alongside and flagged where the two
   contradict. Since 4.11.12 board occurrence is measured over the 36,400-game
   rating tournament -- 10,000 of them Rarog's own, against thirteen other
   engines -- instead of 3,915 self-play games of one engine pair.
3. **Layers are never aggregated.** Conversion deficit and drawn-share bias are
   NOT summed into one number. Each family gets both, normalised within its own
   measurement, and the rank uses the larger -- so a family cannot be promoted by
   being mediocre twice.
4. **Unverifiable last.** A family the local tablebases cannot adjudicate is
   ranked last whatever it scores, because its evidence cannot be produced.
5. **Thin measurements do not rank.** A family whose drawn subset was too small
   to report is ranked on conversion alone, and says so.
6. **A measured zero is not a certain zero -- and two of these were never
   zero.** RAR-M15 reported KQKR, KQKRPs and KRPPKRP in zero of 3,915 games and
   this ranking floored all three by the rule of three, treating the problem as
   sample size. PLAN 4.11.12 re-measured occurrence over 36,400 rated games and
   found **all three occur**: KRPPKRP in 4.97% of games, KQKR in 0.55%,
   KQKRPs in 0.39%. The floor stays -- it is still right that a sample which
   fails to contain something bounds rather than annihilates it -- but it is no
   longer load-bearing for any family here.
7. **Unmeasured is not unimportant.** Five reference functions have no cohort
   family in the six-man corpus at all. They are grouped as MEASURE FIRST and
   ordered by occurrence, between the ranked families and the unverifiable tail
   -- not at the bottom, which would confuse "we have no evidence" with "there
   is nothing here". KPsK is 4.5% of Rarog's games and has never been
   measured.

WHY TREE OCCURRENCE IS NOT THE MULTIPLIER. It is the more direct gate in
principle -- a scoring defect misguides the search wherever the evaluator is
called, whether or not the game reaches that ending. But PLAN 4.11.5 measured
that Rarog's tree-occurrence instrument is weak: three of forty bench roots
produce 56% of the whole census, and four families read zero over all forty.
Using it as a multiplier would give it more authority than that finding allows.
It is carried as a flag and the contradictions are named in the registration.
The retry trigger that asked for a better occurrence corpus has since been
DISCHARGED by 4.11.12; what remains open is tree occurrence itself.

The output is a PRIORITISER. It is not an Elo estimate and not an acceptance
target (`analysis/endgame_measurement_layers.md`).

Example:

  # the REGISTERED order (v2, PLAN 4.11.12)
  python tools/diag/endgame_ranking.py \\
      --board-occurrence tools/diag/endgame_board_occurrence_v1.json \\
      --occurrence-scope engine --output tools/diag/endgame_ranking_v2.json

  # v1, on RAR-M15's constants -- kept reproducible so the correction stays visible
  python tools/diag/endgame_ranking.py --output tools/diag/endgame_ranking_v1.json
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

DIAG = Path(__file__).resolve().parent

# Board occurrence is an INPUT, and since PLAN 4.11.12 it arrives as an
# artifact (`--board-occurrence`, schema `rarog-board-occurrence-v1`) rather
# than as constants. What remains here is RAR-M15's original table, kept as the
# fallback so `endgame_ranking_v1.json` still reproduces exactly -- a frozen
# artifact whose inputs cannot be reconstructed is not evidence.
#
# ⚠ RAR-M15's own numbers, not a replay of them. Four are known wrong: it
# capped positions at six men, so KRPPKRP (seven men) could not be seen at all
# and read zero, and KRPKR/KRPKB/KBPsK were counted with a plural strong side.
# `tools/diag/endgame_board_occurrence.py` documents each difference with the
# measurement that establishes it.
M15_BOARD_OCCURRENCE = {
    "KXK": 0.3734, "KRPKR": 0.1004, "KPsK": 0.0419, "KPK": 0.0284,
    "KRKP": 0.0240, "KBPsK": 0.0192, "KRPKB": 0.0123, "KPKP": 0.0123,
    "KQKP": 0.0117, "KBPKB": 0.0089, "KBPPKB": 0.0066, "KRKN": 0.0061,
    "KRKB": 0.0051, "KBPKN": 0.0028, "KBNK": 0.0028, "KNNKP": 0.0005,
    "KNNK": 0.0003, "KQKR": 0.0, "KQKRPs": 0.0, "KRPPKRP": 0.0,
}

# Reference function -> the truth/drawn cohort family that measures it. Several
# reference functions have no direct cohort family (KPsK and KBPsK are plural
# pawns; KQKRPs and KRPPKRP are outside the six-man cohort), and those are
# marked rather than given a borrowed number.
COHORT_FAMILY = {
    "KXK": "KQ-K", "KRPKR": "KRP-KR", "KPsK": None, "KPK": "KP-K",
    "KRKP": "KR-KP", "KBPsK": None, "KRPKB": "KRP-KB", "KPKP": "KP-KP",
    "KQKP": "KQ-KP", "KBPKB": "KBP-KB", "KBPPKB": None, "KRKN": "KR-KN",
    "KRKB": "KR-KB", "KBPKN": "KBP-KN", "KBNK": "KBN-K", "KNNKP": "KNN-KP",
    "KNNK": "KNN-K", "KQKR": "KQ-KR", "KQKRPs": None, "KRPPKRP": None,
}

# Seven men, and the local tables stop at six -- so its conversion and its
# drawn share cannot be adjudicated, and it is ranked last on that alone.
#
# ⚠ The OTHER half of the old reason is gone. This set used to read "reachable
# neither by sampling play nor by verified construction", on RAR-M15's zero.
# PLAN 4.11.12 measured KRPPKRP in **1,808 of 36,400 rated games (4.97%)**, the
# fourth most common family in the set, and exhibited a position from RAR-M15's
# OWN corpus (`8/1r3p2/8/7P/8/4kPK1/1R6/8 b - - 0 66`). It is reached constantly
# and cannot be measured, which is a far worse gap than a rare ending and must
# be recorded as one rather than filed at the bottom of a list.
UNVERIFIABLE = {"KRPPKRP"}

# Rule of three: zero events in n games bounds the rate at ~3/n at 95%, so a
# family the corpus merely failed to contain is floored rather than annihilated
# by a multiply-by-zero. Scaled to the corpus actually used.
DEFAULT_CORPUS_GAMES = 3915


def board_occurrence(path: Path | None, scope: str) -> tuple[dict, str, int]:
    """Occurrence shares, their provenance, and the corpus size behind them.

    Without `--board-occurrence` this returns RAR-M15's constants, so the v1
    artifact reproduces. With one, it reads a `rarog-board-occurrence-v1`
    document and takes either the whole pool (`all`) or only the games one
    engine played (`engine`).
    """
    if path is None:
        return dict(M15_BOARD_OCCURRENCE), "RAR-M15, 3,915 games", DEFAULT_CORPUS_GAMES
    doc = load(path)
    schema = doc.get("schema")
    if schema != "rarog-board-occurrence-v1":
        raise SystemExit(f"{path}: unexpected schema {schema!r}")
    block, games = ((doc["all"], doc["games"]) if scope == "all"
                    else (doc["engine"], doc["engine_games"]))
    if not block or not games:
        raise SystemExit(f"{path}: no games in the {scope!r} block")
    missing = set(M15_BOARD_OCCURRENCE) - set(block)
    if missing:
        raise SystemExit(f"{path}: missing families {sorted(missing)}")
    return ({name: block[name]["share"] for name in M15_BOARD_OCCURRENCE},
            f"{doc.get('source', path)} [{scope}], {games:,} games", games)


def load(path: Path) -> dict:
    if not path.is_file():
        raise SystemExit(f"missing input: {path}")
    return json.loads(path.read_text(encoding="utf-8"))


def build(reference: dict, drawn: dict, occurrence: dict,
          board_shares: dict, corpus_games: int) -> list[dict]:
    drawn_fams = drawn.get("families", drawn)
    tree = occurrence["families"]
    tree_total = occurrence["evaluations"]["all_roots"]

    floor = 3 / corpus_games
    rows = []
    for name, board in board_shares.items():
        cohort = COHORT_FAMILY[name]
        row = {
            "function": name,
            "cohort_family": cohort,
            "board_occurrence": board,
            "unverifiable": name in UNVERIFIABLE,
        }

        # Conversion deficit, as a share of the family's clean wins.
        fam = reference["families"].get(cohort) if cohort else None
        if fam and fam["clean_wins"]:
            row["clean_wins"] = fam["clean_wins"]
            row["conversion_deficit"] = fam["deficit_to_reference"]
            row["conversion_defect"] = max(
                0.0, fam["deficit_to_reference"] / fam["clean_wins"])
        else:
            row["conversion_defect"] = None

        # Drawn-share bias.
        d = drawn_fams.get(cohort) if cohort else None
        if d and d.get("overclaim_rate") is not None:
            row["drawn_n"] = d["drawn"]
            row["overclaim_rate"] = d["overclaim_rate"]
            row["overclaim_mean_cp"] = d["mean_cp"]
            row["scale_defect"] = d["overclaim_rate"]
        else:
            row["scale_defect"] = None
            row["drawn_thin"] = bool(d)

        key = f"eg_{name.lower()}"
        t = tree.get(key)
        row["tree_share"] = t["share_all"] if t else None
        row["tree_zero"] = bool(t and t["all_roots"] == 0)

        defects = [v for v in (row["conversion_defect"], row["scale_defect"])
                   if v is not None]
        row["defect"] = max(defects) if defects else None
        row["defect_kind"] = (
            None if row["defect"] is None
            else "scale" if row["defect"] == row.get("scale_defect")
            else "verdict"
        )
        gate = max(board, floor)
        row["occurrence_gate"] = round(gate, 6)
        row["priority"] = (
            None if row["defect"] is None else round(row["defect"] * gate, 6)
        )
        # Flagged, not silently reordered: the two occurrence instruments
        # disagree by more than an order of magnitude for these families, and
        # the disagreement belongs in the registration prose.
        row["occurrence_conflict"] = bool(
            row["tree_share"] is not None and (
                (board <= floor and row["tree_share"] > 0.01)
                or (board > 0.005 and row["tree_share"] == 0.0)
            )
        )
        rows.append(row)

    def sort_key(r):
        # Three groups: ranked, MEASURE FIRST (no cohort family exists), then
        # unverifiable. Within the middle group, by occurrence -- an unmeasured
        # family that occurs in 4% of games outranks one that occurs in 0.7%.
        if r["unverifiable"]:
            return (2, 0.0, 0.0)
        if r["priority"] is None:
            return (1, -r["board_occurrence"], -(r["tree_share"] or 0.0))
        return (0, -r["priority"], 0.0)

    rows.sort(key=sort_key)
    for i, r in enumerate(rows, start=1):
        r["rank"] = i
    return rows


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--reference",
                    default=DIAG / "endgame_reference_results_v1.json", type=Path)
    ap.add_argument("--drawn",
                    default=DIAG / "endgame_drawn_census_v1.json", type=Path)
    ap.add_argument("--occurrence",
                    default=DIAG / "endgame_tree_occurrence_v1.json", type=Path)
    ap.add_argument("--board-occurrence", type=Path,
                    help="a rarog-board-occurrence-v1 artifact; without it the "
                         "RAR-M15 constants are used and v1 reproduces")
    ap.add_argument("--occurrence-scope", choices=("all", "engine"), default="all",
                    help="whole pool, or only the games one engine played")
    ap.add_argument("--output", type=Path)
    args = ap.parse_args()

    shares, provenance, corpus_games = board_occurrence(
        args.board_occurrence, args.occurrence_scope)
    rows = build(load(args.reference), load(args.drawn), load(args.occurrence),
                 shares, corpus_games)
    print(f"board occurrence: {provenance}\n")

    print(f"{'#':>3} {'function':<9}{'kind':<9}{'defect':>8}{'board':>8}"
          f"{'priority':>10}{'tree':>9}  notes")
    print("-" * 78)
    for r in rows:
        notes = []
        if r["unverifiable"]:
            notes.append("7 men, UNVERIFIABLE")
        if r["defect"] is None:
            notes.append("no cohort measurement")
        if r.get("drawn_thin"):
            notes.append("drawn subset thin")
        if r["tree_zero"]:
            notes.append("tree occurrence ZERO")
        if r["occurrence_conflict"]:
            notes.append("BOARD/TREE CONFLICT")
        if r["defect"] == 0.0:
            notes.append("NO DEFECT MEASURED -- close it")
        print(f"{r['rank']:>3} {r['function']:<9}{r['defect_kind'] or '-':<9}"
              f"{(r['defect'] if r['defect'] is not None else float('nan')):>8.3f}"
              f"{r['board_occurrence']:>8.4f}"
              f"{(r['priority'] if r['priority'] is not None else float('nan')):>10.5f}"
              f"{(r['tree_share'] if r['tree_share'] is not None else float('nan')):>9.5f}"
              f"  {'; '.join(notes)}")

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps({
            "schema": "rarog-endgame-ranking-v1",
            "layer": "prioritiser",
            "layer_note": (
                "not an Elo estimate and not an acceptance target. Ranks how "
                "much a measured defect is worth fixing, nothing more."
            ),
            "inputs": {
                "reference_results": str(args.reference),
                "drawn_census": str(args.drawn),
                "occurrence": str(args.occurrence),
                "board_occurrence": provenance,
            },
            "order": [r["function"] for r in rows],
            "families": rows,
        }, indent=2) + "\n", encoding="utf-8", newline="\n")
        print(f"\nFrozen: {args.output.resolve()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
