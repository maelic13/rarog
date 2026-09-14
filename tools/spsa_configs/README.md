# SPSA surfaces

This directory holds the SPSA surface of a **registered** tune, one
`config_<group>.json` per group, read by `tools/spsa.ps1` and checked by
`tools/audit_spsa_coverage.ps1`. **It is empty of surfaces, and no tune is
registered.**

B.1 (2026-09-14) deleted the twelve historical groups (`aspiration`, `corr`,
`futility`, `histcov`, `history`, `lazymargin`, `lmr`, `probcut`, `pruning`,
`see`, `selectivity`, `tm`). They fitted the pre-B.2 selectivity core, which
B.2 replaces with donor-shaped terms under new names, and several named
parameters B.1 removed as inert. Their fitted values are baked into
`src/search/params.rs`; the files remain in Git history. The next surface is
B.2.3's, registered fresh in `EXPERIMENTS.md` over the coordinates B.0 section 9
names, and SPSA stays conditional on activation and curvature evidence (PLAN
rule 4).

## Durable rules

1. Establish a strength-bearing mechanism and a positive prior before tuning
   its consumers. A clean schedule is necessary, not a go decision.
2. Keep categorical switches out of SPSA. Gate them independently, then tune
   continuous consumers under the accepted architecture.
3. Register the coordinate list, horizon, gain, fixed options and final-theta
   estimator before launch. Do not select a flattering checkpoint afterward.
4. SPSA produces a candidate. Bake it into a fresh PGO binary and accept it only
   after a paired SPRT against the pre-tune baseline.
5. Integer parameters need `step * c_t(N) >= 0.5`. With `gamma=0.102` and
   N=5,000 that means `step >= 2`; a step-1 coordinate becomes unobservable
   around iteration 894 while continuing to random-walk.
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
./tools/build_test.ps1 -Suffix <name> -Tune
./tools/spsa.ps1 -ConfigGroup <group> -EngineSuffix <name> -SetupOnly
./tools/spsa.ps1 -ConfigGroup <group> -LaunchOnly -Iterations <registered-N>
```
