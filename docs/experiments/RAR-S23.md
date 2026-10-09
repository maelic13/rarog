# RAR-S23 — Phase-4.2 typed evidence refactor: `OutcomeKind`/`NodeEvidence`/`MoveEvidence`, all 7 producers …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.2 typed evidence refactor: `OutcomeKind`/`NodeEvidence`/`MoveEvidence`, all 7 producers typed, all 13 read sites routed through named capability predicates. Ryzen 9 5950X, non-PGO, debug+release+diag.

## Result / disposition

**Retained infrastructure, behaviour-neutral.** `bench 13` = 6,502,902 / EBF 2.449 on both the normal and diag builds. Against the pre-refactor binary, 8 positions × 12 iterations produced 96 identical depth lines (nodes, seldepth, score, hashfull, tbhits, full PV) plus identical bestmove/ponder. Every pre-existing diag counter was identical, including the whole sampled interaction map. New exact producer census: full 31.18%, stand pat 35.87%, qsearch tail 21.07%, qsearch move 10.56%, ProbCut 1.32%, tablebase 0%; depth-0 total **67.50%**; reconciles with `tt_store_fresh + tt_store_same_key` at 2,659,461.

## Conditional lesson and retry trigger

Two conditional lessons. (a) Sampled counters taken at different node classes do not share a denominator: the census confirmed RAR-S22's depth-0 share but showed its ProbCut share understated 2.4x, so a sampled producer count is not a share unless its denominator is stated. (b) Centralizing the admission rules exposed a real divergence that was invisible while each rule sat at its own call site — the main search's eval refinement enforces a depth floor and a `VALUE_NONE` test and the qsearch stand-pat path enforces neither. It is preserved as two named capabilities with a test pinning the difference, because RAR-S15 showed a cleaner primitive can de-tune consumers fitted around the looser one. Unifying them is 4.3 and needs its own gate.

## Source

`src/evidence.rs`; Plan 4.2–4.3
