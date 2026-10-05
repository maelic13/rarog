# RAR-E09 — 4.9.1 post-fit residual audit of the accepted HCE

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**4.9.1 post-fit residual audit of the accepted HCE.** `--report-endgames` over the accepted vector at the fit's pinned K=1.37011, on the 127,778 published-but-unused confirmation positions -- never fitted on, never selected anything, not the frozen test. Global loss 0.12241022 over 299 material classes.

## Result / disposition

**Closed: no 4.9 entry evidence found, and a label defect found instead.** The largest residual is **KR-K**: 379 positions, **284 (75%) labelled a draw** in a class that is a 100% theoretical win. The evaluator predicts **0.849** there against a label mean of 0.625 -- **it is closer to the truth than its own training data**. Mechanism confirmed directly: Rarog scores a won KR-K at **+426 cornered / +487 centralised**, while `datagen-v1`'s resign rule needs 600 cp from both engines for three moves, so it never fires; the game plays on at 8,000 nodes, fails to mate inside fifty moves, and is labelled 0.5.

## Conditional lesson and retry trigger

**A residual the surface can represent is not structural evidence.** The surface would price KR-K correctly if the labels said 1.0, so this licenses no structural work -- 4.9 closes on it. It is strong prior evidence for RAR-E08 arm B, and it names the mechanism behind RAR-M18's 13.27% disagreement: not random, but concentrated in classes the engine scores BELOW the resign threshold while being theoretically won. It also qualifies the endgame audit's drawn-subset overconfidence -- on KR-K that overconfidence is the evaluator being right. Above 6 men the same question cannot be answered locally; do not assume it generalizes.

## Source

`analysis/hce_residuals_2026-09-01.md`; `tools/results/hce-accepted/residual-endgames-accepted.csv`; RAR-M18; RAR-E08
