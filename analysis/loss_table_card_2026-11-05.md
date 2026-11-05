# D.4.5 card: the loss table by cell

Date: 2026-11-05. Leaf: PLAN D.4.5 (`V`). Source:
`analysis/triumviratus_review_2026-11-03.md` TT-1. Tool:
`tools/diag/loss_table.py`. Registered before any game has been judged.

## 1. The question

Where does Rarog lose its points: in which position class, with how much
clock left and with how much material, and how does that compare with its
opponents' moves in the same cells? Triumviratus found a third of its loss
to Stockfish 19 in one cell, defending with under a tenth of the clock, and
half its real errors gone with more nodes. Nothing here says whether Rarog
has such a cell.

Users: H.3 (TT-9's time in defence), E.1 (which re-reads the table at the
release head on E.2's games), the open draw-rate question (GUIDE,
*Conversion*) and the order of what follows.

## 2. The games

The C head's two one-thread pools, the last rated games of a Rarog binary
against other engines:

| Run | Ledger | Games of the C head | Field |
|---|---|---|---|
| `tools/results/c11-gauntlet` | RAR-M73 | 2,400 (83.6%) | Houdini 3, Critter 1.6a, Fritz 16, Rybka 4.1, Basilisk 1.10.0, Rarog 2.4.0 |
| `tools/results/pool2-rr` | RAR-M75 | 2,400 of 10,800 | Dragon 1, Stockfish 11, Houdini 6.03, Komodo 13.3 and 10.4, Houdini 4 and 3, Critter 1.6a |

Both at `3+0.03`, one thread, Hash 128, the UHO book; the C head is
`tools/test_engines/rarog-c11head-pext-pgo.exe` (SHA-256 `9750D3DE…`,
13,987,223), named `Rarog C.11 head` in both PGNs. The games come from the
CLI run directories: the CLI keeps no database, and Colosseum's GUI database
holds no game of the C head.

Left out, with the reason:

- **The four-thread pool (RAR-M74).** A four-thread search does not repeat
  at fixed nodes, so its errors cannot be searched again (section 6), and
  mixing thread counts would blur the clock cells.
- **The gates since the C head.** They are Rarog against Rarog, so the
  opponents' moves would control nothing, and the question is where Rarog
  loses to other engines.

The C head predates D.1's clock repairs. The clock cells describe that
clock; E.1's re-read on E.2's games describes the current one.

## 3. The sample

Triumviratus's rule, applied by `loss_table.py sample` with
`--spoil-cp 50 --spoil-moves 5`:

- every game the C head lost;
- every draw it spoiled: for at least five consecutive moves of each side,
  its printed score was at least +50 and its opponent's at most −50 (both
  saw it ahead); plies without a printed score are skipped;
- the twin of each (the pair's other game: the same opening with colours
  reversed), as the selection control.

Drawn on 2026-11-05 (game results and printed scores only, nothing judged):

| Run | Losses | Spoiled draws | Twins | Moves |
|---|---|---|---|---|
| `c11-gauntlet` | 173 | 107 | 254 | 70,091 |
| `pool2-rr` | 634 | 247 | 481 | 182,086 |
| **Total** | **807** | **354** | **735** | **252,177** in 1,896 games |

The losses by opponent: Dragon 1 162, Stockfish 11 130, Houdini 6.03 100,
Komodo 13.3 69, Houdini 3 64 + 44, Komodo 10.4 52, Houdini 4 48, Critter
1.6a 41 + 29, Fritz 16 36, Rybka 4.1 17, Basilisk 9, Rarog 2.4.0 6. The
sample file, `analysis/artifacts/d45-loss/sample.tsv` (SHA-256
`3cb3f6c4a66d9f74…`), records each `games.pgn`'s SHA-256, `46b703ec…` and
`8e8b02c4…`, the hashes RAR-M73 and RAR-M75 recorded; the judge refuses a
changed file. The sample is fixed: it is neither cut nor extended after the
judge has run.

## 4. The judge and the measures

