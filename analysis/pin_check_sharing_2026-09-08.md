# Shared pin/check information — RAR-M34 / 4.11b.10

## Decision

**Disposition, 2026-09-08: `NO_CHANGE`.** The duplication this leaf was opened
to exploit has either already been removed or is forbidden by an existing
correctness contract. No producer/state/consumer/invalidation design is
promoted, and no implementation work is justified.

This is a structural finding, not a cost estimate. Region size cannot make
un-shareable work shareable, so the conclusion does not depend on refreshing
the RAR-M30 profile.

## Research card

1. **Mechanism.** The hypothesis was that `compute_pinned`, `check_info` and
   SEE king-safety queries recompute overlapping slider geometry, so a shared
   producer with an explicit lifetime would remove duplicated lookups.
2. **Interactions.** Producers are movegen and `Board::check_info`; consumers
   are legality filtering, the `gives_check` fast path, and SEE recapture
   legality. Any cached state must survive real moves, null moves, undo and
   worker cloning.
3. **Invariants.** The 4.11b.5 SEE repair forbids original-position pin masks
   inside an exchange; `gen_moves_pinned` carries a debug assertion that a
   handed-in pinned set is not stale; node/depth and TT identity must not move.
4. **Falsifier.** If the three producers share no subexpression, or sharing
   would violate the exchange-occupancy contract, the leaf closes `NO_CHANGE`
   regardless of measured region cost. That is what the evidence shows.

## Producer map — what each actually computes

| Producer | King square | Slider set | Occupancy | Result |
|---|---|---|---|---|
| `compute_pinned` | **our** king | **their** B/R/Q | `all_occ`, plus an x-ray pass with our first blockers removed | our pieces pinned to our king |
| `check_info` | **their** king | **our** B/R/Q | `all_occ` and empty-board | check-from squares per piece type, plus discovered-check blockers |
| `see_recapturer` -> `attackers_to_color` | the recapturing side's king (or the target square when the king recaptures) | both colours, all types | **evolving** `after = occ ^ from`, mutating per exchange step | is this recapture legal |

## Why there is nothing left to share

1. **Intra-node pin sharing is already implemented and active.** The 10.3 speed
   pass made `generate_captures_pinned` hand its pinned set to
   `generate_quiets_pinned`. Counters confirm it: **422,246** staged quiet
   generations were served at **zero** additional `compute_pinned` calls. The
   leaf text already said not to implement this again; it is done.
2. **`compute_pinned` and `check_info` share no subexpression.** They query
   from *different king squares* against *different slider colours*. Their only
   common input is `all_occ`. A cache of either cannot serve the other, in any
   node, ever — this is a property of what they compute, not a cost question.
3. **Sharing into SEE is forbidden, not merely unprofitable.** `see_recapturer`
   tests king safety against an occupancy that differs from `all_occ` by every
   piece removed so far in the exchange. Substituting a real-position pin or
   attack mask is precisely the stale `see_pins` defect that 4.11b.5 repaired.
   This is a correctness boundary; no lifetime design removes it.
4. **The cross-ply candidate does not survive inspection either.** A parent's
   `check_info` (their king, our sliders) and the child's `compute_pinned`
   (same square, same slider colour after the side flip) look like a reuse
   opportunity. They are not: occupancy changes across the move, and the two
   compute different predicates — `check_info` accepts a sole blocker of
   *either* colour for discovered check, while `compute_pinned` requires a sole
   *friendly* blocker behind an x-ray pass.

## Activation — these producers are not hot

Exact counters, `bench 13`, `RAROG_DIAG_SAMPLE_STRIDE=1`, diag build
`6b6c3e18...b467a282ce`, nodes **7,601,220** (unchanged by the diag build, so
the instrument observes without perturbing the search):

| Counter | Calls | Per node |
|---|---|---|
| `board_see_threshold_calls` | 7,547,296 | **0.993** |
| `board_gives_check_fast_calls` | 25,540,503 | 3.360 |
| `board_check_info_calls` | 2,245,089 | 0.295 |
| `board_compute_pinned_calls` | 2,079,992 | 0.274 |
| `board_calculate_checkers_calls` | 1,155,770 | 0.152 |
| `board_see_full_calls` | 445,100 | 0.059 |
| `board_gives_check_full_calls` | 49,385 | 0.006 |

Counter units were reconciled before differencing, per the standing rule.
`generate_legal_movelist` + `generate_captures` + `generate_captures_pinned`
= 531,004 + 935,991 + 776,483 = **2,243,478** calls against **2,079,992**
`compute_pinned` calls. The **163,486** difference is exactly the
`generate_captures` early-out, which increments the call counter and returns
before computing pins when `has_pseudo_capture` fails. The counters are
per-call on both sides and the residual is fully explained.

The two producers this leaf targets run on roughly **27–30% of nodes** and are
already lazily gated — `check_info` via `get_or_insert_with` at the search
site. The dominant board-side consumer is SEE at ~1 threshold call per node.
That is **4.11b.11**'s subject, not this leaf's.

## Remaining cost after 4.11b.9 — derived, not re-measured

RAR-M30's profile predates `5c439da`. A fresh ETW capture requires an elevated
prompt and is a maintainer job, and it cannot change this leaf's structural
conclusion, so it was not requested for 4.11b.10. The share update is instead
derived and bounded.

4.11b.9's whole-search gain was **+0.876%**, and the isolated benchmark showed
the change confined to the `make/unmake only` column. Taking the gain as
entirely from that region, new total time is `1/1.00876 = 0.99132` of old, so
every unchanged region's share rises by the same 0.876% factor:

| Region | RAR-M30 | Derived post-`5c439da` |
|---|---|---|
| make/unmake | 7.143% | **6.330%** |
| generation/legality | 6.751% | 6.810% |
| SEE | 5.304% | 5.350% |
| check queries | 5.177% | 5.222% |
| king lookup | 0.544% | 0.549% |

Every unchanged region moves by **at most 0.06 percentage points**. This is
arithmetic under a stated assumption, explicitly **not** a measurement, and no
decision in this section depends on it.

## Observation owed to 4.11b.11

The 4.11b.9 board benchmark showed `threshold SEE only` down in all three
alternating rounds (**-1.77 / -0.98 / -1.68%**), consistent-signed unlike the
other noise controls. SEE calls no changed code, so the likely cause is code
layout shifting after `make_move_inner` grew 468 -> 542 instructions. It is
already inside the accepted net-positive whole-search result and changes
nothing here, but **4.11b.11 starts from a slightly perturbed SEE baseline**
and should re-baseline rather than compare against pre-`5c439da` SEE numbers.

## Retry trigger

Do not reopen shared pin/check state on general principle or on donor-engine
similarity. Reckless's `InternalState`/`update_threats` and Stockfish's
`StateInfo`/`set_check_info` are ownership models for engines whose consumers
differ from Rarog's; neither is evidence for this codebase. Reopen only if a
**new consumer** appears that needs pin or check geometry Rarog does not
already compute once per node — for example a threat-based pruning or history
term — in which case the producer question is genuinely new and must be
registered fresh with its own activation evidence.
