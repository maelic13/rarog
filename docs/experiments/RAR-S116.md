# RAR-S116 — D.6.4 after its gate: which consumer of the stored move carries the depth, and whether the kept move orders better, on replayed gate games

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-11 before any read it names as
  unexposed. The read is the agent's (deterministic, no games).
- **Kind:** research and diagnosis after a rejected repair. No game, no gate.
- Disposition: open — registered, the deciding arms not run.
- **Baseline:** `d6` at `2ad9593`, engine behaviour of `ee0b3e4`, bench
  13,038,236 / EBF 2.540.
- **Inherits:**
  - [RAR-S113](RAR-S113.md): the readiness read that called the store change
    ready.
  - [RAR-S115](RAR-S115.md): its gate. H0 at 2,452 games, nElo −14.7 ± 13.8,
    with mean depth 14.01 against 13.12 on 3% more nodes a move.
  - PLAN rule 6: two rejected clusters in one programme force a new evidence
    audit before a third is built. D.6.3 (RAR-S114) and D.6.4 are the two;
    this is D.6.4's part of that audit.

## The question

At a fail low the source stores the node's best-scoring move, and the rejected
repair stored none, so the entry kept the move it held. That one change gives
five consumers a different input at the next visit: the move searched first,
IIR, the razoring guard, the cut-node LMR term for a missing move, and the
fail-low parent-reward factor. The gate rejected them together. Two questions
follow.

1. **Which consumer makes the tree smaller?** The candidate completed 0.89
   more of depth on the same clock, so its tree at a given depth is far
   smaller. Something reduced it.
2. **Is anything left to gate?** The repair's own premise is that the entry's
   older move orders better than the fail low's best-scoring move. With the
   consumers held at one rule, does it?

## Competing explanations, and what each predicts here

- **(a) The cut-node LMR term over-reduces the new population.** Holding that
  term alone under the store change (arm 36) returns most of the tree.
- **(b) IIR fires where it was not fitted.** Holding IIR alone (arm 20)
  returns most of the tree.
- **(c) The best-scoring move was useful ordering at the next visit.** Under
  held consumers the store change (arm 116 against arm 112) searches more
  nodes to the same depth and cuts on the first move less often; taking the
  move from the picker (arm 128 against arm 0) enlarges the tree.
- **(d) The razoring guard or the reward factor.** Holding the guard alone
  (arm 68) returns the tree. The reward factor reads whether the parent's move
  was its table move and has no arm of its own.
- **(e) The depth is bought by pruning what matters.** This read cannot tell
  it from (a) and (b): they are how it would happen. The games' own data is
  read below.
- **(f) An instrument or protocol effect.** Read below.

## Read before this registration

All of it was read after RAR-S115's result, and none of it was registered. It
chose this read's arms and decides nothing.

**The donors, for mechanism only** (Reckless at `aec5cb2` of 2026-10-29;
Stockfish 19 and the development tree of 2026-11-06):
- Both set the best move only when alpha rises, and their tables keep the old
  move when a store brings none. A fail-low node without a move is their
  normal state.
- Neither reduces an all node for it. Stockfish's IIR runs only at PV and cut
  nodes, at depth 6 or more. Reckless at this commit has no IIR.
- Both add a cut-node LMR term for a missing move, as Rarog does.
- Razoring: Stockfish 19 has no table-move guard; Reckless excludes a quiet
  table move and a lower bound, as Rarog does.

**Rarog's consumers** (`src/search/node.rs`): IIR fires at any node of depth 4
or more without a table move, with no node-type condition. The LMR term for a
missing move applies only at a cut node off the PV line.

**RAR-S113's saved counters,** the rejected store (arm 4) against the source
(arm 0), game-like searches, with the fresh ones in brackets
(`analysis/artifacts/d64-research-2026-11-11/consumers_existing.log`):

| | source | store change |
|---|---:|---:|
| IIR applied, of interior nodes | 11.71% (14.33%) | 15.10% (16.83%) |
| LMR at a cut node without a move, of LMR computations | 6.79% (10.76%) | 10.70% (14.48%) |
| razoring drops, of interior nodes | 0.59% (1.06%) | 0.61% (1.28%) |
| cutoffs on the first move | 89.15% (89.00%) | 87.97% (89.21%) |
| mean reduction of a reduced move, plies | 1.215 (1.548) | 1.059 (1.584) |

