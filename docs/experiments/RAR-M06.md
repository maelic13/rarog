# RAR-M06 — Resignation threshold replay against 69,350 historical games

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Resignation threshold replay against 69,350 historical games.

## Result / disposition

`400/3` one-sided was too aggressive for Rarog's scale; `600/3` one-sided became `strength-v1`.

## Conditional lesson and retry trigger

Adjudication scores are engine-scale dependent. Recalibrate after a material score-scale change such as NNUE integration.

## Source

legacy plan at `757e9a3^`; `PLAN.md` §2
