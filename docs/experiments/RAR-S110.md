# RAR-S110 — D.6.3 and D.6.4 as one cluster: reach, coupling and the canary question, a screen

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-10 before any read of what it names as
  unexposed. The screen is the agent's (deterministic). A gate, if the cluster
  ends `READY_FOR_IMPLEMENTATION`, is registered later, before any game, and is
  the maintainer's command.
- **Kind:** a zero-game screen for one candidate cluster: D.6.3's two child types
  and D.6.4's fail-low TT move.
- **Baseline:** `d6` at `44f14d6`, engine source `ee0b3e4`, bench 13,038,236 /
  EBF 2.540.
- **Inherits** [RAR-S109](RAR-S109.md) and its correction (`44f14d6`):
  - Each half's child is mostly an all node. ProbCut children fail high 2.15%
    and null children 31.19%.
  - Interior subtree share is void as a reach measure for both halves.
  - D.6.3 and D.6.4 couple: arm 7 searched 30.25% fewer nodes, 34% below the
    product of arms 3 and 4. Arms 1 and 2 together search 12% more than their
    product (1.0772 against 0.9580).
  - RAR-S109's record is history and is not edited here.
- **Research questions:**
  1. Does each half of D.6.3 reach its parent's decision, measured from the
     parent's side?
  2. What carries the coupling?
  3. Should the canary rule tell a later settle from a lost tactic?

## The four questions (AGENTS)

- **Mechanism.** The null child and ProbCut's verification children are
  searches the parent expects to fail high, so they are all nodes. They are
  typed `true` (RAR-S109 confirms they behave as all nodes). A fail-low store
  overwrites the stored move with the least-bad one. In RAR-S109 that move
  appeared at 10.27% of interior nodes and alone kept IIR off at 338,465 of
  them. What should improve is how well these nodes' pruning, reductions and
  ordering fit what the nodes are.
- **Interactions:**
  - The null child's result decides whether its parent searches any move.
  - The ProbCut child's type decides its own pruning and whether a fail-low
    there rewards the capture's noisy history.
  - Every descendant's type flips with it.
  - `lmr_cut_node_no_tt_move` (2,073/1024, about two plies) reads both items:
    D.6.3 changes which nodes are cut nodes, and D.6.4 which have a TT move. In
    RAR-S109's arm 4, the TT move from an `Upper` entry fell from 10.27% to
    2.24% of nodes, and LMR's no-TT-move share rose from 25.60% to 65.54%.
  - The cut-node TT admission reads `Upper` entries.
- **Invariants:**
  - Arm 0 is the source as it is, and each bit changes only what it names.
  - The counters never steer.
  - Arms 0, 1, 2, 3, 4 and 7 are the same searches as in RAR-S109, so they must
    reproduce its `bench 13` counts exactly.
  - In the code that would follow: the table keeps a stored move when a store
    passes none, and an implementation's unit test pins the `cut_node` each
    recursive call receives (NODE-14).
- **Falsifier.** The cluster has no reach worth a gate if neither half moves its
  own parent's decision beyond what unrelated tree changes do (the floors
  below). It is also out if the coupling turns out to be a few bench positions
  rather than a mechanism. The rules below stop it either way.

## Hypotheses for the coupling

- **C1, the no-TT-move LMR term (lead):** D.6.4 multiplies the cut nodes without
  a TT move, and D.6.3 changes which nodes are cut nodes. Together they reduce
  far more lines by about two plies than either alone. **Separating read:** the
  probe arms with the term zeroed (bit 8).
- **C2, null decisions:** D.6.4 changes what the null child finds in the table,
  and with the child typed `false` more null moves cut, saving whole parent
  trees. **Separating read:** arm 5 (null + D.6.4) against arm 6 (ProbCut +
  D.6.4). Which one carries arm 7's reduction, and do the null-failed parents'
  trees shrink?
- **C3, TT admission:** flipped types admit `Upper` cutoffs at nodes that
  refused them, and the coupling runs through the table's cutoffs.
  **Separating read:** the TT cutoff counts across arms.
- **C4, a distribution artifact:** a few of the 40 bench positions, sharing one
  table, collapse in arm 7, and no mechanism is general. **Separating read:**
  the per-position interior node counts.

These are not exclusive.

## The screen (frozen)

**Patch.** `analysis/arm_patches/d63-d64-rar-s110-screen.patch`, on `ee0b3e4`,
is RAR-S109's screen patch plus three things:
- counters that never steer;
- bit 8 of `RAROG_S109_ARM`, which zeroes `lmr_cut_node_no_tt_move`;
- a drop guard that closes the governed-tree count on every return, because the
  move loop's cutoff leaves by `return`.

Bits 1, 2 and 4 are unchanged. **Candidate arms 0 to 7** each get `bench 13`,
counters, WAC at 100k and the canary probe. **Probe arms 8, 11, 12 and 15** get
`bench 13` and counters only; they are mechanism reads, never candidates.

