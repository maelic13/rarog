# RAR-E39 — C.6.1 residual step: the donor's passed and pawns families on the current head, by cohort

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-11, before the cohorts below were looked at; agent-run (zero games, no engine calls).
- **Baseline SHA / candidate SHA / dirty-diff hash:** the current head's evaluation, `6c73d65`'s (12,567,796 / EBF 2.535; nothing since changed the played evaluation: the later engine commits are texel-only or comments, and the bench reproduces). No candidate.
- **Binary / compiler / PGO identity:** none. The report is RAR-E37's `analysis/artifacts/coherence-2026-10-11/e37/report-cand-q.json`: `donor_residual.py analyse --within` (`043c8f9`) over `analysis/artifacts/c43-screens/scores-9587-cand.csv` (`7963307b…`), which is RAR-E23's donor rows on `hce-v4-tb` validation with this evaluation's played score substituted. The same report reproduced RAR-E32's to the last digit (RAR-E37, step 1).
- **Research question:** do the donor's passed and pawns families carry outcome information beyond Rarog's current score in any registered cohort (PLAN C.6: the cluster opens with its own residual step and closes `NO_CHANGE` if it finds none)?
- **Exposure, stated before the read:** three cohorts of this report were read for the order of work (B8): passed over all rows +0.091 ± 0.013, at seven men or more +0.078 ± 0.014, at six or fewer +0.348 ± 0.095; pawns +0.028 ± 0.009, +0.029 ± 0.008 and +0.21 ± 0.12. None reaches the line. **The deciding cohorts are the seven not yet read:** phase ≥ 96, phase 32–95, phase < 32, queens, no queens, |score| ≤ 500, |score| > 500.
- **Hypothesis / proposed mechanism:** H1 of the card (`analysis/c6_pawns_passers_2026-10-11.md`): the donor's passed family carries endgame conditioning Rarog lacks (king proximity weighted by rank, path and block-square safety), so it has signal in the endgame band or without queens. H2: the pawns family has signal nowhere.
- **Competing hypotheses:** (1) *magnitude:* the passed residual is the size of Rarog's passer bonus, not its conditioning, so it concentrates at |score| > 500 and not in a phase band. (2) *nothing left:* the refits since RAR-E17 absorbed the passed direction and no cohort reaches the line. (3) *pawns has signal* in the opening or middlegame band (structure matters most with pieces on).
- **Interacting mechanisms / consumers:** C.6.3 onwards (an attribution and a unit, if any); C.5.2's opposite-bishop rule (it counts passers); the lazy gate (the per-evaluation passer terms run below it).
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Passed:* signal in the endgame band (phase < 32): +0.30, 80% band [+0.15, +0.50], at four standard errors or more with probability 0.6. Without queens: +0.17, band [+0.10, +0.28], signal with probability 0.55. **Passed has signal in at least one deciding cohort: probability 0.7.**
  2. *Pawns:* no deciding cohort reaches the line (probability 0.65); the largest is in the endgame band.
  3. *Magnitude check:* passed at |score| > 500 is larger than at ≤ 500 (probability 0.6).
  - Expected Elo: none.
- **Falsification criteria:** H1 is refuted if no deciding cohort shows passed at 0.10% or more at four standard errors; H2 if a deciding cohort shows pawns there.
- **Cheapest prior falsifier:** this read.
- **Registered order and rule (frozen):** read `rarog+passed` and `rarog+pawns` (gain and standard error) in the seven deciding cohorts of the report, once, and print all ten cohorts. A family **has signal** at a gain of 0.10% or more at four standard errors in a deciding cohort.
- **Use, frozen:** a family with signal continues into a sub-term attribution (C.6.3, its own registration: the donor's components rebuilt from the FEN and verified against the printed rows before any loss, as RAR-E31). A family without signal closes `NO_CHANGE` inside C.6. Both without: C.6's residual part closes `NO_CHANGE`, and C.6 continues only as far as C.6.2 (RAR-E40) takes the unstoppable-passer test.
- **Result (read 2026-10-11, once, after the registration commit `254c9b4`; `analysis/artifacts/c6/e39_read.txt`):** gain in percent of Rarog's held-out squared error, ± one standard error; deciding cohorts marked *.

  | Cohort | Rows | Passed | Pawns |
  |---|---:|---:|---:|
  | all | 194,444 | +0.091 ± 0.013 | +0.028 ± 0.009 |
  | phase ≥ 96 * | 42,953 | +0.048 ± 0.019 | +0.011 ± 0.014 |
  | phase 32–95 * | 50,906 | −0.003 ± 0.003 | +0.044 ± 0.019 |
  | **phase < 32** * | 100,585 | **+0.204 ± 0.031 (6.5 SE)** | **+0.121 ± 0.028 (4.3 SE)** |
  | queens * | 107,638 | +0.001 ± 0.004 | +0.011 ± 0.006 |
  | **no queens** * | 86,806 | **+0.196 ± 0.035 (5.7 SE)** | **+0.141 ± 0.031 (4.6 SE)** |
  | men ≤ 6 | 30,570 | +0.348 ± 0.095 | +0.210 ± 0.122 |
  | men ≥ 7 | 163,874 | +0.078 ± 0.014 | +0.029 ± 0.008 |
  | |score| ≤ 500 * | 138,507 | +0.095 ± 0.016 | +0.038 ± 0.012 |
  | |score| > 500 * | 55,937 | −0.013 ± 0.011 | +0.014 ± 0.024 |

- **Disposition:** observation. **Both families have signal** in the endgame band and without queens; **both continue into a sub-term attribution** (C.6.3, its own registration, the donor's components rebuilt from the FEN and verified against the printed rows before any loss). H1 holds; H2 is refuted. The residual is conditioning, not magnitude: both families are nil above |score| 500 and carry their gain inside ±500 (competing (1) refuted), and both are nil with queens on. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 3 above.
  - Observed: (1) hit: endgame band +0.204 (band [+0.15, +0.50]), without queens +0.196 (band [+0.10, +0.28]), both signal. (2) **miss**: pawns has signal in two cohorts, and its largest is without queens (+0.141), not the endgame band (+0.121). (3) **miss** in sign: passed above |score| 500 reads −0.013, below it +0.095.
  - Which assumption failed: I took pawn structure for a middlegame matter already rich in Rarog's terms; its residual sits in queenless and endgame positions, where structure decides which passers appear and which pawns fall. The miss is in mechanism; magnitude and confidence on passed were right.
- **Conditional lesson:** pawn structure's value is read where pawns decide the game, not where the terms are most numerous.
- **Retry trigger or `closed`:** closed as a read; C.6.3 follows.
- **Artifacts / commits:** the report above; `analysis/artifacts/c6/e39_read.txt` (ignored storage).
