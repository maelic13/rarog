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
