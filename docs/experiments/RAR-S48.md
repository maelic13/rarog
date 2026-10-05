# RAR-S48 — Phase-4.9d: SIZE the in-check qsearch staging that 4.6c deferred here, before building it

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.9d: SIZE the in-check qsearch staging that 4.6c deferred here, before building it. An in-check qnode generates every evasion and scores ALL of them up front, so a node cutting on its first move paid for the rest; staging would emit the TT move before scoring anything, which is order-IDENTICAL because `score_moves` already gives it a dominating score. Exact counters (the population is small enough that sampling would add noise to the deciding number), `bench 13`.

## Result / disposition

**Population real, payoff below this project's own measurement floor — NOT BUILT.** 169,780 in-check qnodes score **608,608** evasions and try only **285,769**: 3.58 scored against 1.68 tried, so **53.0% of all evasion scoring is wasted**. But that is 322,839 wasted scorings across 6,502,902 nodes — **0.0496 per node** — and at a plausible 5–10% of a node's cost per move-scoring the ceiling is **0.25–0.50% NPS**. The 4.2c pooled PGO A/B resolved to ±0.125%, so the upper end is barely two noise widths and the lower end is inside it.

## Conditional lesson and retry trigger

**A large SHARE and a small ABSOLUTE are different findings, and only the second decides.** 53% waste reads like an obvious win; 0.05 wasted scorings per node is one that could not be validated even if built, because a staged in-check picker is real code that risks perturbing move order — and an unmeasurable speed change is exactly what the bench-identical-plus-pooled-NPS rule exists to refuse. ⚠ Retry trigger is NEGATIVE, not neutral: after NNUE a node costs far more while a move-scoring costs the same, so this share of runtime SHRINKS and the case gets weaker, not stronger. Revisit only if in-check qnodes become a materially larger share of the tree. The counters are retained so the decision stays checkable rather than remembered.

## Source

`src/search.rs`; `src/diag.rs`; Plan 4.9d
