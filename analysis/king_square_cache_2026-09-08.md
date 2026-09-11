# King-square caching — RAR-M37 / 4.11b.12

## Decision

**Disposition, 2026-09-08: `NO_CHANGE`. No cache is prototyped.** The leaf's own
condition — "prototype only if 4.11b.7 still shows a material cost after shared
geometry work" — is not met, and the candidate is unqualifiable under this
project's gate regardless of how the cost is judged.

## A floor declared now would be post-hoc, so the decision does not use one

The register entry asks for "a predeclared practical floor". That floor was
never registered, and the measurement is already exposed twice — RAR-M30 read
king-square lookup at **0.544%** and RAR-M36 reads **0.502%**. Inventing a
threshold now and observing that 0.502% falls below it would be choosing a
number to fit a result, which is the same act as moving a bracket after seeing
games. It is not done here.

The decision rests instead on two facts that do not depend on picking a
threshold at all.

## 1. The candidate cannot be qualified by this project's instrument

The 2x-local whole-search ceiling for a 0.502% region is **0.25%**. That is the
gain if the lookup became *infinitely* fast, which it cannot.

RAR-M33's qualification run — 32 alternating pairs at 1,200,000 nodes on a
verified-idle host — produced a measured bootstrap half-width of **1.003%**, and
even the heavier design registered for RAR-M35 projected 0.6-0.7%. The absolute
maximum possible effect is therefore **two to four times smaller than the
uncertainty of the instrument that would have to accept it**.

This is an instrument-capability argument, not a threshold. No budget this
project has used could distinguish the best possible version of this candidate
from zero, so it cannot reach `LOCAL_QUALIFIED` however it is implemented.

## 2. The realistic gain is a fraction of the ceiling

```rust
pub fn king_sq(&self, color: Color) -> Square {
    self.pieces(color, Piece::King).lsb()
}
```

One bitboard load plus a `tzcnt`. A cache replaces the `tzcnt` with a field
load — it does not remove the memory access, and in the hot callers the board is
already resident. The 0.502% is also an **overlapping** mechanism share, already
counted inside the generation, check-query and SEE regions rather than additive
to them.

Against that, the maintenance surface is the whole of make/unmake: castling
moves the king while promotion and null moves do not, undo must restore both
colours, and worker cloning must copy them. The fields would also have to join
independent consistency reconstruction, per the leaf. That is precisely the
class of derived state whose staleness went undetected in 4.11b.5's `see_pins`,
bought here for at most a quarter of a percent that cannot be measured.

## Interaction check

The leaf was conditional on 4.11b.10-11 first. Both closed `NO_CHANGE` without
touching the board, so no shared-geometry work redistributed this cost. The
region moved 0.544% -> 0.502% only because 4.11b.9 shrank make/unmake and
slightly re-weighted every other share. Nothing is pending that would make king
lookup material later.

## Retry trigger

Reopen only if a future profile shows king-square lookup above **2%** — the
level at which a 2x-local improvement would reach a 1% whole-search ceiling and
so become resolvable by the instrument described above — **and** a caller is
identified that calls it in a loop rather than once per node. Donor-engine
similarity is explicitly not a trigger: Basilisk maintaining two king squares is
a fact about Basilisk, and Reckless extracts from bitboards exactly as Rarog
does.

## Evidence

`analysis/board_search_profile_2026-09-08.md` (RAR-M36, refreshed at head),
`analysis/board_search_profile_2026-09-07.md` (RAR-M30), and RAR-M33's measured
instrument width in `analysis/relocation_2026-09-07.md`. No engine change, no
prototype, no games, no Elo claim.
