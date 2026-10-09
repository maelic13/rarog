# RAR-S98 — B.7 speed pass in games — REGISTERED 2026-10-02, before any game

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.7 speed pass in games — REGISTERED 2026-10-02, before any game** (asked for by the maintainer at B.7's close). The three accepted changes (`5fe42cf`, `0d95763`, `a42fadc`) are behaviour-neutral and were accepted on NPS: +8.85%, +6.93% and +1.69%, together about +18.4%. This reads the same thing in games. `tools/test_engines/rarog-b7head-pext-pgo.exe` (`5296F2B8…`, `6872c04`) against `rarog-b7base-pext-pgo.exe` (`1256A60C…`, `dda5112`, the head before B.7, built from a frozen worktree), both clean PGO, both bench 11,171,726. Colosseum `sprt`, `sprt-default`: **`[0,3]` nElo**, cap **5,000 pairs**, UHO random, `3+0.03`, Hash 64, one thread, seed 20261020, `tools/results/b7-speed-gate`; dry run at policy. **Rule, fixed here:** H1 confirms the pass in games and nothing more is played (one gate, one read). H0 or the cap contradicts the NPS evidence: B.8 does not start until the cause is found (the clock, the prefetch under fourteen concurrent games, a behaviour difference the fingerprint does not reach). A time loss or fault by the head is investigated whatever the SPRT says. **Prediction, frozen:** +25 ± 12 Elo, between the doubling conversion (about 60 Elo a doubling, +15 Elo for 0.24 doublings) and this engine's one measured speed pass at this control (+10.35% NPS read +20.3 ± 7.1 Elo, which scales to about +35); H1 in 1,200 to 2,700 pairs; H0 probability 0.02. Maintainer-run, about 30 to 60 minutes.

## Result / disposition

**Played 2026-10-02** (`tools/results/b7-speed-gate`, 22 min 30 s, binaries at their registered hashes, host 6.4% busy, recount equal): **H1 accepted at 1,051 pairs**, LLR +2.95, **+35.5 ± 9.0 Elo** (+59.0 ± 14.9 nElo; 664-988-450, [14, 187, 484, 303, 63]); 0 time losses, 0 faults; 14 post-terminal pairs kept as evidence. Inside the games the head searched **+16.7% nodes a second** (1,948,814 against 1,669,362, pooled over 122,000 moves a side) and **+0.42 plies** (`ingame_speed.txt`; the mover is taken from the position, since 926 of 2,130 games start with Black to move). **The speed pass is confirmed in games; nothing more is played.**

## Conditional lesson

**Calibration:** +25 ± 12 predicted, +35.5 read: inside, at the upper edge; the stop came at 1,051 pairs, just under the predicted 1,200. Of the two conversions the prediction straddled, the doubling figure (60 a doubling, +15) is refuted for this engine at this control and its own earlier pass (+10.35% for +20.3 ± 7.1) is confirmed: about 2 Elo per 1% NPS, 145 Elo a doubling, over the range to +18%. An SPRT's estimate at its stop leans high; at 2,102 games no longer match is owed (one gate, one read). The gain inside fourteen concurrent games (+16.7%) is a little under the idle bench's (+18.4%), so match load did not enlarge it. A first reading of the in-game speed (+2.2%) was a parse error of the audit's own: it assigned movers by move index.

## Source

PLAN B.7; RAR-P28, RAR-P31
