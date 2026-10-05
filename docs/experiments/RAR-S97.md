# RAR-S97 — Tablebase PV extension start rule, activation and correctness read — REGISTERED 2026-10-01, before any game

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Tablebase PV extension start rule, activation and correctness read — REGISTERED 2026-10-01, before any game** (AGENTS' first repair case: the fix reproduces the fingerprint, so no strength gate). The fix (`b2c98c8`): under a clock the extension starts only when at least ten move overheads remain before the hard ceiling once the search has ended; RAR-S94's forfeit came from the extension running at 58 ms. `tools/test_engines/rarog-b52fix-pext-pgo.exe` (`C0CF3C39…`, `488d34a`, bench 11,171,726, clean PGO) against `rarog-b52cand-pext-pgo.exe` (`13AA4824…`, `ddeffc7`, the binary RAR-S93 and RAR-S94 played). RAR-S93's configuration: `tools/colosseum/tb-activation.toml`, the endgame cohort, 1,500 games, `3+0.03`, the same `SyzygyPath` on both sides, seed 20261019, `tools/results/b52fix-activation`; read by `tools/diag/tb_activation.py`. **Rule, fixed here:** a correctness read. Any `b52fix` game that stood on a clean tablebase win and did not win it, or any `b52fix` time loss or fault, is a defect and reopens the fix. Moves over optimum plus box are reported, not a stop condition (RAR-S93 recorded that rule as unsatisfiable). **What it cannot show:** the forfeit repaired. The cohort's clocks stay near 2 to 3 s, where the rule changes nothing; the repair is shown by the unit test at the forfeited move's numbers and by the paired behaviour read below. **Predictions, frozen:** tablebase-root share 100% on both sides; 0 unconverted; 0 time losses; every pair split, Elo undefined; moves over optimum plus box at most 30 a side, the page cache being warmer than in RAR-S93. Maintainer-run, about 2 minutes.

## Result / disposition

**Before any game, the paired behaviour read** (`tools/results/b6-20261001/ext_start_check.txt`, local 6-man tables, the two binaries above): at `100+30` on two 6-man roots the old binary extends its line (22 and 19 moves, the "requires more time" notice on one) and the fixed one does not (the search's own line, no notice), with the same `bestmove` and ponder move; at `3000+30` both print the same extended line. **Played 2026-10-01** (`tools/results/b52fix-activation`, 1,500 games in 3 min 7 s, 0 faults, every game a normal termination, recount equal, binaries at their registered hashes, host 2.4% busy): both sides 456-588-456, every pair split ([0, 0, 750, 0, 0]), Elo 0, nElo undefined. `tb_activation.py`: tablebase roots 100% of moves on both sides (22,426 fixed, 22,423 old); 456 of 456 clean wins converted by each; mean 56 ms a move on both, max 144 and 148; moves over optimum plus box 23 fixed, 14 old. **No defect by the rule: the fix stands.**

## Conditional lesson

**Calibration:** every prediction hit (share, conversion, no time loss, every pair split, at most 30 over a side). The 23 against 14 is not the rule's doing, which does not act at these clocks; slow reads vary from run to run (the replay's finding). The recount tool failed on this all-split sample after its pentanomial check had passed; repaired in `4bc0d6a`.

## Source

PLAN D.3, B.5.2.2
