//! KPK (king + pawn vs king) bitbase, generated via the standard iterative
//! (retrograde) classification used by Stockfish. Lets the endgame
//! scale-factor framework score KPK draws exactly without tablebases.
//!
//! The pawn is always treated as **White's**, advancing toward rank 8. A caller
//! whose pawn belongs to Black mirrors the board vertically (square `^ 56`) and
//! swaps the side to move before probing.
//!
//! Index space is `stm(2) × wk(64) × bk(64) × pawn(64)`. The build script
//! generates the table (`kpk_generate.rs`) and the engine keeps one bit per
//! position: generated at run time, it would run inside the first search that
//! reached KPK, on that search's clock, or delay start-up.

const SIZE: usize = 2 * 64 * 64 * 64;

/// One bit per position, set where the pawn's side wins: bit `i % 8` of byte
/// `i / 8`, for `i` from [`index`].
static BITBASE: [u8; SIZE / 8] = *include_bytes!(concat!(env!("OUT_DIR"), "/kpk.bin"));

#[inline]
fn index(stm: usize, wk: usize, bk: usize, p: usize) -> usize {
    ((stm * 64 + wk) * 64 + bk) * 64 + p
}

/// Returns true iff the side with the (White-oriented) pawn wins the KPK
/// position. `white_to_move` is the side to move *after* any mirroring the
/// caller applied; `wk`/`bk`/`p` are 0..63 square indices with the pawn White's.
pub fn probe(white_to_move: bool, wk: usize, bk: usize, p: usize) -> bool {
    let i = index(usize::from(!white_to_move), wk, bk, p);
    BITBASE[i / 8] >> (i % 8) & 1 != 0
}

#[cfg(test)]
mod tests;
