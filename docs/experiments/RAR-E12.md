# RAR-E12 — Complete HCE refit on `hce-v3-tb`

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Complete HCE refit on `hce-v3-tb`** -- the 4.9a.6 corpus: 602,619 non-adjudicated games from the phase-weighted `phase_book_v1.epd`, 3,500,000 train rows, <=6-man labels Syzygy-corrected. Gated against the accepted RAR-E08 head.

## Result / disposition

**H1 ACCEPTED at 7,388 games: +11.81 +/- 5.33 Elo, +17.57 +/- 7.92 nElo**, LOS 100.00%, LLR 2.95, 1h23m, 4 time forfeits (0.054%, under the ceiling). **ADOPTED 2026-09-03 at `d1d95ab`, with the KBN-K breach waived to an owner.** The registered disposition required a repair or a recorded waiver. A partial revert of `king_safety_table` was built and measured as RAR-E13, then WITHDRAWN: the 1,218 slots are jointly fitted, so reverting one block leaves the compensations the other slots made for it in place, producing a vector that is the optimum of nothing -- and it would have become the seed for every 4.10 refit cycle. The waiver instead assigns KBN-K to **4.9a.26**, which is open and already scoped as "gradient owned by 4.9a.4; verify and close", with **0.7260** as its target. This mirrors RAR-E08 handing KQ-KP's -3.8 pp to 4.9a.14. Verified before adoption: 286 debug and 264 release tests pass including the 64 frozen theory vetoes, clippy `--all-features --all-targets` clean, and a rebuild with the exact feature set reproduces 8,044,078 / 2.481. Offline: frozen test improved by **0.00042745** within its own corpus (0.08884903 -> 0.08842158), best validation epoch 22 of 60 on a flat curve. **Conversion side-note SUPERSEDED by RAR-M24.** Historical v1 aggregate conversion was **0.8345 -> 0.8477**. The matched corrected v2 pair is **1254/1372 = 0.9140 -> 1278/1372 = 0.9315**. KP-K, KQ-K and KQ-KP DTZ progress improve in the v2 pair, but **KQ-KP conversion is 96/98 -> 94/98**; the prior statement that its RAR-E08 conversion debt was repaid was overbroad.

## Conditional lesson and retry trigger

**Two disclosed defects, registered before any games so neither can be discovered afterwards.** (1) `bench 13` **7,165,683 -> 8,044,078, +12.3%**, EBF 2.462 -> 2.481 -- far larger than RAR-E06's +3.6% or RAR-E08's -0.8%. Search pruning margins are calibrated to the old eval scale and PROCESS rule 10 defers remeasuring them to 4.11, so **this gate measures eval gain MINUS search-efficiency loss** and can land negative while the evaluation itself is better. (2) The endgame floors **FAIL**: KBN-K dtz progress **0.7260 -> 0.6753, -4.4 SE**, in the family 4.9a.4 fixed. Conversion there is 0.9184 -> 0.8980 (-2 of 98, not significant) and win-preservation improved to 0.9994, so it still mates and never discards a win -- it takes longer routes (3,178 graded moves against 2,989). The mate-drive constants are NOT in the Texel surface and the patch does not touch them; the only changed terms that fire against a bare king are king-safety ones, `king_safety_table` having lifted 414 -> 515 at the top and 316 -> 374 mid-band. **That mechanism is plausible and UNVERIFIED.**

## Source

this registration; `analysis/artifacts/rar-e12-{final-vector.txt,candidate-eval.patch}`; `tools/results/hce-fit-20260902_221030`; record: `analysis/ledger_records_2026-09-14.md`, RAR-E12 (Evaluation and data experiments)
