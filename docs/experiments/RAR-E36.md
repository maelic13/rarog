# RAR-E36 — C.5.4 residual step: KR-KN, KR-KB and KRP-KB drawn-cohort overclaim on the current head, their wins against Stockfish, and what separates won from drawn on Syzygy truth

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-11, before any number was computed; agent-run (zero games).
- **Baseline SHA / candidate SHA / dirty-diff hash:** head = branch `c5` at `4c92908`, engine source equal to `master`'s `36f04d5` (12,567,796 / EBF 2.535), `analysis/artifacts/c53/rarog-head.exe` (`417b814b…`, RAR-E34's build). Comparisons: the drawn census v1 (`tools/diag/endgame_drawn_census_v1.json`, the 2.4.0-era engine, the same seed and cohorts) and Stockfish `bf2d8bf6…`'s 2026-09-05 truth reports (`tools/results/budget-transfer-20260905/reference/`).
- **Research question:** KR-KN and KR-KB have no recogniser in the engine, and KRP-KB has one only for rook-pawn fortresses. On the 2.4.0-era engine the drawn cohort overclaimed almost every draw: KR-KN 100% of 796 drawn positions above +100 cp (mean +346), KR-KB 99.6% of 1,002 (+307), KRP-KB 100% of 38 (+328). Does that defect survive on the current head? Do the families' wins convert, so that any scale has a baseline it must not break? And what separates won from drawn, on Syzygy truth, for a scale to use?
- **Hypothesis / proposed mechanism:** nothing since 2.4.0 touched these material classes, so the overclaim persists: the score is the material balance plus the general mop-up push (up to about 80 cp above |score| 200). The wins come from the defending king on the edge with its minor piece far away (KR-KN), or caught in the corner of the bishop's colour (KR-KB), as theory says.
- **Competing hypotheses:** (1) *the refits moved it:* C.2's to C.4's refits lowered the rook-minus-minor balance enough that the overclaim fell well below 100%. (2) *wins are few and fragile:* the families' wins (45 in KR-KN and 28 in KR-KB of the reference's 100 positions) convert unreliably, so a scale would also hide a conversion problem. (3) *no simple separator:* won and drawn positions are not separated by king-edge or minor-distance features, so a scale would have to be nearly flat.
- **Interacting mechanisms / consumers:** the general mop-up and its |score| > 200 gate; the specialised scales (`krpkb_scale`); the search's rule-50 damping; transitions into these endings (KRP-KN, KR-KNP, KRB-KB and the like), which a scale would create as a boundary. A later candidate registration reads those transitions, as RAR-E35 did for C.5.3; this read does not. Consumer: C.5.4's scale unit, if any.
- **PRE-REGISTERED PREDICTION (freeze before exposure):**
  1. *Overclaim* (`endgame_drawn.py`, 1,500 positions a family, 60k nodes, +100 cp; the census's cohorts): KR-KN at least 95% (probability 0.85), KR-KB at least 95% (0.85), KRP-KB at least 80% (0.7). Mean scores of drawn positions between +200 and +400 cp in KR-KN and KR-KB (probability 0.7).
  2. *Wins* (`endgame_budget_bracket.py`, 100 positions, 60k/200k/600k, paired with Stockfish's reports): KR-KN, KR-KB and KRP-KB each within 2 converted positions of Stockfish at 200k (probability 0.7).
  3. *Separation* (`separation.py`; descriptive): in KR-KN the win rate is at least twice as high with the defending king on the edge and its knight 3 or more squares away as otherwise (probability 0.6); in KR-KB the win rate is at least twice as high with the defending king in or next to a corner of the bishop's colour as in or next to a corner of the other colour (probability 0.65).
  - Expected Elo: none.
- **Falsification criteria:** void if a family's cohort hash differs from the census's or the reference's.
- **Cheapest prior falsifier:** the drawn read (seconds).
- **Registered order (frozen):**
  1. `python tools/diag/endgame_drawn.py --engine analysis/artifacts/c53/rarog-head.exe --syzygy D:/chess/tablebases/syzygy3456 --families KR-KN,KR-KB,KRP-KB --positions 1500 --nodes 60000 --threshold 100 --workers 14 --output analysis/artifacts/c54/drawn-head.json`.
  2. `python tools/diag/endgame_budget_bracket.py --engine analysis/artifacts/c53/rarog-head.exe --syzygy D:/chess/tablebases/syzygy3456 --families KR-KN,KR-KB,KRP-KB --positions 100 --budgets 60000,200000,600000 --workers 14 --out-dir analysis/artifacts/c54/head`, paired with the reference's per-position records.
  3. `python analysis/artifacts/c54/separation.py analysis/artifacts/c54/separation.json`.
  - Nothing is added after the numbers are seen.
- **Use (frozen):**
  - A family has a **residual** if its overclaim is at least 50% with at least 100 drawn positions, or at least 25 for KRP-KB.
  - Families with a residual continue into C.5.4's candidate design: a scale for the rook-against-minor endings, registered separately, with the drawn cohort as the deciding instrument and three vetoes: conversion of the wins in read 2 (within two standard errors), win preservation, and a transition read into and out of the scaled endings.
  - A family converting 3 or more fewer wins than Stockfish at 200k is recorded as a separate conversion defect.
  - Without a residual anywhere, C.5.4 closes `NO_CHANGE`.
- **Result (run 2026-10-11, once each, after the registration commit `97cd6da`):**
  - *Overclaim* (`drawn-head.json`; cohort hashes equal to census v1's), drawn positions above +100 cp, old (census v1) → head:
    - KR-KN: 100% of 796 (mean +346) → **98.4%** (783 of 796; mean +208, median +212, max +333)
    - KR-KB: 99.6% of 1,002 (mean +307) → **94.6%** (948; mean +182, median +186, max +340)
    - KRP-KB: 100% of 38 (mean +328) → **97.4%** (37; mean +195, median +203)
    - No mate claims.
  - *Wins* (`compare.json`; cohort hashes equal to the reference's), Stockfish / head converted at 60k · 200k · 600k: KR-KN 45/44 · 45/45 · 45/45; KR-KB 28/28 · 28/28 · 28/28; KRP-KB 95/92 · 96/94 · 96/94. KRP-KB's failures are ply limits (3, 2, 1) and insufficient material (1, 0, 1). Win preservation 0.984 to 0.999.
  - *Separation* (`separation.json`; 1,500 generated positions a family, 704 and 498 Syzygy wins), win rate:
    - KR-KN by defending king to knight distance: 1: 0.03 (167); 2: 0.27 (234); 3: 0.41 (293); 4 or more: 0.64 (806).
    - KR-KN by the defending king's edge distance: 0: 0.57; 1: 0.40; 2: 0.40; 3: 0.26. On the edge with the knight 3 or more away: 0.66 (560), against 0.35 otherwise.
    - KR-KB by king to bishop distance: 1: 0.03; 2: 0.27; 3: 0.43; 4 or more: 0.38. By edge distance, 0.31 to 0.35 throughout.
    - KR-KB by corner: in or next to a corner of the bishop's colour 0.37 (217); of the other colour 0.29 (190); no corner 0.33.
- **Disposition:** **all three families have a residual; C.5.4 continues into candidate design.**
  - The refits since 2.4.0 cut the drawn positions' scores by about 125 cp, but 94.6% to 98.4% of draws are still scored above +100 cp. Competing (1) holds for the magnitude only.
  - The wins convert like Stockfish's (competing (2) refuted), so a scale has a clean baseline for its conversion veto.
  - On generated positions the strongest separator is the defending king's distance from its own minor piece: a minor next to its king is almost never lost (3%), one 4 or more squares away usually is (64% in KR-KN). The theory features are weaker here: the defending king on the edge matters in KR-KN (2.2× between edge and centre) and not in KR-KB, and the bishop's corner colour barely separates (1.28×). Random placements are dominated by tactics against a loose minor piece, so competing (3) holds in part.
  - The candidate (a scale for KR-KN and KR-KB, with KRP-KB's existing fortress scale kept) is a separate registration. Its deciding instrument is the drawn cohort, with three vetoes: this read's conversion as the baseline (within two standard errors), win preservation, and a transition read into and out of the scaled endings (RAR-E35's design). Not Elo.
- **PREDICTION CALIBRATION (appended after exposure):**
  - Original prediction (not rewritten): items 1 to 3 above.
  - Observed:
    - (1) KR-KN hit (98.4%); **KR-KB miss** by 0.4 points (94.6% against 95%); KRP-KB hit (97.4%); means: KR-KN hit (+208), **KR-KB miss** (+182, below the +200 to +400 band).
    - (2) hit (KRP-KB exactly 2 behind at 200k).
    - (3) KR-KN **miss**: 1.9× against 2×. KR-KB **miss**: 1.28× against 2×.
  - Which assumption failed: I carried the old magnitude forward, but the refits did lower the rook-minus-minor balance (by about 125 cp), just not enough to change the verdict. And I expected textbook geometry (edge, corner colour) to separate random positions. Random placements are decided mostly by whether the minor piece is loose, a tactical fact the textbook features miss. The miss is in magnitude and mechanism.
- **Conditional lesson:** on generated endgame positions, read the tactical separators (piece cohesion) before the textbook ones; game positions may weight them differently.
- **Retry trigger or `closed`:** not closed; the candidate's registration follows.
- **Artifacts / commits:** `analysis/artifacts/c54/` (ignored storage): `separation.py`, the drawn report, the bracket and the separation table.
