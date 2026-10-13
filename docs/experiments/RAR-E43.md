# RAR-E43 — C.6.7 pawn and passer definitions: does the donor's form of each carry outcome information beyond Rarog's?

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-12, after the instrument was built and smoke-verified on 4,000 rows and before any loss was computed with Rarog's columns; agent-run (zero games).
- **Baseline SHA / candidate SHA / dirty-diff hash:** the current head, `c5` with engine `38cbb02` (**14,197,472 / EBF 2.557**). Instrument `tools/diag/pawn_subterms.py` at `11a3589`. No candidate.
- **Binary / compiler / PGO identity:** `tools/texel-tuner` built at the head (for `--dump-feature`), its SHA-256 recorded with the result. Scores: `analysis/artifacts/c65-screens/scores-9587-cand.csv`, Stockfish `9587eeeb`'s trace on `hce-v4-tb` validation (194,444 rows) with this head's played score substituted (RAR-E42's screen).
- **Research question:** C.6's unit left the donor's pawns family unabsorbed (+0.120% in the endgame band) and its weak-unopposed term carrying +0.055 points. Which of the definitions the two engines draw differently (`analysis/c67_pawn_definitions_2026-10-12.md`, *Inventory*) carry outcome information that Rarog's form misses?
- **Hypothesis / proposed mechanism:** the pawns residual sits in definitions, not prices: the donor's backward pawn (level neighbours exclude it, blocked pawns qualify), its weak unopposed built on it, and its opposition-conditioned connected value describe weak and strong pawns better than Rarog's.
- **Competing hypotheses:** (1) *prices, not definitions:* Rarog's forms, freely priced, recover the donor's information, and nothing carries beyond them. (2) *spread:* each difference adds a little and none reaches the line. (3) *the passed set:* the donor's lever and candidate rules matter more than any structure term.
- **Interacting mechanisms / consumers:** C.6's unit (weak unopposed is built on Rarog's backward and isolated); the passed set feeds every passer term, C.5.2's opposite-bishop rule and the unstoppable passer; the pawn cache (all structure terms are pure functions of the pawns); C.8's refit, which would fit any change.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** "carries" = the donor's form beyond Rarog's gains at least 0.04 points at four standard errors in the endgame band (phase < 32) or without queens.
  1. *Verify:* the donor's rows reproduce and Rarog's counts equal the engine's trace on every readable row (probability 0.9).
  2. `weak_unopposed` carries (probability 0.45); `backward` carries (0.4); `connected` carries (0.35); `doubled` carries (0.3); `passed_set` carries (0.25); `isolated`, `doubled_isolated`, `lever` and `blocked` do not (0.7 each).
  3. At least one definition carries (0.6).
  - Expected Elo: none.
