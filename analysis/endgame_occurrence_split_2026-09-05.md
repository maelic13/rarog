# Occurrence split by root, and why one threshold is a choice (PLAN 4.11.5)

Basilisk's occurrence census read 47.9% and, restricted to non-endgame roots,
**zero** -- the entire signal had come from the suite's own endgame roots
(BAS-E43). 4.11.5 exists to check whether Rarog's census has the same defect.

**It does, and the first answer I got said it did not.** That is the finding.

## Method

`tools/diag/endgame_occurrence.py`, driving the sanctioned counter reader
(`bench_counters.py`, extended to retain per-position values). `bench` dumps
counters once per position, so attribution to roots needs no extra run. The 40
FENs are parsed from `src/bench.rs` so the suite cannot drift away from its own
classification, and the parse is checked against the array's declared length.

`bench 13` under `--features diag` reproduces **6,901,489 / EBF 2.458**, so the
diag feature's own gate holds and these counters describe the shipped search.

## The result depends entirely on where the line is drawn

| endgame root = <= men | endgame roots | middlegame evaluations | share | families reached only from endgame roots |
|---:|---:|---:|---:|---|
| **7** | **0** | 80,589 | **1.0000** | none |
| **8** | **3** | 35,435 | **0.4397** | KQKR, KQKRPs |
| **10** | **8** | 4,494 | **0.0558** | KBPKB, KBPKN, KBPPKB, KPKP, KQKP, KQKR, KQKRPs, KXK |
| 12 | 10 | 242 | 0.0030 | + KPK, KPsK, KRKP |
| 14 | 14 | 84 | 0.0010 | + KBPsK |
| 16 | 16 | 65 | 0.0008 | |

**Three roots out of forty produce 56% of every reference-family evaluation in
the run. Eight produce 94%.**

At a 7-man threshold the suite contains no endgame roots at all, because its
smallest position is 8 men, and the census looks perfectly clean. That was my
first reading and it was reassuring and useless. Moving the line by one man
turns it into "more than half of this measurement comes from three positions".

So the tool now sweeps and prints the whole curve, and `--endgame-men` chooses
only which threshold gets the detailed per-family table. **A report quoting one
threshold would be quoting a choice, not a measurement.**

## What survives either reading

Four families read **zero over all forty roots**: KBNK, KNNK, KNNKP and KRKN.
No threshold argument touches those -- a depth-13 search from any of these 40
positions never reaches them.

The families that dominate the census -- KRPPKRP 5.88%, KQKRPs 4.41%, KRPKR
3.46% -- are exactly the ones that collapse when late-material roots are
removed. KQKRPs is reached only from endgame roots at a threshold as low as 8.

## The tension this creates for 4.11.6

**KR-KN has tree occurrence ZERO and the worst drawn-share bias in the cohort**
(4.11.4: 796 of 796 drawn positions overclaimed, mean +346). Those two facts
have to be reconciled by the re-ranking, not averaged away.

The honest reading is that the bench suite is a weak instrument for this
question. It is 40 positions chosen to fingerprint the SEARCH, not to sample the
game distribution -- the analysis it supersedes says so itself. A family can be
unreachable from those 40 roots at depth 13 and common in real games at real
time control.

**A far better occurrence corpus exists and is already on disk**: the 36,400-game
rated tournament. Occurrence measured over real games at 3+0.03 would answer the
question the bench suite is being asked to answer and cannot. That is proposed
rather than done here, because it is outside this leaf.

## What this does not say

Occurrence PRIORITISES. It is never evidence of value
(`analysis/endgame_measurement_layers.md`). A zero here is a reason to rank a
family lower, not a reason to call its defect acceptable -- and a family absent
from middlegame trees can still occur inside four-man endgame trees, which is
how Basilisk's KBNK term leaked into families its own safety argument had
excluded (BAS-E49).
