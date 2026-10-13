# RAR-E60 — C.8.2 refit cycle 1 on `hce-v5-tb`: the initialization control, static screens, tree and NPS reads, gate 1 `[0,3]`

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-13, before either fit, any screen and any game. The fits, the control, the screens, the tree read and the NPS read are agent-run; the games are the maintainer's.
- **Baseline SHA / candidate SHA / dirty-diff hash:**
  - *Baseline:* the accepted head, `master`'s `52cc6de` (engine equal to `ab2a6ab`, **14,197,472 / EBF 2.557**). On `c7` the last commit with that engine is this registration's.
  - *Candidate:* the fit's baked vector, committed on `c7` after this registration (an engine commit of `src/eval/params.rs` and `src/eval/material.rs` only); its fingerprint is recorded below before any screen.
  - No structural change and no `src/search/` change: this is the surface refitted on the new corpus.
- **Binary / compiler / PGO identity:**
  - `rustc 1.98.1`. The fits use `tools/texel-tuner` built by the fit driver at its tree (`0d3bd73`'s tooling: `-InitialVector`, `--dump-scores --weights`, `neutral_vector.py`).
  - Screen builds are tuner and `tune,diag` builds of each arm's own tree.
  - Gate arms are `build_test.ps1` PGO builds from clean trees. The head arm is C.8.1's `tools/test_engines/rarog-c8head-pext-pgo.exe` (`E73E3230…C1C7`, bench 14,197,472), whose engine is the head's.
- **Research question:**
  - Does refitting the unchanged surface on `hce-v5-tb`, labelled by the accepted head, make the engine stronger at `3+0.03`, search frozen (PLAN C.8, cycle 1)?
  - Does the fit's start bias it (the initialization control, PLAN C.8)?
- **Hypothesis / proposed mechanism:** better labels and positions from a stronger labeller (the C-phase head over the B.9 head) refine the same coefficients. Starting from the accepted vector carries no material bias at this budget.
- **Competing hypotheses:**
  1. *Nothing to gain:* `hce-v4-tb` and `hce-v5-tb` label nearly alike (RAR-M68: 6.74% against 6.86% of clean wins not won), so the refit moves coefficients without strength, and gate 1 reads near zero.
  2. *Magnitude drift:* a refit on another corpus moves magnitudes the frozen search's margins rely on, the tree read flags, and the margin block (C.8.3) decides.
  3. *Initialization bias:* the accepted vector sits in a basin the corpus would not choose, and the neutral start fits the frozen test better.
- **Interacting mechanisms / consumers:**
  - Every cp-valued margin and the correction history (through the score), and `LazyMargin` (the gate's share moves with magnitudes).
  - The eval and pawn caches (pure functions).
  - RAR-E58 and RAR-E59's phase-dependent calibration of large scores, re-read here only through the magnitude screen.
- **Invariants and how they are tested:** the fit driver's own gates. The trace reconstructs the baseline and the final vector exactly, the baked candidate passes debug and release suites and clippy, the source restores byte for byte, and the candidate patch applies.
- **Registered order (frozen):**
  - **Fits** (`analysis/artifacts/c82/run_fits.sh`), from a clean tree at this registration's commit:
    - *Arm A* (accepted start): `pwsh -NoProfile -File tools/texel/fit_complete.ps1 -DatasetDir tools/texel/data/hce-v5-tb -TargetTrain 3500000 -DeferFrozenTest RAR-E60`.
    - *Arm N* (neutral start): the same with `-InitialVector analysis/artifacts/c82/neutral.txt`. That vector is written by `tools/texel/neutral_vector.py` from the compiled defaults and manifest v5: the 1,142 free linear coefficients are zeroed, except the piece values at 100 / 300 / 300 / 500 / 900 in both phases; fixed and coordinate-stage coefficients are kept.
    - The script's defaults hold otherwise: manifest v5, 200,000 coordinate positions, 40 / 200 / 40 / 60 epochs, `lr` 0.3, L2 1e-7, K pinned from the defaults on validation. Both arms must pin the same K, or the control is void.
  - **The initialization control (the frozen test, read once):**
    - `rarog-texel --compare-frozen tools/texel/data/hce-v5-tb/test.csv <A's final> <N's final> tools/texel/data/hce-v5-tb/frozen-test.RAR-E60.opened --fix-k K` is the official, one-shot read.
    - Then `--dump-scores --weights` of each final vector and of the defaults (D) on the same test rows. Each row's game comes from the source PGN with the extractor's split rule (`c82/provenance_split.py`, RAR-E59's method).
    - `c82/init_control.py` computes the tuner's own per-row loss and paired differences with a game-clustered bootstrap standard error (2,000 resamples, seed 20261013).
    - *Wire, else void:* the dumps' mean losses for A and N equal `--compare-frozen`'s printed losses to 1e-7; every test row finds its game.
    - **Rule:** N becomes the cycle's candidate when its frozen-test loss is lower than A's at two or more game-clustered standard errors; otherwise A is the candidate and initialization carries no material bias. A over D (the refit's own static gain on unseen rows) is reported and decides nothing; games decide.
    - The frozen test is consumed by this read. Every later figure is on validation or in games.
  - **The candidate** (A or N by the rule) is applied from its `candidate-eval.patch` and committed on `c7` as an engine commit. Its `bench 13` is recorded before any screen; it must equal the fit's `bench-candidate`.
  - **Pre-game reads, in order** (into `analysis/artifacts/c82/`), RAR-E42's method on `hce-v4-tb` validation, where RAR-E23's donor rows are:
    - (a) *The donor screen:* the head side is RAR-E57's and RAR-E58's dumps (`c71/head_played.csv`, `c72/head_full.csv`, `c71/scores-9587-head.csv`), reproduced first byte for byte by a tuner build of the head tree. Then the candidate's played score substituted, and `donor_residual.py analyse --within` on both.
    - (c) *Magnitude:* `analysis/artifacts/c2-screens/magnitude.py` over the played and `--full` dumps of both arms.
    - (d) *Tree read:* `analysis/artifacts/c03-lazy/positions_v1.epd`'s 72 positions at depth 12, cleared table, `tune,diag` builds of both arms reproducing their `bench 13`, `analysis/artifacts/c2-screens/tree_read.py`.
    - (e) *Pooled-PGO NPS:* PROCESS's method, four clean `pext` PGO builds an arm (`nps_build_pool.ps1`), two cycles, idle host.
  - **Flags** (as RAR-E42): a magnitude band ≤ 600 shifting by more than 20 cp toward the sign, or the share above the gate moving by 2 points or more; the tree read's total nodes changing by more than 5% either way, or its geometric-mean ratio leaving 0.95 to 1.05. A flag sends the candidate to C.8.3's margin block after gate 1 whatever gate 1 reads, and rejects nothing.
  - **Stops before any game:** a void control or a void reproduction; the NPS point estimate below −1.5% (the handover waits for the maintainer).
  - **Gate 1:**
    - Colosseum (`tools/colosseum.ps1 -Mode sprt -Bracket default`): pentanomial GSPRT `[0,3]` nElo, `3+0.03`, one thread, Hash 64, the UHO book in random order, no adjudication, cap **20,000 pairs**.
    - Cap check (RAR-M10, prospectively): at the expected +2 Elo, about +3 nElo, the drift per game is 8.3e-6 × 3 × (3 − 1.5) ≈ 3.7e-5, so about 80,000 games to the upper bound. That is past the cap: a true gain this small most likely ends at the cap, and the cap is kept as RAR-E42's.
    - The runner's fault policy applies by reference (PROCESS, *Harness*). Bounds, cap, book and adjudication never change after games.
  - **Acceptance and use:** gate 1 passing with no flag accepts, and PROCESS step 4's question about the optional margin block goes to the maintainer. Otherwise C.8.3 (the margin block, gate 2, the `10+0.1` read) decides. A rejected cycle is reverted on `c7` in its own commits, and C.8 stops at the first non-accepting cycle.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Fits:* every coordinate pass in both arms converges (0.8). N's best validation loss is above A's (0.75); the epoch budget does not take a neutral start all the way.
  2. *Control:* A is the candidate (0.8). A's frozen-test loss is below D's at two clustered standard errors or more (0.85).
  3. *Candidate:* `bench 13` changes (0.99).
  4. *Donor screen:* every family within 0.03 points of the head over all rows (0.7).
  5. *Magnitude:* no flag (0.7). *Tree read:* within ±5% and inside 0.95 to 1.05 (0.6). *NPS:* within ±0.5% of the head (0.7).
  6. *Gate 1:* **+2 Elo**, 80% band [−3, +8]; probability of passing 0.3.
  - Most likely failure mode: competing (1), gate 1 at the cap or H0.
- **Falsification criteria:** gate 1 and gate 2 both fail: the refit is no better at this corpus, and C.8 stops at cycle 1.
- **Cheapest prior falsifier:** the control and the static screens (no game).
- **Artifacts:** `analysis/artifacts/c82/` (ignored storage); the fits in `tools/results/hce-fit-<stamp>/`.
