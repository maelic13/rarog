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
- **Fits (2026-10-13, agent-run, after the registration commit `e10ccb5`; `analysis/artifacts/c82/run_fits.sh`, `fits-status.txt`):**
  - *Arm A* (`tools/results/hce-fit-20261013_210320`):
    - The coordinate passes converged (21 and 16 epochs), the linear stage's best validation epoch was 9 of 200 and the polish's 20 of 60 (validation 0.09314569), K 1.61991.
    - Its final vector (`fc3fc678…`) baked, but the debug suite failed one test, `a_saturated_king_is_held_at_the_cap`, whose fixture must drive the danger index past twice the cap: the refit's weights take it to 3,149 against 3,200 (the defaults give 3,381). The property the test asserts needs only an index past the cap (1,600).
    - The precondition was fixed in its own commit, `f634ced` (test only; debug 395 and release 396 pass, clippy and fmt clean, bench 14,197,472 unchanged).
    - The run was then resumed with `fit_complete.ps1 -Resume`. That keeps the fitted vector and the deferral and reruns only verification: debug 416 and release 417 pass, clippy clean, source restored, candidate `bench 13` **14,291,377 / EBF 2.538**.
  - *Arm N* (`tools/results/hce-fit-20261013_210532`, from the neutral vector, 1,090 of 1,263 coefficients changed):
    - The coordinate passes converged (29 and 18). The linear and polish stages ended at their budget, best at epoch 200 of 200 and 60 of 60 (validation 0.09530478); final vector `2c9ee183…`.
    - Its baked vector failed three tests (the same king fixture, the KNN-KP one-capture-neighbour invariant, and a search test of the verification region). N is not the candidate, so they decide nothing.
  - *Deviations, recorded:*
    1. The run script did not stop when the fit drivers reported their suite failures. So the frozen test was read, as registered, on the two fitted vectors, which exist independently of the suites and are what the control compares.
    2. `init_control.py` failed once on a JSON type error after computing its numbers, and was rerun once on the same dumps with the cast fixed; nothing else changed.
    3. A fresh fit of arm A after `f634ced` was refused by the driver, because `RAR-E60` had read the frozen test. That is the guard working, so the resume path was used.
- **The initialization control (`init_control.json`):**
  - *Wire:* the dumps' mean losses equal `--compare-frozen`'s printed ones (A 0.09176799, N 0.09395511) to 1e-8; all 194,444 test rows found their game (29,275 games; 3 ambiguous; the split's game counts reproduce the manifest's); both arms pinned K 1.61991.
  - *N against A:* N's frozen-test loss is **2.38% higher** (± 0.18 game-clustered, −13.2 SE). **A is the candidate**, and initialization carries no bias toward the accepted vector at this budget.
  - *A against D* (the accepted vector, reported): A lowers the frozen-test loss by **+0.080% ± 0.036** (2.2 clustered SE; naive ± 0.021).
  - The frozen test is consumed (`frozen-test.RAR-E60.opened`).
- **Candidate:** arm A's `candidate-eval.patch` (`d26196f2…`) applied unchanged and committed as `bfd008a` (`src/eval/params.rs`, `src/eval/material.rs`); `bench 13` **14,291,377 / EBF 2.538** on the committed tree, equal to the fit's.
- **Pre-game reads (2026-10-13, `run_screens.sh`, every step exit 0; host idle):**
  - *Reproduction:* a tuner build of the head tree (worktree at `f634ced`) dumped validation byte-identical to RAR-E57's and RAR-E58's dumps (played and full), and its substituted scores file is byte-identical to `c71/scores-9587-head.csv`.
  - *(a) Donor screen,* head → candidate over all rows: material +0.081 → +0.071, pawns +0.021 → +0.024, pieces and queens +0.038 → +0.033, mobility +0.043 → +0.037, king +0.077 → +0.076, threats +0.042 → +0.052, passed +0.004 → +0.002, space +0.024 → −0.004, initiative +0.012 → +0.011. Every family moves by 0.028 points or less.
  - *(c) Magnitude:* **no flag.** The candidate's played score differs on 95.8% of rows. The shift toward the sign is +0.3, +1.3 and +2.8 cp in the bands up to 600, +3.3 and +8.7 above. The share above the gate is 24.57% → 24.73% (+0.16 points).
  - *(d) Tree read:* **no flag.** Total depth-12 nodes 11,707,547 → 12,102,045 (+3.4%), per-position geometric mean 1.004 (median 1.022; 34 grew and 31 shrank by more than 5%); `tune,diag` builds reproduce 14,291,377 and 14,197,472.
  - *(e) Pooled-PGO NPS:* **not read.** Both two-cycle reads were disturbed and the tool says to repeat rather than interpret (`nps/read.log`, `nps/read2.log`; deltas +0.62% and +0.09%, intervals about ±2.5%). The host was in interactive use (a browser, Task Manager), bursting past the tool's cycle-start check at 10–13% CPU. By AGENTS a busy host is not measured, so the read goes to the maintainer on an idle machine, from the pools already built. The stop rule (below −1.5%) is read before gate 1.
    - **Read by the maintainer, 2026-10-13 21:49–21:56** (`nps/read3/read.txt`, `raw.json` `3498ab16…`): no disturbed cycles. Levels 2,430,577 (head) and 2,418,864 (candidate), **−0.48%, 95% interval −0.93% to −0.04%**; the tool's verdict is "closed NO_CHANGE by step 1".
    - The stop rule does not fire, so gate 1 may start. At about 2 Elo per 1% at `3+0.03`, the slowdown is worth about −1 Elo inside the gate's result. It is a slightly larger tree (+3.4%) at nearly the same rate, not a cost in the evaluation's code.
