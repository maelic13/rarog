//! King safety: the king-danger index through its capped quadratic map,
//! shelter and storm, and the central-king danger.

use super::attacks::{KsMaps, RingAttackers};
use super::pawns::{FILE_BBS, FORWARD_RANKS, SQUARE_FILE, SQUARE_RANK};
use super::trace::{tr_eg, tr_mg};
#[cfg(not(feature = "texel"))]
use super::{CachedShelter, PAWN_TABLE_SIZE};
use super::{Evaluator, color_sign, relative_rank};
use crate::board::attacks::AttackTables;
use crate::board::movegen::between;
use crate::board::{ATTACKS, Bitboard, Board, CastlingRights, Color, Piece, Square};
use crate::infra;

/// The king-danger index is clamped here before the map, so one king's
/// penalty is bounded whatever its inputs: 306 cp mg and 49 cp eg at the
/// seed scales. The magnitude contract's instrument; it changes only with a
/// new magnitude read.
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
    KS_MAP_MG_MAX < 1000 && 2 * KS_MAP_MG_MAX * 8 < crate::tt::TB_WIN_SCORE,
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
        // One LazyLock resolution covers every king-zone attack lookup below.
        let atk = &*ATTACKS;
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
    pub(super) fn shelter_score(&self, files: &[ShelterFile; 3]) -> (i32, i32) {
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

#[cfg(test)]
mod tests {
    use super::super::attacks::AttackMaps;
    use super::*;

    /// The donor's ring-attacker weights for knight, bishop, rook and queen,
    /// which the fixture's weight column was printed with.
    const DONOR_WEIGHTS: [i32; 4] = [81, 52, 44, 10];

    fn pawn_attacks(board: &Board) -> [Bitboard; 2] {
        let white = board.pieces(Color::White, Piece::Pawn);
        let black = board.pieces(Color::Black, Piece::Pawn);
        [
            white.north_east() | white.north_west(),
            black.south_east() | black.south_west(),
        ]
    }

    /// Both kings' danger inputs from attack maps filled for `board`.
    fn inputs(board: &Board) -> [KingDangerInputs; 2] {
        let atk = &*ATTACKS;
        let mut maps = AttackMaps::new();
        maps.fill(board, atk, &pawn_attacks(board));
        let color_occ = [board.color_occ(Color::White), board.color_occ(Color::Black)];
        [Color::White, Color::Black].map(|color| {
            let ks = KsMaps::new(&maps, color, board.occupied(), &color_occ);
            Evaluator::king_danger_inputs(board, atk, color, &ks)
        })
    }

    /// The fixture's eleven columns for one king, in its order.
    fn columns(king: &KingDangerInputs) -> [i32; 11] {
        let a = &king.attackers;
        let weight = a.pieces.iter().zip(DONOR_WEIGHTS).map(|(n, w)| n * w).sum();
        let [knight, bishop, rook, queen] = king.safe_checks;
        [
            a.pawn_attacks + a.pieces.iter().sum::<i32>(),
            weight,
            king.weak_ring,
            king.unsafe_checks,
            king.blockers,
            a.king_attacks,
            rook,
            queen,
            bishop,
            knight,
            king.ring_size,
        ]
    }

    /// Every king-danger component the donor printed for 500 positions, its
    /// piece attacks made plain as Rarog's are
    /// (`tools/diag/king_danger_fixture.py`), is reproduced exactly.
    #[test]
    fn king_danger_inputs_reproduce_the_donor_fixture() {
        let fixture = include_str!("../../tests/data/king-danger-9587eeeb-plain.tsv");
        let mut lines = fixture.lines();
        let header = lines.next().expect("header");
        assert_eq!(header.split('\t').count(), 23, "fixture columns: {header}");
        let mut rows = 0;
        for line in lines {
            let cells: Vec<&str> = line.split('\t').collect();
            let board = Board::from_fen(cells[0]).expect("fixture FEN");
            let expected: Vec<i32> = cells[1..]
                .iter()
                .map(|c| c.parse().expect("integer cell"))
                .collect();
            let [white, black] = inputs(&board);
            let actual: Vec<i32> = columns(&white).into_iter().chain(columns(&black)).collect();
            assert_eq!(actual, expected, "{}", cells[0]);
            rows += 1;
        }
        assert_eq!(rows, 500);
    }

    /// An enemy slider in front of another on the king's line is not the
    /// rear one's blocker; a piece in front of both is, once. The fixture
    /// holds no such line.
    #[test]
    fn king_blockers_lift_the_other_sliders_on_the_line() {
        let atk = &*ATTACKS;
        let rook_before_queen = Board::from_fen("4q2k/8/8/8/4r3/8/8/4K3 w - - 0 1").unwrap();
        assert_eq!(
            king_blockers(&rook_before_queen, atk, Color::White).count(),
            0
        );
        let knight_before_both = Board::from_fen("4q2k/8/8/4r3/8/8/4N3/4K3 w - - 0 1").unwrap();
        assert_eq!(
            king_blockers(&knight_before_both, atk, Color::White),
            knight_before_both.pieces(Color::White, Piece::Knight)
        );
    }

    /// A king with every input saturated reads an index far beyond the cap,
    /// and its penalty is the cap's, inside the bound the const assertion
    /// ties to the decisive band, at any map scale the evaluation accepts.
    #[test]
    fn a_saturated_king_is_held_at_the_cap() {
        let board = Board::from_fen("6k1/8/8/8/3b4/4nq2/3r4/qr4K1 w - - 0 1").unwrap();
        let [white, _] = inputs(&board);
        let mut ev = Evaluator::default();
        let index = ev.king_danger_index(&white, false, false, 0, 0);
        assert!(
            index > 2 * KS_INDEX_CAP,
            "the fixture must saturate the index: {index}"
        );
        let capped = ev.king_danger_penalty(index.min(KS_INDEX_CAP));
        assert_eq!(capped, ev.king_danger_penalty(KS_INDEX_CAP));
        ev.params.ks_map_mg[0] = i32::MAX;
        ev.params.ks_map_eg[0] = i32::MAX;
        let (mg, eg) = ev.king_danger_penalty(KS_INDEX_CAP);
        assert_eq!(mg, KS_MAP_MG_MAX);
        assert!(eg < mg);
    }

    /// The index sums its inputs at their weights, takes the multiple-check
    /// weight from two safe checks of a type, and applies the reductions and
    /// the shelter feedback; the fixture compares counts only, so this pins
    /// the arithmetic.
    #[test]
    fn the_index_weighs_each_input() {
        let ev = Evaluator::default();
        let p = &ev.params;
        let inputs = KingDangerInputs {
            attackers: RingAttackers {
                pawn_attacks: 1,
                pieces: [1, 0, 2, 1],
                king_attacks: 3,
            },
            weak_ring: 2,
            unsafe_checks: 1,
            blockers: 1,
            safe_checks: [0, 1, 2, 0],
            ring_size: 9,
        };
        let count = 1 + 1 + 2 + 1;
        let weight =
            p.kd_attacker_weight[0] + 2 * p.kd_attacker_weight[2] + p.kd_attacker_weight[3];
        let expected = count * weight
            + 2 * p.kd_weak_ring[0]
            + p.kd_unsafe_check[0]
            + p.kd_blockers[0]
            + 3 * p.kd_king_attacks[0]
            + p.kd_mobility[0] * 150 / 100
            - p.kd_shelter[0] * 200 / 100
            + p.kd_constant[0]
            + p.kd_safe_check[2]
            + p.kd_safe_check[5]
            - p.kd_no_queen[0]
            - p.kd_knight_defender[0];
        assert_eq!(
            ev.king_danger_index(&inputs, true, true, 150, 200),
            expected
        );
        let without_reductions = expected + p.kd_no_queen[0] + p.kd_knight_defender[0];
        assert_eq!(
            ev.king_danger_index(&inputs, false, false, 150, 200),
            without_reductions
        );
    }

    /// The donor's shelter and storm tables in its own units, so the
    /// shelter fixture's scores can be compared exactly.
    fn donor_shelter_evaluator() -> Evaluator {
        let mut ev = Evaluator::default();
        let p = &mut ev.params;
        p.shelter_strength = [
            -6, 81, 93, 58, 39, 18, 25, -43, 61, 35, -49, -29, -11, -63, -10, 75, 23, -2, 32, 3,
            -45, -39, -13, -29, -52, -48, -67, -166,
        ];
        p.unblocked_storm = [
            85, -289, -166, 97, 50, 45, 50, 46, -25, 122, 45, 37, -10, 20, -6, 51, 168, 34, -2,
            -22, -14, -15, -11, 101, 4, 11, -15, -29,
        ];
        p.blocked_storm_mg = [0, 0, 76, -10, -7, -4, -1];
        p.blocked_storm_eg = [0, 0, 78, 15, 10, 6, 2];
        p.shelter_constant_mg = [5];
        p.shelter_constant_eg = [5];
        ev
    }

    fn shelter_of(ev: &Evaluator, board: &Board, color: Color) -> (i32, i32) {
        let attacks = pawn_attacks(board);
        let shelter = ev.best_shelter(board, color, attacks[(!color) as usize]);
        (shelter.mg, shelter.eg)
    }

    /// The donor's shelter and storm score (its units, the castling squares
    /// included) and its pawnless-flank flag on the 500 fixture positions,
    /// reproduced exactly from its own table values.
    #[test]
    fn king_shelter_reproduces_the_donor_fixture() {
        let ev = donor_shelter_evaluator();
        let fixture = include_str!("../../tests/data/king-shelter-9587eeeb.tsv");
        let mut lines = fixture.lines();
        assert_eq!(lines.next().expect("header").split('\t').count(), 7);
        let mut rows = 0;
        for line in lines {
            let cells: Vec<&str> = line.split('\t').collect();
            let board = Board::from_fen(cells[0]).expect("fixture FEN");
            let expected: Vec<i32> = cells[1..]
                .iter()
                .map(|c| c.parse().expect("integer cell"))
                .collect();
            let mut actual = Vec::new();
            for color in [Color::White, Color::Black] {
                let (mg, eg) = shelter_of(&ev, &board, color);
                actual.extend([mg, eg, i32::from(pawnless_flank(&board, color))]);
            }
            assert_eq!(actual, expected, "{}", cells[0]);
            rows += 1;
        }
        assert_eq!(rows, 500);
    }

    /// A king that can still castle is credited with the better side's
    /// cover; without the right, only its own square counts.
    #[test]
    fn shelter_takes_the_better_castling_square() {
        let ev = donor_shelter_evaluator();
        let can_castle = Board::from_fen("4k3/8/8/8/8/8/5PPP/4K2R w K - 0 1").unwrap();
        let cannot = Board::from_fen("4k3/8/8/8/8/8/5PPP/4K2R w - - 0 1").unwrap();
        let castled = Board::from_fen("4k3/8/8/8/8/8/5PPP/5RK1 w - - 0 1").unwrap();
        assert_eq!(
            shelter_of(&ev, &can_castle, Color::White),
            shelter_of(&ev, &castled, Color::White)
        );
        assert!(
            shelter_of(&ev, &cannot, Color::White).0 < shelter_of(&ev, &castled, Color::White).0
        );
    }

    /// The files are centred on the king's file kept within b–g, so a king
    /// on h reads f, g and h like a king on g; a pawn on our pawn is a
    /// blocked storm.
    #[test]
    fn shelter_files_clamp_and_block() {
        let board = Board::from_fen("4k3/8/8/8/8/6p1/6P1/7K w - - 0 1").unwrap();
        let attacks = pawn_attacks(&board);
        let on_h = shelter_files(&board, Color::White, Square(7), attacks[1]);
        let on_g = shelter_files(&board, Color::White, Square(6), attacks[1]);
        assert_eq!(on_h, on_g);
        assert_eq!(on_h.map(|f| f.edge), [2, 1, 0]);
        let g_file = on_h[1];
        assert_eq!((g_file.ours, g_file.theirs), (1, 2));
        assert!(g_file.blocked());
    }

    /// The pawn entry keeps each king's shelter with the square and rights
    /// it was computed for: the same pawns with the king on two squares
    /// evaluate as a fresh evaluator does, in either order. The queens give
    /// the position a middlegame phase, where the shelters differ.
    #[cfg(not(feature = "texel"))]
    #[test]
    fn the_pawn_entry_recomputes_the_shelter_for_a_moved_king() {
        let on_g1 = Board::from_fen("3q2k1/5ppp/8/8/8/8/5PPP/3Q2K1 w - - 0 1").unwrap();
        let on_e1 = Board::from_fen("3q2k1/5ppp/8/8/8/8/5PPP/3QK2R w - - 0 1").unwrap();
        let on_e1_castling = Board::from_fen("3q2k1/5ppp/8/8/8/8/5PPP/3QK2R w K - 0 1").unwrap();
        assert_eq!(on_g1.pawn_key(), on_e1.pawn_key());
        assert_eq!(on_e1.pawn_key(), on_e1_castling.pawn_key());
        let fresh = |board: &Board| {
            let mut ev = Evaluator::default();
            ev.set_lazy_margin(i32::MAX);
            ev.evaluate(board)
        };
        let mut ev = Evaluator::default();
        ev.set_lazy_margin(i32::MAX);
        for board in [
            &on_g1,
            &on_e1,
            &on_e1_castling,
            &on_e1,
            &on_g1,
            &on_e1_castling,
        ] {
            ev.eval_table.fill(super::super::EvalEntry::default());
            assert_eq!(ev.evaluate(board), fresh(board));
        }
        let shelter = |board: &Board| {
            let attacks = pawn_attacks(board);
            Evaluator::default()
                .best_shelter(board, Color::White, attacks[1])
                .mg
        };
        assert_ne!(shelter(&on_g1), shelter(&on_e1));
        assert_ne!(shelter(&on_e1), shelter(&on_e1_castling));
    }
}
