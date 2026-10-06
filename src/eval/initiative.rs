//! Initiative: a complexity term that reads the sign of the running endgame
//! sum, so it must stay the last term piece activity adds.

use super::Evaluator;
use super::pawns::{FILE_BBS, SQUARE_FILE};
use super::trace::tr_eg;
use crate::board::{Board, Color, Piece};
use crate::infra;

impl Evaluator {
    /// Initiative / complexity (Phase 3.10): nudges the endgame score away
    /// from (or toward) a draw based on a cheap complexity proxy — total
    /// pawns, king-file separation, and pawns on both flanks — mirroring
    /// SF's `Initiative` adjustment. Seeded 0, so it is a no-op until tuned.
    pub(super) fn eval_initiative(&self, board: &Board, eg: &mut i32) {
        let pawns =
            board.pieces(Color::White, Piece::Pawn) | board.pieces(Color::Black, Piece::Pawn);
        let pawn_count = infra::to_i32(pawns.count());
        let kf_w = infra::to_i32(SQUARE_FILE[board.king_sq(Color::White).index()]);
        let kf_b = infra::to_i32(SQUARE_FILE[board.king_sq(Color::Black).index()]);
        let king_file_distance = (kf_w - kf_b).abs();
        let queenside = FILE_BBS[0] | FILE_BBS[1] | FILE_BBS[2] | FILE_BBS[3];
        let kingside = FILE_BBS[4] | FILE_BBS[5] | FILE_BBS[6] | FILE_BBS[7];
        let both_flanks = (pawns & queenside).any() && (pawns & kingside).any();
        let complexity = pawn_count + king_file_distance + i32::from(both_flanks);
        let outcome_sign = (*eg > 0) as i32 - (*eg < 0) as i32;
        let n = outcome_sign * complexity;
        *eg += n * self.params.initiative_weight[0];
        tr_eg!(self, initiative_weight, 0, n);
    }
}
