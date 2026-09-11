# Rarog HCE maturity against the classical Stockfish reference

Status: current Phase-4 decision record

Analysis date: 2026-08-25; method/order updated 2026-08-31 after Basilisk and
Rarog corpus/instrument audits

Rarog baseline: accepted RAR-S70 search, `bench 13` 6,977,070 / EBF 2.466

Classical reference: Stockfish `9587eeeb`, frozen by `hybrid` at `75d0d43`

## Verdict

Rarog's HCE has mature infrastructure and broad classical coverage, but it is
not yet a mature *fitted model* under the standard Phase 4 now requires. It is
approximately 1,200 traced tapered parameters with caches, symmetry and
reconstruction checks, nonlinear king danger, material imbalance, passers,
threats, mobility, specialized endgames, rule-50 damping, lazy evaluation and
search correction history. The old description "Stockfish-11-class feature
list" remains fair. Feature-name coverage is not functional equivalence:
Stockfish's classical HCE conditions the same concepts more strongly on king,
material, pawn, attacker-victim and conversion context.

The reciprocal hybrid result establishes a large aggregate population, not a
recoverable per-family budget. With Stockfish search held constant,
Stockfish-HCE beat Rarog-HCE by about 328.6 Elo in the stopped no-adjudication
RAR-O02 sample. The result says that evaluation matters greatly. It does not
say which family owns that Elo, that family losses add, or that copying a
Stockfish formulation will transfer.

The pre-NNUE objective is therefore narrower and testable: qualify every
parameter and fitting instrument, refit the complete existing representation,
improve only residual-supported families, gate each completed model in games,
and leave a mature classical fallback and teacher. No parameter family is
frozen by historical acceptance. Matching Stockfish's classical strength is
not an acceptance criterion.

## Corrections to the 2026-07-13 audit

The historical `analysis/hce_analysis.md` found four concrete activation
defects. All four are already fixed in current code and must not be scheduled
again:

| Historical finding | Current disposition |
|---|---|
| Two-pawn overlap missing from `attacked2` | Fixed; pawn overlap is inserted before other attack-map consumers |
| Enemy rook behind a passer depended on a friendly rook | Fixed; independent activation is regression-tested |
| "Unstoppable" passer ignored defending pieces | Fixed; the bonus is denied when the defender has non-pawn material |
| Support and same-rank phalanx were conflated | Fixed; separate phalanx tables and activation tests exist |

The remaining work is conditional representation, calibration and validation,
not another correctness sweep over those four items.

## Measured local evidence

| Evidence | What it licenses |
|---|---|
| RAR-E01 staged Texel programme: about +240 Elo on the external cohort | Linear fitting can create large value when the representation and starting weights are immature |
| RAR-E03 Stockfish-label distillation: -17.11 Elo despite 4.9% lower holdout loss | Teacher-fit loss cannot promote an HCE and Stockfish scores are not target weights |
| RAR-E04 on-policy refresh: -1.28 +/- 2.79 Elo despite better validation | More labels on an unchanged representation are not automatically useful |
| RAR-E05 anchored refresh: +11.56 +/- 5.19 Elo, moving 57/1,204 weights mostly 1 cp | Small constrained refits can still pay; preserve anchoring and attribution |
| Manta MAN-E19: 25 audited concepts plus constrained fit, +35.91 +/- 11.19 Elo at -36.2% evaluator throughput | Coverage audit -> coherent structure -> constrained fit -> one gate is a sound process; broad structure still owes an NPS price |
| Manta MAN-E05/MAN-E07: reference-family scaling and imbalance lost 16.32 and 7.00 Elo | Reference fidelity is a hypothesis, never a promotion rule |
| Manta MAN-E18: a fitted imbalance block improved static loss slightly but lost about 7 Elo | Fitting cannot rescue a semantically weak block; materially covariant terms must actually be free and move |
| Manta MAN-E20: a context-space fit crossed its required sign, missed its registered validation floor and cost 3.693% evaluator speed | Semantic direction, static-loss floor and NPS are valid prospective refutation filters, never promotion criteria |
| Manta MAN-E21: shelter-moderated danger was plausible and faster but lost 6.44 Elo | Mechanistic plausibility and throughput do not predict playing strength |
| Basilisk BAS-E21: nonlinear king-safety refit accepted +2.64 +/- 2.05 Elo | Existing capped/bucket-selecting surfaces need a native re-evaluation fitter; a linear trace cannot price them safely |
| Basilisk BAS-E23: full-surface refit accepted +9.52 +/- 4.66 Elo after 768 PSTs were unfrozen | Historical stage boundaries can conceal the largest remaining gain; enumerate actual parameter coverage before adding features |
| Basilisk 5.9 additions: sixteen terms plus distilled fit lost 77.92 Elo; the terms were later neutral when removed | Feature-list completion is lower priority than correct labels, instruments and whole-surface calibration |
| Basilisk BAS-E18: self-play labels recovered about 75 Elo of distillation harm, but all three label-source arms failed to improve | Correct labels prevent damage but do not create headroom on a saturated surface; stronger-engine outcomes are not automatically better targets |

