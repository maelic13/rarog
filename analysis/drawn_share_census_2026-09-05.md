# Drawn-share bias census (PLAN 4.11.4)

Conversion asks whether the engine finishes a win. This asks the complementary
question a SCALE function exists to answer: **does the evaluator claim an
advantage in positions the tablebase calls dead drawn?** An engine that scores a
drawn rook ending at +300 will steer into it from a position it could have won
differently, and no conversion measurement taken inside the ending can see that
happen.

## Conditions

`tools/diag/endgame_drawn.py`, 1,500 sampled positions per family over 19
families, tablebase-filtered to the theoretical draws, each searched at
**60,000 nodes**, one thread, 16 MB hash, 30 workers. Overclaim = the share of
drawn positions scored above **+100 cp** for the strong side. Seed `6200600`,
family-name seeded, so the sample matches `endgame_truth.py`'s.

Artifact: `tools/results/drawn-census/drawn-v1.json`.

Layer: this is the drawn-share measurement, not one of the four
(`analysis/endgame_measurement_layers.md`). It grades static claims about drawn
positions; it is not a strength claim.

## Result

| Family | drawn / 1500 | overclaim | mean cp |
|---|---:|---:|---:|
| **KRP-KB** | 38 | **1.0000** | +328.1 |
| **KR-KN** | 796 | **1.0000** | +346.0 |
| **KR-KB** | 1002 | **0.9960** | +307.4 |
| KBP-KB | 884 | 0.6086 | +142.0 |
| KNN-KP | 1009 | 0.5768 | +159.6 |
| KBP-KN | 635 | 0.5071 | +143.6 |
| KRP-KR | 482 | 0.3071 | +84.0 |
| KR-KP | 125 | 0.2640 | +72.1 |
| KBP-K | 57 | 0.2456 | +90.3 |
| KPP-K | 25 | 0.0800 | +39.3 |
| KP-K | 371 | 0.0458 | +18.0 |
| KP-KP | 502 | 0.0378 | +4.5 |
| KNN-K | 1499 | **0.0000** | +0.0 |
| KR-K, KQ-K, KBB-K | 0 | thin | — |
| KQ-KR | 2 | thin | — |
| KQ-KP, KBN-K | 13 | thin | — |

Families below 25 drawn positions are reported as thin rather than as a rate.
KQ-K, KR-K and KBB-K have **no drawn subset at all**, which PLAN 4.9a.2 already
predicted from theory and this confirms empirically. KPP-K sits exactly at the
threshold with n=25 and its 0.0800 should be read as indicative only.

## What it says

**Rook against a lone minor is priced as close to winning, always.** KR-KN
overclaims **every one** of its 796 drawn positions and KR-KB 998 of 1002, at
means of +346 and +307. Those are far above the material difference the fitted
evaluator assigns (rook 537 mg against knight 394, bishop 418), so this is not
"material says a rook is better" -- positional terms are stacking on top of a
material edge in positions that are theoretically dead. There is no drawishness
scaling for these endings at all.

**KNN-K is perfect and is the control.** 1,499 of 1,500 sampled positions are
drawn and the evaluator claims **zero** of them. The recognizer that exists
works exactly as intended, which is what makes the failures above legible as
missing knowledge rather than as an instrument artifact.

**KRP-KR at 0.3071 is the family 4.9a.7 worked**, and it now sits mid-table
rather than at the top. Its recorded 37.1% -> 25.8% was measured on a different
sample size, so these are not the same number; the useful reading is relative --
it is no longer among the worst.

## Why this had to come before the re-ranking

The two rankings barely overlap:

| By conversion deficit (4.11.3) | By drawn-share bias |
|---|---|
| KQ-KR 23 | KR-KN 1.0000 |
| KBN-K 10 | KR-KB 0.9960 |
| KNN-KP 9 | KBP-KB 0.6086 |
| KR-KP 8 | KNN-KP 0.5768 |
| KRP-KR 5, KRP-KB 5 | KBP-KN 0.5071 |

**KR-KB and KR-KN have conversion deficits of 2 and 3 -- among the smallest in
the cohort -- and are the two worst drawn-share offenders in it.** They finish
their wins about as well as the reference does and misprice their draws
completely. Ranking 4.12 on conversion alone would have put them near the
bottom.

The mirror image is **KQ-KR**: the largest conversion deficit at 23, and only 2
drawn positions in 1,500, so drawn-share has nothing to say about it. It is a
pure technique problem.

This is 4.9a.7's lesson arriving from the other direction. That step nearly
concluded a working scale change had done nothing, because it was read on
conversion. Here, three families would have been called healthy for the same
reason.

## Consequence for 4.12.1

4.12's provisional table classifies **KRKB (ref 7) and KRKN (ref 8) as
VERDICT functions**. Their measured defect is drawn-share, which is
scale-shaped. 4.12.1 owns the classification and should revisit those two
against this evidence rather than against the reference's own taxonomy --
what matters is which instrument can see the defect, not what the donor called
the function.

## Two instrument defects found and fixed while doing this

1. **The tool's numbers were order-dependent.** `engine.analyse` was called with
   no `game=` token, so no `ucinewgame` was sent between positions and the
   transposition table carried over: a position's score depended on which
   positions preceded it. Caught by the serial-versus-sharded byte-identity
   check, where KBP-KB read 0.702 serially and 0.750 over six workers **on the
   same positions**. Fixed by forcing `ucinewgame` per position; serial and
   sharded output are now byte-identical. Prior drawn-cohort numbers (4.9a.7,
   4.9a.8) were paired within themselves and so remain valid as comparisons,
   but their absolute rates carried this contamination.
2. **A completed census died on its write** because the tool did not create its
   output directory, losing 28,500 positions of work. `endgame_truth.py` had
   always done this; `endgame_drawn.py` had not.
