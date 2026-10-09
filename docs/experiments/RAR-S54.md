# RAR-S54 — Blind uniform 15% shift of the whole selectivity surface toward **less** pruning, on a throwaway probe …

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Blind uniform 15% shift of the whole selectivity surface toward **less** pruning, on a throwaway probe branch, versus the 2.3.1 head. Both arms final-PGO with clean manifests and the same pinned rustc; bench 5,173,540 versus 6,373,363. `3+0.03`, 1T, paired UHO.

## Result / disposition

**Positive, deliberately stopped at LLR 1.68 of 2.94 and recorded as a STOP, not an H1.** +4.06 ± 3.71 Elo, nElo +6.27 ± 5.72, LOS 98.42%, 14,196 games, draw ratio 41.97%. Zero time losses, timeouts, crashes or illegal moves despite **+23.2% nodes** — the width-for-depth trade at a clock TC was the live forfeit risk and it did not materialise. Not a bake candidate; the values were registered as such before the run.

## Conditional lesson

An **untuned, uniform, blind** de-selectivity shift beat the fitted values, which confirms the over-pruning diagnosis from the opposite direction to RAR-S53. The estimate was stable in +2.0…+4.2 over the final 7,000 games and nElo was about 2× `elo1`, so it was the width of the interval, not the centre, that kept the bound uncrossed. This licenses a structural selectivity rework with its own refit; it does **not** license shipping a uniform scalar, and the magnitude is a stopped point estimate, never a release claim.

## Source

`tools/test_engines/rarog-p100*-pext-pgo.exe` and their JSON manifests; **reconstruction recipe below this table** — `d472f6c` was a docs-only commit and the probe's real source `7693010d` was dangling, so the recipe replaces both; record: `analysis/ledger_records_2026-09-14.md`, RAR-S54 (Search and selectivity)
