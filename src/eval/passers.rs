//! Passed pawns beyond the cached structure: stop-square and path bonuses,
//! the unstoppable-passer race, rooks behind and blockaders, and king
//! proximity.

use super::pawns::{FILE_BBS, FORWARD_RANKS, KING_DISTANCE, SQUARE_FILE, SQUARE_RANK};
use super::trace::{tr_eg, tr_mg};
use super::{Evaluator, color_sign, forward_square, relative_rank};
use crate::board::attacks::{knight_attacks_of, pawn_attacks_of};
use crate::board::movegen;
use crate::board::{ATTACKS, Bitboard, Board, Color, Piece, Square};
use crate::infra;

/// `set` moved one rank toward `color`'s promotion rank.
fn forward_one(color: Color, set: Bitboard) -> Bitboard {
    match color {
        Color::White => set.north(),
        Color::Black => set.south(),
    }
}

/// Every square on or ahead of a square of `set`, toward `color`'s
/// promotion rank.
fn forward_fill(color: Color, set: Bitboard) -> Bitboard {
    let mut fill = set.0;
    match color {
        Color::White => {
            fill |= fill << 8;
            fill |= fill << 16;
            fill |= fill << 32;
        }
        Color::Black => {
            fill |= fill >> 8;
            fill |= fill >> 16;
            fill |= fill >> 32;
        }
    }
    Bitboard(fill)
}

/// Every square `color`'s pieces attack under `occupied`.
fn attacks_of(board: &Board, color: Color, occupied: Bitboard) -> Bitboard {
    let atk = &*ATTACKS;
    let queens = board.pieces(color, Piece::Queen);
    let mut attacks = pawn_attacks_of(color, board.pieces(color, Piece::Pawn))
        | knight_attacks_of(board.pieces(color, Piece::Knight))
        | atk.king(board.king_sq(color));
    for sq in board.pieces(color, Piece::Bishop) | queens {
        attacks |= atk.bishop(sq, occupied);
    }
    for sq in board.pieces(color, Piece::Rook) | queens {
        attacks |= atk.rook(sq, occupied);
    }
    attacks
}

