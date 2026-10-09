# RAR-S95 — B.6 go/no-go from the tune journals — zero games, 2026-10-01. Rule and predictions were frozen in …

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.6 go/no-go from the tune journals — zero games, 2026-10-01. Rule and predictions were frozen in `tools/results/b6-20261001/FROZEN.md` before the first run, not in this ledger: a process miss.** Question: do the finished tunes' journals show the joint surface still carrying gradient? Statistic: per coordinate `z = net travel / sqrt(sum of squared steps)`, variance 1 without a gradient because every SPSA step carries a fair random sign; per run `mean(z²)` against a 2,000-draw sign-redraw null (`journal_read.py`; wires: complete journals, every step equal to the journal's schedule times the difference, last centres rounding to the tuned values). Journals: `b27all-spsa` (RAR-S78), `b33m-spsa-1` and `-2` (RAR-S83), `b43-spsa-1` (RAR-S86). **Rule, frozen:** a first joint block is justified when RAR-S78's second half reads above its null's 95% point on the coordinates no later tune touched, or at least three RAR-S78 coordinates re-fitted after a cluster landed read `\|z\| ≥ 2` in the block that re-fitted them; neither closes B.6 `NO_CHANGE` unless other evidence is named. **Predictions, frozen:** RAR-S78 whole run above 2.0 (0.8); its second half on the untouched coordinates above the null (0.6); B.3.3 block 1 above its null (0.6), block 2 not (0.6); `QsFutilityMargin` `z ≥ 3` (0.7) and the block without it inside its null (0.6); at least three shared coordinates (0.4).

## Result / disposition

**Read:** RAR-S78 whole 1.62 (null 95% 1.27, p 0.0005), first half 1.48 (p 0.004), second half 0.99 (p 0.52); on the 74 untouched coordinates the second half 1.02 against 1.28: **condition 1 not met.** RAR-S83 block 1 1.12 (p 0.28), block 2 0.99 (p 0.51); RAR-S86 block 1 1.46 (p 0.11), 0.68 without `QsFutilityMargin` (`z` +3.41). Shared coordinates: one of eight (`CoreLmrQuiet` +2.57 in B.3.3's block 1; the frozen text miscounted nine): **condition 2 not met.** Beside the rule: consumers' firing rates per interior node moved 10% or more since RAR-S78 for quiet futility pruning (1.42), history pruning (1.14), hindsight increase (0.85), LMR deeper re-search (1.24) and the correction residual (1.16) (`rate_diff.py`, three stride-1 dumps). **Disposition: `NO_CHANGE` recommended** (no joint tune before C; the whole-surface re-fit goes to C.10), the remainder being hours against Elo. **Closed 2026-10-01 by the maintainer: skipped; C.10 is the joint search SPSA, after the classical evaluation.**

## Conditional lesson

**Calibration:** two hits on `QsFutilityMargin`, block 2 hit; RAR-S78 whole (1.62 against 2.0), its second half and B.3.3 block 1 missed: the gradient was spent by RAR-S78's half and I expected more to remain. Instrument: the rule was frozen before the read's power was checked. It sees an effect the size of RAR-S78's and misses one half that size, and RAR-S77 had already measured +4.43 ± 2.90 Elo from a tail with no full-step mover, so "neither" excludes another RAR-S78, not a gain of a few Elo. The firing-rate check selects a surface and has not forecast movement (0 of the 3 consumers it added to RAR-S86 moved). **Retry:** at C.10; on a game read showing a direction on a carried coordinate; on a B.9 deficit attributed to a named family.

## Source

PLAN B.6; `analysis/b6_research_2026-10-01.md`
