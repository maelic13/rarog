# RAR-P46 — Thread scaling of the B.9 and C heads without games: bench NPS at 1–16 threads and the games' conditions — OBSERVED 2026-10-27: an engine-side loss and a C-only, system-level slow state

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** 2026-10-27, the agent, opening D.2 on the hold GUIDE
  carries from RAR-M74.
- **Question:** RAR-M74 found the C head's in-game NPS at four threads 9.2%
  below the B.9 head's while it is 29.8% above at one (4T/1T 2.60 against
  3.72). Is the loss in the engine's scaling or in the games' conditions?
- **Kind:** zero-game diagnostic reads, not registered. No prediction was
  frozen before the first read, so there is no calibration to append; the
  reads choose what D.2 investigates and decide nothing about strength.
- **Arms:** the binaries the games played, hashes checked before the reads:
  - B.9 head `tools/test_engines/rarog-b9head-pext-pgo.exe` (`aac92114…01ee`,
    `24aefb4`, 11,171,726 bench nodes, rustc 1.98.1);
  - C head `tools/test_engines/rarog-c11head-pext-pgo.exe` (`9750d3de…7fd2`,
    13,987,223, rustc 1.99.1; recipe
    `analysis/arm_patches/a0146a3-rar-m73-c11head.patch`, as RAR-M73).
- **Host:** the match host, AMD Ryzen 9 5950X (16 cores, 32 logical CPUs, two
  eight-core CCDs), idle before each read (2–7% CPU, no engine, harness or
  build process).
- **Instruments:** in ignored `analysis/artifacts/d2-scaling/`, SHA-256
  prefixes beside each name.
  - `run_scaling.ps1` (`619f58f7…`) drives `tools/nps_scaling.ps1`
    (`f2ae7a38…`, at `bd1c1a5`) unchanged: one process per search, unpinned,
    Hash 256, `go movetime 5000`, its two pinned middlegame FENs, two
    repetitions, threads 1/2/4/8/16; four passes in ABBA order; data
    `scaling.csv` (`16543d4b…`), summary by `summarise.py` (`9bfcbddf…`).
  - `conditions.py` (`018a57b3…`): Hash 512 and 4T as the games, one
    process playing a run of searches on the same two FENs with no
    `ucinewgame` between them (40 at `movetime 100`, or 4 at 5,000). The
    concurrent cell starts three processes of one arm at once, pinned to the
    games' CPU sets 2–9, 16–23 and 24–31. NPS is the engine's own: nodes
    over time on each search's last `info … nodes` line. Working set after
    `isready` tells a large-page table (about 7 MiB) from an ordinary one
    (648 MiB).
  - `commit_check.py` (`b3a2fda6…`): working set, commit and page faults
    after `isready` and after one search.

## Results

