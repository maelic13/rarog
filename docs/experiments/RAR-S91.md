# RAR-S91 — B.5.1 card (b), draw-score randomisation, one 2,000-game categorical read — REGISTERED 2026-09-30, before any …

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.5.1 card (b), draw-score randomisation, one 2,000-game categorical read — REGISTERED 2026-09-30, before any game.** `CoreDrawJitter=1` (a draw met below the root in the core search and quiescence scores `nodes % 5 − 2`, Reckless's form, instead of 0) as side A by option against the defaults as side B, on **one tune build**, `tools/test_engines/rarog-b51-tune.exe` (sha256 `529E9525…4512`, `cargo build --release --features tune` at `edc69ad` clean, bench 11,171,726 / EBF 2.512 off and 11,176,453 / EBF 2.521 with the switch on, both read from this binary) (`-CategoricalTuneBuild`). Colosseum `match-fixed` (`3+0.03`, 20 ms margin, Hash 64, one thread, 14 slots, UHO random, no adjudication), 2,000 games, seed 20261014, `tools/results/b51-cat-drawjitter`, `-ExpectBench 11171726,11171726 -ExpectRevision edc69ad`; the dry run resolved to policy with the option on side A only. **Reading rule (RAR-S80's, fixed here):** ≥ +5 Elo with the 95% lower bound above −5 promotes the card to `READY_FOR_IMPLEMENTATION` and a `[0,3]` SPRT of the switch baked on (PGO) against the head is registered next; between −5 and +5 it closes `NO_CHANGE` at this budget (insufficient evidence, not refutation) and the switch stays 0 for B.8 to remove; at or below −5 it closes `NO_CHANGE`. Never summed with B.5.4's read. **Prediction, frozen:** **+1 ± 10 Elo**, adopted 0.2, at or below −5 0.1. The read cannot resolve an effect of the donors' likely size; it catches a large gain or a clear loss. Maintainer-run, about 21 minutes.

## Result / disposition

**Played 2026-09-30** (`tools/results/b51-cat-drawjitter`): `CoreDrawJitter=1` **−1.2 ± 9.4 Elo** (−2.0 ± 15.2 nElo; 520-953-527, [37, 237, 455, 238, 33]; 2,000 games, 0 faults, the registered binary, idle host). Between −5 and +5: **card (b) closes `NO_CHANGE`** at this budget, which means the evidence is insufficient, not that the idea is refuted. The switch stays 0 and is B.8's to remove. B.5.1 is closed.

## Conditional lesson

**Calibration:** predicted +1 ± 10, adoption 0.2; read −1.2: inside the interval, the expected branch. **Evidence behind the registration:** draws are 0.28% of the bench's visited nodes; in RAR-M63's 40,000 games, 19.5% end by repetition or rule 50, and after Rarog's root printed 0 on three consecutive moves the games went 1,554 won and 1,558 lost, so no blindness defect is measured; the ledger's perturbation rows give no prior (RAR-S67's stop note).

## Source

PLAN B.5.1; RAR-S80 (the rule); `analysis/b51_research_2026-09-30.md`; `tools/results/b51-20260930/`; B.8 removed the switch: `analysis/b8_removed_2026-10-03.md`
