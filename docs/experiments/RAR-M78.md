# RAR-M78 — Self-relative scaling: Rarog, Reckless and Stockfish, each against itself at four threads and at twice the time — RAROG DEFICIENT 2026-11-01: effective speedup 1.65 against Reckless 2.30 and Stockfish 2.48

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-30 by the agent; the maintainer runs
  it (one night, 12 to 13 hours).
- **Question:** is Rarog's lazy SMP weak at four threads against engines
  that built theirs for it? This is PLAN D.2's self-relative scaling games
  (maintainer steer 2026-09-11): each engine plays itself, so strength,
  evaluation and NPS accounting cancel.
- **Why time odds as well (maintainer decision 2026-10-30).** A raw Elo gain
  from four threads is not comparable between engines.
  - A weaker engine gains more Elo from the same extra search: Rarog about
    145 Elo for a doubling of time at `3+0.03`, top engines far less.
  - So Rarog could show the largest four-thread gain of the three with the
    weakest SMP.
  - Each engine therefore also plays itself at twice the time on one
    thread. That measures its own Elo per doubling, and the four-thread
    gain divided by it is an effective speedup, comparable across engines.
- **What it does not answer:** whether the C head lost Elo to its 6–7%
  four-thread shared-search loss against B.9 (RAR-P49, RAR-P50).
  - At `10+0.1` that loss is a few percent of NPS at most, worth a few Elo,
    far inside this design's error.
  - RAR-M76's note says why a match that could see it would take days.
- **Kind:** fixed-length matches with intervals; they decide no gate and no
  default. Their outcome decides whether D.2 has a four-thread SMP target.

## Design

