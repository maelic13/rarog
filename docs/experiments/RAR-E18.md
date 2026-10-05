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
- **Result:** not run at registration.
- **Disposition:** registered.
- **PREDICTION CALIBRATION (append after exposure):** pending.
- **Conditional lesson:** pending.
- **Retry trigger or `closed`:** pending.
- **Artifacts / commits:** pending.