**Judge:** Stockfish 19
(`D:\chess\engines\stockfish\stockfish-19-windows-x86-64-universal.exe`),
1,000,000 nodes per position, one thread, Hash 64, `UCI_ShowWDL`, one judge
per worker, `ucinewgame` before each game and the game's positions in order
with its moves, so each game's records repeat exactly. A final position the
rules decide (mate, stalemate, insufficient material, threefold, fifty
moves) is scored by the rule. If the smoke (section 9) puts the run above
eight hours at fifteen workers, the nodes drop to 500,000, decided from its
speed alone, before any sampled game is judged.

**Per move**, from the mover's side: the loss in expected score, the
judge's (W + D/2) before the move minus after it (the primary measure, in
points); the loss in centipawns, capped at ±300 when averaged; an error is
a move losing at least 50 cp (the donor's measures, for comparison).

**Losses telescope.** In each game the C head's summed loss minus its
opponent's equals its expected score at the first position minus the
judge's at the last, so the table divides each game's points among cells.
A game the rules end (mate, a rule draw) ends at its result; where an
adjudication ended it, the judge's last expected score minus the result is
carried by no move, and the tool reports that remainder apart. The tool
prints both for every read and fails when a game's losses do not
telescope. (Corrected 2026-11-05 by the smoke, before the sample was
judged: this paragraph said the losses telescope to the result, which
holds only where the rules end the game; section 10.)

Cautions. Stockfish 11 is an opponent and an ancestor of the judge, which
may favour its moves slightly. The sample is made of lost and spoiled
games, so comparisons are made within a cell, the C head against its
opponents there. Below about 10 cp a single move's loss is noise at these
nodes; the cells sum many moves.

## 5. The cells and the tables

Each move is placed by side (the C head, its opponents), by position class
from the judge's score before the move (defending at −50 cp or less, level,
ahead at +50 or more), by clock left before the move as a share of the
3,000 ms base (under 10%, 10–25%, 25–50%, 50% or more; reconstructed from
the PGN's charged time and the increment) and by men on the board (9 or
fewer, 10–15, 16–23, 24 or more). Read:

1. class × clock, the donor's table;
2. class × men;
3. by sample reason (losses, spoiled draws, twins);
4. by opponent.

For each cell: moves, games, mean capped centipawn loss, errors and their
share, points lost, the cell's share of the side's points lost, and the
mean in thousandths of a point. The C head's **excess** in a cell is its
points lost there minus its opponents'; the excesses sum to the sample's
start minus the judge's last, and the adjudication remainder is reported
beside them.

## 6. The worst moves, searched again

Every C head move that lost at least 0.10 points with at least 1,000 nodes
in the game, the 2,000 largest if there are more, is searched again by the
C head from a fresh table (Hash 128, one thread) at 1, 4 and 16 times the
game's printed nodes. The judge scores afresh the position before, the
game's move and each new move. Verdicts:

- **unconfirmed**: the fresh judge sees no loss of 0.10 in the game's move;
- **not reproduced**: a fresh search at the game's own nodes avoids it, so
  the error belonged to the game's table state or timing;
- **clock, 4× or 16×**: more nodes avoid it;
- **persists**: the error stays at 16×; its engine-minus-judge score gap
  says whether Rarog's evaluation was optimistic or pessimistic there.

Errors read as clock or evaluation only among those reproduced at 1×.

## 7. Predictions, frozen before any position is judged

| # | Prediction | Confidence |
|---|---|---|
| P1 | The C head's defending, under-10%-clock cell holds less than 20% of its points lost (the donor's 33.8% does not transfer) | 60% |
| P2 | The level class carries at least 40% of the C head's excess over its opponents (errors in balanced middlegames, where the evaluation deficit sits) | 55% |
| P3 | In defending positions the C head's mean loss per move with 15 men or fewer is at least 1.5 times that with 24 or more | 60% |
| P4 | Of the confirmed errors reproduced at 1×, between 30% and 60% are clock errors (more nodes repair them) | 55% |
| P5 | Between 20% and 50% of confirmed errors are not reproduced at 1× | 50% |
| P6 | In spoiled draws more than half the C head's points lost come from moves losing under 50 cp (drift, not single errors) | 55% |
| P7 | No prediction on the sign of the persisting errors' engine-minus-judge gap | — |

