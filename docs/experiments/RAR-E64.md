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
- **Result (run 2026-10-15, once, after the registration commit `f182acc`; exit 0, 26 s; `analysis/artifacts/c134cal/check.json`, `check.log`):**
  - *Wires held:* scores, family sums and games as in RAR-E63. With the `phase` base both pairs reproduce RAR-E63's guard gains exactly (+0.130708, +0.108938; difference 0).
  - *Gains on the non-lazy rows* (144,537, ± game-clustered SE):

    | Base | material × men | mobility × men |
    |---|---:|---:|
    | phase (RAR-E63's guard) | +0.131 ± 0.021 (6.3 SE) | +0.109 ± 0.017 (6.3 SE) |
    | men | +0.139 ± 0.022 (6.4 SE) | +0.107 ± 0.017 (6.4 SE) |
    | pawns | +0.151 ± 0.023 (6.6 SE) | +0.103 ± 0.017 (6.2 SE) |
    | **phase + men + pawns** (deciding) | **+0.122 ± 0.021 (5.8 SE)** | **+0.102 ± 0.017 (6.0 SE)** |

  - *By the frozen rule, both pairs survive*: each is above 0.10%, 4 SE and RAR-E63's null threshold (+0.024%) with every calibration in the base. No calibration slope moves either gain by more than 0.03 points. The score × pawns slope removes nothing from material × men (+0.151 against RAR-E63's +0.151) and 0.006 from mobility × men. H-cal is refuted for both; H-term stands.
  - *Margin:* mobility × men clears the 0.10% clause by 0.002 points. The game bootstrap does not carry the boosters' fitting noise (RAR-E63's instrument note), so this pair's survival is the weaker of the two, while its 6.0 SE and the null threshold, a quarter of the gain, stand.
- **Disposition:** observation. **By the frozen use, both pairs keep their C.13.5 candidacy, and each gets one definition card there:**
  - material × men: a men- or pawn-dependent material scale, the value of a material edge by the men left beyond the piece-based phase taper;
  - mobility × men: a mobility weight by men.
  - The cards state the form, the interactions (the material and imbalance terms, the endgame scale and C.5.2's rule, the lazy gate's margin, mobility's own eg tables and the closedness term), the invariants and the falsifier before C.13.5 builds. Each is fitted in C.13.5's refit and gated with the cluster.
  - Not Elo: static loss ranks questions and accepts nothing.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 5 above.
  - Observed:
    - (1) hit.
    - (2) **miss**: material × men survives (+0.122), and the pawn slope absorbed nothing of it.
    - (3) hit: mobility × men survives (+0.102).
    - (4) hit: both survive.
    - (5) **miss** for material (the men base removes more than the pawns base; the pawns base removes nothing) and hit for mobility (pawns 0.103 against men 0.107); as a statement about either pair, a miss.
  - Which assumption failed: I read the descriptive table's men-band main effect as the pair's content. The pair is the within-band pattern (a large edge overclaims when crowded, underclaims when sparse), which no monotone score slope can express. The miss is in mechanism.
- **Conditional lesson:** a residual table's dominant main effect says nothing about an interaction. Read the within-band contrast before predicting what a calibration will absorb.
- **Retry trigger or `closed`:** closed. Both candidates go to C.13.5's definition cards.
- **Artifacts:** `analysis/artifacts/c134cal/` (ignored storage): `check_e64.py`, `check.json`, `check.log`, and the smoke `smoke30k-check.*` (exit status only).