**Wire.**
- Arm 0 reproduces 13,038,236.
- Arms 1, 2, 3, 4 and 7 reproduce RAR-S109's counts: 12,385,895, 13,148,505,
  14,045,048, 12,785,308 and 9,094,587.
- Arm 5 differs from arm 4, arm 6 from arm 4, and arm 8 from arm 0.

**New counters,** one thread, stride 1, summed by
`tools/diag/bench_counters.py`.

Null half, from the parent's side:
- the null-move cut rate, `nmp_cut / nmp_attempt`;
- `ch_null_int_n` and `ch_null_int_fh`: the null children that are interior
  nodes (the child's own search counted at least one interior node), and those
  that failed high. They are read against `nmp_attempt`;
- `nf_n`, `nf_nodes` and `nf_nodes_outer`: the parents whose null move was tried
  and did not cut, and that reached their move loop; the interior nodes their
  move loops searched, at every level; and those under the outermost such
  parent only, against `nodes`.

ProbCut half, from the parent's side:
- the ProbCut cut rate per node where ProbCut ran, `probcut_cut / probcut_nodes`;
- the verification searches per ProbCut attempt, `ch_pc_n / probcut_attempt`,
  and their fail-high rate;
- `pf_n`, `pf_nodes` and `pf_nodes_outer`: as for the null half, for parents
  where ProbCut ran and did not cut;
- `pc_child_reward`: the fail-low parent rewards issued at a ProbCut child,
  against `hist_noisy_upd`, every noisy-history update.

Coupling reads:
- `tt_cut_exact`, `tt_cut_lower` and `tt_cut_upper`;
- the per-position interior `nodes`, from each position's own dump;
- `lmr_cut_no_tt`.

**Coupling index.** κ = (n₇ · n₀) / (n₃ · n₄) for `bench 13` nodes, so RAR-S109
read 0.66. With the term zeroed, κ₈ = (n₁₅ · n₈) / (n₁₁ · n₁₂). κ_tt is the same
index on the total TT cutoffs of arms 0, 3, 4 and 7.

## Exposure, stated

Already seen: arms 0, 1, 2, 3, 4 and 7, with their bench counts, WAC at 100k,
canary outcomes and every RAR-S109 counter. That includes the null-move cut
rate in each, so arm 1's shift (68.74% to 74.83%) is known, as is the
per-attempt ProbCut cut rate. Arm 7 already loses WAC.069 by a later settle, and
arm 4 loses WAC.069 and WAC.182.

Unexposed:
- arms 5, 6, 8, 11, 12 and 15;
- every counter above that RAR-S109 did not read, including `probcut_nodes`,
  so the per-node ProbCut cut rate is unseen in every arm;
- the per-position nodes.

Each threshold below is set without reference to a seen value:
- **The reach floors** are defined by a rule over this run's own arms, the
  largest change that bits outside the mechanism produce. Several of the
  floor's members are seen, and the rule is chosen so that a seen member can
  only raise the floor.
- **The WAC floor** is the B.3 screen ladder's, from RAR-S79.
- **The 1% reward share** is RAR-S109's registered sparseness convention.
- **The half-magnitude rules for C1 and C3** are chosen without data.

## Readiness for the cluster as one unit (frozen)

**Void, back to `RESEARCH` as an instrument fault,** if:
- any wire condition fails, or
- the reduced LMR children fail high under 70% (RAR-S109's void rule).

**Reach.** A half is in the cluster only if it moves its own parent's decision
beyond what bits outside the mechanism do:
- **Null half.** Its floor F_null is the largest |change| in the null-move cut
  rate over the eight toggles of bit 2 or bit 4 alone (0→2, 1→3, 4→6, 5→7, 0→4,
  1→5, 2→6, 3→7). The half is in if both its toggles in D.6.4's presence exceed
  F_null: 4→5 and 6→7. The seen members already put F_null at 4.26 points or
  more (1→3).
- **ProbCut half.** Its floor F_pc is the largest |change| in the per-node
  ProbCut cut rate over the toggles of bit 1 or bit 4 alone (0→1, 2→3, 4→5,
  6→7, 0→4, 1→5, 2→6, 3→7). The half is in if both 4→6 and 5→7 exceed F_pc.
  The child's verdict almost never changes (2% fail high), so the reward
  channel also counts: the half is also in if arm 4's `pc_child_reward` is at
  least 1% of its `hist_noisy_upd`.
- **D.6.4** is in. Its reach is seen and large (65.76% of `Upper` stores change
  the move).
- **A half that fails reach** leaves the cluster and closes `NO_CHANGE` for
  this cluster. Its retry trigger is a change to that half's decision (the
  null-move margin or reduction; ProbCut's margin, depth or verification
  rule) or to the cut-node consumers.

**The cluster arm, K:**
- 4 if neither half is in;
- 5 if only the null half is;
- 6 if only ProbCut is;
- 7 if both are.

**Back to `RESEARCH`** if any of these holds:
- **C4 holds:** three positions carry at least half of the change in total
  interior nodes between arm 0 and arm 7. The bench then cannot characterise
  the coupling, and a fresh-process per-position read comes first.
- **No mechanism is supported:**
  - **C1 support:** |κ₈ − 1| is at most half of |κ − 1|.
  - **C2 support:** arm 5's or arm 6's node ratio to arm 0 is within five
    points of arm 7's.
  - **C3 support:** κ_tt − 1 has κ − 1's sign and at least half its magnitude.
- **WAC:** arm K solves fewer than arm 0's count minus 6 at 100k nodes.
- **The canary rule in force** (PLAN's as it stands): arm K loses a canary
  that arm 0 passes. The verdict is then "held by the canary rule". The
  outcome under the proposal below is reported beside it, and nothing is
  adopted.

**`NO_CHANGE` for the whole cluster** requires tactical harm on both
instruments: arm K leaves unsolved at 100k a canary that arm 0 solves at 100k,
and arm K's WAC is below arm 0's minus 6.

**`READY_FOR_IMPLEMENTATION`** otherwise: arm K's change, nothing else, with the
unit test above. Its gate is registered before any game. It is case 3,
`[-5,5]`, unless that registration predicts a gain and says why `[0,3]`.

## How WAC is read (frozen)

WAC at 100k is a fixed-node tactical read, not Elo.
- **A loss** in an arm that searches fewer bench nodes is read as added pruning
  that removes tactical lines faster than the saved nodes buy depth at that
  budget.
- **A gain** in such an arm is read as depth bought.
- Neither is a strength claim. Only the floor above acts on WAC.

## The canary question, for the maintainer

Every loss in RAR-S109 was a later settle on the `depthpv 10` probe, with the
position still solved at 100k nodes: WAC.069 (settled at 6 against a limit of
3), WAC.115 (9 against 3) and WAC.182 (9 against 8).

**Position: the rule should tell a later settle from a lost tactic, and these
arms show why.**
- The stable leg is indexed by iteration. The arms change what an iteration
  costs: `bench 13` moves by −30% to +8% across them. An iteration number is
  therefore not a fixed amount of search, and a depth-indexed criterion
  confounds "spends less per ply" with "finds the tactic later".
- The 100k leg is indexed by budget, so it is invariant to that confound.
- A later settle is not free: under a clock, a move found an iteration later can
  be missed. A fixed-node leg prices that only at its own budget.

**Proposal (a decision, not adopted here):**
- **A.** Keep the rule as it is.
- **B.** A canary is lost only when its 100k leg fails, and a later settle is
  reported beside it.
- **C.** Express the stable leg in nodes: the cumulative nodes at the settling
  iteration, held to a multiple of arm 0's.

B is the simplest node-indexed form. C keeps a settle criterion but needs the
probe to record nodes per iteration. This screen applies A. It also reports
what B would give, and adopts neither.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The six seen arms reproduce RAR-S109 exactly. 0.97.
2. Arm 5's node ratio to arm 0 is 0.70 to 0.90; arm 6's is 0.95 to 1.10. 0.6
   each.
3. The null-move cut rate in arm 5 exceeds arm 4's (65.73%) by more than F_null.
   0.45. So the null half is in. 0.4.
4. The per-node ProbCut cut rate moves less than F_pc on both ProbCut toggles.
   0.6. `pc_child_reward` is at least 1% of `hist_noisy_upd` in arm 4. 0.5. So
   the ProbCut half is in. 0.5.
5. Interior null children are 15% to 40% of null attempts (RAR-S109's 0.25
   interior nodes per child, with each interior child carrying some subtree),
   and they fail high at 25% to 45%. 0.6.
6. `nf_nodes_outer` is at least 50% of `nodes` in arm 0: most of the tree lies
   under some parent whose null move failed. 0.6.
7. C1 supported (|κ₈ − 1| ≤ ½|κ − 1|). 0.4. C2 supported. 0.5. C3 supported.
   0.3. C4 holds. 0.2.
8. WAC at 100k: arm 5 at or above arm 4's 237. 0.5. Arm 6 within 4 of 237. 0.6.
   Arm K above arm 0's 239 − 6. 0.85.
9. Arm K loses a canary under rule A. 0.7. So the cluster ends "held by the
   canary rule". 0.65. Under rule B it would lose none. 0.8.
10. `READY_FOR_IMPLEMENTATION`. 0.15. `NO_CHANGE` for the whole cluster. 0.05.
11. **Most likely failure mode:** the null cut-rate floor, raised by D.6.4's own
    reshaping (seen at −3.01 and 4.26), swallows a real null-half effect, so the
    half is dropped although it reaches its parent.
