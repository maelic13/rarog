# RAR-P36 — Toolchain update rustc 1.98.1 -> 1.99.1, pooled NPS and the exact fingerprint — REGISTERED 2026-10-21, before any build

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Toolchain update rustc 1.98.1 -> 1.99.1 — REGISTERED 2026-10-21, before either pool is built.** PLAN C.11's note (maintainer decision 2026-10-19): the update runs after C.10.5's gate and before C.11's checkpoint reads, as its own build change, so that C.11 measures the toolchain that ships and no gate mixes toolchains between its arms. `rustup check` on 2026-10-21 reports stable **1.99.1 (58cf8f9c3 2026-10-13)**. The bump crosses an LLVM major: 1.98.1 carries LLVM 22.1.8, the 1.99 line LLVM 23.1 (read from `rustc -Vv` on 1.99.0 before registration; 1.99.1's is recorded with its pool).

- **Arms.** The same engine source, `60e865f` on `c9` (this registration's commit and the pin bump change no engine input), built once per toolchain. **Base:** the registration commit, `rust-toolchain.toml` at `1.98.1`, built before the pin is touched. **Candidate:** the pin-bump commit, which changes only `rust-toolchain.toml`'s channel and `Cargo.toml`'s `rust-version`. A lint fix the new compiler demands, if any, lands as its own exact engine commit before the candidate pool, and the candidate is built at that head; the fingerprint must still match.
- **Method.** PROCESS's pooled-PGO read: four pext PGO builds an arm from a clean tree, `tools/nps_build_pool.ps1 -Arch pext -Builds 4 -OutDir analysis/artifacts/toolchain-1.99/pool-<rustc> -ExpectFingerprint 13987223`, with `rustc -Vv` saved beside each pool; then `python tools/nps_read.py --base analysis/artifacts/toolchain-1.99/pool-1.98.1 --cand analysis/artifacts/toolchain-1.99/pool-1.99.1 --out analysis/artifacts/toolchain-1.99/read --label "RAR-P36" --no-regression`, on an idle host with nothing else running. `--no-regression` because the change is behaviour-neutral: step 1 passes at −0.1% or above and fails at −0.9% or below; only in between, `--extend` adds four cycles once and the line is −0.5%. A disturbed read is repeated, not interpreted.
- **Expected fingerprint.** **13,987,223 / EBF 2.565**, exact on both toolchains: on every pooled PGO build (the pool script refuses otherwise), on the plain `cargo build --release` and on `tools/build_test.ps1 -Suffix tc199`. A mismatch is a finding to investigate as RAR-P14 and RAR-P16 were, never a new fingerprint.
- **Qualification on the new compiler,** each once on the final state: `cargo fmt --check`; `cargo test` debug and `--release`; `cargo clippy --all-features --all-targets` at zero warnings; the bench above; `build_test.ps1` proving the xtask PGO path, whose target-feature inspection asks rustc. The unsafe floor stays 23 sites.

**Prediction, frozen 2026-10-21 before exposure:** pooled-PGO NPS **0.0%**, 80% band **[−1.5%, +1.5%]**; the fingerprint exact (probability 0.97). Basis: the last bump (RAR-P18, 1.97.1 -> 1.98.1, an LLVM 22 minor step) read −0.53% with an interval across zero, no resolvable change; this one crosses an LLVM major, which moves inlining and PGO decisions more, so the band is wider than RAR-P18's ±1%; after C.12 the engine is memory-latency bound, which damps codegen effects in either direction. Most likely failure mode: a sign-unpredictable shift of about 1% from changed PGO inlining in the move picker or evaluation. Evidence layer: bench NPS, not Elo. **No Elo expectation and no strength gate:** a compiler change gets none, by the decision; PROCESS's 2 Elo per 1% at `3+0.03` is an expectation, not a measurement.

**Stop rule.** A fingerprint mismatch on either compiler stops the update before any NPS read and is reported. A clippy or test failure is fixed in its own commit only if the fix is exact; a fix that would change behaviour stops the update and is reported. The read is a measurement: a regression is reported to the maintainer, who decides whether the pin stays; the agent does not revert it.

## Result / disposition

Pending.

## Conditional lesson and retry trigger

Pending. RAR-P18's standing obligations carry over: RAR-P08's `rust-lld` Windows ARM64 PGO workaround and the CI matrix are re-verified on every pinned bump, on hosts other than this one.

## Source

PLAN C.11; PROCESS, *Toolchain and harness notes*; RAR-P18; RAR-P19; RAR-P32
