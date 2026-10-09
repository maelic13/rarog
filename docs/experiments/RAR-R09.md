# RAR-R09 — Phase-4.9c: implement per-thread ITERATION STAGGERING — the depth-diversity mechanism RAR-R08 showed is …

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.9c: implement per-thread ITERATION STAGGERING — the depth-diversity mechanism RAR-R08 showed is absent — behind `SmpIterationSkip`, then try to size it locally. Classic Lazy-SMP skip tables, helpers only, main thread never skips. Same binary both arms, option toggled, so there is zero build variance.

## Result / disposition

**Landed inert and correct; LOCAL SIZING IS IMPOSSIBLE, and a null pair proves it rather than asserting it.** Bench 6,502,902 unchanged (inert at 1T twice over: no shared state, and thread 0 never skips). A property test pins that main never skips, no helper skips everything or nothing, and helpers disagree at every depth. Then the sizing failed, in both metrics. Depth at fixed `movetime 5000`, n=2 per cell: **−0.5 / 0.0 / 0.0 plies** at 4/8/16T on an integer metric whose own spread is ±1–2. Time-to-depth(20), n=6 per cell: ratios **1.33 / 0.63 / 1.20** — no coherent direction. **The null pair at 8T, both arms `skip=0`, returned medians 742 versus 902 ms, a 21.6% swing from nothing**, with 1207/1997/1531 ms inside a single cell. Every skip ratio sits inside that. NPS rose +3.6% at 8T and +4.6% at 16T with skipping on, but NPS is not what this mechanism is for.

## Conditional lesson and retry trigger

**Run the null BEFORE believing the arm, not after.** Without it this would have been written up as '+37% time-to-depth at 8T' and '−33% at 4T' — both pure noise, and the first is exactly the sort of number that gets a mechanism adopted. The metric is unusable at this rep count: SMP time-to-depth carries ≥20% variance at n=6, so resolving a 5% effect needs order-100 samples per cell, and depth-at-fixed-time is too coarse to resolve half a ply at all. ⚠ So there is a strong MECHANISTIC case (RAR-R08: 2.61 plies unrealized at 16T, no thread ever ahead) and **zero local evidence the fix captures any of it**. Diversification is also 0 for 2 in this engine (RAR-R06, 9.7.5(j) at −5.54). Spending a 4T/8T gate on it is a judgement call, not a formality — and a properly powered local run (order-40 reps per cell at a shallower depth) is far cheaper than the gate and should come first.

## Source

`src/search.rs`; `src/params.rs`; Plan 4.9c
