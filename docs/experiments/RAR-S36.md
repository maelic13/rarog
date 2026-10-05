# RAR-S36 — Phase-4.4b guards landed inert and sized: NMP cut-node guard, NMP decisive-window guard, NMP …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.4b guards landed inert and sized: NMP cut-node guard, NMP decisive-window guard, NMP static-vs-TT-refined null threshold, and a singular extension cap. Plus a new `tests/zugzwang.rs` soundness suite. `bench 13` deterministic node counts; idle host.

## Result / disposition

**Diagnostic, no strength claim.** Each arm alone versus 6,502,902: `NmpRequireCutNode=1` **7,440,358 (+14.42%)**, `SingularMaxExtension=1` 6,930,264 (+6.57%), `NmpUseStaticEval=1` 6,675,647 (+2.66%), and **`NmpDecisiveGuard=1` 6,502,902 (0.00% — zero population on bench)**. All defaults inert. The zugzwang suite passes with every switch off, each on individually, and all ten on together.

## Conditional lesson and retry trigger

Every 4.4b arm **costs** nodes, so none belongs in a first bundle on current evidence — RAR-S34 showed a +4.34% time-to-depth candidate landing dead neutral, and `NmpRequireCutNode` at +14.42% would need to buy a great deal. The decisive guard is the interesting case: zero bench population means the **fingerprint cannot verify it in either direction**, so it is a soundness guard rather than a strength arm, and `tests/zugzwang.rs` is the only evidence that enabling it is safe. That suite is the reusable product here: null-move unsoundness is invisible to bench fingerprints and tactical suites because a bad null cutoff yields a *plausible* move, so it needed its own instrument. Method note: the suite's first draft asserted properties of positions that did not have them — a "one legal move" position that was stalemate, a "blocked draw" the engine correctly scored −1352 because the kings were not symmetric, and two "in check" positions that were checkmate. **Probe a position's legal-move count, check status and score before asserting anything about it**; three of six tests failed on my premises, not on engine behaviour.

## Source

`tests/zugzwang.rs`; `src/params.rs`; Plan 4.4b
