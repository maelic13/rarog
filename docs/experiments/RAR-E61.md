# RAR-E61 — C.8.6 refit cycle 2 on `hce-v6-tb`: the fit from the accepted vector, static screens, tree and NPS reads, gate 1 `[0,3]`

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-14, before the fit, any screen and any game. The fit, the screens, the tree read and the NPS read are agent-run; the games are the maintainer's.
- **Baseline SHA / candidate SHA / dirty-diff hash:**
  - *Baseline:* the accepted cycle-1 head, `bfd008a` (**14,291,377 / EBF 2.538**). The engine source is unchanged on `c7` since, through this registration's commit.
  - *Candidate:* the fit's baked vector, committed on `c7` after this registration as an engine commit of `src/eval/params.rs` and `src/eval/material.rs` only. Its fingerprint is recorded below before any screen.
  - No structural change and no `src/search/` change: this is the same surface refitted on the cycle-2 corpus.
- **Binary / compiler / PGO identity:**
  - `rustc 1.98.1`, the toolchain of the baseline arm.
  - The fit uses `tools/texel-tuner`, built by the fit driver at its tree. The screen builds are tuner and `tune,diag` builds of each arm's own tree.
  - The gate arms are `build_test.ps1` PGO builds from clean trees. The head arm is `tools/test_engines/rarog-c8cycle1-pext-pgo.exe` (`76CD1F3C…5DA7`, bench 14,291,377), the arm RAR-E60 accepted and C.8.5's labeller.
