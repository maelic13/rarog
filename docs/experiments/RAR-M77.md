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
