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

## Finding (2026-10-12, RAR-E43; for the maintainer's review)

**What was measured.** Both instruments are exact: the donor's rows reproduce
on all 194,444 rows, and Rarog's offline counts equal the engine's own trace
on all 189,284 readable rows for every one of the 11 fields. For each
difference, the donor's form beyond Rarog's form (Rarog's counts priced
freely in mg and eg), in points of held-out loss:

| Difference | Endgame band | No queens | Grade |
|---|---:|---:|---|
| Passed set (#1, #2) | **+0.141 ± 0.039** | **+0.121 ± 0.037** | weak (3.6 and 3.3 SE; the line asks four) |
| Weak unopposed (#11) | +0.032 ± 0.018 | +0.038 ± 0.018 | nil |
| Isolated (#7) | −0.004 | +0.026 ± 0.014 | nil |
| Doubled and isolated (#9) | −0.011 | +0.025 ± 0.013 | nil |
| Doubled (#8) | +0.008 | +0.019 | nil |
| Backward (#6) | −0.006 | +0.002 | nil |
| Connected (#10) | −0.006 | −0.005 | nil (+0.033 ± 0.012 over all rows, in the middlegame) |
| Lever (#12) | −0.005 | −0.006 | nil |
| Blocked (#13) | −0.009 | −0.023 ± 0.010 | nil; it predicts worse |

**What it means.**

1. *The pawn-structure definitions are not where the pawns residual sits.*
   Backward, the definition RAR-E42 suspected, carries nothing. Each
   structure difference is worth 0.03 points or less where pawns matter, and
   re-pricing Rarog's own form gains about as much as the donor's form does.
   The donor's pawns residual (+0.12 in the endgame band) is spread thinly
   over many terms: a pricing question for a refit, not a definition to port.
2. *The passed set is the one lead.* Neither engine's set predicts better on
   its own, but the two together gain +0.14 points in the endgame band: the
   pawns on which the sets disagree carry outcome information. Those are
   Rarog's rear pawn of a doubled passer (Rarog calls it passed, the donor
   does not), the donor's extra passers (only levers in the way, a lever push
   outnumbered by its phalanx, a supported push past a single blocker on the
   fifth rank or higher), and the donor's removal of blocked candidates
   without help. This read cannot say which group carries it.

**What a change would touch and cost.**

- *A structure definition* (backward, isolated, doubled, connected and the
  rest) lives in the pawn cache: free per node, a refit and a gate per
  change. Weak unopposed (C.6.4) is built on Rarog's isolated and backward,
  so changing either moves it too. At 0.03 points or less each, the expected
  gain is below what a gate can resolve.
- *The passed set* feeds every passer term (rank bonus, stop and path, king
  proximity, rooks behind, blockade, the unstoppable passer), C.5.2's
  opposite-bishop rule (it counts passers, so RAR-E28's fit would shift) and
  the candidate-passer term (which the donor's set absorbs). A change is a
  unit: about half a day to implement, a refit, the screens (including a
  re-read of the opposite-bishop cohort) and its own gate. For scale: C.6's
  unit took about 0.2 points of the donor's passed residual for +6.4 Elo at
  gate 1, and C.4.3's took about 0.07 for +13.0 (about 5 of it speed). A
  passed-set change on +0.14 points is worth a few Elo at most, uncertain
  and possibly zero.

**Recommendation.**

1. **No structure-definition change.** Close those differences `NO_CHANGE`
   at this resolution; their residual is the refit's (C.8). Retry trigger:
   C.8's first cycle re-reads the donor's pawns family, and a residual still
   at 0.10 points or more at four standard errors in the endgame band would
   reopen a joint structure question (the donor's whole chain in place of
   Rarog's tables) rather than any single definition.
2. **One cheap follow-up for the passed set, before deciding anything:** a
   zero-game split read (about one to two hours of tooling, seconds to run)
   that gives each disagreeing group its own column (Rarog's rear doubled
   passers; each of the donor's three extra conditions; its removal of
   blocked candidates without help), registered with the same line. If one
   group carries, build that one change and fit it in C.8's refit (gated in
   C.8's cycle), not as a separate unit now; if none does, close the passed
   set `NO_CHANGE` too.
3. **Leave alone:** levers and blocked pawns (nil, and blocked pawns predict
   worse); pawn islands (Rarog only, nothing to compare); the path, defended
   block and file terms (absorbed after C.6's refit); rooks behind, blockade
   and the unstoppable passer (Rarog's own, outside this comparison).

The order of work after the review: the passed-set split read if approved,
then C.5.2.2, then C.8.
