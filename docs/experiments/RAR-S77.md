# RAR-S77 — B.2.3 tail: theta at N = 5,000 against theta at 3,900, SPRT — REGISTERED 2026-09-19, before any game; the …

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.2.3 tail: theta at N = 5,000 against theta at 3,900, SPRT — REGISTERED 2026-09-19, before any game; the candidate is not yet built.** Question (maintainer): did RAR-S75's last 1,100 iterations add strength? At 3,900 the tune had spent 86% of its gain budget. An unresolved result is an answer too: it bounds the tail's value and bears on the horizon and the late step sizes of later tunes. **Candidate:** RAR-S75's final theta, baked in `14a7079` and built as `rarog-b24b-core-pext-pgo.exe` (sha256 `2A586F3B…`, bench 7,185,678). **Baseline:** `rarog-b23-theta3900-pext-pgo.exe` (RAR-S76, sha256 `1B6A49F5…77B9`, 6,199,302). **Test:** `tools/sprt.ps1 -Elo0 0 -Elo1 3 -MaxGames 40000`: `[0,3]` nElo (the default bracket), cap 40,000 games (about 7 hours at RAR-S76's 93 games per minute), `3+0.03`, one thread, Hash 64, UHO, no adjudication, concurrency 14 with affinity. **Budget from RAR-M10** (drift ≈ 8.3e-6 × 3 × (true − 1.5) nElo per game, LLR bounds ±2.94; extrapolated, since this pair is near equal): within 40,000 games H1 needs a true value of about +4.5 nElo or more and H0 about −1.5 or less; a true 0 needs about 79,000 games to reach H0, and a true value near 1.5 never resolves. **Prediction, frozen here:** the tail is worth 0 to +2 nElo, so the cap is reached unresolved (about 0.6); H0 (about 0.25) if the tail was a noise walk; H1 (about 0.15). **Reading:** H1 means the tail moved strength, so the horizon was not too long; H0 means the tail hurt, which argues for a shorter horizon or larger late steps; a cap means the tail was worth less than about 4.5 nElo at this budget. Whatever the outcome, the zero-game movement read (each coordinate's 3,900 → 5,000 displacement against the ±0.3-step noise band of the last session; snapshot `state_t124800.json`, sha256 `9b418f88…d033`) says whether coordinates were still travelling. No stop rule but the SPRT and the cap; bounds, cap, book and adjudication never change after games are seen. Runs after RAR-S75 finishes and the fitted binary exists; its order relative to B.2.4b is free, never concurrently.

## Result / disposition

**Played 2026-09-20** (06:26–10:09 UTC, 3 h 42 m; `tools/results/sprt_theta5000_vs_theta3900_20260920_082658.*`): **H1 accepted** at 20,806 games, LLR 2.94, **+4.43 ± 2.90 Elo, +7.20 ± 4.72 nElo** (W 5,377, L 5,112, D 10,317; pentanomial [342, 2410, 4657, 2629, 365]). All 20,806 games ended normally, no fault.

## Conditional lesson

**Calibration: the outcome missed, and the reasoning behind it was wrong.** Predicted: cap unresolved 0.6, H0 0.25, H1 0.15, from a frozen band of 0 to +2 nElo. The tail measured about 7 nElo. The failed step was reading the zero-game movement (median 0.15 of a step from 3,900 to 5,000, none a full step) as if displacement were an Elo proxy: 82 coordinates each sharpened by a fraction of a step summed to a real gain. **Consequences:** the registered horizon N = 5,000 paid for itself and stopping at 3,900 would have cost about 4 Elo; a tail is not assumed worthless in B.2.7 or B.6; the movement read stays descriptive, never a substitute for games.

## Source

RAR-S75, RAR-S76; RAR-M10; PLAN B.2.3.3
