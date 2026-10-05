# RAR-E10 — 4.9a.4 minor-piece mate drive. ACCEPTED 2026-09-01 on maintainer judgement, with NO game gate

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**4.9a.4 minor-piece mate drive. ACCEPTED 2026-09-01 on maintainer judgement, with NO game gate.** The drive used Chebyshev corner distance, which is flat: over 300 won KBNK positions, 19 legal moves collapsed into a median of 3 distinct scores and **94% had a TIED best move**, median gap 0 cp. Replaced by a diagonal pull (`abs(7 - rank - file)`, mirrored for the dark corners) dominating the king-proximity terms, scoped to minor-piece bare-king mates.

## Result / disposition

**KBN-K 19.4% -> 96.9%, KBB-K 78.0% -> 100.0%** on the Syzygy truth corpus, paired position-for-position. `bench 13` **unchanged at 7,226,051 / 2.460**. The original claim that 15 of 19 families were exactly equal is **SUPERSEDED by RAR-M23**: six families change through the dispatcher's promotion closure, including net -1 conversion debt in KBP-KB and KBP-KN. KBN-K's failure mode changed rather than shrinking: 61 fifty-move losses became **zero**, and the residue is 4 positions where the engine gives away the bishop or knight. Floors ratcheted.

## Conditional lesson and retry trigger

**Three axes were needed and the third was nearly missed.** Resolution (a flat metric cannot order its own moves; 40x a 0 cp gap is still 0), magnitude (32.7% -> 57.1% from doubling), and RATIO (the corner pull must dominate, not merely exceed, the king pull). The diagonal shape was tested first at ~1:1 against the king terms, measured WORSE than what it replaced, and recorded as non-transferring; at ~6:1 it is the whole gain. **Sweeping a mechanism's shape while holding its proportions fixed can refute the mechanism for the wrong reason.** Also: this change is bench-INVISIBLE but behaviour-changing, so `bench 13` cannot identify a build carrying it. **Acceptance departs from the stated rule that only a registered SPRT accepts a candidate, and is recorded as a judgement call rather than a gate.** What justified it: bench byte-identical, activation triply gated (`|eval| > 200` AND a bare losing king AND no pawn/rook/queen for the winner), the then-recorded isolation argument, hard theory vetoes and floors passing, and a tier-3 occurrence of 0.28% at which a `[0,3]` gate cannot resolve anything at any budget this project has. What that does NOT establish: bench-identical proves only that 40 bench positions' trees never reach a minor-piece bare-king mate within depth 13, while real games at 3+0.03 reach greater depth with endgames on the board and do fire the term in roughly 1.6% of games. **Retry trigger: any endgame-shaped strength anomaly reopens this without needing new argument.**

## Source

`analysis/endgame_conversion_audit_2026-09-01.md`; `tools/results/mopup-diag/endgame-truth.json`; 4.9a.4
