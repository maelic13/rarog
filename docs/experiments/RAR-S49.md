# RAR-S49 — Phase-4.9e: size the carried-in retry PLAN 4.9 reserved — RAR-S27's surviving hypothesis that …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.9e: size the carried-in retry PLAN 4.9 reserved — RAR-S27's surviving hypothesis that `EvalPruneTtMinDepth=2`'s 'materially smaller tree' relieves TT write pressure in a way 1T cannot test. Paired within-process A/B at 1/4/8T (`go depth 18`, 20 pairs x 2 positions), plus the deterministic `bench 13` fingerprint for each knob value on a tune build.

## Result / disposition

**THE PREMISE IS FALSE: the tree is BIGGER, not smaller, so the hypothesis is void and no games are owed.** Against the same 6,502,902 baseline, `EvalPruneTtMinDepth=1` measures **7,384,102 (+13.55%)** and `=2` measures **6,981,350 (+7.36%)**. The record claims **−15.3% and −43.8%** — inverted in sign, and not matching in magnitude either. The mechanism agrees with the measurement, not the record: the 4.3 refinement shadow on this same head counts refinement CAUSING 73 prunes against PREVENTING 19, so denying refinement prunes LESS and searches MORE. The paired A/B is consistent (1T time ratio 1.163, nodes 1.135) and inconclusive at 4/8T (CIs [0.77,1.10] and [0.84,1.10] straddle 1).

## Conditional lesson and retry trigger

**A retry can rest on a number nobody re-measured.** This hypothesis survived RAR-S27's rejection, was carried forward through 4.3, and was written into PLAN 4.9 as a reserved 4T/8T job — all resting on '43.8% smaller tree', a figure that reproduces with the opposite sign. There was never a smaller tree to relieve pressure, so the retry is CLOSED without spending a gate. ⚠ It also re-frames RAR-S27 itself: the honest reading is not 'we can cut 44% of the tree for free' but 'refinement buys a 7.4% smaller tree that is worth no measurable Elo'. **Re-measure a carried premise before building a job on it** — this is the fifth recorded claim this cycle that did not survive contact with its own instrument, after the multicut claim, the `Corr*Scale` seeds, the late-evasion predicate and the losing-check population.

## Source

`src/params.rs`; Plan 4.9e
