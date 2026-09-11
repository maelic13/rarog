#!/usr/bin/env python3
"""Reference-family occurrence in the search tree, split by ROOT (PLAN 4.11.5).

`analysis/endgame_search_occurrence_2026-09-03.md` measured how often each of
the twenty reference families is reached inside the search tree, using the
`--features diag` counters over the 40-position bench suite. That number has a
trap in it, and the trap is the suite itself.

**A census that includes endgame roots partly measures its own corpus.**
Basilisk's first reading of the same question said 47.9%; restricted to
non-endgame roots it was **zero**, the entire signal having come from the
suite's own endgame roots (BAS-E43, BAS-E49). A family reached only from a
position already in that family tells you nothing about whether a middlegame
search will ever arrive there -- which is the question that decides whether a
recognizer can pay.

So this reports BOTH numbers: over all roots, and over non-endgame roots only.
Neither is wrong; they answer different questions, and quoting one without the
other is what produced the error above.

The counters are dumped once per position, so attribution needs no extra run --
`bench_counters.py --per-position` retains the sequence, and bench order matches
`BENCH_FENS` order. The FENs are read from `src/bench.rs` rather than copied,
so the suite cannot drift away from its classification.

Occurrence PRIORITISES. It is never evidence of value, and a family absent from
middlegame trees can still occur inside four-man endgame trees -- which is how
Basilisk's KBNK term leaked into families its safety argument had excluded.

Requires a build with `--features diag`. Its own gate is bench identity with
diag off: 6,901,489 / EBF 2.458 both ways.

Example:

  cargo build --release --features diag
  python tools/diag/endgame_occurrence.py --exe target/release/rarog.exe
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench_counters  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
BENCH_RS = ROOT / "src" / "bench.rs"

# A root with this many men or fewer is an ENDGAME root.
#
# **There is no defensible single value, and picking one silently is how this
# measurement goes wrong.** At 7 the bench suite contains ZERO endgame roots and
# the census looks perfectly clean. At 10 it contains eight, and those eight
# produce 94% of every reference-family evaluation in the run. Same data, same
# counters, opposite conclusion.
#
# So the tool sweeps and prints the whole curve, and `--endgame-men` only
# chooses which threshold gets the detailed per-family table. A report that
# quoted one threshold would be quoting a choice, not a measurement.
ENDGAME_MEN = 7
SWEEP = (7, 8, 10, 12, 14, 16)


def bench_fens() -> list[str]:
    """The bench suite, read from source so it cannot drift from this tool."""
    text = BENCH_RS.read_text(encoding="utf-8")
    m = re.search(r"pub const BENCH_FENS: \[&str; (\d+)\] = \[(.*?)\n\];",
                  text, re.S)
    if not m:
        raise SystemExit(f"could not find BENCH_FENS in {BENCH_RS}")
    declared = int(m.group(1))
    fens = re.findall(r'"([^"]+)"', m.group(2))
    if len(fens) != declared:
        raise SystemExit(
            f"BENCH_FENS declares {declared} entries, parsed {len(fens)}; "
            "the parser and the source have diverged"
        )
    return fens


def men(fen: str) -> int:
    """Piece count from the board field of a FEN."""
    board = fen.split()[0]
    return sum(1 for ch in board if ch.isalpha())


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--exe", default="target/release/rarog.exe")
    ap.add_argument("--depth", type=int, default=13)
    ap.add_argument("--stride", type=int, default=1)
    ap.add_argument("--endgame-men", type=int, default=ENDGAME_MEN,
                    help="threshold for the DETAILED table only; the "
                         "sensitivity sweep is always reported")
    ap.add_argument("--output", type=Path)
    args = ap.parse_args()

    fens = bench_fens()
    counts = [men(f) for f in fens]
    endgame_roots = [i for i, n in enumerate(counts) if n <= args.endgame_men]
    middlegame_roots = [i for i, n in enumerate(counts) if n > args.endgame_men]

    nodes, totals, dumps, sequence = bench_counters.run_bench(
        args.exe, args.depth, args.stride, [])
    eg = {k: v for k, v in sequence.items() if k.startswith("eg_")}
    if not eg:
        raise SystemExit(
            "no eg_ counters in the dump; rebuild with --features diag"
        )
    for name, seq in eg.items():
        if len(seq) != len(fens):
            raise SystemExit(
                f"{name}: {len(seq)} dumps for {len(fens)} bench positions; "
                "the counter dump and the suite have diverged"
            )

    denom = "eg_classified"
    if denom not in eg:
        raise SystemExit(f"missing denominator counter {denom!r}")

    def over(rows):
        total = sum(eg[denom][i] for i in rows)
        return total, {k: sum(v[i] for i in rows)
                       for k, v in eg.items() if k != denom}

    all_total, all_fam = over(range(len(fens)))
    mid_total, mid_fam = over(middlegame_roots)

    # The sensitivity sweep is the headline, not an appendix.
    sweep = []
    for threshold in SWEEP:
        mids = [i for i, n in enumerate(counts) if n > threshold]
        total, fam = over(mids)
        only_endgame = sorted(k for k, v in fam.items()
                              if all_fam[k] > 0 and v == 0)
        sweep.append({
            "endgame_men": threshold,
            "endgame_roots": len(fens) - len(mids),
            "middlegame_evaluations": total,
            "share_of_evaluations_from_middlegame_roots":
                round(total / all_total, 6) if all_total else None,
            "families_reached_only_from_endgame_roots": only_endgame,
        })

    report = {
        "schema": "rarog-endgame-occurrence-v1",
        "layer": "occurrence",
        "layer_note": (
            "gates whether layers 1-3 can ever reach layer 4. PRIORITISES "
            "only; never evidence of value "
            "(analysis/endgame_measurement_layers.md)."
        ),
        "exe": str(Path(args.exe).resolve()),
        "depth": args.depth,
        "nodes": nodes,
        "endgame_men_threshold": args.endgame_men,
        "roots": {
            "total": len(fens),
            "endgame": len(endgame_roots),
            "middlegame": len(middlegame_roots),
            "endgame_indices": endgame_roots,
        },
        "evaluations": {"all_roots": all_total, "middlegame_roots": mid_total},
        "sensitivity": sweep,
        "families": {},
    }
    for name in sorted(mid_fam):
        a, m = all_fam[name], mid_fam[name]
        report["families"][name] = {
            "all_roots": a,
            "middlegame_roots": m,
            "share_all": round(a / all_total, 6) if all_total else None,
            "share_middlegame": round(m / mid_total, 6) if mid_total else None,
            "from_endgame_roots_only": a > 0 and m == 0,
        }

    print(f"bench {args.depth}, nodes {nodes}")
    print("\nSENSITIVITY TO THE ENDGAME-ROOT THRESHOLD -- read this first.")
    print(f"{'<= men':>8}{'eg roots':>10}{'mid evals':>12}{'share':>9}"
          f"  families only from endgame roots")
    print("-" * 78)
    for row in sweep:
        names = ", ".join(n.replace("eg_", "") for n
                          in row["families_reached_only_from_endgame_roots"])
        print(f"{row['endgame_men']:>8}{row['endgame_roots']:>10}"
              f"{row['middlegame_evaluations']:>12,}"
              f"{row['share_of_evaluations_from_middlegame_roots']:>9.4f}"
              f"  {names[:40]}")
    print()
    print(f"roots: {len(fens)} total, {len(endgame_roots)} endgame "
          f"(<= {args.endgame_men} men), {len(middlegame_roots)} middlegame")
    print(f"evaluations: {all_total:,} all roots, {mid_total:,} middlegame\n")
    print(f"{'family':<14}{'all roots':>12}{'middlegame':>12}"
          f"{'share all':>11}{'share mid':>11}")
    print("-" * 60)
    for name, f in sorted(report["families"].items(),
                          key=lambda kv: -kv[1]["all_roots"]):
        flag = "  <- endgame roots only" if f["from_endgame_roots_only"] else ""
        print(f"{name:<14}{f['all_roots']:>12,}{f['middlegame_roots']:>12,}"
              f"{f['share_all']:>11.5f}{f['share_middlegame']:>11.5f}{flag}")

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n",
                               encoding="utf-8", newline="\n")
        print(f"\nReport: {args.output.resolve()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
