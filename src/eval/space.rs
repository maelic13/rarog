//! Space: safe central squares behind the pawn chain.

use super::Evaluator;
use super::pawns::FILE_BBS;
use super::trace::tr_mg;
use crate::board::{Bitboard, Board, Color, Piece};
use crate::infra;

impl Evaluator {
    pub(super) fn eval_space(&self, board: &Board, pawn_attacks: &[Bitboard; 2], mg: &mut i32) {
        let center_files = FILE_BBS[2] | FILE_BBS[3] | FILE_BBS[4] | FILE_BBS[5];
        let white_space_ranks = Bitboard::RANK_2 | Bitboard::RANK_3 | Bitboard::RANK_4;
        let black_space_ranks = Bitboard::RANK_5 | Bitboard::RANK_6 | Bitboard::RANK_7;
        let white_space = center_files
            & white_space_ranks
            & !board.pieces(Color::White, Piece::Pawn)
            & !pawn_attacks[Color::Black as usize];
        let black_space = center_files
            & black_space_ranks
            & !board.pieces(Color::Black, Piece::Pawn)
            & !pawn_attacks[Color::White as usize];
        let space_net = infra::to_i32(white_space.count()) - infra::to_i32(black_space.count());
        *mg += space_net * self.params.space_weight[0];
        tr_mg!(self, space_weight, 0, space_net);

        // Space weighted by piece count (Phase 3.12, SF-style shape, seeded 0):
        // space matters more when more pieces remain to exploit it. The flat
        // `space_weight` term above stays active; Phase 4 retires it if this
        // shaped term earns its fit.
        let piece_count = |c: Color| -> i32 {
            let count = (board.pieces(c, Piece::Knight)
                | board.pieces(c, Piece::Bishop)
                | board.pieces(c, Piece::Rook)
                | board.pieces(c, Piece::Queen))
            .count();
            infra::to_i32(count)
        };
        let space_weighted = infra::to_i32(white_space.count()) * piece_count(Color::White)
            - infra::to_i32(black_space.count()) * piece_count(Color::Black);
        *mg += space_weighted * self.params.space_piece_mg[0];
        tr_mg!(self, space_piece_mg, 0, space_weighted);

        // SF-style refinement (Phase 6.2.1, seeded 0): safe central squares
        // BEHIND own pawns count extra — space is only usable when the pawn
        // chain shields it. behind = own pawns shifted 1–3 ranks toward the
        // own side; still weighted by piece count.
        let wp = board.pieces(Color::White, Piece::Pawn);
        let bp = board.pieces(Color::Black, Piece::Pawn);
        let behind_w = wp.south() | wp.south().south() | wp.south().south().south();
        let behind_b = bp.north() | bp.north().north() | bp.north().north().north();
        let space_behind = infra::to_i32((white_space & behind_w).count())
            * piece_count(Color::White)
            - infra::to_i32((black_space & behind_b).count()) * piece_count(Color::Black);
        *mg += space_behind * self.params.space_behind_piece_mg[0];
        tr_mg!(self, space_behind_piece_mg, 0, space_behind);
    }
}
