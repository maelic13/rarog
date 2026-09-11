# Board-v2 instrument and correctness corpus

## Scope

RAR-M25 completes PLAN 4.11b.2. It adds a Rarog-only, versioned board corpus;
it does not alter `cross-engine-board-v1`, `tests/board_performance.rs`, board
semantics, evaluator behavior, or playing strength.

## Corpus and oracle

`tests/data/board-v2.tsv` has ten named FENs. Its tags mechanically require
single and double checks with evasions, legal and pinned-illegal en passant,
quiet and capture underpromotions, white and black king/queen castling, and a
sparse position with halfmove/fullmove counters 97/143. The independent
`tools/diag/board_v2_oracle.py` uses `python-chess` 1.11.2 to produce
`tests/data/board-v2-oracle.tsv`, including canonical legal-EP FEN, sorted
legal and capture UCI identities, perft and divide results. Regenerate/check:

```powershell
python tools/diag/board_v2_oracle.py --check
```

`tests/board_v2.rs` consumes the static oracle. It validates the categories,
canonical FEN, legal/capture identities, perft/divides and all tracked board
state (hashes, occupancies, pieces and checkers) after normal, hinted, staged,
null, clone and long unwind paths. It corrupts a known EP move, perft total and
board state and requires the preflight to reject the actual failed construct.
`tests/slider_backends.rs` independently walks coordinate rays through every
relevant blocker subset on both backends.

## Isolated timing instrument

`benches/board_v2.rs` builds its inputs before timing and reports five separate
primitives: legal generation, capture generation, staged generation,
make/unmake only and threshold SEE only. It prints every sample and a final
`black_box`-dependent checksum. `tests/board_v2_allocations.rs` arms a
test-local allocator only after warming these paths and observed zero
allocations. Rust's `std::hint::black_box` is the portable compiler barrier;
there is no no-op non-GNU/Clang fallback.

Run and archive a result with:

```powershell
python tools/diag/board_v2_run.py --output analysis/artifacts/board-v2-YYYYMMDD
```

The runner records the exact cargo command, git state, rustc/cargo versions,
host/processor, Rust flags, and SHA-256 hashes of every benchmark input beside
the raw benchmark output. RAR-M25's magic-backend run is preserved at
`analysis/artifacts/board-v2-20260906/`. It is a local baseline only: short
samples and a changed corpus make it neither a cross-engine comparison nor an
NPS/strength result.
