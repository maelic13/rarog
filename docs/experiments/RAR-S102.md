# RAR-S102 — C.10.3.1: `CoreLmrResearchShallower` read as a switch, two 2,000-game reads

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-19, before any game. The run is the maintainer's command, or the review session's if the maintainer delegates the night's work (about 50 minutes).
- **Kind:** two categorical reads that fix one mechanism's state before C.10's pilot and blocks (PROCESS, *SPSA go/no-go*, step 3; RAR-S100's form). They accept nothing on their own.
- **Why:** the reduced-move re-search goes one ply shallower when the move beat alpha by less than this margin (`src/search/node.rs`, the LMR re-search). The condition needs the margin to be at least 2 to fire at all, because the move has just beaten alpha and the best score so far is at most alpha. The donor's converted seed was 4 (Reckless: `score < best_score + 9`, live); RAR-S75's tune walked it to −3 and RAR-S78's restart to −1, so the mechanism has been off since B.2's fit, and every later tune carried a coordinate that could not move the engine. C.10.3's relevance check found it (`bench 13` unchanged from −50 to +1) and removed it from the pilot. On or off is a categorical question the tunes answered only by drifting; this read asks it in games.
- **Binary:** one tune build on both sides, `tools/test_engines/rarog-c103-tune.exe` (SHA-256 `784FB228…`, `b7bc142`, `pext-tune`, bench **10,521,141**), `-CategoricalTuneBuild`; the option's range is `-50..=50`.
- **Wire** (`analysis/artifacts/c10/bench_opt.py` on this build, 2026-10-19): `CoreLmrResearchShallower=-1` reproduces 10,521,141 / EBF 2.521; `=2` gives 6,761,349 / 2.449 and `=4` gives 8,967,538 / 2.457, so the mechanism is live at both read values and shrinks the fixed-depth tree. A wire check, not a forecast: fewer nodes to depth 13 says nothing about strength (RAR-S80's calibration).
- **Reads (frozen):** Colosseum `match`, `match-fixed`, 2,000 games each, `3+0.03`, Hash 64, one thread, the UHO book in random order, no adjudication; side A carries the option, side B the default (−1, off).
  - **Read 1:** `CoreLmrResearchShallower=2`, the smallest live value, seed **20261022**, `tools/results/c103-shallower2`.
  - **Read 2:** `CoreLmrResearchShallower=4`, the donor's converted seed, seed **20261023**, `tools/results/c103-shallower4`.
  - Both run from one wrapper, `analysis/artifacts/c103/reads_s102.ps1` (RAR-S100's with the names, values, seeds and directories changed), which waits for an idle host before each read; it is executed once with `-DryRun` before the live run.
- **Rule (frozen, RAR-S80's):** a read of **+5 Elo or more with its 95% lower bound above −5** is a direction. A direction fixes the mechanism **on**: the coordinate enters C.10's pilot and blocks with its range floored at 2 and its seed at the winning value (the larger read if both qualify), so the tune can only size it, never switch it off. With no direction the mechanism stays off at −1, the coordinate stays out of the surface, and the mechanism is recorded as a removal candidate for after Phase C's freeze (a simplification, its own registration under the loss-permitting bracket). The pilot's registration (RAR-S101) is amended with the outcome before it is relaunched.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  - 2 reads −3 ± 10 Elo; a direction 0.15; −5 or worse 0.35.
  - 4 reads −2 ± 10 Elo; a direction 0.15; −5 or worse 0.30.
  - Neither finds a direction (0.72).
  - Low confidence: the only local evidence is that two tunes walked the value below 2 at 150,000-game horizons, which is movement, not an Elo read (B.6: movement is not an Elo proxy), and the fixed-depth tree shrinks 36% and 15%, which has pointed both ways before (RAR-S80, RAR-S86).
- **Cheapest prior falsifier:** these reads.
- **Result:** pending.
- **Disposition:** pending.
- **PREDICTION CALIBRATION (append after exposure):** pending.
- **Artifacts / commits:** this registration; `analysis/artifacts/c103/reads_s102.ps1` (`754f9544…`; dry run 2026-10-19 22:05, both reads resolved on an idle host, the option on side A only); the run directories and manifests named above.
