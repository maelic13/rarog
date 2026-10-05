# RAR-S31 — Phase-4.3a **arm B**, `SingularTtDepthMargin=2` versus 3

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.3a **arm B**, `SingularTtDepthMargin=2` versus 3. Former-policy `[0,3]` nElo SPRT, `3+0.03`, 1T, 64 MB, paired UHO, one non-PGO tune binary both sides, repo `3eeea89`.

## Result / disposition

**H1 reached on the tune binary:** 31,822 games, Elo +3.35 ± 2.44, nElo +5.24 ± 3.82, W-D-L 8,318-15,493-8,011, Ptnml [617, 3817, 6815, 3966, 696], LOS 99.64%, LLR 2.96, zero time losses/crashes. **Parked, not accepted into the baseline:** default remains 3 under the later material-gain/final-PGO policy.

## Conditional lesson and retry trigger

This establishes only that margin 2 outperformed margin 3 under the tune conditions. It does not isolate speculative evidence: margin 2 also excludes legitimate full-search entries at `depth-3`, and older/deeper ProbCut entries can still qualify at shallower consumers. Retain value 2 as an inert 4.10 coordinate/ablation; explicit persisted provenance owns the producer question.

## Source

`tools/results/sprt_SingMargin2_vs_Head_20260806_172117.*`; Plan 4.3a–b
