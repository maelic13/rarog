# RAR-M85 — The canary rule reviewed against its own game record: a later settle is a diagnostic, not a hold — DECIDED 2026-11-11

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** 2026-11-11, the agent's review. The maintainer delegated the
  decision on that date, on condition that a rule is changed only for a stated
  and valid reason and never because a candidate failed it.
- **Kind:** a review of an instrument, from records and one read of existing
  files. No engine run, no game.
- Disposition: retained — decided 2026-11-11: the canary rule is kept as a
  diagnostic; a later settle never holds a candidate from its gate, and a lost
  tactic stops one only with WAC under its floor.
- **Question:** what do the canary rule's two legs measure, and what may hold a
  candidate before its gate?
- **Why now:** [RAR-S109](RAR-S109.md) and [RAR-S110](RAR-S110.md) held D.6.3
  and D.6.4 in research because an arm lost one or two canaries, each by a
  later settle with the position still solved.

## The rule, and how it was used

PLAN rule 8's canary line: no oracle-anchored canary the baseline solves may be
lost, where solving means stable on the `depthpv 10` probe no later than the
oracle's stable depth plus two, and solved at 100k nodes. There are 116
canaries: WAC positions the oracle settles by depth 6 and solves at 100k.

The same rule 8 says what a zero-game floor is: a diagnostic. A floor failure
calls for a written cause. It never holds a candidate that the games cleared,
and a floor passed never accepts one. PROCESS adds that a true correctness
canary may veto.

RAR-S109 registered something stricter for its screen: an arm that loses a
canary keeps its item in research. RAR-S110 carried it. That was each
registration's own rule, read as PLAN's.

## The record against games

Every earlier change that lost canaries and then played:

| change | canaries lost / new | WAC at 100k | games |
|---|---|---|---|
| B.3 proof cluster, unfitted ([RAR-S79](RAR-S79.md)) | 4 / 6 | — | +18.4 ± 9.4 Elo, 2,000 games |
| quiescence evasion pruning ([RAR-S87](RAR-S87.md)) | 3 / — | 5 or 6 over 236 | +7.8 Elo, 2,000 games |
| quiescence noisy history ([RAR-S87](RAR-S87.md)) | 3 / — | 5 or 6 over 236 | −4.7 Elo, 2,000 games |
| one aspiration retry ([RAR-S89](RAR-S89.md)) | 2 / 5 | 241 against 232 | −18.3 ± 6.9 Elo, H0 at 3,982 games |

- The change that lost the most canaries gained the most.
- The change with the best zero-game picture, three more canaries than its
  baseline and nine more WAC solutions, was rejected at −18.
- RAR-S87's own calibration says it: the rule's zero-game default was
  overturned by games for one switch and confirmed for the other.

In the D.6 screens, six of the seven changed arms lost one or two canaries.
Two of those arms pass 94 canaries against the source's 93, and solve 242 and
245 of WAC against 239.

## What each leg measures

**The settle leg** counts iterations. The per-depth moves of the D.6 losses,
from RAR-S110's probes:
- **WAC.115**, ProbCut typed `false`: the right move at depths 1 to 7, another
  move at depth 8, the right move at 9 and 10. One iteration's flicker.
- **WAC.069**, the fail-low move: right at depths 3 and 4, another move at
  depth 5, right from 6 to 10. One iteration's flicker.
- **WAC.182**, both: the source finds the move at depth 8, which is exactly the
  limit, and each arm at depth 9.

A flicker or a one-iteration delay at depth 5 to 9 falls early in a 100k
budget; [RAR-S112](RAR-S112.md) and [RAR-S113](RAR-S113.md) read the node
counts. Any change to the search rules reshuffles each position's tree:
with ProbCut's typing alone, the per-position node ratio at depth 13 has
quartiles of 0.645 and 1.268 ([RAR-S111](RAR-S111.md)). A rule that fails on
one shifted position in 93 therefore fires on nearly every real change, and it
ignores the positions that shift the other way.

RAR-S85 kept the rule after twelve neutral perturbations lost no canary. Those
were table sizes and a node budget. They change no search rule, so they do not
reshuffle a depth-10 tree, and the rule's rate on rule-changing but harmless
changes was never measured. The table above is that measurement.

**The 100k leg** can fail. From B.0's files (`tools/results/b0-20260913/`),
read 2026-11-11:

| engine | WAC at 100k | canaries unsolved at 100k | later settle only |
|---|---:|---:|---:|
| the oracle | 242 | 0 | 0 |
| Rarog at B.0 (2.4.0) | 200 | 12 | 37 |
| Reckless | 224 | 11 | 28 |
| the source now (RAR-S110's arm 0) | 239 | 5 | 18 |

Reckless fails 39 canaries, more than the source's 23, and is the stronger
engine by a wide margin. So neither leg orders engines by strength. Both
describe how a search spends its first 100k nodes on shallow tactics, which is
what rule 8 already says a tactical suite is: an explanation, not Elo.

## Decision

1. **A later settle is a diagnostic,** with the standing rule 8 gives every
   zero-game floor. It is reported, with new passes beside the losses, and it
   gets a written cause from the per-depth moves. It never holds a candidate
   from its gate.
2. **A lost tactic** is a canary the baseline solves at 100k nodes and the
   candidate does not. It stops a candidate before any game only together with
   WAC at 100k under its floor, the baseline's count minus 6: harm on both
   instruments, as RAR-S110 registered for its cluster.
3. **Nothing is removed.** The canary count, its losses and its new passes are
   still computed and recorded for every screen.
4. **The reason is not that candidates failed.** It is that rule 8 already
   gives the floor this standing, that the settle leg has no predictive record
   against games, and that its losses are single-iteration shifts.

This rule would not have released the one aspiration retry on its own merit
either: that change passed every zero-game screen and lost its gate. Zero-game
tactical screens cannot stand in for the gate in either direction.

## What does not change

- RAR-S109's and RAR-S110's verdicts are history, made by the rules they
  registered. They are not edited.
- The repair gates in AGENTS are untouched. A repair is still accepted only by
  its registered read.
- A registration may still set a stricter stop for its own screen, if it says
  why the candidate at hand calls for it.

## Retry trigger

Revisit if a candidate that this rule lets through loses its gate and its
later settles, read in nodes, are the cause. One such case is an observation.
Two make the settle leg worth a threshold of its own.
