# RAR-R15 — D.1.4: the clock fit, its wire checks, its activation read and SPSA block 1 over the twelve clock coordinates

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-22, before any game of the wire
  checks below or of the block. The wire checks and the block are the
  maintainer's commands; the activation read is the agent's.
- **Kind:** PLAN D.1.4, the fit of D.1.3's clock cluster (PLAN rule 4 with
  the zero-game sweep replaced by what a clock-only coordinate allows). It
  accepts nothing: theta is the last completed block's rounded centres,
  baked only for D.1.5's gate.
- **Binary:** `tools/test_engines/rarog-d13cluster-tune.exe` (SHA-256
  `87c92774550c41d3…`, `pext-tune`, built clean at `eff17c3`, whose engine
  inputs are `5fec0d7`'s; bench 13,987,223). The activation read used
  `analysis/artifacts/d14-clock-fit-2026-10-22/rarog-d14-diag.exe`
  (`03355c83c33fd5ac…`, `cargo build --release --features diag` at
  `a9599af`, the same engine inputs; bench 13,987,223 / EBF 2.565).

## (a) Wire checks

**`TmOptScale` (played 2026-10-22, PLAN D.1.4).** The wire is live:
−22.5% a game, −55% a move in plies 0–19. The registered −60% a game was
missed through clock feedback. That run sizes the rest: arm `seeds`, plies
0–19, read 85.5 ms a move with SD 64.5 over 1,952 moves, an SE of 1.7% of
the mean, so a difference between arms has an SE of about 2.4% at 200 games.
A measured ratio was about the formula's ratio to the power 0.66 (0.45
against 0.3).

**The five new coordinates (frozen here).** Each check plays the coordinate
at a range end (arm A) against the seeds (arm B), `-CategoricalTuneBuild`,
on the binary above. The read is per move at equal clock, from
`tm_wire_read.py`:
- plies 0–19, where both arms' clocks are still close, for a factor that
  acts at every clock;
- the 500–1000 and 1000–2000 ms clock bands for the horizon, which acts
  only below 2.5 s.

