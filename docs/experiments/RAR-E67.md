# RAR-E67 — C.13.6: a blended fitting target (half outcome, half the datagen search's score), one refit, gate 1 `[0,3]`

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-15, before the extraction, the fit, any screen and any game. The extraction, the relabel and the fit (about an hour), the NPS read and the games are the maintainer's commands. The wires, the candidate commit and the static screens are agent-run.
- **Baseline SHA / candidate SHA / dirty-diff hash:**
  - *Baseline:* the current head, engine `43b34ec` (C.13.1; sources unchanged since `1b56071`), **10,521,141 / EBF 2.521**.
  - *Candidate:* the fit's baked vector, committed on `c7` after this registration as an engine commit of `src/eval/params.rs` and `src/eval/material.rs` only, with its fingerprint recorded before any screen.
  - The surface is the same as the baseline's; only the training target changes.
- **Corpus:**
  - `hce-v6b` is `hce-v6`'s source games (`selfplay-c8cycle1-n8000-s1-g569664.pgn`, `C8B78329…`), re-extracted with the same seed, filters, quotas and 14 ranges.
  - Train targets are `0.5·WDL + 0.5·σ(cp)`, where `σ(cp) = 1/(1 + exp(−1.6166·cp/400))` and cp is the datagen search's own score from the move comments. The datagen search was the cycle-1 head at 8,000 nodes, the same engine as the WDL labels.
  - Validation and test stay pure WDL.
  - The Syzygy relabel then sets every train row of six men or fewer to its tablebase verdict, as standard, giving `hce-v6b-tb`.
- **Why K = 1.6166:** the extractor's old blend mapped cp on the Elo scale (k = ln 10 ≈ 2.30), steeper than the scale the tuner fits Rarog's scores to. RAR-E61's fit pinned K = 1.6166 on `hce-v6-tb` validation. On the Elo scale the search half would have been overconfident. `--blend-k` (`5749681`) maps it on the tuner's own scale.
- **Tooling** (`5749681`, `950958e`): `--blend-k` in both extractors, recorded in the manifest; `fit_complete.ps1` accepts pure targets or this named blend (0.5 at 1.6166) only, and its label audit admits fractional train targets only under that contract.
  - Dry run on the cycle-1 pilot's 3,000 games: pure and blended extractions select identical train rows and byte-identical validation and test, and 9,612 of 10,000 train labels are blended. The relabel sets the train rows of six men or fewer to their verdicts.
- **RAR-E04, and why it does not bind:** RAR-E04 (legacy) found pure WDL beat blended labels *on the shared holdout*. That was a static loss on pure-WDL rows, which a blended fit cannot win by construction, and it judged a target change by a criterion that cannot judge one. Its retry trigger, a representation change, fired with Phase C's evaluation. Here games decide.
- **Research question:** does fitting the unchanged surface to a target that is half the game's outcome and half the datagen engine's own search score make Rarog stronger at `3+0.03`, search frozen?
- **Hypothesis / mechanism:** pure outcome labels are noisy for positions far from the end and blind to who played well. RAR-E65 measured that the 8,000-node search score adds +10.4% to outcome prediction over the static score (+19.7% in the endgame band). RAR-E66 found the only interaction the trees detected sits where the search and the static score disagree. A target that includes the search's verdict teaches the static evaluation what the search sees, which is the offline analogue of correction history.
- **Competing hypotheses:**
  1. *Self-reinforcement:* the cycle-1 search's own errors become the target. The fit learns them, static loss on the pure holdout worsens, and games read zero or negative.
  2. *Magnitude drift:* the search half is milder than outcomes for decided positions, so the fit shrinks magnitudes and the frozen search's margins see smaller scores. The magnitude and tree screens flag, and a margin block decides.
  3. *Nothing to gain:* the refit settles near RAR-E61's vector, and gate 1 reads near zero.
