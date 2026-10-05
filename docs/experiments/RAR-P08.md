# RAR-P08 — Windows ARM64 PGO with pinned Rust used `rust-lld` to work around profile-link failure

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Windows ARM64 PGO with pinned Rust used `rust-lld` to work around profile-link failure.

## Result / disposition

**Retained in 2.3.1:** about +8% NPS locally, unchanged bench/search behavior.

## Conditional lesson and retry trigger

Toolchain workarounds are versioned debt. Re-test on each pinned compiler bump and keep behavior/performance claims separate.

## Source

`CHANGELOG.md` 2.3.1
