# RAR-S33 — Phase-4.3c gate preparation and independent verification of the landed implementation

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.3c gate preparation and independent verification of the landed implementation. Three clean-manifest PGO builds per arm (baseline and candidate) on an idle 5950X; median-NPS build selected per side. **Then:** Phase-4.3c implementation candidate: persist one speculative TT bit by reducing age 5→4 bits, store the actual ProbCut fail-high and deny that class only at singular seeding. Non-PGO release/diag `bench 13`, 1T; baseline.

## Result / disposition

**Implementation verified; the gate carries a measurable speed headwind.** Candidate fingerprint **6,595,869 / EBF 2.447** reproduced independently on normal, diag AND tune builds; 863 blocked speculative singular seeds confirmed; producer census reconciles at 2,694,580; provenance round-trips on both TT backends; fmt, clippy ×3 and tests ×3 all pass. Per-build NPS spread was 0.15% base and 0.86% candidate. **Cross-arm the candidate is −2.45% NPS (CI −2.88…−2.06) while also needing +1.43% more nodes for bench depth 13**, i.e. roughly **4% worse time-to-depth**. **Then:** **Implementation complete; strength unresolved.** Candidate fingerprint 6,595,869 / EBF 2.447 versus baseline 6,502,902. Exact diagnostic count: 863 speculative entries otherwise met the singular seed predicate. Local/shared provenance round-trip, 16-generation age wrap, unchanged per-generation replacement penalty and consumer-contract tests pass. Registered final-PGO `[3,10]`, maximum 12,000 games; only H1 promotes.

## Conditional lesson and retry trigger

Under these conditions the 4.3c contract is correctly implemented and cheap in memory (10-byte entry preserved, per-generation replacement penalty unchanged, 16-generation wrap tested). But at the project's ~2 Elo per 1% NPS constant a ~4% time-to-depth deficit is a real headwind the semantic gain must overcome before it can clear a `[3,10]` bar, so a park outcome would not by itself indict the *contract* — only this cost/benefit balance. Two structural notes for whoever reads the verdict: 4.3c bundles the singular-rejection contract AND the change from a margin-shifted to an actual ProbCut stored score, and the score half has **no ablation switch**, so a failure cannot be attributed between them; and the age field narrowing is bench-visible on its own, so unlike arms A–D this step cannot be parked inert. **Then:** Under this bench, explicit provenance reaches real consumers that the old `depth-3 + Lower` signature could not identify. The count establishes exposure, not benefit: actual-score storage, shorter age horizon and singular exclusion move together in this candidate, so the final-PGO game gate decides the bundle. If it fails, ablate those three effects rather than infer which one caused the result.

## Source

`rarog-43c-pgo.exe`; `rarog-d00e1ac-pgo.exe`; Plan 4.3c **Then:** `src/tt.rs`; `src/evidence.rs`; `src/search.rs`; Plan 4.3c
