# RAR-S90 — B.5.5, the aspiration surface read by games — REGISTERED 2026-09-30, before any game

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.5.5, the aspiration surface read by games — REGISTERED 2026-09-30, before any game.** After RAR-S89 rejected one-retry aspiration at −18.3 ± 6.9 Elo, the loop's own coordinates (fitted for the 2.3 search, in no tune since) are read by games, not by nodes. **Reads 1 and 2**, one tune build on both sides, `tools/test_engines/rarog-b5-tune.exe` (sha256 `B608DF61…`, `cargo build --release --features tune` at `6a8108a` clean, bench 11,171,726), the alternative as side A by option (`-CategoricalTuneBuild`), Colosseum `match-fixed` at `3+0.03`, 2,000 games each: `AspirationDelta=12` (seed 20261011, `tools/results/b5-cat-delta12`) and `AspirationDelta=45` (seed 20261012, `tools/results/b5-cat-delta45`). **Read 3**, the registered retry of RAR-S89 at a slower control: `rarog-b5gate-pext-pgo.exe` (`EAD91C16…`, 6,192,452) against `rarog-b5head-pext-pgo.exe` (`2D263E6F…`, 11,171,726), `match-fixed-ltc` (`10+0.1`), 2,000 games, seed 20261013, `tools/results/b5-ltc-mf1`. All three dry runs resolve to policy. **Rule (RAR-S80's):** a read of ≥ +5 Elo with the 95% lower bound above −5 is a direction; a direction in read 1 or 2 justifies a four-coordinate tune of the loop (`AspirationDelta`, `AspGrowthPct`, `AspGrowthHighPct`, `AspGrowthAdd`) from the current values, registered separately; none closes the aspiration question `NO_CHANGE` at this control. Read 3 is reported as a time-control dependence and changes no default by itself. **Predictions, frozen:** delta 12 **+2 ± 10 Elo**, a direction 0.25; delta 45 **−6 ± 10**, a direction 0.1; one retry at `10+0.1` **−8 ± 10**, at or above +5 0.15. Maintainer-run: about 21 minutes each for reads 1 and 2, about 70 minutes for read 3.

## Result / disposition

**Played 2026-09-30** (2,000 games each, 0 faults, hashes as registered, host idle): `AspirationDelta=12` **+2.1 ± 9.2 Elo** (522-968-510, [30, 232, 468, 236, 34]), not a direction; `AspirationDelta=45` **−25.4 ± 9.2** (445-964-591, [47, 284, 456, 194, 19]); one retry at `10+0.1` **−18.3 ± 8.5** (434-1027-539, [24, 280, 488, 193, 15]), the same loss as at `3+0.03`. **No direction: the aspiration question closes `NO_CHANGE`** and no tune is registered; the surface falls steeply toward wider or sooner-opened windows and is flat toward narrower ones. B.5's cluster verdict is `NO_CHANGE`.

## Conditional lesson

**Calibration:** delta 12 predicted +2 ± 10, read +2.1: hit; delta 45 predicted −6 ± 10, read −25.4: sign right, magnitude missed fourfold; one retry at `10+0.1` predicted −8 ± 10, read −18.3: sign right, at the interval's edge. The loss does not shrink with depth, so it is not a time-control trade.

## Source

PLAN B.5.5; RAR-S89; RAR-S80 (the rule); `analysis/b5_research_2026-09-30.md`
