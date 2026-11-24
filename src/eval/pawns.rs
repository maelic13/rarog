//! Pawn structure, cached by pawn key, and the file, rank, distance and
//! passed-pawn tables the other terms share.

use super::trace::{tr_eg, tr_mg};
use super::{Evaluator, PAWN_TABLE_SIZE, PawnEntry, color_sign, relative_rank};
use crate::board::attacks::AttackTables;
use crate::board::geometry::{
    ADJACENT_FILES, FILE_BBS, FORWARD_RANKS, PASSED_PAWN_MASKS, SQUARE_FILE, SQUARE_RANK,
};
use crate::board::{Bitboard, Board, Color, Piece, Square};
use crate::infra;

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
            // Read in place: copying the whole entry out would move the
            // cached shelters too, which this path does not need.
            let cached = &self.pawn_table[slot];
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
                // Each pawn's material and square, which `evaluate` leaves to
                // this entry (see `PSQ_PIECES`).
                #[cfg(not(feature = "texel"))]
                {
                    mg += sign * self.tables.mg[us as usize][Piece::Pawn as usize][sq.index()];
                    eg += sign * self.tables.eg[us as usize][Piece::Pawn as usize][sq.index()];
                }
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
                    // pawn-structure-only key. They are scored
                    // per-evaluation in `eval_passed_pawn_advance` instead.
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
                // Doubled *and* isolated — an extra penalty on top.
                if is_doubled && is_isolated {
                    mg -= sign * self.params.pawn_doubled_isolated_mg[0];
                    eg -= sign * self.params.pawn_doubled_isolated_eg[0];
                    tr_mg!(self, pawn_doubled_isolated_mg, 0, -sign);
                    tr_eg!(self, pawn_doubled_isolated_eg, 0, -sign);
                }
                // Supported (defended diagonally from behind by an own pawn) —
                // rank-scaled. (The parameter is named `pawn_connected_*`.)
                if (atk.pawn(them, sq) & our_pawns).any() {
                    mg += sign * self.params.pawn_connected_mg[rel_rank];
                    eg += sign * self.params.pawn_connected_eg[rel_rank];
                    tr_mg!(self, pawn_connected_mg, rel_rank, sign);
                    tr_eg!(self, pawn_connected_eg, rel_rank, sign);
                }
                // Phalanx: an own pawn directly beside
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
                let stop_sq = if us == Color::White {
                    sq.0.checked_add(8)
                } else {
                    sq.0.checked_sub(8)
                };
                let is_backward =
                    (our_pawns & PASSED_PAWN_MASKS[them as usize][sq.index()] & adjacent)
                        .is_empty()
                        && stop_sq
                            .filter(|sq| *sq < 64)
                            .is_some_and(|stop| (atk.pawn(us, Square(stop)) & their_pawns).any());
                if is_backward {
                    mg -= sign * self.params.pawn_backward_mg[0];
                    eg -= sign * self.params.pawn_backward_eg[0];
                    tr_mg!(self, pawn_backward_mg, 0, -sign);
                    tr_eg!(self, pawn_backward_eg, 0, -sign);
                }
                // A weak pawn no enemy pawn faces on its file can be attacked
                // down the file by rooks and queens; counted once per pawn.
                let unopposed =
                    (their_pawns & file_bb & FORWARD_RANKS[us as usize][SQUARE_RANK[sq.index()]])
                        .is_empty();
                if (is_isolated || is_backward) && unopposed {
                    mg -= sign * self.params.pawn_weak_unopposed_mg[0];
                    eg -= sign * self.params.pawn_weak_unopposed_eg[0];
                    tr_mg!(self, pawn_weak_unopposed_mg, 0, -sign);
                    tr_eg!(self, pawn_weak_unopposed_eg, 0, -sign);
                }
            }

            // Pawn islands: number of maximal groups of
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
            shelter: [None; 2],
        };
        (mg, eg)
    }
}
