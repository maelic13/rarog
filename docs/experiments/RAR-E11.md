# RAR-E11 — SUPERSEDED IN FULL by RAR-E14/RAR-M24

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**SUPERSEDED IN FULL by RAR-E14/RAR-M24.** The corrected v2 reference rerun is **1361/1372 = 0.9920**, the current head is **1276/1372 = 0.9300**, and the reference is worse in **no** family. The historical v1 figures below are retained only as invalid history; both original arms were contaminated and the reference arm had to be rerun, not re-analysed. **Stockfish 18 measured on the identical truth corpus**, same 100 positions per family, same 60,000 nodes, `SyzygyPath` cleared so it evaluates rather than reads the answer. Modern Stockfish has no endgame dispatcher at all, so this measures achievable conversion at a node budget, not a rival mechanism.

## Result / disposition

**Stockfish does NOT convert everything: 90.2% weighted, not 100%.** It is below 100% in seven families and **worse than Rarog in three** -- KPP-K 75.5% against 76.5%, KBP-K 93.6% against 97.9%, KBP-KB tied at 69.2%. Weighted totals: SF **90.2%**, Rarog before 4.9a.4 **76.1%**, Rarog after **83.2%**. The mate-drive work closed half the total gap, and closed KBN-K's from -80.6 pp to **-3.1 pp**.

## Conditional lesson and retry trigger

**Conversion targets must be measured against what is ACHIEVABLE at the budget, not against 100%.** Several apparent defects are node-budget limits: KRP-KR looked like a 52%-conversion failure, but Stockfish manages only 47.9% there, so Rarog's 43.8% is 4.1 pp off the reachable mark rather than 56 pp off a perfect one. The real gaps are where SF reaches 100% and Rarog does not: KQ-KR -25.0, KR-KP -16.3, KRP-KB -11.5, KR-KN -11.1, KR-KB -10.7, KR-K -7.0. Weighted by RAR-M15 occurrence the ranking is KR-K, KQ-K, KRP-KR, KR-KP -- so the elementary rook and queen mates, not the exotic families, carry the most recoverable value.

## Source

`tools/results/reference-sf18/endgame-truth.json`; RAR-M15; RAR-E10
