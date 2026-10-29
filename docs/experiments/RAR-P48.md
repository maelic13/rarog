# RAR-P48 — Where the C head's 5 s thread-scaling loss lives: hardware, the shared search, the CCD split or large pages — REGISTERED 2026-10-29, before the read

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-29 by the agent; the maintainer runs
  it (about 26 minutes).
- **Question:** [RAR-P46](RAR-P46.md)'s bench put the C head's 4T/1T at
  3.28 against the B.9 head's 3.80, a loss complete at 2T (per-thread
  efficiency 0.81 at 2T, 0.82 at 4T, against 0.96 and 0.95). In 100 ms
  searches the two heads scale alike: 3.54 against 3.68 in RAR-P46, and the
  `d2` head with large pages off reads 3.62 in [RAR-P47](RAR-P47.md). Where
  does the 5 s loss live?
- **Kind:** zero-game diagnostic, registered before the read. It chooses
  D.2's next investigation and decides nothing about strength, a default or
  a gate.

## What the code says, before any measurement

- **Since B.9, nothing that threads share has changed.**
  - `src/tt.rs`, `src/search/history.rs` and `src/search/correction.rs` are
    unchanged.
  - `src/search/shared.rs` renames `fetch_update` to `try_update` and adds
    the `LargePages` field. `src/search/thread.rs` and `threads.rs` change
    no shared write.
  - Production TT probe and store sites are the same in both heads (the new
    ones are tests).
  - The one new thread-dependent branch in `search/mod.rs` is the fail-high
    stop, a clock rule that `go movetime` does not reach.
- **Per thread, C holds less than B.9.**
  - C dropped B.9's whole-evaluation cache (32,768 entries a thread,
    `0f745eb`).
  - `EvalTables` are 6 KB. The `FAMILY_MASKS` atomic is fixed before the
    first evaluation and only read after it.
- **C's speed pass (`6ade0e6`, +22.5% bench NPS) was measured at one thread
  only** (`analysis/c12_speed_pass_2026-10-17.md`).
  - It added three TT prefetch sites in `negamax`, in the singular search.
  - It moved the attack tables into the binary as statics.
- **Per node, at 2T C pays about 113 ns more than at 1T** (580 against
  467 ns a node per thread), where B.9 pays about 26 ns (680 against 654).
  - A loss that is whole at two threads and flat to four looks like
    something that switches on with sharing, not like bandwidth or clock
    speed, which grow with the thread count.
  - Nothing in the diff names that something.

## Design

- **Arms:**
  - `b9head`: `tools/test_engines/rarog-b9head-pext-pgo.exe` (`aac92114…`),
    the reference; it never uses large pages;
  - `c-lp-on`: `tools/test_engines/rarog-d2lp-pext-pgo.exe` (`ebbbc917…`,
    `2b7bca9`, bench 13,987,223), `LargePages true`, which is RAR-P46's
    configuration;
  - `c-lp-off`: the same binary with `LargePages false`.
- **Instrument:** `analysis/artifacts/d2-scaling/split_read.py`
  (`3456bae2…`, ignored).
  - Every search is a fresh engine process, as in RAR-P46's bench: Hash 256,
    `go movetime 5000`, RAR-P46's two positions.
  - NPS is the engine's own, from the last `info … nodes` line.
  - One reader process per engine; concurrent processes pass a barrier
    before `go`.
- **Placement:** one logical CPU per thread, each on its own physical core.
  - Core *k* is CPUs 2*k* and 2*k*+1, and CPUs 0–15 and 16–31 each hold one
    32 MB L3 (CCD0 and CCD1). Both were read from
    `GetLogicalProcessorInformationEx`, not assumed.
  - The engine sets no affinity of its own; each process's mask is read back
    and must equal the one set.
- **Cells,** per arm and position:

  | Cell | Processes × threads | CPUs |
  |---|---|---|
  | `1T` | 1 × 1 | 2 |
  | `2x1T` | 2 × 1, together | 2; 4 |
  | `4x1T` | 4 × 1, together | 2; 4; 6; 8 |
  | `2T` | 1 × 2 | 2, 4 |
  | `4T` | 1 × 4 | 2, 4, 6, 8 |
  | `4x1T-x` | 4 × 1, together | 2; 4; 18; 20 |
  | `4T-x` | 1 × 4 | 2, 4, 18, 20 |

  The *k*×1T cells share the hardware but not the search; the *k*T cells
  share both. The `-x` cells split the four cores two and two across the
  CCDs, as an unpinned process may be placed (RAR-P46's bench ran
  unpinned).
- **Order:**
  - Six rounds. Each round runs the arms in the rotated order
    (on, off, B.9), (off, B.9, on), (B.9, on, off), so each arm holds each
    place twice.
  - A 10 s settle before each arm's block, and a timestamp on every reading.
  - 12 readings per arm and threaded cell; 12 × *k* process readings per
    *k*×1T cell.
