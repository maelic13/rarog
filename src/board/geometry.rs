//! Board geometry: the squares between and through two squares, files and
//! ranks, the passed-pawn spans and the two distances, as tables built at
//! compile time and shared by move generation and the evaluation.

use super::{Bitboard, Color, Square};

/// Bitboard of squares strictly between `a` and `b` on a rank, file, or diagonal.
/// Returns `EMPTY` if they are not aligned.
pub(crate) fn between(a: Square, b: Square) -> Bitboard {
    // Use precomputed table
    BETWEEN[a.index()][b.index()]
}

/// Full ray through both `a` and `b` (including both endpoints).
pub(crate) fn ray_through(a: Square, b: Square) -> Bitboard {
    LINE[a.index()][b.index()]
}

/// Returns true if `from`, `to`, and `king` are all on the same rank/file/diagonal.
#[inline]
pub(crate) fn on_same_ray(from: Square, to: Square, king: Square) -> bool {
    (ray_through(from, to) & Bitboard::from(king)).any()
}

static BETWEEN: [[Bitboard; 64]; 64] = init_between();
static LINE: [[Bitboard; 64]; 64] = init_line();

const fn abs_i8(v: i8) -> i8 {
    if v < 0 { -v } else { v }
}

const fn signum_i8(v: i8) -> i8 {
    if v > 0 {
        1
    } else if v < 0 {
        -1
    } else {
        0
    }
}

const fn aligned(ar: i8, af: i8, br: i8, bf: i8) -> bool {
    ar == br || af == bf || abs_i8(br - ar) == abs_i8(bf - af)
}

// Const-evaluated init — helpers are non-const; any out-of-range would fail
// the compile-time evaluation itself, so plain casts are sound here.
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const fn init_between() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::EMPTY; 64]; 64];
    let mut a = 0usize;
    while a < 64 {
        let mut b = 0usize;
        while b < 64 {
            if a != b {
                let ar = (a / 8) as i8;
                let af = (a % 8) as i8;
                let br = (b / 8) as i8;
                let bf = (b % 8) as i8;

                if aligned(ar, af, br, bf) {
                    let sr = signum_i8(br - ar);
                    let sf = signum_i8(bf - af);
                    let mut r = ar + sr;
                    let mut f = af + sf;
                    let mut bits = 0u64;
                    while r != br || f != bf {
                        bits |= 1u64 << ((r as u8) * 8 + f as u8);
                        r += sr;
                        f += sf;
                    }
                    table[a][b] = Bitboard(bits);
                }
            }
            b += 1;
        }
        a += 1;
    }
    table
}

// Const-evaluated init — helpers are non-const; any out-of-range would fail
// the compile-time evaluation itself, so plain casts are sound here.
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const fn init_line() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::EMPTY; 64]; 64];
    let mut a = 0usize;
    while a < 64 {
        let mut b = 0usize;
        while b < 64 {
            if a != b {
                let ar = (a / 8) as i8;
                let af = (a % 8) as i8;
                let br = (b / 8) as i8;
                let bf = (b % 8) as i8;

                if aligned(ar, af, br, bf) {
                    let sr = signum_i8(br - ar);
                    let sf = signum_i8(bf - af);
                    let mut bits = (1u64 << a) | (1u64 << b);

                    let mut r = ar + sr;
                    let mut f = af + sf;
                    while r >= 0 && r < 8 && f >= 0 && f < 8 {
                        bits |= 1u64 << ((r as u8) * 8 + f as u8);
                        r += sr;
                        f += sf;
                    }

                    let mut r = ar - sr;
                    let mut f = af - sf;
                    while r >= 0 && r < 8 && f >= 0 && f < 8 {
                        bits |= 1u64 << ((r as u8) * 8 + f as u8);
                        r -= sr;
                        f -= sf;
                    }

                    table[a][b] = Bitboard(bits);
                }
            }
            b += 1;
        }
        a += 1;
    }
    table
}

pub(crate) const FILE_BBS: [Bitboard; 8] = [
    Bitboard::FILE_A,
    Bitboard::FILE_B,
    Bitboard(0x0404_0404_0404_0404),
    Bitboard(0x0808_0808_0808_0808),
    Bitboard(0x1010_1010_1010_1010),
    Bitboard(0x2020_2020_2020_2020),
    Bitboard::FILE_G,
    Bitboard::FILE_H,
];
pub(crate) const ADJACENT_FILES: [Bitboard; 8] = init_adjacent_files();
pub(crate) const FORWARD_RANKS: [[Bitboard; 8]; 2] = init_forward_ranks();
pub(crate) const PASSED_PAWN_MASKS: [[Bitboard; 64]; 2] = init_passed_pawn_masks();
pub(crate) const SQUARE_FILE: [usize; 64] = init_square_file();
pub(crate) const SQUARE_RANK: [usize; 64] = init_square_rank();
pub(crate) const RELATIVE_RANKS: [[u8; 64]; 2] = init_relative_ranks();
/// Chebyshev (king) distance between two squares.
pub(crate) const KING_DISTANCE: [[u8; 64]; 64] = init_distance(false);
const fn init_square_file() -> [usize; 64] {
    let mut table = [0usize; 64];
    let mut sq = 0usize;
    while sq < 64 {
        table[sq] = sq & 7;
        sq += 1;
    }
    table
}

