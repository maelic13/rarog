# RAR-P01 — Phase-9 clean-code/build program, each step bench-identical and spot-checked

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-9 clean-code/build program, each step bench-identical and spot-checked.

## Result / disposition

End-to-end result was about −3.2% NPS, inferred around −2 to −3 Elo; infrastructure retained.

## Conditional lesson and retry trigger

On this host, several sub-noise regressions compounded. Every refactor program needs one pooled end-to-end NPS comparison against its starting point.

## Source

legacy plan at `757e9a3^`
