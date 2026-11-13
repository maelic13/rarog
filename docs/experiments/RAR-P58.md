# RAR-P58 — Toolchain update rustc 1.99.1 -> 1.100.0: the exact fingerprint, pooled PGO NPS and plain NPS — REGISTERED 2026-11-13, before any build

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-13 by the agent, at the maintainer's
  request, before either arm is built. The bump runs before D.6.8 and the
  2.7.0 no-regression match, so that both and the release assets share one
  compiler (the maintainer's pre-release order, 2026-11-13).
- **Kind:** performance qualification of a build change. The engine source
  does not change; every build must bench 11,477,040.
- Disposition: open
- **Why:** PROCESS, *Toolchain and harness notes*: Rarog follows the latest
  stable Rust (maintainer decision 2026-10-21); a bump is qualified, its
  speed read and recorded, and a measured slowdown does not revert it.
  `rustup check` on 2026-11-13 reports stable **1.100.0 (0ef1bc386
  2026-11-09)**. The maintainer also asked for the numbers themselves:
  the last bump, 1.98.1 to 1.99.1, read −1.63% on PGO builds (RAR-P37) and
  −3.63% on plain ones (RAR-P38).
- **Compilers.** Base `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, LLVM 23.1.1.
  Candidate `rustc 1.100.0 (0ef1bc386 2026-11-09)`, LLVM 23.1.5, read
  from `rustup run stable rustc -Vv` before registration. Unlike the last
  bump, this one stays within LLVM 23: a patch step.

## Arms

The same engine source, built once per compiler from a clean tree.
- **Base:** this registration's commit, `rust-toolchain.toml` at `1.99.1`,
  built before the pin is touched.
- **Candidate:** the pin-bump commit, which changes only
  `rust-toolchain.toml`'s channel and `Cargo.toml`'s `rust-version`. A
  lint fix the new compiler demands, if any, lands as its own exact engine
  commit before the candidate pools, and the candidate is built at that
  head; the fingerprint must still match.

Two pools an arm, each by `tools/nps_build_pool.ps1 -Arch pext -Builds 4
-ExpectFingerprint 11477040`, in `analysis/artifacts/toolchain-1.100/`:
- **PGO:** `pool-<rustc>`, four pext PGO builds; the shipped configuration
  and the deciding read.
- **Plain:** `plainpool-<rustc>`, the same with `-NoPgo`, RAR-P38's design
  2. It separates the code generation from the use of the profile and
  decides nothing.

## The read

On a host the guard reads idle, the session idle and nothing else running:

- `python tools/nps_read.py --base analysis/artifacts/toolchain-1.100/pool-1.99.1 --cand analysis/artifacts/toolchain-1.100/pool-1.100.0 --out analysis/artifacts/toolchain-1.100/read --label "RAR-P58 PGO" --no-regression`
- `python tools/nps_read.py --base analysis/artifacts/toolchain-1.100/plainpool-1.99.1 --cand analysis/artifacts/toolchain-1.100/plainpool-1.100.0 --out analysis/artifacts/toolchain-1.100/plain-read --label "RAR-P58 plain"`

`--no-regression` as in RAR-P36: step 1 passes at −0.1% or above and
fails at −0.9% or below; only in between, `--extend` adds four cycles
once and the line is −0.5%. The plain read is step 1 only, a measurement.
A disturbed read is repeated, not interpreted.

## Qualification on the new compiler

Each once on the final state: `cargo fmt --check`; `cargo test` debug and
`--release`; `cargo test --release --features texel --lib` and
`--features tune --lib`; `cargo clippy --all-features --all-targets` at
zero warnings; `cargo clippy -p xtask --all-targets` and `cargo test -p
xtask`; `cargo build --release` then `bench 13` at **11,477,040 / EBF
2.537**; `tools/build_test.ps1` proving the xtask PGO path. The unsafe
floor stays 17 sites.

## PRE-REGISTERED PREDICTION (frozen 2026-11-13, before any build)

- **Fingerprint:** exact on both compilers, probability 0.98. A mismatch
  stops the update before any NPS read and is investigated as RAR-P14 and
  RAR-P16 were, never taken as a new fingerprint.
- **PGO:** **0.0%**, 80% band **[−1.0%, +1.0%]**.
- **Plain:** **0.0%**, 80% band **[−1.5%, +1.5%]**.
- **Basis:** RAR-P36's lesson is that a bump across an LLVM major is a
  speed change; this one is an LLVM patch step (23.1.1 to 23.1.5), which
  should not move inlining or code generation materially. The one earlier
  bump within an LLVM major, RAR-P18 (1.97.1 to 1.98.1, a minor step),
  read −0.53% with an interval across zero. The plain band is wider
  because a plain build has one code layout per compiler, so its interval
  covers run noise and not layout luck (RAR-P38).
- **Most likely failure mode:** a sign-unpredictable shift of about 0.5%
  from a changed PGO inlining decision in the move picker or the
  evaluation.
- **Evidence layer:** bench NPS, not Elo. No strength gate: a compiler
  change gets none, by the 2026-10-21 decision; PROCESS's 2 Elo per 1% at
  `3+0.03` is an expectation, not a measurement.

## Stop rule

A fingerprint mismatch on either compiler stops the update before any NPS
read and is reported. A clippy or test failure is fixed in its own commit
only if the fix is exact; a fix that would change behaviour stops the update
and is reported. A regression is reported and does not revert the pin.

## Result / disposition

Not read yet.
