# C.6.7 — Rarog's pawn and passer definitions against the donor's: what differs, what carries outcome information, what else should change

Research record for PLAN C.6.7 (`R2`, state `RESEARCH`), opened 2026-10-12
on the head C.6's gate left (`38cbb02`, 14,197,472 / EBF 2.557). Zero games.
Static outcome loss ranks questions and accepts nothing; nothing here is
Elo. **No change is decided or built before the maintainer has reviewed the
finding below** (maintainer decision 2026-10-11).

## Why

RAR-E42 absorbed the donor's passed-pawn residual with C.6's unit, but the
pawns family stayed where it was (+0.121 → +0.120% of held-out loss in the
endgame band), and the donor's weak-unopposed term still carries +0.055
points beside Rarog's new one. A probable reason is that the two engines
define the underlying pawn categories differently. This record lists every
difference, measures the ones that could matter, and recommends.

## Inventory (from `src/eval/pawns.rs`, `src/eval/passers.rs` and the donor's `pawns.cpp` and `evaluate.cpp` at the `oracle/hybrid` tag)

| # | Item | Rarog | Donor | Read |
|---|---|---|---|---|
| 1 | Passed set | no enemy pawn ahead on its own or an adjacent file; the rear pawn of a doubled pair counts | also passed when the only stoppers are levers, when the lever-push stoppers are outnumbered by its phalanx, or (rank 5 or more) when a single blocker can be bypassed by a supported push; never with an own pawn ahead on its file; `passed()` then drops blocked candidates without help | `passed_set`: the donor's `passed()` over its set against over Rarog's |
| 2 | Candidate passers | a separate term: rank 4 or more, supported, stopped only on its own file | folded into the passed set (the lever cases) | inside `passed_set` |
| 3 | Passed rank bonus | by rank, fitted | by rank | same shape: pricing |
| 4 | Path and stop | free and safe stop square, free and safe path on its own file, linear in rank | 35/20/9/0 by `5r − 13` over the three-file span, rooks or queens behind change what is unsafe, +5 for a defended block square, a file adjustment | after C.6's refit the donor's path, defended-block and file components carry nothing (RAR-E42: −0.007, −0.001, −0.001 points); not re-read |
| 5 | King proximity | the donor's shape since C.6.4 | — | absorbed (RAR-E42: +0.131 → +0.007) |
| 6 | Backward | no own pawn strictly behind on an adjacent file, and the stop square attacked by an enemy pawn | no own pawn behind *or level* on an adjacent file, and the stop square attacked *or blocked* by an enemy pawn; only scored when the pawn is neither connected nor isolated | `backward` |
| 7 | Isolated | no own pawn on an adjacent file | the same, except the doubled-isolated case below takes its place | `isolated` |
| 8 | Doubled | every pawn on a file holding more than one own pawn | only a pawn with an own pawn directly behind it, and only when unsupported | `doubled` |
| 9 | Doubled and isolated | an extra penalty on each such pawn | an opposed isolated pawn with an own pawn behind on its file and no enemy pawn on the adjacent files takes the doubled penalty instead of the isolated one | `doubled_isolated` |
| 10 | Connected | two tables by rank: supported (defended diagonally) and phalanx (a neighbour on the same rank) | one value, `Connected[r] · (2 + phalanx − opposed) + 21 · supporters`, its eg part `· (r − 2) / 4`: halved when opposed, and counting supporters | `connected` (Rarog's ranks 2 to 7 of both tables against the donor's value) |
| 11 | Weak unopposed | since C.6.4: Rarog's isolated or backward pawn with no enemy pawn ahead on its file | the donor's isolated or backward pawn, unopposed | `weak_unopposed` |
| 12 | Lever | our pawn attacking an enemy pawn (weight 0, frozen as sparse) | weak lever: an unsupported pawn attacked by two enemy pawns | `lever` |
| 13 | Blocked pawns | only blocked passers (by a piece) | a pawn on its fifth or sixth rank blocked by an enemy pawn | `blocked` (Rarog has none) |
| 14 | Pawn islands | a penalty per island | — | Rarog only; nothing to compare |
| 15 | Exclusions | every term independent | connected, then isolated, then backward, as an `else if` chain | inside 6, 7 and 10 |
| 16 | Rook behind a passer, blocked passer, ideal blockader, unstoppable passer | Rarog's terms | rooks or queens behind only change path safety (4) | not read |

## Method (RAR-E43)

`tools/diag/pawn_subterms.py` (`11a3589`) writes, beside the donor's
components, Rarog's own counts for every row, checked against the engine's
trace field by field (`verify-rarog`). For each item marked "Read", three
models beside Rarog's score: Rarog's form (its counts, priced freely in mg
and eg), the donor's form, and both. **The donor's form beyond Rarog's**
(both against Rarog's form, paired) is what Rarog's definition misses.
Registration, prediction and rule in `docs/experiments/RAR-E43.md`.

## Finding

Pending RAR-E43.
