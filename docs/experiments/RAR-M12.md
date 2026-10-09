# RAR-M12 — Phase-4 step 4.0 — baseline and oracle freeze

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 step 4.0 — baseline and oracle freeze**, 2026-08-12 on the Ryzen 9 5950X. Reproduced the 2.3.2 baseline from `dev`, first confirming its code tree is byte-identical to `master` `f931722` (the diff is documentation only, so the build is the released revision). Toolchain `rustc 1.97.1 (8bab26f4f 2026-07-14)`, matching the `rust-toolchain.toml` pin.

## Result / disposition

**Closed; baseline accepted.** `cargo fmt --check` clean; all-feature workspace clippy clean at `-D warnings`. Tests **258 passed / 0 failed** in debug and **259 / 0** in release, 23 suites each — the one-test difference is deliberate and documented, `random_position_garbage_never_crashes_or_hangs` being `#[cfg(not(debug_assertions))]` because a debug engine spends ~5 s per process start and the property under test belongs to the shipped binary. Release `bench 13` = **6,519,711 nodes / EBF 2.449**, median 149,097, top-position share 6.8% (440,767), 2,939,454 nps. `--features tune` advertises **101** options: all ten options removed in 2.3.2 are absent, and the sampled later-owned inert options (`CorrSkipWhenTtRefined`, `SelectivityProspectiveDepth`, `SingularTtDepthMargin`, `RootConfPoolInstability`, `SmpIterationSkip`) are present. PGO PEXT asset `rarog-v2.3.2-windows-pext-pgo.exe` reproduces the identical fingerprint, SHA-256 `389E234ECCB725D81BEBB4030D4AF17ED181D130F0803E9948B5437E05046E28`; `verify-isa --arch pext` holds (pext 303, avx 6247, zero sse3/ssse3/sse4.1/sse4.2, tzcnt 344 permitted as `rep bsf`). Oracle `hybrid` at `75d0d43` re-verified: both frozen binaries hash byte-exact to the values recorded under the search-oracle section.

## Conditional lesson and retry trigger

This is the revision every Phase-4 candidate gates against; do not re-derive it per cluster. Two conditions are recorded rather than assumed: the profile-dependent test count is a documented `cfg`, not drift, so a future 258/259 reading needs no investigation, while any *other* asymmetry does; and the doc-only equality between `dev` and `master` is what licenses building the baseline from the integration branch — it must be re-checked, not assumed, the moment Phase-4 code lands. **Risk discharged 2026-08-12:** `hybrid` and `spsa_impr` were pushed; `origin/hybrid` matches `75d0d43` exactly, so the oracle no longer lives on one machine.

## Source

`PLAN.md` §4 step 4.0; `GUIDE.md` tracker; `tests/fuzz_lite.rs`; `target/dist/rarog-v2.3.2-windows-pext-pgo.exe`; record: `analysis/ledger_records_2026-09-14.md`, RAR-M12 (Measurement, harness and tuning)