The formula ratios come from `wire_expectations.py` (`2bf4d446…`): the
stability and effort factors over the activation read's k and share
distributions, the horizon's optimum at ply 80. A check is **live** when its
reading moves in the stated direction by more than twice the difference's SE
(5% at 200 games, 2.5% at 800). A reading past the far end of the band is a
calibration note, not a failure. A check that is not live stops the fit
until it is understood (RAR-R14's falsifier).

| Check | Arm A | Arm B | Games | Read | Formula | Expected (band) | Live if |
|---|---|---|---:|---|---|---|---|
| horizon | `TmHorizonPerSec=20` | seeds | 200 | ms a move, 500–1000 and 1000–2000 ms bands | ×0.68 and ×0.68–0.83 | ×0.78–0.89 (0.65–0.97) | both bands ≥ 5% lower |
| stability base | `TmStabBase=20000` | seeds | 200 | ms a move, plies 0–19 | ×1.595 | ×1.36 (1.15–1.60) | ≥ 5% higher |
| stability slope | `TmStabSlope=2000` | seeds | 200 | plies 0–19 | ×0.844 | ×0.89 (0.70–0.97) | ≥ 5% lower |
| stability floor | `TmStabSlope=2000`, `TmStabFloor=10000` | `TmStabSlope=2000` | 200 | plies 0–19 | ×1.134 | ×1.09 (1.02–1.25) | ≥ 5% higher |
| effort floor | `TmEffortFloor=0` | seeds | 800 | plies 0–19 | ×0.908 | ×0.94 (0.85–0.99) | ≥ 2.5% lower |

What departs from PLAN D.1.4's "each new coordinate at a range end in a
200-game match", and why:
- **The floor is checked at `TmStabSlope` 2000 on both arms.** At the seed
  slope the floor binds only from k = 7, on 5.3% of iterations, for a
  formula effect of ×1.004. That is a null, which cannot show a wire.
- **The effort floor plays 800 games.** Its expected −6% is about 2.5 SE at
  200 games.
- **`TmEffortHigh`, whose range was widened rather than added, is not
  checked.**

The batch is `analysis/artifacts/d14-clock-fit-2026-10-22/run_wire_checks.ps1`
(`593135b3…`). It plays the five checks with seeds 20261024–20261028 into
`tools/results/d14-wire-*` and writes each reading beside itself; a dry run
of all five resolved to policy. It reads with `tm_wire_read.py` and
`wire_resolution.py` (`f999943c…`).

## (b) Activation read (run 2026-10-22, the agent's)

`tools/diag/tm_activation.py` on B.5's 300 positions at `3000+30`, seeds,
one thread, Hash 64, idle host (1.5% CPU, no engine or build process):
3,432 completed iterations (`activation.txt`, `720e4a45…`;
`activation.json`, `2276ebf4…`).

| Factor | At one end | At the other end |
|---|---|---|
| stability | k = 0 (`TmStabBase`): 38.1% | floor: 0.0% |
| effort | `TmEffortHigh`: 87.5% | `TmEffortLow`: 0.0% |
| falling eval | clamp 0.572: 66.9% | clamp 1.708: 9.6% |
| instability | base: 0.0% | — |

- k: 0 on 1,306, 1 on 671, 2 on 436, 3 on 311, 4 on 229, 5 to 9 on 459,
  10 or more on 20.
- Best-move node share: p10 0.071, p50 0.398, p90 0.822.
- Counters: 87 fail-high stops in 300 searches, no mate stop, no
  first-iteration guard.

**No factor is at one end on more than 90% of iterations, so by PLAN D.1.4's
rule no coordinate drops and all twelve enter the block.** The rule's
mapping from a factor to "its coordinate" was not needed. A factor of two or
three coordinates leaves it undefined; had one crossed, the mapping would
have gone to the maintainer before any game.

Reported, deciding nothing:
- The effort factor is 2.5 points from the line. Its high end is a constant
  on those iterations, so `TmOptScale` and `TmEffortHigh` are nearly one
  scale and can drift together along their product.
- `TmStabFloor` binds on no iteration at the seeds; it becomes active only
  if the slope rises.
- The fail-high stop ends 29% of these first-move searches on a lower
  bound. That is not B.5's discarded-best-move case, since the stop plays
  the move that failed high, but it is the mid-iteration share D.1.5's
  games should report.

## (c) SPSA block 1 (frozen)

- **Surface:** `tools/spsa_configs/config_d14block1.json` (`4187c327…`;
  `fixed_d14block1.json` `cc6d1aac…`; `colosseum/d14block1.tune.toml`
  `935a2440…`, `d14block1.run.toml` `4094c935…`). Written by
  `gen_d14block1.py` (`68bad49f…`) and `spsa_config_to_colosseum.py
  d14block1 --iterations 2000`.
- **Coordinates:** the twelve clock coordinates at the engine defaults, the
  b43 step, the declared ranges, except `TmOptScale` at its 2.3.0 range of
  5000..20000.
- **Run:** Colosseum `spsa`, **2,000 iterations × 30 games = 60,000
  games** on 15 slots, `3+0.03`, Hash 64, one thread, the UHO book, seed
  **20261023**, `r-end` 0.0031, α 0.601, γ 0.102, into
  `tools/results/d14-spsa-block1`. Its dry run on 2026-10-22 resolved 12
  coordinates with no rail warning.
- **Command, only after all five wire checks read live:**
  `./tools/colosseum.ps1 -Mode spsa -Engine tools/test_engines/rarog-d13cluster-tune.exe -ConfigGroup d14block1 -Iterations 2000 -TotalGames 60000 -Seed 20261023 -ExpectRevision eff17c3 -ExpectBench 13987223 -Dir tools/results/d14-spsa-block1`.
- **Blocks:** at most two, a smaller ceiling than rule 7c's three, as PLAN
  D.1.4 writes. After block 1, `python tools/spsa_block_rule.py d14block1
  tools/results/d14-spsa-block1`. Three or more full-step movers run block
  2: the same command with `-SeedFrom tools/results/d14-spsa-block1`, seed
  20261029, into `tools/results/d14-spsa-block2`. Fewer, and block 1's
  rounded centres are theta. A chained launch may run block 2
  unconditionally (rule 7c as amended 2026-09-26); theta is the last
  completed block either way.

**Readings after each block (frozen):**
- the rule's movers and verdict;
- the `arm_clamps:` line of the manifest;
- the product `TmOptScale × TmEffortHigh` against its seed product, beside
  each coordinate's own movement;
- the rounded centres.

**Use (frozen):** theta is baked only for D.1.5's gate. A dead wire or a
fault stop leaves nothing to bake, and D.1.4 returns to its registration.

**PRE-REGISTERED PREDICTION (frozen before any game):**
1. Wire checks: all five live, 0.75. The weakest is the effort floor, 0.85
   alone.
2. Block 1's full-step movers (b43 steps): **3**, 80% band 1 to 6.
3. The count of three is reached, so block 2 runs on the rule: 0.5.
4. `TmOptScale` and `TmEffortHigh` move in opposite directions by more than
   half a step each while their product moves less than half a step: 0.3.
5. `TmHorizonPerSec` ends above its seed of 50 (less contraction, RAR-R14's
   H3): 0.5. No better than a coin, written so the read cannot be
   rationalised.
6. No arm clamped at a range end: 0.85.
7. Block 1 completes without a fault stop in 8 to 11 hours: 0.85.
- Expected Elo: none here; only D.1.5's SPRT measures strength (RAR-R14's
  +3 ± 4 at the gate stands until D.1.5 re-freezes it).
