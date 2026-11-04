# RAR-P49 — The C head's thread-scaling loss, hardware or shared search, read per position over one fixed window — SHARED SEARCH 2026-10-30: the whole loss is in the shared search, none in the hardware

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-29 by the agent; the maintainer runs
  it (about 23 minutes).
- **Question:** [RAR-P48](RAR-P48.md) found the C head's 4T loss real under
  pinning, and neither the CCD split nor large pages. Its split between
  hardware and shared search was left unresolved by two instrument
  confounds. Does the loss live in the hardware the threads share or in the
  shared search?
- **Retry trigger fired:** this is the per-position, fixed-window instrument
  that RAR-P48's disposition named.
- **Kind:** zero-game diagnostic. It chooses D.2's next investigation and
  decides nothing about strength, a default or a gate.

## What changed from RAR-P48

- **One fixed window.**
  - Every search runs `go movetime 10000` and keeps every `info … nodes`
    line.
  - A search's NPS is its node count at exactly 5,000 ms, interpolated
    between the lines either side, divided by 5 s.
  - Up to 5 s, a 10 s search is the same search as a 5 s one, because
    `movetime` sets one limit.
  - `go nodes` was rejected as the window: with a node limit set, every node
    reads the shared stop state and the shared node counter
    (`src/search/mod.rs`, `check_stop` and `record_node`). That adds
    shared-memory traffic to exactly the threaded cells under test.
- **Per position.** Every quantity is read at each position separately,
  and the outcome needs both positions to agree.
- **Fewer arms and cells.**
  - RAR-P48 refuted large pages and the CCD split, so the `c-lp-on` arm and
    the `-x` cells are dropped.
  - `c-lp-off` is the C arm, because large pages change concurrent
    processes' speed (RAR-P48's 4x1T), and so would move H.

## Design

- **Arms:**
  - `b9head`: `tools/test_engines/rarog-b9head-pext-pgo.exe` (`aac92114…`);
  - `c-lp-off`: `tools/test_engines/rarog-d2lp-pext-pgo.exe` (`ebbbc917…`,
    `2b7bca9`, bench 13,987,223) with `LargePages false`.
- **Instrument:** `analysis/artifacts/d2-scaling/split_read2.py`
  (`67936536…`, ignored).
  - Every search is a fresh engine process, Hash 256, on RAR-P46's two
    positions.
  - One reader process per engine. Concurrent processes pass a barrier
    before `go`.
  - Each process's affinity mask is read back and must equal the one set.
  - The interpolation and the judgement below are checked on synthetic
    data by `split_read2_check.py` (`1d4698fd…`). Every outcome is
    produced as designed.
- **Placement:** one logical CPU per thread, each on its own physical core
  of CCD0 (core *k* is CPUs 2*k* and 2*k*+1).
- **Cells,** per arm and position:

  | Cell | Processes × threads | CPUs |
  |---|---|---|
  | `1T` | 1 × 1 | 2 |
  | `2x1T` | 2 × 1, together | 2; 4 |
  | `4x1T` | 4 × 1, together | 2; 4; 6; 8 |
  | `2T` | 1 × 2 | 2, 4 |
  | `4T` | 1 × 4 | 2, 4, 6, 8 |

- **Order:**
  - Six rounds, alternating the arms' order, so each arm runs first three
    times.
  - A 10 s settle before each arm's block, and a timestamp on every reading.
  - Six readings per arm, cell and position for the threaded cells, and
    6 × *k* process readings for each *k*×1T cell.
- **Host:** the 5950X. The instrument refuses to start above 15% load, and
  records the load before and after the read.

## Readings and judgement

- Per arm and position, from medians:
  - **S** = NPS(*k*T) / NPS(1T), and **E** = S / *k*;
  - **H** = per-process NPS(*k*×1T) / NPS(1T), what the shared hardware
    costs each thread;
  - **Q** = S / (*k* · H), what sharing the search costs beyond the
    hardware.
- **At `4T`, per position,** take the log gaps of C against B.9:
  - ΔE = ln(E_C / E_B9), ΔH = ln(H_C / H_B9), ΔQ = ln(Q_C / Q_B9).
  - E = H · Q, so ΔE = ΔH + ΔQ exactly.
  - The shared-search share is ΔQ / ΔE.
