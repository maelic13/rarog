# RAR-S111 — D.6.3 and D.6.4's coupling, each bench position in its own process

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-10 before any fresh-process count was
  read, except the one named under *Exposure*. The read is the agent's
  (deterministic, no games).
- **Kind:** a zero-game read. It is the fresh-process per-position read that
  [RAR-S110](RAR-S110.md)'s C4 rule puts before any characterisation of the
  coupling.
- **Baseline:** `d6` at `b17bb30`, engine source `ee0b3e4`, bench 13,038,236 /
  EBF 2.540.
- **Inherits** RAR-S110, which is history and is not reopened:
  - The cluster arm is K = 6: both ProbCut verification children typed `false`
    (bit 2), and an `Upper` store passing no move (bit 4).
  - The null half is out of the cluster, closed `NO_CHANGE` for it.
  - On the bench C1, C2 and C3 were each supported, and C4 held: three
    positions carried 56.5% of the change in interior nodes from arm 0 to arm 7.
  - Arm 6 solved 242 of WAC at 100k against a floor of 233. Under the canary
    rule in force (A) it loses WAC.069 by a later settle. Under the proposed
    rule B it loses none.
- **Research question:** is the coupling between ProbCut's child type and the
  fail-low TT move a property of positions in general, or of a few bench
  positions and the state they share?

## The four questions (AGENTS)

- **Mechanism.** As RAR-S110 states it, narrowed to the cluster arm. Nothing new
  is claimed here. This read asks only where the coupling lives.
- **Interactions.** As RAR-S110. One is added: the bench clears the table, the
  histories and the correction history once per run, not per position
  (`run_bench` in `src/engine.rs`), so each position after the first is searched
  on what the earlier ones left.
- **Invariants:**
  - Arm 0 is the source as it is, each bit changes only what it names, and the
    counters never steer.
  - A process searches one position from an empty state, so its counts depend
    on the position and the arm and on nothing else.
  - The bench searches its first position from the same empty state. A fresh
    process on position 1 must therefore reproduce the bench's first dump in
    the same arm.
- **Falsifier.** The coupling is not general if the typical position does not
  show it. The verdict below says when. The two items then lose their reason
  to be gated as one unit.

## Explanations for the bench concentration

- **E1, a real mechanism in a few position types.** The three carriers keep
  their change in fresh processes, it builds over the iterations, and the
  coupling orders with a property of the position. **Separating read:** the
  rank correlation of ln κ with the piece count, with arm 0's share of the tree
  under parents whose ProbCut did not cut, and with arm 0's share of nodes
  whose TT move came from an `Upper` entry.
- **E2, shared state.** The table and histories of earlier positions made the
  carriers' trees collapse in arm 7. **Separating read:** in fresh processes
  the carriers lose their change.
- **E3, a deep-iteration runaway.** In positions 11, 10 and 22 one arm's last
  iterations run away, and the ratio is made there. **Separating read:** the
  cumulative nodes at each completed depth. A runaway makes more than half of
  the final ln ratio after depth 11. A mechanism that compounds per ply makes
  about two thirteenths of it there.
- **E4, one arm of the index.** On the bench every arm that adds a bit to bit 4
  searched 0.70 to 0.79 of arm 0's nodes (arms 5, 6, 7, 12 and 15), and arm 4
  alone searched 0.98. A pooled index below 1 then needs no coupling in any
  position: it is enough that arm 4's total is held up by one or two positions.
  **Separating read:** the per-position ratio of arm 4 to arm 0 against the
  pooled one, and a per-position κ that centres on 1.

These are not exclusive. The reads that separate them are reported; only the
verdict below decides.

## The read (frozen)

**Binary.** RAR-S110's screen binary, rebuilt:
`analysis/arm_patches/d63-d64-rar-s110-screen.patch` (SHA-256 `05c16f69…`) on
`ee0b3e4`, `--features diag` only, in a worktree at the path RAR-S110 built in.
Before any read the script asserts two things:
- the executable's SHA-256 is RAR-S110's, `6ee8aab9…`;
- arm 0's `bench 13` is 13,038,236, with 40 dumps.

If the rebuild's hash differs, the read uses RAR-S110's archived executable,
whose hash that is, and the difference is recorded.

**Process.** One process per position and arm: `uci`, `ucinewgame`, `isready`,
`position fen <the FEN from src/bench.rs>`, `go depth 13`, one thread, stride 1,
`RAROG_S109_ARM` set to the arm. The 40 FENs and the depth are read from
`src/bench.rs`, and the script asserts 40 and 13.

**Counts.** A search dumps its counters once, so the script asserts that every
counter in a process was dumped exactly once, and that each arm has 40
processes. A position's count is that process's `nodes`, the interior nodes,
which is the unit of RAR-S110's C4. An arm's pooled count is the sum over its
40 processes. The engine's own node count at each completed depth is kept from
the `info depth` lines, for E3 only. No ratio mixes the two units.

