//! Threats: pawn and piece attacks on enemy pieces, weakly defended and
//! hanging pieces, safe pawn pushes, weak pieces and restricted squares.

use super::Evaluator;
use super::attacks::AttackMaps;
use super::trace::{tr_eg, tr_mg};
use crate::board::{Bitboard, Board, Color, Piece};
use crate::infra;

impl Evaluator {
    pub(super) fn eval_threats(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        maps: &AttackMaps,
        pawn_attacks: &[Bitboard; 2],
        pawns: &[Bitboard; 2],
        occupied: Bitboard,
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let them = !color;
        let own_pawns = pawns[color as usize];
        let their_pawns = pawns[them as usize];

        let mut threats = pawn_attacks[color as usize] & board.color_occ(them);
        while threats.any() {
            let sq = threats.pop_lsb();
            match board.piece_type_at(sq) {
                Some(Piece::Knight | Piece::Bishop) => {
                    *mg += sign * self.params.threat_minor_mg[0];
                    *eg += sign * self.params.threat_minor_eg[0];
                    tr_mg!(self, threat_minor_mg, 0, sign);
                    tr_eg!(self, threat_minor_eg, 0, sign);
                }
                Some(Piece::Rook) => {
                    *mg += sign * self.params.threat_rook_mg[0];
                    *eg += sign * self.params.threat_rook_eg[0];
                    tr_mg!(self, threat_rook_mg, 0, sign);
                    tr_eg!(self, threat_rook_eg, 0, sign);
                }
                Some(Piece::Queen) => {
                    *mg += sign * self.params.threat_queen_mg[0];
                    *eg += sign * self.params.threat_queen_eg[0];
                    tr_mg!(self, threat_queen_mg, 0, sign);
                    tr_eg!(self, threat_queen_eg, 0, sign);
                }
                _ => {}
            }
        }

        // ---- Threats package v2 (Phase 3.6); every weight seeded 0, so
        // these contribute nothing until Phase 4 tunes them (bench unchanged).
        let ci = color as usize;
        let ti = them as usize;
        let enemy_occ = board.color_occ(them);

        // Threat by minor / rook, indexed by victim piece type.
        let our_minor_att = maps.attacked_by[ci][Piece::Knight as usize]
            | maps.attacked_by[ci][Piece::Bishop as usize];
        let mut tb = our_minor_att & enemy_occ;
        while tb.any() {
            let sq = tb.pop_lsb();
            if let Some(v) = board.piece_type_at(sq) {
                *mg += sign * self.params.threat_by_minor_mg[v as usize];
                *eg += sign * self.params.threat_by_minor_eg[v as usize];
                tr_mg!(self, threat_by_minor_mg, v as usize, sign);
                tr_eg!(self, threat_by_minor_eg, v as usize, sign);
            }
        }
        let mut tb = maps.attacked_by[ci][Piece::Rook as usize] & enemy_occ;
        while tb.any() {
            let sq = tb.pop_lsb();
            if let Some(v) = board.piece_type_at(sq) {
                *mg += sign * self.params.threat_by_rook_mg[v as usize];
                *eg += sign * self.params.threat_by_rook_eg[v as usize];
                tr_mg!(self, threat_by_rook_mg, v as usize, sign);
                tr_eg!(self, threat_by_rook_eg, v as usize, sign);
            }
        }

        // Hanging refinement: enemy piece (non-king) we attack that is
        // weakly defended — undefended, or doubly-attacked yet defended
        // only once. Generalises the flat hanging penalty (still active).
        let mut hb = enemy_occ & !Bitboard::from(board.king_sq(them));
        while hb.any() {
            let sq = hb.pop_lsb();
            let bb = Bitboard::from(sq);
            let att1 = (maps.attacked[ci] & bb).any();
            let att2 = (maps.attacked2[ci] & bb).any();
            let def1 = (maps.attacked[ti] & bb).any();
            let def2 = (maps.attacked2[ti] & bb).any();
            if ((att1 && !def1) || (att2 && def1 && !def2))
                && let Some(v) = board.piece_type_at(sq)
            {
                *mg += sign * self.params.threat_hanging_refined_mg[v as usize];
                *eg += sign * self.params.threat_hanging_refined_eg[v as usize];
                tr_mg!(self, threat_hanging_refined_mg, v as usize, sign);
                tr_eg!(self, threat_hanging_refined_eg, v as usize, sign);
            }
        }

        // Threat by safe pawn push: enemy non-pawn pieces a pawn would
        // attack after a safe single/double push (push square not attacked
        // by an enemy pawn).
        let empty = !occupied;
        let push1 = if color == Color::White {
            own_pawns.north() & empty
        } else {
            own_pawns.south() & empty
        };
        let push2 = if color == Color::White {
            (push1 & Bitboard::RANK_3).north() & empty
        } else {
            (push1 & Bitboard::RANK_6).south() & empty
        };
        let safe_push = (push1 | push2) & !pawn_attacks[ti];
        let push_attacks = if color == Color::White {
            safe_push.north_east() | safe_push.north_west()
        } else {
            safe_push.south_east() | safe_push.south_west()
        };
        let push_targets = infra::to_i32((push_attacks & enemy_occ & !their_pawns).count());
        if push_targets != 0 {
            *mg += sign * push_targets * self.params.threat_safe_pawn_push_mg[0];
            *eg += sign * push_targets * self.params.threat_safe_pawn_push_eg[0];
            tr_mg!(self, threat_safe_pawn_push_mg, 0, sign * push_targets);
            tr_eg!(self, threat_safe_pawn_push_eg, 0, sign * push_targets);
        }

        // Weak piece: our piece attacked by a strictly lower-valued enemy
        // piece (penalty for us).
        let their_minor_att = maps.attacked_by[ti][Piece::Knight as usize]
            | maps.attacked_by[ti][Piece::Bishop as usize];
        let weak_minor = (board.pieces(color, Piece::Knight) | board.pieces(color, Piece::Bishop))
            & pawn_attacks[ti];
        let weak_rook = board.pieces(color, Piece::Rook) & (pawn_attacks[ti] | their_minor_att);
        let weak_queen = board.pieces(color, Piece::Queen)
            & (pawn_attacks[ti] | their_minor_att | maps.attacked_by[ti][Piece::Rook as usize]);
        let weak_cnt = infra::to_i32((weak_minor | weak_rook | weak_queen).count());
        if weak_cnt != 0 {
            *mg -= sign * weak_cnt * self.params.threat_weak_piece_mg[0];
            *eg -= sign * weak_cnt * self.params.threat_weak_piece_eg[0];
            tr_mg!(self, threat_weak_piece_mg, 0, -sign * weak_cnt);
            tr_eg!(self, threat_weak_piece_eg, 0, -sign * weak_cnt);
        }

        // Restricted squares: squares both sides attack that the enemy does
        // not strongly protect (no enemy pawn attack, not doubly attacked).
        let strongly_protected = pawn_attacks[ti] | maps.attacked2[ti];
        let restricted =
            infra::to_i32((maps.attacked[ti] & maps.attacked[ci] & !strongly_protected).count());
        if restricted != 0 {
            *mg += sign * restricted * self.params.threat_restricted_mg[0];
            *eg += sign * restricted * self.params.threat_restricted_eg[0];
            tr_mg!(self, threat_restricted_mg, 0, sign * restricted);
            tr_eg!(self, threat_restricted_eg, 0, sign * restricted);
        }
    }

