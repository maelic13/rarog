# RAR-M16 — What adjudication actually buys, measured from stored logs

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**What adjudication actually buys, measured from stored logs.** Games/minute taken from every stored gate log over 400 games with a recorded total time, split by the manifest's adjudication line; all 1T `3+0.03`, concurrency 14.

## Result / disposition

**Observation.** Adjudicated runs cluster tightly at **~97.5 games/min** (median over 27 runs, range 94.5-112.9 with the fastest being 2,000-game runs whose fixed startup cost is amortised differently). The one no-adjudication run, RAR-E06, measured **88.4 games/min**: about **9-10% slower**. On RAR-E06's own numbers that is 44.3 minutes instead of ~40, and an 8-hour gate becomes ~8h50m.

## Conditional lesson and retry trigger

**The price of playing games out is far lower than the cost of what adjudication removes, so the default should be no-adjudication and the burden should fall on keeping it, not dropping it.** Adjudication is not *unfair* -- it is symmetric between two arms of the same engine and RAR-M06 found one-sided and two-sided differed on 0.20% of triggers and no final result -- but it is **lossy**, and lossy in exactly the direction this project is now working. It removes conversion and defensive-holding skill from the measurement: RAR-M15 measured it destroying **52.7% of all endgames before they are reached**, and RAR-O01 vs RAR-O02 priced the cross-evaluator adjudication confounder at **74 Elo**. A candidate that is better precisely at converting won endgames is the candidate adjudication is blindest to. Caveat: n=1 for the no-adjudication throughput figure, so treat 9-10% as one measurement rather than a fitted constant. Flipping the harness default is a maintainer decision and introduces a before/after discontinuity of unknown size, unlike RAR-M13's measured 0.20%.

## Source

`tools/results/sprt_*.{log,manifest.txt}`; RAR-M06; RAR-M13; RAR-M15; RAR-O01/O02
