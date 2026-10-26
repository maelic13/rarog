# RAR-R21 — D.1.6.3: the staged road to the cascade cluster's gate: three 2,000-game equal-speed reads, then SPSA block 1 over the cluster and its constants, each stage with its rule — REGISTERED 2026-10-23, before any game

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-23, before any game. Every stage is
  the maintainer's command; the agent prepared and dry-ran each.
- **Kind:** PLAN D.1.6.3, between the cluster's verification (RAR-R20) and
  its gate. The maintainer's requirement of 2026-10-23: be very sure the
  CPU time is not spent for nothing.
- **Why staged.** The cluster ends the stall (RAR-R20's probe) and reads
  −52.5 ± 23.9 Elo unfitted against the PGO head over 200 games, while at
  equal speed on one tune build every part reads within noise of even
  (RAR-R20's decompositions, ±27 to ±30). A 200-game read cannot separate
  "a −50 cluster", which no fit rescues, from "a −15 cluster that leaves
  time on the clock and runs 5% slower in games", which a fit and the
  clock's scale can. The 2,000-game reads resolve about ±9 Elo; they decide
  whether the ten-hour tune runs at all.
- **The binaries.** Stage 1 runs on `tools/test_engines/rarog-d16inst2-tune.exe`
  (`2f17467`, the `d1` head with every switch at its neutral default, bench
  13,987,223), both arms the same binary, arm A by UCI options, so speed is
  equal. Stage 2 runs on `tools/test_engines/rarog-d16fit-tune.exe`
  (`217125b` on `2f17467`, the throwaway branch `d16-fit`: the four parts
  baked as the search defaults, so the tune starts at the cluster's seeds as
  the harness requires; recipe
  `analysis/arm_patches/217125b-rar-r21-d16fit.patch`, SHA-256
  `e9fe1f76…42915c`, which reproduces the arm's tree; binary SHA-256
  `50323E4ED98C479B…`, bench 8,996,837). The runner is `run_r21.ps1` beside
  this entry; every stage dry-ran clean on 2026-10-23.

## Stage 1: three 2,000-game reads at `3+0.03`, equal speed (about 25 minutes each)

Colosseum `match`, `-CategoricalTuneBuild`, Hash 64, one thread, 14 slots,
the UHO book in random order, no adjudication, 2,000 games each:

1. **`all4`:** the four parts (`MopupKnownWin` 6000, `AspDeltaSquareDiv`
   10193, `CoreLmrDeeperBudget` 1, `TmDepthFreeze` 5000) against the
   defaults; seed 20261051, `tools/results/rar-r21-all4`.
2. **`nofreeze`:** parts 1–3 against the defaults; seed 20261052,
   `rar-r21-nofreeze`.
3. **`freeze`:** the freeze alone; seed 20261053, `rar-r21-freeze`.

**Readings:** each read's Elo with its interval from the manifest; each
arm's time a move and nodes a second from the manifest's `search:` lines
(the time rule: 2 Elo per 1% of thinking time at `3+0.03`, RAR-S98,
RAR-S99; between arms of one clock whole-game time is nearly conserved, so
the rule is read on time a move in plies 0–19 and on the clock left at the
end, as RAR-R18 learned).

**Rule for Stage 2 (frozen):**
- `all4` at **−15 Elo or better** (point estimate), or `nofreeze` at −10 or
  better with `freeze` carrying the rest: the loss is the size a fit and
  the clock's scale can recover; **Stage 2 runs**.
- `all4` below −15 with `nofreeze` also below −15: the search parts
  themselves cost decisions; Stage 2 does **not** run. The research then
  reads the three parts one at a time (2,000 games each, registered then),
  drops the one that costs, and re-registers.
- `all4` within ±5: the fit is optional; the gate may run directly on a
  rebuilt arm (RAR-R20's form) with the fit as its fallback on H0.

**Predictions (frozen):** `all4` **−18 ± 10 Elo** (band −30 to −6; the
−52.5 at PGO less about −20 of time left unused and in-game speed, less the
200-game lean); `nofreeze` **−8 ± 10**; `freeze` **−12 ± 10**, its time a
move 4–6% under the defaults'. P(Stage 2 runs) 0.6.

## Stage 1 results (2026-10-24, agent-run, idle host; the maintainer delegated the night's runs)

Three Colosseum `match` runs, 2,000 games each, `3+0.03`, both arms
`rarog-d16inst2-tune.exe` (`2f17467`), 0 faults and 0 time losses in each,
host 0.4–6.2% busy at launch. The runner's final report lines:

| Read | Elo | nElo | W/D/L | Pentanomial |
|---|---|---|---|---|
| `all4` (the four parts) | **−6.9 ± 8.8** | −12.0 ± 15.2 | 462/1036/502 | 22, 273, 447, 239, 19 |
| `nofreeze` (parts 1–3) | **−9.7 ± 8.9** | −16.7 ± 15.2 | 480/984/536 | 24, 270, 469, 212, 25 |
| `freeze` (the freeze alone) | **−7.8 ± 8.7** | −13.7 ± 15.2 | 467/1021/512 | 24, 255, 485, 214, 22 |

Time use from the PGN move comments (arm A against the defaults, same
binary, `nps/nps_by_arm.py` in the artifact directory):

| Read | Time a move, plies 1–20 | Time a move, all | NPS, all | Whole-game time a side |
|---|---|---|---|---|
| `all4` | −10.7% | −3.0% | +2.0% | 4.07 s against 4.19 s |
| `nofreeze` | +0.9% | −0.1% | −0.9% | 4.19 s against 4.19 s |
| `freeze` | −12.1% | −2.6% | +2.8% | 4.09 s against 4.20 s |

**Readings.** The three reads sit within one standard error of each other
and do not add: the four parts together read no worse than either half.
At equal speed the in-game NPS of the arm is not below the defaults', as
RAR-R20's composition section predicts for one binary. The freeze cuts the
opening's time a move by 11–12% and the whole game's by 2.6–3.0%, the time
it leaves on the clock; `nofreeze` spends the same time as the defaults and
still reads −9.7 ± 8.9, so the search parts' cost, if any, is in decisions
or in the constants the tune covers, and is bounded above by about −1 Elo
at one standard error.

**Rule applied:** `all4` at −6.9 is better than −15, so **Stage 2 runs**;
it is not within ±5, so the fit is not optional. The run started
2026-10-24 (the fit's command and directory below).

**Calibration.** `all4` was predicted −18 ± 10 and read −6.9: inside the
band's edge, a miss in magnitude (the loss is about a third of the
prediction; the 200-game −52.5 at PGO over-weighted the composition
effect, which RAR-R20 later showed is not speed). `nofreeze` −8 ± 10 read
−9.7 and `freeze` −12 ± 10 read −7.8: both held. The freeze's time a move
was predicted 4–6% under the defaults' and read 2.6% under over the game
but 12% under in the opening: a miss in where the time is cut, not in
whether. P(Stage 2 runs) was 0.6; it runs.

## Stage 2: SPSA block 1 over the cluster and its constants (about ten hours)

The surface `d16fix` (`tools/spsa_configs/config_d16fix.json`, generated
Colosseum files current at N = 2000): fifteen coordinates, the five
aspiration coordinates, `MopupKnownWin`, `TmDepthFreeze`, the clock's
scale `TmOptScale`, and the seven reduction terms whose negative values
the budget now charges; the b43 steps, except `TmOptScale` at the clock
fit's range and step (5000..20000, 938: a (max−min)/8 step would
perturb thinking time by 19% a side), `AspirationDelta` at 6 and
`CoreLmrTtPv` at 128 (the steps C.10's and B.2's tunes used for them;
see the rail amendment below); `r-end` 0.0031; 2,000 iterations × 30 games = 60,000 games on the
fit arm; seed 20261050; `tools/results/rar-r21-fit`. Read by PLAN rule 7c:
a full-step mover in the rounded centres continues to block 2 with
`-SeedFrom`, none stops the fit at block 1.

**Predictions (frozen):** at least two full-step movers among
`MopupKnownWin`, `TmDepthFreeze`, `AspDeltaSquareDiv` and
`CoreLmrCorrection` (0.6); `TmDepthFreeze` moves up (later freezing, more
time spent), 0.6; the journal's mean z² above its sign-redraw null (0.6).

**Amendment, 2026-10-23, before any game:** `TmOptScale` joined the
surface as its fifteenth coordinate. The cluster's visible in-game cost
is time left on the clock (RAR-R20: 5.3% less a move); `TmDepthFreeze`
governs only the freeze's share of it, while smoother iterations lowering
the instability factor is the clock's business, and the clock was fitted
(RAR-R15) on the head's stability signal, which the cluster changes.
RAR-R15 found `TmOptScale` and `TmEffortHigh` nearly collinear, so one
of them suffices and the scale is the natural one. Added prediction:
`TmOptScale` moves up by at least half a step (more time spent), 0.6.
The in-game speed loss itself is not a tune target: RAR-R20 shows it is
the tree's composition, unchanged per position.

**Rail amendment, 2026-10-24, at iteration 2 of the first start, no
centre read:** the harness's preflight warned that `AspirationDelta`
(12, step 12 on 1..100) and `CoreLmrTtPv` (179, step 256 on 0..2048)
send one first arm to the rail, and c decays only 2.2× over the run, so
the asymmetric perturbation would have stayed for most of it. The
earlier tunes of these coordinates used half the step (`c10block1`:
AspirationDelta 6; `b23core`, `b27all`: CoreLmrTtPv 128) and RAR-R15
records "no rail warning" as a preflight check. The run was stopped at
iteration 2 (60 games, no centre looked at), its directory and manifest
moved to `fit-aborted-rail-010710/` in the artifact directory, the two
steps set to the precedent, the Colosseum files regenerated, and the
fit restarted from iteration 0 under the same seed. No other coordinate
rails at c_0 (the amending script asserts it). Predictions unchanged.

### The fit's start and its pause test (2026-10-24, agent-run)

- **Started 23:21:17Z** (01:21 local) under seed 20261050 in
  `tools/results/rar-r21-fit`, manifest
  `colosseum_spsa_rar-r21-fit_20261024_012116.manifest.txt`, host 4.5%
  busy, `rail_warnings: none`, 15 slots, about 20 s an iteration (5,300
  games an hour, about 11 h). An earlier start at 01:07 local is the rail
  amendment's aborted one (above).
- **Pause tested on the live run:** Ctrl+C in the runner's tab at iteration
  19 (23:27:50Z). The runner launched no new game, gave the games in flight
  30 s, and exited at a committed boundary; no engine or runner process
  remained; the manifest records `interrupted: ... Re-run the same command
  to resume`; `colosseum-cli spsa status` read 19/2000 from the durable
  snapshot.
- **Resume tested:** the same command, `run_r21.ps1 -Read fit`, carried the
  recorded seed over, started a second manifest
  (`colosseum_spsa_rar-r21-fit_20261024_013011.manifest.txt`, 23:30:12Z,
  host 3.8% busy) and continued from iteration 19 (34/2000 five minutes
  later). The 15 iterations before the pause were kept, as the journal
  promises.
- **Host-idle refusals:** the harness's 2-second window refused six starts
  at 16–35% CPU on a box that a 12-second read put at 0.2%, the bursts
  coming from the desktop app while the agent read its terminal. Two
  wrappers beside `run_r21.ps1` retry a refused start once a minute and
  never retry a run that ran: `start_fit_when_idle.ps1` (a first start,
  stops once the run directory exists) and `resume_fit_when_idle.ps1`
  (stops unless the refusal was the idle check). Both record their attempts
  in a log beside them. The plain command remains the resume of record.
- **How the maintainer pauses and resumes:** Ctrl+C once in the tab that
  runs the fit, wait for the prompt (up to 30 s); resume with the plain
  command, or with `resume_fit_when_idle.ps1` when the preflight keeps
  refusing a quiet host. `colosseum-cli spsa status tools/results/rar-r21-fit`
  reads the state without touching it.

### Stage 2 results (2026-10-24; the maintainer ran the tune from 09:52 local, the agent read it)

Block 1 completed: 2,000 iterations, 60,000 games, 0 faults, no rail, no
clamp (`colosseum_spsa_rar-r21-fit_20261024_095217.manifest.txt`,
`tools/results/rar-r21-fit`, the runner's closing table pasted by the
maintainer and equal to `tuned-options.txt`). The rounded values, with the
move in units of the registered step:

| Coordinate | Seed | Tuned | Step | Moved |
|---|---|---|---|---|
| AspirationDelta | 12 | 12 | 6 | 0.00 |
| AspGrowthPct | 164 | 178 | 38 | +0.37 |
| AspGrowthHighPct | 165 | 168 | 38 | +0.08 |
| AspGrowthAdd | 8 | 10 | 6 | +0.33 |
| AspDeltaSquareDiv | 10193 | 14707 | 4750 | **+0.95** |
| MopupKnownWin | 6000 | 5826 | 1438 | −0.12 |
| TmDepthFreeze | 5000 | 5781 | 875 | **+0.89** |
| TmOptScale | 10000 | 10801 | 938 | **+0.85** |
| CoreLmrPv | 399 | 401 | 256 | +0.01 |
| CoreLmrPvWindow | 306 | 279 | 256 | −0.11 |
| CoreLmrCorrection | 3330 | 3357 | 1024 | +0.03 |
| CoreLmrGivesCheck | 1030 | 840 | 512 | −0.37 |
| CoreLmrImprovement | 333 | 282 | 256 | −0.20 |
| CoreLmrTtPv | 179 | 155 | 128 | −0.19 |
| CoreLmrQuietHistory | 316 | 347 | 128 | +0.24 |

**Rule 7c (`tools/spsa_block_rule.py d16fix tools/results/rar-r21-fit
--min-moved 1`, its record `block-rule.txt` beside the run):** "0 of 15
coordinates moved >= 1 step from the block's seeds; rule needs 1: STOP".
The fit ends at block 1 and these rounded values are theta. Three
coordinates stopped within 5–15% of a full step; the rule was frozen and is
applied as written, and the gate, not a second block, decides their worth.

**Sign-redraw null** (`nps/spsa_signnull.py` over `spsa history --csv`: a
coordinate's final displacement divided by the root of the sum of its
squared per-iteration updates, which is its standard deviation when every
update's sign is a coin flip): mean z² **1.93** against a null of 1.00;
`AspDeltaSquareDiv` z = +2.96, `TmDepthFreeze` +2.78, `TmOptScale` +2.66,
every other coordinate within ±1.2. The trajectories of the three were
monotone from iteration 145 on (read mid-run at 1,453 iterations, after the
maintainer asked; descriptive, no decision taken on it).

**Reading.** The tune found three directions and nothing else. The two
clock coordinates both spend more time: the freeze moves from 50% to 58%
of the soft target and the scale adds 8%, which is the time Stage 1
measured the freeze leaving on the clock (12% of the opening's time a
move, 3% of the game's). The squared divisor rises by 44%, so the first
window at a large score is narrower than Stockfish's constant gives: the
HCE's known-win scores are smaller than Stockfish's, so the same divisor
over-widened. The known-win offset did not move, as predicted for a step
function. The seven reduction terms moved at most 0.37 step, within what a
coin-flip walk gives: the budget did not move where the reductions want to
be, and the fit changes few decisions.

**Calibration.** Prediction 1 (at least two full-step movers among
`MopupKnownWin`, `TmDepthFreeze`, `AspDeltaSquareDiv`, `CoreLmrCorrection`,
0.6) **missed in magnitude**: the two named ones reached 0.89 and 0.95 of a
step, none a full step; `MopupKnownWin` and `CoreLmrCorrection` did not
move at all, a miss in mechanism for those two (the offset is a step
function, and the correction term's negative values were charged by the
budget without wanting a new scale). Prediction 2 (`TmDepthFreeze` moves
up, 0.6) **held**. The amendment's prediction (`TmOptScale` up by at least
half a step, 0.6) **held**. Prediction 3 (mean z² above the null, 0.6)
**held** at 1.93.

**Evidence:** `tools/results/rar-r21-fit` (result.json, games.jsonl,
tuned-options.txt, block-rule.txt); the trajectory and the null statistic
under `nps/` in the artifact directory (`fit_history_full.csv`,
`fit_signnull.txt`).

## Stage 3: the gate

RAR-R20's gate, re-registered then on an arm rebuilt with the fitted
values (a new recipe and binary, `[-5,5]` at `3+0.03`, cap 20,000 pairs,
the probe with no clock stop, the `10+0.1` read), with a fresh frozen
prediction from Stage 1's and Stage 2's numbers. A gate that passes lands
D.1.6 (D.1.6.2's landing list); one that fails returns the cluster to
research with the part-by-part reads.

**Stop rule for the whole road:** no stage starts before the previous
one's rule allows it; a stage that runs is never shortened after its
results are seen; a refused run or a fault stop halts the road.

**Evidence layer:** Stage 1 is Elo at ±9 and time use; Stage 2 is a tune's
movement, not Elo; only Stage 3 accepts anything.
