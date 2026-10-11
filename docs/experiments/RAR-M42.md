# RAR-M42 — 4.11b.18 endgame evidence refresh after the accepted board head, COMPLETE 2026-09-09; section 4.11b CLOSED

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**4.11b.18 endgame evidence refresh after the accepted board head, COMPLETE 2026-09-09; section 4.11b CLOSED.** Arms are the binaries RAR-E15 gated (the candidate, bench 7,601,220, against the section-entry baseline, bench 6,901,489); nothing rebuilt, every instrument node-budgeted and seeded so none of it depends on host load.

## Result / disposition

**Layer 1 clean, floors PASS both arms, 4.12 order verified UNCHANGED.** `endgame_truth.py` over 19 families x 100 positions at 60,000 nodes produced cohort digest `fe4866045506636f...` on both arms, matching the registered floors, and **theory verdicts are identical on every family — no clean win newly discarded**, which is the absolute veto. Floors: the 4.11 head reproduces the registered aggregate exactly (0.9300 -> 0.9300, 0 reports), the accepted head reads 0.9300 -> **0.9336** (+0.4 SE) with one non-blocking report. Eleven families moved both ways; largest **KRP-KR conversion 0.9178 -> 0.9726** (the order's top family at 10.04% occurrence) and **KQ-KR dtz +2.7 SE**. **Order rederived**: the 4.11 head reproduces `endgame_ranking_v2.json` across all twenty families exactly, and the accepted head equals it.

## Conditional lesson

**Conversion alone would have produced a false negative.** Its four families are bare-king and came back byte-identical; over the frozen 83-position corpus the split is **34.5% (19/55) of both-sides positions differing against 0.0% (0/28) bare-king**, because SEE fires only where captures exist. **An assumption was caught by checking**: `endgame_measurement_layers.md` calls drawn-share bias "static", but `endgame_drawn.py` takes `--engine` and searches every position, so reusing it on that reading would have been reuse on a false premise. **A parameter error was caught by requiring self-reproduction**: the first rederivation used `--occurrence-scope all` and disagreed with registered v2, which records `Rating Tournament [engine], 10,000 games` — a rederivation that cannot reproduce its own registered baseline is a broken instrument until proven otherwise. **Floors were NOT updated**: KQ-KR qualifies as a ratchet candidate but KRP-KB fell 2.2 SE, and moving floors in the same commit as the change that moved them is forbidden. **Owed:** KRP-KB win-preserving 0.9990 -> 0.9949, reported and non-blocking, owner 4.12.6, blocking past 3 SE. **Not done and stated:** reference-results and tree-occurrence artifacts were held constant to isolate the census effect and are not established as current. No games, no Elo.

## Source

`analysis/endgame_refresh_2026-09-09.md`; `tools/diag/endgame_drawn_census_v2.json`; `tools/diag/endgame_truth_baseline_v2.json`; RAR-E15; RAR-M15; RAR-M24
