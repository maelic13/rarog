# RAR-S113 — D.6.4's fail-low TT move as its own repair: the mechanism, its tree and the tactical read, fresh and game-like

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11 with [RAR-S112](RAR-S112.md), before
  any read either names as unexposed. The read is the agent's (deterministic,
  no games). A gate, if the item ends `READY_FOR_IMPLEMENTATION`, is registered
  with the implementation, before any game, and is the maintainer's command.
- **Kind:** a zero-game readiness read for one repair candidate.
- **Baseline:** `d6` at `e23fe05`, engine source `ee0b3e4`, bench 13,038,236 /
  EBF 2.540.
- **Inherits:**
  - [RAR-S109](RAR-S109.md): on the bench every `Upper` store carries a move,
    65.76% of them change the stored one, and a TT move from an `Upper` entry
    is at 10.27% of interior nodes.
  - [RAR-S111](RAR-S111.md): no coupling with D.6.3 without shared state. Alone,
    in fresh processes, this change searches 0.5594 of the source's interior
    nodes pooled and 0.754 at the median, below 1 in 32 of 39 positions. The
    bench's −1.94% is superseded as its size. D.6.4's arm differed sixfold at
    one position between an empty table and the bench's, so the registration
    must say which table state its reads use.
  - [RAR-M85](RAR-M85.md): a later settle is a diagnostic with a written
    cause. Only a lost tactic together with WAC under its floor stops a
    candidate before its gate.
- **The candidate:** an `Upper` store passes no move, so the table keeps the
  move it holds for that position. This is bit 4 of the screen patch. Arm 4 is
  the change on the source. Arm 6 is the change on top of D.6.3's ProbCut
  typing, the head it lands on if D.6.3's gate accepts.

## The four questions (AGENTS)

- **Mechanism.** At a fail-low node no move raised alpha, so the move stored is
  the least bad of moves that all failed. It replaces whatever move the entry
  held, which may be one that once raised alpha or cut. At the next visit that
  move is searched first, and its presence alone switches off what the search
  does without a TT move. What should improve is the first move tried, and the
  meaning of "this node has a TT move".
- **Interactions.** The consumers at the next visit: IIR, the razoring guard,
  the no-TT-move LMR terms, the first-move slot and the fail-low parent reward.
  IIR's depth and the no-TT-move terms were fitted with the defect in place,
  so the populations they act on change.
- **Invariants.** Arm 0 is the source as it is, bit 4 changes only the move an
  `Upper` store passes, and the counters never steer. The table keeps a stored
  move when a store passes none. The implementation must reproduce the arm's
  `bench 13`: 12,785,308 on the source, 9,892,346 on top of D.6.3.
- **Falsifier.** The item is not worth a gate if the stores change nothing in
  a game-like table, where an entry more often already holds the same move, or
  if the moved TT move reaches few nodes there. The rules below say when.

## Table state

As RAR-S112: fresh searches and game-like ones, on the same frozen lines
(`lines.json`, SHA-256 `d5d48831…`), 40 fresh and 117 game-like searches per
arm.

## The read (frozen)

**Binary and script.** As RAR-S112: RAR-S110's archived executable
(`6ee8aab9…`) and `read_items.py` (`16f10101…`).

**Runs,** arms 4 and 6, against arms 0 and 2 from RAR-S112:
- `collect`: the 157 searches of each arm.
- `tactics`: WAC at 100k and at 400k nodes, and the `depthpv 10` probe.
- `settle`: for WAC.069 and WAC.182, the first move and the engine's node count
  at each depth to 10.

Arm 4 is read against arm 0, and arm 6 against arm 2.

**Wire.**
- The first search of every line equals RAR-S111's fresh count, in arms 4
  and 6.
- WAC at 100k is 237 for arm 4 and 242 for arm 6, as RAR-S110 read.

## Exposure, stated

Seen:
- the bench's store shares and the consumers' counts, as RAR-S109 prints them;
- arms 4 and 6's fresh per-position node counts (RAR-S111);
- WAC at 100k, 237 and 242 against 239, and the old rule's losses with their
  per-depth moves: WAC.069 and WAC.182 for arm 4, WAC.069 for arm 6.

Unexposed: every counter of the fresh and game-like searches except `nodes` in
the fresh ones; every game-like count, in every arm, arm 0's included, since
this is registered before RAR-S112's read runs; WAC at 400k; the node counts
per depth of the settles.

The thresholds are RAR-S109's for this item, 1% of `Upper` stores and 0.5% of
nodes, and RAR-M85's tactical stop.

## Readiness (frozen)

**Void** if a wire condition fails or a search fails its dump count.

**`READY_FOR_IMPLEMENTATION`** if all of these hold:
1. **The mechanism, in both states,** on arm 0's pooled counters: the `Upper`
   stores that change the stored move (a different move, or a move where the
   entry had none) are at least 1% of all `Upper` stores, and a TT move from an
   `Upper` entry is at 0.5% or more of interior nodes.
