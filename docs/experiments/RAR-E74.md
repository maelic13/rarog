# RAR-E74 — E.4.0's zero-game reads: where the donor evaluation's games are won, and whether game-result labels see its gain

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-24 (PLAN E.4.0), before any number below was computed; agent-run, zero games, minutes of CPU on an idle host.
- **Kind:** research for E.4.0's packet (`analysis/e40_donor_and_form_2026-11-24.md`). It decides nothing by itself; its frozen uses feed the maintainer's decision on the donor and the form.
- Disposition: observation — the donor's games are won with heavy material (endgame share −0.05 to −0.01); results on `hce-v4-tb` rank Rarog's evaluation above both donors there and are not admissible to fit a rebuild.
- **Research question:** RAR-O13 put Stockfish's classical evaluation +74 to +80 Elo above Rarog's inside the kernel at equal nodes and +68.8 on the clock. (A) Is that gain won in one part of the game, which family repairs could reach, or throughout? (B) Does the static objective Rarog's fitting pipeline uses, cross-entropy against game results on `hce-v4-tb`, rank the donor evaluations above Rarog's as play does? RAR-E70 found Rarog's evaluation alone ahead of `9587eeeb`'s above six men on that corpus (−0.441 ± 0.137% for the donor), while on Stockfish 19's labels both donors lead Rarog by +0.024 to +0.029 skill (E.0's packet).
- **Competing hypotheses for (B):** *H1, the labels.* Game results from Rarog's own games favour the evaluation that produced them and are too noisy to see the gap; the donors do not lead on results. *H2, the gap is static.* The donors lead on results as well, so results can choose a fit. *H3, the corpus.* `hce-v4-tb`'s games were played by the B.9 head at 8,000 nodes, two searches ago (RAR-M67): the donors trail there and lead on the kernel's own games.
- **Inputs (all existing):**
  - (A) RAR-O13's three PGNs, `tools/results/rar-o13-sf11/games.pgn` (`ADA3A052…`), `rar-o13-sf16/games.pgn` (`7D5202D2…`) and `rar-o13-sf16-clock/games.pgn` (`4DA78C14…`); every move carries `s=` (its mover's score).
  - (B1) `tools/texel/data/hce-v4-tb/validation.csv` (194,444 rows, white's result), its games from `analysis/artifacts/c73/provenance.csv` (row-aligned, checked: 0 FEN mismatches).
  - (B2) RAR-S139's games, `tools/results/rar-s139-sprt/games.pgn` (6,046 games, the kernel against a near-neutral variant of itself at `3+0.03`, so neither arm is a donor).
  - (B3, reported only) RAR-O13's two equal-node PGNs, where one arm is the donor.
- **Instrument:**
  - *(A)* `analysis/artifacts/e40-2026-11-24/decided.py`, E.0's `m91_phase.py` rule unchanged: a decisive game is decided, for each side, at the first of its moves after which its own score stays at or past 150 in the result's direction; the band is non-pawn material of both sides in pawns at that move (45+, 25–44, 10–24, 0–9, or never). Per match, the donor arm's wins minus its losses by band, read from the donor's side and from the base's side.
  - *(B)* The bridge gains `donor-dump <rows> <out>` (`tools/donor-eval`): for each row not in check, Rarog's `Evaluator::evaluate` and the donor's raw value at root 0, both from white, with the row's non-pawn material (each evaluation's own K absorbs its scale, so `SCALE_1024` does not enter). Game rows for (B2, B3) by `rows_from_pgn.py`: every position from ply 16 (past the 8-ply book) whose played move is not a capture or a promotion, the side to move not in check, white's result, the game's index. `read_static.py`: for each evaluation its own logistic K by cross-entropy, then the mean cross-entropy against results on the whole corpus and by band, each relative to Rarog's (negative is better), paired, with a game-clustered bootstrap (2,000 resamples of games, seed 20261124). K is refitted inside each resample.
