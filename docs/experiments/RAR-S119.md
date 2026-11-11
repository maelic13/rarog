# RAR-S119 — D.6.5's direction read: does the plain estimate make the exclusion search's decision more accurate, paired at the same nodes on replayed gate games

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, after [RAR-S118](RAR-S118.md) and
  before any probe record of the deciding state was read. The read is the
  agent's (deterministic, no games).
- **Kind:** the zero-game direction read that RAR-S118's rule names. It
  decides D.6.5's readiness, or closes it.
- **Baseline:** `d6` at `8a33096`, engine behaviour of `ee0b3e4`, bench
  13,038,236 / EBF 2.540.
- **Inherits:** RAR-S118. Under the donor's plain estimate, LDSE fires at
  23.8% of exclusion nodes game-like, and never under the refined one. Yet no
  singular outcome's share moves by 0.15 points, and the replay's depth moves
  by −0.03. Unchanged shares do not exclude decisions that flip in both
  directions at once.

## The question and the quantity

What the change should improve, with its consumers held equal, is the
exclusion search's answer. The parent turns that answer into one decision:
- extend the TT move by one, two or three plies;
- multicut;
- demote the TT move;
- reduce it (the negative extension);
- or none of these.

**The quantity.** At sampled singular attempts in the running search, four
exclusion searches run at the same node, from the same state:
- **P0 and P1:** the search's own exclusion depth, `(depth - 1) / 2`, under
  the source's refined estimate (P0) and the donor's plain estimate (P1);
- **R0 and R1:** the same two estimates at the reference depth, `depth - 1`,
  the depth the TT move's alternatives would get.

Each score is mapped to the decision the parent would take, by the parent's
own functions (the extension budget aside). The **reference decision** is
R0's, where R0 and R1 agree. Where they disagree, the reference itself
depends on the change and the record is left out. A form's **error** is the
share of decisions that differ from the reference. The deciding quantity is
**Δ = error(P0) − error(P1)**: positive means the plain estimate decides
better. Its 95% interval comes from a bootstrap over the 60 games, 10,000
resamples, seed 119. The **cost** is P1's nodes over P0's.

## Instrument

- **Isolation.** The probes are the D.4.4 pricing probe's isolation (the
  pruning pricing card, section 3), unchanged: TT stores go to a scratch
  table, histories and corrections are frozen, and the stack, PV rows,
  nodes, selective depth and `nmp_min_ply` are restored. One addition: a
  probe counts its nodes without the stop checks, so neither a node limit
  nor the poll cadence sees it. Each probe starts on an empty scratch table,
  so no probe reads another's results. The probes run before the real
  exclusion search, and so leave it the state it would have had.
- **Patch and binary.** `analysis/arm_patches/d65-rar-s119-probe.patch`
  (SHA-256 `e1c5725c…`) on `8a33096`. Its source tree is `b0b09eaf…`. It is
  built with `--features pricing` only, as
  `analysis/artifacts/d65-direction-2026-11-11/rarog-s119probe.exe`
  (`c23d2acb…`). `RAROG_S119_SHIFT` sets the sample (1 in `2^shift`
  singular attempts, drawn from the position's key and depth). Unset, it is
  off. `RAROG_S119_OUT` names the record file. The plain estimate is set
  only inside a probe, and there for every exclusion node the probe reaches.
- **The state.** RAR-S117's frozen game set: the first 60 games of
  RAR-S115's gate, the baseline's moves, one process per game, Hash 64, one
  thread, `ucinewgame` once. Each position is searched with `go nodes` at
  the count the game recorded, exactly as RAR-S117 did. Sample shift 8.
- **A first form, withdrawn before any probe ran.** It searched to the
  game's recorded depth, capped at 40, so that probe nodes could not end a
  search. In one game a capped depth that the game had reached cheaply ran
  past 20 minutes. The form was stopped there, with no probe record
  written, and replaced by the node-count isolation above.
- **Script.** `analysis/artifacts/d65-direction-2026-11-11/read_dir.py`
  (SHA-256 `7a43d18d…`, frozen with this registration): `replay off`,
  `replay on`, `report`.

## Wire

0. **The state:** with the probes off, every replayed search equals
   RAR-S117's arm 0 in move, depth and nodes at each depth. This already
   passed, 0 of 3,670 differing.
1. **Isolation:** with the probes on, every replayed search equals the
   probes-off one in move, depth and nodes at each depth. The bench already
   passed this: 13,038,236 both ways, with 6,133 records at shift 6.
2. **Identity:** P0's decision equals the real exclusion search's in at
   least 90% of records. They differ only in where the probe's stores go.
3. **Liveness:** P0's and P1's node counts differ in at least 10% of
   records.
4. **Power:** at least 3,000 records with an agreeing reference.

## Exposure, stated

Seen before registration:
- `bench 13` with the probes off and on;
- the bench smoke's record count and its first three records;
- the probes-off replay's moves and node counts, which hold no probe
  output;
- the withdrawn depth-capped form's timing, also with no probe output.

Unexposed: every record of the replay with the probes on.

## Decision (frozen), applied in this order

**Void** if a wire fails. Then:
1. **`NO_CHANGE`** if the two forms' decisions differ at under 2% of all
   records (F). The change rarely changes a decision, so a gate has nothing
   to find.
2. **`NO_CHANGE`** if Δ's interval lies wholly below 0. The plain estimate
   decides worse.
3. **`NO_CHANGE`** if Δ's upper bound is under +0.5 points and the cost is
   0.95 or more. At best the gain is under half a point of decisions, with
   no saving. That floor is set before exposure. RAR-S118 saw no singular
   outcome move by 0.15 points, so a smaller gain could not carry a gate's
   effect.
4. **`READY_FOR_IMPLEMENTATION`** if either holds:
   - Δ's lower bound is above 0 and the cost is at most 1.10;
   - the cost is at most 0.95 and Δ's lower bound is above −0.5 points.

   The gate, registered with the implementation, keeps PLAN's rule: `[0,3]`,
   since LDSE fires at over 2% of exclusion nodes.
5. **`RESEARCH`** otherwise. An example is a gain bought with more than 10%
   more exclusion-search nodes. No further zero-game screen is registered
   for D.6.5 then. The maintainer chooses between a gate and closing the
   item.

**Reported, not deciding:**
- the binary singular verdict alone (score below `singular_beta`), with its
  own interval;
- the same counts by exclusion depth and by node type;
- how often the reference agrees between the forms;
- what the reference costs.

## The four questions (AGENTS)

- **Mechanism, interactions and invariants:** as RAR-S118. This read adds
  the invariant that the probes leave the measured search unchanged (wire 1).
- **Falsifier:** rules 1 to 3 above. They are the cheapest observations that
  end the item.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The wires pass. 0.85. Identity is at 90% or more. 0.8.
2. F is 5% to 20%. 0.6.
3. The reference agrees between the forms in 90% or more of records. 0.7.
4. P0's error is 15% to 35%. 0.5.
5. Δ is within ±1 point. 0.6. Its sign is not predicted: 0.5 each way.
6. The cost is 1.00 to 1.15, since LDSE extends a move inside the search.
   0.6.
7. The decision is `NO_CHANGE` 0.55, `RESEARCH` 0.25,
   `READY_FOR_IMPLEMENTATION` 0.20.
8. **Most likely failure mode:** the reference shares the estimate's own
   bias. A depth-1 search under either form carries the same blind spot, so
   agreement with it rewards the form it resembles. Keeping only the
   records where the two references agree limits this, and does not remove
   it.
