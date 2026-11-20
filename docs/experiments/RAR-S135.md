# RAR-S135 — D.7.11: the finishing fit, one rule 7c run over 225 coordinates of the repaired kernel — REGISTERED 2026-11-20, before any game

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-20, after the activation read and
  before any game, the pilot's included. The pilot and the tune are the
  maintainer's to run (AGENTS, *Expensive jobs*); the reads' preparation is
  the agent's.
- **Kind:** PLAN D.7.11, the finishing fit of one dependency-complete
  cluster: the compact kernel's whole active surface in one joint run from
  its baked centres. Research and decisions:
  `analysis/d711_fit_2026-11-20.md` (the record), frozen; this registration
  states them and adds the binary, the files, the seeds and the drivers.
- **Inherits:** RAR-S128 (the first fit, stopped live at 2,000 and baked,
  `73966eb`), D.7.8's repairs (ProbCut's bar at the donor's scale, the
  quiescence and evaluation-only table depth, the root's own order), D.7.9.1
  (Reckless's time manager, RAR-R34) and RAR-S134 (the repaired kernel
  +63.8 ± 9.1 Elo against the classic at `3+0.03`, the fit's control).
- **Hypothesis:** the kernel's constants were fitted once, on a kernel
  since repaired (ProbCut's family against a bar 0.40 of the donor's) and on
  a surface missing seven degrees of freedom; a second run from the baked
  centres on the repaired kernel and the completed surface gains, and does
  not lose beyond about −15 Elo shown by the read at its stop.

## Binary

`tools/test_engines/rarog-d711fit-tune.exe`, built by
`./tools/build_test.ps1 -Suffix d711fit -Tune` from the clean tree at
`3233da2` (engine source `6843b16`, D.7.11.1), `cargo xtask build --tier pext
--nopgo --features tune`, `rustc 1.100.0`; `id name Rarog
dev-20261120-3233da24`; SHA-256
`e88123b26b649212703f2b04f65ea3b494c4fbb2ba8782a8b88200795bddfc25`; its
sidecar `rarog-d711fit-tune.json` records `bench 13` **3,320,355** (EBF
2.317, the development fingerprint, reproduced by the default build too). It
lists 249 seed options (254 seed fields less the five excluded by
construction, `tune.rs`). Every arm of the pilot, the tune and the reads is
this binary.

## Surface (frozen)

**225 coordinates**, the record's surface (`analysis/d711_fit_2026-11-20.md`,
section 2 and its appendix), in `tools/spsa_configs/config_d711fit.json`
(`66a1f14`): each coordinate's `value` is the engine's default (the first
fit's baked centres), `min_value` and `max_value` the binary's listed range
(half to twice the seed, `KProbcutSeeScale` its declared 512 to 4,096),
`step` range/16 and at least 1. `d711pilot` is the same file.
`tools/spsa_configs/colosseum/d711fit.*.toml` and `d711pilot.*.toml` are the
rule 7c conversions at ceilings 6,000 and 64; the room report is empty at
both (the least room at a seed is 5.0 steps, so every coordinate plays the
full two steps).

- **In for the first time (ten):** the seven new options of D.7.11.1
  (`KPawnHistoryMax`, `KSingularPvPerPly1024`, `KQSeeCorrectionScale`,
  `KExchangeValue1` to `KExchangeValue4`), `KProbcutSeeScale` and
  `KFdsTtPvDeep`; and `KPickCheckPiece`, inactive on `bench 13` and active on
  `bench 17`. The ProbCut family is refitted against its repaired bar.
- **Out, whole units (16):** `KSingularDepth`, `KSingularTtDepth`,
  `KSingularExactDiv`, `KProbcutDepthBase`, `KTtCutTypedDepth`,
  `KTtBonusParentMoves`, `KEvalBonusDepth`, `KHpDepth`, `KLdseDepth`,
  `KFpDepth`, `KBnfpDepth`, `KNmpVerifyDepth`, `KQMoveLimit`,
  `KNegativeExtension`, `KPickLateGood`, `KSeeNoisySquare`. A clean readout
  is chosen over a possible Elo or two on the next unit of three of them
  (the record, *Whole units*).
- **Out, inactive at both bench depths (8):** `KFdsSingular`,
  `KFdsSingularCap`, `KFdsSingularOffset`, `KNmpMin16`, `KRfpMin16`,
  `KPickCheckBase`, `KTtCutFifty`, `KWindowStabilityCap`.
- **Not options, by construction (5, `tune.rs`):** the two thread spreads,
  `pick_escape[0]` and `[5]`, and the pawn's exchange value, the unit of the
  other four. The quiet, noisy and continuation history limits are literals,
  units of their families.

### Activation read (2026-11-20, before any game)

On the binary above, by `analysis/artifacts/d711-fit-2026-11-20/activation3.py`
(SHA-256 `8e85cb9a…`; output `activation3.json`, `8f6c7d78…`): RAR-S128's rule
over all 249 options, `bench 13` at the seed minus and plus its
perturbation, a seed that reproduces it on both sides read again at
`bench 17`. Guards passed: 249 seed options listed; default `bench 13`
3,320,355 and `bench 17` 20,004,610; the positive control (`KLmrLog` at its
plus side) 3,118,411; every `setoption` accepted; no timeout.

- Active on `bench 13`: 240 of 249; three of them on one side only
  (`KFdsTtPvDeep`, `KResearchShallower16`, `KTripleNewPv16`), which is
  active by the rule.
- Inactive on `bench 13`, read at `bench 17` (9): `KPickCheckPiece` moves it
  (30,325,641 and 19,953,020); the eight above move neither depth.
- The seven new options, at the seed minus and plus their perturbation on
  `bench 13`: `KPawnHistoryMax` 3,774,299 and 3,379,579;
  `KSingularPvPerPly1024` 3,553,459 and 3,286,013; `KQSeeCorrectionScale`
  3,216,884 and 2,775,957; `KExchangeValue1` to `4` 4,508,482 / 3,858,410,
  2,890,805 / 3,423,705, 3,629,362 / 3,731,643 and 5,472,367 / 3,296,708.
- The surface files were generated from the binary's own listing and this
  read by `gen_config.py` (SHA-256 `1c77cb1d…`), which compares the result
  with the record's appendix and fails on any difference: none, 225 of 225
  with the same seeds and ranges. `./tools/audit_spsa_coverage.ps1
  -TuneEngine tools/test_engines/rarog-d711fit-tune.exe` ends `RESULT:
  clean`; `spsa_config_to_colosseum.py --check` passes on both groups.

## Fixed, not in the surface

- `fixed_d711fit.json` and `fixed_d711pilot.json`: Hash 64, Threads 1,
  MultiPV 1.
- The clock: the fifteen soft-limit coordinates (`TmNodesBase` to
  `TmChangesScale`) at the donor's values, the budget's constants and the
  fail-high stop's multiple (the record, section 1; its retry trigger is
  PLAN D.7.11, item 1).
