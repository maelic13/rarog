# Import: Basilisk's board findings for Rarog (2026-11-14)

Imported 2026-11-14 for PLAN D.12. Source: Basilisk commit `21ce21c` (branch
`b7-board-audit`, not merged there yet), file
`D:/code/basilisk/analysis/rarog_board_handoff_2026-11-14.md`; SHA-256 of the
imported bytes 84c01b15812cc8c006129d3b4b60fefbeec56d1add2f2f3975b6ce62d2771dbb. The text below is the source verbatim except
that its title is demoted one level; nothing was edited.

**Everything in it is a prior from another engine, not a verdict.** Its
figures are Basilisk's measurements on Basilisk's host and builds, several
from non-PGO screens whose pooled-PGO read was still pending there. Nothing
here is adopted in Rarog without Rarog's own registered measurement. Where
each finding stands against Rarog's records, and the leaves it led to, is
PLAN D.12 (D.12.1.1, D.12.1.2, D.12.2.1, D.12.2.2).

---

## Board findings for Rarog, from Basilisk's B.7 (2026-11-14)

Written at the maintainer's request for Rarog's own board step. It carries
what Basilisk measured on its board on 2026-11-14, what that says about
Rarog's board, and the method. Everything here is a prior from another
engine: Rarog registers and measures its own. Mechanisms are described,
no source is given.

Sources in Basilisk: `analysis/b72_board_review_2026-11-14.md` (the board
review), `analysis/b7_frontier_review_2026-11-14.md` (the method review and
the prototypes), `tools/board_compare/` (the cross-engine contract and
Rarog's adapter, a patch on Rarog `611307e`).

## 1. Where Rarog's board stands

Five boards were timed under one contract on 512 game positions, one
compiler, no PGO, five alternating rounds (ns per operation; Rarog built at
`611307e` with `target-cpu=native --cfg rarog_pext`):

| Column | Basilisk | Rarog | Stockfish 19 | Stockfish classical | Reckless |
|---|---:|---:|---:|---:|---:|
| legal generation, per position | 48.1 | 62.0 | 76.6 | 89.4 | 51.1 |
| capture generation, per position | 17.4 | 19.8 † | 10.2 ‡ | 13.4 ‡ | 16.3 † |
| make and unmake | 17.3 | **14.7** | 52.7 | 35.5 | 40.4 |
| SEE at threshold 0, tactical moves | 15.3 | 13.8 § | 6.6 § | 6.6 § | 18.2 |
| `gives_check` | 6.5 | 7.6 ¶ | 3.2 | 3.2 | — |
| perft, per leaf | 3.35 | 4.18 | 6.19 | 5.14 | 5.41 |

† 885 of the 927 tactical moves: the quiet stage holds the non-capturing
underpromotions. ‡ Pseudo-legal and unfiltered. § Promotions and en
passant pass unexamined (36 of the 927 moves). ¶ Rarog's public full form,
not the fast form its search uses.

Weighted by Basilisk's search mix per node (0.92 makes, 0.53 capture
generations, 0.20 quiet generations, 1.42 SEE, 1.44 `gives_check`):
Basilisk 65.8 ns, Rarog 67.0, classical Stockfish 71.7, Stockfish 19 83.1.

**Read honestly, Rarog's board is level with Basilisk's today, not
behind:**

- **The 2% is inside the instrument's error.** The composite uses full
  legal generation as a stand-in for the quiet stage. Without that term
  Rarog is ahead (54.6 against 56.2).
- **Rarog leads on make and unmake by 15%.** Its make is given
  `gives_check` in advance for every move. That is fair for Rarog's
  search, which hints 89% of its makes.
- **Rarog trails on generation:** 29% on legal generation and 25% on
  perft. Holding 512 boards does not explain it (Rarog's columns are the
  same on 32 boards).
- **After Basilisk's two accepted changes (§3) the gap opens:** its SEE
  column falls to 10.4 ns and its make and unmake to 16.0, about 47.6 ns
  per node against Rarog's 54.6 on the same terms.

## 2. Method: what the microbenchmark gets wrong

A per-primitive column is a good guard and a poor price.

- **It times game positions, not search nodes.** The side to move is in
  check at 6.8% of the file's positions, and at 12.6% of the search's
  capture generations and 21.2% of its quiet generations.
- **It times every legal move equally,** and SEE at threshold 0. The
  search's makes are 39% captures. Its SEE calls are 60.5% captures and
  37% quiets, 71% with a negative threshold, and 24% of them leave before
  any lookup.
- **One fixed order, repeated,** which the branch predictor learns.
- **It underprices the board.** Basilisk's composite reads 65.8 ns per
  node. In real search order the same board costs 80 to 131 ns (two
  bounds), and the engine's profile says 103. Generation costs half as
  much again as on game positions.

**Two better instruments, both cheap to build:**

1. **A search replay.** A trace build records every board operation the
   search makes on its own board, in order and with its result: make,
   unmake, null moves, each generation stage, SEE with its threshold,
   `gives_check`, the table-move test, the draw tests. A replayer runs
   the trace on one board and nothing else.
   - A verify mode compares every recorded result (the key after each
     make, each SEE and `gives_check` answer, each count). It is an exact
     regression test for a board rewrite: 58.6 million results for one
     `bench 13`.
   - The replayer's own dispatch is an unpredictable switch. Pair each
     timed run with a stub run (the same records and switch, empty
     out-of-line calls). Filter the trace to the classes under test before
     timing; skipped records otherwise dominate.
   - An A/B of two boards on one trace needs no correction.
   - It is conservative: both changes that gained in the engine gained
     less on it (4.3 against 6.9 ns per node, 2.1 against 8.2), because
     nothing else competes for the caches.
