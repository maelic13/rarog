//! KPK (king + pawn vs king) bitbase, generated once via the standard iterative
//! (retrograde) classification used by Stockfish. Lets the Phase 3.11 endgame
//! scale-factor framework score KPK draws exactly without tablebases.
//!
//! The pawn is always treated as **White's**, advancing toward rank 8. A caller
//! whose pawn belongs to Black mirrors the board vertically (square `^ 56`) and
//! swaps the side to move before probing.
//!
//! Index space is `stm(2) × wk(64) × bk(64) × pawn(64)`; pawn squares on ranks 1
//! and 8 are simply marked invalid. The table is built lazily on first probe and
//! cached for the process lifetime, so positions that never reach KPK (e.g. the
//! `bench` suite) never pay for it.

use std::sync::OnceLock;

// Result flags, OR-combined while classifying successors.
const INVALID: u8 = 0;
const UNKNOWN: u8 = 1;
const DRAW: u8 = 2;
const WIN: u8 = 4;

const SIZE: usize = 2 * 64 * 64 * 64;

static BITBASE: OnceLock<Vec<u8>> = OnceLock::new();

#[inline]
fn index(stm: usize, wk: usize, bk: usize, p: usize) -> usize {
    ((stm * 64 + wk) * 64 + bk) * 64 + p
}

#[inline]
fn rank_of(sq: usize) -> usize {
    sq / 8
}

#[inline]
fn bit(sq: usize) -> u64 {
    1u64 << sq
}

#[inline]
fn dist(a: usize, b: usize) -> usize {
    let dr = (rank_of(a) as i32 - rank_of(b) as i32).abs();
    let df = ((a % 8) as i32 - (b % 8) as i32).abs();
    dr.max(df) as usize
}

fn king_attacks(sq: usize) -> u64 {
    let r = rank_of(sq) as i32;
    let f = (sq % 8) as i32;
    let mut bb = 0u64;
    for dr in -1..=1 {
        for df in -1..=1 {
            if dr == 0 && df == 0 {
                continue;
            }
            let nr = r + dr;
            let nf = f + df;
            if (0..8).contains(&nr) && (0..8).contains(&nf) {
                bb |= bit((nr * 8 + nf) as usize);
            }
        }
    }
    bb
}

fn white_pawn_attacks(sq: usize) -> u64 {
    let r = rank_of(sq) as i32;
    let f = (sq % 8) as i32;
    let mut bb = 0u64;
    for df in [-1i32, 1] {
        let nr = r + 1;
        let nf = f + df;
        if (0..8).contains(&nr) && (0..8).contains(&nf) {
            bb |= bit((nr * 8 + nf) as usize);
        }
    }
    bb
}

/// Initial (terminal-or-unknown) classification of one position.
fn classify_initial(stm: usize, wk: usize, bk: usize, p: usize) -> u8 {
    let pr = rank_of(p);
    // Pawn cannot stand on rank 1 or 8.
    if pr == 0 || pr == 7 {
        return INVALID;
    }
    // Pieces overlapping, or kings adjacent.
    if dist(wk, bk) <= 1 || wk == p || bk == p {
        return INVALID;
    }
    // White to move but the black king is already attacked by the pawn → the
    // side not to move is in check → illegal.
    if stm == 0 && (white_pawn_attacks(p) & bit(bk)) != 0 {
        return INVALID;
    }

    if stm == 0 {
        // White to move: immediate promotion win when the pawn is on the 7th and
        // the promotion square is safe (king defends it or the black king is too
        // far to capture the new queen).
        if pr == 6 {
            let q = p + 8;
            if wk != q && (dist(bk, q) > 1 || (king_attacks(wk) & bit(q)) != 0) {
                return WIN;
            }
        }
        UNKNOWN
    } else {
        // Black to move: stalemate (no safe king move, and not in check) or a
        // free capture of an undefended pawn are draws.
        let escapes = king_attacks(bk) & !(king_attacks(wk) | white_pawn_attacks(p));
        if escapes == 0 {
            return DRAW;
        }
        if (king_attacks(bk) & bit(p) & !king_attacks(wk)) != 0 {
            return DRAW;
        }
        UNKNOWN
    }
}

