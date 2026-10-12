# RAR-E45 — C.5.2.2 the opposite-bishop boundary: one two-regressor rule (passers and free passers) at every men count, against C.5.2's split

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-12, after the instrument was built and tested on planted rules and before any dump of this read was written or any loss computed; agent-run (zero games).
- **Baseline SHA / candidate SHA / dirty-diff hash:** the current head, engine `38cbb02` (**14,197,472 / EBF 2.557**). Instrument `tools/diag/ocb_free_passers.py` (`two`, `drawn`) at `d699973`, `tools/diag/ocb_scale_screen.py` (`verify`). No candidate binary: the rule is applied offline.
- **Binary / compiler / PGO identity:** `tools/texel-tuner` built at the head, SHA-256 recorded with the result.
- **Research question:** C.5.2's opposite-bishop rule scales pure endings of seven men or more by `s = 1 + 2·pawns + 10·passers` and keeps the old `32 + 4·pawns + 4·passers` at six or fewer, so one exchange can roughly double the score (GUIDE's hold; RAR-E29 measured the incentive as small). RAR-E33 tested free passers *instead of* the passer count and lost above the boundary. Does one rule with both counts, `s = clamp(a + b·pawns + c·passers + d·free, 0, 48)` at every men count, match the split above the boundary and beat it below, without the wins-under-claim that sank RAR-E26's count rule in play (RAR-E27)?
- **Deviation from PLAN's text, stated before any number:** PLAN says to fit on RAR-E33's dumps. Those were written by the evaluation before C.6's refit, which changed passer scoring (king proximity, the square rule) in exactly these endings. The read uses **fresh dumps by the current head**: `rarog-texel --dump-scores <csv> <out> --scale` over `hce-v4-tb` train (3,500,000 rows) and validation (194,444) and over RAR-E26's 2,973 Syzygy-labelled opposite-bishop positions (`analysis/artifacts/c51-ocb/drawn_positions.csv`). K is **1.63034**, RAR-E42's fit.
- **Hypothesis / proposed mechanism:** above the boundary the passer count carries what the split's fitted rule carries (RAR-E33: the count refit is within ±0.24% there); below it, free passers say which passers the defender cannot stop (RAR-E33: won 0.44 against 0.11 in KBP-KB), so a rule with both counts keeps the wins scored high while scaling the draws, where the count alone flattened both.
- **Competing hypotheses:** (1) *count suffices:* the fit puts `d` at or near 0 and the rule is RAR-E26's count rule again, refuted in play. (2) *static only:* the rule beats the split statically below the boundary (as the count rule did, +56%) but still scores won six-man endings low, which RAR-E27 showed costs conversion. (3) *above the boundary:* the shared constants cost the seven-men region more than two standard errors.
- **Interacting mechanisms / consumers:** as RAR-E26 to RAR-E33 (the evaluation fitted with the rule fixed; the search's rule-50 damping; the eval cache and TT store the scaled score); the boundary incentive itself; C.5.6 (KBPPKB) and C.5.7 (KBPKB); C.8's refit.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Verify:* the offline rule reproduces the engine's factor on every row of the three fresh dumps (probability 0.95).
  2. *Fit:* `c > 0` (0.9) and `d > 0` (0.75).
  3. *Against the split:* above the boundary within two standard errors (0.7); below it better at three standard errors or more (0.9).
  4. *Against the count form:* not worse below the boundary (0.6); better there at two standard errors (0.3).
  5. *The static veto* (below): passed in KBPP-KB (0.4).
  6. Eligible to build (rule and veto): 0.35.
  - Expected Elo: none.
- **Falsification criteria:** void if `verify` fails on a fresh dump.
- **Cheapest prior falsifier:** the planted-rule tests (passed); then this read.
- **Registered order (frozen):** into `analysis/artifacts/c522/`:
  1. Build the tuner at the head; dump the three files with `--scale`; `ocb_scale_screen.py verify` each (the engine's factor without the rule-50 factor, the tuner since `0fe4702`).
  2. `ocb_free_passers.py two --train train_scale.csv --held-out validation_scale.csv --k 1.63034 --output two.json`.
  3. `ocb_free_passers.py drawn --dump drawn_scale.csv --form two --candidate <a,b,c,d> --threshold 100` with the fitted two-regressor constants, and the same with `--form count --candidate <a,b,c>` for the count form refitted in step 2, for comparison.
  - Nothing is added after the numbers are seen.
- **Rule (frozen):** the two-regressor rule is **eligible to build** when (a) above the boundary (seven men or more) its gain against the split is no worse than −2 standard errors, (b) below it (six or fewer) its gain is positive at three standard errors or more (PLAN C.5.2.2's rule), and (c) **the static veto**: in KBPP-KB, KBP-KB and KBP-KBP the share of Syzygy wins scored under +100 cp rises by at most 5 points against the head's. Clause (c) is added here because RAR-E26's count rule passed (a) and (b) in RAR-E33 and still lost KBPP-KB conversion in play (RAR-E27), and RAR-E26 had read that failure statically as wins scored under +100 (12 → 75 of 253 in KBP-KB).
- **Use, frozen:** eligible → the finding goes to the maintainer with the recommendation to build it (`I2`), check it on RAR-E27's conversion instrument (a scratch build, Syzygy-graded playouts of KBP-KB, KBPP-KB and KBP-KBP at 60,000 nodes, within two standard errors of the head) and gate it as C.5.2 was (RAR-E29's opposite-bishop cohort SPRT design and a 2,000-game STC harm read); nothing is built before the maintainer agrees. Not eligible → C.5.2.2 closes `NO_CHANGE`, the hold closes on RAR-E29's measurement (small, not more drawn), and the retry trigger is a game record showing a cost at the boundary.
- **Result:** pending.
- **Artifacts / commits:** `d699973`; `analysis/artifacts/c522/` (ignored storage).
