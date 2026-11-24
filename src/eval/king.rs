//! King safety: the king-danger index through its capped quadratic map,
//! shelter and storm, and the central-king danger.

use super::attacks::{KsMaps, RingAttackers};
use super::trace::{tr_eg, tr_mg};
#[cfg(not(feature = "texel"))]
use super::{CachedShelter, PAWN_TABLE_SIZE};
use super::{Evaluator, color_sign, relative_rank};
use crate::board::attacks::AttackTables;
use crate::board::geometry::{FILE_BBS, FORWARD_RANKS, SQUARE_FILE, SQUARE_RANK, between};
use crate::board::{ATTACKS, Bitboard, Board, CastlingRights, Color, Piece, Square};
use crate::infra;

/// The king-danger index is clamped here before the map, so one king's
/// penalty is bounded whatever its inputs: `ks_map_mg × 625 / 100` cp mg and
/// `ks_map_eg × 100 / 100` cp eg, 637 and 18 at the fitted weights and at
/// most [`KS_MAP_MG_MAX`] mg. The magnitude contract's instrument; it changes
/// only with a new magnitude read.
pub(super) const KS_INDEX_CAP: i32 = 1600;

/// The largest map scale the evaluation applies (and the tuner's bound), in
/// hundredths of a centipawn: three times the seed. A loaded parameter
/// vector above it is clamped.
pub const KS_MAP_SCALE_MAX: i32 = 150;

/// The index must exceed this before the map applies.
const KS_MAP_THRESHOLD: i32 = 100;

/// The largest mg penalty the map can put on one king: 937 cp, the cap at
/// the largest scale. It must stay under ten pawns, and both kings' together
/// under an eighth of the band the search reads as a decided game.
const KS_MAP_MG_MAX: i32 = KS_MAP_SCALE_MAX * (KS_INDEX_CAP * KS_INDEX_CAP / 4096) / 100;
const _: () = assert!(
    KS_MAP_MG_MAX < 1000 && 2 * KS_MAP_MG_MAX * 8 < crate::score::TB_WIN_SCORE,
    "the king-danger map's bound grew; lower KS_INDEX_CAP or KS_MAP_SCALE_MAX"
);

/// One king's danger inputs, in the counts the index weighs.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) struct KingDangerInputs {
    pub(super) attackers: RingAttackers,
    /// Ring squares the enemy attacks that we defend at most once, and then
    /// only with the king or queen.
    pub(super) weak_ring: i32,
    /// Checking squares an enemy piece attacks that are not safe for it,
    /// counted for a piece type only when it has no safe check (queens
    /// excluded).
    pub(super) unsafe_checks: i32,
    /// Pieces of either colour standing alone between our king and an enemy
    /// slider.
    pub(super) blockers: i32,
    /// Safe checking squares by knight, bishop, rook and queen. A queen check
    /// is not counted where a rook could check safely or our queen defends;
    /// a bishop check is not counted where a queen could check safely.
    pub(super) safe_checks: [i32; 4],
    pub(super) ring_size: i32,
}

/// Pieces of either colour standing alone between `color`'s king and an
/// enemy slider on one of its lines. The enemy sliders on those lines are
/// lifted off the board first, so a slider standing in front of another is
/// not counted as its blocker.
fn king_blockers(board: &Board, atk: &AttackTables, color: Color) -> Bitboard {
    let them = !color;
    let king = board.king_sq(color);
    let queens = board.pieces(them, Piece::Queen);
    let diagonal = board.pieces(them, Piece::Bishop) | queens;
    let orthogonal = board.pieces(them, Piece::Rook) | queens;
    let snipers = (atk.bishop(king, Bitboard::EMPTY) & diagonal)
        | (atk.rook(king, Bitboard::EMPTY) & orthogonal);
    let occupancy = board.occupied() ^ snipers;
    let mut blockers = Bitboard::EMPTY;
    let mut remaining = snipers;
    while remaining.any() {
        let line = between(king, remaining.pop_lsb()) & occupancy;
        if line.any() && !line.more_than_one() {
            blockers |= line;
        }
    }
    blockers
}

/// The files of each king's flank, by the king's file: a–c for a king on a,
/// a–d on b or c, c–f on d or e, e–h on f or g, f–h on h.
const KING_FLANK: [Bitboard; 8] = {
    const fn files(first: u32, last: u32) -> Bitboard {
        let mut mask = 0u64;
        let mut file = first;
        while file <= last {
            mask |= 0x0101_0101_0101_0101 << file;
            file += 1;
        }
        Bitboard(mask)
    }
    [
        files(0, 2),
        files(0, 3),
        files(0, 3),
        files(2, 5),
        files(2, 5),
        files(4, 7),
        files(4, 7),
        files(5, 7),
    ]
};

