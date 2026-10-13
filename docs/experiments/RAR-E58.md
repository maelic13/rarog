# RAR-E58 — C.7.2 attribution: the Material row and the phase slope among decided scores

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-13, before the full score was dumped and before any model below was fitted on the corpus; agent-run (zero games, one `--dump-scores --full` pass).
- **Kind:** research: attribution of RAR-E57's two signals. No engine change.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at `2e22433`; engine and tuner sources equal `ab2a6ab`'s, **14,197,472 / EBF 2.557**. The played score is RAR-E57's `analysis/artifacts/c71/scores-9587-head.csv` (`c64d5de1…`). The full score comes from the same tuner (`rarog-texel.exe` `0f6a4c8b…`, its identity asserted) run with `--dump-scores --full`, which holds the lazy gate out of reach, and is substituted into the same donor rows. No candidate.
- **What RAR-E57 found:** at |rarog| > 500 the donor's Material row (+0.333 ± 0.056%) and the phase slope `r·(phase/128 − 0.5)` (+1.316 ± 0.201%) passed the line, above that cohort's magnitude re-pricing (+0.058), which is weakest in a cohort selected on the score it rescales. In whole-file terms they are about 0.014% and 0.056% of the loss.
- **What the code says** (`src/eval/mod.rs`): above a tapered material + PST + pawn + passer margin of **423 cp** (`LAZY_MARGIN`, the `LazyMargin` search coordinate), the played evaluation skips piece activity (mobility, threats, king safety, hanging and the small terms) and imbalance, keeping the mop-up, tempo and the endgame scale. So nearly every row at |rarog| > 500 is a lazy score.
- **Research question:** do RAR-E57's two signals survive a comparator that can still bend among decided scores? If they do, are they the lazy gate's (gone on the full score), a phase calibration of large scores, or material knowledge, and is any material knowledge in the donor's piece values or in its placement?
- **Hypotheses (competing, none favoured by the design):** (1) *re-pricing:* a flexible re-pricing of Rarog's own score absorbs both; C.7 has nothing. (2) *lazy gate:* the lazy score lacks the activity terms that tell a material lead that converts from one that does not, and the signal is gone on the full score. That is a search-side coordinate (`LazyMargin`, tuned in margin blocks and in C.10), not C.7's. (3) *phase calibration:* Rarog's large scores mean a different winning chance at different phases, on the full score too; the donor's Material row only echoes that through its endgame piece values exceeding its middlegame ones, and adds nothing once the slope is in the model. (4) *material knowledge:* the Material row adds information beyond the comparator and the slope, either in the piece values (`material_count`) or in placement (`material_placement`).
- **Instrument:** `analysis/artifacts/c72/attrib.py`, using `donor_residual.py`'s two-fold held-out logistic fit:
  - *Nested gain:* each gain is a model `C + D` over `C`, in percent of `C`'s held-out squared error, with a per-row standard error.
  - *Comparator:* `C` is a hinge spline of Rarog's score: `r` plus hinges at the 25/50/75% quantiles of |r| inside the cohort, on each side.
  - *Material split:* the Material row is split exactly into its count-linear part and the remainder. The count-linear part is a least-squares fit of the row on the piece-count differences, mg and eg apart, which recovers the donor's piece values plus each piece's mean placement. The split is asserted to sum back to the row.
  - *Cohorts:* defined by the played score so both versions read the same rows. Reported beside the gains: the slope's standardised weight (positive means Rarog's large scores count for more toward the middlegame), the lazy share, and a calibration table of outcome agreement by phase band and score bin.
  - *Wire, else void:* RAR-E57's two signals reproduce exactly on the played score with Rarog's score alone as the base. Donor columns and labels must be equal between the two files, and the tuner must be RAR-E57's.
- **Registered order (frozen):** `analysis/artifacts/c72/run_e58.sh`, once, every exit status to `status.txt`: tree check; tuner build and identity; `--dump-scores --full`; `substitute.py`; `attrib.py` (exit 1 if the reproduction fails). Nothing is added after the numbers are seen.
- **Rule (frozen), on the cohort |played| > 500;** a gain is *at the line* when it is at least 0.10% and at least four standard errors:
  - **A (comparator):** on the played score, `phase_slope|C` and `material_row|C`. A direction under the line here is re-pricing.
  - **B (lazy gate):** a direction at the line in A is read again on the full score. If it is under the line there, it belongs to the lazy gate.
  - **C (knowledge):** for the Material row at the line in both A and B, `material_row|C+slope` on the played score. At the line, it is material knowledge beyond the phase calibration, and `material_count|C+slope` and `material_placement|C+slope` say where. Under the line, it is the slope's echo.
- **Use, frozen:**
  - Nothing at the line in A: **C.7 closes `NO_CHANGE`** (re-pricing, which C.8's refit and its K handle).
  - Every survivor of A under the line in B: **C.7 closes `NO_CHANGE`**, and the finding is recorded for the `LazyMargin` coordinate (margin blocks, C.10) and for C.8.
  - The phase slope surviving A and B, with no material knowledge at C: C.7 continues to **C.7.3, a research card on phase interpolation**. It decides with the maintainer whether a candidate is worth building at this size.
  - The Material row at the line in C: C.7.3's card covers material knowledge in the part named.
  - Nothing is built from this read.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. Wire: the reproduction is exact (0.97); the count fit's R² is at least 0.97 for mg and eg (0.8); its fitted eg values exceed its mg values for knight, bishop, rook and queen (0.85).
  2. Lazy share at |played| > 500: at least 0.9 (0.75).
  3. A: the phase slope is at the line over the spline comparator (0.55); the Material row is (0.45).
  4. B: of A's survivors, the phase slope falls under the line on the full score (0.5); the Material row does (0.4).
  5. C: the Material row is at the line beyond the slope (0.3); if so, in the count-linear part rather than placement (0.6).
  6. The slope's weight at |played| > 500 on the played score is positive: large endgame scores are less decisive than large middlegame ones (0.6).
  7. **C.7 closes `NO_CHANGE` by A or B (0.55).**
  - Expected Elo: none. Static loss ranks questions and accepts nothing.
- **Cheapest prior falsifier:** this read.
- **Artifacts:** `analysis/artifacts/c72/` (ignored storage): `run_e58.sh`, `attrib.py`, and the outputs they name.