- **Gate 1 command** (dry run passed 2026-10-13, `tools/results/colosseum_sprt_rar-e60-gate1_20261013_214729.dry-run.json`: idle host at 4%, pinned runner `colosseum-cli 0.2.0`, both fingerprints, flavour and compiler equality, policy in every checked field), from PowerShell, after the NPS read clears its stop: `./tools/colosseum.ps1 -Mode sprt -Bracket default -EngineA tools/test_engines/rarog-c8cycle1-pext-pgo.exe -EngineB tools/test_engines/rarog-c8head-pext-pgo.exe -NameA c8cycle1 -NameB c8head -MaxPairs 20000 -Seed 4203 -ExpectBench 14291377,14197472 -Dir tools/results/rar-e60-gate1`. Candidate arm `rarog-c8cycle1-pext-pgo.exe` (`76CD1F3C…`, built at `bfd008a`, clean, bench 14,291,377); head arm C.8.1's `rarog-c8head-pext-pgo.exe` (`E73E3230…`).
- **Gate 1 (played 2026-10-13 22:00–23:16 local, by the maintainer; `tools/results/rar-e60-gate1`, manifest `colosseum_sprt_rar-e60-gate1_20261013_220004.manifest.txt`):**
  - *Conditions, checked from the manifest:* the registered arms (`76CD1F3C…` at `bfd008a`, bench 14,291,377; `E73E3230…` at `c72b41c`, bench 14,197,472; both clean), the pinned runner (`colosseum-cli 0.2.0`, `5A4ECEDB…` at `ca05dfa`), UHO seed 4203, no adjudication; host 5.0% busy with no waiver; repository at `4ae41f9`.
  - **H1 accepted** at 3,601 pairs (7,202 games): **+11.0 ± 4.9 Elo, +18.1 ± 8.0 nElo** (a first-crossing estimate, inflated by the early stop), W/D/L 1,920 / 3,589 / 1,693, pentanomial [99, 794, 1,614, 969, 125], LLR +2.95 against ±2.94.
  - No faults (time 0–0, other 0–0, of 36 allowed). 15 post-terminal pairs are kept as evidence and excluded from the sample.
- **Disposition:** **accepted.** Gate 1 passed and no screen flagged, so by the frozen use it accepts.
  - The fingerprint **14,291,377 / EBF 2.538** becomes the development fingerprint on branch `c7`.
  - PROCESS step 4's question goes to the maintainer: whether a free night makes the optional margin block worth running anyway, and how its margins are accepted if it runs.
  - C.8.3 owes the `10+0.1` direction read (step 6), registered below.
  - The frozen test was consumed by the initialization control, as registered.
  - Cycle 2 of C.8 opens only after C.8.3 closes, and its regeneration waits on C.8.4 (datagen on Colosseum).
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 6 above.
  - Observed:
    - (1) Coordinate passes converged in both arms, hit; N's validation loss above A's, hit (0.09530 against 0.09315). N's linear and polish stages also ran to their budget.
    - (2) A is the candidate, hit; A below D at two clustered SE, hit at the boundary (2.18).
    - (3) hit.
    - (4) hit (0.028 points the largest move).
    - (5) magnitude, tree and NPS, all hit (no flag, +3.4%, −0.48%).
    - (6) **miss in magnitude**: gate 1 crossed H1 at +11.0 ± 4.9 Elo against a prediction of +2 with an 80% band to +8, and the pass (probability 0.3) happened.
  - Which assumption failed:
    - I sized the gain from the frozen test's static improvement (+0.08% of loss) and from RAR-M68's unchanged label quality, so I expected little.
    - Games say the refit's value lies where neither measure looks. The likeliest candidates are positions drawn from the current engine's own play, which a frozen test of the same distribution cannot show as a large loss drop, and coefficients the C units fitted one at a time on `hce-v4-tb` being set jointly here. The mechanism is unmeasured.
    - Part of the size is the early stop: first-crossing estimates overstate.
    - The miss is in magnitude and in the instrument I sized it with: static loss has no exchange rate to Elo, as AGENTS says, and I leaned on one anyway.