/// No pawn of either colour on `color`'s king's flank.
pub(super) fn pawnless_flank(board: &Board, color: Color) -> bool {
    let pawns = board.pieces(Color::White, Piece::Pawn) | board.pieces(Color::Black, Piece::Pawn);
    (pawns & KING_FLANK[SQUARE_FILE[board.king_sq(color).index()]]).is_empty()
}

/// One king's shelter and storm score, with the files it was read from in
/// `texel` builds, which trace them.
#[derive(Clone, Copy, Debug)]
pub(super) struct Shelter {
    pub(super) mg: i32,
    pub(super) eg: i32,
    #[cfg(feature = "texel")]
    pub(super) files: [ShelterFile; 3],
}

/// One of the three files around a king: its distance from the board's
/// edge and the relative ranks of our pawn and theirs nearest our side
/// (0 when the file has none).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ShelterFile {
    pub(super) edge: u8,
    pub(super) ours: u8,
    pub(super) theirs: u8,
}

impl ShelterFile {
    /// An enemy pawn standing directly on our pawn.
    fn blocked(self) -> bool {
        self.ours > 0 && self.ours + 1 == self.theirs
    }

    fn ours_index(self) -> usize {
        usize::from(self.edge) * 7 + usize::from(self.ours)
    }

    fn theirs_index(self) -> usize {
        usize::from(self.edge) * 7 + usize::from(self.theirs)
    }
}

/// The three files around a king on `king` (its file kept within b–g), read
/// from the pawns on its rank and ahead of it; our pawns attacked by an enemy
/// pawn give no shelter.
pub(super) fn shelter_files(
    board: &Board,
    color: Color,
    king: Square,
    their_pawn_attacks: Bitboard,
) -> [ShelterFile; 3] {
    let them = !color;
    let not_behind = !FORWARD_RANKS[them as usize][SQUARE_RANK[king.index()]];
    let pawns = (board.pieces(Color::White, Piece::Pawn) | board.pieces(Color::Black, Piece::Pawn))
        & not_behind;
    let ours = pawns & board.pieces(color, Piece::Pawn) & !their_pawn_attacks;
    let theirs = pawns & board.pieces(them, Piece::Pawn);
    // The pawn nearest our own side: the lowest for White, the highest for Black.
    let nearest = |on_file: Bitboard| {
        if on_file.is_empty() {
            0
        } else if color == Color::White {
            relative_rank(color, on_file.lsb())
        } else {
            relative_rank(color, on_file.msb())
        }
    };
    let centre = SQUARE_FILE[king.index()].clamp(1, 6);
    [centre - 1, centre, centre + 1].map(|file| ShelterFile {
        edge: infra::to_u8(file.min(7 - file)),
        ours: nearest(ours & FILE_BBS[file]),
        theirs: nearest(theirs & FILE_BBS[file]),
    })
}

impl Evaluator {
    /// The danger inputs of `color`'s king from the filled attack maps.
    pub(super) fn king_danger_inputs(
        board: &Board,
        atk: &AttackTables,
        color: Color,
        maps: &KsMaps,
    ) -> KingDangerInputs {
        let us = color as usize;
        let them = (!color) as usize;
        let king = board.king_sq(color);
        let by_us = maps.attacked_by_us;
        let by_them = maps.attacked_by_them;
        let weak = maps.attacked[them]
            & !maps.attacked2[us]
            & (!maps.attacked[us] | by_us[Piece::King as usize] | by_us[Piece::Queen as usize]);
        let safe = !maps.their_occ & (!maps.attacked[us] | (weak & maps.attacked2[them]));

        // Checking lines see through our own queen, which can move off them.
        let occupancy = maps.occupied ^ board.pieces(color, Piece::Queen);
        let rook_lines = atk.rook(king, occupancy);
        let bishop_lines = atk.bishop(king, occupancy);
        let mut unsafe_checks = Bitboard::EMPTY;

        let rook_reach = rook_lines & by_them[Piece::Rook as usize];
        let rook_checks = rook_reach & safe;
        if rook_checks.is_empty() {
            unsafe_checks |= rook_reach;
        }
        let queen_checks = (rook_lines | bishop_lines)
            & by_them[Piece::Queen as usize]
            & safe
            & !(by_us[Piece::Queen as usize] | rook_checks);
        let bishop_reach = bishop_lines & by_them[Piece::Bishop as usize];
        let bishop_checks = bishop_reach & safe & !queen_checks;
        if bishop_checks.is_empty() {
            unsafe_checks |= bishop_reach;
        }
        let knight_reach = atk.knight(king) & by_them[Piece::Knight as usize];
        let knight_checks = knight_reach & safe;
        if knight_checks.is_empty() {
            unsafe_checks |= knight_reach;
        }

        KingDangerInputs {
            attackers: maps.ring_attackers,
            weak_ring: infra::to_i32((maps.king_ring & weak).count()),
            unsafe_checks: infra::to_i32(unsafe_checks.count()),
            blockers: infra::to_i32(king_blockers(board, atk, color).count()),
            safe_checks: [
                infra::to_i32(knight_checks.count()),
                infra::to_i32(bishop_checks.count()),
                infra::to_i32(rook_checks.count()),
                infra::to_i32(queen_checks.count()),
            ],
            ring_size: infra::to_i32(maps.king_ring.count()),
        }
    }

