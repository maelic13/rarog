# Audit: the endgame-truth instrument, 2026-09-04

Prompted by Basilisk's BAS-E47/BAS-E50. Every claim below was checked against
Rarog's own code and artifacts; nothing is imported. **No fix is applied and no
checklist item is un-ticked by this document** -- it is the evidence half of the
Basilisk-derived audit, written before any change so the numbers cannot be
reconstructed after the instrument moves.

## Summary

Three defects, in descending order of what they invalidate.

| # | Defect | Status | What it touches |
|---|---|---|---|
| A | `endgame_truth.py` aborts a playout when the strong side sheds material | **CONFIRMED, large** | every pawn-family conversion number in PLAN 4.9a |
| B | Two disjoint position sets exist in the artifact set; PLAN 4.9a.1 names the superseded one | **CONFIRMED** | 4.9a.7's "52% vs 47.9%" reframing; the `hce-accepted` artifact |
| C | The truth run that set the current `endgame_floors.json` is not on disk or in git | **CONFIRMED** | 4.9a.26's 0.7260 target, the ratchet baseline |

Bare-king families -- KQ-K, KR-K, KBB-K, KBN-K, KNN-K, KP-K -- are **provably
unaffected by A**, so RAR-E10 (4.9a.4) and the KBN-K debt at 4.9a.26 stand.

## Defect A: the material abort

`tools/diag/endgame_truth.py:195`

```python
if strong_material(board) < initial_material:
    outcome = "material_lost"
    break
```

`strong_material` is `popcount(occupied_co[WHITE])`. The comment above it
records an earlier repair -- counting the whole board was wrong because
capturing the weak side's pawn in KR-KP is the winning plan -- but the
remaining rule is wrong for the same reason in the other direction. Shedding
the strong side's own material is the winning method in most pawn technique:
give one pawn to promote the other in KPP-K, give the rook for the rook and
win the resulting KP-K in KRP-KR, give the bishop to clear the promotion square
in KBP-K. The harness scores every one of those as a failure and ends the game.

**Truth, not material, decides whether the win is gone**, and the instrument
already computes truth: `first_discard_ply` is set from Syzygy the moment a
White move drops the position out of WDL 2.

### Magnitude, on the arm that produced the numbers in PLAN

`tools/results/e08-accepted/endgame-truth.json` (RAR-E08 head, 100 positions
per family, 60,000 nodes/move, 100-ply limit). `upper` is conversion if every
abort that occurred on a clean win with **no non-win-preserving move yet
played** had instead converted; it is an upper bound, not a prediction.

| Family | Conversion | aborts | on clean win | of those, no discard | upper bound | median abort ply |
|---|---:|---:|---:|---:|---:|---:|
| KQ-K | 97/100 | 0 | 0 | 0 | 97/100 | - |
| KR-K | 95/100 | 0 | 0 | 0 | 95/100 | - |
| KBB-K | 100/100 | 0 | 0 | 0 | 100/100 | - |
| KBN-K | 90/98 | 0 | 0 | 0 | 90/98 | - |
| KNN-K | 1/1 | 0 | 0 | 0 | 1/1 | - |
| KP-K | 74/80 | 0 | 0 | 0 | 74/80 | - |
| KPP-K | 74/98 | 26 | 24 | 24 | 98/98 | 8 |
| KBP-K | 92/94 | 2 | 2 | 2 | 94/94 | 9 |
| KR-KP | 86/98 | 0 | 0 | 0 | 86/98 | - |
| KR-KB | 26/28 | 0 | 0 | 0 | 26/28 | - |
| KR-KN | 41/45 | 0 | 0 | 0 | 41/45 | - |
| KQ-KP | 96/98 | 2 | 0 | 0 | 96/98 | - |
| KQ-KR | 71/100 | 1 | 1 | 0 | 71/100 | - |
| KNN-KP | 1/23 | 15 | 0 | 0 | 1/23 | - |
| KRP-KR | 36/73 | 62 | 36 | 33 | 69/73 | 8 |
| KRP-KB | 45/96 | 50 | 47 | 46 | 91/96 | 20 |
| KBP-KB | 17/26 | 31 | 8 | 8 | 25/26 | 5 |
| KBP-KN | 46/57 | 48 | 11 | 9 | 55/57 | 6 |
| KP-KP | 57/57 | 27 | 0 | 0 | 57/57 | - |
| **Total** | **1145/1372 = 0.8345** | **264** | **129** | **122** | **1267/1372 = 0.9235** | |

