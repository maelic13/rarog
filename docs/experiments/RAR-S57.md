# RAR-S57 — Phase-4 cluster 4.7 — ACCEPTED

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 cluster 4.7 — ACCEPTED.** The 4.7a+4.7c selectivity bundle: null-move entry moved to a hard `nmp_eval >= beta` with the old margin re-homed onto raw `static_eval`, plus a ProbCut move filter tying capture eligibility to the gap the capture must bridge (`probcut_beta - static_eval`, floored at 0), capping moves SEARCHED rather than candidates examined, and scaling that cap by `cut_node`. Candidate on `p47c-probcut-filter`, baseline `dev`. Final-PGO both arms, `3+0.03`, 1T, 64 MB, paired UHO_Lichess_4852_v1, registered `[3,10]` nElo, cap 16,000. **First gate run under RAR-M13 two-sided adjudication.**

## Result / disposition

**PASSED, H1 accepted at 2,838 games — a fifth of the cap.** Elo **+15.44 ± 8.06**, nElo **+24.50 ± 12.78**, LOS 99.99%, LLR 2.96 of ±2.94. W-D-L 796-1,372-670, 52.22%, draw ratio 44.47%, PairsRatio 1.26, Ptnml(0-2) [40, 309, 631, 363, 76]. **Zero time forfeits, crashes or illegal moves** in 2,838 games despite +5.16% bench nodes — the width-for-depth trade did not cost a single game on the clock. `bench 13` 6,519,711 → **6,856,329**, EBF 2.449 → **2.458**. Mechanism, on the 4.2 suite at depth 8: `probcut_attempt` −56.6% against `probcut_cut` −9.0%, conversion per move 32.6% → 68.4% against the oracle's 71.9%; 4.7a's own split was `nmp_attempt` −21.4% against `nmp_cut` −2.4%.

## Conditional lesson

**The largest accepted search gain of the project, and it beat its own prior by 60%** — 24.50 nElo against a 5–15 band derived before the games. It also does what PLAN 4.7 predicted of a structural rework: RAR-S54's blind uniform 15% de-selectivity scalar measured +4.06 ± 3.71, and the two coherent contracts are ~3.8x that. **No subcomponent is credited.** 4.7a and 4.7c were gated as one cluster under rule 3 and neither has a standalone result; splitting the +15.44 between them requires the ablation rule 7 demands, and until that runs the honest attribution is "the bundle". ⚠ **Process deviation, recorded rather than hidden:** the bounds `[3,10]`, cap 16,000 and the 5–15 prior were fixed in writing before the run and used verbatim, but the `EXPERIMENTS.md` row itself was written after the result. Rule 2 wants the row filed first. Nothing was changed after seeing games; the filing lagged, not the design.

## Source

`tools/results/sprt_47bundle_vs_Head_20260818_153904.{pgn,log}`; `analysis/phase4_differential_47c_depth8.txt`; RAR-S55 v3; RAR-S56; PLAN 4.7; record: `analysis/ledger_records_2026-09-14.md`, RAR-S57 (Search and selectivity)
