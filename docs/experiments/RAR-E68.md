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
- **Result:** pending.
