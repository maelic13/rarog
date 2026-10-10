# RAR-E28 — C.5.2 opposite-bishop scale, seven men or more: the pure rule refitted where no tablebase reaches, the head's rule kept at six or fewer

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-10, before the restricted fit was computed; agent-run (zero games, static evaluation only). Maintainer decisions 2026-10-10: RAR-E27 failed, so C.5.2 takes the seven-men split now, with the boundary hazard registered. The proper fix is C.5.2.1, after C.4.
- **Baseline SHA / candidate SHA / dirty-diff hash:** branch `c4` at `5f108fb`; the evaluation is `master`'s `cc13320` (13,187,295 / EBF 2.546). No candidate binary yet: the rule is applied offline to RAR-E26's dumps.
- **Binary / compiler / PGO identity:** RAR-E26's: `rarog-texel --dump-scores ... --scale` over `hce-v4-tb` train and validation (`train_scale.csv` `b0453954…c41d`, `val_scale.csv` `54b6018c…63bb`, in `analysis/artifacts/c51-ocb/`), K fixed at 1.54638.
- **Research question:** in pure opposite-bishop endings of seven men or more (three pawns or more, where Syzygy cannot check the result and RAR-E27's conversion loss cannot reach), does a refit of the rule's constants still gain held-out outcome loss, so that C.5.2 can build it with the head's rule kept at six men or fewer?
- **Hypothesis / proposed mechanism:** RAR-E26's mechanism (the head leaves 70% of pure rows unscaled while 67% of them are draws) does not depend on the six-man rows. Fitted on the 61,722 training rows with three pawns or more, the rule gains held-out loss on the 3,460 held-out ones.
- **Competing hypotheses:** (1) *the six-man rows carried it:* RAR-E26's gain was mostly the one-to-two-pawn rows, and seven-plus rows gain little alone. (2) *conversion at seven plus:* the same flattening that cost KBPP-KB wins costs wins at seven men or more as well. No tablebase can show it, so it is left to the gate's games.
- **Known exposure:** RAR-E26's post-hoc breakdown of these held-out rows is known: under RAR-E26's all-men candidate, pawns 4–5 gained +15.8%, 6–7 +13.3%, 8–9 +4.1%, 10 or more +7.4%, and 2–3 +35.7% (that group mixes six and seven men). The prediction below uses it. What this registration freezes is the restricted fit, the rule and its use, all before the fit is computed.
- **Registered hazard (maintainer decision 2026-10-10):** the split makes the score jump at the boundary. A pure ending with three pawns scaled by the fitted rule (about 16 to 26 of 48 for RAR-E26's constants) roughly doubles its score on one pawn exchange into KBP-KB (`s = 40`) or KBPP-KB (`s = 48`). That gives the search an incentive to trade down into the overclaimed six-man endings. It is read in C.5.2's gate: the share of endgame-start games that reach a pure opposite-bishop ending of six men or fewer, and those games' draw rate, per arm. The proper fix, one rule continuous across the boundary with position knowledge that keeps the six-man wins, is C.5.2.1.
- **Interacting mechanisms / consumers:** as RAR-E26 (the evaluation fitted with the rule fixed; the search's damping; the eval cache and TT store the scaled score, which stays a pure function of the position). In addition, the boundary incentive above, which acts through move choice (exchanges) and not through any shared signal.
- **PRE-REGISTERED PREDICTION (freeze before exposure):** gains in percent of the head rule's held-out squared error in the pure seven-plus cohort, paired by row, ± one standard error; F1 only (`s = clamp(a + b·pawns + c·passers, 0, 48)`, passers of both sides; RAR-E26 found no support for the strong-side form); the same integer grid.
  1. **+13%**, 80% band [+7, +20]; probability of at least +1.0% at three standard errors 0.9.
  2. The fitted per-pawn step `b` is at most 3 and the per-passer step `c` at least 6 (probability 0.7).
  3. Hazard read, in the gate: the candidate's share of games reaching a pure ending of six men or fewer exceeds the head's (probability 0.6). Reported; it decides nothing.
  - Expected Elo sign/range: none here; the gate measures it.
  - Most likely failure mode: competing (2), seen only in the gate.
- **Falsification criteria:** H1 is refuted if the gain is under +1.0% or under three standard errors.
- **Cheapest prior falsifier:** RAR-E26's breakdown (known, above).
- **Registered gate and stop rule:** no gate here. One `analyse` run: `python tools/diag/ocb_scale_screen.py analyse --train analysis/artifacts/c51-ocb/train_scale.csv --held-out analysis/artifacts/c51-ocb/val_scale.csv --k 1.54638 --pure-min-pawns 3 --output analysis/artifacts/c51-ocb/analyse-7men.json`. **Use, frozen:**
  (i) With F1 gaining at least +1.0% at three standard errors, C.5.2 builds the rule: with no knight, rook or queen on the board and three pawns or more, `s = min(a + b·pawns + c·passers, 48)` with the fitted constants; otherwise the head's `32 + 4·pawns + 4·passers`.
  (ii) Local qualification of the built engine: it reproduces RAR-E27's head reports position by position on KBP-KB, KBPP-KB and KBP-KBP (truth and drawn), which is the split's mechanical check. Debug and release tests, `cargo fmt --check` and clippy at zero warnings, unit tests of the rule on both sides of the boundary, and the bench fingerprint recorded.
  (iii) The gate is a separate registration before any game (an opposite-bishop endgame-start cohort from pure positions of seven men or more, plus an STC SPRT, with the hazard read above), handed to the maintainer.
  (iv) Otherwise C.5.2's opposite-bishop part closes `NO_CHANGE` on RAR-E26 to RAR-E28, and C.5.2.1 stays open after C.4.
- **Full conditions / provenance:** as RAR-E26; the pure cohort restricted by `--pure-min-pawns 3` (a pure ending has four men plus its pawns); `--pure-min-pawns 1` reproduces RAR-E26's `analyse.json` exactly (checked before this registration).
- **Result:** pending.
- **Disposition:** pending.
- **PREDICTION CALIBRATION (append after exposure):** pending.
- **Artifacts / commits:** `5f108fb` (the screen's option and its test); `analysis/artifacts/c51-ocb/analyse-7men.json` once run.
