# RAR-O02 — No-adjudication confirmation of RAR-O01, stopped after 1,238/2,400 games because the architectural decision …

Indexed under *3. Search and selectivity › Search-oracle observations* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

No-adjudication confirmation of RAR-O01, stopped after 1,238/2,400 games because the architectural decision had already resolved. Same four engines, TC/book/1T/concurrency; max-move, draw and resign adjudication all off, tablebases/ponder off. Roughly 205 games were complete per pair and 982 of 1,238 games ended by natural checkmate.

## Result / disposition

**Observation, sufficient and deliberately stopped.** Hybrid–Rarog 118-77-12, score 75.60%, about **+196.5 Elo**; Hybrid–Basilisk 134-42-29, 75.61%, about **+196.5 Elo**; Stockfish-HCE–Hybrid 163-32-11, 86.89%, about **+328.6 Elo**; Basilisk–Rarog 77-70-59, 54.37%, about **+30.4 Elo**. No forfeits. Average NPS: control 2.3M, hybrid 1.5M, Basilisk/Rarog 2.4M.

## Conditional lesson

Removing adjudication left decisive reciprocal search- and HCE-oracle signals despite the hybrid's lower NPS. The 74-Elo gap versus RAR-O01 prices the adjudication confounder directly: **cross-evaluator cohorts run with adjudication off**. More games would refine ratings, not change the decision to attempt one bounded native-Rust programme: freeze HCE while reworking search, freeze the accepted search head, then study HCE contracts under separate gates.

## Source

Colosseum “Rarog Hybrid testing” stopped 2026-08-11; PLAN §4; GUIDE Phase-4 step lifecycle
