#!/usr/bin/env python3
"""Aggregate endgame floors: the ratchet half of the 4.9a.3 contract.

The hard vetoes live in `tests/endgames.rs` and are absolute -- a won position
may not score as drawn, a drawn position may not be claimed as forced mate.
This file owns the other half, which is deliberately NOT absolute: family
conversion, win-preserving and DTZ-progress rates are statistics, and the
audit's policy is that they are aggregate floors rather than per-position
vetoes. A candidate may move an individual position; it may not move the
family averages down.

WHY THE FIRST DESIGN FAILED, since it failed usefully. Version one compared
every family and metric against a flat 5-point tolerance and failed the run on
any breach. That is 19 families x 3 metrics = **57 one-sided comparisons**, each
at roughly 1.5-2 standard errors, so breaches by chance were not merely
possible but expected -- and on RAR-E08 it produced four, of which
re-measurement at n=400 showed three were sampling. One of the false positives
was KBN-K reading -5.1 pp, in the family 4.9a.4 had just taken from 19.4% to
96.9%. An instrument that cries wolf on the thing you just fixed will be
ignored, which is worse than having no instrument.

WHAT REPLACED IT. Two tiers, matching what the policy actually says and what a
human reviewer actually did with the numbers:

  FAIL   the weighted aggregate conversion regressing beyond 2 SE. It pools
         every family, so its n is large and it is genuinely sensitive to a
         real broad regression. Or any single family beyond `--sigma` (default
         3) SE, which is a breach too large to be sampling.
  REPORT any single family beyond 2 SE. Recorded, owned, and NOT blocking --
         which is exactly how KQ-KP's real -3.8 pp regression was handled:
         assigned to 4.9a.14 with a retry trigger, not used to veto a +6.73 Elo
         gain.

Tolerances are computed from the actual denominators rather than assumed:
conversion uses `theoretically_won`, win-preserving uses `graded_moves`, and
DTZ progress uses `dtz_checked_moves`. The floors file therefore stores each
rate WITH the n it was measured at, so the standard error of the difference is
computable on both sides.

Keep DTZ progress. On RAR-E08 the floors flagged KQ-KP on DTZ progress and a
later conversion measurement at n=400 confirmed a real regression there, while
the conversion flag on KBN-K was false. It was the leading indicator.

COHORT IDENTITY (4.10.2). Floors and report must describe the SAME positions.
The floors file stores the per-family SHA-256 of the position set it was
measured on, and a comparison across differing digests is refused rather than
reported. Per family, not per run, so a single-family re-run still works -- the
family seed derives from the family name so a subset reproduces the full run.

Usage:

  python tools/diag/endgame_floors.py --report <endgame-truth.json>
  python tools/diag/endgame_floors.py --report <endgame-truth.json> --update
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

DEFAULT_FLOORS = Path(__file__).with_name("endgame_floors.json")

# metric -> the field holding the number of observations behind it.
METRICS = {
    "conversion_rate": "theoretically_won",
    "win_preserving_rate": "graded_moves",
    "dtz_progress_rate": "dtz_checked_moves",
}
REPORT_SIGMA = 2.0

# THIN-SAMPLE REFUSAL (PLAN 4.10.4). Below this many observations a rate is not
# reported as a number at all -- it is reported as thin.
#
# The failure this prevents is not a wrong verdict, it is a CONFIDENT one. A
# family with one eligible position that fails reads as 0.0%, which looks like
# catastrophe and is actually emptiness; the standard error of the difference
# on n=1 is ~0.5, so nothing could ever breach a 3-SE floor there either, and
# the family would sit in the report looking measured while being unmeasurable.
# Basilisk's first design left a control family with ONE eligible position of
# 24 and a silent 0.0%.
#
# 5 is chosen against the cohort rather than by taste: at 100 positions per
# family the smallest theoretical-win counts on the frozen set are KNN-K (1)
# and KNN-KP (23), so 5 excludes the degenerate family and keeps every real
# one. The number is stated here so it can be argued with.
MIN_ELIGIBLE = 5


# Floors and report must have been measured by the same instrument. A v1 report
# was produced by the harness that aborted correct pawn technique (RAR-E14), so
# its conversion rates are not the same quantity as a v2 report's and comparing
# them would manufacture a large fake improvement in exactly the pawn families
# 4.12 is about. Fail closed rather than mix.
TRUTH_SCHEMA = "rarog-endgame-truth-v2"


def load_report(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    got = data.get("schema")
    if got != TRUTH_SCHEMA:
        extra = ""
        if got == "rarog-endgame-truth-v1":
            extra = (
                " -- v1 was produced by the pre-4.10.1 harness, whose material "
                "abort ended correct pawn technique. Re-run it; do not "
                "re-analyse it."
            )
        raise SystemExit(f"{path}: schema {got!r}, need {TRUTH_SCHEMA!r}{extra}")
    return data


def cohorts(report: dict) -> dict[str, str]:
    """family -> the SHA-256 of its position set.

    A missing digest is an error rather than a shrug: it means the report came
    from a harness that could not identify its own position set, which is the
    condition RAR-E14 defect B describes.
    """
    out = {}
    for name, entry in report["families"].items():
        digest = entry.get("cohort_sha256")
        if not digest:
            raise SystemExit(
                f"family {name} carries no cohort_sha256; the report predates "
                "4.10.2 and its position set cannot be identified. Re-run it."
            )
        out[name] = digest
    return out


def check_cohorts(floors_doc: dict, report: dict) -> None:
    """Refuse to compare two runs over different positions.

    Comparison is PER FAMILY on purpose. A single-family re-run is a legitimate
    and useful thing to do -- the family seed is derived from the family NAME
    precisely so a subset reproduces the full run's positions -- so requiring
    the overall cohort id to match would forbid it for no reason. What must
    never happen is comparing KRP-KR measured on one position set against
    KRP-KR measured on another, which is what produced the "52% versus 47.9%"
    claim in 4.9a.7 from two artifacts sharing zero of 1,900 positions.
    """
    want = floors_doc.get("cohort", {}).get("family_sha256")
    if not want:
        raise SystemExit(
            "the floors file records no per-family cohort digests; it predates "
            "4.10.2 and cannot be shown to describe the same positions as this "
            "report. Re-derive it: PLAN step 4.11.2."
        )
    got = cohorts(report)
    mismatched = sorted(
        name for name, digest in got.items()
        if name in want and want[name] != digest
    )
    if mismatched:
        detail = ", ".join(
            f"{name} floors {want[name][:12]} != report {got[name][:12]}"
            for name in mismatched
        )
        raise SystemExit(
            f"cohort mismatch in {len(mismatched)} family/families: {detail}. "
            "These are different position sets, so their rates are not "
            "comparable and no ratchet or verdict may be taken from them "
            "(RAR-E14 defect B)."
        )


def rates(report: dict) -> dict[str, dict[str, dict]]:
    """family -> metric -> {rate, n}. Thin samples are dropped, not reported."""
    out: dict[str, dict[str, dict]] = {}
    for name, entry in report["families"].items():
        vals = {}
        for metric, n_field in METRICS.items():
            value = entry.get(metric)
            n = entry.get(n_field)
            if value is not None and n and int(n) >= MIN_ELIGIBLE:
                vals[metric] = {"rate": float(value), "n": int(n)}
        if vals:
            out[name] = vals
    return out


def thin(report: dict) -> list[tuple[str, str, int]]:
    """The (family, metric, n) triples suppressed as too thin to report."""
    out = []
    for name, entry in report["families"].items():
        for metric, n_field in METRICS.items():
            value = entry.get(metric)
            n = entry.get(n_field)
            if value is not None and n and int(n) < MIN_ELIGIBLE:
                out.append((name, metric, int(n)))
    return sorted(out)


def se_diff(p0: float, n0: int, p1: float, n1: int) -> float:
    """Standard error of the difference of two independent proportions.

    Independence is an approximation: the two runs share positions, so the
    paired SE is smaller and this is conservative -- it under-reports
    significance rather than over-reporting it.
    """
    v0 = max(p0 * (1.0 - p0), 1e-9) / max(n0, 1)
    v1 = max(p1 * (1.0 - p1), 1e-9) / max(n1, 1)
    return math.sqrt(v0 + v1)


def weighted_conversion(vals: dict[str, dict[str, dict]]) -> tuple[float, int]:
    num = den = 0
    for metrics in vals.values():
        c = metrics.get("conversion_rate")
        if c:
            num += c["rate"] * c["n"]
            den += c["n"]
    return (num / den if den else 0.0), den


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--report", required=True, type=Path)
    ap.add_argument("--floors", type=Path, default=DEFAULT_FLOORS)
    ap.add_argument("--sigma", type=float, default=3.0,
                    help="per-family failure threshold in standard errors "
                         "(default 3; a breach smaller than this is reported, "
                         "not failed)")
    ap.add_argument("--update", action="store_true",
                    help="rewrite the floors from this report (ratchet up)")
    ap.add_argument("--allow-lower", action="store_true",
                    help="permit --update to LOWER EVERY floor to this report; "
                         "blunt, and usually wrong -- prefer --allow-lower-family")
    ap.add_argument("--allow-lower-family", action="append", default=[],
                    metavar="FAMILY.METRIC",
                    help="permit --update to lower ONE named floor, e.g. "
                         "KBN-K.dtz_progress_rate. Repeatable. Requires its own "
                         "commit and a recorded reason. This is the option to "
                         "use when a regression has been accepted with an owner")
    args = ap.parse_args()

    report_doc = load_report(args.report)
    current = rates(report_doc)
    current_cohort = {
        "seed": report_doc.get("cohort", {}).get("seed"),
        "positions_per_family": report_doc.get("cohort", {}).get(
            "positions_per_family"),
        "sha256": report_doc.get("cohort", {}).get("sha256"),
        "family_sha256": cohorts(report_doc),
    }

    if args.update and not args.floors.is_file():
        args.floors.write_text(
            json.dumps({"schema": "rarog-endgame-floors-v2",
                        "truth_schema": TRUTH_SCHEMA,
                        "cohort": current_cohort,
                        "families": current},
                       indent=2, sort_keys=True) + "\n",
            encoding="utf-8", newline="\n")
        print(f"created {args.floors} from {args.report}")
        return 0
    if not args.floors.is_file():
        raise SystemExit(f"no floors file at {args.floors}; create it with --update")

    doc = json.loads(args.floors.read_text(encoding="utf-8"))
    if doc.get("schema") != "rarog-endgame-floors-v2":
        raise SystemExit(
            f"{args.floors} is schema {doc.get('schema')!r}; v2 stores each rate "
            "with the n it was measured at. Regenerate it with --update."
        )
    # The floors file records which truth schema produced it, so a floors file
    # built from the defective harness cannot be compared against a corrected
    # run. The committed floors predate 4.10.1 and carry no such key, so they
    # fail here until 4.11.2 re-derives them -- which is the point.
    if doc.get("truth_schema") != TRUTH_SCHEMA:
        raise SystemExit(
            f"{args.floors} was measured by {doc.get('truth_schema')!r}, not "
            f"{TRUTH_SCHEMA!r}. Floors derived from the pre-4.10.1 harness are "
            "depressed in every pawn family and are superseded (RAR-E14). "
            "Re-derive them from a corrected head run: PLAN step 4.11.2."
        )
    check_cohorts(doc, report_doc)
    floors = doc["families"]

    failures, reports, missing, improved = [], [], [], []
    for family, want in sorted(floors.items()):
        if family not in current:
            missing.append(family)
            continue
        for metric, base in want.items():
            got = current[family].get(metric)
            if got is None:
                missing.append(f"{family}.{metric}")
                continue
            se = se_diff(base["rate"], base["n"], got["rate"], got["n"])
            delta = got["rate"] - base["rate"]
            sigmas = delta / se if se > 0 else 0.0
            row = (family, metric, base["rate"], got["rate"], delta, sigmas)
            if sigmas <= -args.sigma:
                failures.append(row)
            elif sigmas <= -REPORT_SIGMA:
                reports.append(row)
            elif sigmas >= REPORT_SIGMA:
                improved.append(row)

    base_agg, base_n = weighted_conversion(floors)
    got_agg, got_n = weighted_conversion(current)
    agg_se = se_diff(base_agg, base_n, got_agg, got_n)
    agg_delta = got_agg - base_agg
    agg_sigmas = agg_delta / agg_se if agg_se > 0 else 0.0
    agg_failed = agg_sigmas <= -REPORT_SIGMA

    def show(rows, title):
        if not rows:
            return
        print(f"{title}:")
        for f, m, b, g, d, s in rows:
            print(f"  {f:<10} {m:<20} {b:.4f} -> {g:.4f}  ({d:+.4f}, {s:+.1f} SE)")
        print()

    suppressed = thin(report_doc)
    if suppressed:
        print(f"thin samples (n < {MIN_ELIGIBLE}), reported as empty rather "
              f"than as a rate:")
        for family, metric, n in suppressed:
            print(f"  {family:<10} {metric:<20} n={n}")
        print()

    print("layer    : 1-3 (truth, move quality, conversion). NOT strength -- "
          "see analysis/endgame_measurement_layers.md")
    print(f"nodes    : {report_doc.get('nodes_per_move')} per move, "
          f"max_plies {report_doc.get('max_plies')}")
    print(f"cohort   : {report_doc.get('cohort', {}).get('sha256', '?')[:16]}")
    print(f"floors   : {args.floors}")
    print(f"report   : {args.report}")
    print(f"aggregate: weighted conversion {base_agg:.4f} -> {got_agg:.4f} "
          f"({agg_delta:+.4f}, {agg_sigmas:+.1f} SE over n={got_n})")
    print()
    show(improved, "Improved beyond 2 SE (ratchet candidates)")
    show(reports, f"Below floor by 2-{args.sigma:g} SE -- REPORTED, not blocking "
                  "(assign an owner)")
    show(failures, f"BELOW FLOOR by more than {args.sigma:g} SE -- BLOCKING")
    if missing:
        print("MISSING from the report (a floor with no measurement is not a pass):")
        for item in missing:
            print(f"  {item}")
        print()

    if args.update:
        # A floor is the best VERIFIED level, so the default is to keep the
        # higher of old and new. `--allow-lower` drops that for everything at
        # once, which on a real report lowers a dozen floors that merely
        # sampled low -- it discards the ratchet in order to accept one
        # regression. `--allow-lower-family FAMILY.METRIC` lowers exactly what
        # is named and nothing else.
        named_lower = set(args.allow_lower_family)
        unknown = named_lower - {
            f"{fam}.{m}" for fam, vals in current.items() for m in vals
        }
        if unknown:
            raise SystemExit(
                "--allow-lower-family names entries not in this report: "
                + ", ".join(sorted(unknown))
            )
        if (failures or agg_failed) and not (args.allow_lower or named_lower):
            print("REFUSED: this report fails a floor, so --update would LOWER "
                  "it. Pass --allow-lower only with a recorded reason, in its "
                  "own commit.")
            return 1
        merged = {}
        for family, vals in current.items():
            prev = floors.get(family, {})
            merged[family] = {}
            for m, v in vals.items():
                old = prev.get(m)
                may_lower = args.allow_lower or f"{family}.{m}" in named_lower
                keep_old = (
                    old is not None
                    and not may_lower
                    and old["rate"] > v["rate"]
                )
                merged[family][m] = dict(old) if keep_old else dict(v)
        args.floors.write_text(
            json.dumps({"schema": "rarog-endgame-floors-v2",
                        "truth_schema": TRUTH_SCHEMA,
                        "cohort": current_cohort,
                        "families": merged},
                       indent=2, sort_keys=True) + "\n",
            encoding="utf-8", newline="\n")
        print(f"floors updated from {args.report}")
        return 0

    if failures or agg_failed or missing:
        why = []
        if agg_failed:
            why.append(f"aggregate {agg_sigmas:+.1f} SE")
        if failures:
            why.append(f"{len(failures)} family breach(es) beyond {args.sigma:g} SE")
        if missing:
            why.append(f"{len(missing)} missing")
        print("FAIL: " + ", ".join(why))
        return 1
    print(f"PASS ({len(reports)} reported below 2 SE, not blocking)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
