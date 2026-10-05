# RAR-S25 — Phase-4.3a provenance-hazard census at `d354d02`: can a consumer infer a producer from entry shape, given …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.3a provenance-hazard census at `d354d02`: can a consumer infer a producer from entry shape, given provenance is not persisted? Exact counters in the store path, `bench 13`, 1T.

## Result / disposition

**Observation; the absolute counts are exact but every PERCENTAGE below is provisional and biased low.** ⚠ The kind census counts store ATTEMPTS (it runs in `TranspositionTable::store` before dispatch, which is why it reconciles with `fresh + same_key`), while the hazard counters run after the depth-preservation `return` and so count COMMITTED stores only. Dividing one by the other mismatches denominators. A stand-pat store is rejected exactly when it lands on a same-position entry deeper than 3, so the committed denominator is materially smaller and the true rates are HIGHER than printed. Raw findings: a moveless store inherits the resident move, and **33,712 stand-pat stores** walked away carrying a searched move, becoming byte-identical to a searched qmove; against 953,957 *attempted* stand-pat stores that reads as 3.53%. A shape test of `depth 0 + Lower + has a move` — the only provenance-free way to grant searched qmoves capability while denying stand pat — reads as a **10.71% false-positive rate** on the same mismatched basis. **62,821 horizon stores** overwrote a deeper same-position entry. Corrected rates pending committed/rejected counters.

## Conditional lesson and retry trigger

Under this state 4.3 cannot cleanly separate stand pat from searched qmoves without persistence, which is the concrete trigger 4.2 registered for reopening the 1-bit provenance question (age 5→4 bits, `[0,3]` gate, RAR-S22). Suppressing the inheritance instead would make the inference exact but discards ordering evidence RAR-S24 measured as valuable, so it is not a free fix. The direction of the conclusion is unaffected by the denominator defect — correcting it can only raise the leak, and the leak already blocks the inference. The depth-0-versus-deeper split needs no move test and no persistence, so it remains sound at any leak rate; only the within-horizon split is blocked. Method lesson, third of this kind in Phase 4 after the ProbCut sampling and contradiction-gating cases: **a rate is only meaningful when numerator and denominator are collected at the same point in the code**. Add the matched counter before publishing the ratio, not after.

## Source

`src/tt.rs`; `tools/diag_search_quality.ps1`; `d354d02`; Plan 4.2–4.3
