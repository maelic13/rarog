# RAR-S50 — Phase-4.10a: rebuild the accumulated-bundle composition from MEASURED subsets, as RAR-S45 requires

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.10a: rebuild the accumulated-bundle composition from MEASURED subsets, as RAR-S45 requires. `bench 13` is deterministic and fixed-depth, so all **64 combinations** of the six inert members were measured exhaustively — one exact run each, no reps, no noise. Tune build, current head.

## Result / disposition

**Every recorded figure reproduces exactly, and the accumulation strategy fails anyway.** Individually: Prospective −8.70%, CorrSkip −4.50%, NmpSuppress −2.95%, SingReject −0.19%, RazorTtPv +0.11%, NmpDecisive 0.00% — all six match the record to the digit, and the six-member set reproduces RAR-S45's **+4.57%** exactly. Sum of individuals is **−16.23%** against a set effect of **+4.57%**, a 20.8-point swing. Marginal effect averaged over all 32 subsets of the others: only **Prospective (−4.24%)** and NmpSuppress (−0.35%) still help in company; **SingReject flips to +4.75%** (worst +22.10%) and CorrSkip to +1.18%. Pairwise interactions are almost all ANTAGONISTIC — CorrSkip+NmpSuppress +9.65%, CorrSkip+SingReject +6.94%, Prospective+NmpSuppress +5.38%. **The best of all 64 is a PAIR, Prospective+CorrSkip at −9.46%**, and every third member makes it worse (+NmpSuppress → −5.53%, +SingReject → −3.73%).

## Conditional lesson and retry trigger

**The 4.4d 'accumulate before gating' strategy is refuted by its own accumulation.** Deferring the gate to collect a bigger bundle assumed the members would add; they subtract. These mechanisms prune overlapping regions, so each one claims savings the others were going to make, and the ceiling over the entire 64-point space is −9.46% — barely better than the single best member alone (−8.70%). ⚠ And node count is not Elo: the only calibration this project owns says a **+7.36% tree change was worth −1.49 ± 2.87 Elo over 23,044 games** (RAR-S27/S49), i.e. approximately zero. A −9.46% bundle therefore has an expected effect squarely inside the `[3,10]` dead zone §2 exists to keep off the gate queue — the same conclusion 4.4d reached, now with the composition measured instead of assumed. Also corrects RAR-S36: NmpDecisiveGuard reads 0.00% ALONE but has a real marginal effect in company (mean +0.84%, range −6.99% to +7.08%), so 'zero bench population' is true only in isolation.

## Source

`src/params.rs`; Plan 4.10a
