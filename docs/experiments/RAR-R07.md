# RAR-R07 — Phase-4.9 opening profile of accepted semantics at 1/2/4/8/16T on an idle 5950X

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.9 opening profile of accepted semantics at 1/2/4/8/16T on an idle 5950X. Two instruments, deliberately independent: `nps_scaling.ps1` on the pext PGO asset (2 pinned middlegame positions x 2 reps, `movetime 5000`, Hash 256, per-position ratios) for NPS and time-to-depth, and `diag_smp_sweep.ps1` on a diag build (3 reps, `movetime 3000`) for TT contention, aspiration churn and per-thread depth.

## Result / disposition

**Throughput scales almost perfectly and DEPTH DOES NOT MOVE.** NPS **1.81x / 3.95x / 7.88x / 12.31x** at 2/4/8/16T, while depth at fixed time goes **−1.0 / −1.0 / −0.8 / −0.8 plies**. The diag sweep agrees from the other side: every thread completes depth **22 at every thread count**, 1T included, for 10.3x the nodes. This reproduces the 9.7.5(b) observation (16 threads, ~13x nodes, +0 depth) on current head. **Three of the recorded hypotheses are refuted by their own counters.** Aspiration churn: re-searches per thread FALL, 34.0 → 36.0 → 34.0 → 30.2 → 28.4, so pool-seeded windows are not the cost and if anything help. Helpers not reaching main: main TT hit rate RISES 52.1 → 53.2 → 56.8 → 57.6 → **62.5%**, so helper work does arrive. TT store duplication: same-key share rises only 26.2 → **33.8%** across a SIXTEEN-fold thread increase, and 26.2% of stores already hit the same position at 1T.

## Conditional lesson and retry trigger

**The pool contributes, does not duplicate much, does not churn its windows — and still buys no depth.** That is the finding, and it moves the question from 'is SMP working' to 'why does more TT content not make iterations cheaper'. The lead is already in the 4.1 census: at 1T only **275 of 1,447 sampled TT hits (19%) are usable for a cutoff** while 1,172 are `tt_bound_not_usable`, so a rising hit rate may be adding hits that cannot cut. Next measurement is the cutoff-usable share as a function of thread count, not a TT layout audit. ⚠ Descriptive profile, not a verdict on SMP strength: depth at fixed time is the standard SMP quality proxy but RAR-R03's +102.78 Elo was 4T-new versus 4T-old, and **no 1T-versus-4T strength measurement exists for the current engine at all**. Do not read flat depth as 'SMP is worthless'. Also note 16T returns are already diminishing on 16 physical cores (7.88x → 12.31x for a doubling), which is a bandwidth signature rather than a search one.

## Source

`tools/nps_scaling.ps1`; `tools/diag_smp_sweep.ps1`; Plan 4.9
