# RAR-M86 — What the case-3 repair gate returns at small true effects

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11 before the simulation ran. The
  agent's; no engine and no game.
- **Kind:** a method read: the operating characteristic of a registered gate.
- Disposition: observation — read 2026-11-11: at a true zero the gate reverts 34% of
  repairs, passes 34% at H1 and caps 32%.
- **Why now:** two case-3 gates in a row ended at H0, [RAR-S114](RAR-S114.md)
  at nElo −3.7 ± 6.9 and [RAR-S115](RAR-S115.md) at −14.7 ± 13.8. Both
  registrations forecast the cap as the likeliest stop (0.70 and 0.40), and
  both took from RAR-M10 that a near-zero repair "runs to the cap". Whether
  D.6.3's rework is worth a gate depends on what an H0 from this gate says
  about an effect near zero.

## The gate read

AGENTS' repair case 3: SPRT on `[-5,5]` nElo, pentanomial,
`model=normalized`, α = β = 0.05, cap 6,000 pairs. H0 reverts the repair; H1
or the cap keeps it.

## Method

`analysis/artifacts/gate-oc-2026-11-11/gate_oc.py` (SHA-256 `42cea511…`).
Pairs are drawn from a pentanomial, the log-likelihood ratio is recomputed
after every pair, and the first bound crossed, or the cap, is the outcome:
20,000 gates for each true effect, seed 20261111.
- **The ratio** is the normalized form: with *t* the pair mean's distance
  from one half in units of the pair's standard deviation, and a bound in
  nElo times √2 · ln 10 / 800 on that scale, the ratio is
  pairs × (t₁ − t₀) × (t − (t₀ + t₁)/2).
- **The zero-effect pentanomial** is RAR-S114's sample made symmetric. A true
  effect tilts it exponentially until its normalized mean is the target.
- **Beside it,** the same 12,000 games as one fixed read with a harm rule set
  for a 5% false alarm at zero, from the normal approximation.

**Wire, passed before registration:** the formula gives −2.975 for RAR-S114's
final pentanomial and −2.977 for RAR-S115's, against the runner's −2.974 and
−2.969.

## Exposure, stated

Seen: the wire, and a hand estimate from RAR-M10's drift constant (8.3e-6 per
game per nElo of bracket width per nElo of effect), which puts a driftless
walk's standard deviation at the cap near 3.2 against bounds of ±2.94.
Unexposed: every simulated share.

## Use (frozen)

The table is reported as it comes. It changes no rule here: AGENTS' gating is
the maintainer's. It informs D.6.3's verdict and the calibration lesson, and
any proposal to change case 3 goes to the maintainer with it.

## PRE-REGISTERED PREDICTION (frozen before the run)

1. At a true 0: H0 0.34 ± 0.03, H1 the same, the cap 0.32 ± 0.05. 0.7.
2. At a true −5: H0 at least 0.93. 0.85.
3. At a true −2: H0 between 0.50 and 0.70. 0.6. At +2: H0 between 0.08 and
   0.18. 0.6.
4. Median games at a true 0: 7,000 to 10,000. 0.6.
5. **Most likely failure mode:** the drawn pentanomial's variance or the
   every-pair check differs from the runner's enough to move the shares by a
   few points; the sign of the finding would stand.

## Result (2026-11-11)

`gate_oc.py run`, unchanged since registration; log `gate_oc.log` beside it.
The zero-effect pentanomial is 0.025, 0.248, 0.454, 0.248, 0.025.

| true nElo | H0, reverted | H1 | the cap | median games | one fixed read of 12,000 games fires |
|---:|---:|---:|---:|---:|---:|
| −10 | 0.994 | 0.003 | 0.004 | 3,074 | 0.934 |
| −5 | 0.846 | 0.043 | 0.111 | 5,152 | 0.473 |
| −3 | 0.668 | 0.113 | 0.220 | 6,541 | 0.242 |
| −2 | 0.559 | 0.170 | 0.271 | 7,323 | 0.155 |
| −1 | 0.454 | 0.246 | 0.300 | 7,823 | 0.092 |
| 0 | 0.343 | 0.340 | 0.317 | 8,081 | 0.050 |
| +1 | 0.248 | 0.447 | 0.305 | 7,814 | 0.025 |
| +2 | 0.175 | 0.556 | 0.269 | 7,297 | 0.011 |
| +3 | 0.117 | 0.666 | 0.217 | 6,549 | 0.005 |
| +5 | 0.043 | 0.848 | 0.109 | 5,183 | 0.001 |

The last column is the same 12,000 games read once, with a harm rule at −5.22
nElo: a standard error of 3.17 nElo and a 5% false alarm at zero.

**What it says.**
- **A neutral repair reaches a bound two times in three,** and the two
  bounds are equally likely. It is reverted in 34% of gates.
- **Small effects are not told apart.** A repair worth +2 is reverted in 17%
  of gates, and one costing 2 is kept in 44%.
- **Large harm is excluded.** A loss of 10 is reverted in 99%, a loss of 5 in
  85%. The cap takes 11% at −5, so the design's 95% is not reached.
- **RAR-M10's "none at 0" is the drift, not the walk.** At zero the ratio
  has no drift, but its standard deviation at the cap is larger than the
  bounds, so most gates end at one.
- **As one fixed read,** the same games revert a neutral repair in 5% and
  catch a loss of 5 in 47% and of 10 in 93%. The SPRT buys certainty about a
  loss of 5 with a third of the neutral repairs.

**The two gates, reread** (`early_stop.py` and its log beside the script;
after the run, not registered). Share of gates ending at H0 no later than
each gate did:

| true nElo | inside 2,452 games (RAR-S115) | inside 9,662 games (RAR-S114) |
|---:|---:|---:|
| −15 | 0.615 | 1.000 |
| −10 | 0.341 | 0.981 |
| −5 | 0.135 | 0.768 |
| −2 | 0.067 | 0.485 |
| 0 | 0.037 | 0.294 |
| +2 | 0.022 | 0.152 |

- **RAR-S115's early H0 is evidence of a real loss:** nine times as likely
  at −10 as at zero.
- **RAR-S114's H0 is weak:** 2.6 times as likely at −5 as at zero, and
  twice as likely at zero as at +2. It speaks against a gain of 3 or more.
  It does not show a loss of 5.

**Calibration.**
1. At a true 0, H0 0.34 ± 0.03, H1 the same, the cap 0.32 ± 0.05: hits, at
   0.343, 0.340 and 0.317.
2. At −5, H0 at least 0.93: a miss, 0.846. The forecast left out the cap,
   which ends 11% of those gates before a bound.
3. At −2, H0 0.50 to 0.70: a hit, 0.559. At +2, 0.08 to 0.18: a hit at its
   edge, 0.175.
4. Median games at 0, 7,000 to 10,000: a hit, 8,081.
5. The named failure mode was not tested beyond the wire, which agrees with
   the runner to 0.01.

## Disposition (2026-11-11)

**Observation; no rule changes here.** It is used in D.6.3's verdict (PLAN)
and in the ledger's repeated calibration lessons. Put to the maintainer, not
decided: whether a repair with a prior near zero should be gated by case 3,
which reverts a third of neutral repairs, or by a fixed read of the same
games, which reverts 5% of them and catches a loss of 5 half as often.
