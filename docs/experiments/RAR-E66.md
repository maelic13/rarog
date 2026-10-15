# RAR-E66 — C.13.5's kill test: RAR-E63's two pairs under game length and on the search-settled rows

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-15, before either pair was read with a game-length term or on the settled rows; agent-run (zero games, about a minute).
- **Kind:** research. The cheapest test C.13.5's definition cards (`analysis/c135_cards_2026-10-15.md`) name before either candidate may be `READY_FOR_IMPLEMENTATION`. Nothing is built.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at the registration commit; engine sources equal `1b56071`'s (**10,521,141 / EBF 2.521**). No candidate.
- **Inputs:**
  - RAR-E63's family dump (`c133/dump.csv`), the played scores (`c132/head_played.csv`), the provenance with each row's ply and game length (`c132/provenance.csv`) and RAR-E65's search scores (`c134/search.csv`).
  - The instrument, `analysis/artifacts/c135/check_e66.py`, imports RAR-E63's booster and RAR-E64's base code unchanged.
- **Known exposure, stated before the read:**
  - RAR-E63, RAR-E64 and RAR-E65's results.
  - The cards' descriptive tables.
  - A `--size` run that printed only the wires and the subset's size: 98,541 of the 144,537 non-lazy rows (68.2%) are search-settled. That run also reproduced RAR-E65's residual (mean −0.2767, SD 179.34) from its inputs.
  - No smoke on training rows: the loading and residual paths ran in `--size`, and the read reuses RAR-E64's tested code.
- **Research question:** do material × men and mobility × men survive the two explanations that would make a term useless, game length and transient material?
- **Hypotheses** (from the cards):
  - **H-len:** crowded positions are further from the game's end, so any static edge decides less. A game-length term in the base removes the pairs.
  - **H-trans:** the pairs live where the search disagrees with the static score, in transient material. On the rows where the search agrees within a pawn, they vanish.
  - **H-term:** neither removes them.
- **Readings (frozen):** non-lazy rows, the two pairs only, at RAR-E64's deciding base (RAR-E58's spline of `r` plus `r·(phase/128 − 0.5)`, `r·(men/32 − 0.5)` and `r·(pawns/16 − 0.5)`):
  - **len:** that base plus `r·z`, with `z` the cohort-standardised `log1p(plies to the game's end)`, from the provenance's ply and game length.
  - **settled:** that base, on the non-lazy rows whose RAR-E65 residual satisfies |S − g(E)| ≤ 100 cp, with `g` recomputed exactly as RAR-E65 did (98,541 rows).
- **Rule (frozen):** a pair **survives** when it passes **0.10%**, **4 game-clustered SE** and RAR-E63's null threshold (**+0.0236%**) in **both** readings. RAR-E63's threshold is a 105-pair maximum and is conservative for two pairs; on the settled subset the 0.10% clause binds.
- **Wires (void if any fails):**
  - W1 and W2 as in RAR-E63.
  - Every row has a game and a search score.
  - RAR-E65's residual reproduces (mean and SD within 1e-6).
  - The base without the game-length term, on the non-lazy rows, reproduces RAR-E64's deciding gains (+0.12238, +0.10216) within 1e-9.
- **Registered order (frozen):** `python analysis/artifacts/c135/check_e66.py analysis/artifacts/c133/dump.csv analysis/artifacts/c132/head_played.csv analysis/artifacts/c132/provenance.csv analysis/artifacts/c134/search.csv analysis/artifacts/c135/check.json > analysis/artifacts/c135/check.log 2>&1`, once, with the exit status read directly.
- **Use, frozen** (the cards' stop rule):
  - A surviving pair is `READY_FOR_IMPLEMENTATION` with its card's frozen handoff, built, refitted and gated in C.13.5.
  - A pair that fails either reading closes `NO_CHANGE`: game length belongs to C.13.6's target, and transient material is not a term's to fix.
  - Neither surviving closes C.13.5 `NO_CHANGE`, since C.13.2 and C.13.4 admitted nothing.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. The wires hold (0.95).
  2. *len:* both pairs lose under 0.03 points (0.6); RAR-E64's men and pawns slopes already took the monotone part.
  3. *settled:* material × men falls under the line (0.5). Its sharpest cells were large edges in crowded positions, where transient material lives.
  4. *settled:* mobility × men falls under the line (0.6). It sat 0.002 points over the line on all non-lazy rows, and the subset has two thirds of the rows.
  5. **Neither survives, and C.13.5 closes `NO_CHANGE` (0.45).** Exactly one survives (0.35), both (0.2).
  - Expected Elo: none here.
- **Cheapest prior falsifier:** this read.
- **Result (run 2026-10-15, once, after the registration commit `980cfed`; exit 0, 27 s; `analysis/artifacts/c135/check.json`, `check.log`):**
  - *Wires held:* scores, family sums, games and search scores for every row. RAR-E65's residual reproduced. The base without the game-length term reproduced RAR-E64's deciding gains exactly.
  - *Gains* (± game-clustered SE):

    | Reading (rows) | material × men | mobility × men |
    |---|---:|---:|
    | wire: RAR-E64's deciding base (144,537) | +0.122 ± 0.021 (5.8 SE) | +0.102 ± 0.017 (6.0 SE) |
    | **len**: plus `r·z(log1p plies to the end)` (144,537) | +0.109 ± 0.021 (5.3 SE) | **+0.093** ± 0.017 (5.4 SE), under 0.10% |
    | **settled**: the search agrees within a pawn (98,541) | **+0.053** ± 0.025 (2.1 SE) | **−0.021** ± 0.017 (−1.2 SE) |

  - *By the frozen rule, neither pair survives.* Material × men passes the game-length reading but falls under the line on the settled rows. Mobility × men falls under both.
  - On the settled rows the loss is not only power: material × men's point estimate halves (+0.122 → +0.053), and mobility × men changes sign (+0.102 → −0.021), on two thirds of the rows.
  - The pairs live where the search disagrees with the static score: RAR-E65's transient material, an exchange or promotion in progress in a row the quiet filter kept. Game length takes 0.01 to 0.014 points.
- **Disposition:** observation.
  - **By the frozen use, both pairs close `NO_CHANGE`, and with C.13.2 and C.13.4 having admitted nothing, C.13.5 closes `NO_CHANGE`.** Nothing is built, and the cards' handoff is void.
  - The finding goes to C.13.6 and the extraction's quiet filter. A residual concentrated in rows the search does not settle is a data-selection and target question, not a term.
  - Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 5 above.
  - Observed:
    - (1) hit.
    - (2) hit: the game-length term took 0.014 and 0.010 points, both under 0.03; mobility's thin margin then put it under the line.
    - (3) hit: material × men falls on the settled rows.
    - (4) hit: mobility × men falls, and changes sign.
    - (5) hit: neither survives (0.45).
  - Which assumption held: transient material, the cheapest explanation the cards named. A miss in magnitude: mobility × men did not merely weaken on the settled rows, it reversed.
- **Conditional lesson:** an interaction found in quiet-filtered static rows should be re-read on the rows the engine's own search settles before any term is designed. RAR-E65's search dump makes that a one-minute check for every later residual read.
- **Retry trigger or `closed`:** closed. A retry needs a corpus whose quiet filter removes the rows the search does not settle, or a fitting target that takes search scores (C.13.6); either changes what RAR-E63 would read.
- **Artifacts:** `analysis/artifacts/c135/` (ignored storage): `check_e66.py`, `check.json`, `check.log`.
