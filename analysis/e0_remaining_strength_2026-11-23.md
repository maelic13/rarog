# E.0 — Where the remaining strength is: which part of the gap to the strongest hand-crafted engines Rarog can take, where, at what cost and in which order

Research packet for PLAN E.0, 2026-11-23. Zero games played; no engine
source changed. One game read is registered and waits for the maintainer
(RAR-M92). Evidence layers are named where they are used: nodes a second,
nanoseconds a node, profile shares, static accuracy and decided-game
counts are not Elo; an Elo figure is a played match or says "inferred".

## Decision needed

The order of what follows D.6's review: the speed work (D.16, D.12), the
evaluation programme (E.4), the open Phase D steps and the search leads.
The maintainer orders the steps; nothing here is built.

## Known evidence

The gap (RAR-M91, RAR-M90): the kernel is −103.3 [−121.0, −86.0] and
−100.6 [−119.4, −82.3] head to head against Stockfish 16 HCE, −46.3 against
Stockfish 11 and −42.8 against Berserk 4.7.0 HCE, at `3+0.03`, one thread.

### 1. The cost of a node — measured here

*Use of these reads: is the rate gap real, where in the game, where in
the node, and who owns it.* Scripts and outputs:
`analysis/artifacts/e0-2026-11-23/` (ignored). The host was idle before
and after the timed read (0.3% to 1.8% CPU, no engine, harness or build
process; 0% to 2.7% after).

- **The rate gap is real and is the node's own cost.** On 281 positions
  from RAR-M91's games, Hash 128, `go movetime 70`, the table carried,
  nodes over time at each engine's last report (`speed_load.py`):

  | Engine | Alone | 14 at once | Under load keeps | Over the kernel, alone / loaded |
  |---|---|---|---|---|
  | Rarog's kernel (`94d7f72`) | 2.655M, 2.667M | 2.349M | 0.883 | |
  | Stockfish 16 HCE | 3.513M, 3.497M | 3.106M | 0.886 | 1.32 / 1.32 |
  | Stockfish 11 | 3.608M, 3.582M | 3.123M | 0.869 | 1.35 / 1.33 |
  | Berserk 4.7.0 HCE | 4.745M, 4.679M | 3.939M | 0.836 | 1.77 / 1.68 |

  The ratio does not move with load, and the kernel loses no more under
  load than Stockfish does: the gap is not memory traffic between
  searches. At `go nodes 150000` the ratio to Stockfish 16 HCE is 1.36
  (2.316M against 3.144M).
- **In games** (RAR-M91's 52,800 games parsed a move at a time,
  `m91_phase.py`; nodes over time charged, pooled): the kernel 1.438M,
  Stockfish 16 HCE 2.127M (×1.48), Stockfish 11 2.179M (×1.52), Berserk
  2.233M (×1.55), Ethereal 14.40 HCE 2.703M (×1.88), Rarog 2.7.0 1.439M,
  Basilisk 1.11.0 1.520M. By non-pawn material the ratio to Stockfish 16
  HCE is 1.45 (45 or more), 1.47 (25 to 44), 1.46 (10 to 24) and 1.66
  (under 10): the same through the game, wider in the last 7% of its
  thinking time. In RAR-M90's games on the CLI the reported ratio is 1.42
  (1.916M against 2.722M).
- **What the runner's figure hides** (`m90_hold.py`, RAR-M90's 547,660
  kernel moves): the kernel's last report precedes its move by 2.66 ms of
  65.9, 4.0% of its time, over 20 ms on 3.3% of moves; Stockfish's
  reports end at the move. Without that time the kernel reads 1.996M and
  the ratio 1.35. That 4.0% is also work whose result the root discards
  (lever 6).
