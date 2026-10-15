# Fritz 16 HCE in Rust

This library reconstructs the full static-evaluation routine of the supplied
Fritz 16 executable, identified by SHA-256
`264f2eede4c737f938c726e3c91e5ce07460e998110cf197abfc463dfa4622ae`.
It has no runtime dependency on that executable, another engine, an external
model, Windows APIs or an analysis runtime. Rust 2024, minimum Rust 1.98.

```rust
use fritz16_hce::{Evaluator, Position};

let position = Position::from_fen(
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
).unwrap();
let mut evaluator = Evaluator::default();
assert_eq!(evaluator.evaluate(&position), 2); // side-to-move perspective
```

For engine integration, construct `Position` directly from twelve disjoint
bitboards (white P/N/B/R/Q/K then black P/N/B/R/Q/K), a side-to-move flag and
four castling bits. Squares are a1=0 through h8=63. Validate externally supplied
boards with `Position::validate`; inference expects a valid board. Each worker
owns an `Evaluator`. Construction allocates about 7 MiB; inference allocates
nothing. `clear` resets its 1 MiB pawn/king cache. `evaluate_white` exposes the
original white-perspective score, and `evaluate` changes perspective only.

The default search-root material adjustments, tempo, piece-square tables,
material signature, pawn/king cache, mobility, threats, king danger, passer races
and material scaling are retained. Host search still owns terminal positions,
repetition, rule-50 policy, pruning, corrections and tablebases. This component
does not reconstruct Fritz's search or its search-dependent lazy shortcuts.
Fritz-specific tuning controls are not exposed by the Rarog adapter.

`src/recovered.rs` contains safe Rust basic blocks mechanically translated from
the saved assembly at `0x14009e270` and `0x1400ca0f0`, with every original
instruction retained in an adjacent comment. These are compiled Rust statements,
not executed machine-code bytes or an FFI call. The complete original parameter
section bytes and their coordinates/hashes are in `assets/provenance.json`.
The first version intentionally retains addresses rather than assigning
unproved semantic names to every table and temporary.

Validation includes 486 direct original-process static-evaluation snapshots,
independently reconstructed position-state fields, original root adjustments,
cold/warm/reset cache comparisons, arithmetic boundaries and rejection of a
mutated material parameter. This finite corpus does not prove all possible
positions or establish playing strength. Full-engine PLAN gates remain open.
See [HCE contract](docs/contracts/HCE.md) and
[capture authorization/method](docs/info-requests/C.8-static-oracle.md).

```powershell
cargo test
cargo test --release
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Regenerate from verified saved evidence using `python tools/lift_hce.py`, then
`cargo fmt`. Capture tools require Windows and the immutable original;
building and using the library requires neither. Rarog's `dragon_nnue` branch
vendors this library and selects it with `setoption name Evaluation value
Fritz16HCE`; Classical remains default and DragonNNUE remains available.
