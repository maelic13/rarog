# RAR-S117 — RAR-S116's read with its wire restated: D.6.4's consumers and the kept move's ordering on replayed gate games

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, after [RAR-S116](RAR-S116.md)'s
  wire failed and before any deciding count of either read was looked at. The
  read is the agent's (deterministic, no games).
- **Kind:** research and diagnosis after a rejected repair. No game, no gate.
- Disposition: no-change — read 2026-11-11: with the consumers held the kept move
  needs 1.022 of the best-scoring move's nodes; D.6.4 closes `NO_CHANGE`.
- **Baseline:** `d6` at `fbb9a67`, engine behaviour of `ee0b3e4`, bench
  13,038,236 / EBF 2.540.

## Why a new registration

RAR-S116's wire asked the replay of arm 0 to reproduce the games' mean depth
within 0.10. It read 0.146 under, so that read is void by its own rule:

| 60 games, moves without a mate score | replay | the games | under by |
|---|---:|---:|---:|
| arm 0 on the baseline's moves | 12.954 | 13.100 | 0.146 |
| arm 4 on the candidate's moves | 13.808 | 13.933 | 0.125 |
| the candidate above the baseline | 0.854 | 0.833 | |

- **The tolerance was the error.** It was set from a six-game smoke that
  happened to read 0.002.
- **The offset has a stated cause.** RAR-S116 said the game records the last
  `info` line's nodes and the engine searched on, so a replay's table is
  poorer than the game's. That lowers both arms alike.
- **The instrument is only ever read as a difference** between arms on the
  same positions and node counts, and the difference the games measured is
  reproduced within 0.02.

A level tolerance was the wrong wire. It is restated here, before any
deciding count is known, and nothing else changes.

## What is unchanged, by reference to RAR-S116

The question, the competing explanations, the arms, the patch
(`analysis/arm_patches/d64-rar-s116-screen.patch`), the executable (SHA-256
`ce249878…`), the replay script (SHA-256 `1a02dff6…`), the 60 games, the
deciding quantity *r*, the three rules, the quantities reported without
deciding, and predictions 2 to 8. Prediction 1, that the wire passes at 0.85,
was a miss and stays one.

## Wire (restated)

- The replay's depth difference, arm 4 on the candidate's moves less arm 0 on
  the baseline's, is within 0.10 of the games' own difference over the same
  moves. **Passed:** 0.854 against 0.833.
- Each arm's level is within 0.25 of the games'. **Passed:** 0.146 and 0.125
  under.
- Every search dumps each counter exactly once, and the arms search the same
  moves. The script asserts both.

## Exposure, stated

Seen: RAR-S116's exposure, and its wire log, which holds the six numbers
above and the share of moves replayed to the game's depth (34% and 33%) and
move (68% and 66%).

Not looked at: any replay count of the eight arms, the collect's log, and the
report. RAR-S116's run wrote them while this was being registered. The
searches are deterministic, so a rerun would write the same files, and this
read takes them as they are.

## Rules and predictions

RAR-S116's, unchanged: `READY_FOR_IMPLEMENTATION` if *r* ≤ 0.98 with the
first-move share at fail-low entries not lower; `NO_CHANGE` if *r* ≥ 0.99;
`RESEARCH` otherwise. Void if the restated wire fails.

## Result: the read (2026-11-11)

Read by `replay.py report`, unchanged since RAR-S116's registration, on the
files that run wrote: 3,670 searches an arm over the 60 games, 3,446 of them
at moves without a mate score. Log:
`analysis/artifacts/d64-consumers-2026-11-11/report.log`. The restated wire
passed; every search dumped each counter once and the arms searched the same
moves.

**The deciding quantity: *r* = 1.022,** 95% interval 0.990 to 1.043. Arm 116
needs fewer nodes than arm 112 in 1,659 searches and more in 1,756.

**By the frozen rule: `NO_CHANGE`.** With the three consumers held, keeping
the entry's older move does not make the tree smaller than storing the fail
low's best-scoring move. The interval excludes the 0.98 the other verdict
needed.

| nodes to the deepest depth both arms completed, median per search | ratio | 95% interval |
|---|---:|---:|
| the store change, consumers as they are (arm 4 / arm 0) | 0.811 | 0.784 to 0.834 |
| the store change with IIR held (arm 20 / arm 4) | 1.239 | 1.216 to 1.269 |
| with the cut-node LMR term held (arm 36 / arm 4) | 0.995 | 0.979 to 1.015 |
| with the razoring guard held (arm 68 / arm 4) | 0.978 | 0.956 to 0.996 |
| with all three held (arm 116 / arm 4) | 1.258 | 1.231 to 1.294 |
| the holds on the source store (arm 112 / arm 0) | 1.000 | 1.000 to 1.000 |
| **the store change under held consumers (arm 116 / arm 112)** | **1.022** | **0.990 to 1.043** |
| the best-scoring move kept from the picker (arm 128 / arm 0) | 1.235 | 1.190 to 1.272 |

| depth inside the games' node counts | mean | against the source |
|---|---:|---:|
| arm 0, the source | 12.954 | |
| arm 4, the rejected candidate | 13.612 | +0.658 (se 0.037) |
| arm 20, IIR held | 13.025 | +0.070 (se 0.036) |
| arm 36, the LMR term held | 13.624 | +0.670 |
| arm 68, razoring held | 13.651 | +0.696 |
| arm 116, all three held | 13.021 | +0.066 (se 0.038) |
| arm 112, the holds on the source store | 12.970 | +0.016 |
| arm 128, the move kept from the picker | 12.590 | −0.364 (se 0.033) |

