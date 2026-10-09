# RAR-S45 — The two verifications owed after 4.6: (a) count the safe-versus-losing quiet-check population, since …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

The two verifications owed after 4.6: (a) count the safe-versus-losing quiet-check population, since `CheckBonusLosing` had measured 0.00% even at 0; (b) measure the six-member 4.10a bundle as a SET rather than trusting the individual sizings to compose. `bench 13`, idle host.

## Result / disposition

**Both negative, and both changed a decision.** (a) `check_order_safe = 332,683`, `check_order_losing = **0**`. The split could never fire, and zero across 332k moves indicts the PREDICATE rather than chess: `see_ge(mv, 0)` is evidently trivially satisfied for a non-capturing move, so it is the wrong test for "the checker can be taken at a loss". The split is **reverted**; the census is kept and `CheckBonusSafe` survives as a 4.10 coordinate. (b) The bundle measures **6,800,242 nodes, +4.57%, EBF 2.450** - the individual effects summed to about **-17%**, so composition flipped the SIGN.

## Conditional lesson and retry trigger

Two lessons, both expensive to have learned later. **A switch that is inert when off and also inert when on is dead code, not a tunable** - and the way to tell them apart is a population counter, not a node-count delta, because a zero delta is exactly what both look like. **Individually-sized arms do not compose**: this bundle was assembled from six cheap-or-negative members and is a +4.57% HEADWIND as a set, the same shape as RAR-S34's candidate that landed dead neutral. Any bundle must be sized as a set before its bounds and cap are registered, and 4.10a's composition now has to be rebuilt from measured subsets rather than from a sum of singles.

## Source

`src/search.rs`; `src/params.rs`; Plan 4.6c, 4.10a