    /// The king-danger index: the ring attackers' count times their weight,
    /// the other inputs at their weights, the enemy's mobility lead, less the
    /// reductions for a missing enemy queen, a knight beside our king and our
    /// shelter's mg score.
    pub(super) fn king_danger_index(
        &self,
        inputs: &KingDangerInputs,
        no_enemy_queen: bool,
        knight_defends: bool,
        mobility_lead_mg: i32,
        shelter_mg: i32,
    ) -> i32 {
        let p = &self.params;
        let attackers = &inputs.attackers;
        let count = attackers.pawn_attacks + attackers.pieces.iter().sum::<i32>();
        let weight: i32 = attackers
            .pieces
            .iter()
            .zip(p.kd_attacker_weight.iter())
            .map(|(n, w)| n * w)
            .sum();
        let mut index = count * weight
            + p.kd_weak_ring[0] * inputs.weak_ring
            + p.kd_unsafe_check[0] * inputs.unsafe_checks
            + p.kd_blockers[0] * inputs.blockers
            + p.kd_king_attacks[0] * attackers.king_attacks
            // Both truncate toward zero: under one index unit each from an
            // exact sum.
            + p.kd_mobility[0] * mobility_lead_mg / 100
            - p.kd_shelter[0] * shelter_mg / 100
            + p.kd_constant[0];
        for (piece, &checks) in inputs.safe_checks.iter().enumerate() {
            if checks > 0 {
                index += p.kd_safe_check[2 * piece + usize::from(checks > 1)];
            }
        }
        if no_enemy_queen {
            index -= p.kd_no_queen[0];
        }
        if knight_defends {
            index -= p.kd_knight_defender[0];
        }
        index
    }

    /// The (mg, eg) penalty for an index already clamped to the cap.
    pub(super) fn king_danger_penalty(&self, index: i32) -> (i32, i32) {
        if index <= KS_MAP_THRESHOLD {
            return (0, 0);
        }
        let scale_mg = self.params.ks_map_mg[0].clamp(0, KS_MAP_SCALE_MAX);
        let scale_eg = self.params.ks_map_eg[0].clamp(0, KS_MAP_SCALE_MAX);
        (
            scale_mg * (index * index / 4096) / 100,
            scale_eg * (index / 16) / 100,
        )
    }

    #[expect(clippy::too_many_arguments)]
    pub(super) fn eval_king_safety(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        mg: &mut i32,
        eg: &mut i32,
        maps: &KsMaps,
        mobility_mg: &[i32; 2],
        shelter: &Shelter,
    ) {
        let atk = &ATTACKS;
        let them = !color;

        *mg += sign * shelter.mg;
        *eg += sign * shelter.eg;
        #[cfg(feature = "texel")]
        self.trace_shelter(&shelter.files, sign);

        if pawnless_flank(board, color) {
            *mg -= sign * self.params.pawnless_flank_mg[0];
            *eg -= sign * self.params.pawnless_flank_eg[0];
            tr_mg!(self, pawnless_flank_mg, 0, -sign);
            tr_eg!(self, pawnless_flank_eg, 0, -sign);
        }

        let inputs = Self::king_danger_inputs(board, atk, color, maps);
        let no_enemy_queen = board.pieces(them, Piece::Queen).is_empty();
        let knight_defends = (maps.attacked_by_us[Piece::Knight as usize]
            & maps.attacked_by_us[Piece::King as usize])
            .any();
        let mobility_lead = mobility_mg[them as usize] - mobility_mg[color as usize];
        let index = self
            .king_danger_index(
                &inputs,
                no_enemy_queen,
                knight_defends,
                mobility_lead,
                shelter.mg,
            )
            .min(KS_INDEX_CAP);
        let (penalty_mg, penalty_eg) = self.king_danger_penalty(index);
        *mg -= sign * penalty_mg;
        *eg -= sign * penalty_eg;
        // The map is not linear in any weight, so its output is untraced.
        #[cfg(feature = "texel")]
        {
            let mut trace = self.trace.borrow_mut();
            trace.frozen_mg -= sign * penalty_mg;
            trace.frozen_eg -= sign * penalty_eg;
            trace.king_danger |= index > KS_MAP_THRESHOLD;
        }
    }

