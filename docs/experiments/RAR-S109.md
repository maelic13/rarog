# RAR-S109 — D.6.3 and D.6.4: the null-move and ProbCut child type, and the fail-low TT move, a joint screen

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-10 before any read. The screen is the
  agent's (deterministic). A gate, for each item the screen leaves
  `READY_FOR_IMPLEMENTATION`, is registered later, before any game, and is the
  maintainer's command.
- **Kind:** a zero-game screen for two repair candidates, read together because
  they share a signal (PLAN's register: D.6.3 is screened with D.6.4).
- **Baseline:** `d6` at `dcf9058`, engine source `ee0b3e4`, bench 13,038,236 /
  EBF 2.540.
- **Research questions:** do the children Rarog types `cut_node = true` at the
  null move and at ProbCut behave like cut nodes, and how much of the tree do
  they govern? How often does a fail-low store replace the position's stored
  move, and what reads that move at the next visit?

## The defects, from the source

**D.6.3.** The null-move child and both ProbCut verification searches are
called with `cut_node = true` (`src/search/node.rs`, the `negamax` call after
`make_null_move` and the two calls in ProbCut's verification). Reckless passes
`false` at all three (`src/search.rs:606`, `664–684`, read at `aec5cb2e`, the engine review's donor head). Each is
a search its parent expects to fail high, so the child is expected to fail low:
an all node. At the child the type decides:
- which TT bounds cut (`node_cutoff_score`: at a cut node an `Upper` entry cuts
  only above depth 5, a `Lower` entry at any depth);
- the cut-node LMR terms, including `lmr_cut_node_no_tt_move`;
- LDSE and the negative singular extension;
- the history bonus offsets;
- the fail-low parent reward. At a ProbCut child that rewards the capture's
  noisy history; at a null child it returns on the null move.

It also flips the type of every descendant. The reduced LMR search is typed
`true` here and in the donor (`search.rs:922`), so it is the reference for a
correctly typed `true` node.

**D.6.4.** `best_move` is set on every new best score, so an `Upper` store
carries the least-bad move. The table keeps a stored move only when the new
store has none (`tt.rs`, `store_*`), so a fail-low overwrites the position's
earlier move. Reckless sets the move only when alpha rises. At the next visit
the move feeds:
- IIR, which needs no TT move;
- the razoring guard, which a quiet TT move blocks;
- `lmr_cut_node_no_tt_move`, at cut nodes only, the signal D.6.3 also moves;
- the first move searched;
- the fail-low parent reward's `parent.mv == parent.tt_move` term.

## Hypotheses

- **D.6.3 H1, leading for ProbCut:** ProbCut's verification succeeded 98% of the
  time at the B.3 head, so its child is an all node, and typing it `true` sets
  its pruning, reductions and history the wrong way.
- **D.6.3 H2, the null half:** the null child's right type depends on how often
  the null move fails. At the B.3 head 22% of attempts cut, so 78% of null
  children failed high and `true` was the better prior. Later margin changes
  moved that rate, and it has not been recorded since.
- **D.6.3 H3, inert:** these subtrees are a small share of the tree, and the
  SPSA fits absorbed the typing.
- **D.6.4 H1, leading:** the least-bad move orders worse than the earlier move.
  It also blocks IIR and razoring at nodes that keep failing low.
- **D.6.4 H2, useful:** the least-bad move is a fair guess, and IIR on more
  nodes costs more than it saves.
- **D.6.4 H3, sparse:** the TT move is searched first, so the least-bad move is
  usually the stored one already, and few stores change anything.

**Interactions.** `lmr_cut_node_no_tt_move` reads both items: D.6.3 changes
which nodes are cut nodes, and D.6.4 which have a TT move. The cut-node TT
admission reads `Upper` entries, whose moves D.6.4 changes. Hence the arms below
include each item alone and both together.

**Invariants.** Arm 0 is the source as it is. Each arm changes only what its bit
names. The counters never steer.

## The screen (frozen)

`analysis/arm_patches/d63-d64-rar-s109-screen.patch`, on `ee0b3e4`, adds `diag`
counters and one arm switch: `RAROG_S109_ARM`, a bit mask read once per process
(absent or 0 is the source as it is). Bit 1 types the null child `false`. Bit 2
types both ProbCut searches `false`. Bit 4 makes an `Upper` store pass no move,
so the table keeps the stored one. **Arms 0, 1, 2, 3, 4 and 7.** Arm 0's
`bench 13` must reproduce 13,038,236. Arms 1, 2 and 4 must each move it (a wire
check: a dead bit reads equal).

**Counters,** one thread, stride 1, summed by `tools/diag/bench_counters.py`.

D.6.3, per child class: the searches, and those whose child failed high (its
result at or above its own beta, read from the parent's side).
- `ch_null_n`, `ch_null_fh`: the null-move child.
- `ch_pc_n`, `ch_pc_fh`: both ProbCut verification searches.
- `ch_lmr_n`, `ch_lmr_fh`: the reduced LMR search, typed `true` (the reference).
- `ch_zw_cut_n`, `ch_zw_cut_fh`, `ch_zw_all_n`, `ch_zw_all_fh`: the move loop's
  other zero-window searches, by the type passed.
- `ch_null_nodes`, `ch_pc_nodes`: interior nodes inside the outermost null-move
  and ProbCut subtrees (a nested one counted once), against `nodes`.

D.6.4:
- `ups_n`: the `Upper` stores.
- `ups_move`: of those, the stores with a move. `ups_same`: the move equals the
  node's TT move. `ups_replace`: a different move where the node had one.
  `ups_fill`: a move where it had none.
- `tt_upper_move`: non-root nodes with no excluded move whose TT move came from
  an `Upper` entry.
- `iir_kept_by_upper`: such nodes where that move alone kept IIR off.
- `razor_kept_by_upper`: such nodes where that quiet move alone blocked
  razoring.
- `lmr_n`, `lmr_no_tt`, `lmr_cut`, `lmr_cut_no_tt`: at each LMR computation.

The existing `iir_applied`, `razor_drop`, `nmp_attempt`, `nmp_cut`,
`probcut_attempt` and `probcut_cut` are read alongside.

**Per arm:** `bench 13` nodes and EBF, the counters, WAC at 100k nodes
(`tools/diag/fixed_budget_probe.py nodes 100000 src/wac.epd`), and the canary
rule (PLAN's: a canary passes when the `depthpv 10` probe is stable no later
than the oracle's stable depth plus two and the 100k probe solves it, over the
116 oracle-anchored canaries in `tools/results/b0-20260913/`, by
`b41_canaries.py`'s rule). Arm 0's passes are the ones that may not be lost.

**Exposure.** Every search of depth 3 or more reaches a null-move decision,
every one of depth 4 or more a ProbCut decision, and every fail-low node a store.
So every gate game is in the domain of both items. The bench subtree shares
above measure how much of the tree each one governs.

### Readiness (frozen)

The read is void, and both items return to research, if `ch_lmr_fh` is under
70% of `ch_lmr_n`. Then the instrument, or the premise that a correctly typed
`true` node fails high, is wrong.

**D.6.3**, each half on its own:
- **ProbCut half:** `READY_FOR_IMPLEMENTATION` if `ch_pc_fh` is under 30% of
  `ch_pc_n` and `ch_pc_nodes` is at least 1% of `nodes`. It closes `NO_CHANGE`
  at 50% or more, or under 1% of `nodes`. Between 30% and 50% it stays in
  `RESEARCH`, ambiguous.
- **Null half:** `READY_FOR_IMPLEMENTATION` if `ch_null_fh` is under 50% of
  `ch_null_n` and `ch_null_nodes` is at least 1% of `nodes`. It closes
  `NO_CHANGE` at 50% or more, where the typing matches the behaviour (PLAN's
  split), or under 1% of `nodes`.

**D.6.4** closes `NO_CHANGE` if `ups_replace + ups_fill` is under 1% of `ups_n`,
or `tt_upper_move` is under 0.5% of `nodes`. Otherwise it is
`READY_FOR_IMPLEMENTATION`.

**The canary rule** applies to each arm an implementation would take: arm 1, 2
or 3 by D.6.3's verdict, and arm 4. An arm that loses a canary that arm 0 passes
keeps its item in `RESEARCH`, with the positions named.

Each item that ends `READY` is implemented and gated on its own, D.6.3 first
and D.6.4 on its result, each gate registered before any game (PLAN: case 3,
`[-5,5]`, or `[0,3]` where the registration predicts a gain). The arms' benches
and WAC counts are reads for that registration, not acceptance evidence.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. `ch_lmr_fh` 85% to 97% of `ch_lmr_n`, so the read stands. 0.8.
2. `ch_zw_cut_fh` at least 70% of `ch_zw_cut_n`. 0.7. `ch_zw_all_fh` 10% to 50%.
   0.6.
3. `ch_pc_fh` 1% to 8% of `ch_pc_n`. 0.8. ProbCut subtrees 2% to 10% of
   `nodes`. 0.6. The ProbCut half is `READY`. 0.75.
4. `ch_null_fh` 25% to 60%, near 40%. 0.6. Null subtrees 10% to 35% of `nodes`.
   0.6. The null half is `READY`. 0.55.
5. Arms 1 and 2 each move `bench 13` by 0.5% to 8%, in a direction not predicted.
   0.6.
6. `ups_move` at least 90% of `ups_n`; `ups_same` 30% to 60% of `ups_move`;
   `ups_replace + ups_fill` 20% to 70% of `ups_n`. 0.6. `tt_upper_move` 3% to 15%
   of `nodes`. 0.6. D.6.4 is `READY` before the canary rule. 0.8.
7. Arm 4 searches fewer nodes than arm 0 (more IIR). 0.6.
8. Arm 7's node ratio to arm 0 is within 1% of the product of arm 3's and arm 4's.
   0.6.
9. An arm the rule applies to loses at least one canary. 0.5 for each.
10. **Most likely failure mode:** the null children fail high at 40% to 60%,
    near the split threshold. Then the null half's verdict rests on a rate that
    the bench's 40 positions fix only loosely.