2. **No gross tactical harm** in arm 4 against arm 0: it does not both leave
   unsolved at 100k a canary arm 0 solves there and solve fewer than arm 0's
   WAC count minus 6.

**`NO_CHANGE`** if the mechanism is under either threshold in both states, or
on gross tactical harm in arm 4.

**`RESEARCH`** if the two states disagree on the mechanism.

Gross tactical harm in arm 6 against arm 2 does not close the item. It is
recorded for the gate that would build on D.6.3.

**Reported, each with its cause, and not deciding:** the node ratios in both
states, for arm 4 against arm 0 and arm 6 against arm 2; the consumers' counts
in the game-like state; the later settles; WAC at 400k; the per-depth node
counts of positions 11 and 22.

**If the item is ready,** the gate is the implementation session's to register
before any game, after D.6.3's gate has been read: case 3, `[-5,5]`, because
the change moves shared search signals and could be large either way. A repair
that fails its harm rule leaves by the gate with its defect still open, and its
rework is a new registration with IIR's depth and the no-TT-move terms
refitted (AGENTS). This read gives the gate no strength evidence.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The wire passes. 0.9.
2. In arm 0 the `Upper` stores that change the stored move are 55% to 75% of
   all, fresh (0.7), and 45% to 75%, game-like (0.6). The share that replaces
   a different move is higher game-like than fresh. 0.75.
3. A TT move from an `Upper` entry is at 8% to 14% of interior nodes, fresh
   (0.7), and 8% to 16%, game-like (0.6).
4. The game-like node ratio of arm 4 to arm 0 is 0.50 to 0.85 pooled (0.6) and
   0.65 to 0.90 at the median (0.6). Arm 4 searches fewer nodes in at least 60%
   of the game-like searches. 0.7.
5. The game-like node ratio of arm 6 to arm 2 is 0.50 to 0.90 pooled. 0.6.
6. WAC at 400k: arm 4 within 5 of arm 0. 0.6. No canary that arm 0 solves is
   unsolved by arm 4 at 400k. 0.8.
7. `READY_FOR_IMPLEMENTATION`. 0.85.
8. **Most likely failure mode:** the readiness rules pass and the size does not
   carry: the game-like shrink is far from the fresh one, so the fresh −44%
   is no guide to what the gate will see.

## Result: the read (2026-11-11)

Read by `read_items.py` (SHA-256 `16f10101…`, unchanged since registration) on
RAR-S110's archived executable. Logs and counts are beside it:
`line_arm4.json`, `line_arm6.json`, the probes' JSON files, `report_arm4.log`,
`report_arm6_vs2.log`, `settle_arm4.log` and `settle_arm6.log`. Every search
dumped each counter once.

**Wire: passed.** The first search of each of the 40 lines equals RAR-S111's
fresh count in arms 4 and 6. WAC at 100k is 237 and 242.

| arm 0, the source | fresh, 40 searches | game-like, 117 searches |
|---|---:|---:|
| `Upper` stores | 2,115,491 | 6,409,278 |
| storing the entry's own move again | 35.60% | 47.40% |
| replacing a different move | 10.79% | 13.49% |
| filling an empty slot | 53.61% | 39.11% |
| so changing the stored move | 64.40% | 52.60% |
| TT move from an `Upper` entry, of interior nodes | 10.38% | 13.21% |
| nodes where that move alone kept IIR off | 396,447 | 1,464,306 |
| nodes where it alone blocked razoring | 3,116 | 38,315 |

| arm 4 against arm 0 | fresh | game-like |
|---|---:|---:|
| interior nodes, pooled | 0.5594 | 0.8696 |
| interior nodes, median | 0.754 | 0.905 |
| quartiles | 0.467 and 0.932 | 0.510 and 1.187 |
| searches below 1 | 32 of 40 | 73 of 117 |
| IIR applied, of interior nodes | 14.33% to 16.83% | 11.71% to 15.10% |
| LMR with no TT move | 25.64% to 69.01% | 20.31% to 57.10% |
| the same at cut nodes | 10.76% to 14.48% | 6.79% to 10.70% |
| nodes where an `Upper` move kept IIR off | 396,447 to 42,856 | 1,464,306 to 436,622 |
| null-move cut rate | 69.39% to 78.36% | 71.31% to 66.32% |

**Readiness, by the frozen rule: `READY_FOR_IMPLEMENTATION`.**
1. **The mechanism, in both states:** 64.40% and 52.60% of the source's `Upper`
   stores change the stored move, over 1%, and a TT move from an `Upper` entry
   is at 10.38% and 13.21% of interior nodes, over 0.5%.
2. **No gross tactical harm:** arm 4 solves 237 of WAC at 100k against a floor
   of 233, and leaves no canary unsolved there that arm 0 solves.

