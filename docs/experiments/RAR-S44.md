# RAR-S44 — Phase-4.6c: replace the flat `DIRECT_CHECK_BONUS = 32_000` with safe/losing check classes in quiet ordering …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.6c: replace the flat `DIRECT_CHECK_BONUS = 32_000` with safe/losing check classes in quiet ordering, and dispose of 4.6's two remaining items.

## Result / disposition

**Mixed, and one part is NOT verified.** The class split is implemented and inert (bench 6,502,902 / EBF 2.449 on normal, diag and tune) and is free at the default because the SEE probe is skipped when the two bonuses are equal. **But `CheckBonusLosing` measures 0.00% node change at both 16000 and 0**, so it is NOT demonstrated effective: either the losing-check population is empty on this corpus or the class is not reaching the live ordering path. The duplicate `DIRECT_CHECK_BONUS` constant was removed and its test now reads the live parameter, so the two can no longer drift.

## Conditional lesson and retry trigger

**This arm must not enter any bundle until its population is counted and it is shown to change the tree.** A switch that is inert when off AND when on is indistinguishable from dead code, and this cycle has already produced three comment/code mismatches - concluding it works because it compiles would be the same class of error. Owed: a safe-versus-losing check counter. Dispositions for 4.6's other two items, both by PRIOR evidence rather than new work: **post-LMR depth feedback is already rejected twice** (Phase 2.8 at -1.38 Elo, RAR-S14 at -7.29 Elo), so it needs a retry trigger not a third attempt; and **in-check qsearch ordering is already complete** - quiescence calls the full `score_moves` when in check - so what former 4.3d wanted was lazy STAGING, a throughput change that migrates to 4.9.

## Source

`src/search.rs`; `src/params.rs`; Plan 4.6c