- **Where in the node** (`sf_modules.py`, `sf_modules.out`). Stockfish 16
  at `68e1e9b` built with debug information (g++ 16.2.0, `-O3 -flto`, no
  PGO); it searches the rated binary's tree (`bench 16 1 13 default depth
  classical` 3,684,832 on both; the 281 positions at 150,000 nodes
  40,846,986 on both). 24,714 samples on those positions, each charged to
  its nearest named region as `summarize_board_search_etw.py` charges
  Rarog's. Rarog's shares are D.7.8.12's game profile (134,025 samples,
  2026-11-20, before that step's own +2.71%), **not re-measured**; the
  rates are today's on the 281 positions at 150,000 nodes (the kernel
  432 ns a node, Stockfish 16 HCE 315):

  | Block | Kernel, ns a node | Stockfish 16 HCE, ns a node | Difference |
  |---|---|---|---|
  | Search side: the node, move ordering, histories | 186 (43.1%) | 108 (34.4%) | **+78** |
  | Evaluation, pawn and material tables included | 95 (22.1%) | 77 (24.3%) | +18 |
  | Transposition table | 56 (12.9%) | 37 (11.8%) | +19, before D.7.8.12's prefetch |
  | Board: generation, make and unmake, check queries, exchange evaluation | 94 (21.8%) | 89 (28.1%) | +5 |
  | Other | 1 | 4 | −3 |
  | A node | 432 | 315 | +117 |

  Two thirds of the gap is search-side work; the evaluation is a sixth;
  the board is level. Module boundaries differ between the engines (the
  kernel scores quiet moves inside its node, Stockfish in its picker), so
  the search-side block is read whole.
- **The price, inferred:** 1.35 is 0.433 of a doubling; at the kernel's
  own slope of about 145 Elo a doubling at `3+0.03` (three speed passes at
  an identical tree) that is +63, and +58 to +73 over the ratio's range
  of 1.32 to 1.42. RAR-M92 measures it against Stockfish 16 HCE.

### 2. The tree — existing evidence

- On Rarog's evaluation no donor search has done better than Rarog's:
  the head of 2026-11-13 led Stockfish classical's search by 59.8 ± 8.1
  and Stockfish's development search by 53.6 ± 8.2 with the bridge
  returned (RAR-O08), Reckless's unfitted search was level with it at
  equal work (+3.5 ± 8.4 for the donor, RAR-O09), and the compact kernel
  then gained +79.1 [+64.1, +94.4] over 2.7.0 (RAR-M91). Inferred by
  adding matches: the kernel is about 139 above Stockfish classical's
  search on this evaluation.
- The kernel's constants are fitted and quiet (RAR-S135: no gradient in
  two windows, theta +7.8 ± 9.1 over the start).
- Depth is not a ten-ply gap. Without mate scores the kernel reads 15.40
  and Stockfish 16 HCE 18.08 in RAR-M91's games (16.33 and 19.20 in
  RAR-M90's); by material 14.1 against 15.7, 13.7 against 16.1, 15.0
  against 18.0 and 21.9 against 25.9. Stockfish reports a mate score on
  7.4% of its moves, at depths that carry the mean to 25.

### 3. The evaluation — existing evidence and one read here

- Inside Stockfish's search at equal nodes Stockfish's classical
  evaluation is **+99.2 ± 16.6** over Rarog's (RAR-O07); the evaluation
  has not changed since. Phase C's gains moved into that foreign search
  at 0.8 or more of their size in Rarog's own games.
- **Static accuracy, read here** (`static_sf16.py`; RAR-O10's 10,000
  positions, Stockfish 19 labels and method, which this script reproduces
  for Stockfish 11 and Rarog's head to the digit: k 120.7 and 211.7,
  skill 0.2054 and 0.1769). Stockfish 16's classical evaluation: skill
  **0.2010**. Stockfish 16 − Stockfish 11 −0.0044 [−0.0087, +0.0005];
  Stockfish 16 − Rarog +0.0241 [+0.0121, +0.0358]. By non-pawn material,
  Stockfish 16 − Rarog: 45 or more +0.035 [+0.012, +0.056]; 25 to 44
  +0.002 [−0.017, +0.019]; 10 to 24 +0.028 [+0.016, +0.041]; under 10
  uninformative, as in RAR-O10. Not Elo: on this instrument Phase C's 82
  Elo moved skill by 0.0069.
- Berserk's static evaluation was not read: its `eval` prints "HCE eval
  trace not yet implemented".
- Where the kernel's decisive games are decided (its own score past 150
  for good; `m91_report.out`), losses minus wins by material band, from
  most to least: Stockfish 16 HCE +70, +56, +78, +27; Stockfish 11 +52,
  +35, +17, +1; Berserk −13, +25, +63, +23. A lagging sign of where a
  game was lost, not of why.
- From a piece up the kernel draws 20.6 and loses 2.8 games in 1,000
  (RAR-M90, 183 of 7,800); converting all of them would be about +10
  Elo, inferred from the score they would add (1.3%).

### 4. Search and evaluation together

- The kernel spends 43% of its node on search-side work, Stockfish 16
  HCE 34%, Reckless 19% of a node of which its network is 48%
  (D.7.8.12's bench profiles: 155 and 393 of 817 ns). The kernel's
  bookkeeping a node is its donor's (about 147 ns against 155 there);
  under a 95 ns evaluation it is the largest block. Whether that work
  buys the tree is not known.
- `LazyMargin` is not refitted on the kernel; the evaluation's labels are
  from the B.9 head's games, two searches ago (RAR-M67). No read here.

### 5. The instrument

- E.0's leads were per-game means from the GUI's database. Read a move at
  a time: the depth gap to Stockfish 16 HCE is 2.7 to 2.9 plies, not 10;
  the node-rate ratio is 1.32 to 1.42, not 1.37, and 1.48 as that
  tournament reports it.
- Nodes are one unit in name (a move made, D.6.31). They are not shown to
  be one unit of work: the share of quiescence nodes in each tree was not
  read.

## What the evidence says together (inferred)

Against Stockfish 16 HCE the −100 splits into about **−63** for the node
rate, **−99 ± 17** for the evaluation at equal nodes and, by subtraction,
about **+62** for the kernel's tree. The same sum against Stockfish 11
puts the kernel's tree about 116 above Stockfish 11's on one evaluation,
beside the 139 the search matches add up to for Stockfish classical's:
three chains that close within about 25 Elo. At an equal node rate the
kernel would be about 37 below Stockfish 16 HCE and above Stockfish 11
and Berserk. None of this is a measurement; RAR-M92 tests the first term,
and the reverse meter (lever 1) the second.

## Where this contradicts E.0's text or an earlier conclusion

1. *"The cost of a node: evaluation, board and memory traffic."* The
   board is level with Stockfish's, memory traffic between searches is
   not the gap, and the evaluation is 18 ns of the 117. The search side
   is 78.
2. *D.12's case.* The board is 22% of the kernel's node, as D.12 says,
   and costs what Stockfish's costs. The rebuild has no strength case
   against these engines; what is left is F.1's board events.
3. *RAR-O11: Stockfish's evaluation "about 120 ns a node" lighter.* That
   was read inside the hybrid. Natively the difference is 18 ns. The
   likely cause is the adapter, which rebuilds a call what Rarog's board
   carries incrementally; not tested.
4. *Explanation 2, a search built for a hand-crafted evaluation doing
   better here.* Nothing supports it: every such search read on this
   evaluation is far behind the kernel.
5. *E.4.0's donor question.* Stockfish 16's evaluation is not statically
   more accurate than Stockfish 11's, so Stockfish 16 HCE's 51 Elo over
   Stockfish 11 is not shown to be its evaluation; it is no faster
   either (2.127M against 2.179M in RAR-M91's games). Basilisk's games
   have `sf_16` and `9587eeeb` equal in play.
6. *D.7.8.5's retry trigger.* It closed on 1 search in 300 stopping
   inside an iteration, under the earlier clock. Under the adopted time
   manager 4.9% of the kernel's moves end more than 5 ms after the last
   report. The counters have not been read again.

## Levers

Prior bands frozen 2026-11-23, before any game: Elo at `3+0.03`, one
thread, against the pre-NNUE meter. Speed bands are written at 1.6 Elo a
percent of node rate and are restated at RAR-M92's price. A band is what
the step is expected to bank, not the lever's ceiling.

| # | Lever | Evidence | Band | Cheapest read that kills it | Owner |
|---|---|---|---|---|---|
| 1 | The evaluation's quality, in the donor's shape | +99.2 ± 16.6 in Stockfish's search; skill +0.024 to +0.029 behind both Stockfishes, in the 45-or-more and 10-to-24 bands | **+25 to +70** | The reverse meter: Stockfish's classical evaluation inside the kernel against the kernel on its own, equal nodes, both arms Rarog's driver, 2,000 games. Under +25 the rebuild is not indicated and E.4 is repairs. Needs a diagnostic build | E.4.0, as its first sub-step |
| 2 | Search-side work a node | 186 ns against Stockfish's 108; one short pass so far (+2.71%) | **+6 to +20** (4% to 12% of rate) | A function-level profile of the kernel's node, ordering and histories beside Stockfish's on game positions, zero games: no behaviour-neutral item of 1% or more closes it `NO_CHANGE` | D.16.3, opened by this packet |
| 3 | Endgame knowledge and conversion | 23.5 games in 1,000 thrown from a piece up (ceiling about +10); 15% of games reach a pawnless family Rarog does not scale; against Berserk the net loss sits at 24 or less of material | **+3 to +10** | E.4.2's Syzygy screen on the kernel's game positions, zero games: a family overclaimed in under 0.5% of positions is dropped | E.4.2 |
| 4 | The fit's data: labels from the kernel's own games | Labels two searches old; a corpus change alone read +11.8 ± 5.3 (RAR-E12) | **0 to +12** | The frozen surface refitted on a kernel corpus, one 2,000-game read; needs datagen, the maintainer's | E.4.0's method question, then F.4.1 |
| 5 | The evaluation's cost | 95 ns against 77; D.7.8.12 found no line above 0.5% | **0 to +8** (0% to 5%) | Lever 2's profile, the evaluation's functions | D.16.3 for neutral items; E.4's form otherwise |
| 6 | The interrupted iteration | 4.0% of the kernel's time follows its last report and its result is discarded; 3.3% of moves by more than 20 ms | **0 to +5** | D.7.8.5's counters on the head at `3+0.03`, zero games: under 2% of searches discarding a new best or failed-high move closes it again | D.7.8.5's retry; H.3 unless ordered earlier |
| 7 | Search-shape leads: reduction by the child's evaluation, removals, a transition correction, the dropped check rule | Coda's own +1 to +2 each; Rarog's tune is quiet; the kernel solved 201 WAC positions at 50,000 nodes where the classic solved 232 | **0 to +8** together | Each one's activation screen, then its `[0,3]` gate | H.2, not moved |
| 8 | `LazyMargin` on the kernel | Not refitted; no gradient past noise on the classic (RAR-S104) | **−2 to +4** | The margin block's refit at the next evaluation change | E.4, PROCESS's margin block |
| 9 | The board rebuild | 94 ns against Stockfish's 89 | **0 to +5** (0% to 3%) | Lever 2's profile, the board's functions | D.12, with no strength case |

Outside the one-thread meter, each with its own card and not ranked
against it: threads (D.13, D.14; four threads are worth 1.37 doublings
against Stockfish's 1.54, RAR-M83), tablebase policy (D.9, with E.1's
tablebase arm), the clock's unfitted constants (H.3). D.8's
simplification, D.11's disclosure and D.15's packaging carry no strength
and keep their own reasons.

## Unknowns, listed and not ranked

- How much of RAR-O07's 99 holds inside the kernel, whose margins were
  fitted on Rarog's evaluation. Lever 1's read answers it; until then
  its band is a prior.
- Whether the kernel's heavier node is the price of its tree. If it is,
  lever 2 has no neutral items and what remains changes the tree.
- The share of quiescence nodes in each engine's tree, so whether a node
  is one unit of work across engines.
- Berserk's evaluation.
- Every band at a longer control. Speed is worth less there by Rarog's
  own record; nothing here was read beyond `3+0.03`.
- The kernel's profile after D.7.8.12's pass and D.7.11's fit; the shares
  above are three days and two changes old.

## Cheapest discriminating tests

- **Registered, waiting for the maintainer: RAR-M92.** The kernel on
  `4.050+0.041` against Stockfish 16 HCE on `3+0.03`, 1,600 games, about
  25 minutes. Use: the Elo price of the node rate, which sets the speed
  bands and the speed work's place. Predicted gain +55 [+35, +75] over
  the baseline's −102.0 ± 12.7. Dry-run 2026-11-23, every field resolved.

  ```powershell
  ./tools/colosseum.ps1 -Mode match -EngineA tools/test_engines/rarog-d713kern-pext-pgo.exe -EngineB D:\chess\engines\stockfish-16-x64-bmi2-classical-pgo.exe -ForeignB -NameA "Rarog d713kern at 1.35 clock" -NameB "Stockfish 16 HCE" -Games 1600 -Seed 2026112392 -Hash 128 -BaseMs 4050 -IncrementMs 41 -BaseMsB 3000 -IncrementMsB 30 -MarginMs 2000 -OptionsB "Move Overhead=10","Use NNUE=false" -ExpectBench 3390842 -ExpectRevision eb84cce -ExtraArgs '--a-base-ms','4050','--a-increment-ms','41','--a-margin-ms','2000','--b-margin-ms','2000' -Dir tools/results/rar-m92-speed-price
  ```

- **Not registered: the same pair at equal nodes.** E.0 names it beside
  the time-odds read. It measures the same deficit through a worse
  instrument: under a node limit the kernel's last report covers 73% of
  its budget, against 96% on the clock.
- **Not registered: RAR-O08's matches (2) and (5) again on the kernel.**
  Its retry trigger fired at D.7, but the answer is not in doubt (about
  +139 and +57, inferred) and no decision here turns on it.
- **Needs a build, so a step and not a read of E.0's:** lever 1's reverse
  meter.

## Proposed order, for the maintainer

1. **RAR-M92**, now: 25 minutes, the price of every speed band.
2. **D.16, with D.16.3's card first.** The paired profile on game
   positions with Stockfish as the reference, aimed at the search-side
   block, with the evaluation's and the board's functions read in the
   same pass (levers 2, 5 and 9) and D.7.8.5's counters beside it
   (lever 6). Zero games, about a day. Its items, if any survive, are
   neutral and stack with everything after them; D.16.1 and D.16.2 then
   price them.
3. **E.4.0, opened by the reverse meter** (a new first sub-step, `I2`
   then `V`): Stockfish's classical evaluation, `sf_11`'s and `sf_16`'s,
   inside the kernel at equal nodes and on the clock. It sizes lever 1,
   answers the donor question in play and decides rebuild or repairs.
4. **E.4.1, then E.4.2's screen and E.4's units** in the form E.4.0
   decides; lever 4's corpus read where E.4.0's method question puts it.
5. **D.12 reduced** to D.12.0's research for F.1's board events; its
   leaves wait for a measured reason.
6. **D.8, D.9, D.10, D.11, D.13, D.14, D.15** as they stand; D.8.19 (the
   node frame) after step 2, which reads the same code.
7. **Lever 7** stays in H.2.

Step 2 is placed before step 3 because it is short, bounded by its own
kill test, and independent of the evaluation; if RAR-M92 reads +30 or
less it moves behind step 3.

## Falsifiers and stop conditions

- RAR-M92 under +35: the kernel's slope does not hold against Stockfish
  16 HCE; the speed bands shrink by the registered rule.
- The profile finds the search-side block equal once boundaries are
  matched function by function: lever 2 closes and the rate gap is the
  tree's composition.
- The reverse meter under +25: lever 1 is not a rebuild, and explanation
  4, the fit, takes its place as the question.

## Decision

`MORE_RESEARCH`, in one bounded sense: E.0's packet is complete at zero
games, and it closes when RAR-M92 is read and the maintainer has ordered
the steps.

## Measured and inferred

Measured here: node rates alone, under load and at a node budget; rates,
nodes a move and depth by phase in RAR-M91's games; the time after the
last report in RAR-M90's games; Stockfish 16 HCE's profile by region;
Stockfish 16's static accuracy; where decisive games are decided.
Taken from the ledger: every Elo figure, Rarog's profile shares, the
slope. Inferred: the price of the rate gap, the three-term split, the
kernel's standing at an equal rate, the conversion ceiling, every band.
