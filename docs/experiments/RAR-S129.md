# RAR-S129 — D.7 audit: the compact kernel against its donor's hybrid at equal nodes, and the evaluation wrapper's share of RAR-S128's interim gain — REGISTERED 2026-11-19, before any game; READ 2026-11-19: −11.6 ± 10.0 Elo at equal nodes, the root driver's rule not separated; the wrapper pair +24.9 ± 9.0

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-19, before any game. Both reads are
  run by the agent inside the D.7 audit the maintainer asked for
  2026-11-19 ("measure anything extra that you need"), with RAR-S128's tune
  paused at iteration 2,000.
- Disposition: observation — read 2026-11-19: (P) −11.6 ± 10.0 Elo, wholly below zero, with a root
  driver asymmetry the read does not separate from the kernel; (W) +24.9 ± 9.0 Elo, 44% of the
  interim gain. Both feed the D.7 audit; neither changes the tune or the gate.
- **Kind:** two diagnostic fixed matches. They accept nothing and gate
  nothing; D.7.7's gate is unchanged.
- **Inherits:** RAR-O09 (1) (the hybrid +3.47 ± 8.37 Elo over the head at
  equal work), RAR-S126 (the unfitted compact kernel −6.9 ± 9.5 Elo against
  the classic at `3+0.03`), D.7.4's parity (layer A without a mismatch,
  layer B inside bands of ±0.12 to ±0.14 in log tree size) and RAR-S128's
  interim read (centres at 2,000 against the seeds: 702 W / 920 D / 378 L).

## Why

1. **Parity in strength was never read.** Layer A compares decisions on the
   same inputs and layer B tree sizes inside a band of about ±13%. Neither
   sees the state that feeds the rules (the table, the histories, the root),
   and neither is Elo. RAR-S126's arithmetic leaves room: the hybrid was
   level with the head at equal work (+3.5), the compact kernel searched
   4.4% more nodes a move than the classic in games (about +9 at PROCESS's
   2 Elo per 1%), and it read −6.9: about 19 below that sum, three standard
   errors of the two reads' own noise, before the conversion's uncertainty.
   A read at equal nodes separates a port or state defect (the shortfall
   stays) from the clock side (it goes).
2. **What the fit is fitting.** At 2,000 the two wrapper coordinates
   travelled furthest (`KWrapRawBase` 21,032 to 16,385, `KWrapDivisor`
   27,015 to 32,596: the evaluation the kernel sees shrinks by 29% with
   every piece on and by 35% with none), half of it inside 250 iterations.
   Whether the interim gain is mostly that one scale says whether the
   fold's scale (mean |network| over mean |HCE|, 2536/1024) is the kernel's
   main misfit, and how much the other 228 coordinates have yielded.

## Use of the readings (frozen)

- **(P) parity.** Interval wholly below zero: the port is weaker than its
  donor's form in a way the parity instrument does not see; the audit
  reports it as a defect to diagnose (table, histories, root) before more
  is invested in a fit, and RAR-S128's continuation is weighed against it.
  Anything else: the port is taken as its donor's equal in strength to the
  interval's width, the audit says so, and RAR-S126's shortfall is then
  read as the clock side's (Rarog's time management and root driver
  driving the donor's kernel), a lead for D.6.28 and the next fit's
  surface, not a port defect.
- **(W) the wrapper's share.** Read against the interim read's gain: the
  share of it the two wrapper coordinates carry alone. It feeds the audit's
  recommendation on the fold's scale and on continuing RAR-S128; it stops
  or changes nothing in the tune by itself.

## Arms and runs (frozen)

Colosseum `match` through `tools/colosseum.ps1`, 2,000 games each, Hash 64,
one thread, the UHO book in random order, no adjudication, seed 20261120.
Dry runs passed every preflight 2026-11-19
(`colosseum_match_rar-s129-parity_20261119_135235.dry-run.json`,
`colosseum_match_rar-s129-wrap_20261119_135253.dry-run.json`).

- **(P)** `d75cand` against `reckless-hce`, **100,000 nodes a move each**
  (both count a move made, the null move not counted, since D.6.31), no
  clock. `d75cand` is RAR-S126's candidate, `rarog-d75cand-pext-pgo.exe`
  (`2820eaa`, `compact-kernel`, bench 7,543,485, SHA-256 `a8ead317…`): the
  port as built, before the line bound, which binds no bench tree and which
  the donor lacks. `reckless-hce` is RAR-O09's arm, a foreign engine
  (`analysis/artifacts/rar-o09-2026-11-14/reckless-hce.exe`, Reckless
  `0756957` with the hybrid patch on Rarog's frozen evaluation, SHA-256
  `8935a1b8…43296`). Directory `tools/results/rar-s129-parity`.

  ```powershell
  ./tools/colosseum.ps1 -Mode match -EngineA tools/test_engines/rarog-d75cand-pext-pgo.exe -EngineB analysis/artifacts/rar-o09-2026-11-14/reckless-hce.exe -NameA d75cand -NameB reckless-hce -ForeignB -Nodes 100000 -Games 2000 -Seed 20261120 -ExpectBench 7543485 -Dir tools/results/rar-s129-parity
  ```

