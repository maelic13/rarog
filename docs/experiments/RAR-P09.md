# RAR-P09 — Phase-4.8a: freeze the per-tier ISA contract and make it EXECUTE

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.8a: freeze the per-tier ISA contract and make it EXECUTE. Disassembled every x86-64 asset with `llvm-objdump` and classified every mnemonic; added `cargo xtask verify-isa`, which asks rustc what the tier's `target-cpu` enables and holds the artifact to it. Windows x86-64, pinned rustc 1.97.1.

## Result / disposition

**Two shipped defects found, both invisible to every existing gate.** (1) The `x86-64` baseline asset contained **15 `popcntq`**, every one from `vendor/fathom`: `-C target-cpu` is a rustc flag that `cc` never sees and Fathom selected hardware popcount without a feature test. Fixed by defining `TB_NO_HW_POP_COUNT` when the target lacks the feature. (2) The startup CPU guard **could never fire in a specialized asset** because the statically required feature made detection fold to true. The dead guard was removed and README states the measured requirement per asset.

## Conditional lesson and retry trigger

**Three transferable lessons.** (a) An ISA tier is only as strong as its weakest translation unit. (b) A runtime check for a statically-required feature is true by construction; keep specialized binaries simple and consider only a complete baseline universal dispatcher in Phase 8.1. (c) Ask the compiler, not folklore: derive contracts from `rustc --print cfg`.

## Source

`build.rs`; `src/main.rs`; `xtask/src/main.rs`; `README.md`; `.github/workflows/`; `PLAN.md` Phase 8.1
