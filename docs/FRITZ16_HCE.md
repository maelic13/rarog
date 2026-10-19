# Experimental Fritz 16 HCE

The `dragon_nnue` branch provides three selectable evaluators:

| Evaluation | Runtime | Default |
| --- | --- | --- |
| `Classical` | Rarog's existing native HCE | Yes |
| `DragonNNUE` | Bundled qualified Dragon network and Rust inference | No |
| `Fritz16HCE` | Reconstructed Fritz 16 full static evaluator in Rust | No |

```text
setoption name Evaluation value Fritz16HCE
position startpos
eval
go depth 8
```

The static diagnostic for the starting position is
`info string eval backend=Fritz16HCE stm=2 white=2`.
Switch back with `setoption name Evaluation value Classical`, or select
`DragonNNUE` using the same option. No original executable, external table
file, analysis tool or FFI library is needed by the reconstructed backend.

`vendor/fritz16-hce` contains safe Rust for the complete full evaluator and its
pawn/cache routine, mechanically translated from the supplied Fritz 16 assembly
and linked to original instruction addresses. Parameter bytes are embedded;
their original file offsets, VAs and hashes are in
`vendor/fritz16-hce/assets/provenance.json`. The target executable SHA-256 is
`264f2eede4c737f938c726e3c91e5ce07460e998110cf197abfc463dfa4622ae`.

The adapter constructs no FEN and uses each worker's preallocated state.
Construction allocates about 7 MiB per configured evaluator, including a 1 MiB
pawn/king cache. Node inference allocates nothing and takes no locks. Mode
switches use the existing evaluator, TT and correction-history invalidation.
Fritz-specific tuning options are not exposed. Imported evaluators are rejected
in `texel` builds so traces continue to describe Rarog's native HCE.

Qualification:

- 486 direct original static-evaluation snapshots match exactly, with
  independently reconstructed position fields and default root adjustments.
- Cold, warm and reset cache results agree; a changed material-table entry
  fails the comparison.
- Adapter tests cover moves, undo, castling, en-passant, promotions and null
  moves, all three selections, allocation counts and one/two-worker searches.
- A public UCI session switches through all three backends and back to
  Classical, reporting static diagnostics and legal node-limited results.
- Full debug and release suites pass, as do formatting and all-feature,
  all-target Clippy with zero warnings. Classical consumes all forty bench
  positions with its unchanged 14,197,472 / EBF 2.557 fingerprint.
- The generated PLAN/GUIDE board passes. The broad documentation guard
  still reports fourteen missing historical artifact paths; all fourteen
  references predate these changes. No strength or performance gate is claimed.

Exact throughput optimizations (2026-10-19, F.16): component 1.93x;
Fritz search +11.7% in the informational shared-host read.
Direct full-eval control flow and incremental position inputs retain
all native fixtures and all forty search trees for every backend.
See [RAR-P40](experiments/RAR-P40.md) and its artifact manifest for
timings, exact checks, binary/source hashes and limitations.

Remaining fidelity limits: this is the first address-linked Rust reconstruction;
semantic naming of all parameter tables, nondefault original options and
exhaustive boundary qualification remain open in the reconstruction project.
The original search and its lazy-evaluation shortcuts are not imported. Rarog
still owns terminal/draw handling, rule-50 policy, corrections, pruning and
tablebases, so matching static scores does not reproduce Fritz's search or Elo.

Ready-to-use Windows release executable: `target/release/rarog-fritz16-hce.exe`.
SHA-256: `3c37e72b8f49944550f7a0be517ebee0ab9173f7264326cd08ef7a315d0c2c61`.

Raw logs and their hashes are under `analysis/artifacts/fritz16-hce-20261015/`.
Reproduce the engine checks with `cargo test`, `cargo test --release`,
`cargo fmt --all --check` and `cargo clippy --all-features --all-targets -- -D warnings`.
No commits or pushes in this session, as requested.
