# RAR-S41 — Phase-4.5d: does any further correction CONTEXT carry usable signal? Exact residual buckets by halfmove clock …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.5d: does any further correction CONTEXT carry usable signal? Exact residual buckets by halfmove clock on `bench 13`, plus a structural check of the check/evasion context. Defaults inert; bench 6,502,902 / EBF 2.449 on normal and diag.

## Result / disposition

**Observation: no new context is justified, and for two different reasons.** Halfmove clock 0-19 holds **279,741 of 283,590 updates (98.64%)**, clock 20-49 just 2,377 (0.84%) and 50+ only 1,472 (0.52%); buckets reconcile exactly against `correction_updates`. Mean residual magnitude does differ - 130.7 / 115.8 / **61.9 cp**, so high-clock residuals are less than half the size of low-clock ones - but on 0.52% of samples that cannot support a learned context. Separately, the **check/evasion context is unreachable by construction**: correction trains only where `static_eval != VALUE_NONE`, which *is* the not-in-check condition, so its population is zero without needing measurement.

## Conditional lesson and retry trigger

Under these conditions PLAN 4.5's "add contexts only with held-out unique signal" resolves to **add none**. The instructive part is that the halfmove context fails on **population, not on signal** - its 2.1x low-versus-high ratio looks as interesting as 4.5a's 2.27x capture ratio, but 4.5a's split was 51/49 while this one is 99/0.5, so one is learnable and the other is a table of slots that never fill. **A context needs both a distinct mean and a population to learn from; checking only the mean would have justified a useless table.** That leaves the capture/quiet split as the only context with both. Retry trigger: revisit if a future corpus or TC materially shifts the clock distribution - a rule-50-heavy endgame cohort would, and Phase 5.0 freezes exactly such cohorts.

## Source

`src/search.rs`; `tools/diag_search_quality.ps1`; Plan 4.5d
