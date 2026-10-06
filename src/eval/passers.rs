//! Passed pawns beyond the cached structure: stop-square and path bonuses,
//! the unstoppable-passer race, rooks behind and blockaders, and king
//! proximity.

use super::pawns::{FILE_BBS, FORWARD_RANKS, KING_DISTANCE, SQUARE_FILE, SQUARE_RANK};
use super::trace::{tr_eg, tr_mg};
use super::{Evaluator, color_sign, forward_square, relative_rank};
use crate::board::movegen;
use crate::board::{Bitboard, Board, Color, Piece, Square};
use crate::infra;

impl Evaluator {
    /// Passed-pawn advance bonuses that depend on non-pawn occupancy: a clear
    /// stop square ("free stop") and an unattacked stop square ("safe stop").
    /// These are deliberately kept out of `eval_pawns` — whose result is cached
    /// by a pawn-structure-only key — so the whole evaluation stays a pure
    /// function of the position and the eval cache is exact (Phase 3.14).
    pub(super) fn eval_passed_pawn_advance(
        &self,
        board: &Board,
        passed: &[Bitboard; 2],
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let occupied = board.occupied();
        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let them = !color;
            let mut pp = passed[color as usize];
            while pp.any() {
                let sq = pp.pop_lsb();
                let rel_rank = relative_rank(color, sq) as i32;
                if let Some(stop) = forward_square(color, sq)
                    && (occupied & Bitboard::from(stop)).is_empty()
                {
                    *mg += sign * rel_rank * self.params.passed_freestop_mg_per_rank[0];
                    *eg += sign * rel_rank * self.params.passed_freestop_eg_per_rank[0];
                    tr_mg!(self, passed_freestop_mg_per_rank, 0, sign * rel_rank);
                    tr_eg!(self, passed_freestop_eg_per_rank, 0, sign * rel_rank);
                    if !board.is_attacked_by_with_occ(stop, them, occupied) {
                        *eg += sign * rel_rank * self.params.passed_safestop_eg_per_rank[0];
                        tr_eg!(self, passed_safestop_eg_per_rank, 0, sign * rel_rank);
                    }
                }
                // Whole-path weighting (Phase 6.2.1, seeded 0): the free/safe-
                // stop above scores only the ONE square ahead; SF/Ethereal
                // weight the entire path to promotion. free path = every path
                // square empty; safe path = additionally, no path square
                // attacked by the enemy (scanned only when the path is free,
                // mirroring the stop-square nesting).
                let path = FORWARD_RANKS[color as usize][SQUARE_RANK[sq.index()]]
                    & FILE_BBS[SQUARE_FILE[sq.index()]];
                if (path & occupied).is_empty() {
                    *mg += sign * rel_rank * self.params.passed_freepath_mg_per_rank[0];
                    *eg += sign * rel_rank * self.params.passed_freepath_eg_per_rank[0];
                    tr_mg!(self, passed_freepath_mg_per_rank, 0, sign * rel_rank);
                    tr_eg!(self, passed_freepath_eg_per_rank, 0, sign * rel_rank);
                    let mut path_sqs = path;
                    let mut safe = true;
                    while path_sqs.any() {
                        let psq = path_sqs.pop_lsb();
                        if board.is_attacked_by_with_occ(psq, them, occupied) {
                            safe = false;
                            break;
                        }
                    }
                    if safe {
                        *eg += sign * rel_rank * self.params.passed_safepath_eg_per_rank[0];
                        tr_eg!(self, passed_safepath_eg_per_rank, 0, sign * rel_rank);
                    }
                }
            }
        }
    }

    pub(super) fn eval_unstoppable_passers(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        passed: &[Bitboard; 2],
        occupied: Bitboard,
        eg: &mut i32,
    ) {
        let them = !color;
        // Unstoppable passer (rule of the square): a passed pawn with a clear
        // path whose promotion the enemy king cannot reach in time. eg-only.
        // Phase 7.4: the square rule only decides the race when the *king* is
        // the sole defender — a knight/bishop/rook/queen can interpose or
        // capture on the path even when the king cannot arrive. Restrict to a
        // defender with no non-pawn material (passed pawns are already
        // immune to enemy pawns by definition).
        let enemy_king = board.king_sq(them);
        let enemy_to_move = board.side_to_move() == them;
        let defender_has_pieces = board.has_non_pawn_material(them);
        let mut pp = passed[color as usize];
        let mut unstoppable = 0i32;
        while pp.any() {
            let ps = pp.pop_lsb();
            if defender_has_pieces {
                continue;
            }
            let promo = if color == Color::White {
                56 + (ps.index() % 8)
            } else {
                ps.index() % 8
            };
            let promo_sq = Square(infra::to_u8(promo));
            let path = movegen::between(ps, promo_sq) | Bitboard::from(promo_sq);
            if (path & occupied).any() {
                continue;
            }
            let rel = relative_rank(color, ps) as i32;
            let pawn_steps = (7 - rel) - if rel == 1 { 1 } else { 0 };
            let king_steps = KING_DISTANCE[enemy_king.index()][promo] as i32;
            if king_steps > pawn_steps - if enemy_to_move { 0 } else { 1 } {
                unstoppable += 1;
            }
        }
        if unstoppable != 0 {
            *eg += sign * unstoppable * self.params.unstoppable_passer_eg[0];
            tr_eg!(self, unstoppable_passer_eg, 0, sign * unstoppable);
        }
    }

    pub(super) fn eval_rooks_behind_passers(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        passed: &[Bitboard; 2],
        mg: &mut i32,
        eg: &mut i32,
    ) {
        // Phase 7.4: iterate over the *passers*, not the friendly rooks. The old
        // form nested the enemy-rook check inside the friendly-rook loop, so the
        // enemy-rook-behind penalty was only seen when a friendly rook shared the
        // file, and doubled friendly rooks double-counted. Now each passer scores
        // "is any own rook behind it" (bonus) and "is any enemy rook behind it"
        // (penalty) independently and at most once — explicit blocker semantics.
        let them = !color;
        let own_rooks = board.pieces(color, Piece::Rook);
        let enemy_rooks = board.pieces(them, Piece::Rook);
        let mut pp = passed[color as usize];
        while pp.any() {
            let passer = pp.pop_lsb();
            let file = SQUARE_FILE[passer.index()];
            // "Behind" = same file, on the near side of the passer (toward our own
            // back rank) — the ranks the enemy travels *forward* through.
            let behind = FORWARD_RANKS[them as usize][SQUARE_RANK[passer.index()]] & FILE_BBS[file];
            if (own_rooks & behind).any() {
                *mg += sign * self.params.rook_behind_passer_mg[0];
                *eg += sign * self.params.rook_behind_passer_eg[0];
                tr_mg!(self, rook_behind_passer_mg, 0, sign);
                tr_eg!(self, rook_behind_passer_eg, 0, sign);
            }
            if (enemy_rooks & behind).any() {
                *mg -= sign * self.params.enemy_rook_behind_passer_mg[0];
                *eg -= sign * self.params.enemy_rook_behind_passer_eg[0];
                tr_mg!(self, enemy_rook_behind_passer_mg, 0, -sign);
                tr_eg!(self, enemy_rook_behind_passer_eg, 0, -sign);
            }
        }
    }

    /// Passer/blockade detail needing piece squares (Phase 3.8; seeded 0):
    /// a penalty when our own passed pawn is blocked by an enemy piece on its
    /// stop square, and a bonus for our knight as the ideal blockader directly
    /// in front of an enemy passed pawn.
    pub(super) fn eval_passer_blockade(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        passed: &[Bitboard; 2],
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let them = !color;
        let enemy_occ = board.color_occ(them);
        let mut ours = passed[color as usize];
        while ours.any() {
            let sq = ours.pop_lsb();
            if let Some(stop) = forward_square(color, sq)
                && (enemy_occ & Bitboard::from(stop)).any()
            {
                *mg -= sign * self.params.blocked_passer_mg[0];
                *eg -= sign * self.params.blocked_passer_eg[0];
                tr_mg!(self, blocked_passer_mg, 0, -sign);
                tr_eg!(self, blocked_passer_eg, 0, -sign);
            }
        }

        let our_knights = board.pieces(color, Piece::Knight);
        let mut theirs = passed[them as usize];
        while theirs.any() {
            let sq = theirs.pop_lsb();
            if let Some(stop) = forward_square(them, sq)
                && (our_knights & Bitboard::from(stop)).any()
            {
                *mg += sign * self.params.ideal_blockader_mg[0];
                *eg += sign * self.params.ideal_blockader_eg[0];
                tr_mg!(self, ideal_blockader_mg, 0, sign);
                tr_eg!(self, ideal_blockader_eg, 0, sign);
            }
        }
    }

    pub(super) fn eval_passed_pawn_king_proximity(
        &self,
        board: &Board,
        passed: &[Bitboard; 2],
        eg: &mut i32,
    ) {
        for color in [Color::White, Color::Black] {
            let them = !color;
            let sign = color_sign(color);
            let own_king = board.king_sq(color);
            let enemy_king = board.king_sq(them);
            let mut pawns = passed[color as usize];
            while pawns.any() {
                let pawn = pawns.pop_lsb();
                let rel_rank = relative_rank(color, pawn) as i32;
                let own_dist = KING_DISTANCE[own_king.index()][pawn.index()] as i32;
                let enemy_dist = KING_DISTANCE[enemy_king.index()][pawn.index()] as i32;
                *eg += sign
                    * (enemy_dist - own_dist)
                    * (self.params.passer_proximity_base[0] + rel_rank);
                // Only `passer_proximity_base` is a tunable weight here; the
                // `+ rel_rank` term is a frozen constant (absorbed into `rest`).
                tr_eg!(
                    self,
                    passer_proximity_base,
                    0,
                    sign * (enemy_dist - own_dist)
                );
                #[cfg(feature = "texel")]
                {
                    self.trace.borrow_mut().frozen_eg += sign * (enemy_dist - own_dist) * rel_rank;
                }
            }
        }
    }
}
