# RAR-S67 — SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert …

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**SUPERSEDED by B.2, never run (B.1, 2026-09-14): the switch or parameter this arm needed was removed as inert; its question is answered inside B.2's cluster and SPSA or not at all, no retry trigger.** **1T LMR-reduction jitter — REGISTERED, NOT YET RUN.** The 4.10 obligations document's strongest lead. Rarog already runs a per-thread xorshift jitter of ±64/1024 ply on the LMR reduction for SMP diversification, disabled at 1T only to keep bench deterministic. This enables it at 1T at **±128/1024 (⅛ ply)**. Arm A `rarog-47jitter`, bench **6,867,326 / EBF 2.457**. Arm B `rarog-47base`, bench **7,467,143 / EBF 2.477** — the accepted head. `next_jitter` now takes a magnitude and at 64 is exactly the previous expression, so the SMP path is unchanged by construction. Final-PGO both, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 adjudication. **Registered bounds `[0,3]` nElo, cap 60,000, fixed before any games.**

## Result / disposition

**STOPPED early, tracking to H0 — NOT promoted.** See the stop note recorded below the table.

## Conditional lesson

**This is the only candidate in the phase whose prior rests on game results rather than counters.** RAR-S54: a blind, untuned, uniform 15% de-selectivity shift measured **+4.06 ± 3.71 over 14,196 games**. RAR-S62: a ProbCut desync reading an ARBITRARY continuation row beat correct indexing by ~5 Elo. RAR-S64: a stale prior-reduction firing on a quasi-random subset beat the correct one by ~4.5. Twice, a bug that scattered noise into the selectivity surface beat its own correction — which is why the hypothesis is perturbation, not any particular mechanism. ⚠ **The bench cannot screen this, and that is structural, not a tooling gap.** A deterministic bench samples ONE realisation of a randomised reduction; the hypothesised value is diversification across games. The sweep behaves accordingly — nodes bounce 6.3M–7.5M with no trend, `fm` wanders 87.99–88.52, and `cut/node` slightly FALLS. So the magnitude is a judgement and is recorded as one: 128 is double the SMP value, which was itself chosen small enough not to distort the mean reduction. ⚠ **Gate before tuning.** RAR-S13 ran `cutoffCnt` plus a full LMR-family SPSA and lost **7.78 ± 8.00** because the tuner selected a sibling-local optimum that won its own self-play and then lost to the accepted head. Magnitude is the obvious SPSA coordinate, and it stays untouched until the mechanism itself clears a gate. Determinism is preserved: the 1T PRNG re-seeds per search from a fixed seed, so two bench runs agree exactly.

## Source

`tools/test_engines/rarog-47{base,jitter}-pext-pgo.exe`; branch `p410-jitter-1t`; `analysis/archive/phase4_10_obligations.md` B1; RAR-S54; RAR-S62; RAR-S64; RAR-S13; record: `analysis/ledger_records_2026-09-14.md`, RAR-S67 (Search and selectivity)
