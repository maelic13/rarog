# RAR-M02 — Historical unpinned fastchess runs were audited under explicit physical-core placement on the Ryzen 9 5950X

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Historical unpinned fastchess runs were audited under explicit physical-core placement on the Ryzen 9 5950X.

## Result / disposition

Real affinity/topology defects were found; the original +9.34 ± 8.20 null did not itself prove a fixed +9 Elo offset.

## Conditional lesson and retry trigger

On this Windows host, small unpinned results may be biased. Re-audit a borderline old verdict only if it affects a current decision.

## Source

legacy plan at `757e9a3^`
