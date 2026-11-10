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

## Result: the screen (2026-11-10)

Read by `analysis/artifacts/d63-d64-2026-11-10/read_screen.py` on
`rarog-s109screen.exe` (SHA-256 `dcc3a50b…`, built from the screen patch on
`ee0b3e4`; applying the patch to `ee0b3e4` reproduces the screen's source tree,
`a4fa1d9f…`). Log `read.log` and `read.json` beside it. Every counter dumped 40
times in every arm.

**Wire.** Arm 0 reproduced 13,038,236. Every other arm moved it:

| arm | `bench 13` | change | WAC at 100k | canaries | lost against arm 0 |
|---|---:|---:|---:|---:|---|
| 0, as it is | 13,038,236 | — | 239 | 93 | — |
| 1, null child `false` | 12,385,895 | −5.00% | 233 | 93 | none |
| 2, ProbCut children `false` | 13,148,505 | +0.85% | 240 | 91 | WAC.115, WAC.182 |
| 3, both | 14,045,048 | +7.72% | 241 | 92 | WAC.182 |
| 4, `Upper` stores no move | 12,785,308 | −1.94% | 237 | 92 | WAC.069, WAC.182 |
| 7, all three | 9,094,587 | −30.25% | 245 | 94 | WAC.069 |

Arm 4 gained WAC.295. Arm 7 gained WAC.168 and WAC.295. Every lost canary is
still solved at 100k. Each was lost because the depth-10 probe settles later:
- WAC.069 settles at depth 6 against a limit of 3.
- WAC.115 settles at depth 9 against a limit of 3.
- WAC.182 settles at depth 9 against a limit of 8.

**Child types,** arm 0:

| child | searches | fail high |
|---|---:|---:|
| null move, typed `true` | 174,567 | **31.19%** |
| ProbCut, typed `true` | 39,322 | **2.15%** |
| reduced LMR, typed `true` (the reference) | 3,887,769 | 93.56% |
| other zero-window, typed `true` | 2,424,942 | 80.01% |
| other zero-window, typed `false` | 3,123,724 | 41.99% |

- **Subtrees.** Inside the outermost null-move subtrees were 0.52% of the
  8,416,796 interior nodes, and inside ProbCut's 1.44%.
- **Rates.** Null moves cut at 68.74% of attempts, and ProbCut at 75.93%.
- **Arm 1.** The null-move cut rate rose to 74.83%, and the null child's
  fail-high rate fell to 25.10%.

**The fail-low move,** arm 0:
- **Stores.** Of 1,804,780 `Upper` stores, every one carries a move. 34.24% of
  them store the node's TT move again, 10.85% replace a different one, and
  54.91% fill an empty slot, so 65.76% change the stored move.
- **Next visit.** A TT move from an `Upper` entry is present at 10.27% of
  interior nodes. That move alone keeps IIR off at 338,465 of them, and blocks
  razoring at 2,879.
- **Arm 4.** `iir_applied` rises from 1,202,169 to 1,423,877, `razor_drop` falls
  from 86,333 to 79,410, and LMR's share without a TT move goes from 25.60% to
  65.54%. At cut nodes it goes from 11.17% to 14.42%.

**Interaction.** Arm 7's node ratio to arm 0 is 0.6975. The product of arm 3's
and arm 4's is 1.0563, so the two items together search 34% fewer nodes than
their separate effects predict.

### Readiness, by the frozen rules

- **The read stands:** the reduced LMR children fail high at 93.56%, above the
  70% that would void it.
- **D.6.3, ProbCut half:** 2.15% fail high, under 30%, and 1.44% of nodes, over
  1%. That passes. The canary rule then applies to arm 2, which loses WAC.115
  and WAC.182. **The half stays in `RESEARCH`,** with those positions named.
- **D.6.3, null half:**
  - 31.19% fail high, under 50%, so the type is wrong for most null children.
  - The subtree is 0.52% of nodes, under 1%, and the sparseness rule would
    close the half `NO_CHANGE`.
  - That rule's premise was that the subtree share measures what the type
    governs. Arm 1 refutes it: typing the null child alone moves `bench 13` by
    −5.00% and the null-move cut rate by six points. Most null children go
    straight to quiescence, so the interior subtree is small, but the child's
    result decides whether the parent searches its moves at all.
  - A false premise returns the half to `RESEARCH` (AGENTS). The closure is
    recorded as fired on a void measure, not as `NO_CHANGE`.
- **D.6.4:** 65.76% of `Upper` stores change the move, over 1%, and the moved
  TT move is at 10.27% of nodes, over 0.5%. That passes. The canary rule then
  applies to arm 4, which loses WAC.069 and WAC.182. **It stays in `RESEARCH`,**
  with those positions named.

Neither item crosses `READY_FOR_IMPLEMENTATION`. Two findings matter for the
next registration:
- **One cluster.** The −34% interaction says the items are not separable in
  this engine. Gated one after the other, each would be read on a tree the
  other reshapes. They are one candidate cluster.
- **The canary losses.** None of them is a position the engine stops solving.
  Each is a later settle on the depth-10 probe. The rule, as PLAN states it,
  does not tell that apart from a lost tactic. Whether it should is a question
  for that registration, not a rescue inside this one.

**Calibration (screen).**

- 1, the reduced LMR children at 85% to 97%: a hit at 93.56%.
- 2, `true`-typed zero-window searches at least 70%, `false`-typed 10% to 50%:
  hits at 80.01% and 41.99%.
- 3, ProbCut children 1% to 8%: a hit at 2.15%. Its subtrees at 2% to 10%: a
  miss at 1.44%. `READY` (0.75): it passed both measures and was held by the
  canary rule, which the prediction did not weigh.
- 4, null children 25% to 60%: a hit at 31.19%. Null subtrees 10% to 35%: a
  miss by a factor of 20 (0.52%). The prediction counted the null child's tree
  as interior nodes, but at the reductions null moves take, most of it is
  quiescence. A miss in the instrument, as the readiness section says.
- 5, arms 1 and 2 moving `bench 13` by 0.5% to 8%: hits at −5.00% and +0.85%.
- 6, the fail-low stores: hits on every leg (all stores carry a move, 34.24%
  the same move, 65.76% changed, 10.27% of nodes reached).
- 7, arm 4 fewer nodes: a hit at −1.94%, with IIR applied 18% more often.
- 8, arm 7 within 1% of the product: a miss in interaction, −34%. Nothing in the
  registration foresaw a coupling this large.
- 9, a canary lost in each arm the rule applies to: both happened (0.5 each).
- 10, the named failure mode, null children near the split threshold: it did not
  occur. They failed high at 31%, well inside one side.
