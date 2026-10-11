# RAR-S69 — SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert …

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert; its question is answered inside B.2's cluster and SPSA or not at all, no retry trigger.** **RAR-S54's ten pruning margins, ×1.15 on the current head — REGISTERED, NOT YET RUN.** The last candidate in this line. Shifts only the **ten pruning-margin** constants RAR-S54 moved — futility (×2), razoring, LMP (×2), quiet-history, SEE (×2), quiet futility (×2) — at their CURRENT values, leaving the two LMR-table constants alone because RAR-S68 measured that half at zero. `razoring_coeff` is clamped at its declared rail of 300 (×1.095 rather than ×1.15). Arm A `rarog-47margin`, bench **7,483,775 / EBF 2.471**. Arm B `rarog-47base2`, bench **7,467,143 / EBF 2.477**. Final-PGO both, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 adjudication. **Registered bounds `[0,3]` nElo, cap 80,000, fixed before any games.**

## Result / disposition

*Pending.*

## Conditional lesson

**Cap raised to 80,000 deliberately, because RAR-S68's 60,000 could not decide a true +3 (it needs 78,715) and would have died at the cap.** At 80,000 both a true +3 accept and a true 0 rejection land inside the budget, so this run returns an answer either way — the first in this line that can. ⚠ **This is the direct re-test of RAR-S54 on the current head.** That +4.06 ± 3.71 over 14,196 games was measured against 2.3.1, and 4.7's +15.56 came from the ProbCut move filter, a different mechanism — so these ten constants are untouched since. A null says the blind-shift headroom is spent and the whole line closes. ⚠ **Not a bake candidate for the scalar**, per PLAN: RAR-S54 licenses a structural rework, not shipping a uniform multiplier. A pass means locating which margin is systematically too aggressive, as 4.7 did for ProbCut. Worth noting on its own: `razoring_coeff` has moved 193 → 274 since 2.3.1 and now sits within 10% of its rail, so that one term has already been tuned hard in this direction.

## Source

`tools/test_engines/rarog-47{base2,margin}-pext-pgo.exe`; branch `p410-margin-relief`; RAR-S54 recipe; RAR-S68; RAR-S67; record: `analysis/ledger_records_2026-09-14.md`, RAR-S69 (Search and selectivity)
