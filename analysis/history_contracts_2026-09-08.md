# History capacity and mutation contracts — RAR-M38 / 4.11b.13

## Decision

**Disposition, 2026-09-08: tightened and closed.** Integrated in `f70ac19`.
Behaviour-neutral, so no playing gate is owed. **No speed claim is made**, per
the register's own condition: RAR-M30 observed zero growth events, and zero
events cannot support a speed argument in either direction.

## What the leaf asked, and what each obligation resolved to

| Obligation | Outcome |
|---|---|
| Preserve arbitrary game-history length; no clamped fixed array | **Kept as `Vec`.** A fixed array would silently drop repetition evidence in a long game |
| Reserve game history plus search headroom before the hot path | **Done.** `search_impl` reserves `MAX_PLY` on the root once |
| Include worker clones | **Done via `Clone`,** which already preserved capacity; now documented as load-bearing |
| Prove no mid-search growth after reservation | **Test:** 40 played plies + a 128-ply walk with capacity asserted unchanged |
| Prove exact repetition/history restoration | **Test:** hash, history depth, clocks and side-to-move restored after a 64-ply unwind |
| Review `is_legal` vs `legal_move` | **Contract documented + pinned by test.** No production caller of `is_legal` |
| Narrow mutation access without hot validation | **`reserve_history` is `pub(crate)`,** not public |

## The capacity gap that was actually there

Before this change the initial capacity was 128 and `Clone` preserved capacity,
but nothing guaranteed a **search margin**. The peak history depth is
`game_plies_played + search_depth`, so a game reaching ~128 plies — an ordinary
64-move game — would have `len == capacity` and reallocate on the first push of
the next search.

RAR-M30 measured **zero** growth events across 25,718,154 pushes, which is
consistent and not a contradiction: `bench` builds every position from FEN, so
game history is empty and the peak never approaches 128. The gap was real but
structurally invisible to the instrument that looked for it. That is the
interesting part of this leaf, and it is why the fix is justified on contract
grounds rather than on a measured event count.

`search_impl` now reserves `MAX_PLY` (128) on the root before any hot path or
helper exists. `Board::clone` preserves capacity, so each worker's `root.clone()`
inherits the reservation and no thread reallocates while searching. Losing that
`Clone` property would reintroduce growth **on helpers only** — thread-asymmetric
behaviour that a single-threaded bench cannot reveal — so it is now documented
at the impl and pinned by a test.

## `is_legal` versus `legal_move`

`Move::from_uci` always yields `QUIET`: a UCI string carries no flag
information. `legal_move` returns the **canonical** move with the real flags
(`DOUBLE_PUSH`, `CAPTURE`, `EN_PASSANT`, `CASTLE_*`, `PROMO_CAPTURE_*`), while
`is_legal` collapses that to a boolean and discards it. A caller that asks
`is_legal` and then plays **its own input** pushes the wrong `UnmakeInfo` and
corrupts make/unmake.

Audit result: `is_legal` has **no production callers**. Its three uses are test
assertions that never play the move, which is the safe use. All five search
sites (`search.rs` 2370, 2388, 3922, 5139, 5147) bind the move `legal_move`
returns, so the property the leaf asked to preserve holds. It is now stated in
the doc comment and pinned by
`legal_move_canonicalizes_flags_that_from_uci_cannot_know`, which asserts the
raw flags are `QUIET` and the canonical flags differ, for a double push, a
capture and a castle.

## Verification

- **Behaviour-neutral**: a fresh no-feature build reproduces `bench 13` at
  **7,601,220 nodes / EBF 2.474**.
- Debug **280** / release **281** tests pass; `cargo fmt --check` and Clippy
  `--all-features --all-targets` clean with zero warnings.
- **The clone test was proven to fail when the contract breaks.** Changing
  `Clone` to `Vec::with_capacity(self.history.len())` made
  `clone_preserves_reserved_capacity_so_workers_inherit_it` fail, and **only**
  that test failed — so it is specific as well as live.

## What was deliberately not done

No fixed-size history array, no hot-path validation, and no public accessor for
history capacity: the tests live inside `board.rs` as a unit module so the
private field stays private, and `reserve_history` is `pub(crate)`. The leaf's
"narrow mutation access where useful" is satisfied without widening the public
surface it was asking to narrow.