- `LazyMargin` (475), the frozen evaluation's own constant (rule 2);
  `AspMaxFails` and `TmFailHighTarget`, guards; the evaluation.
- Nothing is removed from the kernel: the eight inactive seeds are D.8.21's
  question, after D.7.14.

## Shape (rule 7c, kept whole)

- Colosseum CLI 0.5.0 (the pin), `spsa`, **30 games an iteration on 15
  slots**, `3+0.03`, the UHO book, no adjudication; the perturbation two
  steps, graded by room (`tools/spsa_config_to_colosseum.py --rule7c`);
  `r_end` 0.00155; **ceiling 6,000 iterations (180,000 games)**;
  checkpoints at **2,000 and every 1,000**, window 1,000, read by
  `tools/spsa_read.py`.
- **The test at each checkpoint:** each coordinate's z over the last 1,000
  iterations; live when any |z| passes the familywise 5% line for 225
  coordinates or `Σ z² − n ≥ 2·√(2n)`; otherwise quiet.
- **The stop:** quiet (theta that checkpoint's rounded centres), a rail (a
  centre within one perturbation of a bound with its z outward past the
  familywise line: a new registration of that range), a fault (below), the
  ceiling, or the maintainer's `STOP` in the run directory at a checkpoint.
  A stopped tune is never extended.
- **Hours, at the record's rate** (23.16 s an iteration measured on
  RAR-S128, scaled by the new clock's 0.896 of the budget against 0.851, so
  about 24.3 s): **13.5 hours to the first checkpoint, 6.8 for each further
  1,000 iterations, 40.5 to the ceiling.** The pilot measures the rate; the
  hours are restated from it in an amendment before the tune's first game.
