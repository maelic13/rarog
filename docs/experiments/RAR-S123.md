# RAR-S123 — D.6.8's screen: how much of the node count is the same position counted again

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-12, before its counts were read. The
  read is the agent's: no games and no timing.
- **Kind:** a zero-game readiness screen for a repair to a counting contract.
- **Baseline:** `d6` at `5f9a4a2`, engine behaviour of `aee0b3b`, bench
  11,477,040 / EBF 2.537.

## The question

`check_stop` counts a node each time `negamax` or `quiescence` is entered.
`negamax` counts its node before it tests `depth <= 0`, then hands off to
`quiescence`, which counts the same position again. Three other paths enter
the same position again without a move:
- razoring's quiescence search;
- the null-move verification search;
- the singular exclusion search.

Every count lands in `td.nodes`, which is the reported `nodes` and `nps`, the
`go nodes` limit, the bench count, the LMR spread's phase, the time manager's
effort share and the poll cadence. The donor counts once per move made.

The screen asks how large the double count is, by path.

## The four questions (AGENTS)

- **Mechanism.** The reported count overstates the positions the search
  visits. `go nodes N` searches fewer positions than N, and `nps` reads high.
- **Interactions.** The consumers of `td.nodes` are:
  - **The LMR spread:** `(nodes + 27 × thread) % 128` dithers each reduction.
    A different count moves its phase, not its distribution.
  - **The effort share:** a root move's share of the thread's nodes. Both its
    sides carry the double count, so only a difference in re-entry rates
    between the best move's subtree and the rest moves it.
  - **The poll cadence and the pool's node batch:** both run every fixed
    number of counted nodes, so fewer counts space them further apart in
    positions.
  - **Node-limited runs:** RAR-S117's replays at the games' recorded counts,
    `match-fixed-nodes.toml` and datagen's `go nodes`. A recorded count
    means the same search only within the counting rule that produced it.
- **Invariants.**
  - Every position the search enters by a move or a null move is counted
    exactly once.
  - The node limit still stops the search.
  - A pool's reported count is still the sum of its threads'.
  - A test pins each.
- **Falsifier.** The same-position re-entries are under 1% of `td.nodes` on
  `bench 13`. The count then misstates nothing a user or an instrument acts
  on, and a new fingerprint and a broken comparability with recorded node
  counts would buy nothing.

## The read (frozen)

- **Instrument:** one exact `diag` counter, `q_handoff`, at `negamax`'s
  hand-off to `quiescence`. It is applied as
  `analysis/arm_patches/d68-rar-s123-screen.patch` to `aee0b3b`'s source on
  `d6` and built with `--features diag` only. Razoring's quiescence calls are
  `razor_drop`, the verification searches `nmp_verify_attempt` and the
  exclusion searches `singular_attempt`, all at stride 1.
- **Wire:** the instrumented build's `bench 13` reads 11,477,040. The counts
  then obey `nodes + qnodes + q_handoff ≤ td.nodes`, and the remainder is the
  early returns: stops, the ply cap, draws by rule and mate-distance cuts.
- **Run:** `python tools/diag/bench_counters.py` at depth 13, stride 1, with
  the counters summed over the 40 positions.
- **Quantities,** each as a share of `td.nodes` (bench's "Nodes searched"):
  H for the hand-off, R for razoring, V for verification, X for exclusion,
  and E = H + R + V + X.

## Decision rule (frozen)

- **E under 1%: `NO_CHANGE`.** D.6.8 closes, with a retry only if a new
  same-position entry is added.
- **E at 1% or more: ready for implementation.** The fix counts each
  position the search enters by a move or a null move once:
  - the hand-off tests `depth <= 0` before counting;
  - razoring, verification and exclusion re-enter without counting.

  Every consumer reads the one corrected count. No separate clock keeps the
  LMR spread's phase or the effort share's old units: neither phase carries a
  meaning to keep.

  **Gate:** case 2, a pre-registered 2,000-game fixed read with the −13 nElo
  harm rule. In the rated configuration play changes only through the
  spread's phase, the effort share's ratio and the poll cadence, so the
  change is expected near zero. It is registered alone, never folded into a
  strength change.

## PRE-REGISTERED PREDICTION (frozen before any read)

1. E is 25%, 80% band [12%, 40%]. Most depth-1 children are searched at
   depth 0, and quiescence's nodes are a large share of the tree.
2. H is at least 85% of E: 0.7. R is about 1%, X under 0.5% and V under
   0.05% of `td.nodes`.
3. `NO_CHANGE`: 0.03.
4. **Most likely failure mode:** a counter in the sum is not exact at stride
   1, or one of the three re-entry counters does not sit on every path into
   its search. The wire's inequality or a direct look at each call site then
   fails, and the read stops until the counter is fixed.
