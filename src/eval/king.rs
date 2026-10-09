//! King safety: the attacker-unit danger index with its table, shelter and
//! storm, and the central-king danger.

use super::attacks::KsMaps;
use super::pawns::{FILE_BBS, FORWARD_RANKS, SQUARE_FILE, SQUARE_RANK};
use super::trace::tr_mg;
use super::{Evaluator, color_sign, relative_rank};
use crate::board::{ATTACKS, Bitboard, Board, CastlingRights, Color, Piece, Square};
use crate::infra;

impl Evaluator {
    pub(super) fn eval_king_safety(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        mg: &mut i32,
        pawns: &[Bitboard; 2],
        maps: &KsMaps,
    ) {
        // One LazyLock resolution covers every king-zone attack lookup below.
        let atk = &*ATTACKS;
        let them = !color;
        let king = board.king_sq(color);
        let king_bb = Bitboard::from(king);
        let king_attacks = atk.king(king);
        let mut zone = king_attacks | king_bb;
        zone |= if color == Color::White {
            king_attacks.north()
        } else {
            king_attacks.south()
        };

        // Single king-danger accumulator (Phase 3.5). The attacker-unit sum is
        // the historical term; every other input is multiplied by a weight
        // seeded 0, so `danger == units` today and bench is unchanged. Inputs
        // select the (non-linear) safety-table bucket, so they are SPSA-tuned
        // later; the table itself is Texel-tuned.
        let mut danger = 0i32;
        for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
            // 9.7.5(d): the per-piece unit is invariant across this piece's
            // whole bitboard, so resolve it once instead of re-matching for
            // every piece that attacks the zone.
            let unit = match piece {
                Piece::Knight | Piece::Bishop => self.params.king_safety_unit_minor[0],
                Piece::Rook => self.params.king_safety_unit_rook[0],
                Piece::Queen => self.params.king_safety_unit_queen[0],
                _ => 0,
            };
            let mut pieces = board.pieces(them, piece);
            while pieces.any() {
                let sq = pieces.pop_lsb();
                if (maps.their_from_sq(sq) & zone).any() {
                    danger += unit;
                }
            }
        }

        // Weak king-ring squares: zone squares the enemy attacks but we do not
        // defend (or defend only once while doubly attacked).
        let weak = zone
            & maps.attacked[them as usize]
            & (!maps.attacked[color as usize] | maps.attacked2[them as usize]);
        danger += self.params.ks_weak_ring[0] * infra::to_i32(weak.count());

        // Safe checks: squares from which an enemy piece type could check our
        // king, that the enemy actually attacks with that type and we do not
        // defend (and are not occupied by an enemy piece).
        let occ = maps.occupied;
        let safe = !maps.attacked[color as usize] & !maps.their_occ;
        let knight_from = atk.knight(king);
        let bishop_from = atk.bishop(king, occ);
        let rook_from = atk.rook(king, occ);
        let knight_checks = knight_from & maps.attacked_by_them[Piece::Knight as usize] & safe;
        let bishop_checks = bishop_from & maps.attacked_by_them[Piece::Bishop as usize] & safe;
        let rook_checks = rook_from & maps.attacked_by_them[Piece::Rook as usize] & safe;
        let queen_checks =
            (bishop_from | rook_from) & maps.attacked_by_them[Piece::Queen as usize] & safe;
        danger += self.params.ks_safe_check_knight[0] * infra::to_i32(knight_checks.count());
        danger += self.params.ks_safe_check_bishop[0] * infra::to_i32(bishop_checks.count());
        danger += self.params.ks_safe_check_rook[0] * infra::to_i32(rook_checks.count());
        danger += self.params.ks_safe_check_queen[0] * infra::to_i32(queen_checks.count());

        // King-flank pressure: enemy attacks minus our defenses over the three
        // files around the king (clamped non-negative).
        let king_file = infra::to_i32(SQUARE_FILE[king.index()]);
        let mut flank = Bitboard::EMPTY;
        for df in -1..=1 {
            let f = king_file + df;
            if (0..8).contains(&f) {
                flank |= FILE_BBS[infra::to_usize(f)];
            }
        }
        let flank_attack = infra::to_i32((maps.attacked[them as usize] & flank).count());
        let flank_defense = infra::to_i32((maps.attacked[color as usize] & flank).count());
        danger += self.params.ks_flank_attack[0] * (flank_attack - flank_defense).max(0);

        // Pawnless flank: no pawns of either colour on the king's flank.
        let all_pawns = pawns[Color::White as usize] | pawns[Color::Black as usize];
        if (all_pawns & flank).is_empty() {
            danger += self.params.ks_pawnless_flank[0];
        }

        // Queen relief: a danger *reduction* when the attacker has no queen.
        if board.pieces(them, Piece::Queen).is_empty() {
            danger -= self.params.ks_queen_relief[0];
        }
        let _ = maps.own_occ; // reserved for the Phase 5 blocker/pin danger input.

