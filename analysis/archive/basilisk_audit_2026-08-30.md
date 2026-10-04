# Basilisk method and results audit

Status: directional evidence incorporated into Rarog Phase 4

Audit date: 2026-08-30

Basilisk initial snapshot: `66638ae04bbce4d1ffdecfcf19fd7d65ce88e5f7`

Follow-up snapshot: `7551803d287e404cbc746b2e3ba743c463a8b80c`

The earlier uncommitted 5.7.3 extension-stacking work is resolved in the
follow-up snapshot: the candidate knob was removed, its diagnostic counters
were retained, and the worktree was clean when rechecked. Only the committed
measurements below are treated as results.

Scope: recent work from 2026-08-21 through 2026-08-30, with older evidence read
only where the recent experiments depend on it. Sources were head code, recent
commits, `EXPERIMENTS.md`, `PLAN.md` and the named analysis records. Basilisk's
`GUIDE.md` contains duplicated stale sections, so its old status prose was not
treated as authority.

## Verdict

Basilisk does not supply a Rarog candidate or expected Elo. It supplies a
strong correction to the order of work. Its recent HCE programme gained about
12 Elo entirely by fitting old surfaces that had been excluded—the nonlinear
king-safety funnel and 768 PST values—while sixteen newly added terms produced
no gain and were removed. The portable hypothesis is that Rarog may also be
mis-calibrated before it is under-featured.

The main methodological consequences are:

1. prove which instrument reaches every parameter slot;
2. verify label contents, independent starts and chess coverage rather than
   trusting filenames/manifests alone;
3. smoke the complete fit/bake/rebuild path with deliberate movement;
4. fit all identifiable existing linear and nonlinear degrees of freedom before
   adding evaluation features; and
5. remeasure search populations after the accepted HCE changes.

## What worked

| Evidence | Result | Direction for Rarog |
|---|---:|---|
| BAS-E21 nonlinear king-safety refit | **+2.64 +/- 2.05 Elo**, accepted after 46,864 games | Use re-evaluation/coordinate or finite-difference fitting for capped/bucket-selecting HCE surfaces; a linear trace cannot safely price selector movement |
| BAS-E23 full-surface fit with 768 PSTs unfrozen | **+9.52 +/- 4.66 Elo**, accepted after 9,092 games | Audit actual parameter coverage and refit historical surfaces; “already tuned” and a group named `all` are not evidence |
| BAS-E25 removal of refuted terms | **+0.49 +/- 2.96 Elo**, simplification accepted | Once a term is repeatedly redundant on a complete covariant surface, remove it separately under loss-permitting bounds |
| Existing-surface Phase-5.9 total | about **+12 Elo** | Calibration can still pay without new concepts; the number is Basilisk-local and is not a Rarog prior |
| Full-surface instrument coverage | 1,190 real parameters classified by gradient, coordinate descent, gauge or exclusion | Rarog needs an exact `EvalParams::FLAT_SIZE` accounting, including nonlinear/capped and algebraically redundant coordinates |
| Fit pipeline smoke | Caught a runner that fitted PSTs and silently discarded all 768 during bake | A broad fit must prove vector, source and fingerprint movement before production compute or games |

The king-safety result also rescued a candidate correctly withdrawn earlier for
breaking a mate canary. The later corpus audit showed the canary failure came
from a dataset with no mating material, not from the underlying parameter
surface. This licenses re-opening a withdrawn candidate only when the original
blocking cause is mechanically invalidated; it does not license retrying a game
rejection.

## What did not work

| Evidence | Result | Direction for Rarog |
|---|---:|---|
| Sixteen new HCE terms plus a distilled fit | **-77.92 +/- 15.32 Elo**, rejected | Feature-list completion plus fitting is not maturity; establish data semantics and existing-surface calibration first |
| Same 348-parameter scalar fit, own 8k outcomes | **-2.85 +/- 3.11 Elo**, H0 | Correct labels removed catastrophic harm but found no remaining value on that saturated sub-surface |
| Same fit, own 25k outcomes | **+1.00 +/- 2.11 Elo**, stopped unresolved; LTC **+0.29 +/- 5.46** | Stronger own labels did not reveal a hidden depth gain; do not iterate data without a changed representation/policy hypothesis |
| Same fit, Stockfish 8k outcomes | **-7.30 +/- 4.76 Elo**, worst arm | A stronger engine is not automatically a better label source; evaluation should model value realizable by its consuming search |
| Added-term fit after PSTs were free | Removing the terms slightly improved validation; 12/20 values became zero | The earlier “PSTs hid their signal” hypothesis was fairly tested and refuted; covariance hypotheses must be closed when the complete fit disagrees |
| Predicted NPS recovery from term removal | Did not appear; one PGO pair showed -6.70% NPS | Changed node composition and build variance confound raw NPS; pool PGO builds and do not use speed as the simplification rationale unless measured cleanly |

Holdout magnitude was actively misleading: a **-6.2%** validation change lost
77.92 Elo, while **-0.43%** won 9.52 Elo. Validation still selects checkpoints
within one registered fit; its magnitude does not rank candidates or forecast
playing strength.

## Data and harness failures worth importing

