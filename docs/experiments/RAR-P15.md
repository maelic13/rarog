# RAR-P15 — Phase-4.8h: first full CI matrix dispatch carrying the 4.8 work — the `verify-isa` steps added in 4.8a and …

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.8h: first full CI matrix dispatch carrying the 4.8 work — the `verify-isa` steps added in 4.8a and the AArch64 prefetch added in 4.8b had never executed on the five-cell matrix. Manual `workflow_dispatch` of `ci.yml` against `development`.

## Result / disposition

**GREEN, 14/14 jobs, 4m 0s.** All five bench cells pass their ISA contract — linux-x86-64 and windows-x86-64 against `base`, and linux-arm64, windows-arm64 and macos-arm64 against `arm64`, which is what finally proves **`prfm` reaches every ARM64 asset** rather than only the one measured by hand on an M4. Cross-platform determinism passes with the prefetch in, so all five cells still agree on the fingerprint. debug x release tests pass on ubuntu, windows and macos; fmt/clippy and the feature-build job pass.

## Conditional lesson and retry trigger

**The `--default-cpu` correction was load-bearing, and it was found by inspection rather than by this run.** The macOS cell builds with `cargo build --release`, whose default `target-cpu` on `aarch64-apple-darwin` enables aes, sha2 and dotprod against `generic`'s bare neon; holding it to the tier baseline would have failed the cell for instructions it is entitled to emit. Checking that BEFORE dispatching cost minutes and saved a red matrix plus the wrong diagnosis. ⚠ Scope: this is the CI matrix, NOT the production matrix. The bench cells build plain `cargo build --release`, so the tiered PGO release path is still verified on only three cells by hand (Windows x86-64, macOS ARM64, Windows ARM64) and **both Linux cells' PGO builds remain untested at this head** — carried to 4.11, where the production platform/ISA matrix is a formal gate.

## Source

`.github/workflows/ci.yml`; Plan 4.8h
