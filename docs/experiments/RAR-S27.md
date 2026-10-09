# RAR-S27 — Phase-4.3a **arm A**, `EvalPruneTtMinDepth=2` versus the seeded 0

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.3a **arm A**, `EvalPruneTtMinDepth=2` versus the seeded 0. Registered `[0,3]` nElo SPRT, `3+0.03`, 1T, 64 MB, paired UHO, concurrency 14 with `-use-affinity`, one `rarog-43a-tune.exe` both sides differing only by the option (SHA `559E0522…`), seed 1246079384.

## Result / disposition

**Rejected by the registered acceptance rule after a manual stop at 23,044 games:** Elo −1.49 ± 2.87, nElo −2.33 ± 4.49, W-D-L 5,815-11,315-5,914, Ptnml [475, 2891, 4880, 2810, 466], LOS 15.44%, LLR −2.19 of ±2.94. It did not reach the H0 boundary, but H1 was unreachable (LLR would need +5.13 of travel against the drift), and under "H1 accepts, otherwise revert" the stop and a formal H0 imply the identical action. Default stays 0. **The interval includes zero — this establishes "not a ≥3 nElo gain", NOT a measured loss.**

## Conditional lesson and retry trigger

Two lessons. (a) The candidate searched **43.8% fewer nodes for no measurable Elo**, so the depth-0/1 refinement path is doing a large amount of work of near-zero value — a strong signal that something is winnable there, but not via a flat depth floor. (b) The setting **overshot its own hypothesis**: the grievance is that depth-0 *qsearch* bounds refine deep pruning, but `=2` also denies depth-1 real searches, and the node split (−15.3% at 1, −43.8% at 2) shows depth-1 entries carry nearly twice the tree effect of depth-0. So this run is contaminated by a change the hypothesis never asked for and does **not** condemn the targeted `=1` setting. Retry `=1` before concluding anything about the mechanism; the project's own "non-monotone ≠ converged" lesson applies directly. Also note the LLR stalled for ~7,700 games near −2.2, so whole-run drift extrapolation is invalid for this design.

## Source

`tools/results/sprt_EvalPrune2_vs_Head_20260806_092417.*`; Plan 4.3a
