# RAR-E64 — C.13.5's readiness: RAR-E63's two candidates against a score × men and a score × pawns calibration

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-15, before either pair was read with any base but RAR-E63's; agent-run (zero games, about two minutes of compute).
- **Kind:** research. The cheapest refutation RAR-E63 named for its two admitted pairs; it decides whether either may get a definition card in C.13.5. Nothing is built.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at the registration commit; engine sources equal `1b56071`'s (**10,521,141 / EBF 2.521**). No candidate.
- **Inputs:** RAR-E63's artifacts unchanged:
  - the family dump (`c133/dump.csv`), the played scores (`c132/head_played.csv`) and the provenance (`c132/provenance.csv`);
  - its read (`c133/read.json`, for the reproduction wire and the null threshold);
  - its booster, folds, orientation, inputs and errors, imported from `c133/read_e63.py`.
  - The instrument is `analysis/artifacts/c134cal/check_e64.py`.
- **Known exposure, stated before the read:**
  - RAR-E63's result: on the non-lazy rows material × men +0.151 ± 0.023% and mobility × men +0.103 ± 0.017%; with RAR-E58's phase slope in the base +0.131 and +0.109.
  - Its post hoc descriptive table (the mean oriented residual by quartile and men band).
  - Men correlates 0.933 with the pawn count and 0.905 with the engine's phase.
  - The script was smoke-run on 30,000 training rows with stand-in game ids for its exit status; its gains were not read.
- **Research question:** are RAR-E63's two pairs knowledge in the material and mobility terms that depends on the men left, or a calibration of how decisive Rarog's score is by the men or pawns left? The latter is the C.7 kind, closed as calibration, which a term cannot fix and the fitting target (C.13.6) or the margins (C.10) own.
- **Hypotheses:**
  - **H-cal:** the pairs re-express a score × men or score × pawns calibration. With either slope in the base, both fall under the line.
  - **H-term:** the material term's value depends on the men left beyond the phase taper (a large edge in a crowded position overclaims, a large edge in a sparse one underclaims), and mobility's value does too. The pairs survive every calibration.
  - **H-mixed:** one survives.
- **Bases** (RAR-E58's hinge spline of the played score `r`, plus):
  - `phase`: `r·(phase/128 − 0.5)`, RAR-E63's guard, which is the wire;
  - `men`: `r·(men/32 − 0.5)`;
  - `pawns`: `r·(pawns/16 − 0.5)`;
  - `phase+men+pawns`: all three, the deciding base.
  - The single `men` and `pawns` bases are reported beside it and decide nothing.
- **Rule (frozen):** a pair **survives** when, with the deciding base, its gain is at least **0.10%**, at least **4 game-clustered SE**, and above RAR-E63's null threshold (**+0.0236%**). Only the two pairs are read; nothing else is added after the numbers are seen.
- **Wires (void if any fails):**
  - W1–W3 as in RAR-E63: scores, family sums, games.
  - Reproduction: with the `phase` base, both pairs reproduce RAR-E63's guard gains within 1e-9.
- **Registered order (frozen):** `python analysis/artifacts/c134cal/check_e64.py analysis/artifacts/c133/dump.csv analysis/artifacts/c132/head_played.csv analysis/artifacts/c132/provenance.csv analysis/artifacts/c134cal/check.json analysis/artifacts/c133/read.json > analysis/artifacts/c134cal/check.log 2>&1`, once, with its exit status read directly.
- **Use, frozen:**
  - A surviving pair keeps its C.13.5 candidacy and gets a definition card there, one definition: for material × men, a men- or pawn-dependent material scale; for mobility × men, a mobility weight by men. Each is fitted in C.13.5's refit and gated with the cluster.
  - A pair that does not survive is calibration. It leaves C.13.5 and is recorded for C.13.6's fitting target and C.10's margins, with no retry here until a changed target or label source fires.
  - With neither surviving and nothing from C.13.4, C.13.5 closes `NO_CHANGE`.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. The wires and the reproduction hold (0.95).
  2. With the deciding base, material × men falls under the line (0.6): the descriptive table's men-band main effect is large, and a score × pawns slope absorbs much of it.
  3. Mobility × men survives (0.45): its within-band slope reverses between crowded and sparse positions, which a monotone score slope cannot express.
  4. **At least one pair survives (0.55).**
  5. The `pawns` base removes more of either pair's gain than the `men` base (0.6).
  - Expected Elo: none.
- **Cheapest prior falsifier:** this read.
- **Result:** pending.
