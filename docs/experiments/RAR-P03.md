# RAR-P03 — Post-SMP duplicate-compute/index-hoist cleanup

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Post-SMP duplicate-compute/index-hoist cleanup.

## Result / disposition

**Retained, +0.99% then +1.56% median NPS** in independent pooled passes; search fingerprint unchanged.

## Conditional lesson and retry trigger

Small speed gains became credible only through clean worktrees, pooled builds, self-pair calibration and interleaving.

## Source

legacy plan at `757e9a3^`
