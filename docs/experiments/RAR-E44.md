# RAR-E44 — C.6.7 follow-up: which pawns of the passed-set disagreement carry outcome information?

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-12, after the instrument was built and smoke-verified on 4,000 rows and before any loss was computed with the group columns; agent-run (zero games). The maintainer approved this read after RAR-E43's finding (2026-10-12).
- **Baseline SHA / candidate SHA / dirty-diff hash:** the current head, engine `38cbb02` (**14,197,472 / EBF 2.557**). Instrument `tools/diag/pawn_subterms.py` at `37f195c`. No candidate.
- **Binary / compiler / PGO identity:** none. Scores: `analysis/artifacts/c65-screens/scores-9587-cand.csv` (RAR-E42's and RAR-E43's file).
- **Research question:** RAR-E43 found the donor's passed set and Rarog's together gaining +0.141 ± 0.039 points in the endgame band (3.6 SE), neither alone better. Which of the pawns the two sets disagree on carry it?
- **The groups (fixed):** per side, every pawn the sets disagree on falls into exactly one group. `g_rear`: Rarog's rear pawn of a doubled passer (Rarog's set only). The donor's extra passers: `g_lever` (only levers in the way), `g_leverpush` (only pawns attacking its push square, outnumbered by its phalanx), and the blocked case (a single enemy pawn directly in front, fifth rank or higher, with a supported push), split by the donor's removal of blocked candidates without help into `g_blocked_kept` and `g_blocked_dropped`. Each group's value is the donor's `passed()` family over its pawns (mg, eg, tapered by the donor's phase). The identity "the donor's family = the family over Rarog's set − `g_rear` + `g_lever` + `g_leverpush` + `g_blocked_kept`" must hold on every row.
- **Hypothesis / proposed mechanism:** a pawn whose only stoppers are levers is about to become passed (or is traded), and its passed-pawn value predicts the outcome; Rarog calls it unpassed. So `g_lever` and `g_leverpush` carry RAR-E43's lead.
- **Competing hypotheses:** (1) *the rear doubled passer:* Rarog over-values it (two passers on one file are worth less than two), so letting the model price it separately is the gain. (2) *the blocked case:* the donor's supported-push passers are real. (3) *spread:* no single group reaches the line; the gain needs them together.
- **Interacting mechanisms / consumers:** every passer term consumes the passed set; C.5.2's opposite-bishop rule counts passers; the candidate-passer term overlaps the donor's extra passers; C.8's refit.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** "carries" as RAR-E43: a group's value beyond the family over Rarog's set gains 0.04 points at four standard errors in the endgame band or without queens.
  1. *Self-checks:* the donor's rows reproduce and the identity holds on every row (probability 0.95).
  2. `g_lever` carries (probability 0.35); `g_leverpush` (0.25); `g_rear` (0.3); `g_blocked_kept` (0.15); `g_blocked_dropped` (0.1).
  3. At least one group carries (0.45). All groups together reproduce most of RAR-E43's lead: at least +0.10 in the endgame band (0.8).
  - Expected Elo: none.
- **Falsification criteria:** void if `verify` or `verify-split` finds a mismatch, or if the shuffled null (the lever group permuted) reads outside ±0.03 in any cohort.
- **Cheapest prior falsifier:** the 4,000-row smoke (both checks exact).
- **Registered order (frozen):** into `analysis/artifacts/c68/`: `pawn_subterms.py collect --scores <the file above> --out . --workers 14`, `verify`, `verify-split`, then one `split`. Nothing is added after the numbers are seen.
- **Use, frozen:** the finding goes to the maintainer before anything is built (the C.6.7 rule). A group that carries is recommended as one definition change of Rarog's passed set (excluding the rear doubled passer, or admitting the donor's lever, lever-push or supported-push passers), fitted in C.8's refit and gated in C.8's cycle, with C.5.2's opposite-bishop cohort re-read beside it. With no group carrying, the passed set closes `NO_CHANGE` and C.6.7 closes `NO_CHANGE`; the retry trigger is C.8's first cycle re-reading RAR-E43's passed-set lead at the line.
- **Result:** pending.
- **Artifacts / commits:** `37f195c`; `analysis/artifacts/c68/` (ignored storage).
