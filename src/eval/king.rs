//! King safety: the king-danger index through its capped quadratic map,
//! shelter and storm, and the central-king danger.

use super::attacks::{KsMaps, RingAttackers};
use super::pawns::{FILE_BBS, FORWARD_RANKS, SQUARE_FILE, SQUARE_RANK};
use super::trace::tr_mg;
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
    /// reductions for a missing enemy queen and a knight beside our king.
    pub(super) fn king_danger_index(
        &self,
        inputs: &KingDangerInputs,
        no_enemy_queen: bool,
        knight_defends: bool,
        mobility_lead_mg: i32,
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
            // Truncates toward zero: under one index unit from an exact sum.
            + p.kd_mobility[0] * mobility_lead_mg / 100
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
        pawns: &[Bitboard; 2],
        maps: &KsMaps,
        mobility_mg: &[i32; 2],
    ) {
        // One LazyLock resolution covers every king-zone attack lookup below.
        let atk = &*ATTACKS;
        let them = !color;
        let king = board.king_sq(color);

        let inputs = Self::king_danger_inputs(board, atk, color, maps);
        let no_enemy_queen = board.pieces(them, Piece::Queen).is_empty();
        let knight_defends = (maps.attacked_by_us[Piece::Knight as usize]
            & maps.attacked_by_us[Piece::King as usize])
            .any();
        let mobility_lead = mobility_mg[them as usize] - mobility_mg[color as usize];
        let index = self
            .king_danger_index(&inputs, no_enemy_queen, knight_defends, mobility_lead)
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
        let index = ev.king_danger_index(&white, false, false, 0);
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
    /// weight from two safe checks of a type, and applies both reductions;
    /// the fixture compares counts only, so this pins the arithmetic.
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
            + p.kd_constant[0]
            + p.kd_safe_check[2]
            + p.kd_safe_check[5]
            - p.kd_no_queen[0]
            - p.kd_knight_defender[0];
        assert_eq!(ev.king_danger_index(&inputs, true, true, 150), expected);
        let without_reductions = expected + p.kd_no_queen[0] + p.kd_knight_defender[0];
        assert_eq!(
            ev.king_danger_index(&inputs, false, false, 150),
            without_reductions
        );
    }
}