/// Reclassify an `UNKNOWN` position from the current table state.
fn reclassify(stm: usize, wk: usize, bk: usize, p: usize, db: &[u8]) -> u8 {
    let mut r = 0u8;
    if stm == 0 {
        // White to move maximises toward WIN; illegal king moves land on
        // INVALID(0) entries and are harmless to the OR.
        let mut ka = king_attacks(wk);
        while ka != 0 {
            let ksq = ka.trailing_zeros() as usize;
            ka &= ka - 1;
            r |= db[index(1, ksq, bk, p)];
        }
        let pr = rank_of(p);
        if pr < 6 {
            let push = p + 8;
            if push != wk && push != bk {
                r |= db[index(1, wk, bk, push)];
                if pr == 1 {
                    let push2 = p + 16;
                    if push2 != wk && push2 != bk {
                        r |= db[index(1, wk, bk, push2)];
                    }
                }
            }
        }
        if r & WIN != 0 {
            WIN
        } else if r & UNKNOWN != 0 {
            UNKNOWN
        } else {
            DRAW
        }
    } else {
        // Black to move minimises toward DRAW.
        let mut ka = king_attacks(bk);
        while ka != 0 {
            let ksq = ka.trailing_zeros() as usize;
            ka &= ka - 1;
            r |= db[index(0, wk, ksq, p)];
        }
        if r & DRAW != 0 {
            DRAW
        } else if r & UNKNOWN != 0 {
            UNKNOWN
        } else {
            WIN
        }
    }
}

fn generate() -> Vec<u8> {
    let mut db = vec![INVALID; SIZE];
    for stm in 0..2 {
        for wk in 0..64 {
            for bk in 0..64 {
                for p in 0..64 {
                    db[index(stm, wk, bk, p)] = classify_initial(stm, wk, bk, p);
                }
            }
        }
    }
    loop {
        let mut changed = false;
        for stm in 0..2 {
            for wk in 0..64 {
                for bk in 0..64 {
                    for p in 0..64 {
                        let i = index(stm, wk, bk, p);
                        if db[i] == UNKNOWN {
                            let r = reclassify(stm, wk, bk, p, &db);
                            if r != UNKNOWN {
                                db[i] = r;
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    db
}

/// Returns true iff the side with the (White-oriented) pawn wins the KPK
/// position. `white_to_move` is the side to move *after* any mirroring the
/// caller applied; `wk`/`bk`/`p` are 0..63 square indices with the pawn White's.
pub fn probe(white_to_move: bool, wk: usize, bk: usize, p: usize) -> bool {
    let db = BITBASE.get_or_init(generate);
    let stm = if white_to_move { 0 } else { 1 };
    db[index(stm, wk, bk, p)] == WIN
}

#[cfg(test)]
mod tests {
    use super::*;

    // Square indices: a1=0 .. h1=7, a2=8 ... rank*8 + file.
    const fn sq(file: usize, rank0: usize) -> usize {
        rank0 * 8 + file
    }

    #[test]
    fn rook_pawn_vs_king_in_corner_is_drawn() {
        // White Ka1, Pa2, Black Ka8: a rook pawn cannot win once the defending
        // king holds the queening corner.
        assert!(
            !probe(true, sq(0, 0), sq(0, 7), sq(0, 1)),
            "rook pawn (Ka1/Pa2) vs Ka8 in the corner is drawn"
        );
    }

    #[test]
    fn king_on_key_square_in_front_of_pawn_is_won() {
        // White Ke6, Pe5, Black Ke8: the white king occupies a key square (e6),
        // so it is a win regardless of the move.
        assert!(
            probe(true, sq(4, 5), sq(4, 7), sq(4, 4)),
            "Ke6/Pe5 vs Ke8 is a known win (king on a key square)"
        );
    }

    #[test]
    fn key_square_wins_regardless_of_side_to_move() {
        // Ke6/Pe5 vs Ke8 is a key-square win for either side to move.
        assert!(
            probe(true, sq(4, 5), sq(4, 7), sq(4, 4)),
            "white to move: win"
        );
        assert!(
            probe(false, sq(4, 5), sq(4, 7), sq(4, 4)),
            "black to move: still win"
        );
    }

    #[test]
    fn defender_with_the_opposition_holds_the_draw() {
        // White Ke1, Pe2, Black Ke3, White to move: Black holds the opposition
        // directly in front of the pawn, drawn.
        assert!(
            !probe(true, sq(4, 0), sq(4, 2), sq(4, 1)),
            "Ke1/Pe2 vs Ke3 with Black holding the opposition is drawn"
        );
    }
}