RAR-S113 reported LMR without a move at 20% and 57%. That counted every LMR
computation, and the term applies only at cut nodes off the PV line, where the
share moves from 7% to 11%.

**Depth inside a node count,** from RAR-S113's searches (`depth_at_nodes.log`
beside it): inside 157,000 nodes the store change completes +0.75 of depth
fresh and +0.32 game-like, the latter censored at depth 13 in 61 of 117
searches. Fresh, it reaches depths 8 to 12 in 0.60 to 0.62 of the source's
nodes. The games' +0.89 is nearer the fresh state.

**The gate's games,** from each move's score and depth in `games.pgn`, with
RAR-S114's games as the control, where the arms' depths were equal
(`games_s115.log`, `games_s114.log` beside it):
- **The depth gain is everywhere:** +0.61 in a side's first 15 moves, +1.15
  in moves 51 to 80, and 18.40 against 16.73 where the score is within 50.
- **No more large swings:** a side's own score falls by 150 or more between
  its moves 5.01 times per 1,000 for the candidate and 4.95 for the baseline
  (control: 4.80 and 4.96). By 75 or more: 24.02 against 22.87 (control:
  23.51 against 23.61).
- **It sees its losses later:** in games the baseline won, the winner's score
  settled beyond 200 before the loser's in 72.9% (462 of 634). In games the
  candidate won, in 66.0% (379 of 574). The control reads 66.4% and 66.1%.
- **Conversion is level:** 91.0% against 90.1% of games won after +300.
- **(f) has no support:** no time losses, equal time a move, and the
  baseline's figures match RAR-S114's baseline.

Descriptive, several statistics tried, no strength evidence. It leans against
plain tactical blindness and towards a search that is nominally deeper and
slower to see where it stands worse.

## Arms

`analysis/arm_patches/d64-rar-s116-screen.patch` on `ee0b3e4`: RAR-S110's
screen patch plus these bits of `RAROG_S109_ARM` and seven counters at nodes
entered on a fail-low entry. A **fail-low entry** is an `Upper` bound of depth
1 or more, which only the main search's fail low stores.

| bit | effect |
|---:|---|
| 4 | an `Upper` store passes no move (the rejected repair, as before) |
| 16 | IIR reads a fail-low entry as carrying a move, whatever it holds |
| 32 | the cut-node LMR term does |
| 64 | the razoring guard does, as a quiet move when the entry holds none |
| 128 | a fail-low entry's move is kept from the picker; the consumers still read it |

| arm | store | consumers | it answers |
|---:|---|---|---|
| 0 | source | as they are | the source |
| 4 | no move | as they are | the rejected candidate |
| 112 | source | all three held | the baseline for arm 116 |
| 116 | no move | all three held | **the ordering change alone** |
| 20 | no move | IIR held | IIR's share |
| 36 | no move | LMR term held | the LMR term's share |
| 68 | no move | razoring held | the razoring guard's share |
| 128 | source, move kept from the picker | as they are | what the best-scoring move is worth as ordering |

Arms 112 and 116 give the three consumers the same input at every fail-low
entry, so they differ only in what the entry's move is: the fail low's
best-scoring move, or the older move or none.

