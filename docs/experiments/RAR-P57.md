# RAR-P57 — A statically linked runtime on Windows and Linux against the dynamic one: speed — REGISTERED 2026-11-04, before the read; READ 2026-11-04: Windows PASS +0.13%, Linux inconclusive

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-04 by the agent at the maintainer's
  request: the Windows and Linux builds link their C runtime statically, so
  a user needs nothing installed (macOS cannot: Apple supports no static
  `libSystem`, and Rust's Apple targets have no static runtime).
- **Kind:** performance qualification of a build change. The engine source
  is unchanged; every build benches 14,749,043.
- **What changes.** `-C target-feature=+crt-static` on every `cargo xtask`
  build for `*-windows-msvc` and `*-linux-gnu`. Windows links the Visual C++
  runtime and the UCRT into the binary (no `VCRUNTIME140.dll`, no
  `api-ms-win-crt-*`); Linux links glibc in (a static PIE, no dynamic
  loader, no glibc floor). Fathom's C follows (`cc` reads `crt-static`).
  Rarog's hot path calls the runtime only for `memcpy`-like copies, the
  same code statically linked; on Windows the import thunk disappears.

## Arms

Single-tier `pext` PGO builds, four an arm.
- **Windows:** the dynamic arm is RAR-P56's `pext` pool
  (`analysis/artifacts/e33-universal-nps/pext/`, `e5fceb5`, whose engine
  inputs are the current ones); the static arm is four new builds in
  `analysis/artifacts/e33-static-nps/static-windows/`.
- **Linux (WSL Ubuntu 24.04, a clean clone):** four dynamic builds from the
  last commit before the change and four static builds after it, in WSL's
  `~/static-nps/`, archived to `analysis/artifacts/e33-static-nps/linux/`.

## Rule, fixed before the read

`python tools/nps_read.py --no-regression`, the harness's rule unchanged:
pass at step 1 at −0.1% or better, fail at −0.9% or worse, else four more
cycles and pass above −0.5%. The Windows read is the gate. The Linux read is
**indicative**: WSL has no PowerShell, so `nps_read.py`'s host measure cannot
run there; the Windows host is checked idle by hand before it starts and the
read runs with `--allow-busy-host`, which it archives.

A pass ships the static runtime. A fail is reported to the maintainer before
anything ships: the reason for the change is compatibility, so whether a
measured loss is worth it is the maintainer's decision, not this rule's.

## Amendment (2026-11-04, before either read)

Linux PGO builds of one source are byte-identical here: the four dynamic
builds from `6c2d323` share one hash (`F011FA74…`), so PGO training is
deterministic on Linux, unlike Windows. `nps_read.py` refuses identical
binaries, so it cannot read a Linux pool. The Linux read instead benches the
one dynamic and the one static binary (`e09fc46`, `265A734E…`) interleaved in
WSL: eight cycles, in each both arms run `bench 13` three times in
alternating order and an arm's reading is the best of its three; the
estimate is the median of the eight static/dynamic ratios and the spread is
their range. It covers process noise, not build layout. It stays
indicative, and no rule decides on it. The Windows gate is unchanged.

## Prediction

- Windows: **+0.1%**, 90% interval −0.4% to +0.6%; P(pass) 0.85. The same
  copy routines, called without the import thunk.
- Linux: **+2%**, 90% interval −1% to +5%. Basilisk's static Linux build
  read +5% in one WSL read (BAS-P22's environment), but Basilisk is C++ with
  libstdc++ linked in; Rust links its own std statically already, so less
  should change here.

## Result (2026-11-04)

**Windows (the gate): PASS.** The dynamic arm is RAR-P56's `pext` pool
(`EF3EDF18`, `26E28271`, `D70E6F3A`, `DC63EFDF`); the static arm four builds
from `e09fc46`, clean, in `analysis/artifacts/e33-static-nps/static-windows/`.
Host idle, 1.3% and 1.5% CPU before the cycles, 4.3% after; no disturbed
cycle.

| Arm | Per-build medians (n/s) | Level |
|---|---|---|
| dynamic | 2,948,632, 2,936,044, 2,943,630, 2,944,214 | 2,943,130 |
| static | 2,948,930, 2,942,454, 2,947,159, 2,949,222 | 2,946,941 |

Static over dynamic **+0.13%**, 95% t-interval −0.12% to +0.38%: regression
excluded by step 1. A static binary imports only Windows' own DLLs (no
`VCRUNTIME140.dll`, no `api-ms-win-crt-*`) and is 1,857,024 bytes against
1,679,872 (the no-PGO build measured).

**Linux (indicative): inconclusive.** One dynamic (`F011FA74…`, `6c2d323`)
and one static (`265A734E…`, `e09fc46`, 3,122,352 bytes, no dynamic loader,
no linker warning) binary, eight interleaved cycles in WSL after a manual
host check (0–5% CPU, nothing running); archived as
`analysis/artifacts/e33-static-nps/read-linux.txt`.

| Cycle | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| static / dynamic | 1.0075 | 0.9518 | 0.9382 | 1.0494 | 0.9537 | 0.9715 | 0.9358 | 1.0217 |

Median **0.963 (−3.7%)**, range −6.4% to +4.9%. The dynamic binary alone
read 2.63 M to 2.89 M n/s across cycles, a 10% swing, so WSL timing cannot
resolve a few percent here; static was slower in 6 of 8 cycles, which a
coin gives about one time in seven (one-sided). No rule decides on it.

**Calibration.** Windows: predicted +0.1%, read +0.13%: a hit. Linux:
predicted +2% [−1%, +5%]; the instrument could not resolve the question,
so there is no calibration, only the noise to record.

**Decision.** Static ships on Windows by the gate. On Linux it ships for
compatibility at the maintainer's request (no glibc floor); the speed
question stays open for a read on a native Linux host, where Basilisk's +5%
and this read's −3.7% can be told apart.
