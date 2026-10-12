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
- **Result:** pending.
- **Artifacts / commits:** `11a3589`; `analysis/artifacts/c67/` (ignored storage).
