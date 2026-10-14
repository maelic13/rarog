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
- **Result:** pending; the fit has not run.
