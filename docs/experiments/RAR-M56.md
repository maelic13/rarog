# RAR-M56 — The unfitted B.2 head against Rybka 4.1 — the control for RAR-M55 (PLAN B.2.3.2). Registered 2026-09-19 while …

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**The unfitted B.2 head against Rybka 4.1 — the control for RAR-M55 (PLAN B.2.3.2). Registered 2026-09-19 while its first games were playing: the maintainer launched it at 09:15 UTC on the command and prediction given before launch, and this row writes them down unchanged.** Question (maintainer): is RAR-M55's +98 plausible when Rarog 2.4.0 was about 100 behind Rybka? The same harness, the same Rybka and the same conditions with only the fit removed answer it without a chain of separate measurements. **Rarog:** `rarog-b24a-core-pext-pgo.exe` (the accepted B.2 head, `6e4fa8a`, sha256 `9206A598…71F8`, 4,706,910). **Reference and run:** exactly RAR-M55's: Deep Rybka 4.1 SSE42 x64 (sha256 `15901A12…C9923`) with `Max CPUs=1`, `tools/sprt.ps1 -Mode fixed -Games 2000`, `3+0.03`, Hash 64, UHO, no adjudication, concurrency 14 with affinity (`tools/results/sprt_unfitted_vs_rybka41_20260919_111505.*`). **Prediction, stated before launch:** about −15 Elo (RAR-M55's +98 less RAR-S76's +119). **Reading:** near −15 means both matches and the fit's self-play gain agree; near −100 would mean one of them is wrong, to be chased before anything is recorded as a gain over Rybka. An observation, never a gate.

## Result / disposition

**Played 2026-09-19** (09:15–09:37 UTC; `tools/results/sprt_unfitted_vs_rybka41_20260919_111505.*`, the manifest pins both sha256): **−7.12 ± 12.78 Elo, −8.49 ± 15.23 nElo** for Rarog in 2,000 games (W 761, L 802, D 437; pentanomial [129, 199, 395, 138, 139]). All 2,000 games ended normally with no time loss, disconnect or illegal move; time per move 70.8 against Rybka's 70.6 ms; mean depth Rarog 13.22, Rybka 12.44.

## Conditional lesson and retry trigger

**Calibration: within the band** (−15 predicted; 0.6 σ). Against the same foreign reference the fit is worth **+104.8 ± 18.1** (RAR-M55 less this row), about 88% of its +118.72 self-play value (RAR-S76). The unfitted core itself moved Rarog from 2.4.0's −131 ± 45 (RAR-M54, 200 games) to −7 against Rybka 4.1, more than its +65 self-play margin but inside that old row's wide interval. Both Rybka matches and the fit's self-play gain agree; nothing to chase.

## Source

RAR-M55, RAR-S76, RAR-M54; PLAN B.2.3.2
