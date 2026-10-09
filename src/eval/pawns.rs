//! Pawn structure, cached by pawn key, and the file, rank, distance and
//! passed-pawn tables the other terms share.

use super::trace::{tr_eg, tr_mg};
use super::{Evaluator, PAWN_TABLE_SIZE, PawnEntry, color_sign, relative_rank};
use crate::board::attacks::AttackTables;
use crate::board::{Bitboard, Board, Color, Piece, Square};
use crate::infra;

pub(super) const FILE_BBS: [Bitboard; 8] = [
    Bitboard::FILE_A,
    Bitboard::FILE_B,
    Bitboard(0x0404_0404_0404_0404),
    Bitboard(0x0808_0808_0808_0808),
    Bitboard(0x1010_1010_1010_1010),
    Bitboard(0x2020_2020_2020_2020),
    Bitboard::FILE_G,
    Bitboard::FILE_H,
];
const ADJACENT_FILES: [Bitboard; 8] = init_adjacent_files();
pub(super) const FORWARD_RANKS: [[Bitboard; 8]; 2] = init_forward_ranks();
pub(super) const PASSED_PAWN_MASKS: [[Bitboard; 64]; 2] = init_passed_pawn_masks();
pub(super) const SQUARE_FILE: [usize; 64] = init_square_file();
pub(super) const SQUARE_RANK: [usize; 64] = init_square_rank();
pub(super) const RELATIVE_RANKS: [[u8; 64]; 2] = init_relative_ranks();
pub(super) const KING_DISTANCE: [[u8; 64]; 64] = init_king_distance();
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

// Const-evaluated init — see `init_relative_ranks` for the lint scoping.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
const fn init_king_distance() -> [[u8; 64]; 64] {
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
            table[a][b] = if df > dr { df as u8 } else { dr as u8 };
            b += 1;
        }
        a += 1;
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

