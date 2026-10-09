# RAR-P05 — Pawn-cache enlargement from the profile audit

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Pawn-cache enlargement from the profile audit.

## Result / disposition

A 128× larger table gained about 1.1 hit-rate points but lost 4.5% NPS.

## Conditional lesson and retry trigger

Under that workload, lookup/memory cost dominated the small hit-rate gain. Any future cache size change needs both profile and strength evidence.

## Source

`analysis/archive/speed_profile_8_12c.md`
