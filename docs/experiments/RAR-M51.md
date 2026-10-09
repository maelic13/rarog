# RAR-M51 — B.2.0 architecture-review measurements on the B.1 head `a8b6640`, 2026-09-14, zero games, no source changed

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.2.0 architecture-review measurements on the B.1 head `a8b6640`, 2026-09-14, zero games, no source changed.** Host idle (6–7% CPU), one thread, pinned 1.98.1. Crate outline by `rg` over declarations and `use` edges; a scratch script (`pub_surface.py`, archived with its output) counting, for every `pub` item in `src/`, its references outside its own file across `src/`, `tests/`, `benches/`, `xtask` and `tools/texel-tuner`; regex census of retired phase, step and ledger numbers and of comment lines per file; `cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery` on default features; `cargo build --release` timed after `touch src/lib.rs`; `bench 13` on the binary that build produced; declared-versus-fired `diag` counters by script. Artifacts in ignored `tools/results/b20-20260914/`, sha256: clippy_pedantic.txt `079324da7cc677225f8370524026164fcf3b16aa4ee1b9a206caef4c29be29ce`, build_release.txt `6a7c2f29d89c9c2b0689b1575226f68bc0a59bee4c5da2cf52d6132712d7e01f`, bench13_fresh.txt `576c028f326afaf46fc4619efcb6f39e3033324d01870ef7fb358ee089e8ba35`, pub_surface.txt `0443ec19946b9c77da567e3549d095ceb900a1d87ddb800c0b22e99f03f63bc4`, rarog-a8b6640-release.exe `f6b5d5bfa7ab5219c32196bb9b45efb2d05d1b454bd904463b90e7f041bf5b56`.

## Result / disposition

36 files / 21,916 lines in `src/`; 593 `pub` items, **98 referenced nowhere outside their own file**, 33 crate-`pub` used only by tests and tools; **453 retired-number references in `src/`** (176 in B.2.0-owned files, 162 in the B.2–B.5 mechanism files, 115 in `eval.rs`), 366 more across tools and configuration; comment share `params.rs` 57%, `main.rs` 46%, `diag.rs` 29%; pedantic+nursery census 1,001 warnings (200 `inline_always`, 147 `must_use_candidate`, 132 `unreadable_literal`, 100 `cast_lossless`, 85 `missing_const_for_fn`), zero under the crate's own wall; 182 `diag` counters declared, 3 never fired; `Searcher` 19 fields, `node.rs` reads `params`/`td`/`tt` 93 times through `&mut self`; 22 `info_string!` sites, 5 inside `search/`; `Board` fields read 36 times outside `board/`, written 0; release build 15.8 s wall, binary 937,472 bytes; **`bench 13` 7,601,220 / EBF 2.474, exact.**

## Conditional lesson and retry trigger

The layering is inward and sound; the weight is comments, surface and duplication, all behaviour-neutral to remove; the one strength-relevant question (per-thread state versus engine resources in the kernel) is designed and handed to B.2.1 as ticket 0 so the lines are qualified once. **Prediction frozen for the upgrades:** fingerprint exact at every commit; pooled NPS +0.0% inside ±0.5%, with the TT factoring (T6) the only ticket that can move it and a measured keep if it does; `src/` −700 to −1,200 lines. Retry/repeat trigger: E.1 re-runs this exact recipe on the B.9 and C.11 heads so the counts compare.

## Source

`analysis/architecture_review_2026-09.md`; PLAN B.2.0, B.2.1 ticket 0; `analysis/consolidation_2026-09-10.md`; RAR-P24; `tools/results/b20-20260914/` (ignored, hashed above)
