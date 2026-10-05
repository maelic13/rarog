# RAR-P11 — Phase-4.8c: the ARM64 verdict run PLAN 4.8 item 3 reserves, plus the Apple topology probe item 4 requires

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.8c: the ARM64 verdict run PLAN 4.8 item 3 reserves, plus the Apple topology probe item 4 requires. MacBook Air M4 (4P+6E, fanless), mains power, idle; two revision-matched `--arch arm64 --pgo` builds differing in ONE line of `src/tt.rs`; 12 interleaved `bench 13` rounds, 1 thread.

## Result / disposition

**PREFETCH ACCEPTED. +1.42% NPS** — baseline median 5,194,011, candidate 5,267,640. **12/12 paired wins with ZERO distribution overlap** (baseline max 5,219,022 < candidate min 5,248,508), sign-test p = 0.00049; per-round gain +0.89% to +2.09%. The candidate is also STEADIER (spread 0.48% versus 1.12%), which is what removing memory stalls looks like. Both builds fingerprint **6,502,902**, matching x86 exactly, so the hint is behaviour-neutral as it must be. The ISA contract behaved as its own negative control: the baseline asset FAILED with `REQUIRED prefetch never appears` and the candidate passed. Topology: `hw.cachelinesize` **128**, `hw.pagesize` **16384**, L1d 64 KB, L2 4 MB.

## Conditional lesson and retry trigger

**Two lessons.** (1) **The 128-byte Apple cache line is real, but `3ee4660` aimed at the wrong hazard.** Neither cluster type can straddle a 128 B line (32 and 64 both divide 128, both self-aligned), so the alignment wrapper addresses something that cannot happen. The REAL exposure is that `SharedCluster` is `align(64)` and documented as 'exactly one cache line' — true on x86-64, FALSE on Apple Silicon, where two independent clusters share a line and two threads can contend over unrelated entries. That is a Threads>1 ARM64 question and needs a 4T ARM A/B before anything is over-aligned, since naive padding would halve TT density. (2) **PGO does not reach the vendored C on macOS:** `cc` rejects the inherited `-fprofile-use`, so Fathom builds unoptimised there — the same 'the build contract reaches the Rust half only' shape as RAR-P09's popcnt finding, low impact because it is tablebase-probe code, but it is a gap in the release pipeline rather than a quirk.

## Source

`src/tt.rs`; Plan 4.8b, 4.8c
