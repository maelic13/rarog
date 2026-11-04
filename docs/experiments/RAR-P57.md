# RAR-P57 — A statically linked runtime on Windows and Linux against the dynamic one: speed — REGISTERED 2026-11-04, before the read

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

## Prediction

- Windows: **+0.1%**, 90% interval −0.4% to +0.6%; P(pass) 0.85. The same
  copy routines, called without the import thunk.
- Linux: **+2%**, 90% interval −1% to +5%. Basilisk's static Linux build
  read +5% in one WSL read (BAS-P22's environment), but Basilisk is C++ with
  libstdc++ linked in; Rust links its own std statically already, so less
  should change here.
