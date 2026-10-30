# RAR-M78 — Self-relative scaling: Rarog, Reckless and Stockfish, each against itself at four threads and at twice the time — REGISTERED 2026-10-30, before any game

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
