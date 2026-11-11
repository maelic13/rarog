# RAR-S112 — D.6.3's ProbCut child type as its own repair: the defect, its reach and the tactical read, fresh and game-like

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11 before any read it names as
  unexposed. The read is the agent's (deterministic, no games). A gate, if the
  item ends `READY_FOR_IMPLEMENTATION`, is registered with the implementation,
  before any game, and is the maintainer's command.
- **Kind:** a zero-game readiness read for one repair candidate.
- **Baseline:** `d6` at `e23fe05`, engine source `ee0b3e4`, bench 13,038,236 /
  EBF 2.540.
- **Inherits:**
  - [RAR-S109](RAR-S109.md): on the bench, ProbCut's verification children
    fail high 2.15% of the time and are typed `cut_node = true`.
  - [RAR-S110](RAR-S110.md): the null half is closed `NO_CHANGE`. ProbCut
    children issue 2.00% of the noisy-history updates on the bench.
  - [RAR-S111](RAR-S111.md): no coupling with D.6.4 without shared state, so
    this half is read as its own repair. It asks for each item's effect per
    position, its canary outcome, positions 11 and 22, and a statement of
    which table state the reads use.
  - [RAR-M85](RAR-M85.md): a later settle is a diagnostic with a written
    cause. Only a lost tactic together with WAC under its floor stops a
    candidate before its gate.
- **The candidate:** both ProbCut verification searches pass `cut_node = false`
  (`src/search/node.rs`, the two `negamax` calls inside ProbCut). The null-move
  child stays `true`. This is arm 2 of the screen patch.

## The four questions (AGENTS)

- **Mechanism.** A verification search is one its parent expects to succeed,
  so the child is expected to fail low: an all node. Typed `true`, the child
  takes the cut-node terms: the cut-node LMR terms, LDSE and the negative
  singular extension apply, and an `Upper` entry cuts only above depth 5. The
  verification is then searched as if a cutoff were near. What should improve
  is the fit between the child's pruning and what the child is.
- **Interactions.** The child's type flips every descendant's. A fail-low at
  the child rewards the capture's noisy history. The cut-node LMR terms read
  the TT move, which D.6.4 changes; RAR-S111 found no coupling in fresh
  processes.
- **Invariants.** Arm 0 is the source as it is, bit 2 changes only the two
  calls, and the counters never steer. The implementation must reproduce arm
  2's `bench 13`, 13,148,505, and pin the `cut_node` each recursive call
  receives in a unit test (NODE-14).
- **Falsifier.** The item is not worth a gate if the children are not all
  nodes once shared state is gone, or if the type reaches nothing outside the
  child. The rules below say when.

## Table state

A fresh process is a game's first move. A game's later moves find the table and
histories of its earlier searches. Both are read here:
- **Fresh:** each bench position searched to depth 13 from an empty state, as
  in RAR-S111.
- **Game-like:** each bench position followed by six plies that arm 0 chose at
  depth 10. One process searches the positions at plies 0, 2, 4 and 6 to depth
  13 with no `ucinewgame` between them. The searches at plies 2, 4 and 6 are
  the game-like ones.

The lines are frozen in `lines.json` (SHA-256 `d5d48831…`): 39 full lines, and
position 7 alone, which is a mate in one. That is 40 fresh and 117 game-like
searches per arm. Both arms follow the same lines.

## The read (frozen)

**Binary.** RAR-S110's archived executable, SHA-256 `6ee8aab9…`, asserted by
the script on every call. RAR-S111 verified its `bench 13` for arm 0.

**Script.** `analysis/artifacts/d63-d64-items-2026-11-11/read_items.py`,
SHA-256 `16f10101…`. Every search asserts that each counter was dumped exactly
once.

**Runs,** arms 0 and 2:
- `collect`: the 157 searches of each arm, every counter kept.
- `tactics`: WAC at 100k and at 400k nodes, and the `depthpv 10` probe, by
  `tools/diag/fixed_budget_probe.py`.
- `settle`: for WAC.115 and WAC.182, the first move and the engine's node count
  at each depth to 10.

