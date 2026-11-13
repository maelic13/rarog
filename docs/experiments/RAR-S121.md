# RAR-S121 — D.6.7's screen: how often searches reach the ply cap at three time controls, and what drives a line there

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11, before any of its cap counts was
  read. The read is the agent's (deterministic, no games).
- **Kind:** a zero-game screen for a correctness defect at the search's edge.
- Disposition: observation — read 2026-11-11: READY_FOR_IMPLEMENTATION, the form "bound the lines";
  the cap is reached in 1.96% of searches at `60+0.6`'s node counts (none at `3+0.03`'s), and
  depth gained outside the extension budget carries 88% of those lines' excess plies.
- **Baseline:** `d6` at `01b3d2b`, engine behaviour of `ee0b3e4`, bench
  13,038,236 / EBF 2.540.

## The question

At ply 127, `negamax` and `quiescence` return the corrected static eval. They
do so before any draw or check handling: a node in check, or a draw by rule,
gets an eval. The donor's cap is 240, and it returns a draw in check.

The extension budget bounds a line at about twice the iteration depth plus
quiescence, so the cap should bind only from about iteration 56. Yet:
- bench position 11 reaches selective depth 127 at depth 19 to 20 (D.1.1);
- RAR-S108's promotion race,
  `8/2p2P1P/2n2P2/1k6/p7/6N1/P5K1/8 b - - 0 44`, reaches it by depth 10.

So something outside the budget lengthens lines. The PLAN screen asks how
often the cap is reached in games at `3+0.03`, `10+0.1` and `60+0.6`, with
the in-check share, and which mechanism drives the race position's lines.

## The decomposition

At a cap node at ply `p` with `d` depth left (0 in quiescence),
`p + d = R + E + Q + U`. The terms:
- **R**, the iteration's depth;
- **E**, the positive extensions the line spent from the budget;
- **Q**, the quiescence plies;
- **U**, the rest: depth gained outside the budget, net of reductions.

U's sources, by the code:
- the late-move reduction's room above the nominal depth (up to two plies,
  and two more on a PV line);
- the re-search's extra ply;
- hindsight's extra ply.

**The mechanisms and their forms.** Each points to its own repair:
- **E dominating:** raise the cap, with the band constants PLAN lists.
- **Q dominating:** bound quiescence's check chains, or handle check at
  the cap as the donor does.
- **U dominating:** bound the lines, Triumviratus's form TS-1.

## The four questions (AGENTS)

- **Mechanism.** A line that reaches the cap ends on a static eval, even in
  check or at a draw by rule. What a repair improves is that line's score.
- **Interactions.** The repair's form depends on the mechanism:
  - **Raising the cap** moves the score band and the `i8` table depth.
  - **Bounding the lines** moves the reductions and extensions that feed U.
  - **Bounding quiescence** moves its check evasions.
- **Invariants.** The instrument changes no node count: `bench 13` reads
  13,038,236, seen before registration.
- **Falsifier.** If the cap is not reached in games even at twenty times
  their node counts, the defect cannot reach play. The rules below say when.

## Instrument

- **Patch and binary.** `analysis/arm_patches/d67-rar-s121-screen.patch`
  (SHA-256 `06ce42d8…`) on `01b3d2b`. Its source tree is `7e3e7efd…`. It is
  built with `--features diag` only, as
  `analysis/artifacts/d67-cap-2026-11-11/rarog-s121screen.exe`
  (`21c18113…`).
- **Counters,** at each cap node:
  - the count;
  - whether the node is in quiescence, and past quiescence's 16 plies;
  - whether it is in check;
  - whether it is a draw by rule;
  - the sums of R, E, Q, U and the depth left;
  - the count of negative U.
- **Script.** `analysis/artifacts/d67-cap-2026-11-11/read_cap.py`
  (SHA-256 `7a777cbc…`, frozen with this registration). It reuses RAR-S117's
  replay wrapper and game set unchanged.
- **States.**
  - **Time controls:** RAR-S117's 60 gate games, the baseline's moves, Hash
    64, one thread, `go nodes` at the recorded count times 1, 10/3 and 20.
    These stand in for `3+0.03`, `10+0.1` and `60+0.6`.
  - **A deviation from PLAN, stated:** the positions are the gate games',
    not the rating pool's. Longer time controls are proxied by node counts
    on the same positions and table history; no new games are played.
  - **The race position:** a fresh process, `go depth 10`.

## Wire

1. `bench 13` reads 13,038,236. Passed before registration.
2. The ×1 replay equals RAR-S117's arm 0 in every search.
3. The race position reaches the cap. Seen in RAR-S108 on the source; here
   it proves the counter is live.

## Exposure, stated

Seen: `bench 13`'s node count, from the same run that wrote the cap counters
to `bench.log`; none of those counters was read. Unexposed: every cap count.

## Decision (frozen)

X_k is the share of replayed searches with at least one cap node at scale k.
The ×20 state decides, being the longest.
- **Void** if a wire fails.
- **`NO_CHANGE`** if X_20 is under 0.1% (fewer than 4 of 3,670 searches). The
  cap does not reach play even at `60+0.6`'s node counts. The race position
  then stays a recorded pathology: its retry trigger is a game record that
  shows the cap reached.
- **`READY_FOR_IMPLEMENTATION`** if X_20 is 0.1% or more and one of E, Q and
  U carries 60% or more of the ×20 cap nodes' plies beyond R. The form
  follows that mechanism.
  - The gate is PLAN's: case 1 if X_1 is 0, else an LTC harm read.
  - The implementation's registration states the in-check and draw shares
    at the cap as its exposure.
- **`RESEARCH`** otherwise: the cap is reached and no single mechanism owns
  it.

