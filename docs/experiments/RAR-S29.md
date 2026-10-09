# RAR-S29 — Phase-4.3a **arm A**, `EvalPruneTtMinDepth=1` — denying depth-0 entries the right to refine the main-search …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.3a **arm A**, `EvalPruneTtMinDepth=1` — denying depth-0 entries the right to refine the main-search pruning eval. Same registered design as RAR-S27, one `rarog-43a-tune.exe` both sides.

## Result / disposition

**Rejected at formal H0:** 18,436 games, Elo −3.18 ± 3.23, nElo −4.95 ± 5.02, W-D-L 4,622-9,023-4,791, Ptnml [418, 2271, 3973, 2174, 382], LOS 2.66%, LLR −2.95 crossing −2.94. The Elo interval is **[−6.41,+0.05] and narrowly includes zero**; the registered sequential verdict and one-sided evidence reject promotion, but this is not a two-sided proof of a loss. Four time losses were symmetric.

## Conditional lesson and retry trigger

Under these conditions the flat depth-0 floor did not meet its acceptance rule, so the default remains 0. RAR-S27 and RAR-S29 cannot rank values 1 and 2—their intervals overlap and value 2 never reached a formal boundary. Arm C was retired as low priority because it changes a different qsearch consumer already supported by RAR-S02, not because these games proved equivalence. Do not compile or time other work while a gate runs. Retry these floors only in the 4.10 joint fit, where consumers can move with them.

## Source

`tools/results/sprt_EvalPrune1_vs_Head_20260806_135739.*`; Plan 4.3a
