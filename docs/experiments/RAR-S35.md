# RAR-S35 — Phase-4.4a switch sizing: five mechanisms landed inert, then each measured alone on `bench 13` (deterministic …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.4a switch sizing: five mechanisms landed inert, then each measured alone on `bench 13` (deterministic node counts) with exact diagnostic populations taken with every switch OFF. Idle 5950X; `bench 13` = 6,502,902 / EBF 2.449 on normal, diag and tune builds with all defaults.

## Result / disposition

**Diagnostic, no strength claim.** Populations: the shared `tt_pv` veto blocks all four forward-pruning mechanisms at **24,754** nodes, of which RFP would reach 21,689 (87.6%), NMP 16,544 (66.8%), ProbCut 16,536 (66.8%) and razoring 8,218 (33.2%). Nested nulls inside a verification subtree: **83** exact. IIR at a PV node: **1** sampled. Node cost of each switch alone versus 6,502,902: `NmpSuppressNullInVerification` **6,310,949 (−2.95%)**, `RazorAllowTtPv` 6,509,913 (+0.11%), `NmpAllowTtPv` 6,797,234 (+4.53%), `RfpAllowTtPv` 6,936,480 (+6.67%), `ProbCutAllowTtPv` 7,577,452 (+16.52%).

## Conditional lesson and retry trigger

Under these conditions the 4.4 bundle splits cleanly by cost, which is the lesson 4.3 paid ~100k games to learn. Cheap or free: NMP subtree suppression (−2.95% nodes), razoring `tt_pv` eligibility (+0.11%) and the already-measured 4.3c contract (−1.15% time-to-depth). Expensive: ProbCut `tt_pv` at +16.52%, RFP at +6.67%, NMP at +4.53% — each must buy a lot to survive, and RAR-S34 showed a +4.34% time-to-depth candidate landing dead neutral. Build the first bundle from the cheap set only; hold the expensive three for a second bundle if the first passes. Two items are also **de-scoped by measurement**: PV-safe IIR has a population of ~1 sampled node, so it is a correctness tidy-up and not a strength arm; and node deltas here are per-switch at fixed depth, not Elo, and compound unpredictably when combined (all four `tt_pv` switches *raise* node counts even though three of them enable pruning).

## Source

`src/params.rs`; `tools/diag_search_quality.ps1`; Plan 4.4a
