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
