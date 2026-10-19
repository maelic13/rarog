# RAR-E62 — C.13.2: four corrected definitions as directions over Rarog's score, and the inert-term inventory

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-15, before any gain, inventory value or provenance count was computed on the held-out rows; agent-run (zero games, one tuner build, one `--dump-scores` pass, one trace dump, one read).
- **Kind:** research. The cheapest read that can admit a definition to C.13.5's cluster or close it `NO_CHANGE`; nothing is built.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at the registration commit; engine sources equal `43b34ec`'s (C.13.1's head, **10,521,141 / EBF 2.521**). No candidate.
- **Rows:** `tools/texel/data/hce-v6-tb/validation.csv` (194,444 rows, SHA-256 `7ada0924…`), held out from RAR-E61's fit, whose source games are `selfplay-c8cycle1-n8000-s1-g569664.pgn` (`C8B78329…`, 569,664 games; the manifest's split 512,788 / 28,284 / 28,592).
- **Instrument** (`analysis/artifacts/c132/`, ignored storage):
  - `zz_c132_dump.rs`, a temporary integration test (the audit's pattern: copied into `tests/` for the run and removed after; never committed), built `--release --features texel`. Per row it writes the played and full White-POV scores, the engine's phase, whether the played path was lazy (played and full traces differ), and from the full trace the untapered mg sum without imbalance and tempo, the eg sum without imbalance and initiative, initiative's eg count, the frozen parts, whether the king-danger map fired, the `passed_mg` counts by rank and the two outpost counts. It also writes the inventory sums.
  - `read_e62.py` computes the definitions from the FEN with python-chess, rebuilds the endgame sum at the initiative term, applies the wires, and reads the directions with `tools/diag/donor_residual.py`'s fitting unchanged.
  - `analysis/artifacts/c82/provenance_split.py` (RAR-E60's, unchanged) recovers each row's game; `check_prov.py` checks it.
- **Known exposure, stated before the read:**
  - The audit's frequency counts on these rows (`analysis/artifacts/c13audit/frequencies.log`): 6,832 of 207,961 passers have an own pawn ahead; the donor-shaped trapped rook fires 8,603 times (side-rows), the engine's 173; outposts by relative rank 4 / 5 / 6 / 7 / 8: 26,472 / 22,250 / 6,418 / 1,242 / 197.
  - RAR-E57 (`hce-v4-tb`, an earlier head): the donor's Knights, Bishops and Rooks rows under +0.06% everywhere. RAR-E17 (the 2.4.0 evaluation): the donor's winnability family +0.12%.
  - The scripts were smoke-tested on the first 3,000 rows of `hce-v6-tb/train.csv` with `--smoke`, which prints only the wire checks (all passed: the offline passed, outpost and initiative counts equal the engine's on every row; mop-up fired on 1,394 rows). No gain was computed.
- **Research question:** C.13's audit found four definitions that miss their common case. Does any corrected definition carry outcome information Rarog's played score lacks, enough to join C.13.5's single refit and gate? And which remaining terms are inert, for C.12 and the fitting manifest?
- **Hypothesis:** the fit has already priced what these definitions can add. None reaches the line, and all four close `NO_CHANGE`. Initiative is the likeliest exception, because its current form cannot say that a simple position is drawish.
- **Competing hypotheses:** (1) *initiative:* an intercept and the sign clamp let the term shrink simple endgames toward a draw, which the fitted scale and C.5.2's opposite-bishop rule do not cover, and it reaches the line in the endgame band above six men. (2) *passers:* a rear doubled passer scored as a full passer overclaims, and removing or halving its bonus reaches the line among endgames. (3) *outposts:* rank-4 outposts, the commonest, carry middlegame value the current definition misses. (4) *trapped rook:* the donor-shaped condition fires 50 times as often as the engine's and carries a mobility penalty the mobility tables miss. (5) *pricing, not knowledge:* any gain is a re-pricing of Rarog's score, which the spline comparator absorbs.
- **Interacting mechanisms / consumers:** none in the engine; nothing is built. A definition that is admitted is built and refitted in C.13.5, sharing inputs with: the passer family and C.6's terms (passers); mobility and the open-file terms (trapped rook); the outpost and minor-behind-pawn terms (outposts); and the endgame scale, C.5.2's rule and the lazy gate (initiative).
- **Definitions** (White POV, cp, tapered by the engine's phase `p`: `(mg·p + eg·(24−p))/24`). The trapped-rook, outpost and initiative directions are zero on rows the played path left lazy, because those terms live in piece activity:
  - `passer_own_ahead`: minus the `passed_mg`/`passed_eg` rank bonus of every passer (the engine's mask: no enemy pawn on the three files ahead) that has an own pawn ahead on its file. A weight of +1 excludes such pawns, +0.5 halves them (the donor's shape). The direction covers the rank bonus only, not the passer's other terms.
  - `trapped_rook`: minus `(1 + [no castling rights]) · (25, 5)` for every rook with mobility ≤ 3 (attacks minus own pieces), its king on its home rank, and `(king file < e) == (rook file < king file)`. This is the audit's donor-shaped condition; the value is the donor's `TrappedRook` S(52, 10) at 100/206.
  - `outposts`: the engine's outpost condition (supported by an own pawn, not attacked by an enemy pawn) for knights and bishops at relative rank 4, minus at ranks 7 and 8, each at its piece's current outpost weights. That is the corrected ranks 4 to 6 minus the current ranks 5 and up.
  - `initiative`: `sign(eg)·max(5.34·(cx − 10), −|eg|) − sign(eg)·cx·w`, eg only. Here `cx` is Rarog's complexity (pawns + king-file distance + both flanks) and `w` the fitted `initiative_weight` (1). The donor's 11 per pawn and intercept −110 are taken at 100/206: 5.34 cp per unit and an intercept of −10 units. `eg` is the running endgame sum at the initiative term, rebuilt exactly as the eg sum without imbalance and initiative, plus the frozen eg, minus the mop-up when it fired. Mop-up adds to eg after initiative; it fires when `|(mg+eg)/2| > 200`. Minor-piece mate rows, where the offline mop-up does not apply, are counted and left at zero.
  - Reported only: `initiative_unclamped` (without the `max`), each direction's active rows, and its implied scale against Rarog's score (the fitted weight ratio).
- **Model (frozen):** RAR-E58's nested held-out logistic: the gain of `C + D` over `C`, in percent of `C`'s held-out squared error. `C` is the hinge spline of the played score (`r` and hinges at the 25/50/75% quantiles of `|r|` in the cohort, each side), using `donor_residual.held_out_predictions` (two-fold, standardised on the fitting fold). Standard errors are RAR-E59's game-clustered bootstrap (2,000 resamples, seed 20261013) of `Σd/Σe₀`, with the linearised and naive ones reported beside it.
- **Cohorts:** deciding, RAR-E57's nine: all, phase ≥ 96, phase 32–95, queens, men ≥ 7, |rarog| ≤ 500, |rarog| > 500, phase < 32 with men ≥ 7, and no queens with men ≥ 7 (the donor's phase scale, 0–128). Reported only: phase < 32, no queens, men ≤ 6.
- **Rule (frozen):** a direction is **at the line** in a deciding cohort when its gain is at least **0.10%** and at least **four game-clustered standard errors** (bootstrap).
- **Wires (void if any fails):**
  - W1: the dump's played score equals `rarog-texel --dump-scores` on every row.
  - W2: the provenance reproduces the manifest's games by split exactly, every row finds a game, and there are no parse errors.
  - W3: the offline definitions reproduce the full trace on every row: the passed counts by rank, the knight and bishop outpost counts, and initiative's count `sign(eg)·cx`. That last check also verifies the rebuilt eg.
  - W4: where the map did not fire, the frozen parts equal the offline mop-up exactly.
  - W5: the label as a direction gains at least 50% over all rows, and a seeded Gaussian direction (seed 20261015) stays under 4 game-clustered SE in every deciding cohort.
  - The tree is clean before the run and again after the temporary test is removed.
- **Registered order (frozen):** `analysis/artifacts/c132/run_e62.sh`, once, with every exit status written to `status.txt` and read directly: tree check, provenance and W2, tuner build and `--dump-scores`, the trace dump, `read_e62.py`. Nothing is added after the numbers are seen.
- **Use, frozen:** a definition **at the line** in any deciding cohort enters C.13.5's cluster, built and fitted there. A correctness repair with a cheap test and no static claim would also enter. The classification is frozen here: **none of the four is a correctness repair.** The audit placed all four under definitions; none is a sign error, an asymmetry or a crash. Every other definition closes **`NO_CHANGE`**, per definition. The inventory decides nothing: it is written for C.12 and the fitting manifest.
- **Inventory:** for every field the removal left, on the played and the full trace, the share of rows where any slot's tapered coefficient is non-zero, and the mean |Σ coefficient·weight| (tapered, pre-scale cp) over all rows and over firing rows. Fields whose weights are all zero are marked. The frozen part (king-danger map and mop-up) is reported on its own line.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** gains in percent, game-clustered SE.
  1. Every wire passes (0.85).
  2. *passer_own_ahead:* over all rows +0.005, 80% band [−0.005, +0.03]; its implied scale is positive (0.6); never at the line (0.95).
  3. *trapped_rook:* over all rows +0.005, band [−0.005, +0.03]; never at the line (0.95).
  4. *outposts:* over all rows +0.01, band [−0.005, +0.04]; never at the line (0.9).
  5. *initiative:* over all rows +0.04, band [0.00, +0.15]; the largest of the four over all rows (0.6); at the line in some deciding cohort (0.2), most likely phase < 32 with men ≥ 7.
  6. **No definition at the line, and all four close `NO_CHANGE` (0.7).**
  7. *Inventory:* `pawn_isolated_*` fires on at least 20% of rows with zero contribution (0.8); at least five other fields have a mean |contribution| under 0.5 cp on the played trace (0.6).
  - Expected Elo: none. Static loss ranks questions and accepts nothing.
- **Cheapest prior falsifier:** this read.
- **Result (run 2026-10-15, once, after the registration commit `4a9b085`; `analysis/artifacts/c132/`, every step exit 0, read from `status.txt`; the tree clean before the run and after the temporary test was removed):**
  - *Wires, all held:*
    - W1: the dump's played score equals `--dump-scores` on all 194,444 rows.
    - W2: the provenance reproduces the manifest's 512,788 / 28,284 / 28,592 games exactly; every row found a game (5 ambiguous) among 27,721 games.
    - W3: the offline passed, outpost and initiative counts equal the engine's full trace on every row.
    - W4: the frozen parts equal the offline mop-up on every row where the map did not fire. Mop-up fired on 93,575 rows, 67,695 of them with the map. The 433 minor-piece mate rows were left at zero.
    - W5: the label control gains 100%; the noise control's largest |z| in a deciding cohort is 3.34.
    - The played path was lazy on 25.7% of rows. Game-clustered errors are 1.04 to 2.18 times the naive.
  - *Gains*, `read.log` (percent of the spline comparator's held-out squared error ± one game-clustered SE; no deciding cohort reached the line):

    | Cohort (rows) | passer_own_ahead | trapped_rook | outposts | initiative |
    |---|---:|---:|---:|---:|
    | all * (194,444) | +0.001 ± 0.013 | +0.002 ± 0.007 | +0.014 ± 0.017 | **+0.069 ± 0.019** (3.7 SE) |
    | phase ≥ 96 * (43,145) | −0.037 ± 0.014 | −0.007 | +0.022 | +0.000 |
    | phase 32–95 * (50,634) | +0.014 | +0.021 ± 0.029 | +0.007 | +0.004 |
    | queens * (108,598) | −0.013 | −0.010 | +0.022 ± 0.021 | −0.001 |
    | men ≥ 7 * (163,836) | +0.001 | −0.004 | +0.023 ± 0.017 | −0.002 |
    | \|rarog\| ≤ 500 * (138,473) | +0.003 | +0.002 | +0.022 ± 0.017 | +0.069 ± 0.019 (3.7 SE) |
    | \|rarog\| > 500 * (55,971) | +0.084 ± 0.085 | −0.055 | −0.024 | −0.008 |
    | phase < 32, men ≥ 7 * (70,096) | +0.038 ± 0.068 | +0.012 | +0.018 | +0.090 ± 0.035 (2.6 SE) |
    | no queens, men ≥ 7 * (64,846) | +0.027 | +0.020 | +0.037 ± 0.031 | +0.060 ± 0.027 (2.2 SE) |
    | phase < 32 (100,665) | +0.024 | −0.043 | +0.015 | **+0.542 ± 0.092** (5.9 SE) |
    | no queens (85,846) | +0.021 | +0.025 | −0.018 | +0.423 ± 0.087 (4.9 SE) |
    | men ≤ 6 (30,608) | +0.098 ± 0.274 | 0 (never active) | +0.006 | +0.288 ± 0.155 (1.9 SE) |

  - *By the frozen rule, no definition is at the line in any deciding cohort.* Initiative comes closest: over all rows it is +0.069 ± 0.019, 3.7 SE, under both clauses, and all of that sits at \|rarog\| ≤ 500. In the reported-only endgame band (phase < 32) it reads +0.542 ± 0.092 with an implied scale of about 2, so the data want twice the donor's per-pawn value there. That band's gain is larger than either of its parts (+0.090 above six men, +0.288 at six or fewer). So part of it is the direction separating the small endings, which the tablebases decide in play, from the rest, which one spline over the band cannot do. It is not evidence for the term above six men. The clamp adds little: the unclamped form reads +0.065 over all rows and +0.491 at phase < 32.
  - The passer, trapped-rook and outpost directions stay at or under 0.04% wherever they are deciding. Their activations are 6,604, 6,019 and 22,282 rows.
  - *Instrument note:* the logistic fit overflowed `exp` in the reported-only no-queens cohort (saturated predictions; that cohort decides nothing). No deciding cohort warned.
  - *Inventory* (`read.json`, played trace; full trace beside it in the file):
    - Fifteen fields have every weight zero. Two are `pawn_isolated_*`, which fire on 59% and 60% of rows, contribute nothing and are kept by C.13.1's decision. The other thirteen are zero halves or members of live terms, computed with their live half, so removing them saves nothing: `passed_supported_eg_base`, `passed_freestop_mg_per_rank`, `pawn_backward_mg`, `pawn_doubled_isolated_mg`, `blocked_passer_eg`, `rook_7th_mg`, `rook_behind_passer_mg`, `threat_weak_piece_eg`, `threat_restricted_eg`, `hanging_minor`, `bishop_long_diagonal_eg`, `minor_behind_pawn_eg` and `pawn_islands_eg`.
    - Fifteen more carry no linear coefficient by construction: the king-danger coordinates and map scales, which reach the score through the frozen map, and the two shelter constants, which cancel between the kings.
    - Near-inert with non-zero weights: `hanging_rook` and `hanging_queen` (weight 1, mean 0.02 cp), `trapped_bishop_*` (0.04% of rows, about 21 to 26 cp when firing, hand-frozen at 60/40) and `bishop_outpost_eg` (1). In all, 26 fields with non-zero weights average under 0.5 cp on the played trace.
    - The largest after material, imbalance and the piece-square tables are `passed_eg` (26.4 cp), `passer_king_them_eg` and `passer_king_us_eg` (16.5, 15.5), tempo (14.8) and `shelter_strength` (14.1). The frozen part (map and mop-up) averages 39.4 cp.
- **Disposition:** observation. **By the frozen use, all four definitions close `NO_CHANGE`** and none enters C.13.5's cluster: the passer with an own pawn ahead, the donor-shaped trapped rook, outpost ranks 4 to 6, and initiative with an intercept and the sign clamp. The inventory names no further whole term at exactly zero. C.12 has nothing more to remove for free; the near-inert terms are a fit question for C.13.5 or C.10, not a removal. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 7 above.
  - Observed:
    - (1) hit.
    - (2) passer: +0.001, inside the band, hit; implied scale positive (+2.1 over all rows), hit, though at 0.1 SE it means nothing; never at the line, hit.
    - (3) trapped rook: +0.002, hit; never at the line, hit.
    - (4) outposts: +0.014 against +0.01, inside the band, hit; never at the line, hit.
    - (5) initiative: +0.069 against +0.04, inside the band, hit; the largest of the four, hit; not at the line (probability 0.2 that it would be). Its strongest deciding cohort by gain was phase < 32 with men ≥ 7 (+0.090), as named, but its highest z was \|rarog\| ≤ 500.
    - (6) hit.
    - (7) both hit: `pawn_isolated_*` fire on 59% and 60% of rows at zero, and 26 non-zero fields sit under 0.5 cp.
  - Which assumption failed: none in sign or decision. On magnitude, initiative was under-predicted by about 1.7 times over all rows, and its reported-only endgame value (+0.54) was not foreseen. That miss is in mechanism: a single spline across a band that mixes tablebase-decided small endings with larger ones leaves room for any phase-and-material-correlated direction.
- **Conditional lesson:** a reported-only band that pools populations the comparator treats as one (here endings of six men or fewer with larger ones) can flatter a direction correlated with that split. Read such a band beside its parts before calling it a signal.
- **Retry trigger or `closed`:** the four definitions are closed. Initiative is retried only if C.13.3's interaction tree or C.13.4's self-search residual names endgame complexity (pawn count or king-file distance against phase) as a split above six men, or if C.13.6 changes the fitting target, which changes what any static read measures.
- **Artifacts:** `analysis/artifacts/c132/` (ignored storage):
  - Scripts: `run_e62.sh`, `zz_c132_dump.rs`, `read_e62.py`, `check_prov.py`.
  - Status and provenance: `status.txt`, `tree-status.txt`, `source.txt` (validation `7ada0924…`, manifest `3cf16236…`, PGN `c8b78329…`, tuner `a9949b87…`), `provenance.csv`, `provenance.json`, `check_prov.log`.
  - Dumps: `head_played.csv`, `dump.csv`, `dump.log`, `inventory.tsv`.
  - The read: `read.json`, `read.log`, `run.log`.
  - Smoke files: `smoke-*`.
