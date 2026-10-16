# RAR-E68 — C.13.6 follow-up: is RAR-E67's loss the score size the frozen search sees, or the teacher?

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-15, before any game of this arm. The run is the maintainer's overnight command.
- **Kind:** a diagnostic, fixed-length read. It accepts nothing and changes no head. It decides only whether the blended target is carried to C.10's joint tune.
- **Arms:**
  - *Scaled blend:* `tools/test_engines/rarog-c136blendx-pext-pgo.exe` (`8D9880E0…`), built clean at `502a7df` and reverted in `106439f`; bench **12,820,980 / EBF 2.555**. It is RAR-E67's blended-target weights, with the evaluation's output multiplied by 10183/10000 after the endgame scale.
  - *Head:* `rarog-c13best-pext-pgo.exe` (`3C92DC47…`, `931dcca`, **10,521,141**), RAR-E67's head arm.
- **Why 1.0183:** it is the ratio of the head's to the blended candidate's mean |played score| over RAR-E67's 194,444 screen rows (374.27 / 367.55), fixed from data already read before this registration.
  - The band ratios run from 1.013 to 1.044; one global factor was chosen so the arm restores size without reshaping.
  - Scaling the output scales every score uniformly (the linear terms, the king-danger map and the mop-up) and leaves the king-danger index and the lazy gate's decision unchanged.
- **Research question:** RAR-E67's blended refit lost −13.0 ± 5.7 Elo while predicting outcomes as well. Is the loss the frozen search's cp margins seeing scores about 2% smaller (competing hypothesis 2), or the blend teaching worse values (hypothesis 1, the cycle-1 search's errors as the target)?
- **Read (frozen):** Colosseum `-Mode match` (`match-fixed.toml`): **4,000 games**, `3+0.03`, one thread, Hash 64, the UHO book in random order, seed 4208, no adjudication. At this size the 95% interval is about ±6.5 Elo.
- **Use, frozen** (on the point estimate in Elo):
  - **≥ −4** (at least 9 of the 13 recovered): score size explains most of the loss. The blend is carried to C.10 as a candidate target, fitted with the margins there, and PLAN records it.
  - **≤ −9:** the teacher is the problem. The blend stays closed until a changed datagen budget or label source.
  - **Between:** inconclusive, recorded. C.10's registration decides whether to include the blend, at the maintainer's choice.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  - **−8 Elo**, 80% band [−15, +1].
  - Probability of ≥ −4: 0.3. Probability of ≤ −9: 0.45.
  - Basis: a 2% size change is small for the cp margins, so most of the loss should remain.
- **Result (played 2026-10-15, 23:02–23:44 local, by the maintainer's overnight command; `tools/results/rar-e68`, manifest `colosseum_match_rar-e68_20261015_230231.manifest.txt`):**
  - *Conditions, checked from the manifest:*
    - the registered arms and fingerprints (`8D9880E0…` at `502a7df`, 12,820,980; `3C92DC47…` at `931dcca`, 10,521,141);
    - the pinned runner, UHO seed 4208, no adjudication;
    - the host 9.2% busy with no waiver, and the repository clean at `81d4cda`.
  - **−6.3 ± 6.6 Elo, −10.3 ± 10.8 nElo** over 4,000 games: W/D/L 961 / 2,005 / 1,034, pentanomial [70, 528, 862, 485, 55]. No faults, every termination normal. `colosseum_recount.py` agrees exactly.
- **Disposition:** by the frozen use the point estimate is between −9 and −4: **inconclusive.**
  - Restoring the score size moved the blend from RAR-E67's −13.0 ± 5.7 to −6.3 ± 6.6. That is about half the loss, but the two intervals overlap; the difference is about 6.7 ± 8.7 Elo. So the read neither shows that size explains the loss nor rules it out.
  - The blend stays out of the head. Whether C.10's joint tune includes it as a candidate target is decided in C.10's registration, at the maintainer's choice, as registered.
  - Even scaled, the blend measured no gain over the head.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): −8 Elo, 80% band [−15, +1]; probability of ≥ −4 0.3, of ≤ −9 0.45.
  - Observed: −6.3, inside the band, about 2 Elo above its centre. Neither decisive outcome happened (combined probability 0.75): the read landed in the inconclusive middle, which I gave 0.25.
  - The assumption that a 2% size change is too small to matter is weakened but not refuted: some of the loss may be size, and the rest is not explained. The miss is in confidence; the instrument's power at 4,000 games was the limit.
- **Retry trigger:** C.10's registration, if it includes the blend; otherwise as RAR-E67 (a changed datagen budget or label source).
- **Artifacts:** `tools/results/rar-e68`, `analysis/artifacts/c136/rar-e68.log`, `recount-rar-e68.log`.