- **Position verdict:**
  - **not reproduced** if ΔE > ln 0.96 (C's E within 4% of B.9's);
  - otherwise **shared search** if the share is at least 2/3;
  - **hardware** if it is at most 1/3;
  - **both** between.
- **Outcome:** the position verdict when both positions agree, otherwise
  **mixed**. The instrument prints it.
- **Validity:**
  - A search with no info line at or after 5,000 ms has no reading.
  - If more than 5% of the readings are missing, the read is invalid and
    is recorded as such.
  - The window is not changed after the read starts.

## Exposure before registration

- **RAR-P48's data, read per position** (an observation, not its
  registered result):
  - shared-search share at 4T about 0.8 at both positions;
  - `c-lp-off`'s E 0.943 and 0.905 against B.9's 1.032 and 1.019.
- **A window probe** (`window_probe.py`, `b652ae9b…`; `window_probe.json`,
  `482b178b…`): eight 10 s searches, every line kept, both arms at 1T and
  4T, one search per position.
  - It showed that cumulative NPS rises through a search. C at 1T rose from
    2.03 M at 0.5 s to 2.30 M at 4 s, and the threaded searches rose more
    steeply. This is the window confound RAR-P48 named.
  - Every search printed lines past 8 s.
  - At 5 s the probe's single searches put E at about 1.01 and 0.96 for
    B.9, and 0.93 and 0.92 for C. It took no H.
- **One smoke round** of the instrument, `go movetime 1200` read at 600 ms
  (`smoke2.json`, `f3797189…`):
  - 36 of 36 readings were interpolated, the masks read back as set, the
    hashes matched, and only C reported large pages disabled.
  - At 600 ms the gap was small: ΔE −0.044 and −0.015. This is consistent
    with RAR-P46's 100 ms result.
- **No 5 s reading of H** has been taken at either position under this
  instrument.

## Prediction, frozen before the read

| Outcome | Probability |
|---|---|
| **Shared search** at both positions | 0.55 |
| **Both** at both positions | 0.20 |
| **Mixed** (the positions disagree) | 0.15 |
| **Hardware** at both positions | 0.05 |
| **Not reproduced** at both positions | 0.05 |

- **Reasons:**
  - RAR-P48's per-position shares were about 0.8, from a confounded window.
  - The probe shows the window error is a few percent at most. That is
    enough to move a share between "shared search" and "both", and not
    enough to make the gap.
- **B.9's Q at `4T` above 1.00 at both positions** (B.9's threads faster
  sharing a search than apart): 0.6.
- **No reading missing:** 0.8.

## Decision and stop rule

- **Shared search:**
  - The cost is in how C's threads share the search, not in the cores.
  - Register a bisect of the speed pass's commits
    (`analysis/c12_speed_pass_2026-10-17.md`, e1 to e8) at 1T and 4T on
    this instrument, the three new prefetch sites first.
  - The speed pass is the one change since B.9 measured at one thread
    only. It is also where C's nodes became 22% cheaper, so a shared
    cost per node weighs more.
- **Both:**
  - The same bisect, read on Q.
  - The hardware share is recorded as a cost of C's faster nodes on the
    memory the cores share.
- **Hardware:**
  - C's faster nodes lean harder on shared memory. That is a property of
    the speed pass, not a lazy-SMP defect.
  - Recorded, and D.2 goes to the maintainer's self-relative scaling games.
- **Not reproduced:**
  - RAR-P48's gap belonged to its window or its pooling.
  - D.2's engine target is withdrawn, and D.2 goes to the scaling games.
- **Mixed:** recorded unresolved. No further read until a retry trigger is
  named.
- **Stopping and repeats:**
  - One read.
  - An interrupted read is rerun whole.
  - Nothing in the instrument, arms, cells, rounds or window changes after
    the read starts.

## How to run

From PowerShell 7 on an idle host. Nothing else should run until it
finishes, about 23 minutes:

```powershell
python D:\code\rarog\analysis\artifacts\d2-scaling\split_read2.py D:\code\rarog\analysis\artifacts\d2-scaling\split_read2.json 2>&1 | Tee-Object -FilePath D:\code\rarog\analysis\artifacts\d2-scaling\split_read2.log
```

If it refuses with `HOST NOT IDLE`, run it again. The JSON is written only
when the read completes, and the summary ends with an `OUTCOME:` line.

