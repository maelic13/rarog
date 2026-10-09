# RAR-M55 — Rybka 4.1 benchmark of the B.2.3 fit — REGISTERED 2026-09-19, before any counted game; an observation, never …

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Rybka 4.1 benchmark of the B.2.3 fit — REGISTERED 2026-09-19, before any counted game; an observation, never a gate (PLAN B.2.3.2).** Maintainer request: how close is Rarog to Deep Rybka 4.1, the maintainer's long-standing reference, after RAR-S76's +119 Elo. **Rarog:** `rarog-b23-theta3900-pext-pgo.exe` (RAR-S76; sha256 `1B6A49F5…77B9`, bench 6,199,302). **Reference:** `D:\chess\engines\Deep Rybka 4.1 SSE42 x64.exe` (sha256 `15901A12…C9923`, `id name Deep Rybka 4.1 SSE42 x64`), at its UCI defaults except `Max CPUs=1` (default 2048) and Hash 64. It advertises no `Threads`; fastchess warns once per game and skips that option. **Run:** `tools/sprt.ps1 -Mode fixed -Games 2000 -OptionsB "Max CPUs=1"`, `3+0.03`, UHO, no adjudication, concurrency 14 with affinity. **Wire check, 28 games, not counted** (`tools/results/sprt_theta3900_vs_rybka41wirecheck_20260919_103907.*`): one Rybka process per game with three threads and about one core each, so `Max CPUs=1` holds; Rybka's time per move is 68.7 ms against Rarog's 68.0, so its `TC Buffer` default of 3 does not starve it at this TC; all 28 games ended normally. **Control, 2026-09-19:** at its defaults Rybka starts 31 child processes of its own executable to search, with `Max CPUs=1` none and one core; during a 14-game run of the exact command (`…_vs_rybka41wire2_*`, not counted) the host held 14 Rybka processes for 14 games, so the option reaches the engine. The first launch (`…_20260919_104719.*`) was stopped by the maintainer after 11 games to check the `Threads` warning; it is not counted, and the counted run is a fresh launch. **Prediction:** 0 ± 80 Elo for Rarog, low confidence. It was made after the wire check's 28 games (+63 ± 108), which are too few to anchor it, and with no earlier head-to-head at `3+0.03` in this ledger. **Caveats:** one reference at one TC. It is not comparable with RAR-S76's self-play Elo, and it is not a CCRL distance: Rybka 4.1 was built for longer controls on its own hardware assumptions.

## Result / disposition

**Played 2026-09-19** (08:50–09:12 UTC; `tools/results/sprt_theta3900_vs_rybka41_20260919_105002.*`, the manifest pins both sha256): **+97.69 ± 12.84 Elo, +122.10 ± 15.23 nElo** for Rarog in 2,000 games (W 1045, L 497, D 458; pentanomial [48, 96, 360, 252, 244]). All 2,000 games ended normally with no time loss, disconnect or illegal move; time per move 70.5 against 70.5 ms; mean reported depth Rarog 13.19, Rybka 12.72 (12.36 in the file-redirected wire check), so fastchess's console output of Rybka's over-long PVs cost Rybka nothing measurable.

## Conditional lesson and retry trigger

**Calibration: missed in magnitude** (0 ± 80 predicted, low confidence; the measurement sits 1.2 of the stated σ above). The prediction ignored the chain of measured results: Rarog 2.4.0 against Rybka 4.1 −131 ± 45 (RAR-M54, 200 games), plus the unfitted core +65.09 ± 23.26 (B.2.4a) and the fit to 3,900 +118.72 ± 10.62 (RAR-S76), gives about +53 ± 52, which the result does not contradict. Self-play gains transferred at least in full to this foreign reference. RAR-M56 measures the unfitted head against the same Rybka in the same harness, which removes the chain.

## Source

RAR-S76, RAR-M54, RAR-M56; PLAN B.2.3.2
