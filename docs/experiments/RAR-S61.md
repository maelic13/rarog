# RAR-S61 — Phase-4 cluster 4.5 (A) — REGISTERED, NOT YET RUN

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Phase-4 cluster 4.5 (A) — REGISTERED, NOT YET RUN.** Candidate: `dev` at `c399435` (4.5.1 per-ply context, 4.5.2 named picker stages, 4.5.3 continuation key plus the ProbCut piece-desync fix, 4.5.4 prior-reduction authority at 512/1024 ply). Baseline: the pre-Cluster-A head `36dad5f`, fingerprint **6,922,439 / EBF 2.451** (confirmed by the baked manifest — an earlier attempt baked `aaa715a`, which predates the 4.7 merge and benched 6,519,711; the manifest's recorded fingerprint is what caught it). Candidate **7,587,235 / EBF 2.477**, +9.6% nodes. Final-PGO both arms, `3+0.03`, 1T, 64 MB, paired UHO, RAR-M13 two-sided adjudication. **Registered bounds `[3,10]` nElo, cap 16,000, prior 5–20, all fixed before any games.**

## Result / disposition

**UNRESOLVED at the 16,000-game cap — NOT promoted.** Elo **+4.50 ± 3.50**, nElo **+6.92 ± 5.38** (95% CI [1.54, 12.30]), LOS 99.41%, W-D-L 4,209-7,789-4,002, 50.65%, draw 41.83%, PairsRatio 1.07, Ptnml(0-2) [324, 1926, 3346, 2027, 377]. **LLR 0.39 of ±2.94 after the full budget.** One timeout per side, symmetric, no crashes or illegal moves. **RAR-M10 predicted this to three decimals:** the bracket midpoint is 6.5 nElo and the candidate landed at 6.92, 0.42 away, so drift ≈ 8.3e-6 × 7 × 0.42 × 16,000 = **0.390** against an observed **0.390**. The effect is almost certainly real — LOS 99.41%, CI excluding zero — and simultaneously unable to resolve, because it sits within half an nElo of the one value `[3,10]` cannot decide.

## Conditional lesson

**Prior re-derived from 15–45 down to 5–20, because most of Cluster A turned out to be unnecessary rather than valuable.** PLAN sized 15–45 for a full ordering/history/LMR rework; RAR-S52/S55 had already refuted the ordering premise, and RAR-S60 then rejected four of the six planned per-ply fields. Only two strength-bearing changes shipped: the ProbCut piece fix (correctness repair of unknown sign — continuation history was trained on a mismatched piece/square pair) and prior-reduction authority (de-selectivity, the direction that paid +15.56 at 4.7). ⚠ **A fit was considered and deferred, the first time the condition was actually met.** The curvature probe leaves `cut/node` monotone but first-move cutoff peaks at `LmrPriorReductionAdj=768` (88.41 pct) between 512 (88.18) and 1024 (88.27) — a real interior optimum, where 4.7's surface was flat. 512 is kept anyway: 768 costs **+17 pct nodes** for +0.23 points of a proxy RAR-S59 proved can mislead, the default was chosen ON this sweep rather than inherited from an older structure, and PLAN 4.10 owns consolidation tuning across accepted clusters, so a cluster-local SPSA would duplicate it. **The curvature evidence is handed to 4.10.**

## Source

`tools/test_engines/rarog-45{base,cluster}-pext-pgo.exe`; RAR-S57; RAR-S59; RAR-S60; RAR-M10; PLAN 4.5.5, 4.10; record: `analysis/ledger_records_2026-09-14.md`, RAR-S61 (Search and selectivity)
