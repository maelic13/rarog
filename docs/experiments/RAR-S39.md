# RAR-S39 — Phase-4.5b: continuation correction extended from a single 1-ply `(piece, to)` slot to compact 2- and 4-ply …

Indexed under *3. Search and selectivity › Rejected, neutral or deferred* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

Phase-4.5b: continuation correction extended from a single 1-ply `(piece, to)` slot to compact 2- and 4-ply distances, all three tables aged through one loop, both new weights landed inert. Sized on `bench 13` at the existing 1-ply weight of 152 for comparability.

## Result / disposition

**Diagnostic.** Inert defaults verified: `bench 13` = 6,502,902 / EBF 2.449 on normal, diag and tune builds. Each term alone: `CorrWeightCont2 = 152` **6,914,454 (+6.33%)**, `CorrWeightCont4 = 152` **7,284,605 (+12.02%)**, and 4.5a's `CorrCaptureWeightPct = 44` **6,839,617 (+5.18%)**.

## Conditional lesson and retry trigger

Under these conditions every 4.5 arm **grows** the tree, so none is a free addition and none belongs in a bundle on node cost alone — the same test 4.4's arms had to pass. Deliberately given weights of **0 rather than a plausible-looking default**: adding an eval term with a guessed weight is how RAR-S13 went wrong, and the 4.10 fit is what decides whether distance 2 or 4 carries unique signal beyond the 1-ply term. If neither does, both stay at 0 and the tables cost nothing, since read and write are both skipped at weight 0 — the default performs no table access at all. Aging was centralized in the same commit for a specific reason: three sibling tables on different halving schedules drift out of scale with one another, and then a fitted weight stops meaning what it meant when fitted. That failure would be invisible to a fingerprint.

## Source

`src/search.rs`; `src/params.rs`; Plan 4.5b
