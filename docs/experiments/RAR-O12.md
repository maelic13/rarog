# RAR-O12 — RAR-O11's equal-node reads again, at equal work in one node unit

Indexed under *3. Search and selectivity › Search-oracle observations* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-15 (PLAN D.7.2.4), before any game. Preparation is the agent's; the night command is the maintainer's (about four hours).
- **Kind:** observation. It replaces RAR-O11's withdrawn equal-node reads (its (d2), (d4) and the 150,000-node columns of (a)) and decides nothing by itself; with RAR-O11's clock reads and D.7.2.3 it is an input to the maintainer's decision on D.7's donor.
- **Why:** RAR-O11 compared engines at equal *reported* nodes, which are not equal work. Each engine was measured this time.
- **The unit and its measurement (2026-11-15, zero games):**
  - *The unit.* A position the search tree reaches by a move or a null move, Rarog's head contract since D.6.8 (RAR-S124).
  - *The instrument.* Each engine's own source was built with one counter at its tree's make sites, printed with the engine's own node total before every `bestmove`. The sources were Rarog `v2.4.0`, `v2.5.0` and the head; Basilisk 8bfbbff and its tag v1.10.2, read-only by archive. The patch is `analysis/artifacts/rar-o11-2026-11-14/count_patch.py`.
    - Rarog counts at the null move, ProbCut's make, the move loop and quiescence's make.
    - Basilisk counts in `do_move` and `do_null_move`.
  - *Same tree.* On RAR-O10's first 500 positions at `go nodes 150000`, Hash 64, one thread, `ucinewgame` before each, every instrumented build reported its release binary's nodes and best move on every position: 0 mismatches in 2,500.
  - *Reported nodes over positions reached:*

    | Engine | Ratio | Median | Budget (150,000 positions) | Positions a search at the budget |
    |---|---|---|---|---|
    | Rarog 2.4.0 | **1.4167** | 1.4294 | 212,500 | 146,209 |
    | Rarog 2.5.0 | **1.2105** | 1.2090 | 181,600 | 145,167 |
    | Rarog's head | **1.0000** | 1.0000 | 150,000 | 145,048 |
    | Basilisk 1.10.2 | **1.3445** | 1.3543 | 201,700 | 146,086 |
    | Basilisk's head | **1.0367** | 1.0396 | 155,500 | 144,935 |

    Rarog's head at exactly 1 is the instrument's control: its count is that contract. At the budgets the engines reach 144,935 to 146,209 positions a search, within 0.9% of one another; the rest below 150,000 is searches ending early. In RAR-O11's equal-node reads Rarog 2.4.0 therefore searched about 105,900 positions against Basilisk 1.10.2's 111,600, and Rarog 2.5.0 about 123,900 against Basilisk's head's 144,700.
- **Instrument:**
  1. *Moves at equal work.* RAR-O10's 10,000 positions, each engine at its budget, `moves.py` as in RAR-O11, scored by RAR-O10's Stockfish 19 labels and RAR-O11's. The positions after a move that no earlier read labelled are labelled the same way (1,000,000 nodes). Rarog's head's moves are RAR-O11's 150,000-node run, already in the unit. Its pairs are as in RAR-O11's (a); each pair's ratio at RAR-O11's 100 ms over its ratio at equal work is that pair's speed and clock share in move loss.
  2. *Matches at equal work.* fastchess through `tools/sprt.ps1 -Mode fixed -NoAdjudication -Seed 20261115`, Hash 64, one thread, the UHO book, each side at its budget, 2,000 games each:
     - (m1) Rarog 2.4.0 against Basilisk 1.10.2, the start;
     - (m2) Rarog 2.5.0 against Basilisk's head, the search programmes' ends;
     - (m3) Rarog's head against Basilisk's head.
     The engine copies are RAR-O11's (`analysis/artifacts/rar-o11-2026-11-14/engines/`, SHA-256 there). A 2-game smoke of each ran.
