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
- **Result (run 2026-10-12, once, after the registration commit `e7cb6d8`; `analysis/artifacts/c68/`, every step exit 0):**
  - *Self-checks:* the donor's rows reproduce (0 mismatches); the identity holds on all 194,444 rows in mg and eg (0 mismatches). Rows holding each group: rear 6,153, lever 5,354, lever-push 7,656, blocked kept 3,004, blocked dropped 1,943. *Null:* −0.001 to +0.005 in every cohort. Not void.
  - *Each group beyond the family over Rarog's set* (points ± one standard error; `split.log`, `split.json`):

    | Group | Endgame band | No queens | All rows | Seven men or more |
    |---|---:|---:|---:|---:|
    | `g_leverpush` | +0.052 ± 0.024 | +0.059 ± 0.024 | +0.035 ± 0.010 | +0.037 ± 0.010 |
    | `g_rear` | +0.046 ± 0.028 | +0.045 ± 0.027 | +0.024 ± 0.010 | +0.020 ± 0.010 |
    | `g_lever` | +0.035 ± 0.018 | +0.020 ± 0.017 | +0.008 ± 0.005 | +0.004 ± 0.006 |
    | `g_blocked_kept` | +0.012 ± 0.011 | −0.022 ± 0.011 | −0.003 ± 0.005 | +0.007 ± 0.005 |
    | `g_blocked_dropped` | +0.000 ± 0.001 | +0.001 ± 0.003 | −0.002 ± 0.003 | +0.001 ± 0.002 |
    | all groups together | +0.143 ± 0.042 | +0.103 ± 0.041 | +0.063 ± 0.016 | +0.067 ± 0.016 |

- **Disposition:** observation; **no group carries** (the largest, the donor's lever-push passers, at 2.2 and 2.5 standard errors in the rule's cohorts). The groups together reproduce RAR-E43's lead (+0.143 against +0.141) but it is spread over three of them: the donor's lever-push passers, Rarog's rear doubled passers and the donor's lever passers; the blocked case and the donor's removal of it carry nothing. **By the frozen use the passed set closes `NO_CHANGE`, and C.6.7 closes `NO_CHANGE`.** Retry trigger: C.8's first cycle re-reads RAR-E43's passed-set lead (the two sets together) at the line; if it carries then, the change to build is the donor's set as a whole (rear doubled passers out, lever and lever-push passers in) rather than one group, since no group stands alone. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 3 above.
  - Observed: (1) hit. (2) every "carries" prediction missed (lever, lever-push, rear, blocked kept, blocked dropped); the two blocked groups, given low probabilities, read nil as their probabilities implied. (3) **miss** on the first clause (no group carries); hit on the second (+0.143, at least +0.10).
  - Which assumption failed: I expected one mechanism (the lever passer about to break through) behind the lead; it is three small ones of similar size. The miss is in mechanism concentration.
- **Conditional lesson:** a lead that only appears when two definitions are combined is likely the sum of several small disagreements; split it before planning a single change.
- **Retry trigger or `closed`:** closed `NO_CHANGE`; retry as above.
- **Artifacts / commits:** `37f195c`; `analysis/artifacts/c68/` (ignored storage): `pawn_subterms.csv`, `verify.json`, `verify-split.json`, `split.json`, `split.log`, `status.txt`, `source.txt`.
