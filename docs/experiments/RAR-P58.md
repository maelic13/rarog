# RAR-P58 — Toolchain update rustc 1.99.1 -> 1.100.0: the exact fingerprint, pooled PGO NPS and plain NPS — REGISTERED 2026-11-13, before any build; READ 2026-11-13: PGO −0.24%, pass

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-13 by the agent, at the maintainer's
  request, before either arm is built. The bump runs before D.6.8 and the
  2.7.0 no-regression match, so that both and the release assets share one
  compiler (the maintainer's pre-release order, 2026-11-13).
- **Kind:** performance qualification of a build change. The engine source
  does not change; every build must bench 11,477,040.
- Disposition: accepted — read 2026-11-13: the fingerprint exact on both compilers;
  PGO −0.24% (95% −0.41% .. −0.07%) passes step 2's −0.5% line; plain −0.25%.
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

**PASS: the fingerprint exact on both compilers; pooled PGO NPS −0.24%,
inside the −0.5% line; plain −0.25%, not resolved.**

- **Commits** (on `pre-2.7.0`): registration `449e3dd`; `cc987be`, the
  one fix the new compiler demanded: clippy 1.100's `redundant_clone`
  (denied in `Cargo.toml`) finds four test sites where a board is cloned
  and the original never used again (`tests/board_v2.rs`,
  `tests/multipv_syzygy.rs`, `tests/board_correctness.rs`, and
  `src/search/node.rs` inside its `#[cfg(test)]` module); each now moves
  the value. Test code only, exact. Pin `c020904`: `rust-toolchain.toml`
  channel `1.100.0`, `Cargo.toml` `rust-version = "1.100"`, nothing else.
- **Compilers** (`rustc -Vv` in `analysis/artifacts/toolchain-1.100/`):
  base `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, LLVM 23.1.1; candidate
  `rustc 1.100.0 (0ef1bc386 2026-11-09)`, LLVM 23.1.5, auto-installed by
  rustup from the pin and resolved as `1.100.0-x86_64-pc-windows-msvc`.
- **Pools** (`analysis/artifacts/toolchain-1.100/`, each with its
  manifest): PGO `pool-1.99.1` from `449e3dd` (clean) and `pool-1.100.0`
  from `c020904` (clean), four distinct builds each; plain
  `plainpool-1.99.1` (content `2EF2EC186997A99B`) and `plainpool-1.100.0`
  (content `D3BB649609EAD7B1`), four builds each identical but for link
  stamps. Every copy benched **11,477,040**.
- **Qualification on 1.100.0** (`analysis/artifacts/toolchain-1.100/verify/`,
  each once on the final state): `cargo fmt --check` exit 0; `cargo clippy
  --all-features --all-targets` and `cargo clippy --all-targets` exit 0,
  **0 warnings** (4 errors before `cc987be`); `cargo clippy -p xtask
  --all-targets` 0 warnings; `cargo test` **525 passed, 0 failed, 12
  ignored**; `cargo test --release` **527 passed, 0 failed, 11 ignored**;
  `--features texel --lib` **287 passed**; `--features tune --lib` **286
  passed**; `cargo test -p xtask` **57 passed**; `cargo build --release`
  then `bench 13`: **11,477,040 / EBF 2.537** exact;
  `tools/build_test.ps1 -Suffix tc1100` (pext PGO through xtask) exit 0 at
  11,477,040. No `unsafe` site was touched.
- **PGO read** (`read/`, `read.log`, `read-step2.log`), guard-idle host
  (CPU 0.7–2.7% before it; 1.1–2.3% before each cycle), no disturbed cycle.
  Step 1: levels 2,825,650 against 2,819,224, **−0.23%**, 95% −0.45% ..
  −0.00%, between the two lines, so step 2 ran as registered. Step 2, six
  cycles: per-build medians base 2,819,218 / 2,821,297 / 2,821,994 /
  2,826,509, candidate 2,817,489 / 2,813,004 / 2,813,690 / 2,817,488;
  levels **2,822,254 against 2,815,418**; **−0.24%**, 95% **−0.41% ..
  −0.07%** (t 2.447, df 6); verdict "pass: no regression" against the
  −0.5% line.
- **Plain read** (`plain-read/`, `plain-read.log`), step 1 only: levels
  **2,551,244 against 2,544,899**, **−0.25%**, 95% **−0.70% .. +0.21%**;
  no disturbed cycle. The tool's "closed NO_CHANGE" is its acceptance
  wording; as registered this is a measurement, and it is not resolved.
- **Against the last bump:** 1.98.1 to 1.99.1 cost 1.63% with a profile
  and 3.63% without one (RAR-P37, RAR-P38); this one costs about a quarter
  of a percent either way, so the profile neither gains nor loses
  materially relative to the plain code.
- **Elo, expected not measured:** about −0.5 at `3+0.03` by PROCESS's 2
  Elo per 1% NPS.

**Prediction calibration, appended after exposure.** Predicted: the
fingerprint exact (0.98), PGO 0.0% in [−1.0%, +1.0%], plain 0.0% in
[−1.5%, +1.5%]. Observed: the fingerprint exact; PGO −0.24% (−0.41% ..
−0.07%), plain −0.25% (−0.70% .. +0.21%). All three calls held. The PGO
point estimate missed in **sign** only within the band: a small, resolved
loss where 0.0% was predicted, the size RAR-P18's minor step read. The
assumption that an LLVM patch step moves nothing measurable was slightly
optimistic; that it moves less than a major step held by a factor of
about seven. **Unpredicted:** a clippy failure in test code, which the
stop rule covered as an exact fix. **Instrument note:**
`nps_build_pool.ps1` must be launched with PowerShell 7 (`pwsh`); a first
launch under Windows PowerShell 5.1 failed on cargo's stderr before any
build and was discarded.

**Retry trigger:** the next pinned bump re-runs this row.
