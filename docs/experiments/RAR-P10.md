# RAR-P10 — Phase-4.8b: port the AArch64 TT prefetch from `origin/arm_fix` onto current development and make its presence …

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.8b: port the AArch64 TT prefetch from `origin/arm_fix` onto current development and make its presence enforceable. Emission proved by compiling the exact `prefetch_ptr` body standalone for `aarch64-unknown-linux-gnu` at `-O` (the vendored Fathom C build blocks a full cargo cross-compile on this host).

## Result / disposition

**ACCEPTED on measurement: +1.42% NPS on an M4, and a silent three-release loss closed.** `prefetch_ptr` had an x86 body and `let _ = ptr;` for everything else, so **all three shipped ARM64 assets did no TT prefetching at all** while the x86 assets did. `prfm pldl1keep` (the ARM analogue of `_MM_HINT_T0`) now compiles in behind an exact `target_arch` cfg; the probe emits it at every call site and it survives `#[inline(always)]` at `-O`. `cargo xtask verify-isa --arch arm64` now REQUIRES the class, so every CI bench cell and every release asset proves it is present. x86-64 is untouched: bench 6,502,902 / EBF 2.449, all three x86 tiers still pass their contracts.

## Conditional lesson and retry trigger

**A missing cache hint is invisible to every instrument this project owns.** The engine plays identically with and without a prefetch - same nodes, same moves, same fingerprint, same tests - it merely plays slower, so node agreement across CI cells cannot see it and neither can a strength gate on x86. A REQUIRED instruction class is the only check that catches it, which generalises: for a hint-shaped optimisation, verify the instruction, not the behaviour. The A/B PLAN 4.8 item 3 requires has now run (RAR-P11) and the port is KEPT. The sibling `origin/arm_fix` commit (Apple TT cache-line alignment) is deliberately NOT ported - it has no ARM timing evidence and PLAN 4.8 item 4 requires an Apple result first.

## Source

`src/tt.rs`; `xtask/src/main.rs`; Plan 4.8b
