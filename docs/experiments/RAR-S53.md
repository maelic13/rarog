# RAR-S53 — Paired two-arm decomposition, Rarog 2.3.1 versus Basilisk 1.9.1: identical engines, book and seed, run once …

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Paired two-arm decomposition, Rarog 2.3.1 versus Basilisk 1.9.1: identical engines, book and seed, run once at `3+0.03` and once at `-Nodes 250000`, 1T. 3,000 games in the nodes arm; the **arm difference** is the measurement, because this pair's head-to-head runs 35–45 Elo worse for Rarog than its pool rating and reading a nodes head-to-head against a clock pool rating would charge that whole matchup effect to time management. NPS equality was verified on game positions first (2.81/2.58/4.47M versus 2.87/2.45/4.55M, within 2–5% with the sign alternating), so equalizing nodes granted neither side a speed subsidy.

## Result / disposition

**Observation, decisive for cycle direction.** Clock arm −62.15 ± 9.78; fixed-node arm −65.26 ± 9.88; paired arm difference **+3.11 ± 13.51**, zero time losses. Depth at exactly equal nodes over ~158k moves per engine (`tools/pgn_depth_at_nodes.py`): Basilisk mean **13.96** / median 13 at 3,051,641 nps; Rarog mean **16.47** / median 15 at 3,223,853 nps.

## Conditional lesson

The deficit **survives** with speed and time management removed entirely: at most ~14 Elo of the −62 is speed plus TM combined, and the point estimate is ~0. Rarog reached **2.5 more plies** on the same node budget, at near-identical speed, and still lost by 65 Elo — it buys depth it cannot use by discarding width it needs. A free, falsifiable progress metric follows: re-run the depth script on a post-change fixed-node match; mean depth at 250k nodes should **fall** toward ~14 while Elo **rises**. A change that keeps the +2.5-ply lead has not fixed the over-selectivity, whatever its gate says. Method caveat: same-seed cross-match pairing bought almost nothing here (r = +0.056, CI ±13.90 → ±13.51, 2.9%), because `-games 2 -repeat` already plays each opening from both colours and the pair score has absorbed the opening imbalance by construction. Do not budget resolution on it.

## Source

`tools/sprt.ps1 -Nodes`; `tools/pgn_depth_at_nodes.py`; branch `spsa_impr` at `eaf0965`, design at `1696028`
