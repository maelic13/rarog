# RAR-M01 — Early fixed-`movetime` gates were compared with the deployed clock path at `3+0.03`

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Early fixed-`movetime` gates were compared with the deployed clock path at `3+0.03`.

## Result / disposition

Fixed movetime manufactured false negatives; SPRT/SPSA moved to a unified clock TC.

## Conditional lesson and retry trigger

A test TC must exercise the same time-management semantics as deployment. Fixed movetime remains useful only for deterministic diagnostics.

## Source

legacy plan at `757e9a3^`