- **Falsification criteria:** void if `verify` or `verify-rarog` finds a mismatch on a readable row, or if the shuffled null (the donor's backward column permuted, beyond Rarog's) reads outside ±0.03% in any cohort.
- **Cheapest prior falsifier:** the 4,000-row smoke (both verifies exact).
- **Registered order (frozen):** into `analysis/artifacts/c67/`:
  1. `pawn_subterms.py collect --scores <the file above> --out . --workers 14`; `verify` (the donor's rows).
  2. `rarog-texel --dump-feature tools/texel/data/hce-v4-tb/validation.csv engine/<field>.csv <field> --full` for each of `pawn_isolated_eg`, `pawn_doubled_eg`, `pawn_doubled_isolated_eg`, `pawn_backward_eg`, `pawn_weak_unopposed_eg`, `pawn_lever_eg`, `passed_candidate_eg`, `pawn_islands_eg`, `pawn_connected_eg`, `pawn_phalanx_eg`, `passed_eg`; then `verify-rarog`.
  3. One `compare`. Nothing is added after the numbers are seen.
- **Rule:** a definition **carries** at 0.04 points and four standard errors (the line above); it is **weak** at 0.02 points and three standard errors in either cohort; otherwise **nil**. The donor's form *instead of* Rarog's is reported beside, to say which form alone predicts better.
- **Use, frozen:** the finding in the research record ranks the definitions by what they carry, states for each what a change would touch (the shared signals) and cost, and recommends. **Nothing is decided or built before the maintainer has reviewed it** (maintainer decision 2026-10-11). A change the maintainer approves becomes its own step, fitted in C.8's refit or in a unit of its own.
- **Result (run 2026-10-12, once, after the registration commit `ce38a72`; `analysis/artifacts/c67/`, every step exit 0):**
  - *Verify:* the donor's rows reproduce (passed 175,165 and pawns 188,844 rows matched, 0 mismatches). *Verify-rarog:* all 11 engine fields equal Rarog's offline counts on 189,284 readable rows (5,160 at full phase unreadable), 0 mismatches, none missing. *Null:* the shuffled donor backward beyond Rarog's reads −0.016 to +0.003 in every cohort, inside ±0.03. Not void.
  - *The donor's form beyond Rarog's* (points of held-out loss, ± one standard error; `read.txt`, `compare.json`):

    | Definition | Endgame band | No queens | All rows | Grade |
    |---|---:|---:|---:|---|
    | backward | −0.006 ± 0.011 | +0.002 ± 0.007 | +0.003 ± 0.004 | nil |
    | weak unopposed | +0.032 ± 0.018 | +0.038 ± 0.018 | +0.012 ± 0.008 | nil |
    | isolated | −0.004 ± 0.014 | +0.026 ± 0.014 | +0.001 ± 0.003 | nil |
    | doubled | +0.008 ± 0.013 | +0.019 ± 0.018 | +0.003 ± 0.002 | nil |
    | doubled and isolated | −0.011 ± 0.013 | +0.025 ± 0.013 | +0.002 ± 0.003 | nil |
    | connected | −0.006 ± 0.004 | −0.005 ± 0.005 | +0.033 ± 0.012 | nil (in the rule's cohorts) |
    | lever | −0.005 ± 0.005 | −0.006 ± 0.003 | +0.001 ± 0.001 | nil |
    | blocked | −0.009 ± 0.005 | −0.023 ± 0.010 | −0.006 ± 0.005 | nil (it hurts) |
    | **passed set** | **+0.141 ± 0.039** (3.6 SE) | **+0.121 ± 0.037** (3.3 SE) | +0.054 ± 0.013 (4.1 SE) | **weak** |

  - *Beside it, deciding nothing:* the forms alone are small and close: in the endgame band Rarog's weak unopposed gains +0.059 ± 0.029 and the donor's +0.099 ± 0.028; without queens Rarog's isolated +0.070 ± 0.030 and the donor's +0.029. For the passed set neither engine's set alone beats the other (the donor's instead of Rarog's +0.004 ± 0.004), but the two together gain +0.132 in the endgame band: the information is in the pawns on which the sets disagree.
- **Disposition:** observation. **No definition carries at the registered line**; the passed set is **weak** (3.6 and 3.3 standard errors against the four the line asks). H1 (the pawns residual sits in definitions) is refuted at this resolution; competing (1) and (2) hold: the donor's pawns residual is spread over many small differences, each worth about as much as re-pricing Rarog's own form. The passed set is the one lead, and this read cannot say which disagreeing pawns carry it (Rarog's rear doubled passers, or the donor's lever and supported-push candidates, or its removal of blocked candidates without help). The finding and recommendation for the maintainer are in `analysis/c67_pawn_definitions_2026-10-12.md`; nothing is built before the review. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 3 above.
  - Observed: (1) hit. (2) every "carries" prediction missed (weak unopposed, backward, connected, doubled, the passed set); every "does not" hit (isolated, doubled-isolated, lever, blocked). (3) **miss**: none carries.
  - Which assumption failed: I read the pawns family's residual as a few wrong definitions; it is broad and thin, and Rarog's own forms re-priced take about as much of it as the donor's. The miss is in mechanism; the passed-set lead was the least expected.
- **Conditional lesson:** read a family's residual against Rarog's own forms freely priced before attributing it to definitions; a residual that re-pricing recovers is the refit's, not a definition's.
- **Retry trigger or `closed`:** closed as a read; the passed-set lead is the maintainer's question (the finding).
- **Artifacts / commits:** `11a3589`; `analysis/artifacts/c67/` (ignored storage): `pawn_subterms.csv`, the engine dumps, `verify.json`, `verify-rarog.json`, `compare.json`, `read.txt`, `status.txt`, `source.txt`.
