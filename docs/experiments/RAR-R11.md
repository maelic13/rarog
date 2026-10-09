# RAR-R11 — A.3.3 time-forfeit repair at `3+0.03` - RUN 2026-09-09/10, forfeit rate UNDECIDABLE (0 against 0), Elo bound …

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

*This entry had 5 cells in a table of 4 columns (Experiment and conditions, Result / disposition, Conditional lesson and retry trigger, Source); its parts are kept in their original order.*

## Part 1

**A.3.3 time-forfeit repair at `3+0.03` - RUN 2026-09-09/10, forfeit rate UNDECIDABLE (0 against 0), Elo bound met; repair KEPT on donor parity.** Baseline: the A.3.1 head (`ca8988a`, PGO pext, 1.98.1). Candidate: the same head plus the A.3.3 fix, **corrected before any game (2026-09-09)** once the diagnosis was in: the clock-origin repair of `79d3974` (the search budget starts when `go` is parsed on the UCI thread, as Stockfish and Reckless do, instead of after the engine-thread hand-off), because the reconstructed forfeits are 50-500 ms stalls that the engine's budget could not see (`analysis/time_forfeit_2026-09-09.md`). An interim info-line throttle (`d93f808`) was reverted before any game (`e3430d9`) because neither donor throttles and its reproduction measured the driver's own per-line lag. Baseline binary `rarog-e16cand-pext-pgo.exe` (`a4c4f95`, SHA-256 `889B3179...`); candidate `rarog-r11cand-pext-pgo.exe` built from `79d3974`. `tools/sprt.ps1 -Mode fixed -Games 10000 -NoAdjudication`, `3+0.03`, 1T, Hash 64, paired UHO, concurrency 14 with affinity, `timemargin` 20 ms, `Move Overhead` 10 ms on both arms. Forfeits counted per arm from the log and cross-checked against the reconstructed PGN clocks.

## Part 2

**Prediction, frozen 2026-09-09:** baseline arm 4 to 12 forfeits in 10,000 games (the measured rate is 0.05 to 0.13%); candidate arm 0 to 2; paired Elo **-1 to +1** (the clock origin moves earlier by the hand-off latency, a few hundred microseconds on an idle host, so thinking time is essentially unchanged; any Elo movement is noise). Probability the fix halves the rate: 0.5 -- it closes only the stalls that land before the search starts. Confidence moderate on direction, low on the exact counts at these rates.

## Part 3

Accept only if the candidate forfeits at most a quarter of the baseline's count AND the paired interval excludes -3 Elo; a rate reduction bought with more than that is a harness setting (`Move Overhead`), not an engine change. RAR-M14 and the reconstructed clocks of the seven forfeited games (2026-09-09) are the diagnosis; the engine spends its whole budget by its own accounting and the forfeit is the wall-time gap under a saturated host. Time management is bench-invisible; games decide.

## Part 4

**RESULT:** `sprt_r11cand_vs_r11base_20260909_224551`, 10,000 games, **0 forfeits in either arm**, Elo **-0.69 +/- 3.62**, nElo -1.31 +/- 6.81, LOS 35%, Ptnml [118, 938, 2918, 898, 128]. The registered rate rule (candidate at most a quarter of the baseline) cannot be evaluated at 0 against 0; the Elo rule (interval excludes -3) is met. **Calibration:** the prediction of 4 to 12 baseline forfeits assumed the daytime host; the run started at 22:45 on an otherwise idle machine, and every earlier forfeit came from runs during working hours, so the rate is a daytime-interference rate. Miss in instrument, not mechanism. **Disposition:** the repair stays, because it aligns Rarog's clock origin with both donors and costs nothing measurable; the forfeit rate is watched in the A.5 pool runs, which happen under daytime load. No acceptance rule is invented after the fact.

## Part 5

PLAN A.3.3; RAR-M14; `analysis/time_forfeit_2026-09-09.md`; `tools/results/night-20260909/night_summary.txt`
