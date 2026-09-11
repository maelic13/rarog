# Larger board representation change — RAR-M39 / 4.11b.14

## Decision

**Disposition, 2026-09-08: `NO_CHANGE`. No architecture comparison is
registered and no implementation is opened.** The leaf's own gate — "only open
an implementation if the preceding profile still identifies substantial
representation cost" — is not met, and each of the three named alternatives is
argued against on measured grounds rather than declined on preference.

The only change made is a compile-time guard on the footprint this decision
rests on, so the premise cannot rot silently.

## The gate

RAR-M36 refreshed the profile at head. No board region exceeds **6.7%**:

| Region | RAR-M36 |
|---|---:|
| make/unmake | 6.677% |
| generation and legality | 6.556% |
| SEE | 5.239% |
| check queries | 5.179% |

These are the costs of *doing* the work, not of the representation. A different
representation does not delete them; it trades their constant factors. Nothing
here identifies a substantial representation cost to attack, and the largest
board region's entire 2x-local ceiling is 3.34%.

Two direct experiments this session point the same way. 4.11b.9 fused ordinary
relocation and won **+0.876%**; 4.11b.11 made SEE attacker maintenance
incremental and *lost* 1-3% on its own benchmark. A representation leaving large
wins on the table does not behave like that.

## Alternative 1 — six type boards plus colours

**Rejected on a measured trade.** `Board` is **264 bytes**. Replacing the twelve
colour-piece bitboards (96 bytes) with six type boards (48) saves **48 bytes**,
giving 216. Rarog already keeps `occupancy[2]` and `all_occ`, so the colour
boards this scheme needs exist.

Both 264 and 216 bytes sit far inside L1; neither crosses any boundary that
matters, and the board stays resident across a search either way. Against that
nothing, every `pieces(color, piece)` becomes a second load plus an AND. There
are **208 call sites**, of which **102 are in `eval.rs`** — the single largest
region in the profile at **29.49%** exclusive. The change pays a hot-path tax
across the engine's busiest code to save a footprint that was never binding.

Reckless uses six type boards plus colours. That is a fact about Reckless. Its
evaluation and consumers differ, and the leaf itself says neither donor is
automatically faster here.

## Alternative 2 — copy per-ply state instead of compact restoration

**Already resolved in Rarog's favour, and the donor direction is quantitatively
worse.** `UnmakeInfo` is **24 bytes**, so a full 128-ply search stack costs
**3 KiB** and stays comfortably in L1. Rarog is already at the compact end: the
hash, castling rights, en-passant square, clocks and checkers are restored from
that record, while piece placement is undone by inverse work that 4.11b.9
reduced to a single fused mask.

Copying whole board state per ply, in the shape of Reckless's `InternalState`,
would cost **128 x 264 = 33 KiB** for the same stack and leave L1 entirely. That
is an eleven-fold increase in the hottest stack in the engine, to avoid inverse
work that is now one XOR pair. Stockfish's `StateInfo` chain is a third design
whose cost lies in pointer-chasing rather than copying; neither is a reason to
move off a 24-byte record that already works.

## Alternative 3 — selectively checked instead of legal generation

**Rejected: the cost it targets is already amortized.** Generation and legality
is 6.556%, but Rarog already shares one pinned set per node across the capture
and quiet stages — RAR-M34 measured **422,246** staged quiet generations served
at **zero** additional `compute_pinned` calls. And the check test is already
overwhelmingly the cheap path: `board_gives_check_fast_calls` **25,540,503**
against `board_gives_check_full_calls` **49,385**, a ratio of about **517:1**.

Moving to pseudo-legal generation with legality deferred to make time would
trade a measured, already-shared per-node cost for a per-move cost on a move
population that pruning discards unexamined. That is a plausible-sounding change
with no measured defect behind it, which is exactly what this leaf exists to
refuse.

## What was changed

Only a compile-time guard, with zero runtime effect — the fingerprint is
unchanged at **7,601,220 / EBF 2.474**.

Both arms of this decision are footprint arguments, so the footprints are now
pinned by `const _: () = assert!(...)` on `size_of::<Board>() <= 264` and
`size_of::<UnmakeInfo>() <= 24`. Upper bounds, not equalities: padding may
differ between supported targets, and only growth would invalidate the
reasoning. Each message names this leaf, so a future field addition that pushes
`Board` past its measured size fails the build and forces the comparison to be
re-opened deliberately rather than drifting past it.

**The guard was proven live**: adding a `[u64; 4]` field to `Board` failed the
build with "Board grew past the 264 bytes 4.11b.14 measured".

## Retry trigger

Reopen only if a future profile shows a **single board region above 12%**, where
a representation change could plausibly reach a resolvable whole-search effect,
**and** a specific mechanism is identified by which the new representation
removes work rather than relocating it. Donor-engine similarity is explicitly
not a trigger, and neither is aesthetic preference for fewer bitboards. Per the
leaf: large rewrites require stronger evidence, not an exemption from it.

## Verification

Fresh no-feature build reproduces **7,601,220 / EBF 2.474**. Debug **280** /
release **281** tests pass; `cargo fmt --check` and Clippy `--all-features
--all-targets` clean with zero warnings. No games, no Elo claim, no NNUE
interaction — full NNUE stacks remain Phase 5.
