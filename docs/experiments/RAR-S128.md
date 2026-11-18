# RAR-S128 — D.7.6: the compact kernel's fit, one rule 7c run over its whole surface — REGISTERED 2026-11-18, before any reading

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-18, before the tune build exists and
  before any reading. The activation read and the curvature sweep are the
  agent's; the tune is the maintainer's to start (D.7 decision 6).
- **Kind:** PLAN D.7.6, the fit of one dependency-complete cluster (rule 3):
  every coordinate of the compact kernel in one joint run, so the parts are
  fitted together. No gate of fitted over unfitted (maintainer decision
  2026-11-16); the one gate is D.7.7's.
- **Inherits:** D.7.4.1's fold (`analysis/d741_fold_2026-11-18.md`, seeds
  right to rounding in Rarog's units), RAR-S126 (unfitted −11.1 ± 15.2 nElo
  against the classic kernel) and RAR-S127 (no port defect in won endings).
- **Why a fit:** the seeds are the donor's constants, fitted on Reckless's
  own evaluation, here searching Rarog's. Phase B's core read +65.1 unfitted
  and +138.6 fitted (RAR-S73, rule 7b): donor constants on Rarog's
  evaluation understated their mechanism by more than its unfitted margin.

## Prerequisite: D.7.6.1, the seeds exposed to a tune build

The compact kernel's seeds are a compile-time constant (`rules.rs`,
`SEEDS = &Seeds::DONOR`), and no build can change them. D.7.6.1 (`I1`)
makes them settable in the `tune` build only:

- each candidate of the surface a UCI option named as in
  `analysis/d76_surface_2026-11-18.md` (`K` and the seed's name in Pascal
  case; an array entry takes its index), with the seed as its default and
  the registered range as its bounds;
- the search reads its seeds by reference from its own configuration, set
  once at `setoption`, so a node takes no lock, no `Arc` and no copy;
- the default and `compact-kernel` builds are unchanged: classic `bench 13`
  9,067,595 and compact 7,543,485 reproduced, and the tune build at its
  defaults reproduces 7,543,485;
- the parity test keeps reading `Seeds::DONOR`;
- the tune binary's `uci` output lists exactly the surface's options,
  checked against the record by `tools/audit_spsa_coverage.ps1`.

## Surface (frozen)

`analysis/d76_surface_2026-11-18.md`: 246 numeric seeds, 5 excluded by
construction (each with its reason), **241 candidates**, each with a range
by one rule (`[s/2, 2s]`, or `s ± 4` for `|s| < 8`, never changing sign) and
a step of range/16. `pick_late_good` has one step of room and keeps one.
Fixed, not in the surface: the evaluation (frozen), time management and
everything else D.7 carries over unchanged, and the classic kernel's
parameters, which the compact kernel does not read.

**Activation read (frozen rule, run on the D.7.6.1 build before the
sweep):** for each candidate, `bench 13` at the seed plus and minus its
perturbation (two steps, or the room it has). A candidate whose two readings
both equal 7,543,485 changes no decision on the bench and is dropped, named
in the read; every other candidate is the surface. The read also checks
each option fires (its value reaches the rule), so a name wired to nothing
cannot pass as inactive or as live. The surface's final count and the
config files (`tools/spsa_configs/config_d76fit.json`,
`colosseum/d76fit.*.toml` from `spsa_config_to_colosseum.py d76fit --rule7c
--ceiling 6000`) are committed with the read, before any game.

### Activation read (2026-11-18, before any game)

