# RAR-P14 — Phase-4.8g: retest the Windows ARM64 PGO path on the pinned toolchain

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.8g: retest the Windows ARM64 PGO path on the pinned toolchain. PLAN 4.8 item 2 records the `rust-lld` workaround for rust-lang/rust#156675 as toolchain-versioned debt that must be re-verified on every rustc bump; it is what broke Windows ARM64 PGO before 2.3.1 and had never been checked on 1.97.1. Native Windows ARM64 host, `cargo xtask build --arch arm64 --pgo`.

## Result / disposition

**PASSES — the last release-blocking unknown in 4.8 is cleared.** The instrumented binary trained, `llvm-profdata merge` accepted the profile (`Merging 1 profile file(s)`) and the optimised build linked to `rarog-v2.4.0-windows-arm64-pgo.exe`. Fingerprint **6,502,902 / EBF 2.449**, so **three platforms now agree exactly** — Windows x86-64, macOS ARM64 and Windows ARM64. ⚠ The ISA contract check did NOT run: `verify-isa` looked for the non-PGO asset name while the PGO one sat beside it (a tool bug, since fixed), so `prfm` presence on the Windows ARM64 asset is still owed as one command.

## Conditional lesson and retry trigger

**'The workaround still works' is not 'the workaround is still needed.'** `xtask` forces `rust-lld` for `aarch64-pc-windows-msvc` unconditionally, so this run exercised the workaround path and proves it functions on rustc 1.97.1 / LLVM 22.1.6 — it says nothing about whether upstream has fixed the underlying defect. Removing the override and retesting is the cheap way to find out, and is the right move on some future toolchain bump rather than now, since the override costs nothing while it holds. Retry trigger unchanged: re-run this on every pinned-rustc bump.

## Source

`xtask/src/main.rs` `linker_flags`; Plan 4.8g
