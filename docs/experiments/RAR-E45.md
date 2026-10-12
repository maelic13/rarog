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
- **Result (run 2026-10-12, once, after the registration commit `3ebabec`; `analysis/artifacts/c522/`, every step exit 0):**
  - *Verify:* the offline rule reproduces the engine's factor on all 3,697,417 rows of the three fresh dumps (0 opposite-bishop mismatches; the 13,311 rows with another factor are specialised material of five men or fewer). Not void.
  - *Fit* (72,904 training and 4,097 held-out pure rows, 185 cells): the two-regressor rule **`s = clamp(0 + 2·pawns + 8·passers + 8·free, 0, 48)`**; the count form refitted the same way is `(0, 2, 9)`, RAR-E33's.
  - *Against the split* (`two.json`), gain in % of the head's held-out squared error: two-regressor **+0.63 ± 0.38 at seven men or more**, **+59.8 ± 4.1 at six or fewer** (637 rows), +11.4 ± 0.9 over all; count form +0.07 ± 0.24, +58.1 ± 3.9, +10.6 ± 0.8. Two-regressor over the count form: +4.0 ± 1.5 below the boundary, +0.56 ± 0.29 above.
  - *Static drawn read* on the 2,973 Syzygy-labelled positions, threshold +100 cp (`drawn-two.json`, `drawn-count.json`); head → two-regressor (count form):

    | Family | Draws over +100 | Wins under +100 |
    |---|---|---|
    | KBP-KB | 512 → 151 (91) of 704 | **7 → 60 (78) of 253: +20.9 points** |
    | KBP-KBP | 168 → 96 (75) of 752 | **40 → 62 (78) of 203: +10.8 points** |
    | KBPP-KB | 254 → 221 (227) of 258 | 0 → 8 (8) of 754: +1.1 points |

  - *Rule:* (a) passes (+0.63, not worse); (b) passes (+59.8 at 14.7 SE); **(c) fails** in KBP-KB and KBP-KBP (wins under +100 up by more than 5 points).
- **Disposition:** **not eligible; C.5.2.2 closes `NO_CHANGE`** by the frozen use. The hold closes on RAR-E29's measurement (the incentive across the boundary is small, those games not more drawn); retry trigger: a game record showing a cost at the boundary. Free passers do help statically: below the boundary the two-regressor rule beats the count form by 4.0 points (2.7 SE) and scores fewer KBP-KB wins low (60 against 78), but it still scores far more of them low than the head's split (7). **A note for the maintainer, which changes nothing here:** clause (c) is stricter than the evidence it rests on. In RAR-E27 the count rule's static under-claim in KBP-KB (75 of 253) did not cost conversion in play (253 of 253 converted in both arms); its loss was in KBPP-KB, where this rule's veto passes (8 of 754). A rule that is better statically on both sides of the boundary and passes KBPP-KB would therefore be a fair candidate for RAR-E27's conversion instrument, under a new registration with conversion as the deciding read. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 6 above.
  - Observed: (1) hit. (2) hit on both (c = 8, d = 8). (3) hit on both. (4) hit on both (better by 2.7 SE). (5) hit (KBPP-KB passes). (6) eligible did not happen (probability 0.35): the veto fired in the two families I did not name.
  - Which assumption failed: I took KBPP-KB as the family at risk because RAR-E27 lost conversion there; the static under-claim instead sits in KBP-KB and KBP-KBP, where a single passer's freedom decides less. The miss is in where the risk sits, not in its sign.
- **Conditional lesson:** a static veto should be set on the family and the failure the deciding instrument actually showed; a proxy wider than the evidence can veto what play would accept.
- **Retry trigger or `closed`:** closed `NO_CHANGE`; retry as above, or on the maintainer's choice of a conversion-deciding registration (the note).
- **Artifacts / commits:** `d699973`; `analysis/artifacts/c522/` (ignored storage): the three dumps, `verify.json`, `two.json`, `drawn-two.json`, `drawn-count.json`, `status.txt`, `source.txt`, the tuner.