**Reported, not deciding:** X_1 and X_10/3, the in-check and draw shares at
the cap, the race position's decomposition, and bench's cap counts.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The wires pass. 0.9.
2. X_1 is under 0.1%. 0.7. X_20 is 0.1% or more. 0.5.
3. In the race position, U carries 60% or more of the plies beyond R. 0.45.
   Q does: 0.3. E does: 0.1.
4. If the cap is reached in the replay, the mechanism is the race
   position's. 0.6.
5. In check at the cap: 10% to 40% of cap nodes, where any are reached.
   0.5.
6. The decision: `NO_CHANGE` 0.45, `READY_FOR_IMPLEMENTATION` 0.35,
   `RESEARCH` 0.2.
7. **Most likely failure mode:** the gate games hold too few long endgames
   and races. The ×20 proxy then reads near zero, although the cap binds in
   the rarer positions that the race position stands for.

## Result: the read (2026-11-11)

Read by `read_cap.py` (SHA-256 `7a777cbc…`, unchanged since registration) on
`rarog-s121screen.exe` (`21c18113…`). Logs and counts are beside the script:
`bench.log`, `race.json`, `replay_x1.json`, `replay_x10_3.json`,
`replay_x20.json`, `run.log` and `report.log`. Every search dumped each
counter once.

**Wire: passed.**
1. `bench 13` reads 13,038,236.
2. The ×1 replay equals RAR-S117's arm 0 in all 3,670 searches.
3. The race position reaches the cap, at selective depth 127.

| state | searches reaching the cap | games | cap nodes | in check | draw by rule | R | E | Q | U | E / Q / U of plies beyond R |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| ×1 (`3+0.03`) | 0 of 3,670 | 0 | 0 | — | — | — | — | — | — | — |
| ×10/3 (`10+0.1`) | 4 (0.109%) | 3 | 1,217 | 93.2% | 2.5% | 11.5 | 5.6 | 0.0 | 111.0 | 4.8 / 0.0 / 95.2% |
| **×20 (`60+0.6`)** | **72 (1.962%)** | **22** | 1,589,474 | 22.2% | 15.0% | 14.9 | 13.3 | 0.0 | 100.7 | **11.7 / 0.0 / 88.3%** |
| the race position, depth 10 | — | — | 7,345,137 | 0.3% | 1.3% | 9.8 | 7.5 | 0.1 | 110.7 | 6.4 / 0.1 / 93.5% |

In quiescence: 1.0% to 12.4% of cap nodes, never past its 16 plies. Negative
U: 83 of 1.59M cap nodes at ×20, none elsewhere.

**Decision, by the frozen rule: `READY_FOR_IMPLEMENTATION`, the form
"bound the lines".** X_20 is 1.962%, over 0.1%, and U carries 88.3% of the
×20 cap nodes' plies beyond R, over 60%. `NO_CHANGE` does not apply: the cap
is reached in 22 of the 60 games at `60+0.6`'s node counts.

**The gate the rule names is corrected here, before any game.** The rule
gives case 1, because X_1 is 0. That clause came from PLAN, where it was
written for raising the cap, which changes only lines that reach it. The
form this read selects, bounding the depth a line gains outside the
extension budget, changes reductions and re-searches across the tree. In
AGENTS's terms it changes shared search signals in the rated configuration,
and such a change keeps case 3, the symmetric `[-5,5]` with its cap. Case 1
is reserved for a repair that reproduces the fingerprint, or moves it only
inside a domain a static differential confines. The registration erred in
copying the clause without its premise. The implementation registers its
gate by AGENTS's rules. Case 1 is open only to a form confined to lines that
reach the cap, with that differential as its correctness read.

**What the data say.**
- **The budget does not bound the lines.** At a cap node the iteration is at
  depth 15 and the budget has spent 13, yet the line holds about 100 plies
  more. This is structural, not a reading of E: the budget caps E at R, so U
  is at least 127 − 2R − Q, about 97 at R = 15.
- **Quiescence is not the driver.** Q is 0.0 to 0.1 on average, and no cap
  node is past quiescence's 16 plies.
- **What the cap returns is wrong where it matters.** At ×20, 22.2% of cap
  nodes are in check and 15.0% are draws by rule. Each returns a static eval.
- **STC cannot see it.** No search reaches the cap at the games' own node
  counts. A gate at `3+0.03` measures only what the bound does to the tree
  elsewhere, not the cap repair. The implementation's registration should
  state this exposure (zero at STC, 1.96% of searches at `60+0.6`'s node
  counts) and choose its time control knowingly.
- **U is not yet split by source.** By the code it comes from the late-move
  reduction's room above nominal (two plies, two more on a PV line), the
  re-search ply and hindsight's ply. Before choosing a bound, the
  implementation's instrumentation splits U among these, as AGENTS lets
  implementation own focused instrumentation. TS-1's per-line count of
  negative-reduction extensions is the prior form; it fits only if the
  reduction's room is the larger part.

## Calibration (appended after the read)

1. The wires pass: hit.
2. X_1 under 0.1%: hit (0). X_20 at 0.1% or more: hit (1.96%).
3. U at 60% or more in the race position: hit (93.5%).
4. The replay's mechanism is the race position's: hit (U 88% to 95%).
5. In check 10% to 40% of cap nodes: hit at ×20 (22.2%). At ×10/3 the four
   searches read 93.2%: few positions, a narrow sample.
6. `READY_FOR_IMPLEMENTATION` 0.35: hit.
7. The failure mode did not occur. The gate games reached the cap at ×20 in
   22 of 60 games.
8. Not predicted, and found: the gate clause copied from PLAN does not fit
   the form the read selects. The miss is in the registration's
   construction, not in a measurement.
