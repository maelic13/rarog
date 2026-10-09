# RAR-S76 — B.2.3 checkpoint peek, theta at iteration 3,900 against the unfitted head — REGISTERED 2026-09-19, before any …

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.2.3 checkpoint peek, theta at iteration 3,900 against the unfitted head — REGISTERED 2026-09-19, before any game; diagnostic only.** Maintainer request while RAR-S75 is paused at t = 124,800 games (3,900 of 5,000 iterations; `state.json` snapshot sha256 `9b418f88…d033`). **Candidate:** the 82 coordinates rounded half away from zero and baked as `CoreParams` defaults on the never-merged branch `diag/b23-theta3900` (parent `df6308e`; the diff is preserved as `analysis/arm_patches/23b8a7a-theta3900-coreparams.patch` since B.10, 2026-10-04), built `./tools/build_test.ps1 -Suffix b23-theta3900 -Features b2core` → `rarog-b23-theta3900-pext-pgo.exe`, sha256 `1B6A49F5…77B9`, bench **6,199,302**. **Baseline:** `rarog-b24a-core-pext-pgo.exe` (`6e4fa8a`, 4,706,910; no engine input changed since). **Run:** `tools/sprt.ps1 -Mode fixed -Games 2000`, `3+0.03`, Hash 64, one thread, UHO book, harness defaults; reports an Elo with an interval and decides nothing. **Prediction, frozen here:** +10 ± 11 for the candidate (RAR-S75's +12 ± 11 for the final theta, less a little for the unfinished schedule), moderate confidence. **Rules:** this is a checkpoint read, so it is never baked and never the estimator; RAR-S75's horizon (N = 5,000) and estimator (final theta, no checkpoint selection) are unchanged by it. Any change to N made after this result is a post-exposure amendment and is recorded as one. The host is idle during the run and the SPSA stays stopped until it finishes.

## Result / disposition

**Played 2026-09-19** (08:07–08:28 UTC; `tools/results/sprt_theta3900_vs_unfitted_20260919_100717.*`, the manifest pins both sha256): **+118.72 ± 10.62 Elo, +183.86 ± 15.23 nElo** in 2,000 games (W 924, L 266, D 810; pentanomial [8, 83, 319, 423, 167]). All 2,000 games ended normally, with no forfeit or time loss; time per move 73.3 against 73.4 ms and mean reported depth 16.27 against 16.26, so neither time use nor depth explains the margin. Bench, one run each on an idle host: candidate 6,199,302 / EBF 2.421 against 4,706,910 / EBF 2.391 (a wider tree to the same nominal depth).

## Conditional lesson

**Calibration: missed in magnitude by about ten times** (+10 ± 11 predicted). The failed assumption is that the donor-shaped seeds sat near Rarog's STC optimum; RAR-S75's frozen +12 ± 11 rests on the same assumption and is scored at B.2.4b, not rewritten. A candidate explanation, untested: the donor's margins are in the donor's evaluation units, and Rarog's scale differs. Still a peek: never baked, never the estimator; RAR-S75 continues to N = 5,000 as registered.

## Source

RAR-S75; branch `diag/b23-theta3900`; ignored `tools/test_engines/rarog-b23-theta3900-pext-pgo.{exe,json}`