- **Interacting mechanisms / consumers:**
  - Every cp-valued search margin and the correction history (through the score's magnitude).
  - `LazyMargin` (the share above the gate moves with magnitudes).
  - The endgame scale and C.5's rules, which multiply the fitted score.
  - The relabelled tablebase rows, which stay pure targets inside a blended train set.
- **Invariants:** the fit driver's gates.
  - The trace reconstructs the baseline and the final vector exactly.
  - The candidate passes the debug and release suites and clippy.
  - The source restores byte for byte, and the candidate patch applies.
  - The corpus wires (`analysis/artifacts/c136/check_corpus.py`) hold:
    - after extraction, row counts equal `hce-v6`'s, validation and test are byte-identical (manifest hashes), train FENs are identical in order, the manifest records 0.5 at 1.6166, and train labels changed;
    - after the relabel, validation and test are byte-identical to `hce-v6-tb`'s, and every train row of six men or fewer carries `hce-v6-tb`'s label.
- **Registered order (frozen):**
  - **Data and fit (the maintainer's command):** `analysis/artifacts/c136/run_c136_data_fit.sh`. It stops at the first failure; every exit status goes to `status.txt`.
    1. A tree check.
    2. `extract_parallel.py <pgn> --out-dir tools/texel/data/hce-v6b --target-train 3500000 --jobs 14 --blend 0.5 --blend-k 1.6166`.
    3. The extraction wire.
    4. `relabel_tb.py --source …/hce-v6b --syzygy D:/chess/tablebases/syzygy3456 --out …/hce-v6b-tb`.
    5. The relabel wire.
    6. `fit_complete.ps1 -DatasetDir tools/texel/data/hce-v6b-tb -TargetTrain 3500000 -BaselineFingerprint "10,521,141 / EBF 2.521" -DeferFrozenTest RAR-E67`, with the script's defaults: manifest v6, 200,000 coordinate positions, 40 / 200 / 40 / 60 epochs, `lr` 0.3, L2 1e-7, K pinned from the defaults on the pure validation.
    7. A check for the fit's `COMPLETE:` line.
  - **Static loss** on the pure validation is reported and is not a criterion. The frozen test stays unread under this ID; it is read once, as a diagnosis, only if gate 1 fails.
  - **The candidate** is applied from the fit's `candidate-eval.patch` and committed on `c7` as an engine commit. Its `bench 13` must equal the fit's `bench-candidate`.
  - **Pre-game reads, as RAR-E61 read them** (into `analysis/artifacts/c136/`):
    - (a) the donor screen on `hce-v4-tb` validation;
    - (c) magnitude (`c2-screens/magnitude.py`, played and `--full`);
    - (d) the tree read (`c03-lazy/positions_v1.epd`, depth 12, `tune,diag` builds reproducing `bench 13`);
    - (e) pooled-PGO NPS by PROCESS's method on an idle host. This one is the maintainer's command.
  - **Flags**, as RAR-E61's thresholds, offer a margin block after gate 1 and reject nothing:
    - a magnitude band at or below 600 shifting by more than 20 cp;
    - the share above the gate moving by 2 points or more;
    - the tree read's total nodes moving by more than 5%, or its geometric-mean ratio leaving 0.95 to 1.05.
  - **Stops before any game:** a void wire or reproduction, or an NPS point estimate below −1.5%.
  - **Gate 1:**
    - Colosseum (`tools/colosseum.ps1 -Mode sprt -Bracket default`): pentanomial GSPRT `[0,3]` nElo, `3+0.03`, one thread, Hash 64, the UHO book in random order, no adjudication, cap **20,000 pairs**.
    - Cap check (RAR-M10, prospectively): an expected +3 Elo is about +5.0 nElo at RAR-E61's ratio. The drift is 8.3e-6 × 3 × (5.0 − 1.5) ≈ 8.7e-5 per game, so about 34,000 games (17,000 pairs) reach the upper bound, inside the cap.
    - Bounds, cap, book and adjudication never change after games.
  - **Use:**
    - *Kept*, only by gate 1 H1: the blend joins C.2's label contract for C.9 and C.10, recorded in PLAN.
    - *Rejected* (H0 or the cap): the candidate is reverted in its own commits. The retry trigger is a changed datagen budget or label source, recorded.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Corpus:* both wires hold (0.9).
  2. *Fit:* every pass converges (0.8). The pure-validation loss of the final vector is **worse** than the baseline vector's (0.6), since a blended fit trades outcome fit for the search's verdict.
  3. *Candidate:* `bench 13` changes (0.99).
  4. *Magnitude:* some band at or below 600 shrinks toward zero by more than 10 cp (0.5). Any flag at all (0.5).
  5. *NPS:* within ±0.5% (0.75).
  6. *Gate 1:* **+3 Elo**, 80% band [−5, +11]. Probability of passing 0.3.
  - Most likely failure mode: competing hypothesis 1 or 3, gate 1 at H0 or at the cap.
- **Cheapest prior falsifier:** the static screens (no games); a void corpus wire stops everything.
- **Result:** pending.