    /// One king square's shelter and storm score from its three files.
    fn shelter_score(&self, files: &[ShelterFile; 3]) -> (i32, i32) {
        let p = &self.params;
        let mut mg = p.shelter_constant_mg[0];
        let mut eg = p.shelter_constant_eg[0];
        for file in files {
            mg += p.shelter_strength[file.ours_index()];
            if file.blocked() {
                mg -= p.blocked_storm_mg[usize::from(file.theirs)];
                eg -= p.blocked_storm_eg[usize::from(file.theirs)];
            } else {
                mg -= p.unblocked_storm[file.theirs_index()];
            }
        }
        (mg, eg)
    }

    #[cfg(feature = "texel")]
    fn trace_shelter(&self, files: &[ShelterFile; 3], sign: i32) {
        tr_mg!(self, shelter_constant_mg, 0, sign);
        tr_eg!(self, shelter_constant_eg, 0, sign);
        for file in files {
            tr_mg!(self, shelter_strength, file.ours_index(), sign);
            if file.blocked() {
                tr_mg!(self, blocked_storm_mg, usize::from(file.theirs), -sign);
                tr_eg!(self, blocked_storm_eg, usize::from(file.theirs), -sign);
            } else {
                tr_mg!(self, unblocked_storm, file.theirs_index(), -sign);
            }
        }
    }

    /// `color`'s shelter: the best by mg over the king's square and the
    /// castling squares it still has the rights to, so a king that can still
    /// castle is credited with the better side's cover.
    pub(super) fn best_shelter(
        &self,
        board: &Board,
        color: Color,
        their_pawn_attacks: Bitboard,
    ) -> Shelter {
        let rights = board.castling();
        let (kingside, queenside, home_rank) = match color {
            Color::White => (
                CastlingRights::WHITE_KINGSIDE,
                CastlingRights::WHITE_QUEENSIDE,
                0,
            ),
            Color::Black => (
                CastlingRights::BLACK_KINGSIDE,
                CastlingRights::BLACK_QUEENSIDE,
                56,
            ),
        };
        let at = |king: Square| {
            let files = shelter_files(board, color, king, their_pawn_attacks);
            let (mg, eg) = self.shelter_score(&files);
            Shelter {
                mg,
                eg,
                #[cfg(feature = "texel")]
                files,
            }
        };
        let mut best = at(board.king_sq(color));
        if rights.has(kingside) {
            let castled = at(Square(home_rank + 6));
            if best.mg < castled.mg {
                best = castled;
            }
        }
        if rights.has(queenside) {
            let castled = at(Square(home_rank + 2));
            if best.mg < castled.mg {
                best = castled;
            }
        }
        best
    }

    /// Both kings' shelters, from the pawn entry this evaluation's
    /// `eval_pawns` filled when the king square and castling rights match,
    /// computed and stored there when not. `texel` builds compute every time,
    /// so the trace sees the files.
    pub(super) fn king_shelters(
        &mut self,
        board: &Board,
        pawn_attacks: &[Bitboard; 2],
    ) -> [Shelter; 2] {
        [Color::White, Color::Black].map(|color| {
            let their_pawn_attacks = pawn_attacks[(!color) as usize];
            #[cfg(feature = "texel")]
            {
                self.best_shelter(board, color, their_pawn_attacks)
            }
            #[cfg(not(feature = "texel"))]
            {
                let key = board.pawn_key();
                let slot = infra::index(key) & (PAWN_TABLE_SIZE - 1);
                debug_assert_eq!(
                    self.pawn_table[slot].key, key,
                    "eval_pawns fills this entry"
                );
                let king = board.king_sq(color);
                let side_rights = match color {
                    Color::White => CastlingRights::WHITE_ALL,
                    Color::Black => CastlingRights::BLACK_ALL,
                };
                let castling = board.castling().0 & side_rights.0;
                if let Some(cached) = self.pawn_table[slot].shelter[color as usize]
                    && cached.king == king
                    && cached.castling == castling
                {
                    return Shelter {
                        mg: cached.mg,
                        eg: cached.eg,
                    };
                }
                let shelter = self.best_shelter(board, color, their_pawn_attacks);
                self.pawn_table[slot].shelter[color as usize] = Some(CachedShelter {
                    king,
                    castling,
                    mg: shelter.mg,
                    eg: shelter.eg,
                });
                shelter
            }
        })
    }

    /// Central-king / lost-castling danger: a king still on its
    /// home square, on a central file, with all castling rights for that
    /// side gone — separate from the king-ring attack model, which
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

#[cfg(test)]
mod tests;