122 of 129 aborts on a clean win happened before the engine had played a single
non-win-preserving move, and the median abort is at **ply 5-20**: these games
are killed four to ten moves in, not at the end of a long failed technique.
0.8345 is the "aggregate weighted conversion" quoted in PLAN 4.9a.5 and RAR-E11.

The reference arm is contaminated too. `tools/results/reference-sf18/
endgame-truth.json` carries **258 aborts** on the same 1,900 positions and was
run without `--per-position`, so its split cannot be recovered -- it must be
re-run, not re-analysed. Stockfish converts 100% of every family in which it
records no abort, and 47.9% in KRP-KR where it records 62; that pattern is what
Basilisk's corrected re-run turned into 97%.

### Isolation proof for the bare-king families

Not merely an empirical zero. The termination checks run in the order
checkmate, stalemate, `is_insufficient_material`, fifty-move, material. In all
six bare-king families **any** strong-side material loss leaves a position the
insufficient-material test already terminates on the same ply: KQ-K/KR-K/KP-K
lose their only unit and reach K vs K; KBN-K reaches K+N vs K or K+B vs K;
KBB-K (opposite colours by construction) reaches K+B vs K; KNN-K reaches
K+N vs K. So the material check is unreachable there by construction, and the
artifacts agree: **0 `material_lost` outcomes in those six families across all
ten truth artifacts on disk.**

Therefore RAR-E10's KBN-K 19.4% -> 96.9% and KBB-K 78.0% -> 100.0%, the KXK
numbers, and RAR-E12's KBN-K dtz debt are unaffected by defect A.

`tools/diag/endgame_conversion.py:61` carries the same rule over whole-board
material, but that runner only covers KQ-K, KR-K, KBB-K and KBN-K. It has no
insufficient-material check of its own, so there `material_lost` is doing that
job and the label is semantically correct. It needs no change.

## Defect B: two disjoint position sets

`b9cc252` ("Seed endgame_truth families by name, not list index") changed which
positions the harness generates. Regenerating with the current code and seed
`6200600` and comparing FENs position-for-position:

| Artifact | Matches current generator | Head |
|---|---|---|
| `e08-accepted` | 1900/1900 | RAR-E08 |
| `mopup-diag` | 1900/1900 | 4.9a.4 accepted (KBN-K 95/98) |
| `mopup-final` | 1900/1900 | |
| `v2-accepted-pgo`, `v2-accepted-plain` | 1900/1900 | pre-4.9a.4 (KBN-K 19/98) |
| `v2-narrow` | 1900/1900 | |
| `reference-sf18` | theory vector identical to the above | Stockfish 18 |
| **`hce-accepted`** | **0/1900** | superseded, index-seeded |
| `mopup-cand`, `mopup-narrow` | 0/1900 | superseded, index-seeded |

**PLAN 4.9a.1 names `tools/results/hce-accepted/endgame-truth-accepted.json` as
the artifact of record. It is one of the three on the superseded set** and
shares no position with the reference arm.