- Pause and resume: one Ctrl+C in the tune's terminal pauses it at a whole
  iteration (games in flight get 30 seconds, the iteration in flight is
  replayed); the same command resumes it at its seed, schedule and
  checkpoints (PROCESS, *SPSA go/no-go*, item 10). The tune never pauses
  for a read.

## The fault rule

- **No fixed fault limit:** no `--max-time-losses`, no
  `--max-engine-faults` (PROCESS, *SPSA go/no-go*, item 11; the record,
  section 4). The runner's own allowance stays (the larger of 3 and 0.5% of
  the games so far).
- A fault reads FAULT at the next checkpoint and the tune stops there for
  the maintainer.
- *Acceptance, registered in advance:* a time loss whose `failed-games`
  table shows the losing search held longer than its slack is the host's
  and is accepted (`tools/spsa_read.py <run> --accept-faults "<reason>"`,
  then the same command); a search that overran its own deadline with no
  hold is the engine's and returns D.7 to the clock.

## The pilot (D.7.11.5)

- **64 iterations of 30 games** (1,920 games, about 26 minutes) on
  `d711pilot`, the same surface converted for a ceiling of 64
  (`colosseum.ps1 -Mode spsa -ConfigGroup d711pilot -Ceiling 64
  -AllowShortRun -FirstCheckpoint 64 -Window 64`), `tools/results/d711-pilot`.
  The maintainer interrupts it once (Ctrl+C) and runs the same command
  again.
- **Its uses:** the run proved end to end on this build, the rate, the
  spread of an iteration's score at the first perturbations, and the pause.
- **Pass:** every option accepted by the runner; no first arm clamped; no
  fault; the reader's move check on the resumed run under 1e-6; the rate
  recorded and this registration's hours restated from it before the
  tune's first game.
- **Its theta is never used**, neither promoted nor the tune's seed, which
  is the engine's defaults. PROCESS's re-audit (step 6) follows it: a
  coordinate the pilot finds at a rail at once is reported before the tune.

## Reads of the centres against the start

The tune build on both sides (`colosseum.ps1 -Mode match
-CategoricalTuneBuild`): `d711at<K>` plays the rounded centres at a
completed iteration K, taken from `spsa history` through
`tools/spsa_read.py` without writing into the run directory and checked
name for name against the surface; `d711start` plays the engine's
defaults. `3+0.03`, Hash 64, one thread, the UHO book, no adjudication. A
read never shares the host with a running tune: its driver refuses while
any `colosseum-cli` is alive.

