# Changing engine code

A module of `AGENTS.md`, as binding as the core. Read it in full when any
line below applies; when unsure, read it.

- Scope: implementing and qualifying a change to the engine's code or build.
- Kind: implementation or deterministic qualification.
- Paths: `src/`, `tests/`, `benches/`, `xtask/`, `vendor/`, `.cargo/`, `build.rs`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`.
- Before: suppressing a lint; calling a change behaviour-neutral.

## Research and implementation

- Implementation owns ordinary engineering: idiomatic Rust structure, necessary
  local refactoring, focused instrumentation and tests, compilation, debugging
  and cheap deterministic qualification. The maintainer need not prescribe it.
- Implementation does not replace the hypothesis, broaden the mechanism, add
  adjacent heuristics, tune unrelated constants, port extra donor behaviour,
  change the experiment after exposure, or rescue a weak candidate by changing
  its neighbours. If a material premise is false, keep useful instrumentation,
  record the contradiction and return the leaf to `RESEARCH`.
- Keep default-neutral repairs in a separate sub-step and commit cluster from
  behaviour-changing ones so each read isolates one cause.

## Per-node code

- Per-node code (negamax, qsearch, move picking, movegen, make/unmake, eval,
  SEE, TT probes) does not allocate, clone a `Board`, take a lock, touch an
  `Arc`, or pass or return a large value by copy (check the assembly for
  `memcpy`). Ownership is solved with `&mut` borrows, make/unmake and
  per-thread arrays, never by a clone that satisfies the borrow checker. An
  exception states its reason in one sentence and carries an NPS measurement
  by PROCESS's method. A shared atomic states what it signals and which search
  it belongs to. B.7.1's allocation test checks the first clause.

## Verification

- Rust engine, test, build or dependency changes
  need debug and release tests, `cargo fmt --check` and
  `cargo clippy --all-features --all-targets` at zero warnings; release alone is
  not enough.
- An evaluation, trace or parameter change also runs
  `cargo test --release --features texel --lib` and
  `cargo test --release --features tune --lib`: the trace reconstruction
  gate and the parameter tests compile only there.
- Suppress lints with `#[expect(...)]`, which warns when no longer needed; use
  `#[allow(...)]` only for a lint that fires in one feature configuration and
  not another, with its reason written.
- A behaviour-neutral engine change reproduces the immediate development
  fingerprint (currently **3,694,298 / EBF 2.324**) plus targeted checks
  for behaviour the suite does not reach: an identical bench does not prove a
  narrow feature neutral (RAR-E10). Investigate a cross-platform mismatch
  (RAR-P14, RAR-P16). The comparison fingerprint is revision-specific: a
  deliberately integrated behaviour change updates the fingerprint record;
  never preserve a known defect to keep an obsolete count.

## Comments

- Comments explain the problem or the invariant, briefly: no roadmap step
  numbers, no ledger IDs as the explanation, no narration of earlier versions.
  A measured reason to keep a shape stays, in one sentence. Rewrite a
  mechanism's comments when you rewrite the mechanism.
