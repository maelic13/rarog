# RAR-S93 — B.5.2.2 activation read, the tablebase work on every move — REGISTERED 2026-10-01, before any game

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.5.2.2 activation read, the tablebase work on every move — REGISTERED 2026-10-01, before any game.** `rarog-b52cand-pext-pgo.exe` (`13AA4824…`, `ddeffc7`, B.5.2.1 complete) against `rarog-b52base-pext-pgo.exe` (`A7C9EE6C…`, `c5f470b`, the head before it), both bench 11,171,726, clean PGO. Colosseum `match` with `tools/colosseum/tb-activation.toml`: the endgame cohort (`endgame_cohort_v1.epd`, 788 positions, 3 to 6 men, every start in the tables), 1,500 games (no opening repeats), `3+0.03`, Hash 64, one thread, `SyzygyPath=D:/chess/tablebases/syzygy3456` on both sides (510 WDL, 510 DTZ files, recorded), seed 20261015, `tools/results/b52-activation`; dry run at policy. Read by `tools/diag/tb_activation.py`. **Rule, fixed here:** a correctness read, not a strength result. Any candidate game that stood on a clean tablebase win and did not win it, any time loss or fault, or any candidate tablebase-root move over its optimum plus the 5 ms box is an implementation defect: RAR-S94 is not started, and B.5.2.1 reopens. Otherwise RAR-S94 runs. Elo is reported as context only. **Predictions, frozen:** candidate tablebase-root share above 95%; 0 unconverted clean wins on either side; 0 candidate moves over optimum plus box; candidate mean time per tablebase-root move above the baseline's (which moves almost at once); Elo 0 ± 10. Maintainer-run, about 20 minutes.

## Result / disposition

**Played 2026-10-01** (`tools/results/b52-activation`, 1,500 games in 2 min 11 s, 0 faults, every game a normal termination, recount equal): both sides 453-594-453 with every pair split ([0, 0, 750, 0, 0]), so Elo is undefined (zero variance). `tb_activation.py`: **Tablebase roots:** 100% of moves for both sides (21,644 candidate, 21,652 baseline). **Conversion:** 453 of 453 clean tablebase wins converted by each side; 0 unconverted. **Time at tablebase roots:** candidate mean 56.8 ms (max 127), baseline 24.0 ms (max 817). **Moves over optimum plus box (+1 ms slack):** candidate 50 (0.23%), baseline 860. **The registered rule fired:** it called any candidate move over optimum plus the 5 ms box a defect. The 50 exceed the optimum by 6.1–23.7 ms (median 7.0), all at clocks of 2.0–3.0 s, all on `cp ±20000` lines: the box plus the one-ply overrun the record documented (one Fathom call probes a whole ply, up to about 9 ms cold), not a search overrun. The rule, frozen before the games, did not allow for that documented overrun. By its letter, RAR-S94 should not have started; it was played.

## Conditional lesson

**Calibration:** share, conversion and the time ordering hit; the zero-variance outcome was not foreseen (every start is decided by the tables for both). The rule was mis-specified on an effect the record itself had measured: an instrument and specification miss, not a correctness finding. B.5.2's closure waits on the maintainer (PLAN B.5.2.2). **Correction 2026-10-01:** "the one-ply overrun the record documented (up to about 9 ms cold)" is not what a replay of the 50 shows (`analysis/artifacts/b52-box-replay-2026-10-01/`): the search ends within 0.7 ms of the optimum, and single Fathom root calls of up to 16 ms, not repeatable for the same position, make the overrun. A predictive box replayed on the same plies leaves 17 of 29 engine-side overruns. **Closed 2026-10-01** by the maintainer: the rule is recorded as unsatisfiable for a box checked between Fathom calls, and B.5.2.1 is kept.

## Source

PLAN B.5.2.2; `analysis/b52_research_2026-10-01.md`
