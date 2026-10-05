# RAR-S96 — `CoreIirMinDepth`, two 2,000-game categorical reads — REGISTERED 2026-10-01, before any game

Indexed under *3. Search and selectivity › Registered, open* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**`CoreIirMinDepth`, two 2,000-game categorical reads — REGISTERED 2026-10-01, before any game** (maintainer decision the same day; B.6's carried coordinate). Internal iterative reduction searches a node one ply shallower when it has no TT move (or, off the PV, only a shallow entry); `CoreIirMinDepth` is the remaining depth from which it applies, 4 on `2..=10`. It fires at 15% of interior nodes and has never been read by games on this search: every tune excluded it as a discrete threshold, and B.3's IIR policy categorical was not built. One tune build on both sides, `tools/test_engines/rarog-b6iir-tune.exe` (`D90F59C8…`, `488d34a`, `pext-tune`, bench 11,171,726), `-CategoricalTuneBuild`. Colosseum `match`, `match-fixed`, 2,000 games each, `3+0.03`, Hash 64, one thread, UHO random. **Read 1:** `CoreIirMinDepth=3` against the default, seed 20261017, `tools/results/b6-cat-iir3`. **Read 2:** `CoreIirMinDepth=6` (the donor's threshold) against the default, seed 20261018, `tools/results/b6-cat-iir6`. Wire (`tools/results/b6-20261001/iir_wire.txt`): the option is advertised with its range, the default set by option reproduces 11,171,726, and `bench 13` moves at each value (3: 0.84×, 6: 1.10×; 2: 0.78×, 10: 4.30×); a wire check, not a forecast. **Rule, fixed here (RAR-S80's):** a read of +5 Elo or more with its 95% lower bound above −5 is a direction. A direction makes the card `READY_FOR_IMPLEMENTATION`: the value baked into a PGO build and a `[0,3]` SPRT against the head, registered separately (the larger read first if both qualify). No direction closes it `NO_CHANGE` at this budget: 4 stays and no third value is read. **Predictions, frozen:** 3 reads −1 ± 10 Elo, a direction with probability 0.12, −5 or worse 0.30; 6 reads −2 ± 10, a direction 0.10, −5 or worse 0.35. Low confidence: the only evidence is node counts, which have pointed the wrong way twice on this search. Maintainer-run, about 20 minutes each.

## Result / disposition

**Played 2026-10-01** (2,000 games each, 0 faults, every game a normal termination, recounts equal, binary at its registered hash, host 8.5% and 0.9% busy). **Read 1, `CoreIirMinDepth=3`:** **−4.5 ± 9.3 Elo** (−7.4 ± 15.2 nElo; 498-978-524, [34, 253, 452, 227, 34]). **Read 2, `CoreIirMinDepth=6`:** **+3.0 ± 9.2 Elo** (+4.9 ± 15.2 nElo; 534-949-517, [28, 240, 454, 243, 35]). Neither is a direction by the rule. **Closed `NO_CHANGE` at this budget: 4 stays, no third value is read.**

## Conditional lesson

**Calibration:** both reads inside their intervals (−1 ± 10 against −4.5; −2 ± 10 against +3.0, the opposite sign of the centre, inside the stated chance); no direction, as 0.12 and 0.10 expected. The surface between 3 and 6 is flat at this resolution; the two reads differ by 7.5 ± 13.1 Elo, which is not a slope. The node counts ran against the reads again: 0.84× the nodes read lower, 1.10× read higher. **Retry:** in C.10's tune as a switch read before block 1, or when a change moves IIR's firing rate or the singular extensions it brakes by 10% or more.

## Source

PLAN B.6; `analysis/b6_research_2026-10-01.md`
