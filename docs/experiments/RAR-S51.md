# RAR-S51 — NMP mate-clamp correctness repair: keep an unproven mate score from satisfying a null-move cutoff

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

NMP mate-clamp correctness repair: keep an unproven mate score from satisfying a null-move cutoff. Candidate merged, versus the pre-clamp baseline; final-PGO `[−5,0]` nElo gate at `3+0.03`, 1T/64 MB, paired UHO.

## Result / disposition

**Gate interrupted and formally unresolved.** Last complete report: 2,836 games, **+2.45 ± 7.33 Elo / +4.27 ± 12.79 nElo**, W-D-L 752-1,352-732, LLR +0.80 of ±2.94; no anomaly or regression signal, but the interval is far too wide to claim neutrality. Fingerprint moves from 6,502,902 to **6,519,711**, EBF 2.449.

## Conditional lesson and retry trigger

**Retained by maintainer decision as a correctness repair, not promoted as a strength improvement.** Do not resume or reinterpret the partial SPRT. The clamp is part of the 2.3.2 accepted architecture. This exception is explicit: it prevents an unproved mate from becoming an authoritative cutoff, while the partial games merely show no early alarm.

## Source

`src/search.rs`; `tools/results/sprt_MateClamp_vs_Baseline_20260811_170955.*`; `PLAN.md` Phase-4 disposition
