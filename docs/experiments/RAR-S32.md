# RAR-S32 — Build-transfer diagnostics for arm B on an idle 5950X: tune option and baked PGO fingerprints plus pooled NPS …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Build-transfer diagnostics for arm B on an idle 5950X: tune option and baked PGO fingerprints plus pooled NPS ratios. The first contended timing pass was void.

## Result / disposition

Both forms produced 6,100,099 nodes / EBF 2.437. Cand/base NPS was −1.15% in the same non-PGO binary and −1.355% across pooled PGO builds, about 0.2 percentage points apart under these measurements.

## Conditional lesson and retry trigger

Fingerprint equality establishes matching deterministic search decisions for this corpus; aggregate NPS similarity suggests no large throughput interaction. Neither establishes final-PGO game strength or authorizes a baseline bake. Under the new policy arm B is parked because another long gate for an observed ~3 Elo is low priority. Timing evidence is valid only on an idle host.

## Source

`tools/test_engines/rarog-43a-tune.exe`; `rarog-43b-*-pext-pgo.exe`; Plan 4.3b
