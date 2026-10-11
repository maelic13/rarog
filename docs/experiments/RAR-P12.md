# RAR-P12 — Phase-4.8e: does the Apple 128-byte cache line cost anything? `SharedCluster` is `align(64)`/64 B, so on …

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.8e: does the Apple 128-byte cache line cost anything? `SharedCluster` is `align(64)`/64 B, so on Apple Silicon two INDEPENDENT clusters share one line (RAR-P11). Sized BEFORE building any layout change: 1/2/4-thread NPS scaling, 2 pinned middlegame positions x 2 reps, `movetime 5000`, Hash 256, per-position ratios. M4 (4P+6E, PGO arm64) against a same-protocol 5950X reference (pext, non-PGO) measured the same day.

## Result / disposition

**NO MATERIAL FALSE SHARING — question CLOSED with no code change.** ARM64 scales **1.96x at 2T and 3.89x at 4T**; x86 scales **1.87x and 4.12x**. ARM is within 6% of x86 at 4T and is BETTER at 2T. The pre-registered rule (>=3.8x at 4T closes it, <=3.0x opens the layout experiment) is met, so the rejected `arm_fix` change stays rejected, `SharedCluster` keeps its 64 B layout, and the three-arm density-controlled experiment is not built.

## Conditional lesson and retry trigger

**Sizing the population beat building the fix**, again — the same discipline as 4.4a and 4.5d. The half-a-cache-line fact is REAL and the cost of it is not measurable, so the honest output is a corrected comment rather than a layout change that would have halved TT density or altered associativity and needed a strength gate to resolve. ⚠ The comparison is a RATIO comparison, never raw NPS (PLAN forbids that across machines), and it is still soft: 4 P-cores on a fanless laptop against 16 desktop cores, PGO against non-PGO, n=2. It is strong enough to decline a speculative layout change, not strong enough to certify ARM SMP quality — time-to-depth belongs to 4.9. ⚠ Method note: the first ARM script reported `seldepth`, not `depth`, because `sed 's/.*depth .../'` is GREEDY and matched the later of the two fields; the apparent depth regression it showed was an artifact and no depth conclusion is drawn here.

## Source

`src/tt.rs` (comment only); Plan 4.8e
