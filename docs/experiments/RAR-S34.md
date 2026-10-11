# RAR-S34 — Phase-4.3c **gate result and cost attribution.** Gate: candidate versus baseline …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.3c **gate result and cost attribution.** Gate: candidate versus baseline, final-PGO, registered `[3,10]` nElo at `3+0.03`, 1T, 64 MB, paired UHO, budget 16,000. Attribution: 4.3c peeled into its three sub-changes, three independent PGO builds each, bench fingerprints plus one interleaved 5-cycle NPS pass over all twelve binaries on an idle 5950X.

## Result / disposition

**Gate: not promoted — dead neutral.** Manually stopped at 4,960 games, Elo **+0.35 ± 6.18**, nElo +0.55 ± 9.67, W-D-L 1,261-2,443-1,256, LOS 54.42%, PairsRatio 1.00, LLR −1.71 of ±2.94 (58% toward H0 and steady). RAR-M10 predicted the drift for a truly neutral candidate to within 1% (−1.71 predicted, −1.70 observed at 4,516), so the trajectory was as designed, not anomalous. **Attribution (time-to-depth = node ratio ÷ NPS ratio):** age narrowing 5→4 bits **0.00% nodes, +0.10% NPS, −0.10% TTD — free**; plus singular rejection of speculative evidence **−0.19% nodes, +0.97% NPS, −1.15% TTD — free and slightly FASTER**; plus the actual-ProbCut-score change **+1.43% nodes, −2.79% NPS, +4.34% TTD.** The score change alone accounts for **+5.55% TTD** (v2→v3: +1.62% nodes, −3.73% NPS).

## Conditional lesson and retry trigger

**Two of my own prior claims are refuted by this.** (a) I asserted the age narrowing was bench-visible and so 4.3c "cannot land inert" — it is bench-IDENTICAL at 6,502,902 and the bit is genuinely free. (b) I attributed the headwind to the age narrowing as the likely main cost; it contributes nothing. The entire ~4.3% deficit comes from the bundled ProbCut actual-score change, which the singular contract does not need — provenance delivers that, not the stored value. The plausible mechanism ties to RAR-S30's structural finding: a higher stored `Lower` raises `eval_for_pruning`, and refinement acts almost purely as an upward correction that *prevents* razoring, so more nodes survive and the mix shifts to expensive interior nodes. **Actionable:** the contract without the score change (v2) is strictly cheaper than baseline, so it costs nothing to retain and is the variant that should be carried. Do not read this gate as evidence against the contract — it tested contract plus a costly extra. Retry the contract only inside 4.4's bundle, where "evidence-bound singularity" needs the bit anyway and a ~5 nElo prior (RAR-S31) cannot clear `[3,10]` standalone. **Landed accordingly:** the infrastructure is retained and both behaviour changes became switches defaulting off, so the head is bench-identical to the accepted baseline at 6,502,902 / EBF 2.449 with no strength gate owed. `SingularRejectSpeculative=1` reproduces 6,490,746 and adding `ProbCutStoreActualScore=1` reproduces 6,595,869, both checked against the variants built for this attribution — so the switches are exact reconstructions, not approximations.

## Source

`tools/results/sprt_43c_vs_Baseline_20260807_112013.*`; Plan 4.3c