Manta's successful constrained HCE run used 3,000,000 balanced training rows
plus 166,667 validation and 166,667 frozen-test rows from 1,162,814 unique
starts. It moved 142 weights by at most 7 cp and reduced frozen-test loss from
0.104461517 to 0.102574754 before the integrated MAN-E19 game gate. The useful
lesson is not that those corpus sizes or deltas are universal: adequate data,
semantic rails and whole-cluster gating worked together. Later sweeps returning
to the same attractor supplied a stop rule against reflexively adding data or
epochs.

Basilisk's validation deltas inverted as strength predictors: a 6.2% holdout
improvement lost 77.92 Elo, while a 0.43% improvement won 9.52 Elo. The
portable lesson is methodological only. Rarog must neither rank fits by loss
magnitude nor assume Basilisk's successful families will transfer.

## Rarog corpus and instrument closure

The 2026-08-31 native audit turns two previously qualitative obligations into
exact contracts:

| Item | Measured disposition |
|---|---|
| Corpus | Existing 600,000 independent games support 2,300,000 equal-phase train rows plus 127,778 validation and 127,778 test rows; 3,000,000 is impossible because train openings stop at 460,752 |
| Provenance | Zero replayed starts and parse errors; both sidecars bind one clean engine, book hash/seed, disjoint book ranges, 8,000 nodes/move and conservative `datagen-v1` adjudication |
| Content | 6,501,318 unique eligible rows, all WDL labels, 6,428 natural mates, 26,935 exact material signatures and material coverage from both-queen to pawn-only positions |
| Parameter registry | All 1,218 `EvalParams` scalars have exactly one primary disposition: 1,194 linear, 12 nonlinear king-danger selectors, 10 material/PST gauge anchors and 2 invariant king material slots |
| Nonlinear overlap | The coordinate fitter additionally co-tunes the 40-entry king-safety table; alternating stages now load a strict complete prior vector instead of restarting from source defaults |
| Reconstruction | `--verify` compares the independently accumulated `EvalTrace::raw` directly with reconstruction; the former check was tautological because it derived its residual from the value being checked |
| Feasibility | Complete rows are CSR sparse (about 76 nonzero coefficients per position on the existing corpus), removing the former roughly 11-GiB dense matrix and zero multiplication |
| Pipeline | Smoke moved nonlinear weights through both linear stages, baked a changed source/fingerprint, then restored and forcibly rebuilt exact RAR-S70 at 6,977,070 / 2.466 |

The full recipe and measurements are in
`analysis/hce_archive_audit_2026-08-31.md`. These results close the question of
whether every *existing* `EvalParams` degree of freedom can be fitted. They do
not declare Stockfish-only structure necessary, mature or valuable. Exact
material scalers, broader winnability and richer conditional features remain
structural hypotheses for the post-fit residual, not omissions to insert before
the existing surface has a verdict.

## Current maturity matrix

This table compares contracts, not source shape or constants. A family is
complete only when current code is classified as equivalent, intentionally
different with evidence, accepted after a fitted gate, or rejected after a
dependency-complete test.

| Family | Current Rarog state | Difference from `9587eeeb` worth resolving | Required evidence |
|---|---|---|---|
| Score foundation | Tapered MG/EG, tempo, rule-50 damping, lazy/full paths and traced linear deltas exist | One global phase interpolation, coarse initiative that increases score magnitude, and possible train/serve drift from lazy evaluation | Full-versus-lazy cohort residuals; material/phase calibration; sign-preserving winnability tests |
| Material, PST and imbalance | Broad, historically fitted and now covered by the exact complete instrument; ten PST anchors remove the material/PST null direction without losing a represented score | Fewer material-conditioned interactions; compensation may still alias activity and pawn terms | Complete constrained joint refit, exact-material residuals and post-fit activation/covariance review |
| Pawns and passers | Extensive pawn cache, support/phalanx, candidate/passed terms, paths, blockers, rook relations and king proximity | Conditionality remains shallower for blocker ownership/type, file effects, exchange safety and conversion/race context | Passer-rank/file/material cohorts, paired counterfactuals and Syzygy-labelled conversion sets |
| Mobility, activity and space | Per-count mobility, outposts, x-rays, closedness and space terms exist | Pin-aware mobility and usable/reachable space are incomplete; several space coefficients are zero | Legal-versus-geometric mobility traces, activation/covariance and NPS before any hot-path expansion |
| Threats | Attacker/victim tables, hanging, weak and restricted relations exist | Safe pawn pushes, overload/pin context and exact attacker-defender relations are less conditional | Tactical counterfactuals, qsearch/depth-N disagreement and SEE-safe activation reports |
| King safety | Nonlinear danger table, attacker units, safe checks, shelter/storm and flank inputs exist | Shelter/storm is low-dimensional; castling destination, pinned defenders and several flank/weak-ring inputs are absent, broad or fitted to zero | Queen/phase/shelter cohorts, legal safe-check tests, activation/covariance and a pooled-PGO cost gate |
| Scaling and endgames | KPK/KBNK, fortress/partial scalers, OCB and rule-50 handling exist | Dispatcher and material-specific conversion are narrower; OCB scope and generic winnability remain coarse | Exact material signatures, Syzygy WDL/DTZ, non-amplification and won/drawn/cursed separation |
| Calibration and data | Exact all-slot instrument coverage and a provenance/content-qualified 2.30M/127,778/127,778 publication contract now exist; corpus publication and the complete current-source fit remain | The archive was generated by the immediate Rarog lineage rather than the final RAR-S70 search; transfer and residual calibration remain empirical | Atomic output hashes, complete trajectory, one-time frozen test, semantic/cohort review and a clean PGO game gate |

