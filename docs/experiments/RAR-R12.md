# RAR-R12 — A.3.3 harness reserve: `Move Overhead` 40 against 10 on the same binary - RUN 2026-09-10, REJECTED at -80.85 …

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

*This entry had 5 cells in a table of 4 columns (Experiment and conditions, Result / disposition, Conditional lesson and retry trigger, Source); its parts are kept in their original order.*

## Part 1

**A.3.3 harness reserve: `Move Overhead` 40 against 10 on the same binary - RUN 2026-09-10, REJECTED at -80.85 Elo.** `rarog-r11cand-pext-pgo.exe` on both sides, arm A `option.Move Overhead=40`, arm B the default 10; `tools/sprt.ps1 -Mode fixed -Games 10000 -NoAdjudication`, `3+0.03`, 1T, Hash 64, paired UHO, concurrency 14 with affinity, `timemargin` 20 ms. Runs in the 2026-09-09 night script after RAR-R11. Why: the reconstructed forfeits are 50-500 ms stalls; the engine now counts pre-search latency (RAR-R11) but a stall that lands mid-search is visible to no engine, and Basilisk, which has counted dispatch latency since its Step 5.4, still forfeits at the same rate. The reserve is the only lever left, and it is a harness/profile setting, not an engine change.

## Part 2

**Prediction, frozen 2026-09-09:** arm B (10 ms) 4 to 12 forfeits, arm A (40 ms) 0 to 4; paired Elo of arm A **-2 to +1** (an extra 60 ms of reserve at a three-second clock binds only on the last moves). Probability the wider overhead at least halves the rate: 0.6.

## Part 3

Adopt `Move Overhead=40` in the SPRT and pool profiles only if arm A forfeits at most a quarter of arm B AND the paired interval excludes -3; otherwise keep 10 and record the rate as the harness floor. Symmetric across arms in every gate, so it never changes a verdict.

## Part 4

**RESULT:** `sprt_r12overhead40_vs_r12overhead10_20260910_003730`, 10,000 games, **0 forfeits in either arm**, arm A (40 ms) **-80.85 +/- 4.62 Elo**, nElo -123.43, Ptnml [600, 1858, 1848, 616, 78]. **Calibration: the prediction missed in mechanism.** `Move Overhead` is not a reserve on the last moves; the budget formula (`time_manager.rs`, Stockfish's shape) subtracts `overhead * (2 + movestogo)` from the planning clock, so 40 ms at a 50-move horizon discards about 2.1 s of a 3 s clock and the engine plays at a third of its time. Basilisk's BAS-E57 shows the same (-64.81). **Disposition:** `Move Overhead` stays 10 in every profile; a reserve that binds only at low clocks has to be an engine-side term, which is D.1's question. The forfeit rate is undecidable here for the same reason as RAR-R11.

## Part 5

PLAN A.3.3, D.1; RAR-M14; RAR-R11; BAS-E57; `tools/results/night-20260909/night_summary.txt`
