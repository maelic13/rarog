# RAR-S140 — The with-tables activation read on the `d9` head against 2.7.0: RAR-S93's design as E.1's release check — REGISTERED 2026-11-27, before any game; READ 2026-11-27: passed

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-27 by the agent, before any game;
  the games are the maintainer's command.
- Disposition: accepted — read 2026-11-27: the head passed; 464 of 464 clean tablebase
  wins converted, no time loss, no fault, every game normal, every move from a tablebase root.
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

Played 2026-11-27 by the maintainer with the command above, 12:08:48 to
12:11:38 UTC (`tools/results/rar-s93-d94`; manifest
`tools/results/colosseum_match_rar-s93-d94_20261127_130848.manifest.txt`,
repository at `9b71300`, clean; host 0.4% at the start). The manifest
names both registered binaries by SHA-256 and `bench`, `SyzygyPath` on both
(510 WDL, 510 DTZ files), 1,500 games, exit 0.

- **Colosseum's report:** 750 pairs, pentanomial [0, 0, 750, 0, 0] (the
  recount of `games.pgn` agrees); each side 464 wins as White and 464
  losses as Black, 286 draws each way; terminations Checkmate 928,
  InsufficientMaterial 384, Threefold 106, FiftyMove 56, Stalemate 26;
  time losses 0 and 0; faults none.
- **`tb_activation.py`** (exit 0): every game a normal termination;
  tablebase-root moves 25,472 of 25,472 for the head and 25,391 of 25,391
  for 2.7.0; clean-win games 464 each, all 464 converted, 0 unconverted;
  head mean 46.2 ms a tablebase-root move (max 133), 2.7.0 43.4 ms (max
  152); over the optimum plus the box, reported only, the head 31 of
  25,472 moves (0.12%) and 2.7.0 4,986 (19.6%), 2.7.0 being held to a
  clock that is not its own.
- **Context, not judged:** the head 24.24 plies deep on average against
  2.7.0's 20.21 at 3.30M against 3.49M nodes a second; Colosseum's
  `harness_hold_ms` (the charged time less the last `info` line's,
  PROCESS) p99 81 ms for both.
- **Verdict by the rule: passed.** No unconverted clean win, no time loss,
  no fault, every game normal, and a tablebase-root share of 100%.

## Calibration

Every prediction held: the share, the conversions, no time loss or fault,
the head's mean time inside 30 to 70 ms, moves over the optimum plus the
box under 2% (0.12%), and Elo undefined at zero variance. The prediction
for 2.7.0's over-optimum share was not made; it follows from judging the
old clock by the new optimum, as the smoke on RAR-S93's games showed.
