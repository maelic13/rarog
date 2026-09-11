# Draw-state policy boundary — RAR-M40 / 4.11b.15

## Decision

**Disposition, 2026-09-08: `NO_CHANGE` on all four policies; two contracts
pinned by test.** No playing change is proposed and none is registered. The
engine source is untouched, so the fingerprint holds at **7,601,220 / EBF
2.474**; the only change is `tests/draw_semantics.rs` (`df94b7d`).

The leaf asks for keep/change/retry-trigger **per policy**, explicitly warning
against bundling and against treating the historical combined losses as proof
about each independent part. Each of the four is therefore dispositioned on its
own below, and the section on what RAR-S18 does *not* establish is as important
as what it does.

## What the historical evidence actually establishes

RAR-S18 ran two arms:

| Arm | Content | Result |
|---|---|---|
| A | null-clock **+** cross-null fence **+** root-aware repetition | **−7.21 ± 6.03** |
| B | null-clock **+** cross-null fence (root-aware removed) | **−11.91 ± 7.67** |

Both intervals exclude zero — A's upper bound is −1.18, B's is −4.24 — so both
bundles were genuinely harmful, not merely unproven.

**What it does not establish.** Neither arm isolates a single part. B is
numerically worse than A by 4.70, which might suggest the root-aware component
was the *least* harmful of the three, but the intervals overlap heavily
(A `[−13.24, −1.18]`, B `[−19.58, −4.24]`), so that ordering is **not**
statistically supported. No individual disposition below is justified by
appealing to these numbers as evidence about one part.

## Policy 1 — rule-50 clock

**KEEP.** `is_rule50_draw` is `halfmove_clock >= 100 && (!in_check || legal
moves exist)`, so a mate delivered on the 100th-clock move outranks the draw.
This is the one Phase 7.1 fix that survived; it is bench-identical, fires
approximately never, and shipped without an SPRT for that reason. Four tests
cover it: mate at clock 100 beats the draw, check-with-an-escape at 100 is still
a draw, stalemate at 100 is a draw, and search finds mate in one at clock 99.

**Retry trigger:** none. There is nothing pending here — the rule is correct,
and correctness is not a tuning surface.

## Policy 2 — null-move boundaries

**KEEP.** `make_null_move` increments the halfmove clock and clears en passant.
Semantically a null is not a move and arguably should not advance a rule-50
clock, but changing it was inside both losing RAR-S18 arms and is not revisited
on semantic tidiness alone.

The **cross-null repetition question** is separate and resolves on structure,
not on Elo. `is_repetition` compares the full position hash, which includes side
to move, so crossing a null can only ever produce a **false negative** (a real
repetition missed because the stride-2 scan lands on the wrong parity), never a
false positive. And the arbiter path is unaffected in principle, not just in
practice: `can_declare_draw` is reached only from `game_result` and the root
tablebase gate, both at game level where history contains no null moves.
`can_declare_draw_in_search` is the only consumer that can see them, at
`search.rs:2218` and `3896`.

So the fence Phase 7.1 tried to add was guarding against a scoring imprecision
inside search, not a legality defect — and it lost Elo in both arms it appeared
in.

**Retry trigger:** a measured case where a cross-null match changes a *root*
best move, plus its own dependency-complete registration. A semantic argument
alone is not a trigger, and neither is a Stockfish or Reckless port.

## Policy 3 — pre-root versus in-search repetition

**KEEP, and note that a form of root-awareness already exists.**
`can_declare_draw_in_search` scores an aggressive twofold — one prior occurrence
within the scan bound is a search draw — which is a deliberate strength
heuristic and not the arbiter's threefold rule. The arbiter path keeps
`is_repetition(3)`, and `three_occurrences_are_a_threefold_for_the_arbiter`
pins that the two differ.

The rejected "root-aware repetition" was a *further* change. The guard at
`search.rs:2218` is `ply > 0 && board.can_declare_draw_in_search()`, so the root
position itself is already never scored as a draw by the in-search path. That
much root-awareness is present and kept.

**Retry trigger:** a demonstrated case where the aggressive twofold costs a
*won* game — the mechanism it exists to buy is early pruning of repetition
subtrees, so evidence against it must come from converted results, not from
node counts or from semantic preference for the arbiter's rule.

## Policy 4 — repetition keys versus TT and evaluation keys

**KEEP; audited clean, and now pinned.** These are three distinct identities and
they are correctly separated today:

| Identity | Basis | Rule-50 involvement |
|---|---|---|
| Repetition | position `hash` only | none |
| Transposition table | `board.hash` only | none in the key; the clock is applied on **read** as a mate-score correction in `tt::score_from_tt` |
| Evaluation cache | its own key plus a stored `halfmove_clock` compared for equality (`eval.rs:1254`) | a validity guard on the entry, not part of any hash |

The leaf's specific prohibition — never put rule-50 buckets into the repetition
identity merely because a TT key might use them — holds, and the TT key does not
use them either. `the_halfmove_clock_is_not_part_of_the_position_identity` now
asserts that two positions differing only in the clock share one hash, so this
cannot regress silently.

A second test records why the scan bound is safe. `is_repetition` stops at
`halfmove_clock` plies back, and because null moves advance that clock the bound
can reach past an irreversible move. That is harmless: an irreversible move
changes piece placement permanently, so those older positions carry a different
hash and can never match. **The bound is therefore a cost choice, not a
correctness one**, and over-scanning wastes iterations at worst.

**Retry trigger:** none for the separation itself. Any future TT keying change
must not be allowed to migrate into the repetition identity; the test is there
to make that attempt fail loudly.

## On proving the tests live

`the_halfmove_clock_is_not_part_of_the_position_identity` was verified by
sabotage, and it took **three attempts**:

1. Mixing the clock into `check_consistency`'s hash recomputation — no failure,
   because that is a verification path the draw tests never call.
2. Mixing it into `from_fen` before the clock field is parsed — no failure,
   because the value was still zero, so the XOR was a no-op.
3. Mixing it in after parsing — the test failed, and **only** that test failed.

The first two look exactly like a dead test from the outside. Either would have
supported the wrong conclusion, and the lesson generalises: a sabotage that does
not visibly change the thing under test proves nothing about the test.

## Verification

Engine source untouched; `bench 13` holds at **7,601,220 / EBF 2.474**. Debug
**282** / release **283** tests pass, `cargo fmt --check` and Clippy
`--all-features --all-targets` clean with zero warnings. No games, no Elo claim,
no playing change proposed.
