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
- **Result:** pending.
- **Artifacts / commits:** `043c8f9` (the instrument and its tests); `analysis/artifacts/coherence-2026-10-11/e37/` (ignored storage).
