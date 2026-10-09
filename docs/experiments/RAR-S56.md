# RAR-S56 — Phase-4 step 4.7a — null-move entry contract, PREPARED AND HELD

Indexed under *3. Search and selectivity › Search-accuracy decomposition* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

*This entry had 5 cells in a table of 4 columns (Experiment and conditions, Result / disposition, Conditional lesson, Source); its parts are kept in their original order.*

## Part 1

**Phase-4 step 4.7a — null-move entry contract, PREPARED AND HELD.** Candidate branch `p47a-nmp-entry` at `76e72bb`, baseline `dev` `090dedc`. Replaces Rarog's single relaxed entry test `nmp_eval >= beta − 12·depth − 35·improving` (which at depth 8 admits nodes ~131 cp below beta) with a hard `nmp_eval >= beta` primary gate, re-homing the old margin onto raw `static_eval` as a secondary floor so both tuned parameters stay live. Measured on the 4.2 suite, 50 positions, depth 8, against the identical baseline reading.

## Part 2

**No games. Deliberately not gated.** Mechanism moved as predicted: `nmp_attempt` 16,087 → 12,652 (**−21.4%**), `nmp_cut` 3,086 → 3,013 (−2.4%), conversion **19.2% → 23.8%**, nodes +3.7%, qnodes +3.3%. `bench 13` 6,519,711 → 6,692,786, EBF 2.449 → 2.452. fmt, all-feature clippy and 259/259 release tests clean. Held because conversion reached 23.8%, not the oracle's 83.3%, so the hard gate explains only part of the divergence and the plausible effect is **3–8 nElo**. Sizing that on RAR-M10: `[3,10]` drives a true 4–6 nElo candidate to H0, and `[0,5]` needs 20k–47k games. The registered 4.7 cap is 4,000 games because the **cluster** prior is 25–60 nElo.

## Part 3

A coherent mechanism change that measurably does what it claims is still not worth a gate on its own if its plausible effect sits in the harness's dead zone. Rule 2 and cluster-discipline rule 5 caught this in preparation, before the games. The forward action is to bundle with 4.7b (move-count volume, 13.35x, the largest divergence in the phase) into one coherent selectivity candidate: the 4.3 map already establishes that the leads compete for the same quiet population, so they are one contract rather than three patches. Do not gate 4.7a alone, and do not fold it silently into a later bundle without re-measuring — its −21.4%/−2.4% split is the attribution record.

## Part 4

**REVERTED 2026-08-18 — see RAR-S58.** It was gated only inside the 4.7a+4.7c bundle, the ablation showed 4.7c reproduces that result entirely, and a zero-game curvature probe ruled out its re-homed constants as the explanation. The attribution record above stands as the measurement; the code does not. `analysis/phase4_differential_47a_depth8.txt` is retained.

## Part 5

branch `p47a-nmp-entry`; `analysis/phase4_mechanism_map.md`; PLAN 4.7
