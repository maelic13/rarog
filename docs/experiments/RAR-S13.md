# RAR-S13 — `cutoffCnt` plus full LMR-family SPSA

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

`cutoffCnt` plus full LMR-family SPSA.

## Result / disposition

**Rejected, −7.78 ± 8.00.** Candidate searched about 16% more aggressively and won its tuning self-play before losing to the accepted head.

## Conditional lesson and retry trigger

A tuner can select a sibling-local optimum. Future Plan-4.6/4.10 coordinates must gate against the accepted head and receive post-fit ablations.

## Source

legacy plan at `757e9a3^`
