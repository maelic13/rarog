# RAR-S38 — Phase-4.5: is a capture-caused correction residual actually noisier than a quiet-caused one? Exact per-class …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.5: is a capture-caused correction residual actually noisier than a quiet-caused one? Exact per-class residual magnitudes on `bench 13`, plus a graded `CorrCaptureWeightPct` alternative to the existing binary `CorrGuardCapture`, landed inert.

## Result / disposition

**Observation, and it SUPPORTS the premise for the first time.** Capture-caused: **145,372 updates (51.26%), mean \|residual\| 179.1 cp**. Quiet-caused: **138,218 updates, mean 78.8 cp**. **Ratio of means 2.274.** Both denominators reconcile exactly against `correction_updates` (145,372 + 138,218 = 283,590) and `correction_on_capture`. `CorrCaptureWeightPct = 100` is inert; `bench 13` unchanged at 6,502,902 / EBF 2.449 on normal, diag and tune builds.

## Conditional lesson and retry trigger

This retroactively explains RAR-S16. The capture guard was **directionally right** — capture residuals really are ~2.3x larger, so the positional eval is being asked to absorb much bigger surprises from tactical cutoffs — but the **instrument was wrong**: excluding them discards 51.3% of all training, and RAR-S16 measured that at −55.98 Elo. Scaling preserves coverage while down-weighting the noisier class, which is what the plan asked for and what the evidence now justifies. Under these conditions a weight near 100/2.27 ≈ 44% would roughly equalise each class's contribution per update, but that is a **tuning** question for the 4.10 fit, not a value to bake here — RAR-S13 is the precedent against baking an untuned constant. Retry trigger: the weight enters 4.10 as a coordinate; do not gate it standalone, since a correction weight is exactly the kind of consumer constant lesson 2 says to fit after architecture.

## Source

`src/search.rs`; `tools/diag_search_quality.ps1`; Plan 4.5
