# RAR-E18 — C.0 follow-up cuts of RAR-E17's rows: where the donor's total-level information sits

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-05, after RAR-E17's aggregate reading and before any of the cuts below was computed; agent-run (zero games, no new evaluation calls).
- **Baseline SHA / candidate SHA / dirty-diff hash:** `dev` at the tool commit that adds `analyse --within` (cited under artifacts); engine source `ee02ed1`, unchanged.
- **Binary / compiler / PGO identity:** none run; the input is RAR-E17's `tools/results/donor-residual-20261005/scores.csv` (`F684F571…60DB`).
- **Research question:** RAR-E17 read `rarog+stockfish` at +3.59% and `rarog+all_families` at +1.22%, and Stockfish's total alone ahead of Rarog's by 2.17%. Is the part of the donor's information that its term families do not carry (a) magnitude calibration of decided positions, which a monotone rescaling of Rarog's own score would recover, (b) exact knowledge of positions with six men or fewer, where the labels are Syzygy-corrected, or (c) scaling and winnability above six men, which is what C.5.2 would build?
- **Hypothesis / proposed mechanism:** (c) is a real share: above six men and in undecided positions the donor's total still adds clearly more than its families do, because Stockfish's scale factor (opposite bishops, one-flank rook endings, pawn count) and its winnability term act on the total and are not printed per family.
- **Competing hypotheses:** (a) *magnitude:* the excess is Stockfish scoring won positions far higher than Rarog does (KR-K at 55 pawns against 4 to 7), and `rarog+magnitude` recovers most of it; (b) *tablebase rows:* the excess is concentrated at six men or fewer and nearly vanishes above; (d) *fit artefact:* RAR-E17 fitted one weight vector over all phases, and within-phase fits bring the families close to the total.
- **Interacting mechanisms / consumers:** C.5's scope and its place in the order of work; the reading of RAR-O05.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** gains in percent of Rarog's held-out mean squared error, every model refitted inside its cohort.
  1. `men>=7`: `rarog+stockfish` gains at least **+2.0%** (probability 0.75) and `stockfish` alone still beats `rarog` alone (probability 0.7).
  2. `men<=6`: `rarog+stockfish` gains at least **+8%** (probability 0.6).
  3. `|rarog|<=500`: `rarog+stockfish` gains at least **+1.5%** (probability 0.7).
  4. `all`: `rarog+magnitude` gains at most **+0.5%**, under a quarter of `rarog+stockfish` (probability 0.6).
  5. `phase 32-95`: `rarog+king` gains at least **+1.2%** (probability 0.8).
  6. `phase<32`: `rarog+initiative` gains at least **+0.6%** (probability 0.75), and `rarog+all_families` stays below `rarog+stockfish` by at least two points (probability 0.7).
  - Expected Elo sign/range, if defensible: none; static loss only.
  - Most likely failure mode: prediction 4 fails, that is, most of the donor's edge is magnitude calibration; then the "total-level gap" is not evidence for scaling knowledge.
