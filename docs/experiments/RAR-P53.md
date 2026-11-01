# RAR-P53 — The shared table backend's cost to a one-thread search, against the local one, in one binary (F16) — REGISTERED 2026-11-01, before the read

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-01 by the agent; the maintainer runs
  it (about 6 minutes).
- **Leaf:** PLAN D.2, the four-thread card's read 4 (engine review input,
  item 2; DRIVER-2 / TT-6 / TT-4; F16 of `analysis/architecture_review_2026-09.md`
  §4.6).
- **Question.** A shared probe scans up to six (8-byte load, 2-byte load,
  `fold16`, compare) and a store up to six slots through `load_any` and
  `unpack_entry`; the local form does the same over three plain entries. One
  thread never touches the shared form, so no fingerprint or one-thread NPS
  read has ever priced it. What does it cost a node?
- **Kind:** zero-game diagnostic. It orders the card's table items; nothing
  about strength or a default follows.

## The instrument, replaced before any read

- **The review's instrument cannot measure this.** It froze `bench 3` at
  Threads 2 against Threads 1, where the search stays serial over a shared
  table. `bench 3` searches 20,376 nodes in under 1 ms (`Total time (ms) :
  0`), and every search deeper than 3 at Threads 2 goes parallel
  (`MIN_PARALLEL_DEPTH` is 4). It is replaced here, before any read; the
  prediction is kept.
- **Instrument patch** `analysis/arm_patches/84f978b-rar-p53-shared-tt-at-1t.patch`
  on the d2 head, never merged: with `RAROG_INSTRUMENT_SHARED_TT` set, a
  one-thread search keeps the shared table (in `configure` and at the search's
  entry). One binary then runs both backends.
- **Builds** (`analysis/artifacts/d2-backend/build.sh`, `build.log`), rustc
  1.99.1, 2026-11-01:

  | Build | Binary | SHA-256 |
  |---|---|---|
  | PGO pext, as shipped | `tools/test_engines/rarog-d2ttsh-pext-pgo.exe` | `09497b01…` |
  | pext without PGO | `tools/test_engines/rarog-d2ttsh-pext.exe` | `6becffc1…` |

- **The wire** (`wire.log`), after `setoption name Hash value 64` (a bare
  `bench` never configures the table, and the switch then does nothing, as
  the first try showed): `bench 13` reads **13,987,223 / 2.565 off** in both
  builds, the development fingerprint, so the patch is neutral when off, and
  **14,429,386 / 2.566 on**: the six-way geometry grows the bench tree 3.2%.
- **Why two builds.** The PGO training never builds a shared table, so the
  shipped build runs the shared branches untrained ([RAR-P52](RAR-P52.md)).
  The build without PGO runs both backends equally untrained, so R(pgo) /
  R(nopgo) prices the cold layout apart from the backend.

## Read

- `analysis/artifacts/d2-backend/backend_read.py` (`9b101e26…`, ignored): per
  build, four cycles,
  each running the switch off and on in alternating order, one fresh process
  each, pinned to CPU 2, Hash 64, `bench 13 3`; every run's nodes are checked
  against the wire's counts.
- **Readings** per build: **R = median NPS on / median NPS off** (the
  registered reading) and W = median time on / median time off (the bench's
  work, which includes the 3.2% larger tree). The NPS spread within each
  switch is printed.
- **Outcome** on R of the PGO build:
  - **REFUTED** at 0.97 or above;
  - **UNDER BAND** 0.91 to 0.97;
  - **CONFIRMED IN BAND** 0.87 to 0.91;
  - **CONFIRMED ABOVE BAND** under 0.87.
- **Exposure:** the wire's single `bench 13` runs, whose NPS lines were not
  read, and one smoke cycle of `bench 13 1` per build (`smoke.log`), which
  prints only that the node counts held.

## Prediction, frozen before the read

The review froze R at 0.87 to 0.91 for the head, refuted at 0.97 or above.
The B.9 half of its prediction (0.90 to 0.93) is not read: the patch is on
the head only.

| Outcome | Probability |
|---|---|
| UNDER BAND | 0.40 |
| CONFIRMED IN BAND | 0.30 |
| REFUTED | 0.20 |
| CONFIRMED ABOVE BAND | 0.10 |

- R without PGO 0.92 to 0.98; R(pgo) / R(nopgo) under 0.98 with probability
  0.5.
- **Why under band is likeliest.** The review's band is a fit of RAR-P50's
  medians to a fixed per-node cost, which charges the backend with everything
  that differs between four threads and one. The scan is six slots of one
  cache line against three of half a line: more work, but the same memory
  access, in an engine that is memory-latency bound.

## Decision and stop rule

- **Confirmed (in or above band):** F16 stays in the card in cost order: a
  `replacement` that reads only occupancy, depth and age bits
  (behaviour-neutral), skipping byte-identical stores (DRIVER-6), then one
  atomic backend at every thread count (an `I2`, the 1T NPS pool at ±0.5%,
  gate case 3, since the geometry moves the fingerprint).
- **Under band:** the cost is real and smaller than the fit; only the
  behaviour-neutral pieces stay, ranked against the card's other reads.
- **Refuted:** F16 leaves the four-thread card.
- **R(pgo) / R(nopgo) ≤ 0.98:** the cold shared branches cost the shipped
  build; RAR-P52's cure (b), a Threads-2 run in the PGO training, is
  registered next.
- One read; nothing changes after it starts.

## How to run

From PowerShell 7 on an idle host, nothing else running, about 6 minutes:

```powershell
python D:\code\rarog\analysis\artifacts\d2-backend\backend_read.py D:\code\rarog\analysis\artifacts\d2-backend\backend_read.json 2>&1 | Tee-Object -FilePath D:\code\rarog\analysis\artifacts\d2-backend\backend_read.log
```

If it refuses with `HOST NOT IDLE`, run it again. The summary ends with an
`OUTCOME:` line.