const fn init_square_rank() -> [usize; 64] {
    let mut table = [0usize; 64];
    let mut sq = 0usize;
    while sq < 64 {
        table[sq] = sq >> 3;
        sq += 1;
    }
    table
}

// Const-evaluated init: the `infra` helpers are not `const fn` (trait-based),
// and any out-of-range here would surface at COMPILE time, so plain casts are
// sound and the lint is scoped off with this justification.
#[expect(clippy::cast_possible_truncation)]
const fn init_relative_ranks() -> [[u8; 64]; 2] {
    let mut table = [[0u8; 64]; 2];
    let mut sq = 0usize;
    while sq < 64 {
        let rank = (sq >> 3) as u8;
        table[Color::White as usize][sq] = rank;
        table[Color::Black as usize][sq] = 7 - rank;
        sq += 1;
    }
    table
}

const fn init_adjacent_files() -> [Bitboard; 8] {
    let mut table = [Bitboard::EMPTY; 8];
    let mut file = 0usize;
    while file < 8 {
        let mut mask = 0u64;
        if file > 0 {
            mask |= FILE_BBS[file - 1].0;
        }
        if file < 7 {
            mask |= FILE_BBS[file + 1].0;
        }
        table[file] = Bitboard(mask);
        file += 1;
    }
    table
}

const fn init_forward_ranks() -> [[Bitboard; 8]; 2] {
    let mut table = [[Bitboard::EMPTY; 8]; 2];
    let mut rank = 0usize;
    while rank < 8 {
        let mut white = 0u64;
        let mut r = rank + 1;
        while r < 8 {
            white |= 0xFFu64 << (r * 8);
            r += 1;
        }
        table[Color::White as usize][rank] = Bitboard(white);

        let mut black = 0u64;
        r = 0;
        while r < rank {
            black |= 0xFFu64 << (r * 8);
            r += 1;
        }
        table[Color::Black as usize][rank] = Bitboard(black);
        rank += 1;
    }
    table
}

// Const-evaluated init — see `init_relative_ranks` for the lint scoping.
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
const fn init_passed_pawn_masks() -> [[Bitboard; 64]; 2] {
    let mut table = [[Bitboard::EMPTY; 64]; 2];
    let mut color = 0usize;
    while color < 2 {
        let mut sq = 0usize;
        while sq < 64 {
            let file = (sq % 8) as i32;
            let rank = (sq / 8) as i32;
            let mut mask = 0u64;
            let mut df = -1i32;
            while df <= 1 {
                let f = file + df;
                if f >= 0 && f < 8 {
                    if color == Color::White as usize {
                        let mut r = rank + 1;
                        while r < 8 {
                            mask |= 1u64 << (r * 8 + f);
                            r += 1;
                        }
                    } else {
                        let mut r = 0;
                        while r < rank {
                            mask |= 1u64 << (r * 8 + f);
                            r += 1;
                        }
                    }
                }
                df += 1;
            }
            table[color][sq] = Bitboard(mask);
            sq += 1;
        }
        color += 1;
    }
    table
}

/// Chebyshev distance alone is FLAT: every square in a ring around the target
/// scores identically, so over 300 won KBNK positions 94% had a tied best move.
/// A term that cannot order its own moves cannot steer a search, whatever its
/// magnitude. Manhattan breaks the rings.
pub(crate) const MANHATTAN_DISTANCE: [[u8; 64]; 64] = init_distance(true);

// Const-evaluated init: the `infra` helpers are not `const fn`, both
// differences are larger-minus-smaller in 0..=7, so their sum is 0..=14, and
// any out-of-range would surface at compile time.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
const fn init_distance(manhattan: bool) -> [[u8; 64]; 64] {
    let mut table = [[0u8; 64]; 64];
    let mut a = 0usize;
    while a < 64 {
        let af = (a & 7) as i32;
        let ar = (a >> 3) as i32;
        let mut b = 0usize;
        while b < 64 {
            let bf = (b & 7) as i32;
            let br = (b >> 3) as i32;
            let df = if af > bf { af - bf } else { bf - af };
            let dr = if ar > br { ar - br } else { br - ar };
            table[a][b] = if manhattan {
                (df + dr) as u8
            } else if df > dr {
                df as u8
            } else {
                dr as u8
            };
            b += 1;
        }
        a += 1;
    }
    table
}