- **Falsification criteria:** hypothesis (c) is refuted if, in `men>=7`, `rarog+stockfish` gains under 1.0%, or if `rarog+magnitude` recovers more than half of `rarog+stockfish`'s gain there.
- **Cheapest prior falsifier:** RAR-E17 itself; it could not separate these because its cohorts shared one fit.
- **Registered gate and stop rule:** no gate; one run of `analyse --within`, no further cuts of these rows without a new registration. Use, frozen: if (c) survives, C.5.1 and C.5.2 are placed directly after C.3 in the order of work, as RAR-E17's initiative reading already indicates; if (a) or (b) explains the excess, C.5 keeps PLAN's place and its research card starts from the magnitude or tablebase finding. Nothing is accepted and no Elo is estimated.
- **Full conditions / provenance:** `python tools/diag/donor_residual.py analyse --within --scores tools/results/donor-residual-20261005/scores.csv --out tools/results/donor-residual-20261005/report-within.json`. Cohorts: all, the three phase bands, queens on or off, six men or fewer against seven or more (kings included), and Rarog's clipped score within or beyond 500 cp. `rarog+magnitude` adds `r·|r|/1000` to Rarog's score. Disclosed: these rows were seen in aggregate in RAR-E17; no cohort below was computed before this registration except RAR-E17's own six, which were read from a single global fit.
- **Result (run 2026-10-05, once, after the registration commit `e2c6ebd`):** null control within ±0.025% in every cohort. Gains against `rarog` inside each cohort, percent ± SE:

  | Cohort (rows) | `stockfish` alone | `rarog+stockfish` | `rarog+all_families` | `rarog+king` | `rarog+initiative` | `rarog+magnitude` |
  |---|---:|---:|---:|---:|---:|---:|
  | all (194,444) | +2.17 ± 0.16 | +3.59 ± 0.09 | +1.22 ± 0.05 | +0.66 ± 0.04 | +0.12 ± 0.01 | **+0.97 ± 0.05** |
  | phase ≥ 96 (43,017) | +1.89 ± 0.19 | +2.19 ± 0.14 | +1.95 ± 0.13 | +0.71 ± 0.08 | +0.03 ± 0.02 | +0.07 ± 0.02 |
  | phase 32–95 (50,797) | +2.55 ± 0.25 | +3.25 ± 0.17 | +2.24 ± 0.13 | **+1.58 ± 0.12** | +0.03 ± 0.02 | +0.12 ± 0.03 |
  | phase < 32 (100,630) | +5.77 ± 0.30 | +7.33 ± 0.20 | +1.36 ± 0.09 | +0.09 ± 0.02 | **+0.91 ± 0.07** | +2.75 ± 0.12 |
  | men ≤ 6 (33,648) | +29.46 ± 0.95 | **+29.69 ± 0.84** | +7.70 ± 0.46 | +0.01 ± 0.02 | +6.84 ± 0.43 | +15.12 ± 0.61 |
  | men ≥ 7 (160,796) | +1.43 ± 0.15 | **+2.73 ± 0.09** | +1.62 ± 0.06 | +0.76 ± 0.05 | +0.02 ± 0.01 | +0.46 ± 0.03 |
  | \|rarog\| ≤ 500 (139,678) | +2.19 ± 0.15 | **+3.24 ± 0.09** | +2.00 ± 0.08 | +0.80 ± 0.05 | +0.09 ± 0.01 | +0.24 ± 0.03 |
  | \|rarog\| > 500 (54,766) | +0.56 ± 0.67 | +3.39 ± 0.25 | +1.16 ± 0.21 | +0.09 ± 0.04 | +0.07 ± 0.04 | +0.32 ± 0.05 |

  Other families inside their best cohort: space +0.15 ± 0.04 at phase ≥ 96; mobility +0.11 ± 0.03 at phase 32–95; threats +0.20 ± 0.04 and passed +0.16 ± 0.03 without queens; material +2.41 ± 0.32 at six men or fewer. In squared-error units the 3.6% splits into about 2.6 points from positions of seven men or more and 1.2 from six or fewer (which are 17% of the rows and 4% of the loss). Full table: `tools/results/donor-residual-20261005/report-within.json` (`8a7ddb94…9c6b`).
- **Disposition:** observation. Hypothesis (c) survives its falsifier: at seven men or more the donor's total adds 2.73%, a magnitude recalibration of Rarog's own score recovers 0.46 of it (17%), and the term families 1.62. Hypothesis (d) is right for the opening band (families +1.95% of the total's +2.19%) and wrong for the endgame band. Hypotheses (a) and (b) each explain a share: at six men or fewer the donor is 29.7% better and half of that is magnitude. By the frozen use, C.5.1 and C.5.2 follow C.3 in the order of work.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 6 above.
  - Observed: (1) hit, +2.73% and Stockfish alone ahead by 1.43%. (2) hit, far beyond the line (+29.7% against "at least 8%"). (3) hit, +3.24%. (4) **miss**: `rarog+magnitude` gains +0.97%, above the 0.5% line and 27% of the total's gain rather than under a quarter. (5) hit, +1.58%. (6) hit, +0.91% and a six-point gap.
  - Mechanism supported? Partly. The excess above the families at seven men or more (about 1.1 points) is real and is not magnitude, but the winnability term itself adds almost nothing there (+0.02%): its signal is a six-men-or-fewer effect (+6.84%). What carries the excess above six men is not identified by this instrument; the scale factor is the candidate, unmeasured.
  - Missed interaction or instrument limit: I under-weighted how much of a Texel loss on tablebase-corrected rows is the size of a winning score. Rarog scores a won KR-K at 4 to 7 pawns; the label is 1.0.
  - Confidence: about right on five of six.
- **Conditional lesson:** three different things sit behind "Stockfish's evaluation knows more in endgames": exact low-material knowledge that matches tablebase labels, the magnitude it gives a win, and scaling above six men. Only the last is what a generic winnability cluster builds, and its size here is about one point of loss, not seven.
- **Retry trigger or `closed`:** C.5.1 owns the next cut (by material signature); closed here.
- **Artifacts / commits:** tool change `3432fc7`; `tools/results/donor-residual-20261005/report-within.json` and the log `tools/results/donor-residual-20261005.within.log` (ignored storage). Analysis: `analysis/eval_programme_2026-10-05.md`, section 5.
- **Superseded in part, 2026-10-06 (RAR-E22):** the per-family trace here was Stockfish 11's while the total was `9587eeeb`'s. With `9587eeeb`'s own families, `rarog+all_families` reads +2.73% at seven men or more, equal to `rarog+stockfish`: the 1.1-point excess that hypothesis (c) rested on was the version mixture, and the scale factor measured directly as a direction adds +0.09% there (RAR-E21). The six-men-or-fewer findings (exact knowledge and magnitude) stand. The numbers above are preserved as measured.
