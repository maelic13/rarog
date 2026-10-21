# RAR-P37 — The rustc 1.99.1 speed read repeated: replay, fresh pools and two pool nulls — REGISTERED 2026-10-21, before any build; READ 2026-10-21: −1.63% on fresh pools, conditions cleared

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

**The regression is the compiler: −1.63% on fresh pools and −1.66% on RAR-P36's own, with both pool nulls at zero.** Run 2026-10-21 16:04–16:47 local by one script (`analysis/artifacts/toolchain-1.99/p37-status.txt`, every read's `raw.json`, `read.txt` and log beside it).

- **Qualification of the head** (`fb71b2a`, engine source `9fb525e`, rustc 1.99.1; `analysis/artifacts/toolchain-1.99/verify-p37/`): `cargo test` **398 passed, 0 failed, 2 ignored**; `cargo test --release` **399 passed, 0 failed, 2 ignored**; `cargo build --release` then `bench 13`: **13,987,223 / EBF 2.565**.
- **Pools.** C `analysis/artifacts/toolchain-1.99/pool-1.99.1-r2`, source `fb71b2a` (clean), 16:09: `3E00C216…`, `97CF5034…`, `5ED0F756…`, `A9D75EB5…`. A2 `analysis/artifacts/toolchain-1.99/pool-1.98.1-r2`, source `7099ff0` (clean, a detached worktree, removed afterwards), 16:11: `06696F00…`, `AA944FA9…`, `76B5F634…`, `112547F9…`; `rustc.txt` records rustc 1.98.1 / LLVM 22.1.8 and "llvm-profdata on PATH: none", so its profiles were merged by the toolchain's own tool. Every copy benched 13,987,223.
- **Reads**, each started on a host the guard read idle (0.0–2.7% CPU, no competing process):
  - **R1, replay** (A against B): **−1.66%**, 95% **−1.93% .. −1.38%**, levels 3,087,277 against 3,036,175, no disturbed cycle; "fail: regression by step 1".
  - **R2, fresh** (A2 against C): first attempt **disturbed** (base cycle 1; −0.45%, not interpreted); repeated at 16:36: **−1.63%**, 95% **−1.83% .. −1.42%**, levels 3,102,507 against 3,052,066, no disturbed cycle; "fail: regression by step 1".
  - **N1, base null** (A against A2): first attempt **disturbed** (candidate cycle 1; −1.21%, one build reading 2.5% low in it, not interpreted); repeated at 16:42: **−0.00%**, 95% **−0.38% .. +0.38%**, no disturbed cycle.
  - **N2, candidate null** (B against C): **−0.20%**, 95% **−0.56% .. +0.16%**, no disturbed cycle.
- **What this answers.** Conditions did not make RAR-P36's number: the same binaries on a guard-idle host read −1.66% against −1.51%. The merge tool did not either: N1 compares MSYS2's `llvm-profdata` against the toolchain's on one source and compiler and reads 0.00%. Fresh pools give the same answer, so neither pool was lucky. The head's own changes since `da2780f` (the lint) read −0.20% inside zero (N2).
- **Variation, measured.** Two pools of one source and compiler differ by at most 0.2% here, inside ±0.4–0.6%, so four builds an arm resolve a 1.5% change cleanly. The host's level is what moves: pool A read 3,087,277, 3,003,249 and 3,106,662 nps in three reads within half an hour, a 3.4% swing that interleaving cancels inside a read. Two of six reads started on a guard-idle host still had a disturbed cycle, and both disturbed readings were wrong (−0.45% and −1.21% against the clean −1.63% and 0.00%), so the disturbance rule is necessary, and a start-of-read idle check is not sufficient.
- **Elo, expected not measured:** about −3 at `3+0.03` by PROCESS's 2 Elo per 1% NPS.

**Prediction calibration, appended after exposure.** R1 predicted −1.5% in [−1.8%, −1.2%], read −1.66%: a hit. R2 predicted −1.5% in [−2.0%, −1.0%], read −1.63%: a hit. N1 and N2 predicted 0.0% in [−0.6%, +0.6%], read 0.00% and −0.20%: hits, and the band was wider than needed. Not predicted: that two of six reads would be disturbed on a host the guard called idle; the instrument's disturbance rule, not the start check, kept them out.

## Conditional lesson and retry trigger

**The 1.99.1 regression is real, about 1.6%, and not an artefact of conditions, pools or the merge tool.** Four-build pools resolve it with room to spare (pool nulls within ±0.4%). **A guard-idle start does not make a read clean:** the host's level wanders by about 3% over half an hour and disturbs whole cycles, so a disturbed read is repeated, never read. **Retry trigger:** the pin decision (keep 1.99.1 or return to 1.98.1) is the maintainer's; the next stable bump is read the same way, and a localisation (one plain pext build an arm, which separates LLVM 23's code generation from its use of the profile) is the cheapest next question if the loss is to be understood.

## Source

RAR-P36; RAR-P32; PROCESS, *Toolchain and harness notes*