- **Wires (frozen; a failure voids the read):** (1) on `validation.csv`, the squared-error K of each column reproduces the bridges' `donor-k` values to 1e-4 (Rarog 1.64307, `sf_11` 1.09777, `sf_16` 1.15066 on the raw scale), so the dump carries the values the scale rule read. (2) (A) parses every game of each PGN and every move has a score; the per-match decisive counts equal the manifests' (sf11 412 + 834, sf16 395 + 849, clock 419 + 810). (3) `rows_from_pgn.py` replays every game to its recorded result.
- **Rules (frozen):**
  - *(A)* The *endgame share* is the donor's net decided games in bands 10–24 and 0–9 over its net in all bands, `never` included, from the donor's side. It is **concentrated** when the share is at least 0.60 both on the clock read and on the two equal-node reads pooled; otherwise **broad**.
  - *(B)* Results on `hce-v4-tb` are **admissible** to choose the rebuild's fit when both donors lead Rarog there by at least two clustered standard errors; otherwise **not admissible**.
- **Use (frozen):** concentrated leans the packet to endgame repairs first (E.4.2), broad to a whole evaluation unit. Not admissible means any rebuild gates its parity port before any fit, and E.4.0's method question must first find an objective that orders known evaluations as play does; admissible means the pipeline's objective may choose the rebuild's fit, still gated by games. Neither accepts anything.
- **PRE-REGISTERED PREDICTION (frozen 2026-11-24, before any number):**
  1. (A) The endgame share is 0.40, 80% band [0.25, 0.55], in each of the three matches. P(concentrated) 0.15. The pattern is like RAR-M91's kernel against Stockfish 16 HCE, where the endgame bands held 45% of the net.
  2. (B1) On `hce-v4-tb` the two donors are within ±0.3% of Rarog's cross-entropy overall. P(admissible) 0.3. By band, the donors lead in 10–24 and trail at 45+.
  3. (B2) On the kernel's games the donors lead slightly more than on (B1); P(both lead by two clustered SE) 0.35.
  4. (B3) On games the donor arm played, the donor leads by more than on (B2) (P 0.6), the producing engine's own evaluation favoured.
  5. The wires hold (0.95).
  - Expected Elo: none. Static loss and decided-game counts are not Elo.
  - Most likely failure: (A)'s lagging sign. A game the base lost in the middlegame may cross 150 only in an ending, which would push the share up without the endgame being where it was lost.
