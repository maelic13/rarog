# RAR-P50 — Which change from the B.9 head to the C head holds the shared-search loss at four threads: a bisect read on Q — REGISTERED 2026-10-30, before the read

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-30 by the agent; the maintainer runs
  it (about 37 minutes).
- **Question:** [RAR-P49](RAR-P49.md) put the whole of the C head's
  four-thread loss in the shared search: Q 0.981 against B.9's 1.062 and
  0.940 against 0.996, with the shared hardware costing both heads the same.
  Which of the changes between the two heads holds that loss?
- **Why a bisect and not games first.** The maintainer asked whether a
  four-thread match of the last good scaler against the head should come
  first.
  - RAR-M76 already played it: the C head against 2.5.0, which has B.9's
    fingerprint, read +149.7 ± 14.7 at 4T against +153.4 ± 10.7 at 1T.
  - It was played at `3+0.03`, where the heads scale alike. Its null says
    nothing about a long-search loss (RAR-M76's appended note).
  - A match that could see the loss needs moves of a second or more.
    Resolving an effect of about 8 Elo would take days of games.
  - The bisect costs builds and reads of about 40 minutes. A fix it finds
    is judged by its own gate.
  - The maintainer chose the bisect first.
- **Kind:** zero-game diagnostic. It locates the change and decides nothing
  about strength, a default or a gate.

## Arms

The engine-changing squash commits on `master` from the B.9 head to the C
head, in source order:

| Arm | Revision | What it adds | `bench 13` | Large pages |
|---|---|---|---|---|
| `b9` | `24aefb4` | the B.9 head | 11,171,726 / 2.512 | none |
| `e52` | `52cc6de` | C's evaluation rebuilds: `6dd4aee`, `cc13320`, `36f04d5`, `52cc6de` | 14,197,472 / 2.557 | none |
| `e6a` | `6ade0e6` | the evaluation refit, the speed pass and large pages | 10,521,141 / 2.521 | always on |
| `e138` | `138d867` | the search fit | 13,987,223 / 2.565 | always on |
| `c` | `2b7bca9` | time management (`703fe78`) and the `LargePages` option | 13,987,223 / 2.565 | on (the default) |

- **Builds.**
  - `b9` to `e138` are each revision's own `cargo xtask build --arch pext
    --pgo`, built on 2026-10-30 by `bisect_build.sh` (`966f7e56…`) in
    detached worktrees. `RUSTUP_TOOLCHAIN=1.99.1` overrides each revision's
    pin, so the compiler is constant across the arms. The pin moved to
    1.99.1 at `138d867`, and RAR-P37 measured the bump at about −1.6% NPS.
  - Each build merged its training profile, and each binary embeds
    1.99.1's commit `58cf8f9c3`.
  - Each bench matches the fingerprint recorded for its revision.
  - Hashes: `rarog-bis-24aefb4-pext-pgo.exe` `2e4e2820…`,
    `rarog-bis-52cc6de-pext-pgo.exe` `59f2f860…`,
    `rarog-bis-6ade0e6-pext-pgo.exe` `36c89a0b…`,
    `rarog-bis-138d867-pext-pgo.exe` `621136ee…`, all in
    `tools/test_engines/`, with logs in
    `analysis/artifacts/d2-scaling/build_logs/`.
  - `c` is RAR-P47 to RAR-P49's binary, `rarog-d2lp-pext-pgo.exe`
    (`ebbbc917…`, 1.99.1).
- **Every arm runs in its default configuration.**
  - From `6ade0e6` until the option at `2b7bca9`, the engine takes large
    pages with no way to refuse them. So `c` runs with them on, unlike
    RAR-P49's `c-lp-off`.
  - RAR-P48 put large pages' own share of the gap at 0.005 to 0.026 in E.
    It falls in I2 with the rest of `6ade0e6`, and a second stage can
    separate it.

## Instrument and cells

- **Instrument:** `analysis/artifacts/d2-scaling/split_read3.py`
  (`fd489549…`, ignored). This is RAR-P49's method unchanged:
  - `go movetime 10000`, every info line kept;
  - nodes interpolated at exactly 5,000 ms;
  - each position read on its own;
  - fresh processes, Hash 256, RAR-P46's two positions;
  - one logical CPU per thread on its own physical core of CCD0;
  - masks read back, and hashes checked against the table above.
- **Cells,** the three that give Q at 4T: `1T` (CPU 2), `4x1T` (four
  1-thread processes on CPUs 2, 4, 6, 8, together) and `4T` (CPUs 2, 4, 6,
  8). RAR-P49's 2T and 2x1T cells are dropped: its 2T read the same as its
  4T.
- **Order:**
  - Six rounds. The five arms' order rotates by one place each round, so
    every arm runs in five different places, and `c` runs first twice.
  - A 10 s settle before each arm's block, and a timestamp on every reading.
- **Judgement**, checked on synthetic data by `split_read3_check.py`
  (`e9363521…`), which produces every outcome as designed:
  - Per position, **L(arm) = ln(Q_arm / Q_b9)** at 4T. The intervals are
    I1 = `b9`→`e52`, I2 = `e52`→`e6a`, I3 = `e6a`→`e138` and
    I4 = `e138`→`c`.
  - Each interval's share is its increment of L over L(`c`).
  - Position verdict:
    - **not reproduced** if L(`c`) > ln 0.96;
    - otherwise the interval whose share is at least 0.5;
    - **spread** if none is.
  - **Outcome:** the position verdict when both positions agree, otherwise
    **mixed**. The instrument prints it.
  - A search with no info line at or after 5,000 ms has no reading. Over
    5% missing makes the read invalid.

## Exposure before registration

- RAR-P48 and RAR-P49's results, as recorded.
- **One smoke round** of this instrument, `go movetime 1200` read at
  600 ms (`smoke3.json`, `9755a792…`).
  - 60 of 60 readings were taken, masks and hashes held, and large pages
    showed exactly from `e6a` on (working set 6 MiB against 295).
  - At 600 ms, with one round, L(`c`) was +0.001 at position 0 (not
    reproduced) and −0.044 at position 1, where I2 held. The short window
    has little loss to locate (RAR-P46, RAR-P49).
- The rebuilt `b9` reads 1.83 M at 1T in the smoke, as RAR-P49's 1.98.1
  build did, so the compiler does not move B.9's speed.

## Prediction, frozen before the read

| Outcome | Probability |
|---|---|
| **I2**: the evaluation refit, the speed pass and large pages (`6ade0e6`) | 0.45 |
| **I3**: the search fit (`138d867`) | 0.15 |
| **Spread**: no interval holds half | 0.12 |
| **I1**: C's evaluation rebuilds | 0.10 |
| **Mixed** (the positions disagree) | 0.10 |
| **I4**: time management and the option | 0.05 |
| **Not reproduced** | 0.03 |

- **I2 first.** The speed pass is the one change in the range measured
  only at one thread. It changed how nodes use memory: three new TT prefetch
  sites, the child's TT line prefetched at make, the evaluation cache
  removed, attack tables moved into static data. Large pages arrived with
  it too.
- **I3 next.** The search fit changes which nodes are searched and how
  often threads meet in the table, and that alone can change shared-search
  cost.
- **I1 and I4 last.**
  - I1's evaluation rebuilds make nodes dearer, not more shared.
  - I4's time management is not reached by `go movetime`, and its option
    only lets large pages be turned off.

## Decision and stop rule

- **I2:** a second stage within `6ade0e6`'s branch commits, on this
  instrument.
  - The speed pass's recorded boundaries come first
    (`analysis/c12_speed_pass_2026-10-17.md`, e1 to e8). The prefetch
    sites (`1aaa39e`, `65f490e`, and the singular search's three) and the
    large-pages commit go first among them.
  - The evaluation refit's commits come after.
- **I1:** a second stage over `6dd4aee`, `cc13320`, `36f04d5`.
- **I3:** a second stage within `138d867`'s branch commits.
- **I4:** `703fe78` against `2b7bca9`, one more arm.
- **Spread:**
  - The loss accumulates over several changes, and no single commit holds
    it.
  - The bisect stops. D.2 studies the mechanism instead: the rates of
    shared-table reads and writes per node, by diag counters, on `b9`
    and `c`.
- **Mixed or not reproduced:** recorded unresolved. No further read until
  a retry trigger is named.
- **Stopping and repeats:**
  - One read.
  - An interrupted read is rerun whole.
  - Nothing in the instrument, arms, cells, rounds or window changes after
    the read starts.

## How to run

From PowerShell 7 on an idle host. Nothing else should run until it
finishes, about 37 minutes:

```powershell
python D:\code\rarog\analysis\artifacts\d2-scaling\split_read3.py D:\code\rarog\analysis\artifacts\d2-scaling\split_read3.json 2>&1 | Tee-Object -FilePath D:\code\rarog\analysis\artifacts\d2-scaling\split_read3.log
```

If it refuses with `HOST NOT IDLE`, run it again. The summary ends with an
`OUTCOME:` line.
