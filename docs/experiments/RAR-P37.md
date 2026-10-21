# RAR-P37 — The rustc 1.99.1 speed read repeated: replay, fresh pools and two pool nulls — REGISTERED 2026-10-21, before any build

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Why.** RAR-P36 read rustc 1.99.1 at −1.51% (−1.64% .. −1.38%) against 1.98.1. The maintainer suspects the conditions: its per-cycle CPU figure read 11–12%. That figure is now known to be the instrument's own start-up (a one-shot `Win32_Processor.LoadPercentage` from a fresh PowerShell reads 12–14% on a host the harness measure reads at 3%); `nps_read.py` now reads the host by the harness measure and refuses a busy one (`ee7d3f3`). Two further confounds are open: the RAR-P36 base pool's profiles were merged by MSYS2's `llvm-profdata` from PATH, the candidate's by the toolchain's own (`da2780f`); and one pool an arm leaves the pool-to-pool spread unmeasured. This registration repeats the read so that all three are answered, and reads the final 1.99.1 head (`9fb525e`: the pin, `try_update`, the xtask fix and the `raw_borrows_via_references` lint, which changes no code generation).

**Pools** (pext PGO, four builds each, `tools/nps_build_pool.ps1 -Arch pext -Builds 4 -ExpectFingerprint 13987223`, clean trees):
- **A**, RAR-P36's base: `analysis/artifacts/toolchain-1.99/pool-1.98.1` (`7099ff0`, rustc 1.98.1, MSYS2 `llvm-profdata` 22.1.8).
- **B**, RAR-P36's candidate: `analysis/artifacts/toolchain-1.99/pool-1.99.1` (`da2780f`, rustc 1.99.1).
- **A2**, fresh 1.98.1: `analysis/artifacts/toolchain-1.99/pool-1.98.1-r2`, built from `7099ff0` in a detached worktree with PATH stripped of `C:\msys64`, so xtask falls back to the 1.98.1 toolchain's own `llvm-profdata` (22.1.8-rust-1.98.1); the log records which tool resolved.
- **C**, fresh 1.99.1: `analysis/artifacts/toolchain-1.99/pool-1.99.1-r2`, built from `9fb525e`.

**Reads** (`tools/nps_read.py`, step 1, two cycles, each on a host the new guard calls idle, waited for by polling it every 60 s for at most three hours; the session runs nothing meanwhile):
- **R1, replay:** A against B, `--no-regression`, the same binaries as RAR-P36.
- **R2, fresh:** A2 against C, `--no-regression`; the shipped head's number.
- **N1, base null:** A against A2. **N2, candidate null:** B against C. Same compiler, independent pools.
R1 and R2 take step 2 (`--extend`) once only if step 1 lands between −0.9% and −0.1%. The nulls are step 1 only.

**Qualification of `9fb525e` before its pool:** `cargo test` and `cargo test --release`, `cargo build --release` and `bench 13` at 13,987,223 / EBF 2.565; fmt and clippy at zero already recorded for this content.

**Predictions, frozen 2026-10-21 before any build or read:**
- R1: **−1.5%**, 80% band **[−1.8%, −1.2%]**. A result outside it on the same binaries means conditions moved RAR-P36.
- R2: **−1.5%**, 80% band **[−2.0%, −1.0%]**.
- N1 and N2: **0.0%** each, 80% band **[−0.6%, +0.6%]**.
Basis: RAR-P36's builds agreed within 0.11% in each arm and its cycles were interleaved, so I expect the −1.5% to be the compiler; the null band is wider than that spread because whole pools of one source have never been read against each other here, and an earlier four-build pool spanned 1.7% (C.12, 2026-10-17). Evidence layer: bench NPS, not Elo.

**What decides what.** The pin stays the maintainer's decision. R1 says whether RAR-P36's conditions mattered; R2 is the number for the shipped head; a null whose interval excludes zero is a finding on the instrument (pool-to-pool variation of that size), reported and carried into how four-build reads are interpreted, not a reason to discard R1 or R2. A disturbed read is repeated, not interpreted.

## Result / disposition

Pending.

## Conditional lesson and retry trigger

Pending.

## Source

RAR-P36; RAR-P32; PROCESS, *Toolchain and harness notes*