On `rarog-d76fit-tune.exe` (`f7c89fb`, `tune compact-kernel`, SHA-256
`a536482f09f8214367d92ef076d9afbdd19263a2a8d34735ee839829f83688b7`), by `analysis/artifacts/d76-fit-2026-11-18/activation.py`
(`activation.json`). Guards passed: the binary's 241 option lines equal the
surface record one for one; every `setoption` accepted; the seed's `bench
13` 7,543,485; the positive control (`KLmrLog` at its plus
side) 6,433,138.

- **Active: 231 of 241**, 0 of them on one side only.
- **Inactive, dropped (10):** `KFdsSingular`, `KFdsSingularCap`, `KFdsSingularOffset`, `KFdsTtPvDeep`, `KNmpMin16`, `KPickCheckBase`, `KPickCheckPiece`, `KRfpMin16`, `KTtCutFifty`, `KWindowStabilityCap`.
- **`KPickLateGood` dropped** (amendment before any game): one step of room
  (2 in `[1, 6]`), so its end perturbation, 0.41, rounds to zero from
  iteration 894 of 6,000 (`audit_spsa_coverage.ps1`, class 6); the range
  cannot widen without an engine change. It stays at its seed.
- The surface's files: `tools/spsa_configs/config_d76fit.json` (230
  coordinates, seeds, ranges and steps as the record), `fixed_d76fit.json`
  (Hash 64, Threads 1, MultiPV 1), and `colosseum/d76fit.tune.toml` and
  `d76fit.run.toml` from `spsa_config_to_colosseum.py d76fit --rule7c
  --ceiling 6000`. `tools/audit_spsa_coverage.ps1 -TuneEngine` declares
  the compact seeds from this binary's listing.

*Calibration of the activation prediction* (8, band 2 to 20):
10 inactive.

## Amendment, 2026-11-18, before any game: the pilot (PROCESS, SPSA go/no-go, step 5)

The registration omitted the sensitivity pilot that PROCESS asks of a
nontrivial surface. Added before any game, after the activation read and
before the tune: 128 iterations × 30 games (3,840 games, about 40 minutes)
on the same binary and surface, files converted for a 128-iteration ceiling
into the pilot's own directory, `colosseum.ps1 -Mode spsa -Ceiling 128
-AllowShortRun -FirstCheckpoint 128 -Window 128`, seed 20261119,
`tools/results/d76-pilot`. The maintainer's command.

- **Its use:** the whole run proved end to end before 30 hours: no fault,
  every option accepted by the runner, no first arm clamped. A fault or a
  refused option stops the tune until fixed.
- **Its readings are diagnostics only** (PROCESS step 5): each z and each
  centre's travel is recorded and read again after the tune; pilot theta is
  never promoted and never the tune's seed, which stays the donor's values.
- **Re-audit (step 6):** a coordinate the pilot finds rails at once is
  reported before the tune, and its range is registered again if its z
  points outward past the familywise line.

## Amendment, 2026-11-18, before any game: the line bound

The first activation read, on `89eeb41`'s tune build, timed out 106 of its
482 readings at 90 s, across 95 coordinates and both directions. Every case
read further runs away on one position, bench 16
(`1k6/1b6/8/5p2/p1p2p2/P7/1P3P2/K7 b - - 0 1`, a won pawn ending): 43,186
nodes at the seeds, about a billion and depth 10 to 12 after 210 s under a
perturbation of two steps. That is RAR-S127's class, lines carried to the
ply cap with a mate in view, and a fit over it would tune constants around
it. A line bound, a depth held to a multiple of the iteration's depth before
quiescence, was read at zero games on a diagnostic build (`bench 13`; the
six cases at a 60 s limit; the KBNK anchor `kbnk_keeps_both_minors`):

| Factor | `bench 13` | Six cases | KBNK anchor |
|---|---|---|---|
| none | 7,543,485 | none finishes | passes |
| 3 | 6,968,853 | | |
| 4 | 7,381,391 (−2.1%) | all finish | **fails** (a minor lost at 40k and 120k nodes) |
| 5 | 5,751,023 | all finish | passes |
| 6 | 7,119,115 | all finish | passes |
| 8 | **7,543,485 (unchanged)** | all finish | passes |

At eight times RAR-S127's two positions also finish: 202 at depth 14 in
120 million nodes (34 s), the KBPK position at depth 17 in 5.7 million.
Rarog's own bound of twice the depth moved the bench −36% (RAR-S127).

**Decided by the maintainer 2026-11-18, before any game:** the compact
kernel carries a line bound (proposed at four times the depth; eight after
the KBNK anchor failed at four), and the fit runs on that kernel
(`f7c89fb`). The bound binds no tree of the bench, so the fingerprint stays
**7,543,485 / EBF 2.422**; the classic kernel's 9,067,595 is unchanged.
The seeds, the surface and every rule above are unchanged; the activation
read is taken again on the bounded kernel's tune build, and the first read
stays on record in
`analysis/artifacts/d76-fit-2026-11-18/activation_unbounded.json`.

## Curvature sweep (frozen; PLAN rule 7c, *Curvature sweep*)

Run on the D.7.6.1 build after the activation read, at zero games.
Evidence path `analysis/d76_sweep_<date>.md`, raw results under
`tools/results/d76-sweep-<date>/`.

- **Coordinates (five, one a mechanism):** `lmr_log` (every late-move
  reduction), `rfp_square_16` (reverse futility), `nmp_r_base` (the null
  move's reduction), `fp_linear_16` (move-loop futility) and
  `singular_per_ply_1024` (the singular margin).
- **Points:** 0.5×, 0.75×, 1×, 1.5× and 2× the seed, rounded to the
  coordinate's unit; at each, `bench 13` and WAC solved at 100,000 nodes
  (`tools/diag/fixed_budget_probe.py nodes 100000 src/wac.epd`).
- **Floor:** WAC at the seed and at one unit either side on each of the five
  (eleven readings); the floor is their range, read before any sweep point.
- **Classification on WAC, per coordinate:** *flat* when the five points'
  range is within the floor; otherwise *curved* when the maximum occurs only
  at interior points; otherwise *monotone*. `bench 13` is reported and does
  not classify.
- **Rule:** flat or monotone on all five skips the SPSA, and D.7.6 is
  reported to the maintainer with the sweep (rule 4); any curved coordinate
  opens the run.

### Sweep read (2026-11-18, zero games; `analysis/d76_sweep_2026-11-18.md`)

Floor 5 (WAC range over 11 readings). `KLmrLog`, `KNmpRBase` and
`KSingularPerPly1024` curved; `KRfpSquare16` and `KFpLinear16` monotone, best
at a range end. **A curved coordinate opens the run.** *Calibration:* three
curved, as predicted (at least three, 0.6).

## Run (frozen)

- **Rule 7c's shape:** Colosseum `spsa`, 30 games an iteration on 15 slots,
  `3+0.03`, Hash 64, one thread, the UHO book, no adjudication; the
  perturbation two steps; `r_end` 0.00155; ceiling **6,000 iterations
  (180,000 games)**; checkpoints at 2,000 and every 1,000, read by
  `tools/spsa_read.py`. About 10 hours to the first checkpoint and 30 to the
  ceiling at RAR-S126's rate (5,734 games an hour on 14 slots).
- **Binary:** `./tools/build_test.ps1 -Suffix d76fit -Tune -Features
  compact-kernel` from a clean worktree at D.7.6.1's commit; its hash and
  `uci` output recorded before the run.
- **Seed** 20261118, directory `tools/results/d76-fit`.
- **The test at each checkpoint:** each coordinate's z over the last 1,000
  iterations; live when any |z| passes the familywise 5% line for the
  surface's count (about 3.7 at 241) or `Σ z² − n ≥ 2·√(2n)`; otherwise
  quiet. Quiet stops the tune and theta is that checkpoint's rounded
  centres; live runs 1,000 more. A centre within one perturbation of a bound
  with its z outward past the familywise line stops for a new registration
  of that range. Faults stop for the maintainer.
- **Each read records** every z, the tune's own gain read, the stop's floor
  in Elo, and a frozen forecast for the next 2,000 iterations.
- **Use:** theta goes to D.7.7's gate against the head, registered there.
  This run accepts nothing.

**Watch item.** The surface is about three times Rarog's largest tune (82
coordinates, RAR-S104) and five times Basilisk's record (49, floor 4.5 Elo
at 2,000). A quiet stop rules out only what its floor states; a floor well
above 5 Elo is evidence for a new registration of the shape, never a reason
to extend a stopped tune.

## PRE-REGISTERED PREDICTION (frozen 2026-11-18, before any reading)

- **Activation:** 8 candidates dropped as inactive (80% band 2 to 20); the
  floors (`rfp_min_16`, `nmp_min_16`) and the sparse PV-only extension bars
  most likely among them.
- **Sweep:** at least three of the five curved (0.6); all five flat or
  monotone, so the SPSA skipped (0.1).
- **Tune:** live at 2,000 (0.8); stops quiet between 3,000 and 5,000 (0.5)
  or at the ceiling (0.3); a rail stop for a range (0.3).
- **The tune's own gain read at its stop:** **+35 nElo**, 80% band **+10 to
  +80**. Basis: RAR-S73's fit gained more than its unfitted margin, and the
  donor's constants were fitted to another evaluation; against that, Rarog's
  later fits of an already fitted surface read +1 to +2 (RAR-S104), and a
  surface this large spreads its signal thin.
- **Floor at 2,000:** about **10 Elo** (80% band 6 to 16): Basilisk's 4.5 at
  49 coordinates scaled by the square root of the surface's size.
- **Most likely failure mode:** a quiet stop whose floor is too high to rule
  out a real gain, which this rule then answers by a new registration of the
  shape.
- **Evidence layer:** the tune's own read is not a gate; D.7.7 accepts or
  rejects.
