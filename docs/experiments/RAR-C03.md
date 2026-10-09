# RAR-C03 — Multi-thread diagnostics reset/dumped inside helper-called root search and the diagnostic build had stopped …

Indexed under *7. Correctness and protocol lessons* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment or failure mode

Multi-thread diagnostics reset/dumped inside helper-called root search and the diagnostic build had stopped compiling.

## Disposition

Fixed; previous multi-thread counter history was declared unreliable.

## Conditional lesson / coverage

Telemetry must have one owner and a build/runtime canary before it can guide search decisions.

## Source

`CHANGELOG.md` 2.3.1; legacy plan