## How the Stockfish comparison may be used

Use the reference to enumerate a family, map its dependencies and design
counterfactual tests. Do not use equal-mask Elo loss as Rarog headroom. A
matched ablation measures a family's marginal value inside each evaluator,
including its scale, overlap and downstream search interaction. Search Phase
4 already demonstrated that this quantity is not transplantable.

The reciprocal HCE instrument is still useful as a coarse sensitivity map if:

1. every mask bit is mechanically proved live;
2. each arm stays in a readable score band;
3. outputs are normalized for score scale and checked for gross calibration;
4. the result ranks questions only, never implementations or expected Elo;
5. local residual, activation, covariance and NPS evidence must agree before a
   Rarog structural candidate is registered.

## Improvement and fitting programme

1. **Publish the qualified evidence base.** The archive, label and capacity
   audits are complete. Publish the deterministic 2.30M/127,778/127,778 split
   once, retain the rule-50 clock and freeze every input/output/setting hash.
2. **Retain the exact instrument partition.** The 1,218-slot partition and
   end-to-end smoke are complete. Re-run coverage, reconstruction, support and
   pipeline assertions in the production command; any changed total or dead
   wire aborts the fit.
3. **Refit the complete existing surface first.** Jointly fit every identifiable
   linear degree of freedom, including PSTs and historically staged families;
   use the correct re-evaluation/coordinate or finite-difference instrument for
   king danger and other capped nonlinear parameters. Alternate only under a
   registered schedule, bake the whole vector and gate it in games.
4. **Select at most two structural clusters.** Selection requires a missing or
   misspecified local signal, meaningful activation, manageable covariance and
   an acceptable projected NPS cost. Likely areas are king-safety
   conditionality, material-specific winnability/endgames, and passer/threat
   conditionality, but the corpus chooses.
5. **Fit each changed cluster locally.** Hold categorical semantics fixed, fit
   all moved and materially covariant existing weights with anchoring, validate on
   frozen cohorts, and retain initial vector plus full optimization trajectory.
   Apply registered semantic/loss/NPS filters as refutation only, bake clean
   PGO and run the registered SPRT. Untuned draft weights are not a fair test.
6. **Consolidate after structural changes.** If any representation changes,
   rerun the complete linear/nonlinear schedule. Require a settled
   validation-selected vector rather than a transient best epoch. A second data
   cycle is allowed only if registered beforehand and supported by the first
   held-out report and game gate; stop at a failed/same-attractor cycle.
7. **Use SPSA only for the residue it can identify.** HCE SPSA is optional and
   limited to activated nonlinear/global parameters that the linear trace
   cannot fit. Search-margin SPSA is separate and occurs only after the HCE is
   accepted and held fixed for that search experiment, over cp-valued consumers
   whose populations demonstrably moved.
8. **Re-measure search, then checkpoint.** Rebuild raw/lazy/corrected/qsearch/
   depth-N residuals plus qsearch/TT/branching and cp-margin evidence on the
   accepted HCE; compare the final fitted HCE/search with the RAR-S70 search
   baseline using revision-matched final-PGO binaries, no adjudication, NPS,
   STC and an LTC direction check. Ablate surprising contributors.

## Stop rules and maturity bar

- Two fully fitted structural HCE clusters without an accepted gain stop new
  representation work and force a re-audit of the corpus and family map.
- Lower Texel or teacher loss never accepts a candidate.
- Zero or sign-flipped fitted weights trigger activation/covariance review;
  they do not by themselves prove a chess concept useless.
- A material NPS loss must be priced in the same game gate; evaluator
  throughput alone is not search NPS.
- Phase 4 may call the HCE mature only when the family matrix has no unknown or
  first-draft row, every real parameter has a verified fitting/disposition
  owner, accepted representations reconstruct through `EvalTrace`, the complete
  existing-surface fit and any post-structure consolidation have game verdicts,
  and any SPSA is either completed and gated or explicitly skipped.