**Arms, and what each separates:**
- **0, 4 and 6:** the source, D.6.4 alone, and the cluster arm.
- **2**, ProbCut typed `false` alone. Without it the cluster arm's shrink cannot
  be told from the coupling: κ needs both single arms.
- **7**, all three bits. RAR-S110's C4 was stated on arm 0 to arm 7, so E2 is
  tested on the same pair of arms that showed the concentration.
- **8, 10, 12 and 14:** arms 0, 2, 4 and 6 with `lmr_cut_node_no_tt_move`
  zeroed. Together they give κ₈ for K = 6. RAR-S110 read C1 on arm 7's index
  only, and arms 10 and 14 have never been run. Arm 14 against arm 6 is the
  term's weight inside the cluster arm, arm 12 against arm 4 inside D.6.4
  alone, and arms 10 and 8 are their references.

Arms 1, 3, 5, 11 and 15 are not read: the null half is out.

**Indices,** on interior nodes N, for position p and for the pooled sums S:
- r_a = N_a / N_0, an arm's ratio to arm 0.
- κ = (N_6 · N_0) / (N_2 · N_4), the coupling of bit 2 with bit 4.
- κ₈ = (N_14 · N_8) / (N_10 · N_12), the same with the term zeroed.
- κ_tt, the pooled index on the total TT cutoffs of arms 0, 2, 4 and 6.
- κ_Σ is κ on the pooled sums. κ₋₃ is κ_Σ after removing three positions, one
  at a time, each time the one whose removal raises the index most.
- A position is **active** if its κ is not exactly 1.
- **z** is the Wilcoxon signed-rank score over the active positions. Rank them
  by |ln κ|, tied values sharing the mean rank. W is the sum of the ranks of
  the positions with κ below 1, and z = (W − m(m+1)/4) / √(m(m+1)(2m+1)/24) for
  m active positions. With 40 positions, three carriers lift z by 0.79 over an
  even split of the other 37, so three positions cannot pass the test by
  themselves.

**Wire.**
- Position 1 in arms 0, 2, 4, 6, 7, 8 and 12 equals the bench's first dump of
  `nodes` in that arm, from RAR-S110's `read.json`.
- Arms 10 and 14 have no bench read. Arm 10's pooled count differs from arm
  2's, and arm 14's from arm 6's.
- Arms 2 and 4 differ from arm 0, and arm 6 from arm 4.

The pooled fresh counts are not expected to equal the bench's totals, and that
is not a wire.

**Script.** `analysis/artifacts/d63-d64-fresh-2026-11-10/read_fresh.py`,
SHA-256 `5ecf7c24…`. Its statistics pass a self-test on synthetic numbers, and
its process loop reproduced the one value under *Exposure*. It writes
`fresh.json` and prints the analysis. The counts are deterministic, so the
host's load does not change them.

## Exposure, stated

Seen:
- the bench totals and counters of RAR-S110's twelve arms, as its record prints
  them;
- from the bench's per-position counts, only what RAR-S110's record prints:
  positions 11, 10 and 22 in arms 0 and 7, and position 4;
- one fresh-process count, arm 0 on position 1 (236,119), read to prove the
  script. It is the wire's own value: the bench's first dump.

Unexposed: every other fresh-process count, in every arm. Arms 10 and 14 have
never been run in any form. RAR-S110's `read.json` holds the bench's
per-position counts for its arms. This registration was written without
reading them, and the script uses them only for the wire and for the three
carriers' bench ratios.

Each threshold is set without a seen value:
- **z of 1.96** is the one-sided 2.5% point. Without any coupling the test
  passes one time in forty.
- **30 active positions** is three quarters of the suite.
- **The half rules** are RAR-S110's, chosen there without data.
- **Three positions** is RAR-S110's C4 count.

## C4 restated, and the verdict (frozen)

On the bench, concentration could mean two things: a few positions that couple,
or a few positions whose shared state collapsed. Without shared state only the
first is left. So concentration here means that the pooled coupling rests on a
few positions while the typical position shows none. Absolute node changes
cannot show that on their own, because a large tree carries a large change. So
each position gets one vote, by rank, and the pooled index must survive losing
its three strongest carriers.

**Void, an instrument fault,** if the binary check or any wire condition fails,
or any process fails its dump count. Nothing is then read.

**GENERAL** if all of these hold:
- at least 30 positions are active;
- z is at least 1.96;
- κ_Σ is below 1;
- 1 − κ₋₃ is at least half of 1 − κ_Σ.

**NOT GENERAL** if at least 30 positions are active, z is below 1.96, and either
κ_Σ is 1 or more (no pooled coupling without shared state) or κ₋₃ is 1 or more
(three positions carry all of it).

**INCONCLUSIVE** otherwise.

