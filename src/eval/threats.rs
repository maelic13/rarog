//! Threats: pawn and piece attacks on enemy pieces, weakly defended and
//! hanging pieces, safe pawn pushes, weak pieces, restricted squares, the
//! king attacking weak pieces and sliders bearing on the enemy queen.

use super::Evaluator;
use super::attacks::AttackMaps;
use super::trace::{tr_eg, tr_mg};
use crate::board::attacks::AttackTables;
use crate::board::{Bitboard, Board, Color, Piece};
use crate::infra;

impl Evaluator {
    pub(super) fn eval_threats(
        &self,
        board: &Board,
        atk: &AttackTables,
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

        let strongly_protected = strongly_protected(maps, pawn_attacks, color);

        // Restricted squares: squares both sides attack that the enemy does
        // not strongly protect.
        let restricted =
            infra::to_i32((maps.attacked[ti] & maps.attacked[ci] & !strongly_protected).count());
        if restricted != 0 {
            *mg += sign * restricted * self.params.threat_restricted_mg[0];
            *eg += sign * restricted * self.params.threat_restricted_eg[0];
            tr_mg!(self, threat_restricted_mg, 0, sign * restricted);
            tr_eg!(self, threat_restricted_eg, 0, sign * restricted);
        }

        let weak = weak_enemies(board, maps, color, strongly_protected);
        if king_threatens(maps, color, weak) {
            *mg += sign * self.params.threat_by_king_mg[0];
            *eg += sign * self.params.threat_by_king_eg[0];
            tr_mg!(self, threat_by_king_mg, 0, sign);
            tr_eg!(self, threat_by_king_eg, 0, sign);
        }

        let on_queen =
            slider_threats_on_queen(board, atk, maps, color, occupied, strongly_protected);
        if on_queen != 0 {
            *mg += sign * on_queen * self.params.threat_slider_on_queen_mg[0];
            *eg += sign * on_queen * self.params.threat_slider_on_queen_eg[0];
            tr_mg!(self, threat_slider_on_queen_mg, 0, sign * on_queen);
            tr_eg!(self, threat_slider_on_queen_eg, 0, sign * on_queen);
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

/// Squares the enemy strongly protects against `us`: an enemy pawn defends
/// them, or the enemy attacks them twice and we do not. The restricted, king
/// and queen threats share it; a square both sides attack twice is therefore
/// restricted, not protected.
fn strongly_protected(maps: &AttackMaps, pawn_attacks: &[Bitboard; 2], us: Color) -> Bitboard {
    let ci = us as usize;
    let ti = (!us) as usize;
    pawn_attacks[ti] | (maps.attacked2[ti] & !maps.attacked2[ci])
}

/// Enemy pieces, pawns and the king included, that we attack and the enemy
/// does not strongly protect.
fn weak_enemies(
    board: &Board,
    maps: &AttackMaps,
    us: Color,
    strongly_protected: Bitboard,
) -> Bitboard {
    board.color_occ(!us) & !strongly_protected & maps.attacked[us as usize]
}

/// Whether our king attacks a weak enemy piece.
fn king_threatens(maps: &AttackMaps, us: Color, weak: Bitboard) -> bool {
    (weak & maps.attacked_by[us as usize][Piece::King as usize]).any()
}

/// Squares from which one of our bishops or rooks would attack the enemy's
/// only queen, that we attack twice and that are safe: not our pawn, king or
/// queen, and not strongly protected (which covers the enemy pawn attacks).
/// Counted twice when that queen is the only one on the board; zero unless
/// the enemy has exactly one queen.
///
/// A counted square is attacked twice by us, so the strongly-protected set
/// can exclude it only through an enemy pawn attack.
fn slider_threats_on_queen(
    board: &Board,
    atk: &AttackTables,
    maps: &AttackMaps,
    us: Color,
    occupied: Bitboard,
    strongly_protected: Bitboard,
) -> i32 {
    let their_queens = board.pieces(!us, Piece::Queen);
    if their_queens.count() != 1 {
        return 0;
    }
    let q = their_queens.lsb();
    let ci = us as usize;
    let safe = !board.pieces(us, Piece::Pawn)
        & !(board.pieces(us, Piece::King) | board.pieces(us, Piece::Queen))
        & !strongly_protected;
    let lines = (maps.attacked_by[ci][Piece::Bishop as usize] & atk.bishop(q, occupied))
        | (maps.attacked_by[ci][Piece::Rook as usize] & atk.rook(q, occupied));
    let count = infra::to_i32((lines & safe & maps.attacked2[ci]).count());
    if board.pieces(us, Piece::Queen).is_empty() {
        count * 2
    } else {
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{ATTACKS, Square};

    fn setup(fen: &str) -> (Board, AttackMaps, [Bitboard; 2]) {
        let board = Board::from_fen(fen).unwrap_or_else(|e| panic!("bad test FEN {fen}: {e}"));
        let white = board.pieces(Color::White, Piece::Pawn);
        let black = board.pieces(Color::Black, Piece::Pawn);
        let pawn_attacks = [
            white.north_east() | white.north_west(),
            black.south_east() | black.south_west(),
        ];
        let mut maps = AttackMaps::new();
        maps.fill(&board, &ATTACKS, &pawn_attacks);
        (board, maps, pawn_attacks)
    }

    fn weak_of(fen: &str) -> Bitboard {
        let (board, maps, pawn_attacks) = setup(fen);
        let sp = strongly_protected(&maps, &pawn_attacks, Color::White);
        weak_enemies(&board, &maps, Color::White, sp)
    }

    fn king_threat_of(fen: &str) -> bool {
        let (board, maps, pawn_attacks) = setup(fen);
        let sp = strongly_protected(&maps, &pawn_attacks, Color::White);
        king_threatens(
            &maps,
            Color::White,
            weak_enemies(&board, &maps, Color::White, sp),
        )
    }

    fn queen_threats_of(fen: &str) -> i32 {
        let (board, maps, pawn_attacks) = setup(fen);
        let sp = strongly_protected(&maps, &pawn_attacks, Color::White);
        slider_threats_on_queen(&board, &ATTACKS, &maps, Color::White, board.occupied(), sp)
    }

    fn a5() -> Bitboard {
        Bitboard::from(Square(32))
    }

    #[test]
    fn weak_set_is_attacked_and_not_strongly_protected() {
        // The rook on a1 attacks an undefended knight.
        assert!((weak_of("4k3/8/8/n7/8/8/8/R3K3 w - - 0 1") & a5()).any());
        // A pawn defends it.
        assert!((weak_of("4k3/8/1p6/n7/8/8/8/R3K3 w - - 0 1") & a5()).is_empty());
        // Two defenders against one attacker protect it strongly ...
        assert!((weak_of("r3k3/2b5/8/n7/8/8/8/R3K3 w - - 0 1") & a5()).is_empty());
        // ... but not against two attackers.
        assert!((weak_of("r3k3/2b5/8/n7/8/8/3B4/R3K3 w - - 0 1") & a5()).any());
        // Pawns count as weak pieces.
        assert!((weak_of("4k3/8/8/p7/8/8/8/R3K3 w - - 0 1") & a5()).any());
    }

    #[test]
    fn king_threat_needs_a_weak_piece_next_to_the_king() {
        assert!(king_threat_of("4k3/8/8/3Kp3/8/8/8/8 w - - 0 1"));
        // The f6 pawn defends e5.
        assert!(!king_threat_of("4k3/8/5p2/3Kp3/8/8/8/8 w - - 0 1"));
        // Weak, but attacked by the rook, not by the king.
        assert!(!king_threat_of("4k3/8/8/n7/8/8/8/R3K3 w - - 0 1"));
    }

    #[test]
    fn slider_threat_counts_safe_double_attacked_squares_and_doubles_alone() {
        // The rook on h5 reaches d5 and h8 on the queen's lines; the knight
        // on f4 attacks d5 a second time. Only d5 counts, doubled because the
        // black queen is the only queen on the board.
        assert_eq!(queen_threats_of("3q4/8/k7/7R/5N2/8/8/K7 w - - 0 1"), 2);
        // With a white queen on the board as well it counts once.
        assert_eq!(queen_threats_of("3q4/8/k7/7R/5N2/8/8/K6Q w - - 0 1"), 1);
        // Without the knight no square is attacked twice.
        assert_eq!(queen_threats_of("3q4/8/k7/7R/8/8/8/K7 w - - 0 1"), 0);
    }

    #[test]
    fn slider_threat_safe_set_exclusions() {
        // d5 holds our pawn, our king or our queen.
        assert_eq!(queen_threats_of("3q4/8/k7/3P3R/5N2/8/8/K7 w - - 0 1"), 0);
        assert_eq!(queen_threats_of("3q4/8/k7/3K3R/5N2/8/8/8 w - - 0 1"), 0);
        assert_eq!(queen_threats_of("3q4/8/k7/3Q3R/5N2/8/8/K7 w - - 0 1"), 0);
        // An enemy pawn on c6 attacks d5 (strongly protected).
        assert_eq!(queen_threats_of("3q4/8/k1p5/7R/5N2/8/8/K7 w - - 0 1"), 0);
    }

    #[test]
    fn slider_threat_needs_exactly_one_enemy_queen() {
        assert_eq!(queen_threats_of("8/8/k7/7R/5N2/8/8/K7 w - - 0 1"), 0);
        assert_eq!(queen_threats_of("3q3q/8/k7/7R/5N2/8/8/K7 w - - 0 1"), 0);
    }
}
