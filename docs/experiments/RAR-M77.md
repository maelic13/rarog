# RAR-M77 — `LargePages` on against off in games at four threads, RAR-M74's conditions — REGISTERED 2026-10-27, before any game

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-27 by the agent; the maintainer runs
  it overnight.
- **Question:**
  - RAR-M74 found the C head's in-game NPS at four threads 9.2% under the B.9
    head's.
  - Zero-game reads placed most of that in a system-level slow state the C
    head alone enters with three concurrent four-thread processes
    ([RAR-P46](RAR-P46.md)).
  - That state appeared only with large pages on ([RAR-P47](RAR-P47.md): 2
    of 16 passes against 0 of 16, weak evidence).
  - In games, under RAR-M74's conditions, does `LargePages` off against on
    change in-game NPS and strength?
- **Kind:** a fixed match, a measurement with an interval (`colosseum.ps1
  -Mode match`), not an acceptance gate. It informs D.2 and the option's
  default; it changes neither.
- **Binary:** both sides run `tools/test_engines/rarog-d2lp-pext-pgo.exe`
  (`ebbbc917…7032`; `id name Rarog 2.7.0-dev`).
  - Built from `2b7bca9` on `d2` with `cargo xtask build --arch pext --pgo`,
    rustc 1.99.1.
  - Sidecar fingerprint **13,987,223**, which the harness checks on each side
    (`-ExpectBench`), along with the revision (`-ExpectRevision`).
  - The arms differ only by the option: A `LargePages=true`, B
    `LargePages=false`. So no build offset separates them, and they share
    every machine state of the night.
  - The option's live wire is proved in RAR-P47 (working set 6.8 against
    648 MiB at Hash 512).
- **Conditions:**
  - RAR-M74's four-thread setup: Threads 4, Hash 512, `3+0.03`, three games
    at a time, `--cores-per-game 4`. The dry run puts the slots on CPUs 2–9,
    16–23 and 24–31, enforced, as RAR-M74.
  - UHO book, random with seed 77; no adjudication; **4,000 games**.
  - Margin 20 ms, the Rarog match policy. RAR-M74 used 2,000 ms for the
    older engines in its field. The margin is only the time-loss tolerance.
  - Runner: Colosseum CLI 0.4.0, the pinned one.
  - Driver `analysis/artifacts/d2-largepages-ingame/run_m77.ps1`
    (`1489d3a0…`), started by `start_when_idle.ps1` (`7d7192c8…`; first
    registered as `bc3eeacb…`, before its `-DryRun` pass-through was added,
    before any game).
  - Dry run 2026-10-27, exit 0, every guard passed (host 1%, runner pin,
    both fingerprints, flavour and compiler, policy). `dry-run.json`
    SHA-256 `07c96769…7cf1`; the run's manifest must repeat its
    configuration.
- **Host at registration:** the 5950X match host, up 91 h since
  2026-10-23 16:50 UTC, 102 of 128 GB free. Memory fragmentation may matter
  to large pages, so the machine is not rebooted for this run.
- **Readings:**
  1. **Primary:** in-game NPS per side, Colosseum's `stats <run dir>`
     "total nodes over total time", as RAR-M74 quoted it. Reported as the
     ratio on/off, with RAR-M74's C head (6,468,917) and RAR-M66's B.9 head
     (7,123,826) beside it.
  2. Colosseum's Elo for A (on) over B (off), with its interval; at 4,000
     games about ±7.
  3. Time losses and faults per side; mean depth per side.

## Prediction, frozen before any game

| On/off NPS ratio | Probability |
|---|---|
| Under 0.95: the slow state is common in games | 0.50 |
| 0.95 to 1.00 | 0.15 |
| Over 1.00: large pages' ordinary gain of 1–4% | 0.35 |

- The off side's NPS: about 7.0–7.5 M, near the C head's zero-game
  concurrent level.
- **Elo, A (on) over B (off):**
  - interval wholly below zero (off stronger): 0.30;
  - interval covering zero: 0.65;
  - interval wholly above zero: 0.05.
- **Most likely failure mode:** the slow state is rare in games too. Then
  RAR-M74's −9.2% has another cause, such as the opponents' load on the
  shared slots, which a self-match does not have.

## Decision and stop rule

- **Fixed 4,000 games.** No early stop except a harness fault. An
  interrupted run resumes with the same command and seed.
- **Ratio under 0.95 and Elo interval below zero:**
  - Large pages cost strength in the four-thread rated conditions.
  - I recommend default off for multi-threaded use. The maintainer decides
    the default; a changed default is a change in the rated configuration
    and takes its own registered gate unless the maintainer accepts this
    read for it.
  - D.2 then researches why: large-page placement across processes, against
    the CCD's L3.
- **Ratio at least 0.98 and the interval covering zero or above it:**
  - Large pages are not the four-thread problem in games, and the default
    stays.
  - D.2 turns to the L3 and frequency explanations, and to the engine's own
    2T loss.
  - RAR-M74's −9.2% gets a separate question.
