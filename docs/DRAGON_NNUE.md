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

Send these commands to the running engine (use an absolute weights path in a GUI):

```text
uci
setoption name Dragon Eval File value D:\code\dragon\assets\dragon-network.nnue
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

The weights file remains external. Its default name is `dragon-network.nnue`,
resolved against the process working directory. This machine's example above
uses the existing extracted asset; neither the build nor the runtime requires a
parent repository when the file is supplied elsewhere. The loader accepts only
the qualified model SHA-256
`c6e25a7fde5971dc47e78e19764c82b8d18a438bbd5ee4da065d061115064417`.
Other weights, truncated files and modified bytes are rejected.

Missing weights produce an explicit notice and keep the previous evaluator
active. If a GUI sends `Evaluation = Dragon NNUE` before its file, supplying a
valid `Dragon Eval File` completes that pending request. Selecting Classical
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
parsing. Each cache miss performs a full scalar refresh, including a 3 KiB bias
copy. This temporary copy exception preserves the qualified reference; incremental
accumulators and Rust/SIMD optimization are later work.

## Repeat the checks

The ordinary debug/release suites need no external model. Model-dependent tests
are explicitly ignored in an unconfigured checkout; run them against the asset:

```powershell
$env:RAROG_DRAGON_NNUE = 'D:\code\dragon\assets\dragon-network.nnue'
cargo test -p rarog --test dragon_nnue -- --ignored --nocapture --test-threads=1
cargo test -p rarog --release --test dragon_nnue -- --ignored --nocapture --test-threads=1
cargo test -p rarog --lib dragon_mode_scale_and_helpers -- --ignored --test-threads=1
cargo test -p rarog --release --lib dragon_mode_scale_and_helpers -- --ignored --test-threads=1
```

They check all 2,738 frozen native score rows, both perspectives, scale/mode
switches, full-refresh/cache agreement, make/unmake/null and special moves,
TT/correction invalidation and helper ownership, zero inference allocations,
and short legal searches with one and two workers.

For fresh original-process comparisons, use a new output directory:

```powershell
python tools/diag/dragon_nnue_probe.py `
  --engine target/release/rarog.exe `
  --dragon-base D:/code/dragon/originals/base/dragon-3.3-64bit.exe `
  --dragon-avx2 D:/code/dragon/originals/avx2/dragon-3.3-64bit-avx2.exe `
  --model D:/code/dragon/assets/dragon-network.nnue `
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
  --model D:/code/dragon/assets/dragon-network.nnue `
  --out analysis/artifacts/dragon-nnue-local/bench
```

It runs one `bench 13` for each evaluator, with one thread and Hash 64, and
checks all 40 records and aggregate nodes, time and NPS. It normally refuses a
busy host; `--allow-busy` records an explicit rough-measurement waiver. The
2026-10-15 run used that waiver at the maintainer's request. There are no repeats,
warm-up, PGO pool, confidence interval or playing-strength estimate. The trees
also differ between evaluators. Measurements and hashes belong to
[RAR-P36](experiments/RAR-P36.md).