- **Engines:**

  | Engine | Binary | SHA-256 | Options |
  |---|---|---|---|
  | Rarog (the C head) | `tools/test_engines/rarog-d2lp-pext-pgo.exe` | `EBBBC917…7032` | defaults (large pages on, `Move Overhead` 10) |
  | Reckless 0.9.0 | `D:/chess/engines/reckless-windows-avx2.exe` | `B74EAD56…91B8` | `MoveOverhead` 10 (PROCESS, *Harness*: opponents' clock options) |
  | Stockfish 18 | `D:/chess/engines/stockfish/stockfish-18-windows-x86-64-bmi2.exe` | `BF2D8BF6…3288` | defaults (`Move Overhead` already 10) |

  - Stockfish 18's fixed bmi2 tier is the build the PLAN names. It avoids
    a run-time dispatcher.
  - Reckless 0.9.0 is the donor build already in the library.
- **Two legs per engine**, each a 1,000-game `match` of the engine against
  itself (one binary on both sides):

  | Leg | Arm A | Arm B | Clock | Games at a time | Seed |
  |---|---|---|---|---|---|
  | `odds` | 1 thread, `10+0.1` | 1 thread, `20+0.2` | per side | 14 | 71, 72, 73 |
  | `threads` | 1 thread | 4 threads | `10+0.1` | 3, each slot owning four physical cores (`--cores-per-game 4`) | 61, 62, 63 |

- **Common conditions:**
  - Hash 256 on every side. That is ample at `10+0.1` even at four threads,
    and it keeps the odds legs' 28 Rarog processes within reliable
    large-page allocation.
  - `match-fixed-ltc.toml` with the UHO book, random order, no adjudication.
  - Colosseum CLI 0.4.0 (pinned).
- **Harness:**
  - `tools/colosseum.ps1`, with `-ThreadsB` (`f077997`) and `-BaseMsB` and
    `-IncrementMsB` (`9b6aec8`). The guard suite watches each fail and
    pass: 69/69.
  - The policy check reads each side's resolved threads and clock from the
    runner's own dry run.
  - **The first dry run caught a wrapper defect.** `-Hash` reached the
    engines only when a side had its own options, so the odds legs would
    have resolved the run file's Hash 64. The policy check refused them
    before any game.
    - Each side now always states its Hash and Threads (`150c481`).
    - The case that rested on the omission was rebased first (`5b57833`).
    - No earlier run was affected: the same check refused every run the
      omission touched.
- **Dry runs, 2026-10-30, after the fix** (`run.ps1 -DryRun`): all six
  legs resolve as designed.
  - Odds legs: 10000+100 against 20000+200, 14 games at a time.
  - Threads legs: 1 thread against 4, three slots of four physical cores
    each inside one CCD (CPUs 2–9, 16–23, 24–31).
  - Everywhere: Hash 256 on both sides, Reckless's `MoveOverhead=10` on
    both sides, the UHO book, seeds 61–63 and 71–73.
  - The driver's self-test showed live streaming, and that a failing leg
    stops the sequence.
- **Drivers:**
  - `analysis/artifacts/selfscale/leg.ps1` (one leg) and `run.ps1` (all
    six).
  - `run.ps1` checks the three hashes, then for each leg waits for an idle
    host (at most 10% CPU and no engine, harness or build process), plays
    it, and tees Colosseum's output.
  - A finished leg is skipped on a rerun. The odds legs run first, so a
    night cut short still leaves whole engines.
- **Runtime:**
  - RAR-M76's `10+0.1` games averaged about 30 s, so each threads leg takes
    about 3 hours at three games at a time.
  - An odds game has about 1.5 times the clock, so each odds leg takes
    about 1 hour at fourteen at a time.
  - 12 to 13 hours in all.

## Readings

- **G4** = arm B's Elo over arm A in the threads leg, the gain from four
  threads.
- **G2** = arm B's Elo over arm A in the odds leg, the gain from one
  doubling of time.
- **D = G4 / G2**: four threads in doublings of the engine's own time. The
  effective speedup is **X = 2^D**.
- **Uncertainty:**
  - Each Elo's standard error is taken from Colosseum's reported 95%
    interval (half-width / 1.96).
  - D's is computed by the delta method:
    var(D) = σ4² / G2² + G4² σ2² / G2⁴.
  - A difference of two engines' D adds their variances.
- Reported in Elo and nElo. The outcome is judged on Elo, the unit both
  legs share.

## Prediction, frozen before any game

| Engine | G2 (Elo) | G4 (Elo) | D | X |
|---|---|---|---|---|
| Rarog | +110 [+80, +140] | +140 [+100, +180] | 1.3 [1.0, 1.6] | 2.5 |
| Reckless | +80 [+50, +110] | +110 [+70, +150] | 1.4 [1.1, 1.7] | 2.6 |
| Stockfish | +70 [+40, +100] | +100 [+60, +140] | 1.45 [1.2, 1.7] | 2.7 |

- **Outcome probabilities:**
  - **Rarog deficient**, as defined below: 0.25.
  - **Not shown deficient**: 0.75.
- **Reasons:**
  - Rarog's four-thread NPS speedup is about 3.6 to 3.8 at these move times
    (RAR-P46's short cells, RAR-P49's smoke). That is close to B.9's, and
    the 5 s loss barely reaches `10+0.1`.
  - Its lazy SMP is plain: no thread voting, no depth staggering. That
    leaves room for a real gap in what the extra nodes are worth.
  - Modern engines' four-thread effective speedups are taken as about 2.5
    to 3. That is a prior, not a measurement here.
- **Confidence:** low on the levels, moderate on the outcome. No engine's
  four-thread or doubling gain has been measured on this host at this
  control.

## Decision and stop rule

- **Rarog deficient:**
  - Rarog's D is below both Reckless's and Stockfish's, and both
    differences have their 95% upper bound below zero.
  - D.2 then opens a four-thread SMP research card on what the extra
    threads' nodes are worth: helper depth, shared-table use, root move
    diversity.
  - Expected value is stated before any engine work.
- **Not shown deficient:** anything else.
  - D.2's four-thread SMP target stays closed. The RAR-P46 to RAR-P50 loss
    stays recorded and unfixed.
  - Scaling beyond four threads stays with G.1.
- **Stopping and repeats:**
  - Six legs, played to completion.
  - An interrupted leg resumes by the same command at its seed. A leg
    abandoned for an engine fault is reported, not replaced.
  - No leg's clock, threads, Hash, book or game count changes after any
    game. Legs are not added after seeing results.
- **Time losses:** a leg with over 0.5% losses on time
  (`TimeLossRateCeiling`) is reported as such, and its D is not judged.

## How to run

From PowerShell 7 on an idle host, overnight:

```powershell
pwsh -File D:\code\rarog\analysis\artifacts\selfscale\run.ps1
```

It prints every idle check and Colosseum's progress, and resumes after an
interruption with the same command.

## Result — 2026-11-01, appended after exposure

- **The run.** The maintainer ran `run.ps1` overnight from 2026-10-31 23:17 to
  2026-11-01 07:53; the Stockfish threads leg was interrupted at 680 games
  (10:02) and resumed by the same command at 15:56, finishing at 16:56, as the
  stop rule allows. Six legs of 1,000 games, every pentanomial recounted from
  `games.pgn` by the wrapper, zero time losses, zero engine faults, zero
  infrastructure faults. Hashes as registered (Rarog `EBBBC917…`, Reckless
  `B74EAD56…`, Stockfish `BF2D8BF6…`); Hash 256 and the registered clocks and
  threads on every side
  (`tools/results/colosseum_match_selfscale-*.manifest.txt`; run directories
  `tools/results/selfscale-*/`).
- **Colosseum's results** (arm A's Elo; the gain is arm B over A):

  | Leg | Elo (A) | nElo | Pentanomial | W/D/L (A) |
  |---|---|---|---|---|
  | Rarog `10+0.1` against `20+0.2` | −101.8 ± 11.5 | −201.1 ± 21.5 | 33, 246, 194, 27, 0 | 109/497/394 |
  | Rarog 1T against 4T | −73.7 ± 11.5 | −142.0 ± 21.5 | 16, 231, 199, 54, 0 | 124/543/333 |
  | Reckless `10+0.1` against `20+0.2` | −86.2 ± 9.8 | −197.4 ± 21.5 | 2, 266, 205, 27, 0 | 128/501/371 |
  | Reckless 1T against 4T | −103.7 ± 9.2 | −256.9 ± 21.5 | 1, 303, 181, 15, 0 | 91/528/381 |
  | Stockfish `10+0.1` against `20+0.2` | −86.9 ± 9.9 | −197.9 ± 21.5 | 2, 269, 201, 28, 0 | 131/493/376 |
  | Stockfish 1T against 4T | −114.1 ± 9.3 | −282.5 ± 21.5 | 6, 317, 165, 12, 0 | 99/485/416 |

