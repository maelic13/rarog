# RAR-M03 — Identical-binary null testing after harness changes

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Identical-binary null testing after harness changes.

## Result / disposition

The old symmetric `[-3,+3]` setup had zero expected LLR drift at equality; current policy is fixed-N 30k at 1T, requiring the full 95% nElo CI inside ±5.

## Conditional lesson and retry trigger

Equivalence needs a calibration design, not an ordinary gain SPRT. Repeat after runner, scheduler or topology changes that can create arm asymmetry. Merely disabling both draw and resign adjudication symmetrically does not change placement or side assignment and does not consume another 30k null.

## Source

`PLAN.md` §2
