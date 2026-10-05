# RAR-M05 — SPSA schedule audit: iteration/game units, PowerShell `$A`/`$a` collision and integer perturbation resolution

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

SPSA schedule audit: iteration/game units, PowerShell `$A`/`$a` collision and integer perturbation resolution.

## Result / disposition

Several schedule defects were repaired; old runs annealed faster than intended. Accepted bakes remain accepted because independent SPRTs passed.

## Conditional lesson and retry trigger

A plausible SPSA trajectory is not proof of a correct schedule. Assert every emitted derived constant and inspect coordinate observability over the full horizon.

## Source

legacy plan at `757e9a3^`