- **(W)** the tune binary on both sides (`rarog-d76fit-tune.exe`,
  `f7c89fb`, SHA-256 `a536482f…`, `-CategoricalTuneBuild`), `3+0.03`:
  `d76wrap2000` sets only `KWrapRawBase=16385` and `KWrapDivisor=32596`,
  the interim read's values; `d76seed` keeps every seed. Directory
  `tools/results/rar-s129-wrap`.

  ```powershell
  ./tools/colosseum.ps1 -Mode match -CategoricalTuneBuild -EngineA tools/test_engines/rarog-d76fit-tune.exe -EngineB tools/test_engines/rarog-d76fit-tune.exe -NameA d76wrap2000 -NameB d76seed -OptionsA KWrapRawBase=16385,KWrapDivisor=32596 -ExpectBench 7543485,7543485 -Dir tools/results/rar-s129-wrap -Games 2000 -Seed 20261120
  ```

**Validity:** every game a rules result; a run with time losses or engine
faults in more than 0.5% of its games is void. (P): no move of either side
reports more than its budget plus two nodes (RAR-O09's amended tolerance).

**Reading:** each manifest's Elo and nElo with their 95% intervals.

## Zero-game companion, read before the games

`bench 13` on the tune binary (SHA-256 `a536482f…`), one UCI session each
(`activation.py`'s runner): the seeds **7,543,485 / EBF 2.422**; the centres
at 2,000 **3,488,485 / 2.336**; the two wrapper values alone **5,678,657 /
2.398**; the other 228 alone **4,171,048 / 2.344**. A fingerprint, not a
tree size and not Elo: the fit has roughly halved the tree at a depth, and
both parts move it.

## PRE-REGISTERED PREDICTION (frozen 2026-11-19, before any game)

- **(P):** `d75cand` **−5 Elo**, 80% band **−20 to +10**; the interval
  wholly below zero with probability 0.2. Basis: every rule reproduces the
  donor's decision and tree sizes agree, so near zero; taken down a little
  for RAR-S126's shortfall against the sum of RAR-O09 (1) and the speed
  read, and for the kept departures (Rarog's table and root, SEE-2 and
  SEE-3, position 305's second divergence).
- **(W):** `d76wrap2000` **+30 Elo**, 80% band **+10 to +50**, a little
  over half the interim read's gain. Basis: the wrapper pair travelled
  furthest and fastest, and the first hundred iterations' score spread was
  twice the game noise, which a few steep coordinates explain; against it,
  the other 228 move the bench as much as the pair does.
- **Most likely failure mode:** (W) reads near the whole interim gain, and
  the fit of the other 228 coordinates is then unproven at 2,000.
- **Evidence layer:** games. (P) at fixed nodes says nothing of speed or
  clock; (W) is on a tune build without PGO, both sides alike.

## Result (read 2026-11-19)

Both matches run by the agent 2026-11-19, 12:55 to 13:37 UTC, Colosseum CLI
0.5.0 (pinned `8fa63cc`), the host idle at each start (3.9% and 1.8%), the
tree clean at `ed5046a`. Each line below is the runner's report or the
wrapper's manifest. While (W) played, the agent ran four single-threaded
reads of PGN files, a few seconds each, on the host's spare core; both
sides of (W) are one binary and shared it. (P) is at fixed nodes.

### (P) the compact kernel against its donor's hybrid, 100,000 nodes a move

`tools/results/colosseum_match_rar-s129-parity_20261119_135553.manifest.txt`:

- "Elo -11.6 +/- 10.0; nElo -17.7 +/- 15.2 over 2000 paired games"
- `d75cand` 543 W / 847 D / 610 L; pentanomial 60, 249, 430, 220, 41
  ("recount of games.pgn agrees"); faults none, time losses none.
- search, `d75cand`: depth 17.87 mean (non-mate 16.08), 78,889 reported
  nodes and 43.6 ms a move; `reckless-hce`: depth 20.98 (non-mate 17.29),
  98,169 reported nodes and 69.9 ms a move.
- **Validity, a deviation:** `d75cand` never reports more than 100,000.
  `reckless-hce` reports 3 to 10 nodes over its budget on 126 of 141,447
  moves (0.09%: 102 at three over, 16 at four, 5 at five, one each at six,
  seven and ten), past the registered budget plus two. It is RAR-O09's
  class (the donor tests its limit at node entry) and at most 0.01% of a
  budget; the match is read with the deviation recorded, and the maintainer
  may void it.

**Reading by the frozen use:** the interval [−21.6, −1.6] is wholly below
zero, so the use asks for a diagnosis before more is invested in a fit.

**The diagnosis, at zero games, after the read.** Two differences between
the arms are not the kernel's:

1. *The root driver's rule on a stopped search.* Rarog plays the last
   completed iteration's move and discards the interrupted one
   (`src/search/mod.rs`, the window loop's `break` on a stop); the donor
   plays `root_moves[0]` with the interrupted iteration's results. Under a
   node budget every search is stopped inside an iteration. From this
   match's PGN, `d75cand`'s last report covers 81% of its budget where
   |score| < 100 and 85% at 100 to 300, and 6% to 7% of its moves report
   the whole budget; `reckless-hce` reports 99% to 100%. The donor's
   depths above are the interrupted iteration's, Rarog's the last
   completed one's, which is most of the 1.2 plies between them.
2. *The same rule applied to the classic kernel in RAR-O09 (1),* whose
   `rarog-head` reports covered 74% to 77% of its budget in the same bands,
   and there the hybrid led by 3.47 ± 8.37.

So this read and RAR-O09 (1) carry the same driver asymmetry, and chained
they put the unfitted compact kernel about 8 Elo below the classic at equal
nodes (±13), which is RAR-S126's −6.9 ± 9.5 on the clock. The reach of the
donor's rule, measured: on 400 positions (every twelfth ply of this match's
first games, a fresh process each, `go nodes 100000`) the hybrid's move
differs from its last completed iteration's on 11 (2.8%). Its worth in Elo
is not measured, so the read does not separate the driver's rule from a
kernel-side residue; it bounds such a residue at about 11 Elo if the rule
were worth nothing. One kernel-side defect is known from the audit's code
review, found before this read: ProbCut's exchange bar was not converted
by the fold (`node.rs`, `probcut_beta - eval` in evaluation units against
the donor's exchange values), which D.7.4.1's own layer B rate anomaly
(`probcut_depth` firing 27% more a position) already showed.

*Calibration.* Predicted −5 Elo (80% band −20 to +10; wholly below zero
0.2); measured −11.6 ± 10.0, inside the band, the 0.2 event. The assumption
that failed was in the instrument, not the sign: equal node budgets were
taken as equal work between two root drivers, and they are not while one
discards its interrupted iteration. RAR-O09 (1) carries the same flaw in
the classic kernel's disfavour.

### (W) the two wrapper coordinates alone, `3+0.03`

`tools/results/colosseum_match_rar-s129-wrap_20261119_141541.manifest.txt`:

- "Elo +24.9 +/- 9.0; nElo +42.0 +/- 15.2 over 2000 paired games"
- `d76wrap2000` 591 W / 961 D / 448 L; pentanomial 16, 200, 449, 295, 40
  ("recount of games.pgn agrees"); faults none, time losses none.
- search: `d76wrap2000` depth 16.52 mean, 104,544 nodes and 59.7 ms a
  move; `d76seed` 16.18, 106,321 and 60.9 ms; both 1.75 M nodes a second.

**Reading:** the wrapper pair alone carries +24.9 of the interim read's
+56.8 ± 9.5 (RAR-S128, centres at 2,000 against the seeds), 44%. The other
228 coordinates then add about +32 (±13, a difference of two reads, and
the two parts need not add). The fold's scale, mean |network| over mean
|HCE|, is the kernel's largest single misfit and not the whole of it.

*Calibration.* Predicted +30 Elo (80% band +10 to +50); measured +24.9 ±
9.0, inside the band. The named failure mode (the pair carrying nearly the
whole gain) did not occur.

### What the audit takes from both

- The port is not shown weaker than its donor's form beyond what the root
  driver's rule and the fold's ProbCut bar can carry; neither is separated
  in Elo here. Both go to D.7.8 as named items.
- The fit works on the surface as a whole: under half its gain at 2,000 is
  the evaluation's scale. The kernel's scores are then 0.50 to 0.82 of the
  HCE's, which the time manager's falling-score slopes and the reported
  score read unconverted: an item for D.7.9.
- Scripts and logs: `analysis/artifacts/d7-audit-2026-11-19/` (ignored):
  `nodeshare.py` and `nodeshare_fc.py` (the report's share of the budget,
  this match and RAR-O09's), `partial.py` (the hybrid's interrupted
  iteration), `bench3.py` (the zero-game companion), `traj.py` and
  `groupgain.py` (RAR-S128's travel and its gain by coordinate group), and
  both runs' logs.
