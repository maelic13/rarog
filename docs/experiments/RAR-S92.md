# RAR-S92 — B.5.4 optimism, one 2,000-game categorical read — REGISTERED 2026-10-01, before any game

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.5.4 optimism, one 2,000-game categorical read — REGISTERED 2026-10-01, before any game.** `CoreOptimism=1` (the corrected evaluation leans toward the root's running average score by `52·avg/(|avg| + 92)` for the root's side and its negative for the other, weighted `(1200 + material)/21000`; Reckless's form at the 0.457 scale ratio; `d754577`, default off and behaviour-neutral) as side A by option against the defaults as side B, on **one tune build**, `tools/test_engines/rarog-b54-tune.exe` (sha256 `88BAAA29…`, `cargo build --release --features tune` at `d754577` clean, bench 11,171,726 off and 14,387,454 on). Colosseum `match-fixed` (`3+0.03`, Hash 64, one thread, 14 slots, UHO random, no adjudication), 2,000 games, seed 20261015, `tools/results/b54-cat-optimism`, `-ExpectBench 11171726,11171726`; the dry run resolved to policy with the option on side A only. **Reading rule (RAR-S80's):** ≥ +5 Elo with the 95% lower bound above −5 promotes the card to `READY_FOR_IMPLEMENTATION` and a `[0,3]` SPRT of the switch baked on (PGO) against the head is registered next, the two coordinates joining B.6's surface; between −5 and +5 it closes `NO_CHANGE` at this budget and the switch stays 0 for B.8; at or below −5 it closes `NO_CHANGE`. Never summed with RAR-S91. **Zero-game, recorded only:** on, reverse futility at depth 1–3 −9%, razoring −22%, ProbCut −10%, history pruning +20%, aspiration failures −25%, WAC at 100k 234 against 232 (`analysis/b54_research_2026-10-01.md`). **Prediction, frozen:** **+1 ± 10 Elo**, adopted 0.2, at or below −5 0.25. Maintainer-run, about 21 minutes.

## Result / disposition

**Played 2026-10-01** (`tools/results/b54-cat-optimism`): `CoreOptimism=1` **−5.6 ± 9.4 Elo** (−9.0 ± 15.2 nElo; 496-976-528, [44, 232, 470, 220, 34]; 2,000 games, 0 faults, the registered binary, idle host). At or below −5: **card B.5.4 closes `NO_CHANGE`**; the switch stays 0 for B.8 to remove with its coordinates.

## Conditional lesson

**Calibration:** predicted +1 ± 10, adoption 0.2, at or below −5 0.25; read −5.6: inside the interval, on H2's side. The donors' gain did not transfer to a classical evaluation fitted without the bias.

## Source

PLAN B.5.4; RAR-S80 (the rule); RAR-S91; B.8 removed the switch: `analysis/b8_removed_2026-10-03.md`
