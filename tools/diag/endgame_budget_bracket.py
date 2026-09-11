#!/usr/bin/env python3
"""Repeat a family verdict across a bracket of node budgets (PLAN 4.10.6).

One budget is a guess. A verdict that appears only at a low budget is
PROVISIONAL, because the losing move it turns on may be a tactic a real search
sees: Basilisk rejected its leading KBNK candidate at 60,000 nodes on exactly
that, and the rejection did not reproduce at 200,000 or 600,000 (BAS-E45).

So this runs `endgame_truth.py` unchanged at each budget over the SAME cohort
and tabulates the result, rather than inventing a second measurement path. Each
run writes its own single-budget report -- one report, one budget, per the
layer contract -- and this tool only compares them.

What it cannot tell you: whether the deployment budget is inside the bracket.
That is `nodes_per_move.py`'s job, and the bracket should be chosen to straddle
its median.

Example:

  python tools/diag/endgame_budget_bracket.py \\
      --engine target/release/rarog.exe \\
      --syzygy D:/chess/tablebases/syzygy3456 \\
      --families KBN-K,KRP-KR --positions 24 \\
      --budgets 60000,200000,600000 --workers 12 \\
      --out-dir tools/results/budget-bracket
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

RUNNER = Path(__file__).with_name("endgame_truth.py")


def parse_budgets(spec: str) -> list[int]:
    budgets = []
    for part in spec.split(","):
        part = part.strip()
        if not part:
            continue
        value = int(part)
        if value <= 0:
            raise ValueError(f"budget must be positive, got {value}")
        budgets.append(value)
    if not budgets:
        raise ValueError("no budgets given")
    if len(set(budgets)) != len(budgets):
        raise ValueError("duplicate budgets")
    return sorted(budgets)


def cohorts_agree(reports: dict[int, dict]) -> str | None:
    """Every arm must have measured the SAME positions.

    A bracket comparing different position sets would be defect B of RAR-E14
    with extra steps, so this fails closed rather than reporting a table.
    """
    digests = {b: r.get("cohort", {}).get("sha256") for b, r in reports.items()}
    if any(d is None for d in digests.values()):
        return "at least one report carries no cohort digest"
    unique = set(digests.values())
    if len(unique) != 1:
        detail = ", ".join(f"{b}: {d[:12]}" for b, d in sorted(digests.items()))
        return f"arms measured different position sets ({detail})"
    return None


def table(reports: dict[int, dict], families: list[str]) -> list[str]:
    budgets = sorted(reports)
    width = max(len(f) for f in families) + 1
    lines = ["", "conversion by node budget (layer 3; see "
                 "analysis/endgame_measurement_layers.md)", ""]
    header = f"{'family':<{width}}" + "".join(f"{b:>12,}" for b in budgets)
    lines.append(header)
    lines.append("-" * len(header))
    for family in families:
        row = f"{family:<{width}}"
        for budget in budgets:
            entry = reports[budget]["families"][family]
            won = entry["theoretically_won"]
            row += f"{entry['converted']:>6}/{won:<5}" if won else f"{'n/a':>12}"
        lines.append(row)
    lines.append("")
    lines.append("A verdict that does not survive the whole bracket is "
                 "PROVISIONAL and must say so.")
    return lines


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--engine", required=True, type=Path)
    ap.add_argument("--syzygy", required=True, type=Path)
    ap.add_argument("--families", required=True)
    ap.add_argument("--positions", type=int, default=24)
    ap.add_argument("--max-plies", type=int, default=100)
    ap.add_argument("--seed", type=int, default=0x5E9D18)
    ap.add_argument("--workers", type=int, default=1)
    ap.add_argument("--budgets", default="60000,200000,600000")
    ap.add_argument("--out-dir", required=True, type=Path)
    ap.add_argument("--reuse", action="store_true",
                    help="skip a budget whose report already exists")
    args = ap.parse_args()

    try:
        budgets = parse_budgets(args.budgets)
    except ValueError as exc:
        ap.error(str(exc))
    families = [f for f in args.families.split(",") if f]
    if not families:
        ap.error("no families given")
    args.out_dir.mkdir(parents=True, exist_ok=True)

    reports: dict[int, dict] = {}
    for budget in budgets:
        path = args.out_dir / f"truth-{budget}.json"
        if args.reuse and path.is_file():
            print(f"reusing {path}", flush=True)
        else:
            cmd = [
                sys.executable, str(RUNNER),
                "--engine", str(args.engine), "--syzygy", str(args.syzygy),
                "--families", ",".join(families),
                "--positions", str(args.positions),
                "--nodes", str(budget),
                "--max-plies", str(args.max_plies),
                "--seed", str(args.seed),
                "--workers", str(args.workers),
                "--per-position",
                "--output", str(path),
            ]
            print(f"\n=== {budget:,} nodes/move ===", flush=True)
            proc = subprocess.run(cmd)
            if proc.returncode != 0:
                raise SystemExit(
                    f"endgame_truth.py failed at {budget} nodes "
                    f"(exit {proc.returncode}); no table will be printed"
                )
        reports[budget] = json.loads(path.read_text(encoding="utf-8"))

    problem = cohorts_agree(reports)
    if problem:
        raise SystemExit(f"refusing to tabulate: {problem}")

    for line in table(reports, families):
        print(line)

    summary = args.out_dir / "bracket.json"
    summary.write_text(json.dumps({
        "schema": "rarog-endgame-budget-bracket-v1",
        "layer": "3_conversion",
        "budgets": budgets,
        "families": families,
        "cohort_sha256": reports[budgets[0]]["cohort"]["sha256"],
        "conversion": {
            str(b): {f: {"converted": reports[b]["families"][f]["converted"],
                         "theoretically_won":
                             reports[b]["families"][f]["theoretically_won"]}
                     for f in families}
            for b in budgets
        },
    }, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"Summary: {summary.resolve()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
