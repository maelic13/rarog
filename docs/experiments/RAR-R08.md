# RAR-R08 — Phase-4.9b: measure the cutoff-USABLE share of TT hits against thread count, the lead RAR-R07 left

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.9b: measure the cutoff-USABLE share of TT hits against thread count, the lead RAR-R07 left. `tt_bound_not_usable` was first SPLIT by cause (shallow / PV / excluded / wrong-window), because it lumped three unrelated reasons and only one of them can grow with threads. Same sweep as RAR-R07: diag build, 1/2/4/8/16T, 2 positions x 3 reps, `movetime 3000`.

## Result / disposition

**The hypothesis is REFUTED — TT hits get BETTER with threads, and depth still does not move.** Usable share rises 11.0 → 11.0 → 12.4 → 13.5 → **15.6%** and shallow rejections FALL 74.1 → 73.8 → 70.5 → 69.8 → **67.0%**, the opposite of the prediction. Combined with the rising hit rate this is **usable cutoffs per probe 5.63% → 9.89%, +76%**. Wrong-window rejections rise only 8.3 → 10.8%; pv/excluded is flat at 6.4–7.9%; the shallow deficit is **thread-independent at 2.4–2.5 plies**. So ALL FOUR recorded hypotheses are now dead. The per-thread depths say why: at 16T they read 22,21,20,22,22,22,21,21,21,22,20,22,21,22,22,22 — **no thread ever exceeds the depth 1T reaches alone**. `thread_id` seeds only the LMR jitter and the root-move rotation; the iteration loop `for depth in 1..=max_depth` is IDENTICAL on every thread, so there is **no depth staggering** and no thread is ever ahead to populate the table with entries the others could skip to.

## Conditional lesson and retry trigger

**The pool is diversified in WIDTH and not at all in DEPTH, and that is a missing mechanism rather than a tuning problem.** Every thread's TT hits come from its own previous iteration, which is exactly why the shallow deficit sits at ~2.4 plies regardless of thread count and why 67–74% of hits cannot cut — a structurally normal figure for iterative deepening, and NOT a defect, which is the second thing this measurement corrects. The unrealized headroom is concrete: at EBF 2.449, 10.34x the nodes is **2.61 plies** of theory at 16T (2.27 at 8T, 1.48 at 4T) and the realized gain is **0**. ⚠ PLAN 4.9 forbids reopening worker diversification 'without a specific measured independent-work failure' — this is that measurement, so the guard is satisfied rather than bypassed. Iteration staggering is a Threads>1 behaviour change and needs a 4T/8T gate; it must not be landed on this profile alone.

## Source

`src/search.rs`; `src/diag.rs`; `tools/diag_smp_sweep.ps1`; Plan 4.9b