- **Research question:** does a second refit of the unchanged surface, on `hce-v6-tb` (labelled by the cycle-1 head), make the engine stronger at `3+0.03`, search frozen (PLAN C.8, cycle 2)?
- **Hypothesis / proposed mechanism:** the cycle-1 refit's value lay in positions from the current engine's own play and in coefficients set jointly (RAR-E60's calibration). A corpus from the refitted engine moves the positions again, so part of that gain repeats, smaller, because the labeller changed less this time.
- **Competing hypotheses:**
  1. *Nothing to gain:* RAR-M69 finds the labels as good as `hce-v5`'s, and the labeller differs only by the refit. The coefficients settle where they were, and gate 1 reads near zero.
  2. *Magnitude drift:* the second refit moves magnitudes the frozen search's margins rely on; the tree read flags, and the margin block (C.8.7) decides.
  3. *Overfitting to the labeller's own play:* a corpus labelled by the engine being fitted rewards its own habits. Static loss improves while games do not, or regress.
- **Interacting mechanisms / consumers:**
  - Every cp-valued margin and the correction history (through the score), and `LazyMargin` (the gate's share moves with magnitudes).
  - The eval and pawn caches (pure functions).
  - RAR-E58 and RAR-E59's phase-dependent calibration of large scores, re-read only through the magnitude screen.
- **Invariants and how they are tested:** the fit driver's own gates.
  - The trace reconstructs the baseline and the final vector exactly.
  - The baked candidate passes the debug and release suites and clippy.
  - The source restores byte for byte, and the candidate patch applies.
- **Registered order (frozen):**
  - **Fit** (`analysis/artifacts/c86/`), from a clean tree at this registration's commit: `pwsh -NoProfile -File tools/texel/fit_complete.ps1 -DatasetDir tools/texel/data/hce-v6-tb -TargetTrain 3500000 -BaselineFingerprint "14,291,377 / EBF 2.538" -DeferFrozenTest RAR-E61`.
    - The script's defaults hold: manifest v5, 200,000 coordinate positions, 40 / 200 / 40 / 60 epochs, `lr` 0.3, L2 1e-7, K pinned from the defaults on validation.
    - There is no initialization control; C.8 runs it in the first cycle only.
  - **Frozen test:** unread under this ID. It is read once, through `frozen-test.RAR-E61.opened` with the command the fit's `summary.json` records, after the cycle's last game and only if gate 1 or gate 2 fails, as a diagnosis. It decides nothing.
  - **The candidate** is applied from the fit's `candidate-eval.patch` and committed on `c7` as an engine commit. Its `bench 13` is recorded before any screen and must equal the fit's `bench-candidate`.
  - **Pre-game reads, in order** (into `analysis/artifacts/c86/`), as RAR-E60 read them:
    - (a) *The donor screen:* on `hce-v4-tb` validation, where RAR-E23's donor rows are. The head side is the cycle-1 head's played score, dumped first by a tuner build of `bfd008a`'s tree. The candidate's played score is then substituted, and `donor_residual.py analyse --within` runs on both.
    - (c) *Magnitude:* `analysis/artifacts/c2-screens/magnitude.py` over the played and `--full` dumps of both arms.
    - (d) *Tree read:* `analysis/artifacts/c03-lazy/positions_v1.epd`'s 72 positions at depth 12, with a cleared table and `tune,diag` builds of both arms reproducing their `bench 13`, read with `analysis/artifacts/c2-screens/tree_read.py`.
    - (e) *Pooled-PGO NPS:* PROCESS's method, with four clean `pext` PGO builds per arm (`nps_build_pool.ps1`), two cycles, on an idle host.
  - **Flags** (as RAR-E42 and RAR-E60) send the candidate to C.8.7's margin block after gate 1, whatever gate 1 reads, and reject nothing:
    - a magnitude band at or below 600 shifting by more than 20 cp toward the sign;
    - the share above the gate moving by 2 points or more;
    - the tree read's total nodes changing by more than 5% either way, or its geometric-mean ratio leaving 0.95 to 1.05.
  - **Stops before any game:** a void reproduction, or an NPS point estimate below −1.5% (the handover then waits for the maintainer).
  - **Runner checks before gate 1** (PROCESS, *Harness*; the Colosseum 0.3.0 re-pin): the backup-path cross-check and the null pair, as C.8.7 sets them out. They are the maintainer's runs and judge the instrument, not the candidate.
  - **Gate 1:**
    - Colosseum (`tools/colosseum.ps1 -Mode sprt -Bracket default`): pentanomial GSPRT `[0,3]` nElo, `3+0.03`, one thread, Hash 64, the UHO book in random order, no adjudication, cap **20,000 pairs**.
    - Cap check (RAR-M10, prospectively): +4 Elo is about +6.6 nElo at cycle 1's ratio of Elo to nElo. The drift per game is 8.3e-6 × 3 × (6.6 − 1.5) ≈ 1.3e-4, so about 23,000 games, or 11,600 pairs, reach the upper bound, inside the cap.
    - The runner's fault policy applies by reference (PROCESS, *Harness*). Bounds, cap, book and adjudication never change after games.
  - **Acceptance and use:**
    - Gate 1 passing with no flag accepts, and PROCESS step 4's question about the optional margin block goes to the maintainer. Otherwise C.8.7 decides: the margin block, gate 2 and the `10+0.1` read.
    - A rejected cycle is reverted on `c7` in its own commits, and C.8 stops at the first non-accepting cycle.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Fit:* every coordinate pass converges (0.8). The polish's best validation loss is below the baseline vector's on `hce-v6-tb` validation (0.9).
  2. *Candidate:* `bench 13` changes (0.99).
  3. *Donor screen:* every family within 0.03 points of the head over all rows (0.75).
  4. *Magnitude:* no flag (0.75). *Tree read:* within ±5% and inside 0.95 to 1.05 (0.65). *NPS:* within ±0.5% of the head (0.7).
  5. *Gate 1:* **+4 Elo**, 80% band [−3, +11]; probability of passing 0.35. Basis: RAR-E60's lesson, sized from regenerations' game results rather than static loss. Cycle 1 measured +11.0 at gate 1 (a first-crossing estimate) and +17.0 at `10+0.1`. This cycle's labeller differs from the last only by that refit, and RAR-M69 finds its labels no better, so the gain should be a fraction of cycle 1's.
  - Most likely failure mode: competing hypothesis 1, gate 1 at the cap or at H0.
- **Falsification criteria:** gate 1 and gate 2 both fail: the second refit is no better, and C.8 stops at cycle 2.
- **Cheapest prior falsifier:** the static screens (no game).
- **Fit (2026-10-14, agent-run; `tools/results/hce-fit-20261014_212116`, from `c0c4c45` clean):**
  - The first run fitted and baked, then stopped on the candidate's debug tests. `knnkp_stays_above_its_one_capture_neighbours` failed on `8/5N2/2N4K/3k4/8/8/6p1/8 w` and its mirror. The refit lifted that position's generic score to 20, which activated a comparison against the pawn-gone draw (0), and the KNN-KP adjustment scores it 0.
    - Syzygy calls the position a draw: WDL 0, only Ne7+ and Nb4+ hold, and every other move loses. The 0 is therefore correct.
    - Under the candidate's weights these were the test's only failing comparisons.
    - The test's precondition was fixed in its own commit, `fd6bb30`: it now excludes a pawn one push from a promotion square that no knight holds or covers. The test then passes under the head's weights (11,838 active comparisons) and the candidate's (11,914), against its floor of 9,000. Bench is unchanged at 14,291,377, and fmt, clippy and the debug and release tests pass.
    - The fit then resumed from its recorded run (`resume-20261014_213007`) and completed.
  - Both coordinate passes converged (15 and 14 epochs). K is 1.6166. The polish's best validation epoch was 39 of 60 (0.09265154), against 0.09274854 for the baseline vector, so validation loss fell by 0.10%.
  - The frozen test is unread, deferred to this ID as registered.
- **Candidate:** `75c3db5`, the fit's `candidate-eval.patch` (`5528C4D0…`, as `summary.json` records), an engine commit of `src/eval/params.rs` and `src/eval/material.rs` only.
  - **Bench 14,126,183 / EBF 2.557**, equal to the fit's `bench-candidate`.
  - fmt, clippy (all features and targets, 0 warnings) and the debug (395) and release (396) tests pass; the two counts differ as they did at `bfd008a`.
- **Pre-game reads (2026-10-14, `analysis/artifacts/c86/run_screens.sh`, `screens-status.txt`):** every reproduction passed. The head side's played, full and substituted dumps equal RAR-E60's candidate dumps byte for byte, and both `tune,diag` builds reproduce their `bench 13`.
  - *(a) Donor screen,* head → candidate over all rows, RAR-E60's family set: material +0.071 → +0.032 (−0.039), passed +0.002 → +0.038 (+0.037), space −0.004 → +0.024, initiative +0.011 → +0.027, king +0.076 → +0.062, mobility +0.037 → +0.027, threats +0.052 → +0.043, pieces and queens +0.033 → +0.023, pawns +0.024 → +0.018. The largest move is 0.039 points. The donor's whole score adds +1.979 → +1.916 (all families together +1.131 → +1.139).
  - *(c) Magnitude:* **no flag.** The largest shift toward the sign at or below 600 cp is −5.2 cp (101–300), and the share above the gate moves +0.16 points (24.73% → 24.89%).
  - *(d) Tree read:* **no flag.** Total depth-12 nodes 12,102,045 → 11,714,307 (−3.2%); per-position geometric mean 1.007 (median 1.049; 35 grew and 31 shrank by more than 5%). The endgame band grows +13.2% and the middlegame bands shrink.
  - *(e) Pooled-PGO NPS* (`c86/nps/read.log`, against RAR-E60's four `bfd008a` builds): levels 2,449,533 (head) and 2,427,104 (candidate), **−0.92%, 95% interval −1.26% to −0.57%**, no disturbed cycle, idle host. The stop rule (below −1.5%) does not fire. At about 2 Elo per 1%, that is about −2 Elo inside the gate's result.
- **Gate 1 command** (dry run passed 2026-10-14, `tools/results/colosseum_sprt_rar-e61-gate1_20261014_215250.dry-run.json`: idle host, the pinned runner `colosseum-cli 0.3.0`, both fingerprints, flavour and compiler equality, policy in every checked field), from PowerShell, **after RAR-M70's runner checks pass**: `./tools/colosseum.ps1 -Mode sprt -Bracket default -EngineA tools/test_engines/rarog-c8cycle2-pext-pgo.exe -EngineB tools/test_engines/rarog-c8cycle1-pext-pgo.exe -NameA c8cycle2 -NameB c8cycle1 -MaxPairs 20000 -Seed 4205 -ExpectBench 14126183,14291377 -Dir tools/results/rar-e61-gate1`.
  - Candidate arm: `rarog-c8cycle2-pext-pgo.exe` (`A3A11321…`, built clean at `75c3db5`, bench 14,126,183). Head arm: `rarog-c8cycle1-pext-pgo.exe` (`76CD1F3C…`).
- **PREDICTION CALIBRATION (the reads now, gate 1 after its games):**
  - Original prediction (not rewritten): items 1 to 5 above.
  - (1) Hit: both passes converged, and validation loss fell below the baseline's.
  - (2) Hit.
  - (3) **Miss in magnitude:** the largest family move is 0.039 points against the predicted 0.03. Material (−0.039) and passed (+0.037) move most, in opposite directions. The refit takes more of the donor's material information and gives up some of its passed-pawn share.
  - (4) Magnitude and tree read hit. **NPS is a miss:** −0.92% against ±0.5%. The same arm pair at C.8.2 read −0.48%, and a smaller tree does not bring a faster rate.
- **Gate 1 (played 2026-10-14 22:01 to 2026-10-15 00:12 UTC, by the maintainer, after RAR-M70's cross-check; `tools/results/rar-e61-gate1`, manifest `colosseum_sprt_rar-e61-gate1_20261015_000150.manifest.txt`):**
  - *Conditions, checked from the manifest:* the registered arms (`A3A11321…` at `75c3db5`, bench 14,126,183; `76CD1F3C…` at `bfd008a`, bench 14,291,377), the pinned runner (`colosseum-cli 0.3.0`, `381EB7A5…` at `cb12cd3`), UHO seed 4205, no adjudication. The host was 0.4% busy with no waiver, and the repository was clean at `5da823e`.
  - **H1 accepted** at 6,181 pairs (12,362 games): **+6.7 ± 3.7 Elo, +11.1 ± 6.1 nElo** (a first-crossing estimate), W/D/L 3,285 / 6,030 / 3,047, pentanomial [175, 1,393, 2,826, 1,593, 194], LLR +2.96 against ±2.94.
  - No faults (time 0–0, other 0–0, of 61 allowed), and the recount agrees. 13 post-terminal pairs are kept as evidence and excluded from the sample.
- **Disposition:** **accepted.** Gate 1 passed and no screen flagged, so by the frozen use it accepts.
  - The fingerprint **14,126,183 / EBF 2.557** becomes the development fingerprint on branch `c7`.
  - PROCESS step 4's question goes to the maintainer: whether the optional margin block runs.
  - C.8.7 owes the `10+0.1` direction read (step 6), registered below.
  - The frozen test stays unread, since no gate failed.
  - A third cycle of C.8 opens after C.8.7 closes.
- **C.8.7 registration (2026-10-15, before any `10+0.1` game):**
  - **The `10+0.1` direction read (PROCESS step 6):** 1,000 games, `match-fixed-ltc.toml`, the gate-1 arms, UHO seed 4206, no adjudication.
    - Use, frozen: a point estimate below 0 Elo reopens the cycle (C.8 back to `RESEARCH`, and the maintainer decides). 0 or above leaves the acceptance standing, and once the margin-block question is answered, C.8.7 closes. The read accepts nothing on its own.
  - **Prediction:** **+5 Elo**, 80% band [−6, +16]; probability of 0 or above 0.75. Basis: gate 1's +6.7, discounted for its early stop; cycle 1 held its size at `10+0.1` (+11.0 at gate 1, then +17.0).
  - **Command,** from PowerShell: `./tools/colosseum.ps1 -Mode match -RunFile tools/colosseum/match-fixed-ltc.toml -EngineA tools/test_engines/rarog-c8cycle2-pext-pgo.exe -EngineB tools/test_engines/rarog-c8cycle1-pext-pgo.exe -NameA c8cycle2 -NameB c8cycle1 -Games 1000 -BaseMs 10000 -IncrementMs 100 -Seed 4206 -ExpectBench 14126183,14291377 -Dir tools/results/rar-e61-ltc`.
    - Its dry run is owed on an idle host. A Basilisk tune held the host when this was registered.
- **Gate-1 calibration (appended after exposure):** observed +6.7 Elo against the predicted +4 (80% band [−3, +11]): inside the band. The pass (probability 0.35) happened. The sign was right, and the magnitude is above the central value, partly from the early stop. The NPS cost (−0.92%, about −2 Elo) is inside the read. Competing hypothesis 1 (nothing to gain) is refuted at gate 1. Hypothesis 3 (overfitting to the labeller's own play) is not supported in games.
