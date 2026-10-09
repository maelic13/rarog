# RAR-M18 — `datagen-v3`: Syzygy tablebase truth for labels, 2026-09-01

Indexed under *2. Measurement, harness and tuning* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**`datagen-v3`: Syzygy tablebase truth for labels, 2026-09-01.** Removing eval adjudication (`datagen-v2`) does not make labels truthful; it makes them reflect what the datagen engine can convert at 8,000 nodes. 4.9a.1 measured conversion at 60,000 nodes -- KBN-K **7%**, KRP-KR **52%**, KBB-K 86% -- and 8,000 is worse, so a theoretically won endgame is played out, drawn on the fifty-move rule, and recorded as a draw, mislabelling every position sampled from it. `datagen-v3` adds `-tb -tbpieces 6 -tbadjudicate BOTH` and deliberately keeps the fifty-move rule, so a cursed win is labelled the draw it really is.

## Result / disposition

**Instrument addition, verified.** A 40-game probe ended **20 of 40 games on tablebase truth** (9 White wins, 3 Black wins, 8 draws), the rest by rules, with the manifest recording `datagen-v3`.

## Conditional lesson and retry trigger

**Truth and realized skill are different measurements and want opposite instruments.** Datagen asks what a position is WORTH, so tablebase adjudication is strictly better than either alternative. A strength gate asks what this engine can actually CONVERT, so the same flag would credit both arms for an endgame only one of them wins -- never use it there. **More nodes is not a substitute and that is measured:** Basilisk's same fit read -2.85 +/- 3.11 on 8k-node outcomes and **+1.00 +/- 2.11, stopped unresolved** on 25k-node outcomes with LTC +0.29 +/- 5.46. About 3x the datagen compute bought a result indistinguishable from zero; reading that +1.00 as an improvement is the RAR-S61 point-estimate error.

## Source

`tools/harness_common.ps1`; `tools/datagen.ps1`; RAR-M15; RAR-M17; RAR-S61; `analysis/archive/basilisk_audit_2026-08-30.md`
