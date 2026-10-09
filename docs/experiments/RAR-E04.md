# RAR-E04 — 500k-game on-policy refresh yielding 2.18M unique positions

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

500k-game on-policy refresh yielding 2.18M unique positions; pure WDL beat blended labels on the shared holdout.

## Result / disposition

**Rejected, −1.28 ± 2.79 over 26.8k games;** pipeline and inert parameters retained.

## Conditional lesson and retry trigger

Even on-policy lower validation loss did not improve this unchanged representation. Retry only after representation/policy changes and with a frozen external holdout.

## Source

legacy plan at `757e9a3^`
