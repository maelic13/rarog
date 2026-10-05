# RAR-M62 — The Colosseum tune shape — 15 slots and 30 games per iteration, measured 2026-09-18, recorded 2026-09-21 from …

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**The Colosseum tune shape — 15 slots and 30 games per iteration, measured 2026-09-18, recorded 2026-09-21 from its artifacts, zero new games.** Three 60-iteration SPSA runs of the registered `b23core` surface on `rarog-b23core-tune.exe`: `3+0.03`, Hash 64, one thread, `MultiPV 1`, `r_end` 0.0031, UHO_Lichess_4852_v1 in random order, no adjudication, placement auto with one physical core of headroom on the 16-core host. `colosseum-verify-spsa`: 14 slots x 32 games = 1,920 games, seed 60. `colosseum-spsa-15x30` and `colosseum-spsa-15x30-s`: 15 slots x 30 games = 1,800 games, seed 1530.

## Result / disposition

**15 x 30 runs about 1.65x the games per hour of 14 x 32.** 14 x 32: 42 s/iteration, **2,745 games/hour**, 3,998 s. 15 x 30: 24 s/iteration, **4,517 games/hour**, 1,434 s; the repeat 22 s/iteration, **4,910 games/hour**, 1,320 s. No engine fault and no time loss in any of the three. **Most of the gap is wave packing, and that part is arithmetic rather than a measurement:** 32 games over 14 slots needs three waves with a four-wide tail, 30 over 15 needs exactly two full waves, which predicts 1.5x before the 32/30 game count is applied; the residue is not separately measured. **Registered shape for the next tune (PLAN B.2.6.2): 15 slots, 30 games per iteration, budget stated in games** — 5,000 iterations is 150,000 games, about 31 to 33 hours at these rates. The surface, steps, horizon and gate stay with B.2.7's own registration (RAR-S78).

## Conditional lesson and retry trigger

A tune's throughput is set by how its mini-match divides into slots, not by the slot count alone: a games-per-iteration that is not a multiple of the slot count pays for a partly idle final wave on every iteration. Check that division before registering a horizon. Re-measure if the slot count or the host's core count changes.

## Source

PLAN B.2.6.2, B.2.7; RAR-S75, RAR-S78; `tools/results/colosseum-verify-spsa/`, `tools/results/colosseum-spsa-15x30/`, `tools/results/colosseum-spsa-15x30-s/`
