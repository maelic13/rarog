# SPSA surfaces

This directory holds the SPSA surface of a **registered** tune, one
`config_<group>.json` per group, read by `tools/spsa.ps1` and checked by
`tools/audit_spsa_coverage.ps1`. An optional `fixed_<group>.json` holds UCI
options applied to both sides at fixed values. **One group is registered:
`b23core`** (PLAN B.2.3, RAR-S75).

## `b23core` — B.2.3, the selectivity core

- **Arm:** `b2core` at 4,706,910 / EBF 2.391 (engine `308abe9`). Build the
  binary with `./tools/build_test.ps1 -Suffix b23core -Tune -Features b2core`
  (flavor `b2core-tune`). `spsa.ps1` refuses any other tune build for a
  group that names `Core*` options.
- **`config_b23core.json`**, 82 coordinates: every `CoreParams` spin except
  the five categorical switches (`CoreRazorGuards`, `CoreCorrTrainDecisive`,
  `CoreCorrTrainExcluded`, `CoreLmrFullDepth`, `CoreLmrCheckRoot`, gated by
  games in RAR-S74 and never coordinates), `CoreIirMinDepth` (a discrete
  depth threshold owned by B.3) and `CoreEvalRule50Damping` (too few games
  reach a high clock at `3+0.03` to carry a gradient). The four LMR clamp
  bounds are included.
  - `value` is the engine default and `min_value`/`max_value` the declared
    range.
  - `step` is `max(2, round((max - min) / 16))`, rounded half away from zero.
  - Every step keeps `step * c_t(5000) >= 0.5` at the registered horizon; the
    smallest step is 3, giving 1.26.
- **`fixed_b23core.json`:** Hash 64, Threads 1, MultiPV 1 and the five
  switches at their defaults (fixed because they were gated, not pinned).
- **Schedule and horizon** are registered in RAR-S75: 32 games per
  iteration, `r_end` 0.0031, N = 5,000 run in resumable sessions (staged
  reviews at `-StopAfter 1250` and `2500`). The estimator is the final
  theta, rounded.

B.1 (2026-09-14) deleted the twelve historical groups (`aspiration`, `corr`,
`futility`, `histcov`, `history`, `lazymargin`, `lmr`, `probcut`, `pruning`,
`see`, `selectivity`, `tm`). They fitted the pre-B.2 selectivity core, which
B.2 replaces with donor-shaped terms under new names, and several named
parameters B.1 removed as inert. Their fitted values are baked into
`src/search/params.rs`; the files remain in Git history. SPSA stays
conditional on activation and curvature evidence (PLAN rule 4).

## Durable rules

1. Establish a strength-bearing mechanism and a positive prior before tuning
   its consumers. A clean schedule is necessary, not a go decision.
2. Keep categorical switches out of SPSA. Gate them independently, then tune
   continuous consumers under the accepted architecture.
3. Register the coordinate list, horizon, gain, fixed options and final-theta
   estimator before launch. Do not select a flattering checkpoint afterward.
4. SPSA produces a candidate. Bake it into a fresh PGO binary and accept it only
   after a paired SPRT against the pre-tune baseline.
5. Integer parameters need `step * c_t(N) >= 0.5`. With `gamma=0.102`,
   `c_t(5000) = 0.42` and `c_t(10000) = 0.39`, so either horizon needs
   `step >= 2`. A step-1 coordinate becomes unobservable around iteration 894
   while it keeps random-walking.
6. More coordinates are not free. Include parameters only when the mechanism,
   activation population and interaction justify the extra gradient noise.
7. Stop/resume correctness preserves an experiment; it does not justify
   running one. State is transactional, logs append, and the schedule is fixed
   at first launch.

Run the mechanical audit before registering a tune; `src/search/params.rs` is
the source of truth for defaults:

```powershell
./tools/audit_spsa_coverage.ps1
```

## Opening book and match conditions

The harness uses `tools/books/UHO_Lichess_4852_v1.epd` at `3+0.03`, Hash 64,
one engine thread and 32 games per iteration, the same book and time control as
the default SPRT path, so the tuner is less likely to learn an opening or
time-control artifact that disappears in confirmation.

Once PLAN names the experiment, its surface and gates:

```powershell
./tools/build_test.ps1 -Suffix <name> -Tune            # add -Features b2core for a Core* surface
./tools/spsa.ps1 -ConfigGroup <group> -EngineSuffix <name> -SetupOnly -Iterations <registered-N>
./tools/spsa.ps1 -ConfigGroup <group> -LaunchOnly -Iterations <registered-N>
```
