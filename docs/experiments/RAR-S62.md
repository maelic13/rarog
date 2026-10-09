# RAR-S62 — Phase-4 cluster 4.5 ablation — REGISTERED, NOT YET RUN

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 cluster 4.5 ablation — REGISTERED, NOT YET RUN.** Isolates the marginal contribution of the 4.5.3 ProbCut piece-desync correctness fix INSIDE Cluster A, measured directly rather than inferred by subtracting two gates. Arm A `rarog-45cluster` `774000b`, bench 7,587,235. Arm B `rarog-45nofix` `46fa4c4`, bench 7,560,177 — identical except the ProbCut site writes `mv` without its piece, reintroducing the desync. Final-PGO both, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 adjudication. **Registered bounds `[-5,5]` nElo, cap 12,000, fixed before any games.**

## Result / disposition

**H0 ACCEPTED at 4,436 games — the correctness fix COSTS strength.** Elo **−5.09 ± 6.47**, nElo **−8.04 ± 10.22** (95% CI [−18.26, +2.18]), LOS 6.15%, W-D-L 1,090-2,191-1,155, PairsRatio 0.92, LLR −2.95 of ±2.94. The bracket resolved in 4,436 games against RAR-M10's ~7,100 estimate for ±5, so the effect is at the larger end of that. Combined with RAR-S61 (+4.50 ± 3.50 against Head), the **inferred** value of Cluster A without the fix is **+9.59 ± 7.36 Elo** — an inference from two comparisons, not a measurement, and RAR-S50 plus this cluster's own node arithmetic both warn that these do not add.

## Conditional lesson

**Bracket chosen from RAR-M10 rather than by habit, after `[3,10]` cost RAR-S61 its whole budget.** `[-5,5]` has midpoint 0, so it resolves in ~7,100 games if the fix is worth ±5 nElo and ~11,800 at ±3 — and if the truth is ~0 it never resolves, runs to the cap, and returns an estimate. That is the correct behaviour here: for a correctness repair costing **0.36% nodes in company** the decision only changes if the fix is clearly HARMFUL, so the test is powered to detect harm, not to prove benefit. `[0,5]` was rejected — it needs 141,687 games for a true +3. ⚠ **Pre-declared follow-up, recorded now so it cannot be invented after seeing the result:** if the fix measures negative, the first hypothesis is NOT that correctness costs Elo but that the constants consuming continuation history — `lmr_hist_div`, `quiet_hist_prune_coeff`, the LMP history thresholds — were fitted against the CORRUPTED signal and are now mis-set. That is the clearest displaced-continuous-optimum case this project has had (unlike 4.7a, whose probe was flat), and it is the condition PLAN rule 4 reserves targeted SPSA for. Removing the repair would be the last resort, not the first. Precedent for retention regardless: RAR-S51's mate clamp.

## Source

`tools/test_engines/rarog-45{cluster,nofix}-pext-pgo.exe`; RAR-S61; RAR-S51; RAR-M10. **Arm B recipe** (branch deleted; it held a deliberate bug): in the ProbCut block of `negamax`, replace the `board.moving_piece(mv)` + `push_move(ply, mv, piece)` pair with a bare `self.stack[ply].mv = mv;`, leaving piece and `cont_key` stale. Rebuild and confirm `bench 13` = **7,560,177 / EBF 2.482**.
