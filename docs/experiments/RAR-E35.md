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
- **Result (run 2026-10-11, once each, after the registration commit `7dc9fef`):**
  - *Static* (`transition_static.json`; 1,165 pairs from 400 generated positions a family), the score's change, White's view, when the pawn leaves the board:

    | Family (pawn removed) | Pairs | Score rises | Rises by more than 500 cp | Median (quartiles) |
    |---|---:|---:|---:|---|
    | KBNP-K (White's own) | 399 | **79.9%** | 56.6% | **+595** (+130, +1,207) |
    | KBBP-K (White's own) | 394 | **98.7%** | 87.3% | **+1,178** (+767, +1,525) |
    | KBN-KP (Black's last) | 372 | 99.7% | 77.7% | +919 (+556, +1,502) |

  - *Play* (`run_trans.sh`, both brackets exit 0, cohort hashes equal; `compare_trans.json`), head / Stockfish over the clean wins:

    | Family | Wins | Converted 200k · 600k | Median plies 200k · 600k | Winner's material fell, 200k · 600k |
    |---|---:|---|---|---|
    | KBNP-K | 100 | 100/100 · 100/100 | 17/17 · 17/17 | 0.14/0.20 · 0.12/0.18 |
    | KBBP-K | 100 | 100/100 · 100/100 | 15/14 · 14/13 | 0.12/0.05 · 0.05/0.06 |
    | KBN-KP | 92 | 90/88 · 91/89 | 63/57 · 61/55 | 0/0 · 0/0 |
    | KQ-KRP | 91 | 89/89 · 90/89 | 27/27 · 27/29 | 0.011/0 · 0/0 |

    The head's failed wins: KBN-KP, one ply limit and one insufficient material at 200k, one ply limit at 600k; KQ-KRP, one insufficient material and one stalemate at 200k, one stalemate at 600k. Stockfish failed more in both families.
- **Disposition:** **the boundary does not bite; C.5.3 stays closed `NO_CHANGE`.**
  - The static incentive is real and large: without the winner's own pawn the score is higher in 80% of KBNP-K positions and 99% of KBBP-K positions.
  - Play does not feel it at 200k or 600k nodes. The head converts as many positions as Stockfish or more in every family, mates KBNP-K and KBBP-K as fast (17 and 15 plies), and gives up material in KBNP-K less often than Stockfish does. The closest line is KBBP-K's shedding at 200k (+7 points against the 10 registered); its plies to mate are within 7%.
  - H-static and H-play both hold, and competing (1) and (2) are refuted at these budgets. The search reaches promotion and mate before the jump matters, so a static preference for the pawnless ending costs nothing it can see.
  - **Recorded as a known shape of the evaluation, not a defect.** Any change to the mop-up term (C.5.7's KBNK table, a continuous drive) re-reads these families with this read as its baseline. Untested: budgets below 200k nodes. Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 and 2 above.
  - Observed: (1) hit on both clauses: 79.9% and a +595 median against +600, and 99.7% for KBN-KP; the bishop pair's jump, which I did not predict separately, is twice the knight's. (2) hit on both clauses (shedding within 10 points; conversion level or better everywhere).
  - Which assumption failed: none; confidence about right (0.6 to 0.8 on outcomes that happened).
- **Conditional lesson:** a static discontinuity is a hypothesis about play, read where the search acts; here mate scores dominate before the evaluation's preference can.
- **Retry trigger or `closed`:** closed; re-read with any change to the mop-up term.
- **Artifacts / commits:** `analysis/artifacts/c53/` (ignored storage): `transition_static.py`, its pairs and dump, and the two brackets.