        // Shelter/storm folded into the danger index (Phase 6.2.1, seeded 0):
        // a small integer "pawn-cover deficit" — missing shelter files (own
        // file 2, adjacent 1; same castled-flank gate as the linear shelter
        // term) plus advanced storm pawns (rel_rank − 2 each, rel ≥ 3). The
        // deficit multiplies into the nonlinear table lookup, expressing the
        // "exposed king × piece pressure" interaction the linear terms cannot.
        if self.params.ks_shelter_storm[0] != 0 {
            let kf = infra::to_i32(SQUARE_FILE[king.index()]);
            let krank = infra::to_i32(SQUARE_RANK[king.index()]);
            let mut deficit = 0i32;
            if kf <= 2 || kf >= 5 {
                for df in -1..=1 {
                    let f = kf + df;
                    if !(0..8).contains(&f) {
                        continue;
                    }
                    let file_pawns = pawns[color as usize] & FILE_BBS[infra::to_usize(f)];
                    if (file_pawns & FORWARD_RANKS[color as usize][infra::to_usize(krank)])
                        .is_empty()
                    {
                        deficit += if df == 0 { 2 } else { 1 };
                    }
                }
            }
            let mut sf = Bitboard::EMPTY;
            for df in -1..=1 {
                let f = kf + df;
                if (0..8).contains(&f) {
                    sf |= FILE_BBS[infra::to_usize(f)];
                }
            }
            let mut storm_pawns = pawns[them as usize] & sf;
            while storm_pawns.any() {
                let p = storm_pawns.pop_lsb();
                let rel = relative_rank(them, p) as i32;
                if rel >= 3 {
                    deficit += rel - 2;
                }
            }
            danger += self.params.ks_shelter_storm[0] * deficit;
        }

        // Non-linear table lookup: trace one-hot on the bucket actually read.
        let safety_idx = infra::to_usize(
            danger.clamp(0, infra::to_i32(self.params.king_safety_table.len()) - 1),
        );
        *mg -= sign * self.params.king_safety_table[safety_idx];
        tr_mg!(self, king_safety_table, safety_idx, -sign);

        let king_file = infra::to_i32(SQUARE_FILE[king.index()]);
        if king_file <= 2 || king_file >= 5 {
            let king_rank = infra::to_i32(SQUARE_RANK[king.index()]);
            for df in -1..=1 {
                let file = king_file + df;
                if !(0..8).contains(&file) {
                    continue;
                }
                let file_pawns = pawns[color as usize] & FILE_BBS[infra::to_usize(file)];
                let in_front =
                    file_pawns & FORWARD_RANKS[color as usize][infra::to_usize(king_rank)];
                if in_front.is_empty() {
                    if df == 0 {
                        *mg -= sign * self.params.shelter_missing_file_mg[0];
                        tr_mg!(self, shelter_missing_file_mg, 0, -sign);
                    } else {
                        *mg -= sign * self.params.shelter_missing_adjacent_mg[0];
                        tr_mg!(self, shelter_missing_adjacent_mg, 0, -sign);
                    }
                } else {
                    let pawn_sq = if color == Color::White {
                        in_front.lsb()
                    } else {
                        in_front.msb()
                    };
                    let distance = if color == Color::White {
                        infra::to_i32(SQUARE_RANK[pawn_sq.index()]) - king_rank
                    } else {
                        king_rank - infra::to_i32(SQUARE_RANK[pawn_sq.index()])
                    };
                    if distance == 1 {
                        *mg += sign * self.params.shelter_dist1_mg[0];
                        tr_mg!(self, shelter_dist1_mg, 0, sign);
                    } else if distance == 2 {
                        *mg += sign * self.params.shelter_dist2_mg[0];
                        tr_mg!(self, shelter_dist2_mg, 0, sign);
                    }
                }
            }
        }

        let enemy_pawns = pawns[them as usize];
        let mut storm_files = Bitboard::EMPTY;
        let king_file = infra::to_i32(SQUARE_FILE[king.index()]);
        for df in -1..=1 {
            let file = king_file + df;
            if (0..8).contains(&file) {
                storm_files |= FILE_BBS[infra::to_usize(file)];
            }
        }
        let mut storm = enemy_pawns & storm_files;
        while storm.any() {
            let pawn = storm.pop_lsb();
            let rel = relative_rank(them, pawn) as i32;
            if rel >= 3 {
                if SQUARE_FILE[pawn.index()] == SQUARE_FILE[king.index()] {
                    *mg -= sign * (rel * self.params.storm_file_weight[0]);
                    tr_mg!(self, storm_file_weight, 0, -sign * rel);
                } else {
                    *mg -= sign * (rel * self.params.storm_adjacent_weight[0]);
                    tr_mg!(self, storm_adjacent_weight, 0, -sign * rel);
                }
            }
        }
    }

    /// Central-king / lost-castling danger (Phase 3.10): a king still on its
    /// home square, on a central file, with all castling rights for that
    /// side gone — separate from the king-ring attack model (3.5), which
    /// scores nothing until attackers actually arrive.
    pub(super) fn eval_king_centrality_danger(&self, board: &Board, mg: &mut i32) {
        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let ksq = board.king_sq(color);
            let home_sq = Square([4u8, 60u8][color as usize]);
            let own_castling_all = match color {
                Color::White => CastlingRights::WHITE_ALL,
                Color::Black => CastlingRights::BLACK_ALL,
            };
            if ksq == home_sq && !board.castling().has(own_castling_all) {
                *mg -= sign * self.params.king_centrality_danger_mg[0];
                tr_mg!(self, king_centrality_danger_mg, 0, -sign);
            }
        }
    }
}