- **Falsification criteria:** the read is void on a wire failure. Prediction 1 misses if a share falls outside its band; prediction 2 if a donor lies outside ±0.3%.
- **Registered design and stop rule:** one run of each script, after this registration's commit; no cohort, label or model added after the numbers are seen. A script defect found during the run is fixed and the whole run repeated, with the defect recorded. Outputs are in `analysis/artifacts/e40-2026-11-24/` (ignored).
- **Result (run 2026-11-24 14:50–15:10, after the registration commit `1d0f7f9`, every script exit 0 on the final run; outputs `decided.log`, `static.log`, `decided.json`, `static.json`):**
  - *A script defect, recorded as the stop rule asks.* The first run's replay wire failed on one RAR-S139 game, a decisive result with no mate. The game carries `Termination "time forfeit"` and is recorded correctly; the wire was written too narrowly. `rows_from_pgn.py` now counts a recorded time forfeit instead of failing, and the whole run was repeated (S139 1 forfeit, 0 failures; RAR-O13's PGNs 0 and 0). (A) was unaffected and reproduced exactly.
  - *Wires, all holding:* every game of the three RAR-O13 PGNs parsed with a score on every move, and the decisive counts equal the manifests'. The squared-error K on `hce-v4-tb` reproduces the bridges' values (1.64307, 1.09777, 1.15066). Every row of the four dumps is scored, and Rarog's column is identical between the two bridges.
  - *(A), the donor arm's wins minus its losses by the band where its own score passed 150 for good:*

    | Match | 45+ | 25–44 | 10–24 | 0–9 | never | Endgame share | Base's side |
    |---|---|---|---|---|---|---|---|
    | `sf_11`, equal nodes | +337 | +108 | +32 | −55 | 0 | −0.055 | 0.104 |
    | `sf_16`, equal nodes | +357 | +118 | +25 | −43 | −3 | −0.040 | 0.108 |
    | `sf_16`, `3+0.03` | +294 | +100 | +47 | −50 | 0 | −0.008 | 0.215 |

    Equal nodes pooled −0.047, the clock −0.008: **broad** by the rule. The gain is won with most of the material on the board: about three quarters of the net is decided at 45 pawns of non-pawn material or more, and the donor arms lose more games than they win among those decided under 10.
  - *(B), cross-entropy against results relative to Rarog's, %, negative where the donor predicts better, game-clustered 95% intervals (each evaluation's own K):*

    | Corpus | Cohort | `sf_11` | `sf_16` |
    |---|---|---|---|
    | B1 `hce-v4-tb` (194,444 rows, 29,769 games) | all | −0.058 [−0.279, +0.153] | +0.283 [+0.051, +0.501] |
    | | 45+ | +0.584 [+0.183, +0.956] | +0.997 [+0.588, +1.394] |
    | | 25–44 | +0.653 [+0.300, +1.029] | +1.025 [+0.652, +1.417] |
    | | 10–24 | −1.373 [−1.756, −0.993] | −0.660 [−1.057, −0.256] |
    | | 0–9 | −0.992 [−1.571, −0.432] | −1.252 [−1.800, −0.733] |
    | B2 the kernel's games, RAR-S139 (586,012 rows, 6,043 games) | all | −1.819 [−2.367, −0.955] | −1.356 [−1.918, −0.496] |
    | | 45+ | −0.180 [−0.813, +0.505] | +0.101 [−0.559, +0.801] |
    | | 25–44 | +1.612 [+1.058, +2.229] | +2.079 [+1.522, +2.668] |
    | | 10–24 | −3.124 [−3.553, −2.650] | −2.464 [−2.917, −1.974] |
    | | 0–9 | −3.875 [−6.277, +0.350] | −3.756 [−6.123, +0.423] |
    | B3 RAR-O13 `sf_11`'s games (reported) | all | −5.004 [−7.266, −0.866] | −4.931 [−7.155, −0.840] |
    | | 45+ | −8.628 [−10.812, −6.186] | −7.917 [−9.997, −5.504] |
    | B3 RAR-O13 `sf_16`'s games (reported) | all | −5.149 [−7.331, −1.258] | −4.921 [−7.163, −1.014] |
    | | 45+ | −9.295 [−11.279, −7.038] | −8.948 [−11.126, −6.516] |

    `sf_16` minus `sf_11`: +0.341 ± 0.045 on B1, +0.462 ± 0.078 on B2; `sf_11`'s evaluation predicts better on every corpus and in every band but the last. **Not admissible** by the rule: on the fitting corpus `sf_11` is level with Rarog and `sf_16` behind.
- **The frozen uses, applied as written:** (A) **broad** leans the packet to a whole evaluation unit, not endgame repairs first. (B) **not admissible**: results on `hce-v4-tb` may not choose the rebuild's fit. Reported, deciding nothing: with heavy material, exactly where the donor's games are won, game results rank Rarog's evaluation *above* both donors on the fitting corpus (+0.6 to +1.0%), and at 25–44 on the kernel's own games (+1.6 to +2.1%). On games the donor arm shaped, the donor leads by 5 to 9%. Results from one engine's games favour that engine's evaluation by more than the whole gap being measured.
- **PREDICTION CALIBRATION (appended after exposure):**
  1. (A) **missed in mechanism:** 0.40 predicted, −0.05 to −0.01 measured (0.10 to 0.22 from the base's side), outside the band. The assumption that failed: that the evaluation's gain has the shape of the whole-engine gap to Stockfish 16 HCE, whose node rate and depth reach the endings. Alone, the evaluation's gain is a middlegame gain. The lagging-sign failure mode named at registration would have pushed the share up; it came in low.
  2. (B1) **hit** overall: both within ±0.3% (−0.058, +0.283); not admissible as most likely (0.7). The band pattern predicted, donors ahead in 10–24 and behind at 45+, held, and 25–44 went with 45+.
  3. (B2) **hit:** the donors lead by more than on B1, both by more than two clustered SE (stated 0.35, so this outcome was the less likely one).
  4. (B3) **hit:** more than on B2, by 3 to 4 points.
  5. The wires held, after the script defect above.
