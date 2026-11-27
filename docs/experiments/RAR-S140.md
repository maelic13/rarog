# RAR-S140 — The with-tables activation read on the `d9` head against 2.7.0: RAR-S93's design as E.1's release check — REGISTERED 2026-11-27, before any game

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-27 by the agent, before any game;
  the games are the maintainer's command.
- Disposition: open — registered 2026-11-27; the maintainer's run.
- **Kind:** a correctness read, not a strength result (PLAN E.1, *The
  with-tables activation read*; D.9.4). Evidence layer: conversions, time
  losses and terminations; Elo is context only.
- **What it is for:** whether the compact kernel converts clean tablebase
  wins and keeps its clock with tables loaded. No game has had tables on
  this kernel (D.9.3); the read is the check E.1 and E.3.4 now require of a
  release head, run first on the `d9` head so that a defect shows before
  the release, not at it. A failure opens a defect; a pass changes nothing
  and the check stands for the release.
- **Arms:** `tools/test_engines/rarog-d94head-pext-pgo.exe` (SHA-256
  `A859A05CB7FF7BEDDC9CEFB8BCBDB4831415C1BE8183165433E3053DDD472896`,
  `tools/build_test.ps1 -Suffix d94head` at `4ad82cf` on `d9`, a clean
  tree; `id name Rarog dev-20261127-4ad82cf8`; `bench 13` 3,694,298; its
  engine source is `cc08bc3`'s) against
  `tools/test_engines/rarog-v270-pext-pgo.exe` (SHA-256
  `9B68BD10F532F6AEB1078255975AFD98FC9DED7D21127DDD40A0138D32FCD62A`,
  the same wrapper with `-SourceRoot` on a worktree of `v2.7.0`,
  `3c57a89`; `id name Rarog 2.7.0`; `bench 13` 9,077,016, the release's
  fingerprint; a rebuild, not the release's own file). Both `pext-pgo`,
  rustc 1.100.0.
- **Conditions:** Colosseum CLI 0.5.0 (pinned) `match` with
  `tools/colosseum/tb-activation.toml`: the endgame cohort
  (`tools/books/endgame_cohort_v1.epd`, 788 positions, 3 to 6 men, every
  start in the tables), 1,500 games, `3+0.03`, Hash 64, one thread,
  `SyzygyPath=D:/chess/tablebases/syzygy3456` on both arms (510 WDL, 510
  DTZ files), seed 20261127, `tools/results/rar-s93-d94`. The dry run
  resolved the policy in every checked field and both arms' tables;
  both binaries were checked to load them (a KQK root scores the
  tablebase win at the first probe).
- **Read:** `tools/diag/tb_activation.py` as of `4ad82cf`, the optimum
  `budget`'s with Rarog's reserve.

## Command

```
./tools/colosseum.ps1 -Mode match -RunFile tools/colosseum/tb-activation.toml -Book tools/books/endgame_cohort_v1.epd -EngineA tools/test_engines/rarog-d94head-pext-pgo.exe -EngineB tools/test_engines/rarog-v270-pext-pgo.exe -NameA d94head -NameB v270 -OptionsA "SyzygyPath=D:/chess/tablebases/syzygy3456" -OptionsB "SyzygyPath=D:/chess/tablebases/syzygy3456" -ExpectBench 3694298,9077016 -Seed 20261127 -Dir tools/results/rar-s93-d94; if ($LASTEXITCODE -eq 0) { python tools/diag/tb_activation.py --run tools/results/rar-s93-d94 --syzygy D:/chess/tablebases/syzygy3456 --json tools/results/rar-s93-d94/tb_activation.json }
```

## Rule, fixed here

The head (`d94head`) passes with: no unconverted clean tablebase win, no
time loss, no fault, every game a normal termination, and a
tablebase-root share above 95% (below it, the wire is broken and the read
says nothing). Moves over the optimum plus the 5 ms box are reported, not
judged (RAR-S93: unsatisfiable for a box checked between Fathom calls). A
2.7.0 failure is recorded, not judged. A head failure is a defect: it is
reported to the maintainer and opens a repair before the release.

## Prediction

- Tablebase-root share about 100% for both sides; 0 unconverted clean
  wins on either; 0 time losses, 0 faults.
- Head mean time per tablebase-root move 30 to 70 ms (RAR-S93's candidate
  56.8 ms under the old clock, whose optimum was higher); moves over the
  optimum plus the box under 2% of the head's.
- Elo near zero or undefined: every start is decided by the tables for both
  (RAR-S93's pairs all split).

## Result

Not read.
