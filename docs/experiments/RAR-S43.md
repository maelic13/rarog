# RAR-S43 — Phase-4.6b: derive LMP, futility and SEE pruning from the same PROSPECTIVE depth LMR will search the move at …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.6b: derive LMP, futility and SEE pruning from the same PROSPECTIVE depth LMR will search the move at, instead of from raw `depth`. One shared reduction formula extracted so both callers use it, with a `debug_assert` that they agree. Sized on `bench 13`.

## Result / disposition

**Diagnostic, and the largest cheap arm in Phase 4.** `SelectivityProspectiveDepth = 1` measures **5,937,163 nodes, −8.70%**, with EBF falling 2.449 to **2.424** — a genuinely narrower tree, not just a cheaper one. Default 0 inert; bench 6,502,902 / EBF 2.449 on normal, diag and tune. The shared formula is verified rather than asserted: a `debug_assert_eq!` at the LMR site checks it derives the same reduction units as the pre-move estimate, and it held across the whole debug test suite.

## Conditional lesson and retry trigger

Under these conditions the audit's finding is confirmed and fixed: a move about to be reduced by 3 plies was being judged for pruning as if it were not, and making the two coherent removes 8.7% of the tree. The three consumers move together on ONE knob by design — switching them individually would recreate exactly the mixed-depth incoherence the step exists to remove. Two terms are excluded from the shared depth and documented rather than hidden: the per-thread jitter, which must be drawn once at the real reduction site and is not drawn at all at `Threads = 1`; and the singular extension, because pruning runs before the extension is known. ⚠ Pruning MORE is not automatically better — RAR-S27's arm pruned 43.8% more and lost — but that arm restricted EVIDENCE while this one aligns a decision with the depth actually searched, which is a different kind of change. It is the strongest single member of the 4.10a bundle and still needs the gate.

## Source

`src/search.rs`; `src/params.rs`; Plan 4.6b
