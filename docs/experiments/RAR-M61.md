# RAR-M61 — Colosseum CLI against fastchess on the same fixed match — the parity read no stopping rule biases …

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Colosseum CLI against fastchess on the same fixed match — the parity read no stopping rule biases, 2026-09-17/18, recorded 2026-09-21 from its artifacts, zero new games.** Arms: `rarog-b22core-pext-pgo.exe` (sha256 `C51476EB…`) against `rarog-b22base-pext-pgo.exe` (`CDAF2AE5…`). Conditions on both: fixed length, no stop rule, `3+0.03`, 20 ms margin, Hash 64, one thread, UHO_Lichess_4852_v1 in random order, no adjudication, concurrency 14 on pinned physical cores — Colosseum `--placement auto` with one core of headroom, fastchess `-use-affinity 2,4,…,28`, which `tools/diag/colosseum_parity.py` confirms resolve to the same fourteen cores. fastchess: 2,000 games, seed 818905001 (`sprt_b22core_vs_b22base_20260918_131307`, 22m07s). Colosseum: three 2,000-game runs whose resolved configurations are identical in every recorded field, including the seed 20260918 and the config hash `701ae452…` (`colosseum-fixed-b22-5`, `colosseum-slotpool-b22`, `colosseum-latency-b22`), plus two 1,000-game runs at concurrency 7 (`…-disjoint7`, `…-shared7`). All recounted from the PGNs.

## Result / disposition

**The instruments agree, and the spread among Colosseum's own runs is as wide as the gap between the instruments.** fastchess **+54.29 +/- 11.13 Elo, +75.56 +/- 15.23 nElo**, Ptnml [35, 173, 362, 307, 123], **zero time forfeits**. Colosseum, same arms, 2,000 games each: `fixed-b22-5` +62.15 +/- 10.95 / +88.27 +/- 15.23, [29, 160, 363, 324, 124], 15 forfeits; `slotpool-b22` +65.92 +/- 10.97 / +93.72 +/- 15.23, [27, 163, 340, 348, 122], **zero forfeits**; `latency-b22` +55.71 +/- 10.84 / +79.62 +/- 15.23, [33, 162, 371, 322, 112], 14 forfeits. At concurrency 7 over 1,000 games: `disjoint7` +67.55 +/- 14.46 (3 forfeits), `shared7` +58.93 +/- 15.35 (4). Every interval overlaps every other. The three 2,000-game runs share a seed and therefore the same openings, so their 10.2-Elo spread is the engines' own timing nondeterminism, not opening sampling.

## Conditional lesson and retry trigger

**The forfeit difference has no recorded cause and must not be attributed.** The three runs' own records are identical down to the configuration hash and all report `product_version` 0.1.0, so nothing in the evidence identifies which Colosseum build ran which — which is the argument for pinning the runner by SHA-256 (`tools/colosseum/colosseum.pin.json`) rather than by a version string. Do not read any of these Elos as a strength result: one binary pair, gated elsewhere. The parity claim is the overlap, and its resolution is about +/-11 Elo per 2,000 games.

## Source

PLAN B.2.6.2; RAR-M48 (why games are pinned at all); `tools/results/sprt_b22core_vs_b22base_20260918_131307.{pgn,log,manifest.txt}`; `tools/results/colosseum-fixed-b22-5/`, `colosseum-slotpool-b22/`, `colosseum-latency-b22/`, `colosseum-fixed-b22-disjoint7/`, `colosseum-fixed-b22-shared7/`; `tools/diag/colosseum_recount.py`