- **Conditional lesson:** a refit on regenerated data is not sized by its static frozen-test gain; register a regeneration's prediction from earlier regenerations' game results (RAR-E12 +11.8, RAR-E24), not from loss.
- **Maintainer decision 2026-10-14: the optional margin block is skipped** (PROCESS step 4); the acceptance rests on gate 1 and the `10+0.1` read.
- **C.8.3 registration (2026-10-14, before any `10+0.1` game):**
  - **The `10+0.1` direction read (PROCESS step 6):** 1,000 games, `match-fixed-ltc.toml`, the gate-1 arms, UHO seed 4204, no adjudication.
    - Use, frozen: a point estimate below 0 Elo reopens the cycle (C.8 back to `RESEARCH`, and the maintainer decides). 0 or above leaves the acceptance standing, and with the margin-block question answered, C.8.3 closes. It accepts nothing on its own.
  - **Prediction:** **+7 Elo**, 80% band [−5, +19]; probability of 0 or above 0.8. Basis: gate 1's +11.0 discounted for its early stop. The earlier corpus refit held its size at `10+0.1` (RAR-E24, +35.9 then +39.1).
  - **Command,** from PowerShell: `./tools/colosseum.ps1 -Mode match -RunFile tools/colosseum/match-fixed-ltc.toml -EngineA tools/test_engines/rarog-c8cycle1-pext-pgo.exe -EngineB tools/test_engines/rarog-c8head-pext-pgo.exe -NameA c8cycle1 -NameB c8head -Games 1000 -BaseMs 10000 -IncrementMs 100 -Seed 4204 -ExpectBench 14291377,14197472 -Dir tools/results/rar-e60-ltc`.
    - Its dry run on 2026-10-14 was refused by the harness's idle check, because a Basilisk tune was running (`analysis/artifacts/c82/dryrun-ltc.log`). The dry run is owed on an idle host before the read.
    - Dry run passed 2026-10-14 12:47 local, by the maintainer (`tools/results/colosseum_match_rar-e60-ltc_20261014_124748.dry-run.json`): idle host at 2%, pinned runner, both fingerprints, flavour and compiler equality, policy in every checked field.
- **`10+0.1` read (played 2026-10-14 12:48–13:24 local, by the maintainer; `tools/results/rar-e60-ltc`, manifest `colosseum_match_rar-e60-ltc_20261014_124846.manifest.txt`):**
  - *Conditions, checked from the manifest:* the registered arms (`76CD1F3C…` at `bfd008a`, bench 14,291,377; `E73E3230…` at `c72b41c`, bench 14,197,472; both clean, and the files' SHA-256 recomputed equal), the pinned runner (`colosseum-cli 0.2.0`, `5A4ECEDB…` at `ca05dfa`), 10000+100 ms, UHO seed 4204, no adjudication; host 3.4% busy with no waiver; repository clean at `5288fdb`.
  - **+17.0 ± 11.8 Elo, +31.2 ± 21.5 nElo**, W/D/L 255 / 539 / 206, pentanomial [12, 83, 257, 140, 8]; 1,000 games, every termination normal, no fault. An independent recount of `games.pgn` gives the same W/D/L and +17.0 Elo.
  - By the frozen use the acceptance stands and C.8.3 closes. Cycle 2 opens after C.8.4.
  - *Calibration:* +17.0 against the predicted +7 (80% band [−5, +19]): inside the band, near its top. The 0-or-above event (probability 0.8) happened. Sign right; magnitude larger than the discount for gate 1's early stop assumed, so the refit holds at least its STC size at the longer control, as RAR-E24 did.
- **Artifacts:** `analysis/artifacts/c82/` (ignored storage): `run_fits.sh`, `fits-status.txt`, `neutral.txt`, `defaults.txt`, `fitA.log`, `fitN.log`, `fitA2.log` (the refused fresh fit), `fitA-resume.log`, `compare.log`, `test-A.csv`, `test-N.csv`, `test-D.csv`, `provenance_split.py`, `provenance-test.csv`, `init_control.py`, `init_control.json`, `kfix-*.log`, `run_screens.sh`, `screens-status.txt`, the dumps and reports, `magnitude.txt`, `tree_d12.txt`, `run_nps.sh`, `nps/`, `nps-status.txt`, `build-gate-cand.log`, `dryrun-gate1.log`; the fits in `tools/results/hce-fit-20261013_210320/` (with `resume-20261013_211626/`) and `tools/results/hce-fit-20261013_210532/`.
