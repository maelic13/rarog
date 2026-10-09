# RAR-S87 — B.4.3 categoricals, two 2,000-game reads — REGISTERED 2026-09-29, before any game

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.4.3 categoricals, two 2,000-game reads — REGISTERED 2026-09-29, before any game.** RAR-S85 provides one categorical run where the zero-game screens disagree; they do for both quiescence switches (on: WAC at 100k 241 and 242 against the arm's 236, three canaries lost each), and RAR-S86's tune would otherwise fit fourteen coordinates around two pinned, unmeasured switches. Each is one fixed match on **one tune build**, `tools/test_engines/rarog-b43-tune.exe` (sha256 `8F84CB1E…`, 10,226,874), the alternative as side A by UCI option against the arm's defaults as side B (`-CategoricalTuneBuild`): `CoreQsEvasionPrune=1`, seed 20261009, `tools/results/b43-cat-evasion`; `CoreQsNoisyHistory=1`, seed 20261010, `tools/results/b43-cat-history`. Colosseum `match-fixed` policy (`3+0.03`, Hash 64, one thread, 14 slots, UHO random, no adjudication), `-ExpectBench 10226874,10226874`; both dry runs resolved to policy with the option on side A only. **Reading rule (RAR-S80's, fixed here):** a switch is adopted only if its run reads **≥ +5 Elo with the 95% lower bound above −5**; between −5 and +5 it stays off and is closed as undecidable at this budget; at or below −5 it is closed off. Effects are never summed: if both are adopted, one further 2,000-game run of both against the defaults precedes the tune. An adopted switch is set to 1 in `fixed_b43.json` (regenerated, re-hashed, dry-run) before block 1 and carried by B.4.4's candidate, with its three lost canaries recorded as the cost. **Predictions, frozen:** evasion pruning **+2 ± 10 Elo**, adopted 0.3; quiescence history **0 ± 10 Elo**, adopted 0.2. Maintainer-run, about 21 minutes each.

## Result / disposition

**Played 2026-09-29** (both on the registered binary, idle host, 0 faults, no abnormal game; the evasion read's first start was stopped at 22 games and restarted whole): `CoreQsEvasionPrune=1` **+7.8 ± 9.4 Elo** (+12.7 ± 15.2 nElo; 528-989-483, [26, 239, 441, 252, 42]), lower bound −1.6: **adopted**; `CoreQsNoisyHistory=1` **−4.7 ± 9.4 Elo** (−7.6 ± 15.2 nElo; 507-959-534, [44, 239, 444, 246, 27]): stays off, closed as undecidable at this budget and left a switch for B.6. One adopted, so no combined run is owed.

## Conditional lesson

**Calibration:** evasion pruning predicted +2 ± 10, adoption 0.3, read +7.8: inside, the less likely branch; history predicted 0 ± 10, adoption 0.2, read −4.7: inside. The canary rule's zero-game default was overturned by games for one switch and confirmed for the other.

## Source

PLAN B.4.3; RAR-S85 (Q6, Q7); RAR-S80 (the rule); RAR-S86; B.8 removed the switch: `analysis/b8_removed_2026-10-03.md`
