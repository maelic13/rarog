# RAR-P29 — B.7.2.9 candidate 3 falsifier, the selection scan's local speedup, COMPLETE 2026-10-02

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.7.2.9 candidate 3 falsifier, the selection scan's local speedup, COMPLETE 2026-10-02.** A sampling build on `0d95763` recorded every 32nd `pick_next` tail over `bench` (536,249 tails, mean 13.84; bench unchanged). A standalone microbenchmark (toolchain 1.98.1, pext flags, fat LTO) replays each tail as a copy plus one selection, minus a copy-only pass, for the engine's 16-byte entries and for scores held contiguously. Rule fixed in PLAN B.7.2.9 before running: `s` ≥ 1.25 goes on, < 1.1 closes, between is the maintainer's.

## Result / disposition

All variants select the same entry on every tail. Same loop on contiguous scores: **`s` = 1.106 and 1.093** in two runs on an idle host (1.8–4.0% CPU); maximum-then-position 0.76. Two earlier runs (1.114, 1.099) at 9–12% load with a video playing are superseded; they agree within spread. Copy-only cost 10.5 → 15.5 ns per tail with two arrays, so the layout is slower overall in the microbenchmark. Ceiling about +0.6% before that cost.

## Conditional lesson and retry trigger

On the closing bound: decision to the maintainer, `NO_CHANGE` recommended. Retry only if the scan's share rises well above 6% or a layout change is made for another reason.

## Source

PLAN B.7.2.9; `analysis/artifacts/b72-c3-falsifier/`
