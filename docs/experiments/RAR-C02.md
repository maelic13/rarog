# RAR-C02 — Rule-50 draw could override mate

Indexed under *7. Correctness and protocol lessons* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment or failure mode

Rule-50 draw could override mate; null moves changed the halfmove clock; repetition lacked root/null awareness.

## Disposition

Free mate-precedence fix retained; optional draw-policy bundle followed RAR-S18's strength verdict.

## Conditional lesson / coverage

Separate legal terminal precedence from heuristic repetition policy; they can have different acceptance criteria.

## Source

legacy plan; `analysis/archive/search_analysis.md`
