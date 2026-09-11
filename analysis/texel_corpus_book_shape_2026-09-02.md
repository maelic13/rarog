# The datagen book was the wrong shape, not the wrong size — 2026-09-02

Owner: 4.9a.6. This note records why corpus regeneration needed roughly a
third of the games the extractor preflight asked for, and why the fix is the
start book rather than the schedule.

## The problem as it presented

The 20,000-game RAR-E08 pilot (`selfplay-e08head-n8000-s1-g20000.pgn`,
`datagen-v2`, 8,000 nodes, 1,905 games/min) preflighted at

```
train /opening       rate= 0.6735/game required=1,113,504
train /early_mid     rate= 1.3006/game required=  576,680
train /middlegame    rate= 2.2890/game required=  327,654
train /endgame       rate= 4.0769/game required=  183,964
train /deep_endgame  rate= 2.8890/game required=  259,606
Recommended total independent games: 1,113,504
```

The binding bucket is `opening`, which was the opposite of the expectation
going in — dropping adjudication was supposed to leave the deep endgame
starved, and instead it is the most abundant bucket at 2.889/game.

1,113,504 games is more than the 750,000 openings in `beast_seed.epd`, and
`datagen.ps1` refuses to wrap. So the target was unreachable at any schedule.

## What was actually wrong

`beast_seed.epd` holds **exactly 150,000 positions in each of the five
buckets**. Its phase is the extractor's, which is MATERIAL and not ply:

```
opening 20-24 | early_mid 14-19 | middlegame 8-13 | endgame 3-7 | deep_endgame 0-2
```

A game started below phase 20 can therefore never produce an `opening` row.
Splitting the pilot PGN by the phase of its own start position and preflighting
each split separately gives the yield matrix (train rows per game):

| start bucket | opening | early_mid | middlegame | endgame | deep_endgame | total |
|---|---|---|---|---|---|---|
| opening | **3.4392** | 2.9157 | 2.8238 | 3.0342 | 1.0922 | **13.30** |
| early_mid | 0.0008 | 3.6496 | 3.4337 | 3.7585 | 1.5484 | 12.39 |
| middlegame | 0.0000 | 0.0084 | 4.7598 | 4.7542 | 2.1278 | 11.65 |
| endgame | 0.0000 | 0.0045 | 0.2757 | 6.5862 | 2.9842 | 9.85 |
| deep_endgame | 0.0000 | 0.0005 | 0.2820 | 2.1865 | 6.6310 | 9.10 |

Two facts follow, and they are the whole finding:

1. **Only an opening start feeds the opening bucket.** The 0.0008 on the
   `early_mid` row is promotion restoring material; every row below is zero.
   80% of a balanced book cannot contribute to the bucket that binds.
2. **An opening start is also the most productive overall**, because one game
   traverses every phase on its way down. The balanced book spends four fifths
   of its starts on the least productive positions.

The observed mixed-book rate is the matrix times the start distribution:
0.6735 / 0.196 = 3.44, which is the `opening` row exactly. The preflight was
not wrong; it sizes GAMES and had no way to say the BOOK was the constraint.

## Supply was never the constraint

The read-only store `A:\Chess\Beast\data\txt\positions.txt` is 7,121,976,716
bytes. Sampling 826,608 lines from 12 sequential chunks:

- average line 57.1 B, so **~124.8M positions**
- phase mix **36.8 / 21.4 / 20.6 / 17.4 / 3.9 %** — about **46M opening-bucket**
- exact 4-field duplicate rate within the sample **0.02%**

150,000 per bucket was a quota someone chose, not a ceiling anyone hit.

## Choosing the composition

Maximising the worst-off bucket over the matrix (grid search, step 0.02, the
per-bucket yield being linear in the start mix) puts the optimum at
**68/10/0/0/22** for 2.326 rows/game/bucket. The adopted default is the hedged
**50/10/10/10/20** at 1.720:

| composition | min rows/game/bucket | games for 3.0M rows |
|---|---|---|
| current 20/20/20/20/20 | 0.688 | 1,090,116 |
| hedged **50/10/10/10/20** | **1.720** | **436,127** |
| LP optimum 68/10/0/0/22 | 2.326 | 322,498 |

The corner is not taken because it buys its extra rows by making every
middlegame and endgame row a *reached* position, correlated with the opening
play that led there. Keeping 10% direct starts in each of those buckets holds
them independently sampled, at a cost of 26% of the theoretical yield.

At the pilot's 1,905 games/min the hedge puts 3.5M rows (Basilisk's target) at
~509k games, about **4.5 hours**; 5.0M at ~727k games, about 6.4 hours.

## Two secondary levers, measured and not taken

- **`--skip-start`.** Rarog discards the first 2 plies of every game; Basilisk
  discards none. Those plies are opening-phase by construction, and on
  opening-start games the cost is 3.7796 -> 3.4392 rows/game, or **9% of the
  scarcest bucket**. Left at 2 for now so the corpus contract still matches
  `hce-v2` in everything except the book.
- **`--max-per-game`.** Rarog caps at 16, Basilisk at 0 (uncapped). Raising it
  to 40 moves the mixed-book opening rate 0.6735 -> 0.9458, but it buys rows
  with within-game correlation rather than with games. The book fix is
  strictly better and this is not needed.

Basilisk also exposes `--phase-weights` (default `1,1,1,1,1`); Rarog's quotas
are hardcoded equal. That knob is not the fix here — the weights control the
extractor's QUOTA, and no quota can conjure opening rows out of games that
never had full material.

## Stockfish is not a reference for this

Checked at source rather than from recollection: `D:\code\stockfish\src\tune.h`
is a harness that exposes parameters as UCI options for **fishtest**, and there
is no logistic, sigmoid or Texel fitting anywhere in the tree. Stockfish tuned
its HCE by playing games (SPSA on fishtest), not by fitting labelled positions,
so it has no corpus-sizing practice to borrow. Its analogue of this problem is
game budget, which our SPRT and SPSA rules already cover.

## Reproducing this

The yield matrix, from the pilot PGN and nothing else:

```bash
python tools/diag/book_yield.py tools/texel/data/selfplay-e08head-n8000-s1-g20000.pgn
```

The store profile and the book built from it:

```bash
python tools/texel/build_book.py --count 1000000 --out tools/texel/data/phase_book_v1.epd
```

`build_book.py` imports `PHASE_BUCKETS` and `PHASE_W` from `extract.py` so the
book's phase definition cannot drift from the extractor's, and it **shuffles**
its output — `datagen.ps1` hands out contiguous segments from `-Start`, so a
book grouped by bucket would give each segment a single phase. Verified: 500-
position segments at offsets 0, 1000, 2500 and 4500 carry 51/50/50/49% opening.

The book also reproduces the measured `beast_seed.epd` contract, which was
checked rather than assumed (30,000 entries: 0 in check, 0 terminal, 49.7%
white to move). Rebuilding `beast_seed.epd` itself through `build_book.py`
rejects 0 positions, which is the cross-check that `keep()` agrees with it.
