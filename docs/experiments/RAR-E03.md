# RAR-E03 — Stockfish-at-60k off-policy distillation with material scale pinned

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Stockfish-at-60k off-policy distillation with material scale pinned.

## Result / disposition

**Rejected, −17.11 Elo,** despite 4.9% lower holdout loss and 9/10 improved buckets.

## Conditional lesson and retry trigger

For this well-fitted HCE/corpus, lower teacher-fit loss did not predict play. Basilisk's +6.75 opposite result reinforces that transfer is engine-state dependent.

## Source

legacy plan; `analysis/archive/hce_analysis.md`