- **Readings** (delta method, σ from the 95% half-widths):

  | Engine | G2 (Elo) | G4 (Elo) | D = G4/G2, 95% | X = 2^D |
  |---|---|---|---|---|
  | Rarog | +101.8 ± 11.5 | +73.7 ± 11.5 | 0.72 [0.58, 0.86] | 1.65 |
  | Reckless | +86.2 ± 9.8 | +103.7 ± 9.2 | 1.20 [1.03, 1.38] | 2.30 |
  | Stockfish | +86.9 ± 9.9 | +114.1 ± 9.3 | 1.31 [1.13, 1.50] | 2.48 |

- **Outcome: Rarog deficient**, by the registered rule. Rarog's D minus
  Reckless's is -0.48 ± 0.22 (upper bound -0.26) and minus Stockfish's -0.59 ±
  0.23 (upper bound -0.36): both intervals lie below zero.
- Four threads are worth 0.72 of a doubling of time to Rarog and 1.2 to 1.3 to
  the other two: Rarog gains from four threads what the others gain from about
  1.6 times the time, where they gain what 2.3 to 2.5 times the time buys.
- **What the runner's own search statistics add** (the manifests' `search` and
  `harness_hold_ms` lines; observations, not registered readings). Rarog's
  four-thread side reached a *lower* mean depth than its one-thread side,
  15.41 against 17.28 (non-mate 15.84 against 16.87), at 6.29 M against 2.76 M
  nps (2.28× in games, against 3.28 in RAR-P46's bench cells); Reckless went
  20.01 → 21.33 at 3.45× nps and Stockfish 26.37 → 29.13 at 3.47×. Rarog's
  four-thread process was also held outside its own clock far more than anyone
  else's: 18,171 of 64,486 moves over the 20 ms margin (28%), p50 4 ms, p99
  175 ms, against 2% and p99 53 ms at one thread, and p99 4 ms (Reckless) and
  1 ms (Stockfish) at four. Reckless's and Stockfish's `20+0.2` arms gained
  3.1 and 4.2 plies over `10+0.1`; Rarog's gained 0.9.

- **Confound, found 2026-11-01 at 17:40 and reported here:** a review
  subagent's stray `python -I -` process (a line-counting script fed through a
  malformed heredoc) ran on one logical CPU from 13:49 until it was killed, so
  the Stockfish threads leg's resumed games (15:56 to 16:56, 320 of its 1,000)
  played beside one busy core of 32. `run.ps1`'s idle check allows 10% load
  and passed. The interim read at 680 games, −110.1 ± 11.4, agrees with the
  final −114.1 ± 9.3, and the other five legs ran before 13:49, so the outcome
  is unchanged; the leg is recorded as not fully idle.