| Read | When | Games | Seed | Rule |
|---|---|---:|---:|---|
| At the stop (D.7.11.7) | after the tune has ended, at the checkpoint the maintainer names | 2,000, about 27 minutes | 2026113503 | **theta is baked unless the 95% interval lies wholly below zero** (D.7.8's keep rule). It catches a true −15 Elo nine times in ten and a true −10 a little over half; "baked" means no harm beyond about −15 shown, not a gain |
| Interim (optional, the maintainer's) | with the tune paused | 1,000, about 14 minutes | 2026113504 | wholly below zero is reported, and the maintainer may then place `STOP`. It catches −20 at 85% |

No forecast is made from a read, two reads never rank two checkpoints, and
theta is never chosen by a read. *Not registered:* a `10+0.1` read of theta
against the start (D.7.13's companion reads the fitted kernel there).

## Seeds and directories (frozen)

| Run | Seed | Directory |
|---|---:|---|
| Pilot | 2026113501 | `tools/results/d711-pilot` |
| Tune | 2026113502 | `tools/results/d711-fit` |
| Read at the stop | 2026113503 | `tools/results/rar-s135-read<K>-2000` |
| Interim read | 2026113504 | `tools/results/rar-s135-read<K>-1000` |

None of the four occurs in any earlier registration, driver or run
directory's `resolved-config.json` (checked 2026-11-20).

## Drivers (D.7.11.4)

Three drivers in `analysis/artifacts/d711-fit-2026-11-20/` (ignored), each
run from the machine's own terminal, where the harness's live output shows.
Each checks the binary's SHA-256 before anything else and passes
`-ExpectBench 3320355 -ExpectRevision 3233da2` to `colosseum.ps1`. The pilot's
and the tune's retry their start once a minute only while the harness
refuses the host as busy, which it does before any game; any other failure
ends them. A run directory that exists keeps its recorded seed.

| Driver | SHA-256 | What it runs |
|---|---|---|
| `run_pilot.ps1` | `7f5bae34e790992fa3f12ac0499e3bb6d2d2981bc2ea3316011a70d3cf3636a3` | `colosseum.ps1 -Mode spsa -ConfigGroup d711pilot -Ceiling 64 -AllowShortRun -FirstCheckpoint 64 -Window 64`, seed 2026113501, `tools/results/d711-pilot` |
| `run_tune.ps1` | `b62eea210359cbe0976dd29d95bcc686221e9c036cacf758308482bd1c03bda5` | `colosseum.ps1 -Mode spsa -ConfigGroup d711fit -Ceiling 6000`, seed 2026113502, `tools/results/d711-fit`; no fault limit; starts or resumes |
| `run_read.ps1` | `435c6079f8a4a07460ae24f543b15b747293768ab363ef28bef7fe4a0f8cc99d` | `-At <K> -Games 2000` (the stop, seed 2026113503, only on an ended tune) or `-Games 1000` (interim, seed 2026113504); the centres from `tools/spsa_read.py` without `--record`, checked name for name against the surface; refused while any `colosseum-cli` is alive; `colosseum.ps1 -Mode match -CategoricalTuneBuild` |

*Dry runs (2026-11-20, before any game):* each driver's `-DryRun` path ran
once to the wrapper's own dry run, every guard passed and no game was
played (manifests `tools/results/colosseum_spsa_d711-pilot_20261120_174734`,
`colosseum_spsa_d711-fit_20261120_174749` and
`colosseum_match_rar-s135-dryrun-2000-2000_20261120_174903`, each
`.manifest.txt`). The pilot's and the tune's verified the surface at their
ceilings (225 coordinates at N = 64 and N = 6000), the files current and no
rail warning; the runner's resolved configuration holds seeds 2026113501
and 2026113502, 30 games an iteration in two waves of 15, `r_end` 0.00155,
and its own fault allowance (3, or 5 per mille of the games), no limit
passed. Neither created its run directory. The read's ran on RAR-S128's
stopped run (`-RunDir tools/results/d76-fit -ConfigGroup d76fit -At 2000`),
the only tune with a history before the pilot: its 230 centres equal that
run's `theta-2000.txt` line for line, its run directory was unchanged, and
the match resolved at seed 2026113503 on this binary. Its first dry run
stopped at its "ended tune" guard, which read only the last verdict; the
guard was corrected to the wrapper's own rule (a last verdict other than
CONTINUE, or that checkpoint's theta written), which RAR-S128's maintainer
stop meets, and the passing dry run and the hash above are the corrected
file's.

## PRE-REGISTERED PREDICTION (frozen 2026-11-20 in the record, before the tune build existed; copied unchanged)

1. *Activation on the tune build:* eight inactive at both depths,
   `KPickCheckPiece` active at 17, the seven new options all active, a
   surface of 225 (0.75); one option differing (0.2). *Amended twice the
   same day, before the tune build exists:* it read 223, then 228.
2. *Pilot:* no fault (0.95); 140 to 155 iterations an hour (0.8).
3. *Tune:* quiet at 2,000 (0.50); quiet at 3,000 (0.25); later or the
   ceiling (0.20); a rail (0.03). A FAULT read at some checkpoint from a
   time loss (0.35).
4. *The tune's own gain read at its stop:* +10 Elo, 80% band −8 to +30.
5. *The read at the stop:* **+8 Elo**, 80% band −3 to +20; wholly above
   zero (0.40); wholly below zero (0.03).
6. *Furthest movers:* the ProbCut family and `KProbcutSeeScale` (down),
   `KLmrQuietHistory` (up), the wrapper pair (small); the pawn table's
   limit within noise.
- Basis: the first fit's forecast at its stop, RAR-S78 and RAR-S104, the
  moved tree, seven new coordinates.
- Most likely failure: a quiet stop at 2,000 with a read inside ±9 Elo,
  which shows neither gain nor harm; theta is then baked on the keep rule
  alone.

*Calibration of item 1* (appended after the activation read, before any
game): eight inactive at both depths, `KPickCheckPiece` active at 17, the seven
new options active, a surface of 225: the predicted outcome (0.75), with no
option differing.

## The pilot's result (2026-11-20; read from its files, before any tune game)

`tools/results/d711-pilot`, one invocation (manifest
`colosseum_spsa_d711-pilot_20261120_175215`), started 16:52:16 and ended
17:17:17 UTC: 64 of 64 iterations, 1,920 games, the runner `completed` with no
invalid iteration. The wrapper's read at 64: **FAULT** (`engine_b 1,
time_losses_b 1`), so the run stopped there for the maintainer.

| Pass criterion | Read |
|---|---|
| Every option accepted | **yes**: the wrapper verified 225 coordinates at N = 64; no refused or unknown option in `run.log` or the segment log |
| No first arm clamped | **yes**: "clamped none"; arms clamped on over 1% of iterations: none |
| No fault | **no**: one time loss, analysed below |
| The move check on the resumed run under 1e-6 | **not tested**: the run was not interrupted (one manifest, no `interrupted` line, the run log one stream to 64). The reader's move check on the uninterrupted run is 8.8e-13 |
| The rate recorded | **yes**: 23.43 s an iteration, 153.7 iterations an hour (4,610 games), slots busy 82.1% (`tune_journal.py`, `34035fca…`) |

**The time loss is the host's, by the rule registered above.** Game 1,072
(iteration 35, slot 8), the minus side forfeited at ply 52. By the runner's
`failed-games` table, the search's own time was 24 ms and its last `info`
came at 24.2 ms, but `bestmove` reached the runner at 125.5 ms against a
77.1 ms deadline; 112.1 ms of that is charged as *held* (the engine's process
not running). Without the hold the move took 13.5 ms. That is "the losing
search held longer than its slack". Read beside the other slots
(`pilot_fault.py`, `fd4bd5e4…`), it was a host-wide stall: all fifteen games
in play held 66 to 670 ms in the same seconds, against a median of 0.9 ms a
side for a whole game; 110 of the run's 111 sides held over 50 ms fall in
iterations 35 and 36, between 17:05:55 and 17:06:43 UTC. By the same tool on
RAR-S128's 60,000 games, the worst burst held 24 sides on 12 slots, the
largest 178 ms; this one held 132 sides on all 15 slots, the largest
670 ms. The host's scheduled task `\Microsoft\Windows\Shell\CreateObjectTask`
ran at 17:05:41 UTC, and DCOM logged events 10016 from 17:05:56 to 17:06:08:
a coincidence in time, not a proven cause. No session of this project ran
anything in that window.

**Re-audit (PROCESS, *SPSA go/no-go*, step 6), diagnostics only:** one
coordinate over the familywise line at 64 iterations, `KQuietMalusCap`
(z +3.81 against 3.69; with 225 coordinates one over the line is a chance
reading about one time in twenty), then `KNoisyMalusSlope` (+3.20) and
`KExchangeValue1` (−2.52); `Σ z² − n` +19.9 (+0.9 SD); no rail, no rail
candidate, no clamped arm. Nothing in it argues for a change of the surface,
and the pilot's theta is not used.

### Amendment, 2026-11-20, before the tune's first game: the hours

From the pilot's measured 153.7 iterations an hour, the stall included:
**13.0 hours to the first checkpoint at 2,000, 6.5 for each further 1,000,
39.0 to the ceiling of 6,000.** These replace the record's 13.5, 6.8 and
40.5.

*Calibration of item 2:* no fault (0.95): **missed**, one time loss in 1,920
games, the host's; the miss is in the instrument's environment, not the
engine (the search itself was inside its deadline). 140 to 155 iterations an
hour (0.8): **hit**, 153.7.

## Use

Theta, if kept by the read at the stop, is baked by D.7.11.8 (the tune
build at its defaults and the default build reproducing the tune binary's
`bench 13` at theta); D.7.12 then calibrates the reported score on the
read's games or a self-play match of the baked kernel, never on the tune's
games; D.7.13's SPRT against the classic decides adoption. This run accepts
nothing by itself.