**Wire.**
- The first search of every line equals RAR-S111's fresh count, in both arms.
- WAC at 100k is 239 for arm 0 and 240 for arm 2, as RAR-S110 read.

## Exposure, stated

Seen:
- the bench's child fail-high rate and reward share, and every bench counter
  RAR-S109 and RAR-S110 print;
- arm 2's fresh per-position node counts: 0.9577 of arm 0's pooled, 0.950 at
  the median, below 1 in 22 of 39 positions;
- WAC at 100k, 240 against 239, and the old rule's two losses with their
  per-depth moves (RAR-M85).

Unexposed: every counter of the fresh and game-like searches except `nodes` in
the fresh ones; every game-like count; WAC at 400k; the node counts per depth
of the two settles.

The thresholds are earlier registrations', set there without these values:
30% and 50% for the child (RAR-S109), 1% for the reward share (RAR-S110), and
RAR-M85's tactical stop.

## Readiness (frozen)

**Void** if a wire condition fails or a search fails its dump count.

**`READY_FOR_IMPLEMENTATION`** if all of these hold in both states, read on
arm 0's pooled counters:
1. **The defect:** ProbCut children fail high under 30%.
2. **Reach:** the fail-low rewards issued at a ProbCut child are at least 1% of
   the noisy-history updates.
3. **No gross tactical harm:** arm 2 does not both leave unsolved at 100k a
   canary arm 0 solves there and solve fewer than arm 0's WAC count minus 6.

**`NO_CHANGE`** if, in both states, the children fail high at 50% or more or
the reward share is under 1%; or on gross tactical harm.

**`RESEARCH`** otherwise: the two states disagree, or the child's rate lies
between 30% and 50%.

**Reported, each with its cause, and not deciding:** the later settles, WAC at
400k, the node ratios in both states, and the per-depth node counts of
positions 11 and 22.

**If the item is ready,** the gate is the implementation session's to register
before any game: case 3, `[-5,5]`, as PLAN states for D.6.3, unless that
registration argues another case from AGENTS' *Gating*. This read gives it no
strength evidence.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The wire passes. 0.9.
2. ProbCut children fail high 1% to 5% in arm 0, fresh (0.85) and game-like
   (0.8).
3. The reward share is 1% to 4% of noisy-history updates, fresh (0.75) and
   game-like (0.7).
4. The game-like node ratio of arm 2 to arm 0 is 0.90 to 1.10 pooled (0.6) and
   0.90 to 1.05 at the median (0.6).
5. WAC at 400k: arm 2 within 4 of arm 0. 0.75.
6. No canary that arm 0 solves is unsolved by arm 2 at 400k. 0.85.
7. `READY_FOR_IMPLEMENTATION`. 0.85.
8. **Most likely failure mode:** none of the rules fails, and the read adds
   little: the game-like ratio is as scattered as the fresh one and says
   nothing about the sign of the change in games.

## Result: the read (2026-11-11)

Read by `read_items.py` (SHA-256 `16f10101…`, unchanged since registration) on
RAR-S110's archived executable. Logs and counts are beside it:
`line_arm0.json`, `line_arm2.json`, the probes' JSON files, `report_arm2.log`
and `settle_arm2.log`. Every search dumped each counter once.

**Wire: passed.** The first search of each of the 40 lines equals RAR-S111's
fresh count in arms 0 and 2. WAC at 100k is 239 and 240.

| arm 0, the source | fresh, 40 searches | game-like, 117 searches |
|---|---:|---:|
| ProbCut children searched | 44,337 | 99,709 |
| of those, failing high | 2.19% | 2.91% |
| their fail-low rewards, of noisy-history updates | 1.85% | 1.55% |
| ProbCut cuts per node where it ran | 48.10% | 45.42% |

| arm 2 against arm 0 | fresh | game-like |
|---|---:|---:|
| interior nodes, pooled | 0.9577 | 1.0067 |
| interior nodes, median | 0.950 | 0.904 |
| quartiles | 0.658 and 1.273 | 0.611 and 1.346 |
| searches below 1 | 22 of 40 | 70 of 117 |
| ProbCut cuts per node where it ran | 46.32% | 41.05% |
| fail-low rewards at a ProbCut child | none | none |

