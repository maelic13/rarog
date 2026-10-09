# RAR-M10 — Calibrating this harness's LLR drift so a game budget can be derived rather than guessed

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Calibrating this harness's LLR drift so a game budget can be derived rather than guessed. Fitted on three completed `[0,3]` nElo gates at `3+0.03`, 1T, paired UHO, concurrency 14 with affinity: RAR-S31 (+5.24 nElo, LLR 2.96 in 31,822 games), RAR-S29 (−4.95, −2.95 in 18,436) and RAR-S27 (−2.33, −2.19 in 23,044).

## Result / disposition

**Retained method tool.** `drift per game ≈ 8.3e-6 × (Elo1 − Elo0) × (true_nElo − midpoint)` predicts all three observed drifts within 1% (9.30e-5 vs 9.30e-5; −1.61e-4 vs −1.60e-4; −9.54e-5 vs −9.50e-5). Applied to the `[3,10]` default it gives ~9,200 games for a true 12 nElo, **~14,500 for a candidate exactly on H1 (10 nElo)**, ~33,700 for 8 nElo, and ~14,500 to H0 for one exactly on H0. That is why the default cap moved from 12,000 to **16,000**: at 12,000 the effective bar was about 12 nElo rather than the stated 10.

## Conditional lesson and retry trigger

Under this harness a cap and its bounds must be checked against each other, or the gate silently becomes stricter than it reads. Use the fit to size a budget **prospectively** only; it is a design tool and never a justification for extending a run whose games have been seen. Recalibrate after any change to adjudication, book, TC or the pentanomial model, since `k` absorbs all of them. Three points spanning +5.2 to −5.0 nElo at one bound pair is a narrow basis — treat predictions outside roughly ±6 nElo, or under different bounds, as extrapolation.

## Source

`tools/results/sprt_*`; `tools/sprt.ps1`; `PLAN.md` §2
