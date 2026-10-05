# RAR-M09 — Phase-4.1 normal versus diagnostic release builds on the Ryzen 9 5950X, non-PGO, `bench 13` plus four …

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.1 normal versus diagnostic release builds on the Ryzen 9 5950X, non-PGO, `bench 13` plus four depth-10 positions.

## Result / disposition

**Retained infrastructure:** both builds matched 6,502,902 nodes / EBF 2.449; the four probes matched nodes, scores, PVs and best/ponder moves. Paired best-of-three NPS was 2,585,646 versus 2,301,097 (11.0% diagnostic cost including legacy exact atomics).

## Conditional lesson and retry trigger

Under these conditions, the sampled observers did not alter the tree and their offline cost was bounded. This does not prove equivalence on every ISA/thread count or make counter movement an Elo proxy; repeat the gate after diagnostic control-flow changes.

## Source

`src/diag.rs`; `tools/diag_search_quality.ps1`; Plan 4.1
