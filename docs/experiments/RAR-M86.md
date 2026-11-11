# RAR-M86 — What the case-3 repair gate returns at small true effects

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11 before the simulation ran. The
  agent's; no engine and no game.
- **Kind:** a method read: the operating characteristic of a registered gate.
- Disposition: open — registered, not run.
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
