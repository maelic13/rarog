# Experimental Dragon NNUE evaluation

This branch can use either Rarog's classical hand-crafted evaluation or the
extracted Dragon 3.3 neural network in Rarog's own search. Classical remains
the default. The imported model is a comparison experiment requested by the
maintainer; it does not replace Phase F's own-data NNUE release requirements.

## Build and select

```powershell
cargo build --release --locked
.\target\release\rarog.exe
```

Send these commands to the running engine:

```text
uci
setoption name Evaluation value Dragon NNUE
setoption name Dragon NNUE Scale value 72
isready
position startpos
eval
go depth 6
```

`Evaluation` is a UCI combo with values `Classical` and `Dragon NNUE`.
To switch back:

```text
setoption name Evaluation value Classical
isready
```

The qualified model is committed under `assets/nnue/dragon-network.nnue` and
embedded at build time. `Dragon Eval File` defaults to `<embedded>`, so selection
works from any working directory without an external checkout or weights file.
To use a separate copy, set `Dragon Eval File` to its absolute path. Set it back
to `<embedded>` to use the bundled asset. The loader accepts only the qualified
model SHA-256
`c6e25a7fde5971dc47e78e19764c82b8d18a438bbd5ee4da065d061115064417`.
Other weights, truncated files and modified bytes are rejected.

Missing weights produce an explicit notice and keep the previous evaluator
active. If a requested external file cannot load, supplying a valid `Dragon Eval File`
completes that pending request. Selecting Classical
cancels the pending request. A bad replacement file leaves the resident model
and its path unchanged. `Dragon NNUE Scale` accepts 10..500 and defaults to
Dragon's 72. Texel tracing builds reject Dragon selection so their HCE traces
remain meaningful.

## Static evaluation and parity

The diagnostic `eval` command invokes the same `Evaluator::evaluate` path used by
search and prints, for example:

```text
info string eval backend=DragonNNUE stm=52 white=52
```

`stm` is side-to-move relative; `white` is White relative and matches Dragon's
native `nnue` command at the same scale. With Black to move in the starting
position the output is `stm=52 white=-52`. Search applies its own corrections,
rule-50 handling, pruning and minimax, so a depth-one search score or matching
best move is not a static-evaluation comparison. Send `stop` and `isready`
before `eval` during a search; the diagnostic refuses an active search while
keeping input responsive.

The scalar runtime was reconstructed in Rust from the saved Dragon code/bytes,
then integrated through Rarog's board representation. It adds no HCE material,
tempo or endgame terms to the network score. Search retains Rarog's existing
draw and mate rules, correction machinery and fitted margins. Changing the
mode, scale or model configuration clears evaluator caches, TT entries and
correction history before new searches; helpers clear their private state
without clearing the live shared TT.

Implementation and exact model provenance are in `vendor/dragon-nnue/`.
Evaluation borrows one immutable process-lifetime model across workers, with
stack scratch and no per-node heap allocation, locks, Arc operations or FEN
parsing. Each worker owns about 408 KiB of lazy accumulator entries, allocated
when Dragon is configured. Entries are keyed by perspective and exact king
square; bitboard differences update only removed/added features. They remain
correct across skipped evaluations, null moves, unmake and arbitrary position
jumps, without NNUE hooks in Board or search. All eight auxiliary heads are
cached so material changes preserve scaling/head selection.

Runtime dispatch selects dense AVX-VNNI, AVX2 or a portable vectorizable Rust
kernel at cache construction. Fused common deltas avoid repeated accumulator
loads. The float layer is transposed at load and vectorized across outputs,
preserving the reference's operation order and separate rounding. A cold entry
copies its 3 KiB bias once; warm evaluations use deltas. The full scalar
`Network::evaluate` path remains an independent correctness reference.
The audit, rejected sparse-layout experiment and measurements are in
[RAR-P37](experiments/RAR-P37.md).

## Repeat the checks

The ordinary debug/release suites now include model conformance, worker and
cache tests using the bundled asset. No model environment variable is needed:

```powershell
cargo test -p rarog --locked
cargo test -p rarog --release --locked
cargo test --manifest-path vendor/dragon-nnue/Cargo.toml
cargo test --manifest-path vendor/dragon-nnue/Cargo.toml --release
cargo fmt --manifest-path vendor/dragon-nnue/Cargo.toml --check
cargo clippy --manifest-path vendor/dragon-nnue/Cargo.toml --all-targets -- -D warnings
```

They check all 2,738 frozen native score rows, both perspectives, scale/mode
switches, full-refresh/cache agreement, make/unmake/null and special moves,
TT/correction invalidation and helper ownership, zero inference allocations,
and short legal searches with one and two workers. Every supported forced
kernel also matches all fixtures and the scalar reference over legal walks,
unmake and arbitrary jumps. Runtime-unit tests compare vector integer and
float intermediates bit for bit. `RAROG_DRAGON_NNUE` remains an optional test
override for the identical model at another path.

For fresh original-process comparisons, use a new output directory:

```powershell
python tools/diag/dragon_nnue_probe.py `
  --engine target/release/rarog.exe `
  --dragon-base D:/code/dragon/originals/base/dragon-3.3-64bit.exe `
  --dragon-avx2 D:/code/dragon/originals/avx2/dragon-3.3-64bit-avx2.exe `
  --model assets/nnue/dragon-network.nnue `
  --out analysis/artifacts/dragon-nnue-local/native
```

The tool checks complete executable/model hashes, records every command/output,
consumes exactly one static result per case and rejects a changed-score control.
It compares 2,770 cases per original: the frozen rows plus 32 live move-prefix
positions, including castling, en passant, captures and all four promotions.
All eight heads, every king square for both colors and both sides are covered.
This is finite diagnostic-layer parity, not an oracle for hidden activations,
Dragon's search or its complete regular evaluation entry.

For the requested informational single-run speed comparison:

```powershell
python tools/diag/dragon_nnue_bench.py `
  --engine target/release/rarog.exe `
  --model assets/nnue/dragon-network.nnue `
  --out analysis/artifacts/dragon-nnue-local/bench
```

It runs one `bench 13` for each evaluator, with one thread and Hash 64, and
checks all 40 records and aggregate nodes, time and NPS. It normally refuses a
busy host; `--allow-busy` records an explicit rough-measurement waiver. The
2026-10-15 run used that waiver at the maintainer's request. There are no repeats,
warm-up, PGO pool, confidence interval or playing-strength estimate. The trees
also differ between evaluators. Measurements and hashes belong to
[RAR-P36](experiments/RAR-P36.md).


For the optimized component benchmark, `cargo bench --bench dragon_nnue` runs
three rotated rounds against the full scalar reference with exact checksums.
For engine A/B/A comparisons use `tools/diag/dragon_nnue_opt.py`; the baseline
build, arguments, workload, source/binary guards and measured results are in
[RAR-P37](experiments/RAR-P37.md). The ordinary default-feature release build
selects the supported CPU kernel automatically; no native build flag is needed.