## 8. What the read means

- **H.3.** TT-9's defence-time rule has a target in this clock if the
  defending, under-10% cell holds at least 15% of the C head's points lost,
  its excess there is the largest of any clock band in defence, and at
  least 40% of its reproduced errors there are clock errors. Otherwise the
  rule has no target in the C head's clock; H.3 waits for E.1's re-read,
  since the current clock is D.1's.
- **The draw rate.** If drift (P6) carries the spoiled draws, the question
  points to conversion technique and the evaluation's scaling, not to single
  blunders; if errors carry it, their cells name where.
- **The order of what follows.** Persisting errors by class and their score
  gap say whether evaluation or search carries the loss; this is reported,
  not acted on here.

## 9. Validity and stop rules

- **V1, repetition.** Before the run, the smoke judges three sampled
  games twice, with one worker and with two, and the records must match
  byte for byte after sorting; the smoke's re-search, run twice, must match.
- **V2, identity.** In every game the C head's net loss and its start
  minus the judge's last agree to 10⁻⁶; `table` exits 1 otherwise.
- **V3, coverage.** Every sampled game is judged, every searched position
  printed a WDL, and the judge and the C head have the recorded hashes.

A failure of V1 or V2 voids the run until the tool is fixed. The run is
resumable; a crash continues it, and it is neither cut nor extended. The
smoke's three games carry no reading.

## 10. The smoke, 2026-11-05

Three sampled games, `c11-gauntlet` 15 (a loss), 16 (its twin) and 38 (a
spoiled draw), 376 moves, judged on section 4's settings by the tool at
`158f52f`; their numbers carry no reading. Logs and outputs are in
`analysis/artifacts/d45-loss/smoke/`.

- **V1.** One worker and two wrote the same 383 lines after sorting; a run
  cut inside its last game and resumed wrote them again; the re-search
  (moves losing at least 0.03, four of them, two workers, Hash 128), run
  twice, wrote the same nine lines.
- **V2.** The largest game difference was 5.6 × 10⁻¹⁷; a copy with one
  move's loss raised by 0.01 made `table` exit 1.
- **V3.** Every position printed a WDL (the judge stops otherwise);
  Stockfish 19's SHA-256 is `45bc8e49…` and the C head's `9750d3de…`.
- **A defect, fixed before the run.** The first smoke, on `6a48602`, passed
  V1, but its identity line called the judge's last score the result,
  the claim section 4 then made. `158f52f` writes each game's result into
  its records, reports the remainder apart and fails a game that does not
  telescope; the results above are its rerun.
- **Cost.** One worker judged 379 positions in 276.7 s, 0.73 s a position.
  The sample's 254,073 positions (252,177 moves and 1,896 last positions)
  take about 51.5 hours of one worker, about 3.4 hours on fifteen, so
  section 4's 1,000,000 nodes stand. The re-search took about 8 s a move
  on one worker, so 2,000 moves take about 20 minutes on fifteen.

The two runs are the maintainer's, on an idle host, the re-search after
the judge (a crash is resumed by the same command):

```
python tools/diag/loss_table.py judge analysis/artifacts/d45-loss/sample.tsv --judge D:/chess/engines/stockfish/stockfish-19-windows-x86-64-universal.exe --nodes 1000000 --hash 64 --workers 15 --out analysis/artifacts/d45-loss/judged.tsv
python tools/diag/loss_table.py research analysis/artifacts/d45-loss/judged.tsv --exe tools/test_engines/rarog-c11head-pext-pgo.exe --judge D:/chess/engines/stockfish/stockfish-19-windows-x86-64-universal.exe --hash 128 --min-loss 0.10 --limit 2000 --workers 15 --out analysis/artifacts/d45-loss/research.tsv
```
