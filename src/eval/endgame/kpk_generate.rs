//! The KPK bitbase's generator, which `build.rs` runs: the standard iterative
//! (retrograde) classification used by Stockfish, over every position with
//! the pawn White's. `kpk.rs` reads the bits it writes.

// Result flags, OR-combined while classifying successors.
const INVALID: u8 = 0;
const UNKNOWN: u8 = 1;
const DRAW: u8 = 2;
const WIN: u8 = 4;

/// Positions: `stm(2) × wk(64) × bk(64) × pawn(64)`, an index's four base-64
/// digits. Pawn squares on ranks 1 and 8 are invalid.
pub const SIZE: usize = 2 * 64 * 64 * 64;

fn index(stm: usize, wk: usize, bk: usize, p: usize) -> usize {
    ((stm * 64 + wk) * 64 + bk) * 64 + p
}

fn rank_of(sq: usize) -> usize {
    sq / 8
}

fn bit(sq: usize) -> u64 {
    1u64 << sq
}

/// King distance: the larger of the rank and file distances.
fn dist(a: usize, b: usize) -> usize {
    rank_of(a).abs_diff(rank_of(b)).max((a % 8).abs_diff(b % 8))
}

const NOT_FILE_A: u64 = 0xFEFE_FEFE_FEFE_FEFE;
const NOT_FILE_H: u64 = 0x7F7F_7F7F_7F7F_7F7F;

fn king_attacks(sq: usize) -> u64 {
    let king = bit(sq);
    let sideways = ((king << 1) & NOT_FILE_A) | ((king >> 1) & NOT_FILE_H);
    let row = king | sideways;
    sideways | (row << 8) | (row >> 8)
}

fn white_pawn_attacks(sq: usize) -> u64 {
    let pawn = bit(sq);
    ((pawn << 9) & NOT_FILE_A) | ((pawn << 7) & NOT_FILE_H)
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

/// One bit per position, set where the pawn's side wins: bit `i % 8` of byte
/// `i / 8`. Classifies every position, in index order, until a pass changes
/// none.
pub fn bitbase() -> Vec<u8> {
    let digits = |i: usize| (i >> 18, (i >> 12) & 63, (i >> 6) & 63, i & 63);
    let mut db: Vec<u8> = (0..SIZE)
        .map(|i| {
            let (stm, wk, bk, p) = digits(i);
            classify_initial(stm, wk, bk, p)
        })
        .collect();
    loop {
        let mut changed = false;
        for i in 0..SIZE {
            if db[i] == UNKNOWN {
                let (stm, wk, bk, p) = digits(i);
                let r = reclassify(stm, wk, bk, p, &db);
                if r != UNKNOWN {
                    db[i] = r;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut bits = vec![0u8; SIZE / 8];
    for (i, &verdict) in db.iter().enumerate() {
        if verdict == WIN {
            bits[i / 8] |= 1 << (i % 8);
        }
    }
    bits
}