| Failure | Basilisk finding | Rarog rule |
|---|---|---|
| Wrong target semantics | A corpus believed to contain game results actually held 427-valued Stockfish evaluations | Self-play-WDL audit requires exactly `0`, `0.5`, `1`; other target domains need separate manifests and registrations |
| Duplicate games | 300k rounds over a small deterministic book produced **93.3% duplicates** | Rounds cannot exceed independent starts; audit exact start/game duplication before extraction |
| Missing mating material | Adjudicated/early-ended data made king safety free to destroy mating behavior | HCE datagen defaults to no adjudication and reports natural mate/decisive and exact-material coverage |
| Partial parameter reporting | A “348 of 1,190” fit was discussed as though it represented the HCE; PSTs had been frozen since an older phase | Enumerate every registry slot and fitting owner; historical stage boundaries have no authority |
| Silent bake failure | `bake.py` exited nonzero, but PowerShell continued and reported success | Check every native `$LASTEXITCODE`; smoke requires source and benchmark movement |
| Ignored match option | `-Games` was accepted in a mode that could not honor it | A user-facing option that cannot be honored must fail before launch |

Rarog already prevents opening reuse in `datagen.ps1`, refuses output
overwrite/append, uses three-way stable-start splits and validates hash-bound
assets. Step 4.7 adds the remaining content and all-slot instrument audits.

## Search-analysis lessons

Basilisk also reproduced Rarog's broader experience: a large search deficit did
not make Stockfish-shaped selectivity changes transferable.

| Finding | Consequence |
|---|---|
| Reduction magnitude sweeps were flat/worse; check-depth bundle made the tree 28% smaller and lost 3.48 +/- 3.32 Elo | Tree shrinkage is not strength; do not retry constants without new decision information |
| History pruning was nearly unreachable but activating it did not improve fixed-node depth | A dead mechanism is an anomaly, not automatically headroom; measure overlap with earlier consumers |
| Deep branching matched the reference although cumulative nodes stayed about 1.9x; the apparent shallow target vanished when cumulative totals were differenced | Record per-iteration costs and measure through playing depth; a cumulative ratio cannot localize cost |
| Mean depth gap was 12.07 plies but median was 4.00 because forced-mate outliers reached depth 100–245 | Use paired/per-position medians and report outliers; do not build on aggregate means alone |
| After the accepted HCE refit, qsearch share moved **30.8% -> 35.1%**, while ordering/LMR statistics mostly held | Search diagnostics can become selectively stale after evaluation changes; rerun, do not assume which ones moved |
| `bench` ignored UCI parameter overrides, while real `go nodes` honored them | Prove the exact measurement path consumes the option with an absurd value before sweeping |
| Latest `singularQuietLMR` sweep favored 401 while the reference's 1024 was much worse | Understand actual consumer semantics and fit locally; the exact value and pending candidate are not Rarog evidence |
| Check/singular exclusivity gained 0.065 average ply but failed Basilisk's registered fixed-depth WAC floor, 137 -> 124 | The metrics disagree. Rarog must add equal-node WAC before interpreting this method because extension changes alter work per nominal depth |
| The three-ply stack was only 0.123% of interior nodes; tightening the surprisingly common double extension gained no depth | Measure the suspected population before repairing it; odd-looking frequency is not proof of cost or headroom |
| `double_ext_max` never bound even when reduced from 200 to 16 | A dead safeguard is not a candidate; remove or redesign it only under its owning cleanup step |

Basilisk's 5.7.2 `singularQuietLMR` candidate still has **no accepted game
verdict**. BAS-D12 was stopped by decision at 24,956 games at +1.49 +/- 2.77
Elo, LLR +0.51; it is only carried provisionally into Basilisk's integrated
gate. Its useful output is the method: inventory the full extension/reduction
contract, discover that the flag affects later siblings rather than the
singular move, prove zero-value identity, sweep through a live UCI search path,
and treat the reference constant as a deliberately tested bad prior.

The exact 5.7.3 defect does not transfer. Current Rarog has no unconditional
node-level in-check extension—it gained +30.75 Elo by removing it—and therefore
cannot form Basilisk's check plus singular-double three-ply stack. Rarog has no
equivalent `double_ext_max` cap either. Basilisk's zero-game rejection itself is
also not a Rarog verdict: Rarog's winning removal initially lowered fixed-depth
WAC from 147 to 133 because it searched about 40% fewer nodes at nominal depth
6, but improved equal-node WAC from 185 to 203. The portable finding is the
two-axis screen: if post-HCE 4.11 ever reopens extension authority, measure
average depth and forcing-line retention at equal cost rather than allowing
either fixed-depth aggregate to stand alone. Only a genuine correctness canary
is an unconditional zero-game veto.

## Rarog avenues and plan incorporation

| Rarog step | Incorporated Basilisk direction |
|---|---|
| **4.7** | Verify WDL label domain, independent starts, mate/material coverage, complete parameter-to-instrument accounting and full pipeline smoke |
| **4.8** | Refit every identifiable existing degree of freedom, including PSTs and old sparse/staged families; fit nonlinear king danger and scaling with native instruments; one PGO game gate |
| **4.9** | Add structure only for residual signals the complete existing surface cannot express; refit all covariant old parameters with it |
| **4.10** | Rerun the complete instrument schedule after any accepted structure |
| **4.11** | Rebuild qsearch/TT/score-scale and branching evidence after HCE; use per-position/per-iteration profiles and live UCI sweeps; extension work additionally requires fixed-depth and equal-node tactical screens |
| **4.12** | Search SPSA remains conditional on a displaced interacting surface; no reference constants or pilot theta |

## Non-portable details

Do not import Basilisk's coefficients, 401 setting, parameter groups, corpus
node counts, iteration counts, fit order, +12 Elo expectation, branching ratios
or family ranking. Rarog has a different evaluator, traces, search consumers,
score scale and accepted history. The audit changes questions and instruments;
only Rarog measurements can answer them.
