# RAR-E37 — the pieces rise after C.4.3: is it the donor's Queens row (the weak-queen pin term)?

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-11, before any number with the Queens row read as its own family was computed; agent-run (zero games, no new engine calls).
- **Baseline SHA / candidate SHA / dirty-diff hash:** the instrument is `tools/diag/donor_residual.py` at `043c8f9` (branch `c5`), which reads the donor's Queens row as the family `queens`, keeps Knights, Bishops and Rooks in `pieces`, and still reports the four-row family as `pieces+queens`. The two score files are RAR-E32's, read again: the C.4.3 head (C.5.2's accepted head, `73c4c17`, 13,562,404 / EBF 2.546), `analysis/artifacts/c4-screens/scores-9587-head.csv`, and the C.4.3 candidate (`6c73d65`, 12,567,796 / EBF 2.535), `analysis/artifacts/c43-screens/scores-9587-cand.csv`: RAR-E23's donor rows on `hce-v4-tb` validation (194,444 rows) with each arm's played score substituted.
- **Binary / compiler / PGO identity:** none built; the files' SHA-256 are recorded with the result.
- **Research question:** RAR-E32's screen saw the donor's pieces family rise from +0.016 to +0.052 ± 0.014% of held-out loss (+0.065 → +0.132 in the middlegame band) when Rarog's x-ray slider term was removed. That term counted Rarog's rooks and bishops on a line to the enemy queen through at most pawns. Which donor row now carries that information?
- **Hypothesis / proposed mechanism:** the donor's Queens row is its weak-queen term: a penalty when an enemy rook or bishop stands on a line to the queen with exactly one piece between (a pin or a discovered attack waiting). The removed x-ray term was Rarog's approximation of the same geometry, so removing it moved the information into the Queens row's residual. **H1: the Queens row carries at least two thirds of the rise.**
- **Competing hypotheses:** (1) *the Rooks row:* the donor's rook-on-queen-file bonus (a rook on the queen's file through any pieces) overlaps the x-ray term for rooks, and the rise sits there. (2) *spread:* the rise is shared across the piece rows and no single row holds two thirds. (3) *not the x-ray at all:* the refit moved other coefficients (the candidate's dump differs from the head's on 96.4% of rows), and the rise reflects that, so the Queens row rises little.
- **Interacting mechanisms / consumers:** the new safe-square slider threat on the queen (C.4.3), which covers sliders bearing on squares around a lone queen but not pins through a blocker; C.7 (pieces, which lists "weak queen"); C.8's refit, which fixes the free slots.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** gains are `analyse --within`, the `all` cohort, in percent of Rarog's held-out squared error; the *rise* of a family is its gain on the candidate file minus its gain on the head file.
  1. *Reproduction:* `pieces+queens` reads +0.016 on the head and +0.052 on the candidate, a rise of +0.036, as RAR-E32 reported for `pieces` (probability 0.97; the tool's change is a regrouping).
  2. *Queens rise:* **+0.027**, 80% band [+0.012, +0.040]. **H1 (rise of `queens` ≥ two thirds of the rise of `pieces+queens`): probability 0.55.**
  3. *Pieces without the Queens row:* rise under +0.012 (probability 0.6).
  - Expected Elo: none; this ranks a question.
  - Most likely failure mode: competing (2), the rise spread between the Queens and Rooks rows.
- **Falsification criteria:** H1 is refuted if the Queens row's rise is below two thirds of the four-row family's rise. The read is void if step 1 does not reproduce RAR-E32's numbers (below).
- **Cheapest prior falsifier:** this read is it (minutes, zero games).
- **Registered order and stop rule (frozen):**
  1. Reproduce: `python tools/diag/donor_residual.py analyse --within --scores <file> --out <report>` on each file into `analysis/artifacts/coherence-2026-10-11/e37/`. Every model both reports share with RAR-E30's `analysis/artifacts/c4-screens/report-head.json` and RAR-E32's `analysis/artifacts/c43-screens/report-cand.json` must match to 1e-9 in every cohort, with the new `pieces+queens` matching the old `pieces`; otherwise the read is void and nothing below is used.
  2. Read `rarog+queens`, `rarog+pieces` and `rarog+pieces+queens` on both files: the `all` cohort decides; the middlegame band (`phase 32-95`) and the queens cohort are reported and decide nothing.
  3. **Decision**, on the `all` cohort's point estimates: H1 holds if rise(`queens`) ≥ ⅔ × rise(`pieces+queens`). The tool gives each gain's standard error, not the rise's; the rows are paired and the two gains positively correlated, so the reported combined interval, √(se_head² + se_cand²), is conservative.
  - Nothing is added after the numbers are seen.
- **Use, frozen (maintainer direction 2026-10-11):** if H1 holds, C.7's residual step opens with a weak-queen (pinnable queen) term as a named candidate, and C.8's refit restores Rarog's x-ray slider term beside the new queen threat as a free slot. If not, the record says where the rise sits and nothing is scheduled.
- **Full conditions / provenance:** as RAR-E30 and RAR-E32 (Stockfish `9587eeeb`'s per-term trace, two-fold held-out logistic models, K-free); the files are read as they stand.
- **Result (run 2026-10-11, once, after the registration commit `dc4703d`; `analysis/artifacts/coherence-2026-10-11/e37/read.txt`, by `compare.py` there):**
  - *Files:* head `2ea0f29b…` (the same bytes as `c43-screens/scores-9587-head.csv`), candidate `7963307b…`.
  - *Reproduction:* 480 values a report (every shared model, cohort and field) match RAR-E30's and RAR-E32's reports exactly (largest difference 0), `pieces+queens` included. Not void.
  - *Gains*, `all` cohort, head → candidate (rise; the conservative interval is √(se_head² + se_cand²)):

    | Family | Head | Candidate | Rise |
    |---|---:|---:|---:|
    | `queens` (the Queens row) | +0.0092 ± 0.0071 | +0.0230 ± 0.0105 | **+0.0138 ± 0.0127** |
    | `pieces` (Knights, Bishops, Rooks) | +0.0108 ± 0.0066 | +0.0366 ± 0.0121 | **+0.0258 ± 0.0138** |
    | `pieces+queens` (RAR-E32's `pieces`) | +0.0162 ± 0.0082 | +0.0515 ± 0.0145 | +0.0353 ± 0.0166 |

  - *Reported, deciding nothing:* in the middlegame band the rises are `queens` +0.018, `pieces` +0.054, both +0.067; in the queens cohort +0.018, +0.026, +0.039.
  - *Decision:* rise(`queens`) +0.0138 against ⅔ × +0.0353 = +0.0235: **H1 refuted**; the Queens row holds 0.39 of the rise.
- **Disposition:** observation. **The rise sits mostly in the Knights, Bishops and Rooks rows** (0.61 of it over all rows, 0.81 in the middlegame band), not in the weak-queen term; which of the three rows was not read (this registration did not split them). By the frozen use, nothing is scheduled: C.7's residual step opens without a named weak-queen candidate, and C.8's refit gets no x-ray slot from this read. The whole rise is about two conservative standard errors from zero, so it is a direction at the screen's resolution, not a measured loss. Competing (1) or (2) is the better account; (3) cannot be told from them here. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 3 above.
  - Observed: (1) hit (exact). (2) the Queens rise +0.0138 is inside the band [+0.012, +0.040] at its low end; H1 (probability 0.55) did not happen. (3) **miss**: the other piece rows rose +0.026, not under +0.012.
  - Which assumption failed: I mapped the removed term to the donor's closest-looking term by geometry (a slider on the queen's line through one blocker) and gave too little weight to the donor's other piece terms that read the same lines (the rook on the queen's file among them). The miss is in mechanism attribution, with the magnitude of the total right.
- **Conditional lesson:** a removed term's information can land in several donor rows; register the row-by-row split with the first read when the family has more than one plausible carrier.
- **Retry trigger or `closed`:** closed. C.7's own residual step reads the piece rows on its head; a split of N/B/R for this rise would be a new registration.
- **Artifacts / commits:** `043c8f9` (the instrument and its tests); `analysis/artifacts/coherence-2026-10-11/e37/` (ignored storage): both reports, the logs, `compare.py`, `read.txt`, `source.txt`.
