# RAR-S28 — Owed 4.2 throughput check: does the typed-evidence refactor cost NPS? Refactor versus its parent, **three …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Owed 4.2 throughput check: does the typed-evidence refactor cost NPS? Refactor versus its parent, **three independent PGO builds per arm** (all six SHA-distinct), pooled and interleaved, `bench 13 3`, 10 cycles per direction, idle 5950X. Run in BOTH directions to cancel the estimator's slot bias.

## Result / disposition

**Retained: no measurable throughput cost.** Self-pair null on one binary read −0.25% median (CI −0.54…+0.10), so the estimator carries a slot penalty and anything under ~0.5% is artifact-prone. Forward (post as cand) −0.10% median / −0.10% best-of (CI −0.66…+0.22); reversed (pre as cand) +0.15% / +0.05% (CI −0.25…+0.54). Bias-cancelled `(fwd − rev)/2` = **−0.125% median, −0.075% best-of**, with an implied slot bias of only +0.025% — so the self-pair's −0.25% was mostly noise, not a real asymmetry. At the ~2 Elo per 1% NPS STC constant this bounds the cost at roughly 0.25 Elo, inside the noise floor. All six builds independently reproduced bench 6,502,902.

## Conditional lesson and retry trigger

Under these conditions the eager per-node `NodeEvidence` construction did not cost deployable speed, so 4.2 is clean on both behaviour and throughput. Two method notes. (a) A single self-pair on ONE binary is a weaker bias estimate than the two-direction difference; when both are available, prefer the difference — here they disagreed by 0.28pp and the difference was the better-behaved figure. (b) Build variance dominates the effect: base medians spanned 0.20% while cand spanned 0.62%, with `post42b` alone 0.6% below its siblings. That is precisely the profile luck pooling exists to average out, and it is why a single-build PGO A/B cannot resolve anything at this scale.

## Source

`tools/nps_multibuild.ps1`
