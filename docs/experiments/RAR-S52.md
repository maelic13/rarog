# RAR-S52 — Search-quality ratio readout at the 2.3.1 head

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Search-quality ratio readout at the 2.3.1 head. `bench 13`, 1T, 40 per-position diagnostic dumps aggregated by `tools/diag_search_quality.ps1`. Counters only, bench-identical.

## Result / disposition

**Observation.** First-move cutoff rate **87.65%** (372,605 / 425,098); LMR over-reduction, re-search over applied, **1.80%** (17,900 / 996,204); cutoff nodes over interior nodes 13.75%. Captures delivered 1.86× more cutoffs than quiets. Reduction was clamped to ≥1 ply, so no late move escaped reduction; LMP discarded 3.71M moves against 3.09M interior nodes and RFP cut 21.9% of interior nodes.

## Conditional lesson

87.65% is only marginally under the ~90% healthy band, so raw move-ordering quality is **not** where a 40-Elo class deficit lives — the pre-registered "the deficit is ordering" branch is disfavoured. The rate is mostly carrying the TT/SEE-sorted head of the list, and a counter cannot separate healthy selectivity from over-selectivity; treating either ratio as a verdict would be exactly the failure this ledger's first rule forbids. The clamp, LMP and RFP figures size the selectivity surface for a game gate; they do not price it.

## Source

`tools/diag_search_quality.ps1`; `src/diag.rs`; branch `spsa_impr` at `36bced4`
