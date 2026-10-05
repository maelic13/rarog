# RAR-R06 — Helper-history blending and additional ordering jitter

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Helper-history blending and additional ordering jitter.

## Result / disposition

**Neutral/rejected:** blending −0.52; jitter reverted. Shared TT hit rate already rose strongly with thread count.

## Conditional lesson and retry trigger

In that state, TT coupling made generic diversification/history sharing largely redundant. Reopen only with measured independent-work failure, as required by Plan 4.9.

## Source

legacy plan; `analysis/archive/smp_analysis.md`