| pooled counters | arm 0 | arm 4 | arm 112 | arm 116 | arm 128 |
|---|---:|---:|---:|---:|---:|
| IIR for want of a move, of interior nodes | 9.92% | 13.54% | 9.96% | 10.16% | 9.80% |
| LMR at a cut node without a move, of LMR | 9.99% | 13.52% | 9.98% | 9.61% | 9.18% |
| nodes entered on a fail-low entry, of interior nodes | 9.23% | 9.64% | 9.23% | 9.16% | 9.31% |
| of those, the entry gives the picker a move | 100.00% | 22.50% | 100.00% | 21.47% | 0.00% |
| of those, failing high | 15.98% | 14.45% | 15.98% | 14.91% | 15.53% |
| of the failing high, on the first move searched | 64.65% | 67.04% | 64.70% | 66.12% | 62.87% |
| moves searched to that cutoff | 1.807 | 1.772 | 1.806 | 1.825 | 1.878 |

**What it answers.**

1. **(b) IIR carries the depth, all of it.** The store change searches 0.811
   of the source's nodes to the same depth and gains 0.66 of depth at the
   games' node counts. With IIR alone held, the tree returns (1.239 against
   1/0.811 = 1.233) and the gain is 0.07. Under the store change IIR fires
   for want of a move at 13.5% of interior nodes against 9.9%: the added
   ones are revisited fail-low nodes.
2. **(a) and (d) carry nothing.** Holding the cut-node LMR term gives 0.995
   and holding the razoring guard 0.978. The LMR term's population does move,
   from 10.0% to 13.5% of LMR computations, and the tree does not answer.
3. **(c) The best-scoring move is useful ordering, and the older move is no
   better.** Kept from the picker, it costs 23% more nodes and 0.36 of depth.
   Replaced by the entry's older move, which only 21% of fail-low entries
   have, the tree is 2% larger, inside its interval of level. Those nodes cut
   on the first move a little more often (66.1% against 64.7%) and take a
   little longer to cut (1.825 moves against 1.806).
4. **With the gate.** The candidate's depth was IIR taking a ply from
   revisited fail-low nodes. That bought 0.66 of nominal depth, and the games
   lost 14.7 ± 13.8 nElo (RAR-S115). A ply at those nodes was worth more than
   the ply the clock gave back. That is the sense in which (e) holds: the
   depth was bought by reduction, not by better ordering.
5. **The registered refit is not the right rework.** IIR's depth threshold
   cannot pick out a node class: raising it removes IIR everywhere. The LMR
   term's population moves the tree by nothing, so a sweep of it would be
   flat, which PLAN's rule for a conditional tune reads as evidence against
   the tune. And a refit could only repair a loss; this read finds no gain
   under it to gate.
6. **Not tested:** a store that keeps an entry's older move and otherwise
   stores the best-scoring one. Nothing here predicts a gain from it.

**A lead for its own leaf.** IIR in the source has no node-type condition.
Stockfish's runs only at PV and cut nodes, and Reckless has none. This read
and the gate together say IIR at expected all nodes cost strength where the
store change added it. Whether the source's own IIR at such nodes costs the
same way is not measured here. It is PLAN's D.6.27.

**Calibration** (predictions 2 to 8 of RAR-S116).

2. Arm 4 against arm 0, median 0.50 to 0.75 (0.7): a miss in magnitude,
   0.811. The forecast leaned on RAR-S113's fresh state; a replayed game's
   table is nearer its game-like one. Depth +0.6 to +1.0 (0.75): a hit,
   +0.658.
3. IIR held at 1.25 or more (0.6): a miss by 0.01 at 1.239, although that is
   the whole of the tree. The LMR term at 0.95 to 1.15: a hit, 0.995.
   Razoring at 0.97 to 1.03: a hit, 0.978. IIR's share the larger (0.7): a
   hit.
4. All three held at 1.4 or more (0.6): a miss in magnitude, 1.258, the same
   error as item 2.
5. *r* between 0.99 and 1.06 (0.55), so `NO_CHANGE` (0.75): as forecast.
6. Arm 116 cutting on the first move less often (0.65): a miss in mechanism.
   It cuts first more often, because an entry keeps an older move exactly
   where the node has cut before.
7. Arm 128 against arm 0 at 1.02 or more (0.6): a hit, far under-forecast
   at 1.235.
8. The named failure modes: the interval's lower end, 0.990, sits at the
   `NO_CHANGE` threshold, so the rule's word rests on the point estimate; it
   still excludes 0.98. The hold was not too coarse: on the source store it
   changes no node count in 2,579 of 3,418 searches and the median ratio is
   1.000.

Not predicted: that the LMR term's population would move by a third with no
effect on the tree. RAR-S113 had read the wrong population, every LMR
computation, where the term applies only at cut nodes off the PV line.

## Disposition (2026-11-11)

**`NO_CHANGE`: D.6.4 closes.** The store change has no ordering gain to
gate, and what it did in games was to widen IIR's population, which
RAR-S115 rejected. The source's store stands, pinned by
`a_fail_low_store_carries_its_best_scoring_move`. The registered refit is
withdrawn as the rework. Retry only once IIR no longer reduces a node for a
fail-low entry without a move (D.6.27, or the search re-fit, F.6), and then
as an ordering change with a frozen prediction of fewer nodes at equal
consumers.
