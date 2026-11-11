# RAR-S119 — D.6.5's direction read: does the plain estimate make the exclusion search's decision more accurate, paired at the same nodes on replayed gate games

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, after [RAR-S118](RAR-S118.md) and
  before any probe record of the deciding state was read. The read is the
  agent's (deterministic, no games).
- **Kind:** the zero-game direction read that RAR-S118's rule names. It
  decides D.6.5's readiness, or closes it.
- Disposition: no-change — read 2026-11-11: the forms' decisions differ at 1.96% of sampled
  attempts (rule 1, under 2%); where they differ the plain form is right 747 to 273 and 5.7%
  cheaper, too rare to gate. D.6.5 closes `NO_CHANGE`.
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
- **Patch and binary** (as amended, below).
  `analysis/arm_patches/d65-rar-s119-probe.patch` (SHA-256 `4e1e37fb…`) on
  `8a33096`. Its source tree is `ee59b762…`. It is built with
  `--features pricing` only, as
  `analysis/artifacts/d65-direction-2026-11-11/rarog-s119probe.exe`
  (`80c67734…`). `RAROG_S119_SHIFT` sets the sample (1 in `2^shift`
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
  (SHA-256 `c4b732fa…` as amended; `7a43d18d…` at registration): `replay off`,
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

## Amendment, before any deciding record was read (2026-11-11)

**The first probes-on run was stopped.** It had run 9.4 minutes and
written 65,142 records in all 60 games. Three engines then went 4 to 8
minutes without a record: games 22, 34 and 47. Game 34 ends in the deep
endgame that stalled the withdrawn form. The depth-1 reference probe has no
node limit, and at a node of depth 30 or more it can run for many minutes.
The records were moved unread to
`analysis/artifacts/d65-direction-withdrawn/rec_uncapped/`. Nothing in them
was looked at beyond file times, line counts and the depth fields of each
file's last line.

**The change.** Each probe stops at 2,000,000 nodes. A record with any
probe stopped is void and left out. The cap stops only the probe: the
measured search's `stopped` is restored, as its node count already was.
`RAROG_S119_CAP` overrides the cap, for the plant test only.

**Proved before the rerun:**
- **Plant:** at a cap of 50, 1,243 of the bench's 6,133 records are void,
  and `bench 13` still reads 13,038,236.
- **The real cap:** at 2,000,000, none of the bench's records is void, and
  `bench 13` reads 13,038,236.
- **Wire 0 on the capped binary:** the probes-off replay still equals
  RAR-S117's arm 0 in all 3,670 searches.

**Added wire 5:** under 5% of written records are void by the cap.
Otherwise the read is void, since leaving out the deepest nodes would bias
it. Nothing else changes: the quantity, the sample, the rules and the
predictions are as registered.

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

## Result: the read (2026-11-11)

Read by `read_dir.py` (SHA-256 `c4b732fa…`, unchanged since the amendment) on
`rarog-s119probe.exe` (`80c67734…`). Logs and records are beside it:
`replay_off.json`, `replay_on.json`, `rec/game_*.rec`, `replay_on.log` and
`report.log`. The probes-on run took 305 s.

**Wire: passed.**

| wire | rule | read |
|---|---|---|
| 0, the state | probes off equal RAR-S117's arm 0 | 0 of 3,670 differ |
| 1, isolation | probes on equal probes off | 0 of 3,670 differ |
| 2, identity | P0 decides as the real search, 90% or more | 99.53% |
| 3, liveness | P0 and P1 nodes differ, 10% or more | 13.23% |
| 4, power | 3,000 or more with an agreeing reference | 64,157 |
| 5, the cap | under 5% of records void | 7 of 65,335 (0.01%) |

| 65,328 records kept, 60 games | |
|---|---:|
| the reference agrees between the forms | 98.21% |
| **F, the forms' decisions differ, of all records** | **1.96%** |
| the same where the reference agrees | 1.77% |
| error against the reference, refined / plain | 30.24% / 29.51% |
| wrong only under refined / only under plain | 747 / 273 |
| Δ, refined error minus plain error | +0.739 points, 95% [+0.630, +0.855] |
| cost, plain-form probe nodes over refined | 0.9426 |
| binary verdict alone: differs; Δ | 0.78%; +0.356 [+0.280, +0.437] |
| exclusion depth 4 or less: n, differ, discordant | 60,437, 1.78%, 713 / 250 |
| exclusion depth 5 to 7 | 3,388, 1.80%, 34 / 23 |
| exclusion depth 8 or more | 332, 0.00%, 0 / 0 |
| the reference probe's cost | 6.0 times the refined probe's |

**Decision, by the frozen rule: `NO_CHANGE`** (rule 1). The two forms'
decisions differ at 1.96% of sampled singular attempts, under 2%. Rules 2 to
5 are not reached.

**What the data say, beyond the rule.**
- **The direction is real, and it is the donor's.** Where the decisions
  differ, the plain estimate matches the deeper search 747 times against 273.
  Its probes are also 5.7% cheaper. Had rule 1 not applied, rule 4 would have
  made the item ready: Δ's lower bound is above 0 and the cost under 1.10.
- **The size is what closes it.** 98% of decisions are identical. The net
  gain is 0.74 points of decisions, at singular attempts only, and those are
  4.4% of interior nodes. That matches RAR-S118's unchanged outcome shares and
  depth. The rule exists because a gate cannot resolve an effect this small.
  A `[0,3]` SPRT at a true value near zero runs to its cap, and RAR-M86's
  table puts a revert near one in three.
- **The margin is narrow,** 1.96 against 2.00. The rule was frozen before
  exposure and is not reopened after it. What would change the decision is a
  change that makes the forms differ more often, which is the retry trigger
  below.
- **The change acts only shallow.** There are no differences at exclusion
  depth 8 or more, since LDSE stops at depth 7.

**Retry trigger.** Any change to LDSE's depth or margin, to razoring's
margins, to the singular margins or to the alpha-gap term, or the search
re-fit (F.6). Then re-read F with this probe. Only F at 2% or more reopens the
item, and then with this read's rules.

## Calibration (appended after the read)

1. The wires pass: hit. Identity at 90% or more: hit (99.5%).
2. F 5% to 20%: missed in magnitude (1.96%). LDSE fires at a quarter of
   exclusion nodes (RAR-S118), but its extra ply changes the decision in only
   one case in about twelve. The prediction took firing for change: this is
   RAR-S118's registered failure mode, confirmed per node.
3. The reference agrees 90% or more: hit (98.2%).
4. P0's error 15% to 35%: hit (30.2%).
5. Δ within ±1 point: hit (+0.74). The sign was unpredicted; it favours the
   plain form.
6. Cost 1.00 to 1.15: missed in sign (0.943). The assumption that LDSE's
   extension would dominate the cost failed. A likely cause, not measured
   here: where the eval is below alpha, the plain estimate's larger
   alpha-gap term reduces quiet moves more, and that saves more than the
   extension spends.
7. `NO_CHANGE` 0.55: hit.
8. The reference-bias failure mode could not be read directly. With the
   references agreeing at 98%, its room is small.