2. **An engine screen.** Alternating `bench` runs of plain release builds
   on one pinned CPU, 16 rounds, the order reversed on odd rounds, the
   per-round time ratio. It resolved about ±0.3% in five minutes and
   agreed in direction with the replay on all five kernels read both ways.

Pooled-PGO NPS still decides what ships.

## 3. What gained and what did not

Nine candidate kernels, each exact on the whole trace and on the bench
fingerprint:

| Kernel | Replay, ns per node | Engine NPS, 16 rounds | |
|---|---:|---:|---|
| Unmake without the key work it discards | −2.1 | **+1.86%** | kept |
| SEE pins read from aligned sliders (below) | −4.3 | **+1.56%** | kept |
| SEE: skip the pin computation when no slider is aligned with a king | −0.1 | −0.16% | no gain |
| `gives_check`: a bit test before the discovered-check lookups | +1.5 | −0.32% | no gain |
| `gives_check` without any slider lookup | +0.9 | −0.21% | no gain |
| make derives the checkers from the move | +1.0 | **−0.74%** | loss |
| make told `gives_check` by the search (the hint) | +0.2 | not built | no gain |
| SEE attackers from aligned sliders, no lookup | +2.2 | not built | loss |
| generator pins from aligned sliders (Rarog's RAR-M31) | −0.5 | not built | within noise |

**The pattern.** On a Zen 3 a PEXT lookup is three cycles and a cached
load, and it does not branch. Every kernel that only removed or skipped
lookups gained nothing or lost; a pre-test is a branch. The two that
gained removed a whole code path.

**For Rarog, specifically:**

- **RAR-M31 is confirmed as NO_CHANGE.** The same pin form read −0.5 ns
  per node of a 26 ns generation cost in real search order.
- **The hint is not where make's time is.** Basilisk's search knows
  `gives_check` for only 43% of its makes, and the hinted form read no
  gain there. Rarog's lead on the column is the hint; it does not transfer
  as a technique, and it is not a reason for Rarog to change anything.
- **The eager, Stockfish-shaped check information is a wash,** not a
  loss and not a gain: against classical Stockfish, cheaper queries save
  17.0 ns per node and the dearer make costs 16.8.

## 4. The two changes worth reading in Rarog

**SEE pins from aligned sliders.** Rarog's own profile put the cost of SEE
in the per-candidate king-legality test (`see_recapturer` 4.35% exclusive,
against 0.87% for the exchange loop; RAR-M36). Basilisk's equivalent cost
was a pin computation up front, and this form removed it:

- For each colour, the snipers are the enemy rooks and queens on an
  empty-board rook line of that colour's king, and the enemy bishops and
  queens on a bishop line. Two 64-entry tables; no slider lookup.
- When an attacker of that colour is chosen, a piece is excluded if it
  alone stands between its king and a sniper that is still in the exchange
  occupancy (one between-squares mask and a single-bit test), and the
  target square is not on that line.
- It is read on the occupancy of that moment. So a pin opened by an
  earlier capture is seen, a pinner that has been exchanged off no longer
  pins, and nothing is recomputed or stored.
- At 61% of the calls that reach it, neither king has a sniper and nothing
  more runs. A pinned piece is among the attackers in 0.56% of SEE calls.
- It gave the same answer as the legal exchange tree on Basilisk's SEE
  corpus and on 6.05 million search calls. Rarog's rule (the selected
  king's legality) is not the same contract, so Rarog checks it against
  its own fixtures first.

**Unmake that restores instead of recomputing.** Basilisk's unmake ran
the same helpers as make, which update every key, and then overwrote the
keys from its undo record. A board-only form of the helpers in unmake read
+1.86%. Rarog's undo record is 24 bytes, so its unmake presumably
recomputes the keys. The question for Rarog is the trade: 40 more bytes
saved per ply against the key updates of every unmake. In the engine the
discarded work cost four times what the replay showed. The likely reason,
not measured, is that the Zobrist tables are cold between uses there.

## 5. Two counts worth taking on Rarog's own trace

Both are the search's and the picker's, not the board's:

- **20.6% of SEE calls ask about a move already asked at that node** with
  another threshold, and an earlier answer already settles 10.2% of all
  calls, because threshold SEE is monotone: a yes at `t` answers every
  threshold at or below `t`, a no every threshold at or above.
- **8.7% of capture generations repeat at the same node** (a ProbCut
  picker, then the main picker).

## 6. What Basilisk refused, with the evidence

| Idea | Evidence |
|---|---|
| Check and pin information stored at make | a wash on the five-engine table; Basilisk's BAS-P04 −1.8% to −2.7% |
| Compressed slider tables | BAS-P26 −0.98% |
| Attack tables kept incrementally | Stockfish 19's make costs 17 ns more than classical's for threat records alone |
| A byte mailbox read by vector permutes | needs AVX-512 VBMI; no measured comparison with PEXT exists |
| Pseudo-legal generation with a late legality test | under 0.5% by the columns, and it reorders the unsorted tail |
| Side or stage as a template parameter | Rarog's own RAR-M44(c), −0.55% |

Still open in Basilisk (its B.7.7): undo as one block, a one-byte mailbox,
keys by table in make, next attacker by index in SEE, one enemy attack map
per node.

## 7. For a network

Make writes nothing for the network. The delta of a ply is the triple the
search already holds at every make: the move, the piece that moves and the
piece captured. It is complete (at most four piece changes) and
reversible, so a lazy accumulator walks it back to the last computed ply
and refreshes at the current ply. Stockfish 19's eager records are the
reason its make is the dearest of the five.