**Build proof,** `bench 13` on the archived executable
(`analysis/artifacts/d64-consumers-2026-11-11/rarog-s116screen.exe`, SHA-256
`ce249878…`, built with `--features diag` only; the patch applied to a clean
`ee0b3e4` reproduces the build tree's three files by hash):
- arm 0: 13,038,236 / EBF 2.540, the fingerprint; arm 4: 12,785,308, the
  rejected candidate's.
- arms 32 and 64 alone: 13,038,236. On the source store those holds change
  nothing.
- arm 16 alone: 12,591,552; arm 112: 12,783,101. The IIR hold is not exact.
  A node whose moves were all pruned stores an upper bound without a move in
  the source too, and the source's IIR fires on it. That is why arm 116 is
  read against arm 112 and not against arm 0.

## Instrument: a replay of gate games

RAR-S113's game-like lines followed the engine's own principal variation and
under-read the games' depth gain (RAR-S115's calibration). This read replays
real games instead. `replay.py` beside the executable (SHA-256 `1a02dff6…`)
takes the first 60 official games of RAR-S115's gate (`games.pgn`, hash
asserted). For each, one process with Hash 64 and one thread sends
`ucinewgame` and then, for every move the baseline played, the position by
its game moves and `go nodes n`, with `n` the node count the game recorded
for that move. The moves followed are the game's, so every arm searches the
same positions on the table and histories its own earlier searches left.
Counts are deterministic.

The game records the last `info` line's nodes, and the engine searched on
until its clock stopped it, so a replay's table is a little poorer than the
game's was. It is the same for every arm.

**Wire** (`replay.py wire`, the 60 games):
- arm 0 on the baseline's moves reproduces the games' recorded mean depth
  within 0.10;
- arm 4 on the candidate's moves reproduces the candidate's within 0.20;
- every search dumps each counter exactly once, and the arms search the same
  moves.

## Exposure, stated

Seen: everything under *Read before this registration*; the build proof's
six bench counts; and a smoke of the replay on six games, arms 0 and 4 only.
There arm 0 read 12.758 against the games' 12.756 and arm 4 read 13.603
against 13.716, with the same depth as the game in 35% and 33% of moves and
the same move in 70% and 64%.

Unexposed: every replay count of arms 112, 116, 20, 36, 68 and 128, and the
60-game replay of arms 0 and 4.

## Rules (frozen)

**Void** if the wire fails.

**The deciding quantity** is *r*: over the baseline's moves without a mate
score, the median of arm 116's nodes over arm 112's at the deepest depth both
completed, searches whose deepest shared depth is under 6 left out.

- **`READY_FOR_IMPLEMENTATION`** if *r* ≤ 0.98 and arm 116 cuts on the first
  move at fail-low entries no less often than arm 112. The rework is then the
  store change with IIR, the cut-node LMR term and the razoring guard reading
  a fail-low entry as carrying a move: arm 116. Its gate is registered with
  its implementation.
- **`NO_CHANGE`** if *r* ≥ 0.99. The store change then has no ordering gain
  to gate. What it otherwise does is move IIR's and the LMR term's
  populations, which RAR-S115 rejected. Retry: the next search tune (F.6),
  or a change to IIR's node-type contract.
- **`RESEARCH`** otherwise.

**Reported, not deciding:** each consumer's share (arms 20, 36 and 68
against arm 4, and arm 116 against arm 4), arm 128 against arm 0, each arm's
depth inside the games' node counts, the counters at fail-low entries, and a
95% interval for each median from resampling whole games.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The wire passes. 0.85.
2. Arm 4 against arm 0: median node ratio 0.50 to 0.75 (0.7); depth +0.6 to
   +1.0 (0.75).
3. Shares, as the median ratio against arm 4:
   - IIR held, arm 20: 1.25 or more. 0.6.
   - the LMR term held, arm 36: 0.95 to 1.15. 0.6.
   - razoring held, arm 68: 0.97 to 1.03. 0.8.
   - IIR's share is larger than the LMR term's. 0.7.
4. All three held, arm 116 against arm 4: 1.4 or more. 0.6.
5. *r*: 0.98 or less, 0.15; between 0.98 and 0.99, 0.10; 0.99 to 1.06, 0.55;
   above 1.06, 0.20. So `NO_CHANGE` 0.75, `READY_FOR_IMPLEMENTATION` 0.12,
   `RESEARCH` 0.13.
6. Arm 116 cuts on the first move at fail-low entries less often than arm
   112. 0.65.
7. Arm 128 against arm 0: median 1.02 or more. 0.6.
8. **Most likely failure mode:** *r* lands within its interval of a
   threshold, so the rule's word depends on noise; or the hold is too coarse
   a rule to carry the ordering question, since arm 112 itself is not the
   source.
