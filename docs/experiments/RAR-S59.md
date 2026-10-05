# RAR-S59 — Phase-4 step 4.5.3 — continuation-attribution asymmetry, measured with zero games

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 step 4.5.3 — continuation-attribution asymmetry, measured with zero games.** `update_cutoff_tables` gives a tried-and-failed quiet a malus in main, low-ply and pawn history but never in continuation history, so continuation learns only from the move that worked while the other three learn from both. Built the malus behind a default-off switch, verified the off position reproduces `bench 13` 7,467,143 exactly, then compared ordering counters on the 40-position bench corpus at stride 1.

## Result / disposition

**REJECTED on measurement.** Ordering is FLAT: first-move cutoff rate 88.04% → **88.09%**, and the rank8+ share gets slightly *worse* (0.750% → 0.793%). Meanwhile nodes fall 7.5% (7,467,143 → 6,907,848) and total cutoffs fall **9.6%** — cutoffs drop FASTER than nodes, which is the opposite of what an ordering improvement does.

## Conditional lesson

**The symmetry fix is a selectivity increase in disguise, and that is why the intuition was wrong.** Continuation history feeds `quiet_hist`, which drives two of LMP's four disjuncts and the LMR reduction, so pushing it broadly negative simply prunes more quiets. Rarog's one four-times-replicated diagnosis is that it already prunes too much — RAR-S53 (2.5 plies deeper at equal nodes and still losing), RAR-S54 (+4.06 for a blind de-selectivity shift), RAR-S55, and 4.7 itself paying **+15.56 Elo for pruning LESS**. Adopting this would have moved the engine the one direction every reading forbids, under the cover of fixing an asymmetry that looks like an omission. ⚠ Generalisation for the rest of Cluster A: a history-table change is never *only* an ordering change in this engine, because the same tables gate pruning. Measure cutoffs-per-node alongside the cutoff RATE — the rate alone would have shown nothing here. The switch was built, measured and **removed** rather than left dormant.

## Source

`src/search.rs` `update_cutoff_tables`; `tools/diag/bench_counters.py`; RAR-S53; RAR-S54; RAR-S57; PLAN 4.5.3