- Most likely failure: the surface is flat at `3+0.03` (RAR-R14's H2). Then
  one or two coordinates move, mostly the near-collinear scale pair, and
  the rule stops after block 1.

## Result of (a), the five wire checks (played 2026-10-22, maintainer-run)

`run_wire_checks.ps1` played all five. Each ran on an idle host (0.4–4.4%
busy, no waiver), completed with 0 faults and 0 time losses, and its recount
agreed. The readings are `d14-wire-<check>.txt` beside the script:
horizon20 `c3b835f9…`, stabbase20000 `f8bf5bd5…`, stabslope2000
`d0ba0899…`, stabfloor10000 `69a7a599…`, efffloor0 `365857ee…`.

| Check | Read (arm A / arm B) | Ratio | Expected (band) | Frozen rule |
|---|---|---:|---|---|
| horizon, 500–1000 ms band | 42 / 43 ms a move | 0.977 | 0.78–0.89 (0.65–0.97) | **not ≥ 5% lower** |
| horizon, 1000–2000 ms band | 55 / 68 ms | 0.809 | 0.78–0.89 (0.65–0.97) | live |
| stability base | 129.2 / 92.8 ms, plies 0–19 | 1.392 | 1.36 (1.15–1.60) | live |
| stability slope | 75.4 / 89.1 ms | 0.846 | 0.89 (0.70–0.97) | live |
| stability floor (slope 2000 on both arms) | 89.8 / 76.8 ms | 1.169 | 1.09 (1.02–1.25) | live |
| effort floor, 800 games | 81.6 / 89.4 ms | 0.913 | 0.94 (0.85–0.99) | live |

The standard errors are 1.5–1.9% a mean at 200 games and 0.9% at 800, as
sized.

**Four checks are live. The horizon check fails its frozen rule**, which
required both bands ≥ 5% lower; the 500–1000 ms band read −2.3%. Under
the registration a check that is not live stops the fit until it is
understood.

What is known about the miss, all read after the games:
- **The engine applies the coordinate exactly.** `horizon_engine_check.py`
  (`7b481162…`) on a `tune,diag` build (`rarog-d14-tune-diag.exe`,
  `612f02cb4e9c81c5…`, bench 13,987,223) reads the optimum at a 750 ms
  clock, ply 80, as 35.8 ms at 20 against 52.2 at 50: a ratio of 0.686, the
  formula's to three decimals. At 1,500 ms it reads 0.833, also the
  formula's.
- **The instrument reports the coordinate elsewhere.** The 1000–2000 ms band
  fell 19%, past the expectation. The clock floor rose: p10 146 → 242 ms,
  median 735 → 1,024 ms, games under 200 ms 35 → 11 of 200. The 100–200 and
  200–500 ms bands fell 21% and 9%.
- **So the miss is in the response, not the wire.** At about 40 ms a move,
  time spent stops following the optimum. The seed arm spends 0.83 of its
  52 ms optimum in that band, while arm A spends 1.17 of its 36 ms optimum.
  Likely causes, not tested: the soft stop is checked only between
  completed iterations, whose granularity near 40 ms is a large share of
  the move; and arm A reaches that band later in the game.
- **Calibration of the four live checks:** the 0.66 damping taken from
  `TmOptScale` underpredicted them. Their measured log ratios are 0.71
  (base), 0.98 (slope), 1.24 (floor) and 0.94 (effort floor) of the
  formula's, so the damping is closer to 1 for factors than for a 0.3
  scale.
- **Prediction 1 (all five live, 0.75) missed.** Its weakest-check guess
  (the effort floor) was wrong: the effort floor passed, the horizon did
  not.

**Decision owed to the maintainer before block 1.** The frozen rule stops
the fit, and whether this is "understood" is not the agent's to rule after
the result. The options:
1. **Accept the wire as live on the evidence above, and run block 1 with all
   twelve coordinates,** recording the failed criterion as a calibration
   (the agent's recommendation).
2. Drop `TmHorizonPerSec` from the surface, fixed at its seed 50.
3. Re-register a horizon check at lower clocks, or one read by the clock
   floor, and run it before the block.

## Maintainer decision and review (2026-10-22, before any game of the block)

**Maintainer decision 2026-10-22: `TmHorizonPerSec` stays in the surface.**
The horizon check's miss in the 500–1000 ms band (−2.3% against the frozen
"≥ 5% lower") is recorded as a calibration, not a pass. The engine applies
the coordinate exactly there (the optimum at 750 ms is 0.686 of the seed's),
the 1000–2000 ms band fell 19% and the clock floor rose (p10 146 → 242 ms,
games under 200 ms 35 → 11 of 200): the wire is live, and the frozen rule
read it at a clock where time spent no longer follows the optimum. What
failed was the check's instrument assumption, that a per-band read follows
the optimum at every clock. Prediction 1 stays a miss. Block 1 runs on this
decision with the twelve coordinates.

**Independent review 2026-10-22, before any game.** D.1.3's implementation,
the surface and the block's shape were re-read from source, tests and
artifacts, not from the records. No defect was found; nothing in the surface
or the command changes. Findings:

- **D.1.3 as recorded.** The horizon, stability and effort factors, the
  share over the whole search (`RootMove.nodes` over the thread's nodes,
  reset per search), the mate stop (three iterations, same move and
  distance; a loss in two plies; clock searches only), the fail-high stop
  (main thread, not pondering, elapsed at or past the last completed
  iteration's target; the bounded `info` line and `bestmove` both name
  `pv_table[0][0]`, and `report_result_line` prints no second line), the
  guard (`root_depth > 1`, set at every root window), the main-thread-only
  stops in a pool (`search_parallel` ends the helpers when the main thread
  returns), MultiPV's mate stop without the fail-high stop, and the per-node
  code (the diff to `node.rs` removes three lines) are as PLAN D.1.3 says.
  The fail-high test fails under both mutations: the stop disabled (D.1.3's
  check) and the stop firing on every root fail-high (this review: the
  frozen-clock search then ends at depth 5, not 10). The engine inputs of
  `5fec0d7`, `eff17c3`, `da2df4e`, `a9599af` and the head are identical
  (`git diff --stat` empty), every SHA-256 cited here and in PLAN D.1.3–D.1.4
  matches its file, and the tune build benches 13,987,223 / EBF 2.565 on
  this host.
- **The surface.** `gen_d14block1.py --check` and
  `spsa_config_to_colosseum.py d14block1 --iterations 2000 --check` pass,
  `audit_spsa_coverage.ps1` is clean, and the tune build advertises all
  twelve coordinates as integer spins (its `uci` output). Every seed is more
  than two steps from a range end (the dry run's `rail_warnings: none`; the
  closest is `TmEffortFloor`, 2.7 steps under 9500) and the smallest end
  perturbation, `TmHorizonPerSec`'s 2.30, rounds to 2. `TmOptScale`'s
  surface range of 5000..20000 stays: the 2000 end exists for the wire
  check, 0.3 cost −145 Elo, and the seed is 5.3 steps from 5000.
- **Constants the clock reads that are not coordinates, each left as it
  is:** the falling-eval clamps 0.572 and 1.708 (the lower clamp is the
  factor on 66.9% of iterations, so it is the de facto base scale and would
  be a third near-collinear scale beside `TmOptScale` and `TmEffortHigh`;
  the donor's shape, outside the handoff); the budget's shape (0.8097, the
  `optScale` and `maxScale` constants to 6.873, the `timeLeft` formula),
  which RAR-R14 chose to keep as Rarog's Stockfish-shaped budget; the
  instability count's halving and the score EWMA's weights (shape constants
  the slopes scale); `MATE_STOP_ITERATIONS` = 3 (it spends time in won
  games only, which a fit cannot read); the reserves (D.1.2.2 keeps them);
  the fifty-move cap and the 2,048-node poll. Adding any would broaden the
  mechanism past RAR-R14's handoff on no measured defect.
- **`TmStabFloor` stays, with its noise quantified.** It has no gradient at
  the seeds: it binds from k = 13 at the seed slope, on under 0.6% of
  iterations, so block 1 can only random-walk it. Under this schedule a
  gradient-free coordinate ends a block 0.41 steps from its seed (SD;
  `tools/spsa_convergence_model.py`'s flat-coordinate variance at N = 2000,
  `r-end` 0.0031, gradient noise 4.2), so a full-step move by noise alone
  has about 1.5% probability a block. It stays because `TmStabSlope` is
  live (−15% at 2000) and a slope that rises inside the block gives the
  floor its gradient. Reading rule, frozen now: its move under a step is
  noise, and a full-step move counts for rule 7c only with the slope above
  its seed. The same 0.41-step noise bounds the drift of the near-collinear
  pair along its product.
- **Search coordinates are not added.** A ±20% change in time a move is
  0.2 of an iteration at EBF 2.565, inside the per-position spread C.10's
  fit averaged over; the aspiration constants set a window's growth after a
  fail, which the fail-high stop reads at a boundary and does not change
  (the loop is RAR-S89's and RAR-S17's refuted territory and D.1.3's
  non-goal); any aspiration coordinate moves `bench 13`, so the fitted theta
  would no longer be the clock-only change D.1.5 gates and the gate would
  attribute a search refit to the clock; and five more coordinates cost
  resolution on the twelve. No measurement in this engine says the
  aspiration optimum moved with the clock. Trigger: a D.1.5 result that H0
  returns to research while the fail-high stop ends a large share of moves,
  or the next search refit, which reads the aspiration constants under the
  fitted clock.
- **Named, not tested:** a fail-high after a fail-low, whose bound can lie
  under the previous exact score (not a discard: the first move searched is
  the previous best, and a later move that fails high beats it at this
  depth); MultiPV's mate stop; and in a pool the fail-high stop's result
  enters `select_parallel_result`'s vote rather than being played outright
  (the rated 1T configuration plays it; D.1.5's 4T read covers the pool).
  Known open items left as they are: `tm_profile.py` transcribes the
  fifty-move horizon (D.1.3's finding 3, to fix before D.1.5's ≥ 0.9 ×
  maximum read) and labels its 99.9th percentile `p99`;
  `first_kpk_search_in_a_fresh_process_costs_what_a_warm_one_does` is a
  load flake on the unchanged engine.
- **The shape.** 2,000 × 30 games and at most two blocks is rule 7c's shape
  for seeds never fitted on this search, whose distance from the optimum is
  unknown; the 2026-10-20 amendment's single longer run is for a surface a
  pilot has read as near its optimum, which no zero-game instrument can do
  for a clock coordinate.

**The block, prepared 2026-10-22 (dry run through `colosseum.ps1` on an
idle host, 5.4% busy, no waiver;
`tools/results/colosseum_spsa_d14-spsa-block1_20261022_202421.manifest.txt`):**
12 coordinates, 2,000 iterations × 30 games = 60,000, `r-end` 0.0031, no
rail warning, 15 slots, Hash 64, Threads 1, MultiPV 1, seed 20261023,
`3+0.03` with a 20 ms margin, the tune build at `eff17c3` with bench
13,987,223 verified against the registration; the run directory unused. The
command is (c)'s, unchanged. The frozen readings and predictions 2–7 stand.

## Amendment 2026-10-22, before any game of the block: a thirteenth coordinate, the cross-move score trend

**Maintainer decision 2026-10-22:** this fit is the last clock work for some
time, so a mechanism both donors carry and the cluster lacked joins it now
rather than in a later fit: the score trend across the move pair. Stockfish's
falling eval compares the previous move's average score with this
iteration's (`bestPreviousAverageScore`); Reckless's score trend is the
previous move's score less this one's. Rarog's falling eval read only this
search's prior iterations, which the ADR-0065 audit ticked as "score trend:
present" without the cross-move half.

**Evidence, zero games** (`cross_move_screen.py`, `4d777fdf…`;
`cross_move_screen.txt`, `a7c9c372…`, beside this entry's artifacts): over
the seeds arm of the six wire checks (1,600 games, 89,676 consecutive
own-move pairs) the score moves 18 cp at the median across a pair, 39 at p75
and 80 at p90, symmetrically (22.3% of pairs fall by 20 cp or more, 25.8%
rise). A cross-move term at Stockfish's raw slope lifts the falling factor
off its lower clamp on 20.0% of moves, beside the within-search term's 33.1%
of iterations in the activation read: a live signal of the same order from
another source, the opponent's reply. No Elo is claimed; the fit sets the
weight.

**Mechanism** (engine `1f185cb`, clock-only, bench 13 unchanged). Each
thread keeps the root score its previous search in the game ended on
(`previous_search_score`, cleared by `ucinewgame`; a search that completed no
iteration leaves it in place), and the falling eval becomes `TmFallBase +
TmFallSlope × (the within-search fall) + TmFallPrevSlope × (the previous
search's score − this iteration's)`, clamped at 0.572..1.708 as before.
`TmFallPrevSlope` seeds at 204 (0.0204 a cp, Stockfish's raw value; its
score unit is not this engine's, so the seed is a start, not a claim), range
0..1000, b43 step 63. Interactions: it shares the clamp with the
within-search slope, so the two trade off inside the unclamped band; nothing
per node; helpers keep their own previous score (own-thread signals,
RAR-R05). Invariants: `bench 13` (no clock, the target is infinite); the
first search of a game reads a zero cross-move fall. Falsifier: a wire check
that does not move time a move, and the fit, which may send the slope to 0.

**Wire check (frozen before its games).** Arm A `TmFallPrevSlope=1000`
(0.1 a cp) against the seeds, 200 games, `-CategoricalTuneBuild` on the
thirteen-coordinate tune build, seed 20261030, into
`tools/results/d14-wire-fallprev1000`, read per move at equal clock in plies
0–19 by `tm_wire_read.py` and `wire_resolution.py`. Formula: the cross-move
term alone gives a mean factor of 0.930 at arm A against 0.684 at the seed,
×1.36; expected ×1.25 (band 1.10–1.60) after the four live factor checks'
calibration (0.7 to 1.0 of the log ratio). Live if ≥ 5% higher (twice the
SE at 200 games). The activation replay cannot read this signal (every
position there is a first search), so the screen above is its activation
evidence.

**The surface at thirteen, regenerated before any game:**
`config_d14block1.json` (`89f2da6c…`), `d14block1.tune.toml`
(`7ca3514b…`), `fixed_d14block1.json` and `d14block1.run.toml`
unchanged, by `gen_d14block1.py` (`86b04bb2…`) and the converter at
N = 2000; `audit_spsa_coverage.ps1` clean. The tune build is
`tools/test_engines/rarog-d13cluster7-tune.exe` (`a5efb11c5e25ea9f…`, `pext-tune`, built clean at
`2b9831f`, whose engine inputs are `1f185cb`'s; bench 13,987,223).
The block's command becomes
`./tools/colosseum.ps1 -Mode spsa -Engine tools/test_engines/rarog-d13cluster7-tune.exe -ConfigGroup d14block1 -Iterations 2000 -TotalGames 60000 -Seed 20261023 -ExpectRevision 2b9831f -ExpectBench 13987223 -Dir tools/results/d14-spsa-block1`,
the run directory still unused, after the wire check above reads live.
Blocks, readings and use are unchanged.

**Prediction, amended before any game** (items 2–7 above were frozen over
twelve coordinates and stand as written):
- 2a. Block 1's full-step movers over thirteen: **3**, 80% band 1 to 6,
  `TmFallPrevSlope` among them with 0.5 (a donor seed in a foreign unit).
- 8. The cross-move wire check is live: 0.85.
- 9. `TmFallPrevSlope` ends above zero: 0.8; above its seed: 0.5.
- The coordinate changes neither the block's shape nor the chance of a
  second block, which rule 7c's count decides.

**Result of the cross-move wire check (played 2026-10-22, agent-run as cheap
qualification, idle host 3% busy, no waiver).** 200 games, 0 faults, 0 time
losses, the runner's pentanomial 4, 28, 43, 24, 1 (recount agrees), holds
p999 103 ms; manifest
`tools/results/colosseum_match_d14-wire-fallprev1000_20261022_211325.manifest.txt`
(`828c1321…`), reading `d14-wire-fallprev1000.txt` (`776a8557…`). Plies
0–19 at equal clock: **112.9 ms a move against the seeds' 97.1, ×1.163**,
SE 1.9% a mean, so the difference is about six SE: **live** by the frozen
rule (≥ 5% higher), inside the band (1.10–1.60) and under the point
estimate of 1.25. Calibration: the log ratio is 0.49 of the formula's
(×1.36), the smallest damping of the six factor checks (the four earlier
ones read 0.71–1.24); the within-search term, held at zero in the formula,
already lifts the factor on a third of iterations, so the cross-move term
has less clamp room than the formula gave it. Arm A spends its gain early:
its clock floor median is 400 ms against the seeds' 654, and 59 of 200
games touch 200 ms against 38. Prediction 8 (live, 0.85) held.

**The block at thirteen, prepared 2026-10-22** (dry run through
`colosseum.ps1`, host 0.5% busy, no waiver;
`tools/results/colosseum_spsa_d14-spsa-block1_20261022_211548.manifest.txt`):
13 coordinates, 2,000 × 30 = 60,000 games, `r-end` 0.0031, no rail
warning, 15 slots, Hash 64, Threads 1, MultiPV 1, seed 20261023, the tune
build `a5efb11c…` at `2b9831f` with bench 13,987,223 verified; the run
directory unused. The manifest's `repo_dirty: True` is two untracked files
of the maintainer's in the working tree, not an engine or surface input.
