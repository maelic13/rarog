# What a move actually costs at 3+0.03 (PLAN 4.10.6)

Every fixed-node screen in this project implies a claim it never checked: that
its budget stands in for what the engine really gets at the time control it is
tested and shipped at. Measured now.

## Method

`tools/diag/nodes_per_move.py`, 4 self-play games at **3+0.03**, one thread,
64 MB hash, on the accepted head. The clock is managed by the harness and
decremented by MEASURED wall time, so the engine receives genuine
`wtime/btime/winc/binc` and exercises its own time management; a fixed
`movetime` would measure a different code path (RAR-M01). 492 moves sampled.

This is a **run condition**, not a layer -- it does not measure play. See
`analysis/endgame_measurement_layers.md`.

## Result

| Statistic | Nodes/move |
|---|---:|
| min | 4 |
| p25 | 114,780 |
| **median** | **153,466** |
| mean | 176,208 |
| p75 | 210,088 |
| p90 | 319,892 |
| max | 650,615 |

By move number, descriptive only:

| Band | Moves | Median |
|---|---:|---:|
| opening (plies 0-29) | 120 | 164,631 |
| middlegame (plies 30-79) | 200 | 171,472 |
| endgame (plies 80+) | 172 | 115,899 |

The `min` of 4 is a position with one legal move, not an anomaly.

## Cross-check

The maintainer's 12,000-game Colosseum arena on an Apple M4 recorded Rarog at
**2.0 M nps and 74 ms/move**, which is **~148,000 nodes/move** at the same
3+0.03. That is an independent instrument on different hardware agreeing with
the 153,466 median here to within 4%. Two measurements agreeing is worth more
than either alone, and it is the reason to believe this number rather than
merely to have it.

## What it means for the screens

**The endgame screen budget is 60,000. Deployment is 153,466 median, and
115,899 even in the endgame band.** So every fixed-node endgame verdict this
project has taken was measured at roughly **2.6x below** the deployment budget,
and 1.9x below it in the phase the screens are actually about.

That does not make 60,000 wrong. It makes any verdict taken there
**PROVISIONAL**, in the precise sense of PLAN rule 12: a losing move that a
116,000-node search sees can be invisible at 60,000, and Basilisk rejected its
leading KBNK candidate on exactly such a move -- a rejection that did not
reproduce at 200,000 or 600,000 (BAS-E45).

**The 60k / 200k / 600k bracket is now justified rather than copied.** Against
this distribution it straddles deployment properly: 60,000 sits below p25,
200,000 just above p75, and 600,000 near the observed maximum. The bracket is
run by `tools/diag/endgame_budget_bracket.py`, which drives `endgame_truth.py`
unchanged at each budget over the same cohort and refuses to tabulate if the
arms measured different position sets.

## What is NOT decided here

- **The primary budget stays 60,000 for now.** 4.11.1 re-measures the corrected
  baseline, and changing the budget in the same step as the termination rule
  would confound the one delta that step exists to isolate.
- **Which budget a family verdict is taken at belongs to 4.11.5**, which owns
  the bracket runs, and to each 4.12 leaf for its own family.
- 4 games on one machine is a small sample for the tails; the median and
  quartiles are what this is used for, and the M4 cross-check supports them.
  Re-measure after any change to time management (4.17) or a large NPS change.
