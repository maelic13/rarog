# RAR-S42 — Phase-4.6a: resolve the documented late-evasion contradiction

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.6a: resolve the documented late-evasion contradiction. The LMR comment claimed "`!in_check` removed, so late evasions are reducible" while the live predicate still carried `&& !in_check`. Comment corrected, behaviour made testable behind `LmrReduceLateEvasions`, sized on `bench 13`.

## Result / disposition

**Observation, and it resolved in favour of the CODE.** The comment was false: evasions were never reducible. Making them reducible measures **7,467,531 nodes, +14.83%** — one of the most expensive arms in Phase 4. Default 0 is inert; bench 6,502,902 / EBF 2.449 on normal, diag and tune. Also confirmed a *correct* piece of documentation for contrast: `check_extensions` reads 0 because the extension was deliberately removed and the counter is left defined as explicit confirmation — that comment says exactly what the code does.

## Conditional lesson and retry trigger

Under these conditions the stale comment was describing a change that would have cost ~15% of the tree, so the code being authoritative was the better state — reducing an evasion triggers far more LMR re-searches than it saves. It is categorical and therefore excluded from the final SPSA, stays OFF for final-theta ablation, and has no retry owner after 2.4. **Third comment/code mismatch this cycle** (after the false multicut claim and the `Corr*Scale` "seeds are 0" claim): in this codebase a comment asserting a mechanism's state is not evidence — read the predicate.

## Source

`src/search.rs`; `src/params.rs`; Plan 4.6a
