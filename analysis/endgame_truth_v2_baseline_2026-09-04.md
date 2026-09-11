# The corrected endgame truth baseline (PLAN 4.11.1)

Three arms, one frozen cohort, one changed rule.

## Conditions

100 positions per family over 19 families, seed `6200600`, **60,000 nodes/move**,
one engine thread, 16 MB hash, 100-ply limit, engine tablebases disabled, no
adjudication, `--per-position`, `--workers 30`. Worker count changes wall time
only (4.10.3, proved byte-identical against serial).

All three arms report cohort
`fe4866045506636f884ee30526b4188c3def9ca9747f5960ea5c5e7cba5dbb5e`.

| Arm | Binary | Artifact |
|---|---|---|
| current head | `target/release/rarog.exe` (RAR-E12 + 4.9a.7 + 4.9a.8 + 4.10.11) | `tools/results/truth-v2-head/` |
| RAR-E08 head | `tools/test_engines/rarog-e08-head.exe` | `tools/results/truth-v2-e08/` |
| reference | `stockfish-windows-x86-64-bmi2.exe` | `tools/results/truth-v2-reference/` |

## The instrument delta, isolated

The first comparison written for this step compared the v1 RAR-E08 arm against
the v2 CURRENT-head arm and so moved two things at once. That is the confound
this whole cluster exists to prevent, and it was caught by checking the recorded
engine paths rather than by remembering which binary was which. The RAR-E08
binary still existed, so the arms below are matched.

**Instrument only** -- same binary, only the termination rule differs:

| Arm | v1 | v2 | Delta |
|---|---:|---:|---:|
| RAR-E08 head | 1145/1372 = **0.8345** | 1254/1372 = **0.9140** | +109, +0.0794 |
| reference | 1237/1372 = **0.9016** | 1361/1372 = **0.9920** | +124, +0.0904 |

**Engine only** -- both arms under the corrected instrument:

| | Conversion |
|---|---:|
| RAR-E08 head | 1254/1372 = 0.9140 |
| current head | 1276/1372 = **0.9300** (+22) |

The six families the instrument fix moves are exactly the pawn families, and
every bare-king family moves by zero -- the isolation argued at RAR-E14 and
proved by construction now holds empirically at full scale:

| Family | v1 | v2 | Delta |
|---|---:|---:|---:|
| KRP-KB | 45/96 | 85/96 | +40 |
| KRP-KR | 36/73 | 68/73 | +32 |
| KPP-K | 74/98 | 97/98 | +23 |
| KBP-KN | 46/57 | 54/57 | +8 |
| KBP-KB | 17/26 | 21/26 | +4 |
| KBP-K | 92/94 | 94/94 | +2 |
| every other family | | | 0 |

## The corrected baseline

| Family | head | reference | deficit |
|---|---:|---:|---:|
| KQ-K | 98/100 | 100/100 | 2 |
| KR-K | 96/100 | 100/100 | 4 |
| KBB-K | 100/100 | 100/100 | 0 |
| KBN-K | 88/98 | 98/98 | 10 |
| KNN-K | 1/1 | 1/1 | 0 |
| KP-K | 77/80 | 80/80 | 3 |
| KPP-K | 96/98 | 98/98 | 2 |
| KBP-K | 94/94 | 94/94 | 0 |
| KR-KP | 90/98 | 98/98 | 8 |
| KR-KB | 26/28 | 28/28 | 2 |
| KR-KN | 42/45 | 45/45 | 3 |
| KQ-KP | 94/98 | 98/98 | 4 |
| **KQ-KR** | **77/100** | **100/100** | **23** |
| KNN-KP | 5/23 | 14/23 | 9 |
| KRP-KR | 67/73 | 72/73 | 5 |
| KRP-KB | 90/96 | 95/96 | 5 |
| KBP-KB | 24/26 | 26/26 | 2 |
| KBP-KN | 55/57 | 57/57 | 2 |
| KP-KP | 56/57 | 57/57 | 1 |
| **total** | **1276/1372 = 0.9300** | **1361/1372 = 0.9920** | **85** |

## Paired matrix

Over the 1,372 clean wins, head against reference:

| | Count |
|---|---:|
| both converted | 1273 |
| head only | 3 |
| reference only | 88 |
| **neither** | **8** |
| paired union | 1364/1372 = 99.42% |

**The genuinely hard residue is 8 positions.** The v1 equivalent CANNOT be
computed for comparison: `reference-sf18` was run without `--per-position`, so
the old pairing does not exist. That is why 4.11.1 required re-running the
reference arm rather than re-analysing it.

## What this overturns

1. **RAR-E11 is superseded in full.** "Stockfish does not convert everything:
   90.2% weighted" is 99.2%. It is below 100% in four families, not seven, and
   **worse than Rarog in none** -- the v1 claim that it was worse in three
   (KPP-K, KBP-K, KBP-KB) was entirely the abort.
2. **4.9a.7's reframing is dead.** It argued "52% conversion is not the defect,
   Stockfish manages only 47.9%, so the reachable mark was four points away, not
   fifty-five." The reachable mark in KRP-KR is **72/73 = 98.6%**. Rarog's head
   is 67/73 = 91.8%, so the gap is about 7 points against a family that is
   nearly always convertible -- not a hard family where both engines struggle.
   The step's accepted mechanism still stands; it was measured on the drawn
   cohort, which plays nothing.
3. **At 60k, the largest single deficit is KQ-KR at 77/100 against 100/100.** In the
   pre-correction ordering that family sits at **4.12.17**, near the END of the
   list, on a 0% board occurrence. That ordering is historical: 4.11.6 owned
   the first re-ranking, and 4.11.12 superseded it with ranking v2 after the
   occurrence-classifier correction. KQ-KR now belongs to **4.12.10**.
4. **Defect C is closed by reproduction.** The floors recorded KBN-K conversion
   0.8980 (n=98) and dtz progress 0.6753 (n=3178) from a run that existed
   nowhere. The current-head arm reproduces **both to four decimals with
   identical n**, which re-derives the lost artifact and confirms the current
   head is the binary those floors were built from. KBN-K contains no
   `material_lost` outcomes, so v1 and v2 agree there by construction.

## What this does NOT establish

None of these numbers is a strength claim. This is layers 1-3
(`analysis/endgame_measurement_layers.md`); conversion never establishes
strength, and the 85-position deficit is a priority signal for 4.12, not an Elo
estimate. The budget is also 60,000 nodes, below the p25 of deployment
(`analysis/node_budget_2026-09-04.md`), so every family verdict taken here is
budget-specific. **4.11.7 is now complete (RAR-M21, 2026-09-06)**; the former
4.11.5 budget-transfer owner was renumbered. The completed follow-up,
`analysis/endgame_budget_transfer_2026-09-05.md`, finds net deficits of
85/27/16 at 60k/200k/600k. KBN-K and KQ-KP reach full conversion at both
higher budgets; KQ-KR's gap narrows from 23 to 13 to 3. Thus those original
60k magnitudes do not transfer unchanged to deployment-representative search.
Preserve this report as the 60k baseline, not a budget-independent verdict.