**What the two states say:**
- **The size depends on the table.** In fresh searches the change searches 44%
  fewer interior nodes pooled. In game-like searches it searches 13% fewer,
  and 9.5% fewer at the median. Games are played in the second state. The
  fresh figure is the size at a game's first move only.
- **The mechanism is as active in a game-like table.** More stores repeat the
  entry's own move there (47% against 36%), and more replace a different one
  (13.5% against 10.8%).
- **The consumers move a long way.** LMR without a TT move goes from a fifth
  of its computations to more than half, and IIR applies at 15% of nodes
  instead of 12%. IIR's depth and the no-TT-move terms were fitted on the
  smaller populations.
- **The null move goes opposite ways.** Its cut rate rises nine points fresh
  and falls five points game-like.

**On top of D.6.3,** arm 6 against arm 2:
- **Nodes.** Fresh, 0.6557 pooled and 0.791 at the median. Game-like, 0.6112
  pooled and 0.871 at the median, below 1 in 75 of 117 searches.
- **WAC.** 242 against 240 at 100k, and 267 against 266 at 400k.
- **Lost tactics.** None at 100k. At 400k, WAC.170 and WAC.228, among seven
  positions lost and eight gained. That is not gross harm, and it is recorded
  here for the gate that would build on D.6.3.

**Reported, with causes, not deciding:**
- **WAC at 400k,** arm 4: 265 against 264, five lost and six gained. At 100k
  fifteen are lost and thirteen gained: 28 of 300 positions flip, and the
  count moves by two.
- **Canaries unsolved at 100k:** four in arm 4 against five in arm 0. No lost
  tactic at 100k or at 400k.
- **The later settles,** WAC.069 and WAC.182, the old rule's losses, with
  WAC.295 a new pass:
  - WAC.069: arm 4 has the right move at depths 3 and 4, another at depth 5
    (1,391 nodes) and the right one again from depth 6 (1,898 nodes). Arm 6
    does the same.
  - WAC.182: arm 0 finds the move at depth 8 after 43,569 nodes. Arm 4 finds
    it at depth 9 after 42,927 nodes: one iteration later and in fewer nodes.
- **Positions 11 and 22,** the engine's nodes at depths 11, 12 and 13, fresh:
  - position 11: arm 0 550,097, 1,725,628 and 3,522,746; arm 4 167,397,
    639,895 and 688,658;
  - position 22: arm 0 99,218, 317,372 and 590,124; arm 4 108,592, 162,429
    and 353,294.

  Arm 0 triples from depth 11 to 12 in both. Arm 4 is already smaller at
  depth 11 in position 11 and nearly stops growing at depth 13.

**After exposure, not registered** (`after_exposure.py` and its log). RAR-S111's
coupling index on the 117 game-like searches:
- pooled 0.7029, and 1.0791 without three searches (position 11 at plies 4 and
  6, position 15 at ply 4);
- median 1.044, below 1 in 51 of 115 active searches, z −0.650.

By RAR-S111's rule that is again not general. The pooled figure is three large
searches. Gating D.6.4 on the head that D.6.3's gate leaves measures its
effect there directly, whatever the index is.

**For the gate's registration.** This read is no strength evidence. What it
gives the gate:
- The implementation is the store's move and nothing else. Its `bench 13` must
  read 12,785,308 on the source, or 9,892,346 on top of D.6.3's typing.
- The change is large and its sign in games is open. Fewer nodes at a fixed
  depth here means more reduction, not a cheaper tree of the same quality.
  Case 3, `[-5,5]`, is the gate, and it resolves quickly if the effect is
  large either way.
- If the gate fails its harm rule, the rework names IIR's depth and the
  no-TT-move LMR terms as the constants fitted around the defect.

**Calibration.**

1. The wire passes (0.9): a hit.
2. The stores that change the move at 55% to 75% fresh: a hit at 64.40%. At 45%
   to 75% game-like: a hit at 52.60%. The replacing share higher game-like: a
   hit, 13.49% against 10.79%.
3. The TT move from an `Upper` entry at 8% to 14% fresh: a hit at 10.38%. At 8%
   to 16% game-like: a hit at 13.21%.
4. The game-like ratio at 0.50 to 0.85 pooled: a miss at 0.870. At 0.65 to 0.90
   at the median: a miss at 0.905. Fewer nodes in at least 60% of game-like
   searches: a hit at 62.4%. The miss is in magnitude: the prediction put the
   game-like size between the bench's and the fresh one, and it is nearer the
   bench's.
5. Arm 6 against arm 2 at 0.50 to 0.90 pooled, game-like: a hit at 0.611.
6. WAC at 400k within 5: a hit, 265 against 264. No canary lost at 400k: a hit.
7. `READY_FOR_IMPLEMENTATION` (0.85): it is.
8. The named failure mode occurred: the rules pass, and the fresh −44% does not
   carry to the game-like state.
