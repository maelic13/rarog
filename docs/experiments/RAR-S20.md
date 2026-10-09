# RAR-S20 — Half-run aspiration SPSA snapshot `ba3170b` (`15/148/149/9/20/8/0`) versus clean `p1043-base`

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Half-run aspiration SPSA snapshot `ba3170b` (`15/148/149/9/20/8/0`) versus clean `p1043-base`; `[0,+3]`, `3+0.03`, 1T, 64 MB, paired UHO.

## Result / disposition

**Rejected by acceptance rule after manual stop:** 13,000 games, W-D-L 3,261-6,378-3,361, −2.67 ± 3.83 Elo / −4.16 ± 5.97 nElo, LLR −1.83 (bounds ±2.94). It did not formally hit H0, but did not accept H1; candidate bench was 7,047,226 versus 6,502,902.

## Conditional lesson and retry trigger

In this incomplete fit, narrowing the initial window widened the tree without demonstrated strength. Do not resume or tail-select it; revisit aspiration only through the completed root-confidence model and consolidated Plan-4.10 fit.

## Source

snapshot `ba3170b`; Plan 4.0
