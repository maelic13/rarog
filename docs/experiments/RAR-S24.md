# RAR-S24 — Phase-4.2b shadow test at `7815054`: what a confidence/depth penalty on window-contradicting inexact bounds …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.2b shadow test at `7815054`: what a confidence/depth penalty on window-contradicting inexact bounds would change. `bench 13`, 1T, sampled 1/1024, diagnostic build, no behaviour change (fingerprint 6,502,902).

## Result / disposition

**Observation, and it contradicts the hypothesis that motivated it.** Exposure is **269 of 1,447 sampled hits (18.59%)** — 2.4× the previously reported 113, which counted only the cutoff-eligible subset. Score consumers are materially exposed: 85 of 269 (31.6%) moved `eval_for_pruning`, mean shift **123.7 cp**; **41 of 101** sampled singular attempts were seeded by a contradicting score, 16 of which changed depth; 63 IIR suppressions. Depth-slack histogram (0/1/2-3/4-7/8+ = 20/19/16/22/8) prices a penalty directly: P=1 blocks 23.5% of those refinements, P=2 45.9%, P=4 64.7%, P=8 90.6%. **But the control pair reverses the ordering assumption:** a contradicting entry's move was best **91.79% (179/195)** of the time versus **84.77% (167/197)** for an agreeing entry — contradiction made the move a *better* ordering hint, not a worse one (z ≈ 2.2, p ≈ 0.03).

## Conditional lesson and retry trigger

Under these conditions the penalty must be **split by consumer, not applied to the entry**: it belongs on the score consumers (eval refinement, singular seeding) and must leave move ordering and IIR alone. A plausible mechanism is that a `Lower` at or below alpha records an earlier fail-high whose *move* was genuinely strong while its *score* is stale for this window — so score and move staleness are not the same property, and a single per-entry confidence scalar would destroy real ordering evidence to fix a scoring problem. Caveats: one deterministic bench corpus, sampled, not independent games, so this constrains 4.3's design rather than settling its size. The 123.7 cp figure is not comparable to the whole population's 1,445 cp mean, which is inflated by mate-score refinements. Retry as a gated 4.3 arm; a contradicting entry can never cut off (unit-tested), so cutoffs need no arm at all.

## Source

`src/diag.rs`; `tools/diag_search_quality.ps1`; `7815054`; Plan 4.2b–4.3