**Readiness, by the frozen rule: `READY_FOR_IMPLEMENTATION`.**
1. **The defect:** the children fail high 2.19% fresh and 2.91% game-like,
   under 30% in both states.
2. **Reach:** their rewards are 1.85% and 1.55% of the noisy-history updates,
   over 1% in both states.
3. **No gross tactical harm:** arm 2 solves 240 of WAC at 100k against a floor
   of 233, and leaves no canary unsolved there that arm 0 solves.

**What the change does,** which the registration had half right:
- **It removes the reward.** The fail-low parent reward is issued only at a cut
  node or a PV node (`node.rs`, after the move loop). Typed `false`, a ProbCut
  child issues none: the counter reads zero in arm 2. The change is therefore
  two things that this arm cannot separate: the child's pruning, and the loss
  of those rewards to the capture's history.
- **The verification confirms less often.** ProbCut cuts 4.4 points less often
  per node in game-like searches, and 1.8 points less often fresh. A child
  searched as an all node is reduced less, so it refutes more captures.
- **The tree does not move in one direction.** The pooled ratio is 0.96 fresh
  and 1.01 game-like, with half the searches between 0.61 and 1.35.

**Reported, with causes, not deciding:**
- **WAC at 400k:** 266 against 264. Six positions are lost and eight gained. At
  100k eight are lost and nine gained. A change this small flips 17 of 300
  positions at 100k, and the count moves by one.
- **A lost tactic at 400k:** WAC.168, which arm 0 solves at 400k and arm 2 does
  not. Neither solves it at 100k. It is one of the six lost against eight
  gained.
- **The later settles,** WAC.115 and WAC.182, the old rule's two losses:
  - WAC.115: arm 2 has the right move through depth 7, another at depth 8
    (22,135 nodes) and the right one again from depth 9 (28,812 nodes).
  - WAC.182: arm 0 finds the move at depth 8 after 43,569 nodes. Arm 2 finds
    it at depth 9 after 41,856 nodes: one iteration later and in fewer nodes.
- **Positions 11 and 22,** the engine's nodes at depths 11, 12 and 13, fresh:
  - position 11: arm 0 550,097, 1,725,628 and 3,522,746; arm 2 947,461,
    1,627,127 and 3,240,263;
  - position 22: arm 0 99,218, 317,372 and 590,124; arm 2 202,110, 339,079
    and 426,776.

  Arm 0 triples between depths 11 and 12 in both. Arm 2 is larger at depth 11
  and no larger by depth 13. Neither arm runs away from the other.

**For the gate's registration.** This read is no strength evidence. What it
gives the gate:
- The implementation is the two calls and nothing else, and its `bench 13`
  must read 13,148,505.
- The reach is small and the tree effect has no sign, so the prior is near
  zero. PLAN's case 3, `[-5,5]`, can exclude harm near −5; it cannot tell +1
  from −1.
- If the gate fails its harm rule, the first rework to register is the typing
  with the reward kept, which separates the two things this arm joins.

**Calibration.**

1. The wire passes (0.9): a hit.
2. Children failing high 1% to 5%: hits at 2.19% and 2.91%.
3. The reward share at 1% to 4%: hits at 1.85% and 1.55%.
4. The game-like ratio at 0.90 to 1.10 pooled: a hit at 1.007. At 0.90 to 1.05
   at the median: a hit at 0.904, at its edge.
5. WAC at 400k within 4: a hit, 266 against 264.
6. No canary lost at 400k (0.85): a miss, WAC.168. The prediction treated the
   canaries as safely inside a 400k budget. Five of them are unsolved at 100k
   by the source itself, and those sit at the budget's edge.
7. `READY_FOR_IMPLEMENTATION` (0.85): it is.
8. The named failure mode occurred as written: every rule passed, and the
   game-like ratio is as scattered as the fresh one.

Not predicted: that the change removes the reward outright. The registration
described the reward as one of the things the type decides and did not say
that `false` means none. A miss in mechanism, found by the counter.
