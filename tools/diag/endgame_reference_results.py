#!/usr/bin/env python3
"""Freeze the attained reference result per family (PLAN 4.11.3).

**"Attained reference result", never "ceiling".** The number below is what ONE
engine managed on ONE cohort at ONE node budget. It is not a theoretical bound
and not an empirical one: Rarog may exceed it, and failing to equal it is not by
itself a rejection. Basilisk's version of this artifact was called
`attained_single_engine_ceiling`, was read downstream as an ACCEPTANCE TARGET,
and was wrong in seven families by 77 positions -- far too lenient in exactly
the families the next phase would work (BAS-E50). The name is load-bearing.

The PAIRED UNION is a stretch diagnostic only. It proves that each counted
position was converted by at least one engine. It does NOT prove that one engine
can convert the union, and it must never be quoted as a target.

**Validation before reproduction.** Two reports over different positions, budgets
or ply limits are not comparable, and an artifact that silently mixed them would
be worse than none -- that is RAR-E14's defect B, which is what these checks
exist to make impossible. Every check below fails closed.

Example:

  python tools/diag/endgame_reference_results.py \\
      --candidate tools/results/truth-v2-head/endgame-truth.json \\
      --reference tools/results/truth-v2-reference/endgame-truth.json \\
      --output tools/diag/endgame_reference_results_v1.json
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

TRUTH_SCHEMA = "rarog-endgame-truth-v2"

# Run conditions that must be identical for the two arms to be one measurement.
# The node budget is here because a reference result at a different budget is a
# different quantity, not a better or worse one (PLAN rule 12).
MATCHED_CONDITIONS = ("nodes_per_move", "max_plies", "positions_per_family",
                      "seed", "hash_mb")


def load(path: Path) -> dict:
    doc = json.loads(path.read_text(encoding="utf-8"))
    if doc.get("schema") != TRUTH_SCHEMA:
        raise SystemExit(f"{path}: schema {doc.get('schema')!r}, need {TRUTH_SCHEMA!r}")
    return doc


def validate(candidate: dict, reference: dict) -> None:
    """Refuse anything that would make the two arms different measurements."""
    for key in MATCHED_CONDITIONS:
        a, b = candidate.get(key), reference.get(key)
        if a != b:
            raise SystemExit(
                f"run condition {key!r} differs: candidate {a!r}, reference {b!r}. "
                "These are not two arms of one measurement."
            )
    if candidate["cohort"]["sha256"] != reference["cohort"]["sha256"]:
        raise SystemExit(
            "the two arms measured different position sets "
            f"({candidate['cohort']['sha256'][:12]} against "
            f"{reference['cohort']['sha256'][:12]}); RAR-E14 defect B"
        )
    families = set(candidate["families"])
    if families != set(reference["families"]):
        raise SystemExit("the two arms cover different families")
    for name in sorted(families):
        a, b = candidate["families"][name], reference["families"][name]
        if a["cohort_sha256"] != b["cohort_sha256"]:
            raise SystemExit(f"{name}: per-family cohort digests differ")
        if "positions" not in a or "positions" not in b:
            raise SystemExit(
                f"{name}: a paired matrix needs per-position records; re-run "
                "with --per-position"
            )
        # The theory verdict is a property of the POSITION, so the two arms must
        # agree on it exactly. If they do not, something other than the engine
        # differs and no comparison here is meaningful.
        ta = [(p["fen"], p["theory_wdl"]) for p in a["positions"]]
        tb = [(p["fen"], p["theory_wdl"]) for p in b["positions"]]
        if ta != tb:
            raise SystemExit(f"{name}: FEN/theory pairing differs between arms")


def family_result(candidate: dict, reference: dict) -> dict:
    """Per-family counts and the paired matrix over CLEAN WINS only."""
    both = cand_only = ref_only = neither = 0
    clean = 0
    for a, b in zip(candidate["positions"], reference["positions"]):
        if a["theory_wdl"] != 2:
            continue
        clean += 1
        ac = a["outcome"] == "mated"
        bc = b["outcome"] == "mated"
        if ac and bc:
            both += 1
        elif ac:
            cand_only += 1
        elif bc:
            ref_only += 1
        else:
            neither += 1
    return {
        "clean_wins": clean,
        "candidate_converted": both + cand_only,
        "attained_reference_result": both + ref_only,
        "paired": {
            "both": both, "candidate_only": cand_only,
            "reference_only": ref_only, "neither": neither,
        },
        "paired_union": both + cand_only + ref_only,
        "deficit_to_reference": (both + ref_only) - (both + cand_only),
    }


def build(candidate: dict, reference: dict) -> dict:
    families = {name: family_result(candidate["families"][name],
                                    reference["families"][name])
                for name in candidate["families"]}
    tot = {k: sum(f["paired"][k] for f in families.values())
           for k in ("both", "candidate_only", "reference_only", "neither")}
    clean = sum(f["clean_wins"] for f in families.values())
    return {
        "schema": "rarog-endgame-reference-results-v1",
        "layer": "3_conversion",
        "what_this_is": (
            "The conversion ONE reference engine attained on ONE cohort at ONE "
            "node budget. Not a theoretical bound and not an empirical one."
        ),
        "what_this_is_not": [
            "not a ceiling: the candidate may exceed it",
            "not an acceptance target: failing to equal it is not a rejection",
            "not transferable to another node budget or another cohort",
            "the paired union proves each position was converted by at least "
            "one engine, NOT that one engine can convert the union",
        ],
        "conditions": {k: candidate[k] for k in MATCHED_CONDITIONS},
        "cohort_sha256": candidate["cohort"]["sha256"],
        "candidate_engine": candidate["engine"],
        "reference_engine": reference["engine"],
        "totals": {
            "clean_wins": clean,
            "candidate_converted": tot["both"] + tot["candidate_only"],
            "attained_reference_result": tot["both"] + tot["reference_only"],
            "paired": tot,
            "paired_union": clean - tot["neither"],
            "hard_residue": tot["neither"],
        },
        "families": families,
    }


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--candidate", required=True, type=Path)
    ap.add_argument("--reference", required=True, type=Path)
    ap.add_argument("--output", type=Path)
    args = ap.parse_args()

    candidate, reference = load(args.candidate), load(args.reference)
    validate(candidate, reference)
    report = build(candidate, reference)

    t = report["totals"]
    width = max(len(n) for n in report["families"]) + 1
    print(f"{'family':<{width}}{'candidate':>11}{'reference':>11}{'deficit':>9}"
          f"{'neither':>9}")
    print("-" * (width + 40))
    for name, f in report["families"].items():
        print(f"{name:<{width}}{f['candidate_converted']:>5}/{f['clean_wins']:<5}"
              f"{f['attained_reference_result']:>5}/{f['clean_wins']:<5}"
              f"{f['deficit_to_reference']:>9}{f['paired']['neither']:>9}")
    print("-" * (width + 40))
    print(f"{'TOTAL':<{width}}{t['candidate_converted']:>5}/{t['clean_wins']:<5}"
          f"{t['attained_reference_result']:>5}/{t['clean_wins']:<5}"
          f"{t['attained_reference_result'] - t['candidate_converted']:>9}"
          f"{t['hard_residue']:>9}")
    print(f"\npaired union {t['paired_union']}/{t['clean_wins']} -- a stretch "
          "diagnostic, never a target")
    print(f"hard residue {t['hard_residue']} positions neither engine converted")

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n",
                               encoding="utf-8", newline="\n")
        print(f"\nFrozen: {args.output.resolve()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
