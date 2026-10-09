# RAR-E14 — Audit of the endgame truth instrument, 2026-09-04, prompted by Basilisk BAS-E47/BAS-E50 and verified …

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Audit of the endgame truth instrument, 2026-09-04, prompted by Basilisk BAS-E47/BAS-E50 and verified independently against Rarog's own code and artifacts.** `endgame_truth.py` ended a playout the moment the strong side's piece count dropped. Zero games; the evidence is ten existing artifacts plus a regeneration of the position set from the current code.

## Result / disposition

**Three confirmed defects.** (A) The material abort fired **264 times** on the RAR-E08 arm: 129 on clean wins and **122 of those before the engine had played a single non-win-preserving move**, at a median abort ply of 5-20. Aggregate conversion **0.8345 has a corrected upper bound of 0.9235**; KRP-KR 36/73 bounds at 69/73 and KRP-KB 45/96 at 91/96. The reference arm carries 258 aborts and was run without `--per-position`, so it cannot be re-analysed, only re-run. (B) a later commit changed which positions the harness generates: three artifacts, **including `hce-accepted`, the one PLAN cited as the baseline**, share **zero of 1,900 positions** with the current generator, and 4.9a.7 compared one of them against the reference arm from the other set. (C) The truth run behind the current `endgame_floors.json` exists nowhere -- `tools/results/` is gitignored -- so 4.9a.26's 0.7260 target has no reproducible artifact.

## Conditional lesson and retry trigger

**Bare-king families are provably isolated, which makes RAR-E10 safe rather than merely lucky:** in all six, any strong-side material loss reaches an insufficient-material position, tested one line earlier, so the abort is unreachable there by construction -- and zero `material_lost` outcomes appear in those families across all ten artifacts. **What RAR-E10 does lose is its isolation ARGUMENT.** It recorded "15 of 19 families exactly unchanged"; per-position comparison of the same two artifacts gives **13 of 19**, and the drive reaches **KBB-K, KBN-K, KPP-K, KBP-K, KBP-KB and KBP-KN**, the last two each losing one conversion. The route is knight promotion, which manufactures exactly the material the dispatcher keys on: **a term's blast radius is its dispatcher condition's PROMOTION CLOSURE, not the condition.** RAR-E11 is superseded in full and RAR-E08's and RAR-E12's conversion side-notes with it; both Elo verdicts stand, because fastchess played those games and this harness never touched them. Repair at 4.10, re-measurement at 4.11.

## Source

`analysis/endgame_truth_instrument_audit_2026-09-04.md`; `9281435`; PLAN "Reopened work, 2026-09-04"
