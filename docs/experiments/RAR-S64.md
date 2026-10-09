# RAR-S64 — Phase-4 cluster 4.5 (A) — RE-MEASUREMENT after the stale-reduction fix. REGISTERED, NOT YET RUN

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 cluster 4.5 (A) — RE-MEASUREMENT after the stale-reduction fix. REGISTERED, NOT YET RUN.** Supersedes RAR-S61, whose candidate contained a live defect: `stack[ply].reduction` was written on the LMR branch only, so `lmr_prior_reduction_adj` read a value left by a previous sibling or an unrelated subtree. Fixed structurally in `push_move`. Candidate `rarog-45fixed`, bench **7,436,275 / EBF 2.467**. Baseline `rarog-45base`, bench **6,922,439 / EBF 2.451** — the same pre-Cluster-A head RAR-S61 used, so the two gates are directly comparable. ProbCut keeps the PAIRED contract by maintainer preference: RAR-S63 measured null against paired at +0.41 ± 3.97, a dead tie, so the choice is free and the incumbent stays. Final-PGO both, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 adjudication. **Registered bounds `[0,10]` nElo, cap 20,000, fixed before any games.**

## Result / disposition

**H0 ACCEPTED at 8,088 games — Cluster A is worth NOTHING once the defect is removed.** Elo **+0.39 ± 4.89**, nElo **+0.60 ± 7.57**, LOS 56.16%, W-D-L 2,064-3,969-2,055, PairsRatio 1.01, LLR −2.95 of ±2.94. Against RAR-S61's +4.50 ± 3.50 on the identical baseline, **the entire measured gain of Cluster A was the stale-reduction bug.** `lmr_prior_reduction_adj` reading a value left by an unrelated subtree was worth ~4.5 Elo; reading the correct parent reduction is worth zero. The bracket change worked as designed — `[0,10]` resolved in 8,088 games where `[3,10]` had burned 16,000 without moving.

## Conditional lesson

**This is the second time in one cluster that reading the WRONG value beat reading the right one, and the third such observation in the project.** RAR-S62: the ProbCut desync (arbitrary continuation row) beat correct indexing by ~5 Elo. RAR-S64: stale prior-reduction (a quasi-random subset of nodes reduced less) beat correct prior-reduction by ~4 Elo. RAR-S54 already measured a blind, untuned, uniform 15% de-selectivity shift at **+4.06 ± 3.71 over 14,196 games**. Three independent arrivals at the same place: **Rarog's selectivity surface is over-confident, and scattered perturbation of it gains Elo where principled mechanisms do not.** ⚠ The forward reading is NOT "ship bugs". It is that deliberate randomisation of the reduction surface is a candidate mechanism in its own right — Rarog already runs LMR jitter for SMP diversification, so the machinery exists and has a precedent. That belongs to 4.10, with its own registration. What this row settles is narrower and firm: `lmr_prior_reduction_adj` as a principled mechanism is dead, and Cluster A carries no strength.

## Source

`tools/results/sprt_45fixed_vs_Head_20260819_102841.{pgn,log}`; RAR-S61; RAR-S62; RAR-S54; `analysis/code_audit_2026_08_19.md`