    pub(super) fn eval_hanging_pieces(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        mg: &mut i32,
        eg: &mut i32,
        attacked: &[Bitboard; 2],
    ) {
        let them = !color;
        let mut pieces = board.color_occ(color)
            & !board.pieces(color, Piece::Pawn)
            & !board.pieces(color, Piece::King);
        while pieces.any() {
            let sq = pieces.pop_lsb();
            let Some(piece) = board.piece_type_at(sq) else {
                continue;
            };
            let sq_bb = Bitboard::from(sq);
            let is_attacked = (attacked[them as usize] & sq_bb).any();
            let is_defended = (attacked[color as usize] & sq_bb).any();
            if !is_attacked || is_defended {
                continue;
            }
            let penalty = match piece {
                Piece::Knight | Piece::Bishop => self.params.hanging_minor[0],
                Piece::Rook => self.params.hanging_rook[0],
                Piece::Queen => self.params.hanging_queen[0],
                _ => 0,
            };
            *mg -= sign * penalty;
            *eg -= sign * penalty;
            // The same flat penalty enters both mg and eg, so trace both.
            match piece {
                Piece::Knight | Piece::Bishop => {
                    tr_mg!(self, hanging_minor, 0, -sign);
                    tr_eg!(self, hanging_minor, 0, -sign);
                }
                Piece::Rook => {
                    tr_mg!(self, hanging_rook, 0, -sign);
                    tr_eg!(self, hanging_rook, 0, -sign);
                }
                Piece::Queen => {
                    tr_mg!(self, hanging_queen, 0, -sign);
                    tr_eg!(self, hanging_queen, 0, -sign);
                }
                _ => {}
            }
        }
    }
}
