# RAR-S47 — Phase-4.7b: one `RootConfidence` snapshot per COMPLETED root iteration, consumed by time management and by …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.7b: one `RootConfidence` snapshot per COMPLETED root iteration, consumed by time management and by aspiration behind separate switches, with worker instability pooled for TIME only. Measured on `bench 13`, 1T diag+tune build, 520 completed iterations; the 4T pooling probe is one `movetime 3000` reading per arm on the Kiwipete-style position.

## Result / disposition

**Landed inert and bench-identical (6,502,902 / EBF 2.449 on normal, diag and tune); three of the model's own inputs were measured and two of them changed a decision.** The scalar DISCRIMINATES - mean 400.7 per mille, quartiles 155/173/180/12 - so it is not a constant wearing a model's clothes. **SEPARATION is degenerate and ships weighted OUT:** the root gap is exactly 0 on **428 of 520** iterations (82.3%), and only **12** of those are 'no rival searched' - the other 416 are a rival scoring exactly level, because every root move but the best is searched on a null window and a fail-low reports the WINDOW, not a value. **EFFORT is sparse: the SHIPPED clock's effort factor sits at its endpoint on 473 of 520 iterations (91.0%)** - it is a constant, not a function, on this corpus. Steadiness (94/136/264/26 across octave buckets) and window (297 of 520 take a re-search) are well populated. `RootConfTime` is bench-invisible by construction and was sized by a TM SHADOW instead: reusing the effort endpoints made it an **8.85% budget CUT** (shorter on 507 of 520), so it was given its own endpoints seeded to measured level-neutrality - **+0.09%** total, longer on 295 and shorter on 182, i.e. redistribution. `RootConfAspiration` measures **6,699,671 nodes, +3.03%, EBF 2.449 -> 2.455**, cutting aspiration re-searches **917 -> 552 (-39.8%)**; two cheaper-looking seeds are WORSE (100/50 = +6.27%, 100/50 without the fail bump = +8.94%). At 4T the pooled channel carries a real population and reads **~0.81x** the own-thread instability in both probes.

## Conditional lesson and retry trigger

**Three transferable findings.** (1) The gap's promising mean is a PVS-manufactured degenerate population. (2) Reusing the old effort endpoints silently converted a shape change into a 9% time cut. (3) Every aspiration variant costs nodes, consistent with two prior losses. Final selection fixes only `RootConfTime` ON, tunes its six identifiable consumers, leaves aspiration to Phase 7.3 and pooled instability to Phase 8.0, and marks the root-gap path for removal after 2.4.

## Source

`src/search.rs`; `src/search_threads.rs`; `src/diag.rs`; `src/params.rs`; Plan 4.7b