- **Interacting mechanisms / confounds:** equal work is equal positions reached, not equal time and not equal decisions; it removes speed and the clock, and with them time management. The ratio was measured at about 150,000 reported nodes on 500 positions and moves a little with depth, inside the 0.9% the budgets show. Loss is not Elo (RAR-X03).
- **PRE-REGISTERED PREDICTION (frozen 2026-11-15, before any game; after RAR-O11's results and the count ratios were seen):** two bases disagree, and the predictions sit between them.
  - *Correcting RAR-O11's equal-node matches by the work gap*, at 145 Elo a doubling (RAR-M10): (m1) about −16, (m2) about −43.
  - *Pricing the speed gap the clock reads imply, at the same rate:* in positions a second, Basilisk 1.10.2 ran 25.6% faster than Rarog 2.4.0 and Basilisk's head 29% faster than Rarog 2.5.0 (RAR-O11's 100 ms runs over the ratios above). Removing that from the clock reads (d1 −11.5, d3 −76.3) puts (m1) near +36 and (m2) near −23.

  The two bases disagree because the clock carries time management as well as speed, and 145 a doubling is Rarog's own rate. Elo of the Rarog side, 80% bands:

  | Match | Prediction | 80% band |
  |---|---|---|
  | (m1) | **+5** | [−30, +40] |
  | (m2) | **−35** | [−60, −10] |
  | (m3) | **+85** | [+60, +110] |

  - Moves at equal work: Basilisk's head loses less than Rarog 2.5.0, interval excluding zero: 0.60. Basilisk 1.10.2 loses more than Rarog 2.4.0, interval excluding zero: 0.45. Basilisk's head loses more than Rarog's head, interval excluding zero: 0.80.
  - Most likely failure: (m1) and (m2) inside their bands but with intervals that cannot separate the per-position programme gap from zero.
- **Use of the reading, frozen:**
  - *Per position.* (m2) − (m1) is Basilisk's search programme's gain over Rarog's per position reached; RAR-O11's (d3) − (d1), −64.8 ± 16.1, is the same on the clock.
    - If (m2) − (m1) is within ±15 of zero: the two programmes gained alike per position, and Basilisk's larger gain on the clock was speed and clock use. D.7's question then moves to Rarog's kernel speed and time management (D.12, D.6.28) and away from tree shape.
    - If it is below −25, beyond its interval: Basilisk's tree gained more per position, and the tree question stands for D.7.2.3.
    - Between those: unresolved, said so.
  - *Speed and clock.* (d1) − (m1), (d3) − (m2) and RAR-O10's (c) − (m3) are the speed and clock share at the start, at the ends and at the heads. A share beyond its interval separates the engines by that much in that direction.
  - None opens or closes a leaf by itself; all go to the maintainer's decision on D.7's donor, with D.7.2.3.
- **Stop rules:** a failed step stops the reads; each match is independent; a host not idle within eight hours stops the step it waits for; no re-run on the same binaries, budgets and positions.
- **Command (the maintainer's, from `D:\code\rarog`):** `pwsh -NoProfile -File analysis/artifacts/rar-o12-2026-11-15/night.ps1`. It waits for an idle host before each step, logs each step's exit status in `night.log`, and ends with `RAR-O12 night run finished: exit 0`.
- **Cost:** the moves about 6 minutes, the new labels a few minutes, each match about an hour and ten minutes; about four hours in all.
- **Result:** not measured.
- **Artifacts / commits:** this registration; `analysis/artifacts/rar-o12-2026-11-15/` (ignored): `count_ratio.json` (SHA-256 2fc19eae96d5d9bd9e5068ef5348dbfd2fb9f1b0d8fd889ec499b5f3c1c19e76), `count_ratio_budgets.json` (e6d013646aec47eb1ca166d644c6d8a294aec7a1ba3b996d8dec452f2bb049b0) and their logs, the scripts, Rarog's head's reused moves (cc45ad23c045b7f5739693f746e6e17bedd71197617780c4be1641d125762221), the smoke; the instrumented sources and builds in `analysis/artifacts/rar-o11-2026-11-14/src/` and `count-build/`.
