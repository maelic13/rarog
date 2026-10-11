# RAR-S63 — Phase-4 step 4.5.3 — ProbCut speculative-move contract, third variant. REGISTERED, NOT YET RUN

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 step 4.5.3 — ProbCut speculative-move contract, third variant. REGISTERED, NOT YET RUN.** RAR-S62 showed the correctness fix costs 5 Elo but could not say why: the desync may have carried signal, or it may have been injecting arbitrary noise into continuation history that happened to regularise it. This variant separates them by recording NOTHING at the ProbCut ply, so the child sees a null previous move and neither reads nor trains continuation history, counter-moves or continuation correction through a move the search does not yet believe in. Arm A `rarog-45nullstack`, bench 7,827,899. Arm B `rarog-45cluster`, bench 7,587,235 — the paired-write correctness fix. Final-PGO both, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 adjudication. **Registered bounds `[-5,5]` nElo, cap 12,000, fixed before any games.**

## Result / disposition

**Dead tie, unresolved at the cap.** Elo **+0.41 ± 3.97**, nElo **+0.63 ± 6.22** (95% CI [−3.56, +4.38]), LOS 57.92%, W-D-L 3,065-5,884-3,051, PairsRatio 1.01, LLR 0.63 of ±2.94 over 12,000 games. Recording nothing at the ProbCut ply is indistinguishable from recording the correct pair — and both sit ~5 Elo behind the desync (RAR-S62). ⚠ **My registered discrimination was wrong, and the row it was written into says so.** I predicted that under the regularisation hypothesis "not writing at all captures the same benefit". It does not: injecting noise into continuation history and declining to touch it are different operations, and only the first adds entropy. Both hypotheses in fact predict null ≈ paired, so this arm could never have separated them. The test was still worth running — it establishes that the ONLY thing distinguishing the desync is the wrong continuation ROW, because `mv` itself was always written correctly and the counter-move table therefore behaves identically in the desync and paired arms.

## Conditional lesson

**The two hypotheses make opposite predictions, which is what makes this worth 45 minutes.** If the desync carried usable signal, writing nothing loses it too and this arm measures ≈0 or negative against the fix — leaving targeted SPSA over `lmr_hist_div`, `quiet_hist_prune_coeff` and the LMP history thresholds as the only recovery route. If the desync was accidental regularisation, not writing at all captures the same benefit with correct code and this arm measures clearly positive — and no SPSA is owed at all. Same `[-5,5]` bracket as RAR-S62 for direct comparability, and because it resolves fast in either direction while returning an estimate if the truth is ~0. ⚠ Note the arm is compared against the FIX, not against Head: the question is which contract to ship, not whether Cluster A is worth shipping, which RAR-S61 already priced at +4.50 ± 3.50.

## Source

`tools/test_engines/rarog-45{nullstack,cluster}-pext-pgo.exe`; **Arm A recipe** (branch deleted): in the ProbCut block of `negamax`, replace the `board.moving_piece(mv)` + `push_move(ply, mv, piece)` pair with `self.clear_move(ply);`. Rebuild and confirm `bench 13` = **7,827,899 / EBF 2.484**. RAR-S62; RAR-S61
