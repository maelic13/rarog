# Donor evaluation bridge: Rarog's search on Stockfish's classical evaluation

A diagnostic instrument for E.4.0.1, never Rarog code and never a release.
It is RAR-O07's mirror: there Rarog's evaluation ran inside Stockfish's
search, here one Stockfish tag's classical evaluation runs inside Rarog's
compact kernel, in place of Rarog's evaluation and under the kernel's own
evaluation wrapper (correction, material weighting, halfmove damping,
optimism). Played against the kernel on Rarog's own evaluation, both arms
run Rarog's driver and count nodes alike, so an equal-node read needs no
correction.

| Donor | Tag | Patch | DLL | Bridge build |
|---|---|---|---|---|
| Stockfish 11 | `sf_11` (`c3483fa9`) | `sf11-bridge.patch` | `sf11_eval.dll` | `--features sf11` |
| Stockfish 16 | `sf_16` (`68e1e9b3`) | `sf16-bridge.patch` | `sf16_eval.dll` | `--features sf16` |

## What crosses

- **Stockfish's side** (`<tag>-bridge.patch`): `Position::set_placement`,
  the FEN setter's steps in its order from a piece array; for `sf_16`,
  `Eval::classical_value`, the classical evaluation without `evaluate()`'s
  network choice, halfmove damping and clamp, which the kernel's wrapper
  does instead; and `rarog_bridge.cpp`, the C ABI. Each calling thread gets
  one Stockfish `Thread` for its pawn and material tables, made on its
  first call and never freed. `sf_11`'s contempt is zero, as its `eval`
  command sets it. `sf_16`'s lazy cut reads the root's last score, which
  the kernel hands over after each root window, and minus `VALUE_INFINITE`
  before the first, as `sf_16`'s own search starts.
- **Rarog's side**: the `donor-eval` feature (`src/eval/mod.rs`): an
  installed `fn(&Board, Option<i32>) -> i32` replaces `Evaluator::evaluate`,
  read into the evaluator when it is made, and the search sets the root
  score (`src/search/node.rs`, `src/search/mod.rs`). Without an installed
  evaluation nothing changes, and without the feature it compiles out.
- **The bridge binary** (`bridge/`): `src/main.rs`'s `main` with the
  donor installed, the FFI in its own crate so the engine's `unsafe` floor
  is unchanged. The DLL is linked by name (`raw-dylib`) and sits beside
  the executable.

## Scale

`SCALE_1024` maps the donor's units onto Rarog's by the rule frozen in
RAR-O13: each evaluation's K on `hce-v4-tb`'s validation rows, positions
not in check, fitted as the tuner fits its K; the scale is K_donor over
K_rarog, so a score predicts the same result in both. `donor-k <rows>`
measures it. `donor-scale <fens>` prints the size ratios the rule was
chosen over (mean |·|, RAR-O08's rule, is dominated by Stockfish's
known-win scores; RAR-O13 records them).

## Checks

    rarog-donor.exe donor-parity <fens> <out.tsv>   # placement against the tag's FEN setter
    python -I tools/donor-eval/release_check.py <sf11|sf16> <release exe> <out.tsv>

`donor-parity` evaluates every position not in check through both
setters at four root scores and exits 1 on any difference; it also counts
the positions whose value moves with the root score (`sf_16` only). The
release check compares the table with the release binary's `eval` command
to its printed precision.

## Building

In Git Bash with MSYS2's MinGW64 `g++` installed:

    bash tools/donor-eval/build_dll.sh sf11 <stockfish clone> <out dir>
    RUSTFLAGS="--cfg rarog_pext -C target-cpu=x86-64-v3 -C target-feature=+bmi2" \
      cargo build --release --manifest-path tools/donor-eval/bridge/Cargo.toml --features sf11

then copy `<out dir>/sf11_eval.dll` beside
`tools/donor-eval/bridge/target/release/rarog-donor.exe`. The DLL is
statically linked and imports only `KERNEL32.dll` and `msvcrt.dll`.
`--features probes` adds the kernel's rate probes, for diagnosis only.
The bridge's `bench 11` is its fingerprint (`bench 13` runs for minutes
on `sf_16`; RAR-O13 says why).

## Bridge price

`RAROG_HCE_BRIDGE_REPEAT=<n>` rebuilds each position `n` times and
evaluates it once, without changing a score, so
`tools/oracle-hybrids/bridge_price.py` reads the bridge's share of the
time a node as for RAR-O08's hybrids.

Licence: Stockfish is GPL-3.0. The patches are diagnostic diffs against
its tags and ship nowhere.
