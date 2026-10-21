# RAR-P38 — Where rustc 1.99.1's 1.6% goes: plain builds on both compilers — REGISTERED 2026-10-21, before any build

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Question.** RAR-P37 measured rustc 1.99.1's PGO builds 1.63% slower than 1.98.1's at the same fingerprint (95% −1.83% .. −1.42%). Is that LLVM 23's code generation, or its use of the profile (inlining, layout and branch weights under PGO)? A plain build removes the profile and leaves the code generation. Asked by the maintainer for the information; it decides nothing, because 1.99.1 stays (maintainer decision 2026-10-21).

**Arms.** Plain pext builds (`cargo xtask build --arch pext`, the pext tier's flags, no profile) of the same engine source, one per compiler: **base** at `7099ff0` on rustc 1.98.1 (a detached worktree, removed afterwards), **candidate** at this registration's head on rustc 1.99.1 (engine source `9fb525e`, which differs from `7099ff0`'s only by `try_update` and a lint, both exact). Built by `tools/nps_build_pool.ps1 -Arch pext -Builds 2 -NoPgo -ExpectFingerprint 13987223` (`e62f907`): the second build recompiles and must be byte-identical to the first, which proves the plain build deterministic, so one binary stands for its arm.

**Read.** `python tools/nps_read.py --base <plain 1.98.1> --cand <plain 1.99.1> --out analysis/artifacts/toolchain-1.99/plain-read --cycles 10 --label "RAR-P38"`, on a host its guard reads idle, the session idle. The interval is t on the ten paired per-cycle ratios (9 degrees of freedom); the disturbance rule applies; a disturbed read is repeated. **Caveat:** each compiler gives one code layout, so the interval covers run noise only, not layout luck; a plain difference under about 1% is weak evidence either way.

**Prediction, frozen 2026-10-21 before any build:** plain delta **−1.0%**, 80% band **[−3.0%, +1.0%]**. Reading rule, fixed now: a plain delta at −1.1% or below says the code generation carries most of the PGO loss; at −0.5% or above, the loss is in the use of the profile; between, both. The PGO gain each compiler gets is the PGO delta less the plain one. Basis: C.12 saw changes read differently with and without PGO, and an LLVM major moves both; I have no stronger prior for either. Evidence layer: bench NPS, not Elo.

## Result / disposition

Pending.

## Conditional lesson and retry trigger

Pending.

## Source

RAR-P36; RAR-P37; C.12's speed pass (`analysis/c12_speed_pass_2026-10-17.md`)
