# RAR-S40 — Phase-4.5c: is the correction-uncertainty term applied to an eval the correction is no longer part of? Exact …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.5c: is the correction-uncertainty term applied to an eval the correction is no longer part of? Exact census plus a `CorrSkipWhenTtRefined` guard, landed inert. `bench 13`, idle host.

## Result / disposition

**Observation — a real mis-application, and a large one.** `corr_abs` widens the RFP and futility margins and shrinks LMR in proportion to how far the correction moved the eval. But `eval_for_pruning` can be REPLACED wholesale by a TT bound, and when it is, the corrected eval is discarded while the margins are still widened by the discarded correction's magnitude. Exact population: **360,811 nodes, 9.0% of the 4,005,332-node tree.** Switching the term off in exactly that case measures **6,210,236 nodes, −4.50%** — the only 4.5 arm that is *cheaper* than baseline. Defaults inert; bench 6,502,902 / EBF 2.449 on normal, diag and tune. Also corrected a **stale comment** that claimed these scales were seeded at 0 and the term therefore vanished: the fitted seeds are 3/3/27, so it has been live all along.

## Conditional lesson and retry trigger

Under these conditions the term is charged for an adjustment that is not present in the number being tested, on 9% of nodes — a coherence defect rather than a tuning question, and the kind that a bench fingerprint can never reveal because it is behaviour the baseline has always had. It is a strong first-bundle candidate on both grounds: principled *and* −4.50% nodes. ⚠ Not assumed to be a gain: RAR-S30 showed TT refinement is earning strength, and a wider margin may be doing useful work for reasons unrelated to its stated rationale, so it rides the 4.4 bundle gate rather than being baked. Method note: the stale comment is the second documentation defect this cycle that would have misled a reader into thinking a live mechanism was inert — **re-read a mechanism's seeds, do not trust its comment.**

## Source

`src/search.rs`; `src/params.rs`; Plan 4.5c
