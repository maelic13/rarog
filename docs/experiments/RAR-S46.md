# RAR-S46 — Phase-4.7a: cover the root abort/fallback path, which `bench` structurally cannot reach

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.7a: cover the root abort/fallback path, which `bench` structurally cannot reach. Bench is fixed-depth and never aborts, so `root_interrupted_fallback` reads **0** across the whole 40-position corpus. New `tests/root_abort.rs` interrupts the search at swept poll budgets so the abort lands mid-iteration at many different points.

## Result / disposition

**Retained correctness infrastructure; no behaviour change.** Four properties now hold under abort at budgets 1-100 over four branching positions: the returned move is always LEGAL; no mate-range score is ever reported from an unfinished iteration; the reported depth never reaches the depth limit it did not complete; and aborting at the same point twice gives the same answer. All pass. Suite runtime trimmed 108s to 9.8s by lowering the depth cap, since abort is a mid-iteration property that small budgets already cover.

## Conditional lesson and retry trigger

PLAN 4.7's "abort returns last completed legal evidence" and "incomplete mate/win/loss never becomes authoritative" were previously **unverifiable claims**: nothing in the fingerprint, the tactical suites or the strength gates exercises an interrupted root. This is the third such blind spot found this cycle - after null-move soundness (`tests/zugzwang.rs`) and the zero-population decisive guard - and they share a shape worth naming: **a property that only manifests under a condition the deterministic corpus excludes needs its own test, or it is merely asserted.** Determinism is included deliberately: a fallback that varied run to run would mean ownership depends on something outside the recorded root evidence, which is exactly what 4.7 is meant to rule out.

## Source

`tests/root_abort.rs`; Plan 4.7a