## Result — 2026-10-30, appended after exposure

- **The run.** The maintainer ran the registered read.
  - 216 process readings in 1,351 s, host load 1.8% before and 0.2% after.
  - Evidence: `analysis/artifacts/d2-scaling/split_read2.json`
    (`98e61523…`) and `split_read2.log` (`12900eaf…`).
  - Analysis: `p49_analyse.py` (`4757b27a…`, ignored) and its output,
    `p49_analyse.log`.
- **Integrity.**
  - Both binary hashes match.
  - Every cell has its registered count, and every mask reads back as set.
  - Only `c-lp-off` printed the large-pages-disabled notice.
  - No reading is missing (0 of 216). The info lines either side of
    5,000 ms are 845 ms apart at the median and 2,802 ms at most.
  - Recomputing every reading's interpolation from its raw lines agrees
    with the instrument's, and so does the judgement.
- **Readings at `4T`:**

  | Position | Arm | E | H | Q |
  |---|---|---:|---:|---:|
  | 0 | `b9head` | 1.015 | 0.956 | 1.062 |
  | 0 | `c-lp-off` | 0.937 | 0.956 | 0.981 |
  | 1 | `b9head` | 0.956 | 0.960 | 0.996 |
  | 1 | `c-lp-off` | 0.902 | 0.959 | 0.940 |

- **Outcome: shared search, at both positions.**

  | Position | ΔE | ΔH | ΔQ | Share | Verdict |
  |---|---:|---:|---:|---:|---|
  | 0 | −0.080 | −0.000 | −0.079 | 1.00 | shared search |
  | 1 | −0.058 | −0.000 | −0.058 | 0.99 | shared search |

  - The two heads lose exactly the same to the shared hardware: H 0.956
    against 0.956 at position 0, and 0.959 against 0.960 at position 1.
  - Everything C loses beyond B.9 is in Q.
  - At `2T`, which is not judged, the same holds: shares 1.05 and 1.05.
- **Robustness** (not registered, reported for the reader):
  - Every one of the 12 round-and-position readings has C's E below B.9's,
    with shares from 0.71 to 1.39.
  - A bootstrap over whole rounds puts the share's 90% interval at
    [0.83, 1.12] at position 0 and [0.74, 1.07] at position 1. It is at or
    above 2/3 in 100% and 98.9% of resamples.
  - 1T medians drift by at most 0.3% between the first and the last three
    rounds.
- **Secondary predictions.**
  - No reading missing (0.8): held.
  - B.9's Q at `4T` above 1.00 at both positions (0.6): missed. It was
    1.062 at position 0 but 0.996 at position 1.

## Disposition

- **Shared search**, by the registered rule. The cost is in how C's
  threads share the search, not in the cores, caches or memory they share.
  - RAR-P48's per-position H gaps of −0.017 and −0.020 were its window
    confound. Measured at one fixed time, they vanish.
- **Next: locate the commit.** Register a bisect on this instrument, at 1T
  and 4T, read on Q.
  - The first read halves the space between B.9 and C. Does the speed
    pass's own parent already show the loss? The speed pass's recorded
    reads (`analysis/c12_speed_pass_2026-10-17.md`, e1 to e8) give its
    internal boundaries.
  - The prefetch sites are read first if the speed pass holds the loss.
    These are e6's child-TT-line prefetch, `1aaa39e` and `65f490e`, and
    the singular search's three sites.
  - C also changed the search and the evaluation outside the speed pass,
    so the speed pass is a suspect, not the established cause.
- Nothing about strength follows from this read. Q is a speed ratio, and
  RAR-M74's field and the self-relative scaling games decide what it costs
  in games.

## Calibration

- **The registered outcome hit.** "Shared search" was given 0.55.
- **The magnitude was sharper than predicted.** The share is 1.0, not the
  0.8 that RAR-P48's confounded window suggested. The hardware term is
  exactly zero, which no outcome probability anticipated as a point.
- **The secondary prediction on B.9's Q missed at position 1.** B.9's
  above-one Q in RAR-P48 was partly its window: at a fixed 5 s it reads
  1.062 at one position and 0.996 at the other. "B.9's threads are faster
  sharing a search" holds at position 0 only.
- **The instrument repair was what mattered.** Pooling and the moving
  window had invented a small hardware share and hidden that H is
  identical between the heads.