- **Anything else:** recorded as unresolved, with no further run until a
  retry trigger is named.

## Result (played 2026-10-28)

- **The run:**
  - Started 06:56:59 UTC, at host 5.4% busy.
  - The maintainer stopped it at 09:18:19 UTC at a committed boundary,
    2,578 games scored.
  - Resumed with `run_m77.ps1` at 22:31:20 UTC at host 0.0%, same seed 77;
    completed 23:22:02 UTC.
  - Each manifest repeats the registered `dry_run_sha256` `07C96769…7CF1`.
    The final manifest is
    `tools/results/colosseum_match_rar-m77-largepages-4t_20261028_233119.manifest.txt`:
    exit 0, completed, 4,000 scored games.
- **Faults:** none on either side (time 0-0, other 0-0, infrastructure 0).
- **Terminations:** checkmate 1,917, threefold 1,777, insufficient
  material 243, fifty-move 46, stalemate 17.
- **Elo, A (large pages on) over B (off): +10.3 ± 5.9**, nElo +18.9 ± 10.8,
  over 2,000 pairs.
  - W-D-L 1,018-2,083-899; pentanomial [21, 439, 978, 524, 38], which the
    manifest's recount of `games.pgn` matches.
  - `result.json` `EA8379ED…`, `games.pgn` `BCEA3C69…`.
- **In-game NPS per side: on 8,080,765, off 7,866,935; on/off 1.027.**
  - Mean depth 13.21 against 13.21.
  - Implied NPS median 8.38 M against 8.18 M.
- **The instrument had a gap.**
  - `colosseum.ps1` does not pass `-NameA`/`-NameB` to the CLI's
    `--a-label`/`--b-label`, so both arms ran as `rarog-d2lp-pext-pgo`, and
    Colosseum's search statistics merge them: 7,973,581 nps over 518,279
    moves.
  - The scoring is unaffected: the runner keeps A and B apart.
  - Per side: `analysis/artifacts/d2-largepages-ingame/relabel_pgn.py`
    (`dd9e02b1…`) copies `games.pgn` with only the two player tags renamed,
    from the journal's per-game `white`/`black`. It asserts each game's tags
    and the 4,000-game count. Colosseum's own `stats` then counts the copy:
    `m77_by_arm.pgn` (`0775411e…`), report `m77_by_arm_stats.txt`
    (`c8465bf0…`).
  - The copy reproduces the runner's record exactly. On as White
    601-1,046-353, as Black 417-1,037-546. Its per-side moves, 259,171 on and
    259,108 off, sum to the runner's 518,279.

## Disposition

- **The registered branch: ratio at least 0.98, interval above zero.**
  - In games under RAR-M74's four-thread conditions, large pages are worth
    +2.7% NPS and +10.3 ± 5.9 Elo (95%: +4.4 to +16.2).
  - Large pages are not the four-thread problem in games, and
    **`LargePages` stays on by default.**
- **The zero-game slow state of RAR-P46 and RAR-P47 does not show in these
  games.**
  - The on side's in-game NPS, 8.08 M, is above the off side's and 13%
    above RAR-M66's B.9 head (7.12 M).
  - The C head in RAR-M74's gauntlet read 6.47 M.
  - Colosseum keeps each slot's two engine processes from game to game, so
    a run starts three pairs of processes, not one per game. If the slow
    state belongs to concurrent starts, games rarely meet it.
- **RAR-M74's −9.2% is not reproduced by the same head in a self-match
  under the same conditions.** It becomes a separate question.
  - What differs is the opponents: Houdini 3, Critter 1.6a, Fritz 16,
    Rybka 4.1, Basilisk and Rarog 2.4.0 share the slot's four cores and
    their SMT siblings.
  - Candidate: an opponent whose idle threads keep running while Rarog
    searches. This is an untested candidate, not a finding.
- **D.2 continues** with the engine's own 2T–4T efficiency loss (RAR-P46)
  and the opponent question above.

## Calibration

- **NPS ratio:** the outcome is the "over 1.00" branch, given 0.35. I had
  put 0.50 on a common slow state, extrapolating RAR-P46 and RAR-P47's
  fresh-start passes to games.
  - The miss is in mechanism: the zero-game cell starts three new processes
    every pass, and a match keeps its processes.
- **Elo:** the interval lies wholly above zero, given 0.05.
  - The sign follows from the NPS result.
  - +10.3 for +2.7% is above the about 5 that 2 Elo per 1% at one thread
    gives, with that value inside the interval.
- **Instrument:** the arms' names were not checked in the dry run. The
  resolved configuration did not name them, and the summary line read
  `rarog-d2lp-pext-pgo vs. rarog-d2lp-pext-pgo` only once games began.
  Repaired after the fact as above; the wrapper fix is a tooling change of
  its own.