## Disposition

- **Rarog deficient:** D.2's four-thread SMP target is open, and D.2 opens the
  research card the registration named, on what the extra threads' nodes are
  worth. The depth and hold figures above order its first hypotheses: (1) the
  PGO profile covers only the single-thread copy of the kernel, so at four
  threads the main thread itself runs untrained code
  (`analysis/engine_review_2026-11-01.md` §3.3, RUST-1 / DRIVER-3 / DRIVER-4;
  one build decides it); (2) time-to-depth at four threads against one on
  RAR-R10's harness, since in-game NPS scaled 2.28× while depth fell; (3) the
  four-thread process's holds outside its clock (the join of helpers and the
  bestmove path, `analysis/engine_review_2026-11-01.md` DRIVER-14 and
  DRIVER-15), which cost thinking time the budget never sees; (4) the shared
  backend's per-operation cost (F16); then the card's own questions, helper
  depth, shared-table use and root diversity.
- Scaling beyond four threads stays with G.1. Nothing about a default or a
  gate follows from this read; a fix is gated on its own.

## Calibration

- **The registered outcome missed.** "Rarog deficient" was given 0.25 and
  happened; the miss is in sign of the comparison and in magnitude.
- **Where the levels were wrong.** Rarog's G2 (+101.8) sits inside its band
  [+80, +140]; its G4 (+73.7) sits below the band [+100, +180], so the miss is
  entirely in Rarog's four-thread gain. Reckless's and Stockfish's four
  readings all fell inside their bands (G4 +103.7 and +114.1 against [+70,
  +150] and [+60, +140]; G2 +86.2 and +86.9 against [+50, +110] and [+40,
  +100]).
- **The mechanism assumed was wrong.** The registration reasoned from a bench
  NPS speedup of 3.6 to 3.8 that the four-thread gain would be near the
  others'. In games the speedup was 2.28× and the depth fell, so the NPS proxy
  did not carry: the lazy SMP's defect is in what the nodes buy and in how the
  process behaves under the harness, not only in the count of nodes. RAR-P50's
  recommendation against pursuing the four-thread loss is withdrawn by this
  read.
- **Instrument.** The design worked as intended: each engine was its own
  control, the time-odds leg priced the threads in the engine's own doublings,
  and the Stockfish threads leg's interruption and resumption left the
  registered 1,000 games intact.

## Later reads

- **2026-11-02, [RAR-R31](RAR-R31.md):** twelve of this leg's games replayed
  position by position with their clocks and kept tables, alone: four
  threads report the same depth as one (+0.08 plies) at 3.78 times its NPS.
  In the games the four-thread side ran at 2.28 times and, on the same
  moves, at about 60% of its speed alone, reporting 1.38 plies less. The
  deficit this experiment measured is real under its conditions, and the
  conditions carry it: they cost Rarog's four-thread search about 40% of its
  speed and Reckless's far less. Its cause is D.2's H7, open.
