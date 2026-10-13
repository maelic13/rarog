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
- **Result (run 2026-10-13 14:24, once, after the registration commit `5b368af`; `analysis/artifacts/c72/`, every step exit 0, read from `status.txt`; tree clean; tuner identity asserted):**
  - *Wire:* RAR-E57's two signals reproduce exactly (+1.3156 and +0.3331, difference 0). The material split sums back to the row (largest error 4.6e-13). Donor columns and labels are equal between the files. **The count fit's R² is 0.667 (mg) and 0.676 (eg)**, not the predicted 0.97 or more. Its fitted eg values exceed the mg values for every piece.
  - *Lazy share:* 0.792 of the |played| > 500 rows (0.34 at phase ≥ 96, 0.60 at 32–95, 0.84 under 32); 0.027 at |played| ≤ 500.
  - *Gains at |played| > 500* (55,544 rows; ± one standard error; "line" is 0.10% at 4 SE):

    | Direction | Played score | Full score |
    |---|---:|---:|
    | `phase_slope\|C` | **+1.318 ± 0.203 (6.5 SE), line** | **+0.908 ± 0.216 (4.2 SE), line** |
    | `material_row\|C` | **+0.322 ± 0.058 (5.5 SE), line** | −0.005 ± 0.044 |
    | `material_count\|C` | +1.151 ± 0.149 (7.7 SE) | +0.063 ± 0.068 |
    | `material_placement\|C` | +0.009 ± 0.011 | +0.158 ± 0.060 (2.6 SE) |
    | `material_row\|C+slope` | +0.219 ± 0.084 (2.6 SE) | +0.047 ± 0.040 |
    | `phase_slope\|C+material_row` | +1.216 ± 0.215 (5.6 SE) | +0.960 ± 0.214 (4.5 SE) |
    | slope weight (standardised) | −1.108 | −1.263 |

  - *Where the slope lives (reported):*
    - On the 11,565 decided rows the gate does not touch, the played and full scores are the same, and the slope reads +0.809 ± 0.195 (4.2 SE). On the lazy rows it reads +1.022 ± 0.277 (played) and +1.182 ± 0.374 (full).
    - Inside one phase band it is small: +0.179 ± 0.078 under 32 and +0.221 ± 0.128 at 32–95. The signal is between bands.
    - Over all rows with the comparator it is nil: +0.027 ± 0.017 played, +0.008 ± 0.017 full.
  - *Calibration among decided played scores* (how often the outcome agrees with the score's sign; draws count half):

    | Score | Phase ≥ 96 | Phase 32–95 | Phase < 32 |
    |---|---:|---:|---:|
    | 500–800 cp | 0.907 (1,359 rows; 5.4% draws) | 0.940 (6,067; 5.6%) | 0.971 (22,225; 4.7%) |
    | 800–1,200 cp | 0.963 (175) | 0.986 (2,019) | 0.996 (13,367) |
    | over 1,200 cp | 1.000 (9) | 0.998 (591) | 1.000 (9,732) |

    In the opening band, a +500 to +800 score loses about 6.6% of games outright. In the endgame band, about 0.6%.
  - *Post hoc, deciding nothing* (`post_hoc_zero_rows.py`, run after the result to explain the count fit's miss):
    - The donor prints every trace row zero for its specialised endgames. These are 11,291 rows (5.8%), and **10,250 of the 55,544 decided rows (18.5%)**. Their outcomes agree with Rarog's sign 0.998 of the time, against 0.973 for the other decided rows.
    - Without them the count fit's R² is 0.986 and 0.993, and the fitted values match the donor's piece values in pawns (knight 3.78 mg, 4.14 eg).
    - So on those rows the Material row reads zero, which flags "a specialised ending". The played-score Material signal may be partly that flag, and `material_count` and `material_placement` are not a clean values/placement split there.
- **Disposition by the frozen rule:**
  - **A:** both directions are at the line over the spline comparator.
  - **B:** the **phase slope survives** on the full score. The **Material row falls under the line**, so it belongs to the lazy gate: the played score's lazy rows miss information the full score has. Post hoc, part of that is the donor's specialised-endgame flag. Neither is material knowledge Rarog lacks.
  - **C:** does not apply (the Material row failed B; beyond the slope it is under the line anyway).
  - By the frozen use, **C.7 continues to C.7.3, a research card on phase interpolation**, decided with the maintainer; nothing is built from this read. The Material-row finding is recorded for `LazyMargin` (PLAN C.10) and C.8. Imbalance and the piece rows stay closed (RAR-E57).
  - What the slope says: Rarog's large scores are **over-confident toward the opening and under-confident toward the endgame** relative to each other, at about 0.05% of the whole file's loss and nil over all rows.
  - The decided opening and middlegame rows are mostly not lazy (66% and 40% are scored with the activity terms), so the over-confident large scores there are largely positional scores, not material ones.
  - Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 7 above.
  - Observed: (1) reproduction hit; R² **miss** (0.667 and 0.676; the donor's specialised endgames print zero rows, found post hoc); eg above mg for every piece, hit. (2) **miss** (0.792). (3) both hit. (4) phase slope falls on the full score, **miss** (it stays at 4.2 SE); the Material row falls, hit. (5) not reached; the Material row beyond the slope is under the line, consistent with 0.7. (6) **miss in sign**: the weight is negative, so large endgame scores are the more decisive. (7) **miss**: C.7 continues.
  - Which assumption failed:
    - I expected decided endgames to hide fortresses and wrong-bishop draws and so to be over-confident. In this corpus the opposite holds: a large endgame score is close to certain, and a large opening or middlegame score loses one game in fifteen. The miss is in sign, from the wrong mechanism.
    - I also took the lazy share to be near total among decided rows. A large score with pieces on is mostly built by the activity terms the gate protects, not by material.
    - I did not check that every donor row carries a trace before defining a split on it.
- **Conditional lesson:** before decomposing a donor row, count the rows on which the donor printed nothing; a flag hidden in zeros reads as signal.
- **Retry trigger or `closed`:** closed as a read; C.7.3's card follows. Its first registered check is the slope without the donor's specialised-endgame rows and split by whether Rarog's large score is material or positional.
- **Artifacts:** `analysis/artifacts/c72/` (ignored storage): `run_e58.sh`, `run.log`, `status.txt`, `source.txt`, `tree-status.txt`, `build-tuner.log`, `head_full.csv` (`0d4a4c94…`), `dump.log`, `substitute.log`, `scores-9587-full.csv` (`f6ae59d8…`), `attrib.py` → `attrib.json` (`e49c9664…`) and `attrib.log` (`304f344b…`), `post_hoc_zero_rows.py` → `post_hoc_zero_rows.log` (`49da2571…`).
