# SPSA tuning with weather-factory and fastchess

This directory contains reusable and historical tuning surfaces. **There is no
active SPSA, and none is scheduled.**

Two separate reasons, often confused:

1. The 30-coordinate, 10,000-iteration run proposed by the **closed** Phase-4
   line was canceled before its first game during the 2.3.2 review. Do not
   reconstruct or launch it from old notes; its config files were removed.
2. The **current** Phase 4 (`PLAN.md` §4) is a different programme that reuses
   the number. It explicitly forbids a broad SPSA: it gates categorical
   architecture first, and permits only a narrow fit at 4.17 over constants
   whose structural owner has already passed a game gate. The next broad fit is
   7.3, after NNUE scale freezes.

## Current decision

SPSA is a local optimizer. It can improve constants around a mechanism that
already has evidence, but it cannot manufacture the architectural gain needed
to close a large engine gap. The canceled run had a technically valid schedule
but no credible large-strength prior after its three architecture switches
failed or remained unproven. Spending roughly 320,000 games / 79 hours on that
surface was therefore poor expected value.

The next broad tune is owned by the post-NNUE search fit. It must be registered
fresh after the NNUE baseline passes its own gates, because NNUE changes eval
scale, correction residuals, pruning margins and search/eval cost. No current
JSON file is automatically the right post-NNUE surface.

## Durable rules

1. Establish a strength-bearing mechanism and a positive prior before tuning
   its consumers. A clean schedule is necessary, not a go decision.
2. Keep categorical switches out of SPSA. Gate them independently, then tune
   continuous consumers under the accepted architecture.
3. Register the coordinate list, horizon, gain, fixed options and final-theta
   estimator before launch. Do not select a flattering checkpoint afterward.
4. SPSA produces a candidate. Bake it into a fresh PGO binary and accept it only
   after paired SPRT against the pre-tune baseline; use LTC when transfer is in
   doubt.
5. Integer parameters need `step * c_t(N) >= 0.5`. With the current
   `gamma=0.102` and N=5,000, that means `step >= 2`; a step-1 coordinate becomes
   unobservable around iteration 894 while continuing to random-walk.
6. More coordinates are not free. Include parameters only when the mechanism,
   activation population and interaction justify the extra gradient noise.
7. Stop/resume correctness preserves an experiment; it does not justify
   running one. State is transactional, logs append, and the schedule is fixed
   at first launch.

Run the mechanical audit before registering any future tune:

```powershell
./tools/audit_spsa_coverage.ps1
```

`src/params.rs` is the source of truth for current defaults. The audit rejects
stale names, seed drift, invalid or categorical ranges, and perturbations that
round to zero at the standard 5,000-iteration horizon.

## Opening book and match conditions

The harness uses `tools/books/UHO_Lichess_4852_v1.epd` at `3+0.03`, Hash 64,
one engine thread and 32 games per iteration. This is the same UHO source and
short time control used by the default SPRT path. That alignment is deliberate:
it reduces the chance that the tuner learns an opening or time-control artifact
which disappears in confirmation. The EPD is paired/reversed by the match
runner; it is suitable for SPSA as configured.

Do not replace the book merely to create novelty between tuning and SPRT. Use a
separate robustness book or LTC as an additional confirmation when the tested
mechanism is opening- or time-control-sensitive.

## Reusable historical groups

| Group | Purpose | Status |
|---|---|---|
| `selectivity`, `pruning`, `corr`, `see` | Search/selectivity fits | Historical input; regenerate after NNUE |
| `lmr`, `histcov`, `history` | Reduction/history fits | Historical input; accepted defaults remain in source |
| `probcut`, `futility`, `aspiration` | Focused search fits | Historical/retry-only with new evidence |
| `tm`, `lazymargin` | Time/eval-speed fits | Condition-sensitive; never bulk-merge blindly |

The generic harness remains available for a newly approved experiment:

```powershell
./tools/build_test.ps1 -Suffix <name> -Tune
./tools/spsa.ps1 -ConfigGroup <group> -EngineSuffix <name> -SetupOnly
./tools/spsa.ps1 -ConfigGroup <group> -LaunchOnly -Iterations <registered-N>
```

Do not use those commands until PLAN.md names the experiment, its evidence,
surface and acceptance gates.

## Why removed knobs are gone

The 2.3.2 cleanup removed ten alternatives that had no remaining owner and
hardwired their accepted defaults: correction capture training stays enabled;
TT eval refinement keeps its zero-depth floor; reverse-futility uses the
not-improving direction; histories age between searches; evasions are not LMR
reduced; ProbCut stores at `depth - 3`; qsearch admits depth-0 TT refinement;
double singular extension remains available through its graded margin; and the
degenerate root-gap signal is diagnostic-only, not part of confidence.

This is behavior-preserving cleanup, not feature loss. Reintroducing any
alternative requires new evidence and a new owner rather than a dormant UCI
option.

Other experimental options remain intentionally inert because they do have a
named later decision point:

- post-NNUE search fit: aspiration shape, TT provenance/NMP/singular variants,
  prospective selectivity, correction provenance/weights and root-confidence
  consumers;
- multi-thread phase: pooled root instability and helper iteration skipping.

Keeping those defaults off preserves current strength. Their later owner must
either pass a gate and activate them, or remove them in that phase.