This propagates into 4.9a.7's reframing. That paragraph compares Rarog's "52%"
against Stockfish's "47.9%" in KRP-KR. 52% is 35/67 from `hce-accepted`
(superseded set); 47.9% is 35/73 from `reference-sf18` (current set). The
comparable same-set pair is **49.3% (36/73) against 47.9% (35/73)** -- the
conclusion that the reference is close rather than far ahead survives and in
fact strengthens, but the stated comparison is between disjoint samples. PLAN's
ordering table mixes at least two sets in its Conversion column: KRPKR is quoted
at 52% from `hce-accepted` while RAR-E11 quotes 43.8% for the same family from
`mopup-diag`.

Neither number is trustworthy anyway while defect A stands: the family has 62
aborts out of 100 in the Rarog arm and 62 in the reference arm.

## Defect C: the floors have no artifact

`tools/diag/endgame_floors.json` at `b711d4d` records KBN-K conversion 0.8980
(88/98) and dtz progress 0.6753 over n=3178. **No truth report on disk carries
those numbers** -- the run that produced them is gone. `tools/results/*` is
gitignored, so nothing there is in git either. The 0.7260 acceptance target
that 4.9a.26 owns is therefore a number without a reproducible artifact, which
is exactly what AGENTS.md's evidence rule forbids. It is re-derivable by
re-running the head, which the defect-A repair requires anyway.

Separately, `endgame_floors.py` compares a candidate report against the floors
file without checking that the two were measured on the same position set; its
comment "the two runs share positions, so the paired SE is smaller and this is
conservative" is an assumption the file cannot currently enforce. Given defect
B, that assumption has already been false for at least one pair on disk.

## What this invalidates

Conclusions that rest on a contaminated conversion number:

1. **PLAN 4.9a ordering table**, Conversion column -- the abort fired in 10 of the
   20 listed families, three more are not measured at all, and two position
   sets are mixed.
2. **RAR-E11** -- "Stockfish does not convert everything: 90.2% weighted",
   "Rarog before 4.9a.4 76.1%, after 83.2%", and the ranked gap list. Both arms
   contaminated; the reference arm cannot even be re-analysed.
3. **4.9a.7's reframing** -- "the reachable mark was four points away, not
   fifty-five". Unpaired and contaminated on both sides.
4. **RAR-E08's** "aggregate weighted conversion is flat, 83.24% -> 83.45%".
5. **RAR-E12's** "endgame aggregate conversion 0.8345 -> 0.8477".
6. **`endgame_floors.json`** conversion floors for the ten pawn families, which
   are depressed by the abort and so are lenient exactly where 4.9a works next.
7. **4.9a.8's** near-null verdict, to the extent it is read as a conversion
   result. Its primary instrument was `endgame_drawn.py`, which does no playout
   and is unaffected; the conversion half is contaminated.

Not invalidated: RAR-E10 / 4.9a.4 (proof above), the 64 frozen theory vetoes in
`tests/endgames.rs` (static verdicts, no playout), `endgame_drawn.py` and every
drawn-cohort number including 4.9a.7's 37.1% -> 25.8%, `endgame_book.py`, and
every SPRT result -- games were played by fastchess, never by this harness.

## Proposal, for sign-off before anything is un-ticked

1. Fix `endgame_truth.py`: keep playing on a material shed and record
   `shed_material_ply` as a diagnostic. Do not touch `endgame_conversion.py`.
2. Add a cohort fingerprint to the report and make `endgame_floors.py` refuse
   to compare across position sets, so defect B cannot recur silently.
3. Re-run **both** arms -- current head and the same Stockfish binary -- under
   the recorded conditions with both binaries pinned by SHA-256 and
   `--per-position` on, so only the termination rule differs.
4. Re-derive the floors from the corrected head run and re-state the 4.9a.26
   target on an artifact that exists.
5. Reopen, on the evidence above: the Conversion column of the 4.9a ordering
   table (and hence the order of 4.9a.9-4.9a.26), RAR-E11, and the conversion
   half of 4.9a.7 and 4.9a.8. Whether 4.9a.7/4.9a.8 are un-ticked as *steps* is
   the maintainer's call -- their accepted mechanisms were measured on the
   drawn cohort, which is intact.