**1. Bench, one process, 5 s searches** (eight readings a cell; pooled
median, speedup over the arm's own 1T median):

| Threads | B.9 head | C head | C over B.9 |
|---|---|---|---|
| 1 | 1.53 M | 2.14 M | +40.0% |
| 2 | 2.94 M, ×1.92 | 3.45 M, ×1.61 | +17.4% |
| 4 | 5.82 M, ×3.80 | 7.04 M, ×3.28 | +21.0% |
| 8 | 11.49 M, ×7.50 | 13.13 M, ×6.12 | +14.3% |
| 16 | 20.43 M, ×13.34 | 23.08 M, ×10.76 | +12.9% |

- Per-thread efficiency at 4T is 82% for C against 95% for B.9, and the
  loss is already present at two threads.
- One C reading at 16T is 1.08 M, an outlier the median absorbs; its cause
  was not captured.

**2. Game-like conditions, `conditions.py`, first run** (two passes per
arm, ABBA):

| Cell | B.9 head | C head | C over B.9 |
|---|---|---|---|
| 100 ms, 1T, one process | 1.74 M | 2.24 M | +28.7% |
| 100 ms, 4T, one process | 6.40 M | 7.93 M | +23.8% |
| 100 ms, 4T, three pinned | 6.02 M | 6.24 M | +3.6% |
| 5 s, 4T, three pinned | 5.10 M | 6.17 M | +21.0% |

- At 100 ms in one process the two heads scale alike (4T/1T 3.68 and 3.54).
- At 5 s three concurrent pinned processes cost both heads about 12%.
- The 100 ms concurrent cell is where C changes. It is **bimodal by pass,
  and all three processes of a pass share the state**: 5.37–5.49 M in one
  pass, 6.99–7.41 M in the other. B.9 stays at 5.8–6.3 M.

**3. That cell repeated, four passes per arm alternating:**
- With the readers as threads of one Python process: C 5.56 M median
  against B.9 5.93 M (−6.4%); C slow in three of four passes.
- With one reader process per engine: C 7.42 M against 6.05 M (+22.6%); C
  slow (5.51–5.74 M) in one of four passes, fast (7.28–7.61 M) in three.
- Across the three runs C was slow in 5 of 12 passes. Its slow state,
  about 5.5 M, is 7–10% below B.9, the in-game figure; its fast state is 23%
  above. B.9 shows no such state in 12 passes.
- Every C process held its table in large pages (working set 6–7 MiB) in
  both states. B.9 never uses large pages and touches its whole table during
  `isready` (363,451 faults).

**4. Instrument checks:**
- **Node accounting is unchanged.** `src/search/shared.rs` differs between
  the two heads only by `fetch_update` renamed `try_update`, so the ratios
  are not an accounting artefact.
- **No per-search large-page allocation.** Helpers are persistent threads
  that build their tables once at spawn, and both tables are written in full
  when built. So no first-touch fault cost lands inside a search in either
  large-page mode.
- **The wall-clock gap was the harness.** The threaded harness showed about
  75 ms more wall time than engine time per 100 ms C search. With one reader
  process per engine it is 24 ms, the same as B.9 (26 ms), the tail after
  the last `info` line. So no clock is burned outside the engine's timer.
  Engine-reported NPS does not depend on the reader, but the threaded runs
  may also have perturbed scheduling.

## Disposition

- **Both, in two parts.**
  - **The engine's own scaling fell:** C's 4T/1T on one unpinned process is
    3.28 against 3.80, already at 2T. In that bench C is still 21% faster
    than B.9 at 4T.
  - **The games' conditions add a slow state only C enters:** with three
    concurrent pinned processes at the games' search length, C is either
    23% faster than B.9 or 7–10% slower, and the slow state matches the
    games' −9.2%.
- **The slow state is system-level** (shared by the processes started
  together) **and not attributed.**
  - Large pages are the one system resource C uses and B.9 does not. The
    state did not coincide with large pages failing: every process had
    them.
  - There is no switch to turn them off: no UCI option, no environment
    variable.
  - Not excluded: the two pinned slots that share CCD1's L3, frequency or
    power state, and physical memory placement.
- **D.2 now has a measured target in the engine, the 2T–4T efficiency loss,
  with a system-level component on top.** RAR-M46's contradiction was for
  2.4.0 and does not cover this.

## Retry trigger and next read

- **Discriminate large pages first.** Build an arm of the C head (or the
  `d2` head, which has the same tables and allocator) whose allocator never
  requests large pages, otherwise identical, at the same fingerprint. Repeat
  the concurrent 100 ms cell at eight or more passes per arm with one reader
  process per engine; then `tools/nps_scaling.ps1` on it.
  - If the slow state vanishes with large pages off, the allocator owns it.
  - If it persists, look at L3 sharing (slots on one CCD against split) and
    frequency.
- **The engine-side loss at 2T** needs a profile of shared writes (TT stores,
  correction and history sharing, shared atomics) across the heads. That is
  D.2's research, after the system-level read above.
