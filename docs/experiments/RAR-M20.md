# RAR-M20 — Board audit and native three-engine comparison, 2026-09-05

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Board audit and native three-engine comparison, 2026-09-05.** Rarog, the development head of that day; Basilisk d734766; Reckless 91b56c2 plus the complete benchmark-only adapter. Native optimized non-PGO builds, Ryzen 9 5950X, affinity mask 4; three cyclic rounds, 150ms warmup + 11x150ms per workload.

## Result / disposition

**Basilisk faster in all five comparable workloads.** Median M ops/s Rarog/Basilisk/Reckless: legal moves **447.131/642.646/339.844**; captures **98.204/120.138/61.597**; generation+make/unmake **42.521/55.031/23.494**; perft **273.741/382.726/177.944**; two-ply simulation **351.809/513.537/246.626**. Native SEE **46.676/58.814/39.722** is NOT comparable: value vectors/contracts differ. Confirmed SEE king-exchange defect (-400/true instead of -300/false at zero), Unicode move parser panic, fullmove debug overflow/release wrap; no Rarog fix or games in this audit.

## Conditional lesson and retry trigger

Owner **4.11b**, correctness then HCE profile and bounded optimization. Every Basilisk round beat every Rarog round, which beat every Reckless round, in the five comparable columns; active desktop load 6.25–9.17%, substantial scatter in some cells, no small-gain or Elo inference. Reckless uses **NullBoardObserver**, so this excludes NNUE arithmetic and does not isolate NNUE-related board cost. Preserve raw data for **5.2.1**, measure move-event/scaffold costs at **5.2.5/5.3.4**, actual-network update/inference at **6.4.3**. Keep 4.15 production fitting separate from 4.11b.6's neutral value injection.

## Source

`analysis/board_audit_2026-09-05.md`; `analysis/board_benchmark_recipe_2026-09-05.md` embeds exact builds, complete adapter patch, source/binary hashes, runner and nine raw outputs; machine-readable `analysis/artifacts/board-audit-20260905/manifest.json`. Archived binaries remain at `D:/chess/results/board-audit-20260905/binaries/`; recipes and results do not depend on a candidate branch.
