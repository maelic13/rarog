# RAR-S26 — Phase-4.3a arm sizing: four registered knobs A–D, one `tune` binary, 4 positions at fixed depth …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.3a arm sizing: four registered knobs A–D, one `tune` binary, 4 positions at fixed depth 12 with a cleared table per position. Node counts and best moves only.

## Result / disposition

**Diagnostic, not a verdict.** Versus baseline (552,764 nodes): **A `EvalPruneTtMinDepth=1` −15.25%** and **`=2` −43.75%, both with identical best moves** on all four probes; **B `SingularTtDepthMargin=2` −11.80%**, moves differ; **C `QsRefineMinDepth=1` +5.28%**, moves differ; **D `ProbCutStoreDepthAdj=4` +18.19%**, moves differ. All four defaults are inert — `bench 13` = 6,502,902 / EBF 2.449 on normal, diag and tune builds.

## Conditional lesson and retry trigger

Arm A is the priority: a 44% node reduction with unchanged moves on the probe set suggests real waste in letting depth-0 bounds refine the pruning eval, and the knob's prior SPSA retained 0 while sitting ON a rail, where a fit is least informative. But four positions at one depth is very weak evidence and PLAN lesson 3 applies directly — a smaller tree can make worse decisions, so these are node counts to explain a gate, never to replace one. Arms C and D both COST nodes and must buy accuracy to be worth anything; C additionally guts RAR-S02's accepted mechanism. Registered gates pending; `[0,3]` per arm at `3+0.03`, 1T, no combination before individual verdicts.

## Source

`tools/test_engines/rarog-43a-tune.exe`; Plan 4.3