impl Evaluator {
    /// Passed-pawn advance bonuses that depend on non-pawn occupancy: a clear
    /// stop square ("free stop") and an unattacked stop square ("safe stop").
    /// These are deliberately kept out of `eval_pawns` — whose result is cached
    /// by a pawn-structure-only key — so the whole evaluation stays a pure
    /// function of the position (Phase 3.14).
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
            // The squares the enemy attacks, built at the first question and
            // shared by every stop and path square of this side's passers. A
            // square is attacked exactly when it is in the union of the enemy
            // pieces' attacks, by the symmetry of the attack tables.
            let mut enemy_attacks = None;
            let mut attacked = |squares: Bitboard| {
                (*enemy_attacks.get_or_insert_with(|| attacks_of(board, them, occupied)) & squares)
                    .any()
            };
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
                    debug_assert_eq!(
                        attacked(Bitboard::from(stop)),
                        board.is_attacked_by_with_occ(stop, them, occupied)
                    );
                    if !attacked(Bitboard::from(stop)) {
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
                    if !attacked(path) {
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
        if board.has_non_pawn_material(them) {
            return;
        }
        let enemy_king = board.king_sq(them);
        let enemy_to_move = board.side_to_move() == them;
        let mut pp = passed[color as usize];
        let mut unstoppable = 0i32;
        while pp.any() {
            let ps = pp.pop_lsb();
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
            // The rule of the square, counting the king's capture of the new
            // queen: moving first, the king catches the pawn from one square
            // further than the pawn's steps; moving second, from its steps.
            if king_steps > pawn_steps + if enemy_to_move { 1 } else { 0 } {
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
        // "Behind" = same file, on the near side of the passer (toward our own
        // back rank). A passer has a rook behind it exactly when it stands on
        // the squares ahead of that rook on its file, so each term counts the
        // passers in a forward fill of the rooks.
        let them = !color;
        let passers = passed[color as usize];
        let ahead_of = |rooks: Bitboard| forward_fill(color, forward_one(color, rooks));
        let own = infra::to_i32((passers & ahead_of(board.pieces(color, Piece::Rook))).count());
        let enemy = infra::to_i32((passers & ahead_of(board.pieces(them, Piece::Rook))).count());
        *mg += sign
            * (own * self.params.rook_behind_passer_mg[0]
                - enemy * self.params.enemy_rook_behind_passer_mg[0]);
        *eg += sign
            * (own * self.params.rook_behind_passer_eg[0]
                - enemy * self.params.enemy_rook_behind_passer_eg[0]);
        tr_mg!(self, rook_behind_passer_mg, 0, sign * own);
        tr_eg!(self, rook_behind_passer_eg, 0, sign * own);
        tr_mg!(self, enemy_rook_behind_passer_mg, 0, -sign * enemy);
        tr_eg!(self, enemy_rook_behind_passer_eg, 0, -sign * enemy);
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
        // Our passers whose stop square holds an enemy piece, and enemy
        // passers whose stop square holds our knight: the stop squares of a
        // set of passers are the set moved one rank forward.
        let them = !color;
        let blocked = infra::to_i32(
            (forward_one(color, passed[color as usize]) & board.color_occ(them)).count(),
        );
        let blockaders = infra::to_i32(
            (forward_one(them, passed[them as usize]) & board.pieces(color, Piece::Knight)).count(),
        );
        *mg += sign
            * (blockaders * self.params.ideal_blockader_mg[0]
                - blocked * self.params.blocked_passer_mg[0]);
        *eg += sign
            * (blockaders * self.params.ideal_blockader_eg[0]
                - blocked * self.params.blocked_passer_eg[0]);
        tr_mg!(self, blocked_passer_mg, 0, -sign * blocked);
        tr_eg!(self, blocked_passer_eg, 0, -sign * blocked);
        tr_mg!(self, ideal_blockader_mg, 0, sign * blockaders);
        tr_eg!(self, ideal_blockader_eg, 0, sign * blockaders);
    }

    /// Kings against a passer from its fourth rank on, measured at the square
    /// in front of it: the enemy king's distance there is a bonus, our king's
    /// distance there and to the square after it a penalty (our king escorts
    /// the pawn; the enemy king stops it). Distances are capped at five and
    /// weighted by how far the pawn has come, `5·rank − 13`. Each distance has
    /// its own weight: our king's support predicts outcomes far better than the
    /// enemy king's race, and one shared weight hides that. eg-only.
    pub(super) fn eval_passed_pawn_king_proximity(
        &self,
        board: &Board,
        passed: &[Bitboard; 2],
        eg: &mut i32,
    ) {
        let proximity = |king: Square, sq: Square| -> i32 {
            i32::from(KING_DISTANCE[king.index()][sq.index()]).min(5)
        };
        for color in [Color::White, Color::Black] {
            let them = !color;
            let sign = color_sign(color);
            let own_king = board.king_sq(color);
            let enemy_king = board.king_sq(them);
            let mut pawns = passed[color as usize];
            while pawns.any() {
                let pawn = pawns.pop_lsb();
                let rel_rank = i32::from(relative_rank(color, pawn));
                if rel_rank < 3 {
                    continue;
                }
                let weight = 5 * rel_rank - 13;
                // A passer below its last rank always has a square in front.
                let Some(block) = forward_square(color, pawn) else {
                    continue;
                };
                let them_count = proximity(enemy_king, block) * weight;
                let us_count = proximity(own_king, block) * weight;
                *eg += sign * them_count * self.params.passer_king_them_eg[0];
                *eg -= sign * us_count * self.params.passer_king_us_eg[0];
                tr_eg!(self, passer_king_them_eg, 0, sign * them_count);
                tr_eg!(self, passer_king_us_eg, 0, -sign * us_count);
                if rel_rank < 6
                    && let Some(next) = forward_square(color, block)
                {
                    let next_count = proximity(own_king, next) * weight;
                    *eg -= sign * next_count * self.params.passer_king_next_eg[0];
                    tr_eg!(self, passer_king_next_eg, 0, -sign * next_count);
                }
            }
        }
    }
}
