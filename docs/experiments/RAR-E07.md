# RAR-E07 — 4.8a redundancy inventory on the accepted vector

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**4.8a redundancy inventory on the accepted vector.** Cross-referenced `04-final.txt` (SHA `BAD51F3E...`, the accepted fit) against the source vector and the run's own per-slot `feature-support.log`. Zero games, zero compute -- every input already existed.

## Result / disposition

**Closed without a gate; nothing to remove.** The fit drove only **5 of 1,218** slots to zero, of which 3 are whole 1-slot terms (`passed_candidate_mg`, `passed_freestop_eg_per_rank`, `threat_safe_pawn_push_eg`); it also switched **17** previously-zero slots back on. Of the 132 slots under the sparse cut, **90 are structurally unreachable** (0 activations), 12 are the nonlinear danger selectors, 12 are co-tuned safety-table entries, and the remaining 18 are rare-but-real and **all 18 held**.

## Conditional lesson and retry trigger

Basilisk's BAS-E25 removed sixteen terms that a previous phase had **added**; Rarog's existing surface has no equivalent accumulation, so the analogue does not transfer. Two instrument confirmations fell out of this for free: the 12 zero-linear-activation fields are exactly `KS_DANGER_INPUTS`, independently reproducing 4.7.3's 1,194+12+10+2 partition from a different artifact, and the fitter froze every under-supported coefficient instead of fitting noise into it. **Unreachable is not redundant** -- a pawn PST is 64 entries because the index space is 64, and 16 of them are ranks 1 and 8.

## Source

`tools/results/hce-fit-20260831_095443/{04-final.txt,feature-support.log}`; `tools/results/hce-confirm-20260831_230548/source-vector.txt`