impl Evaluator {
    pub(super) fn eval_pawns(
        &mut self,
        board: &Board,
        atk: &AttackTables,
        passed: &mut [Bitboard; 2],
        attacks: &mut [Bitboard; 2],
    ) -> (i32, i32) {
        let key = board.pawn_key();
        let slot = infra::index(key) & (PAWN_TABLE_SIZE - 1);
        // Bypass the pawn cache under `texel`: a hit skips the trace counts.
        #[cfg(not(feature = "texel"))]
        {
            let cached = self.pawn_table[slot];
            if cached.key == key {
                *passed = cached.passed;
                *attacks = cached.attacks;
                return (cached.mg, cached.eg);
            }
        }

        let mut mg = 0;
        let mut eg = 0;

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let us = color;
            let them = !us;
            let our_pawns = board.pieces(us, Piece::Pawn);
            let their_pawns = board.pieces(them, Piece::Pawn);
            attacks[us as usize] = if us == Color::White {
                our_pawns.north_east() | our_pawns.north_west()
            } else {
                our_pawns.south_east() | our_pawns.south_west()
            };

            let mut tmp = our_pawns;
            passed[us as usize] = Bitboard::EMPTY;
            while tmp.any() {
                let sq = tmp.pop_lsb();
                let file = SQUARE_FILE[sq.index()];
                let rel_rank = relative_rank(us, sq) as usize;
                let adjacent = ADJACENT_FILES[file];

                if (PASSED_PAWN_MASKS[us as usize][sq.index()] & their_pawns).is_empty() {
                    passed[us as usize] |= Bitboard::from(sq);
                    mg += sign * self.params.passed_mg[rel_rank];
                    eg += sign * self.params.passed_eg[rel_rank];
                    tr_mg!(self, passed_mg, rel_rank, sign);
                    tr_eg!(self, passed_eg, rel_rank, sign);

                    if (atk.pawn(them, sq) & our_pawns).any() {
                        mg += sign * self.params.passed_supported_mg[0];
                        eg += sign
                            * (self.params.passed_supported_eg_base[0]
                                + infra::to_i32(rel_rank)
                                    * self.params.passed_supported_eg_per_rank[0]);
                        tr_mg!(self, passed_supported_mg, 0, sign);
                        tr_eg!(self, passed_supported_eg_base, 0, sign);
                        tr_eg!(
                            self,
                            passed_supported_eg_per_rank,
                            0,
                            sign * infra::to_i32(rel_rank)
                        );
                    }

                    // NB: the passed-pawn "free stop / safe stop" bonuses depend
                    // on non-pawn occupancy and enemy attacks, so they are NOT
                    // computed here — this function's result is cached by a
                    // pawn-structure-only key (Phase 3.14 fix). They are scored
                    // per-evaluation in `eval_passed_pawn_advance` instead.
                } else if rel_rank >= 3
                    && (atk.pawn(them, sq) & our_pawns).any()
                    && (their_pawns
                        & adjacent
                        & FORWARD_RANKS[us as usize][SQUARE_RANK[sq.index()]])
                    .is_empty()
                {
                    mg += sign * self.params.passed_candidate_mg[0];
                    eg += sign * self.params.passed_candidate_eg[0];
                    tr_mg!(self, passed_candidate_mg, 0, sign);
                    tr_eg!(self, passed_candidate_eg, 0, sign);
                }

                let file_bb = FILE_BBS[file];
                let is_doubled = (our_pawns & file_bb).more_than_one();
                let is_isolated = (our_pawns & adjacent).is_empty();
                if is_doubled {
                    mg -= sign * self.params.pawn_doubled_mg[0];
                    eg -= sign * self.params.pawn_doubled_eg[0];
                    tr_mg!(self, pawn_doubled_mg, 0, -sign);
                    tr_eg!(self, pawn_doubled_eg, 0, -sign);
                }
                if is_isolated {
                    mg -= sign * self.params.pawn_isolated_mg[0];
                    eg -= sign * self.params.pawn_isolated_eg[0];
                    tr_mg!(self, pawn_isolated_mg, 0, -sign);
                    tr_eg!(self, pawn_isolated_eg, 0, -sign);
                }
                // Doubled *and* isolated — an extra penalty on top (Phase 3.8,
                // seeded 0).
                if is_doubled && is_isolated {
                    mg -= sign * self.params.pawn_doubled_isolated_mg[0];
                    eg -= sign * self.params.pawn_doubled_isolated_eg[0];
                    tr_mg!(self, pawn_doubled_isolated_mg, 0, -sign);
                    tr_eg!(self, pawn_doubled_isolated_eg, 0, -sign);
                }
                // Supported (defended diagonally from behind by an own pawn) —
                // rank-scaled. (Historical param name is `pawn_connected_*`.)
                if (atk.pawn(them, sq) & our_pawns).any() {
                    mg += sign * self.params.pawn_connected_mg[rel_rank];
                    eg += sign * self.params.pawn_connected_eg[rel_rank];
                    tr_mg!(self, pawn_connected_mg, rel_rank, sign);
                    tr_eg!(self, pawn_connected_eg, rel_rank, sign);
                }
                // Phalanx (Phase 7.4, seeded 0): an own pawn directly beside
                // this one on the same rank — the west (sq-1) or east (sq+1)
                // neighbour, guarded against file wrap. Distinct from support:
                // phalanx pawns do not defend each other.
                let has_phalanx = (file > 0
                    && (our_pawns & Bitboard::from(Square(sq.0 - 1))).any())
                    || (file < 7 && (our_pawns & Bitboard::from(Square(sq.0 + 1))).any());
                if has_phalanx {
                    mg += sign * self.params.pawn_phalanx_mg[rel_rank];
                    eg += sign * self.params.pawn_phalanx_eg[rel_rank];
                    tr_mg!(self, pawn_phalanx_mg, rel_rank, sign);
                    tr_eg!(self, pawn_phalanx_eg, rel_rank, sign);
                }
                // Pawn lever: our pawn that attacks an enemy pawn (Phase 3.8,
                // seeded 0).
                if (atk.pawn(us, sq) & their_pawns).any() {
                    mg += sign * self.params.pawn_lever_mg[0];
                    eg += sign * self.params.pawn_lever_eg[0];
                    tr_mg!(self, pawn_lever_mg, 0, sign);
                    tr_eg!(self, pawn_lever_eg, 0, sign);
                }

                let stop_sq = if us == Color::White {
                    sq.0.checked_add(8)
                } else {
                    sq.0.checked_sub(8)
                };
                if (our_pawns & PASSED_PAWN_MASKS[them as usize][sq.index()] & adjacent).is_empty()
                    && let Some(stop) = stop_sq.filter(|sq| *sq < 64)
                    && (atk.pawn(us, Square(stop)) & their_pawns).any()
                {
                    mg -= sign * self.params.pawn_backward_mg[0];
                    eg -= sign * self.params.pawn_backward_eg[0];
                    tr_mg!(self, pawn_backward_mg, 0, -sign);
                    tr_eg!(self, pawn_backward_eg, 0, -sign);
                }
            }

            // Pawn islands (Phase 3.12, seeded 0): number of maximal groups of
            // own pawns on consecutive files. Penalty grows with fragmentation.
            let mut file_mask = 0u16;
            for (f, &file_bb) in FILE_BBS.iter().enumerate().take(8) {
                if (our_pawns & file_bb).any() {
                    file_mask |= 1 << f;
                }
            }
            let islands = infra::to_i32((file_mask & !(file_mask << 1)).count_ones());
            if islands != 0 {
                mg -= sign * islands * self.params.pawn_islands_mg[0];
                eg -= sign * islands * self.params.pawn_islands_eg[0];
                tr_mg!(self, pawn_islands_mg, 0, -sign * islands);
                tr_eg!(self, pawn_islands_eg, 0, -sign * islands);
            }
        }

        self.pawn_table[slot] = PawnEntry {
            key,
            mg,
            eg,
            passed: *passed,
            attacks: *attacks,
        };
        (mg, eg)
    }
}
