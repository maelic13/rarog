# RAR-E35 — C.5.3 transitions: the mate drive's boundary (a bare defending king and a winner with no pawn, rook or queen), read statically and in playouts from the neighbouring families against Stockfish

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-11, before any number was computed (maintainer request the same day, after RAR-E34 closed C.5.3); agent-run (zero games).
- **Baseline SHA / candidate SHA / dirty-diff hash:** head = branch `c5` at `51b7f0c`, engine source equal to `master`'s `36f04d5` (12,567,796 / EBF 2.535); `analysis/artifacts/c53/rarog-head.exe` (`417b814b…`). Reference = Stockfish `bf2d8bf6…` (`D:\chess\results\budget-transfer-20260905\reference.exe`, RAR-E34's reference binary).
- **Binary / compiler / PGO identity:** RAR-E34's head build; `analysis/artifacts/c521/rarog-texel-head.exe` for the static evaluation.
- **Research question:** the mop-up term (`src/eval/endgame/mop_up.rs`) has two branches above |score| 200. The minor-piece mate drive, worth up to about 2,800 cp, applies only when the defender has a bare king and the winner has no pawn, rook or queen. Otherwise the general drive applies, at most about 80 cp. So the evaluation can jump by thousands of centipawns when the winner's own pawn leaves the board (KBNP-K, KBBP-K → KBN-K, KBB-K) or when the defender's last pawn does (KBN-KP → KBN-K). Does that boundary change how the engine plays the neighbouring families: does it give up its own pawn to reach the drive, or convert them more slowly than Stockfish?
- **Hypothesis / proposed mechanism:** (H-static) the jump is real and large: removing the winner's own pawn *raises* the static score in most KBNP-K positions. (H-play) at 200k nodes and more the search sees promotion and mate beyond the jump, so play is unaffected; the incentive is static only.
- **Competing hypotheses:** (1) *it bites:* at these budgets the engine sheds its pawn into the slower bishop-and-knight drive more often than Stockfish does, and converts more slowly or loses wins to the fifty-move rule. (2) *the defender's side:* in KBN-KP the jump makes the search chase the last pawn at the cost of the mating net, and conversion is slower than Stockfish's.
- **Interacting mechanisms / consumers:** the mop-up branches and their |score| > 200 gate; the lazy gate (mop-up runs on both paths); the specialised scales (none fires in these families); the search's rule-50 damping and mate scores. Consumer: C.5.3 (reopened by this read if it bites), then a unit making the drive continuous across the boundary.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Static* (`transition_static.py`; 400 generated positions a family, Syzygy clean wins kept): in KBNP-K, removing White's own pawn raises the score in at least half the pairs (probability 0.7), median jump +600 cp (80% band [+100, +1,500]). In KBN-KP, removing Black's pawn raises it in at least 90% (probability 0.8).
  2. *Play:* the head sheds material in KBNP-K and KBBP-K clean wins no more than 10 points more often than Stockfish at 200k (probability 0.6). In all four families it converts within 2 positions of Stockfish at 200k and 600k (probability 0.65).
  - Most likely failure mode: competing (1) in KBNP-K.
- **Falsification criteria:** void if a family's cohort hash differs between the two arms at a budget.
- **Cheapest prior falsifier:** the static read (seconds).
- **Registered order and stop rule (frozen):**
  1. `python analysis/artifacts/c53/transition_static.py emit analysis/artifacts/c53/transition_pairs.csv 400`, `rarog-texel-head.exe --dump-scores` over it, then `... read`.
  2. `python tools/diag/endgame_budget_bracket.py --families KBNP-K,KBN-KP,KBBP-K,KQ-KRP --positions 100 --budgets 200000,600000 --workers 14`. This runs once for the head (`--engine analysis/artifacts/c53/rarog-head.exe --out-dir analysis/artifacts/c53/trans-head`) and once for the reference (`--engine D:/chess/results/budget-transfer-20260905/reference.exe --out-dir analysis/artifacts/c53/trans-ref`). Seed 6200600, 100 plies, Hash 16, tablebases off, per-position records.
  3. Paired read per family and budget over clean wins: converted positions, median plies to mate, and the share of games in which the winning side's material fell (`shed_material_ply`).
  - Nothing is added after the numbers are seen.
- **Use (frozen):** the boundary **bites** if any of three things holds at 200k nodes:
  - In KBNP-K or KBBP-K the head sheds material in at least 10 percentage points more of the clean wins than Stockfish.
  - In any family it converts 3 or more fewer positions than Stockfish (2 or more at 600k).
  - In KBNP-K or KBBP-K its median plies to mate exceed Stockfish's by 20% or more.

  If it bites, C.5.3 reopens to `RESEARCH`, and a unit making the drive continuous across the boundary goes to registration (the mechanism named here, a cohort read and a harm read as C.5.2). If it does not, C.5.3 stays closed, and the static jump is recorded with this read as the evidence that play does not feel it.
- **Result:** pending.
- **Disposition:** pending.
- **PREDICTION CALIBRATION (append after exposure):** pending.
- **Artifacts / commits:** `analysis/artifacts/c53/` (ignored storage): `transition_static.py`, its pairs and dump, and the two brackets.
