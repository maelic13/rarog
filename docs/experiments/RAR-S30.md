# RAR-S30 — Phase-4.3a refinement shadow at `8822cf2`, sampled 1/1024 over `bench 13`, 1T diagnostic build, no behaviour …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.3a refinement shadow at `8822cf2`, sampled 1/1024 over `bench 13`, 1T diagnostic build, no behaviour change (fingerprint 6,502,902).

## Result / disposition

**Observation with important scope limits.** Across 323 sampled nodes where refinement moved the eval, independent predicate checks counted RFP flips 37/2, NMP 36/4 and razor 0/13. These are not unique nodes: predicates can overlap, and the shadow evaluates later predicates even where real control flow would already have returned. The 64 completed-node tail comparison favored the refined value 45 versus 18 against the score reported by the same search. It excludes pruned nodes and uses an endogenous target.

## Conditional lesson and retry trigger

The data show directional predicate sensitivity in this bench; they do **not** establish that 28.5% of unique nodes changed, causally explain RAR-S29, or prove the refined value is a genuinely better estimator. Use the counters to design controlled consumer ablations in 4.10, not to assign Elo or justify provenance policy.

## Source

`tools/diag_search_quality.ps1`; `8822cf2`; Plan 4.3, 4.10
