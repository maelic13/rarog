# RAR-E63 — C.13.3: pairwise interactions in the held-out residual, read by shallow boosted trees

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-15, before any gain on the held-out rows was computed; agent-run (zero games, one trace dump, one read).
- **Kind:** research. A zero-game read that names interactions for C.13.5, or closes `NO_CHANGE`; nothing is built.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c7` at the registration commit; engine and tuner sources equal `1b56071`'s (`git diff 1b56071 HEAD -- src tests Cargo.toml Cargo.lock tools/texel-tuner` empty), whose engine is `43b34ec`'s, **10,521,141 / EBF 2.521**. No candidate.
- **Rows:** the same as RAR-E62's: `hce-v6-tb/validation.csv`, 194,444 held-out rows (`7ada0924…`). W1 compares the played score with RAR-E62's `c132/head_played.csv` (`--dump-scores` on this engine), and game ids come from RAR-E62's `c132/provenance.csv`, which reproduced the manifest's games.
- **Instrument** (`analysis/artifacts/c133/`, ignored storage):
  - `zz_c133_dump.rs`, a temporary integration test run `--release --features texel` and removed after (RAR-E62's pattern). For every row it writes, from the played evaluation:
    - the White-POV score;
    - whether the played path was lazy;
    - the engine's phase and the endgame scale factor;
    - the linear score the trace reconstructs, and the frozen part tapered (the king-danger map and mop-up);
    - each of eleven term families' tapered contribution: material, piece-square, imbalance, pawns, passers, mobility, threats (with hanging), king, pieces, initiative and tempo. Every weight field belongs to exactly one family; the test asserts it.
  - `read_e63.py` holds the base model, the booster (numpy, histogram-based; no external library), the clustered errors, the permutation null and the wires.
  - `test_booster.py` is a synthetic check of the booster.
- **Known exposure, stated before the read:**
  - RAR-E57 to RAR-E59 (C.7): on `hce-v4-tb` and an earlier head, a phase slope among decided scores, closed as calibration.
  - RAR-E62 (this head, these rows): no corrected definition at the line; initiative +0.069 ± 0.019% over all rows, and +0.54 in the reported-only endgame band.
  - RAR-E17 and later reads: the donor's families add little beyond Rarog's score in every band.
  - No interaction on these rows has been computed.
  - **Development of the instrument (no held-out row touched):**
    - First, pairs were contrasted against stumps after a 300-round stage 1. A 30,000-row smoke on *training* rows with stand-in game ids showed (from the planted wire alone) that leftover main effects read as interactions.
    - The baseline became same-capacity single-input depth-2 trees, and stage 1 was converged at 1,000 rounds.
    - A synthetic test then showed a pure-noise pair reaching +0.117% at 5.2 SE. The game bootstrap covers the held-out sample's noise, not the boosters' fitting noise, so the permutation null (raised from 10 to 20 replicates) became the guard. The phase-slope guard was added for the C.7 calibration.
    - The smoke's training-row gains were not read.
- **Research question:** does the held-out residual of Rarog's played score carry a pairwise interaction between its term families, phase, material or the endgame scale that the linear, additively fitted evaluation cannot express, large enough to join C.13.5?
- **Hypothesis:** no. The tapered families already carry their phase dependence, C.5's scale owns the endings, and C.3's king-danger map owns the king nonlinearity. No pair reaches the line, and C.13.3 closes `NO_CHANGE`.
- **Competing hypotheses:**
  1. *king × phase or king × men:* the map's mg and eg outputs mis-taper.
  2. *passers × men or passers × scale:* a passer's value depends on the material left beyond the taper, or the endgame scale mis-scales passer-driven scores.
  3. *mobility × phase:* the mobility tables' endgame values interact with the material left.
  4. *material or imbalance × phase:* RAR-E58's phase slope re-expressed through a family. That is calibration, which the guard tests.
  5. *initiative × anything:* the residue of RAR-E62's endgame-band signal.
- **Interacting mechanisms / consumers:** none in the engine. An admitted pair becomes a C.13.5 candidate with its mechanism stated, and it needs its own definition there. A material- or phase-slope pair touches C.10's `LazyMargin` and the fitting target (C.13.6).
- **Orientation and inputs:** rows are oriented by `s = sign(played)` (+1 at 0). The twelve signed inputs (eleven families and the frozen part) and the outcome residual are multiplied by `s`. The endgame scale, the engine's phase (0–24) and the men count enter unsigned: 15 inputs, 105 pairs.
- **Model (frozen):**
  - *Base C:* RAR-E58's hinge spline of the played score (`r`, hinges at the 25/50/75% quantiles of `|r|` in the cohort, each side), a logistic fitted per fold.
  - *Folds:* two, by game (sorted ids, alternating), for C and every booster.
  - *Booster:* inputs binned at 32 quantiles of the fitting fold; squared error on the oriented residual `s·(y − p_C)`; learning rate 0.1; minimum leaf 500 rows; prediction `clip(p_C + s·g, 0, 1)`.
  - *Stage 1:* 1,000 rounds of stumps over all inputs, the converged additive model, reported.
  - *Pair (i, j):* on stage 1's residual, 100 rounds of depth-2 trees using only i and j, against 100 rounds of depth-2 trees each confined to one of them. Both have the same capacity per round, so the difference is the cross-input splits alone, and the held-out difference is the pair's interaction gain.
  - *Overall (reported):* the same contrast over any pair.
  - *Units and errors:* gains in percent of C's held-out squared error, with RAR-E59's game-clustered bootstrap (2,000 resamples, seed 20261013).
  - *Permutation null:* 20 replicates (seeds 20261016–20261035) with the oriented residual permuted across the cohort's rows. Each replicate's largest pair gain is kept, and the **null threshold** is the largest of the twenty.
- **Cohorts:** deciding, the non-lazy rows (the played path computed every family there, so the inputs are complete and the lazy gate, C.10's, is not a confound). Reported only: all rows, with no null.
- **Rule (frozen):** a pair is **at the line** when its gain is at least **0.10%**, at least **4 game-clustered SE**, and **above the null threshold**. The **phase-slope guard** then re-reads every pair at the line with RAR-E58's slope `r·(phase/128 − 0.5)` added to the base. A pair that still meets all three clauses is **admitted**; one that does not is the calibration C.7 closed.
- **Wires (void if any fails):**
  - W1: the played score equals `--dump-scores` on every row.
  - W2: the families and the frozen part sum to the trace's linear score within 1 cp on every row.
  - W3: every row has a game.
  - W4: a planted interaction (0.10 × centred `[king > median][phase > median]`, with a planted main effect 0.10 × centred `[passers > median]`), added to the first null replicate's residual, must make king × phase the top pair, meeting the full rule. No pair containing passers, the planted main effect, may meet the rule.
  - **Phase proxies:** tempo (mg only) and initiative (eg only) carry the phase in their tapered values, and the men count tracks it. So king × tempo, king × initiative and king × men may share a king × phase interaction. W4 allows that, and the record reports a pair with a proxy as the same interaction as the pair with phase.
  - The tree is clean before the run and after the temporary test is removed.
  - Before the registration, the synthetic test (`test_booster.py`, three seeds) found its planted pair first, at 6.2 to 7.9 SE and at least twice the next pair. The smoke on 30,000 training rows passed W1 and W2. Its planted wire, re-judged from its stored values under the final W4, passed: king × phase first at +0.561% and 6.2 SE, above that smoke's null threshold of +0.142%; king × tempo, × men and × initiative followed as phase proxies; no passers pair among the top five, the fifth at +0.094. Its observed gains were not read.
- **Registered order (frozen):** `analysis/artifacts/c133/run_e63.sh`, once, with every exit status written to `status.txt`: tree check, engine-source check, the dump, the read. Nothing is added after the numbers are seen.
- **Use, frozen:** an **admitted** pair is named with its mechanism and becomes a candidate for C.13.5's cluster. It is defined, built and fitted there, never here; no form is chosen in this record. With none admitted, **C.13.3 closes `NO_CHANGE`**. Stage 1's additive gain and the overall interaction are reported and decide nothing. An additive gain is a pricing question for a refit, not an interaction.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** non-lazy rows, gains in percent.
  1. Every wire passes (0.85).
  2. Stage 1 over C: +0.3, 80% band [+0.05, +1.0].
  3. Overall interaction: +0.05, band [−0.05, +0.20].
  4. Null threshold: +0.05, band [+0.02, +0.12].
  5. The top observed pair includes phase or men (0.6).
  6. **No pair admitted, and C.13.3 closes `NO_CHANGE` (0.75).** If one is admitted, it is king × phase or passers × men (0.5 of that).
  - Expected Elo: none. Static loss ranks questions and accepts nothing.
- **Cheapest prior falsifier:** this read.
- **Result:** pending.