- **Host:** the 5950X. The instrument refuses to start above 15% busy and
  records the host's load before and after the read.

## Readings, per arm (medians)

- **S** = NPS(*k*T) / NPS(1T), and the efficiency **E** = S / *k*.
- **H** = per-process NPS(*k*×1T) / NPS(1T): what the shared hardware costs
  each thread.
- **Q** = S / (*k* · H): what sharing the search costs beyond the hardware.
- Each for `2T`, `4T` and `4T-x`, the last against `4x1T-x`.

## Exposure before registration

The wire was proved by three smoke runs (one round, 300 ms searches), so
short-search numbers were seen before this prediction.

- **Wire proved:**
  - `LargePages false` reaches the engine: its notice, and a working set of
    293 against 6 MiB;
  - the masks read back as set;
  - both binaries' hashes match.
- **On an idle host with the settle** (third smoke), every arm's E was
  0.94–1.02 at 300 ms.
- **The first two smokes, with no settle, show something else.** The
  host's load spiked to 42–60% around the second, from desktop-app bursts.
  - In both, `c-lp-off`, which ran right after `c-lp-on`, lost about half
    its NPS (about 1.0 M against 2.0 M a process) in its first concurrent
    cells.
  - Neither `b9head` nor `c-lp-on` ever did.
  - This is unattributed and is why the settle and the timestamps exist.
- **No 5 s reading has been taken.**

## Prediction, frozen before the read

Outcomes are judged on the medians and in this order; the first that holds
is the result.

| Outcome | Rule | Probability |
|---|---|---|
| **Not reproduced** | `c-lp-on`'s E at `4T` and at `4T-x` both within 0.06 of B.9's in the same cell | 0.40 |
| **CCD split** | `c-lp-on`'s E at `4T` within 0.06 of B.9's, at `4T-x` at least 0.06 below | 0.15 |
| **Large pages** | `c-lp-on`'s E at `4T` at least 0.06 below B.9's, `c-lp-off`'s within 0.03 of B.9's | 0.10 |
| **Hardware** | both C arms' E at `4T` at least 0.06 below B.9's; `c-lp-off`'s H at `4T` at least 0.04 below B.9's, its Q within 0.03 | 0.15 |
| **Shared search** | both C arms' E at `4T` at least 0.06 below B.9's; `c-lp-off`'s H at `4T` within 0.03 of B.9's, its Q at least 0.04 below | 0.10 |
| **Mixed** | anything else | 0.10 |

- **Reasons for "not reproduced" first:**
  - The 300 ms smokes and RAR-P47's 100 ms cells show no gap with large
    pages off.
  - Pinning one thread per physical core removes two things RAR-P46's
    unpinned bench allowed: two threads on one core's SMT pair, and a search
    split across the CCDs.
- **B.9:** E at `4T` at least 0.93 (0.85).
- **The transient:** count the process readings in the concurrent cells
  below 0.75 × the arm's `1T` median at the same position. Predicted none in
  any arm, with the settle (0.6).

## Decision and stop rule

- **Not reproduced:**
  - RAR-P46's bench loss belongs to its unpinned placement or to that run.
  - One pre-registered follow-up, with the same instrument and arms and the
    threaded cells left unpinned. Then stop and record what it shows.
  - D.2's engine target is then withdrawn, and D.2 goes on to the
    maintainer's self-relative scaling games.
- **CCD split:** D.2 researches what C writes across the CCD boundary that
  B.9 does not: TT store and prefetch rates per node, by diag counters on
  both heads.
- **Large pages:**
  - A bench-only effect: in games, large pages are +2.7% NPS at 4T
    ([RAR-M77](RAR-M77.md)).
  - Recorded; no engine work. D.2 goes on as under "not reproduced".
- **Hardware:**
  - C's faster nodes lean harder on the memory the cores share. That is a
    property of the speed pass, not a lazy-SMP defect.
  - Recorded; D.2 goes on to the scaling games.
- **Shared search:**
  - Bisect the speed pass's commits (`analysis/c12_speed_pass_2026-10-17.md`,
    e1–e8) at 1T and 2T on the same instrument.
  - The three new prefetch sites first, since they are the one change that
    touches the shared table.
- **Mixed:** recorded unresolved. No further read until a retry trigger is
  named.
- No change to the instrument, cells, arms or rounds after the read starts.
  An interrupted read is rerun whole.

## How to run

From PowerShell 7 on an idle host. Nothing else should run until it
finishes:

```powershell
python D:\code\rarog\analysis\artifacts\d2-scaling\split_read.py D:\code\rarog\analysis\artifacts\d2-scaling\split_read.json 2>&1 | Tee-Object -FilePath D:\code\rarog\analysis\artifacts\d2-scaling\split_read.log
```

If it refuses with `HOST NOT IDLE` (the desktop app's bursts can trip the
2 s window), run it again. The JSON is written only when the read
completes.