**Mechanism for the cluster arm,** read only if the coupling is general, on the
pooled sums:
- **C1′:** |κ₈ − 1| is at most half of |κ_Σ − 1|.
- **C3′:** κ_tt − 1 has κ_Σ − 1's sign and at least half its magnitude.

C2 was about null decisions and has no arm here. The null-move cut rate per arm
is reported.

**The explanations,** reported and not deciding:
- E1 is favoured if at least two carriers keep half or more of their bench
  ln ratio (arm 0 to arm 7), with half or less of it made after depth 11, and
  one of the three rank correlations is 0.40 or more in size (the two-sided 1%
  point for 40 positions).
- E2 is favoured if at most one carrier keeps half.
- E3 is favoured if at least two carriers keep half and make more than half of
  it after depth 11.
- E4 is favoured if the verdict is NOT GENERAL and the median r₄ is below the
  pooled r₄.

RAR-S110's C4 statistic, the three largest contributions over the total change
in interior nodes, is reported for arm 0 to arms 7, 6 and 4 on the fresh counts.
It does not decide: it is the statistic the restatement replaces.

## How the result feeds readiness for K = 6 (frozen)

The cluster stays in `RESEARCH` on every branch of this read. It can make
nothing `READY_FOR_IMPLEMENTATION` by itself.

- **GENERAL, with C1′ or C3′ supported.** RAR-S110's C4 and mechanism
  conditions are met on the instrument it asked for, and its WAC condition was
  met there. One condition is left: the canary rule in force holds arm 6
  (WAC.069, a later settle). Two things are then missing before a gate:
  1. The maintainer's decision on the canary rule. Under A the cluster stays
     held. Under B it is `READY_FOR_IMPLEMENTATION`. C needs a read of nodes
     per iteration first.
  2. The gate's registration, before any game: arm 6's change and nothing
     else, with the unit test that pins each recursive call's `cut_node`
     (NODE-14); case 3, `[-5,5]`, unless that registration predicts a gain and
     argues `[0,3]`; and its measured exposure.
- **GENERAL, with neither supported.** The coupling is general and unexplained.
  The next registration tests the channels this read does not: the null
  decisions inside the cluster arm (the toggle from arm 4 to arm 6 moved the
  null-move cut rate by 5.36 points on the bench), the first move searched,
  and the history rewards.
- **NOT GENERAL.** The cluster's reason to be one unit falls. The next
  registration tests the two items as separable repairs, on fresh-process
  reads: each item's own effect per position, each one's canary outcome under
  the rule in force, and, for whichever of E1 to E4 this read favours, the
  positions it names.
- **INCONCLUSIVE.** One wider read is allowed: this rule, unchanged, on a larger
  position set frozen in its own registration before exposure. It is the last
  zero-game read of the coupling. If it is inconclusive too, the question is
  recorded as not decidable at screen cost, and the choice goes to the
  maintainer: gate arm 6 as a case-3 repair with the coupling uncharacterised,
  or close the cluster `NO_CHANGE` with its retry triggers.
- **Void.** The instrument is repaired and the read is registered again. No
  count from the void run is used.

## PRE-REGISTERED PREDICTION (frozen before exposure)

1. The wire passes: position 1 reproduces the bench's first dump in all seven
   arms. 0.9.
2. Arm 0's pooled fresh count is within 10% of the bench's 8,416,796 interior
   nodes. 0.6.
3. Pooled r₆ is 0.70 to 0.92. 0.6. Pooled r₄ is 0.85 to 1.02. 0.6. Pooled r₂ is
   0.95 to 1.06. 0.6.
4. Arm 10's pooled ratio to arm 0 is 0.88 to 1.05, and arm 14's is 0.70 to
   0.90. 0.5 each.
5. κ_Σ is below 1. 0.7. It is 0.72 to 0.95. 0.5.
6. The median κ is 0.92 to 1.04. 0.5. Of the active positions, 45% to 65% have
   κ below 1. 0.5.
7. z is at least 1.96. 0.3.
8. 1 − κ₋₃ is at least half of 1 − κ_Σ. 0.35. κ₋₃ is 1 or more. 0.3.
9. The verdict: GENERAL 0.25, NOT GENERAL 0.35, INCONCLUSIVE 0.40.
10. At least two of positions 11, 10 and 22 keep half of their bench change.
    0.45.
11. The three largest contributions carry half or more of the fresh change
    from arm 0 to arm 7. 0.5.
12. The median r₄ is below the pooled r₄. 0.5.
13. If the coupling is general: C1′ supported 0.45, C3′ supported 0.45.
14. Of the positions that move by 25% or more from arm 0 to arm 6, fewer than
    half make more than half of it after depth 11. 0.6.
15. **Most likely failure mode:** 40 positions are too few for a double ratio
    of node counts at depth 13. The pooled index stays below 1, z lands
    between 0 and 1.96, and the read is inconclusive.
