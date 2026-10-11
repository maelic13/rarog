//! Piece activity: the per-side piece terms (bishops, rooks, knights,
//! mobility, minor and queen placement), x-rays, trapped bishops and
//! closedness, and the order in which piece activity calls the attack-map
//! producer and its other consumers.

use super::TOTAL_PHASE;
use super::attacks::KsMaps;
use super::pawns::{FILE_BBS, KING_DISTANCE, SQUARE_FILE, SQUARE_RANK};
use super::trace::{tr_eg, tr_mg};
use super::{Evaluator, color_sign, relative_rank};
use crate::board::attacks::AttackTables;
use crate::board::movegen;
use crate::board::{ATTACKS, Bitboard, Board, CastlingRights, Color, Piece, Square};
use crate::infra;

// The two main diagonals (a1-h8, a8-h1) minus their corner squares (Phase
// 3.10 bishop-on-long-diagonal term). Square index = rank*8 + file.
const LONG_DIAGONALS: Bitboard = Bitboard(
    (1u64 << 9)
        | (1u64 << 18)
        | (1u64 << 27)
        | (1u64 << 36)
        | (1u64 << 45)
        | (1u64 << 54)
        | (1u64 << 14)
        | (1u64 << 21)
        | (1u64 << 28)
        | (1u64 << 35)
        | (1u64 << 42)
        | (1u64 << 49),
);

impl Evaluator {
    /// The terms the lazy gate skips, apart from imbalance: the attack maps are
    /// filled first, then each side's terms, then the whole-board terms, with
    /// initiative last because it reads the running `eg`.
    pub(super) fn eval_piece_activity(
        &mut self,
        board: &Board,
        atk: &AttackTables,
        mg: &mut i32,
        eg: &mut i32,
        passed: &[Bitboard; 2],
        pawn_attacks: &[Bitboard; 2],
        phase: i32,
    ) {
        let occupied = board.occupied();
        let color_occ = [board.color_occ(Color::White), board.color_occ(Color::Black)];
        let pawns = [
            board.pieces(Color::White, Piece::Pawn),
            board.pieces(Color::Black, Piece::Pawn),
        ];

        let shelters = self.king_shelters(board, pawn_attacks);
        self.attacks.fill(board, atk, pawn_attacks);
        let maps = &self.attacks;
        // Each side's mobility (mg), which king danger reads for both sides.
        let mut mobility_mg = [0i32; 2];

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let them = !color;
            let own_pawns = pawns[color as usize];
            let their_pawns = pawns[them as usize];
            let own_occ = color_occ[color as usize];

            if board.pieces(color, Piece::Bishop).more_than_one() {
                *mg += sign * self.params.bishop_pair_mg[0];
                *eg += sign * self.params.bishop_pair_eg[0];
                tr_mg!(self, bishop_pair_mg, 0, sign);
                tr_eg!(self, bishop_pair_eg, 0, sign);

                // Bishop-pair value scales with fewer pawns on the board
                // (Phase 3.10): seeded 0, additive on top of the flat bonus
                // above so the flat term can be retired once this is tuned.
                let pawn_term = 8 - infra::to_i32((own_pawns | their_pawns).count());
                *mg += sign * pawn_term * self.params.bishop_pair_pawn_mg[0];
                *eg += sign * pawn_term * self.params.bishop_pair_pawn_eg[0];
                tr_mg!(self, bishop_pair_pawn_mg, 0, sign * pawn_term);
                tr_eg!(self, bishop_pair_pawn_eg, 0, sign * pawn_term);
            }

            // Per-bishop terms (Phase 3.10): outpost (mirrors the knight
            // outpost logic below), long diagonal bearing on the enemy king,
            // and bad bishop (own pawns on the bishop's own square colour).
            let enemy_king_sq = board.king_sq(them);
            let king_zone = atk.king(enemy_king_sq) | Bitboard::from(enemy_king_sq);
            let mut bishops_iter = board.pieces(color, Piece::Bishop);
            while bishops_iter.any() {
                let sq = bishops_iter.pop_lsb();
                if relative_rank(color, sq) >= 4
                    && (atk.pawn(them, sq) & own_pawns).any()
                    && (atk.pawn(color, sq) & their_pawns).is_empty()
                {
                    *mg += sign * self.params.bishop_outpost_mg[0];
                    *eg += sign * self.params.bishop_outpost_eg[0];
                    tr_mg!(self, bishop_outpost_mg, 0, sign);
                    tr_eg!(self, bishop_outpost_eg, 0, sign);
                }

                if LONG_DIAGONALS.0 & (1u64 << sq.index()) != 0
                    && (maps.attacks_from(color, sq) & king_zone).any()
                {
                    *mg += sign * self.params.bishop_long_diagonal_mg[0];
                    *eg += sign * self.params.bishop_long_diagonal_eg[0];
                    tr_mg!(self, bishop_long_diagonal_mg, 0, sign);
                    tr_eg!(self, bishop_long_diagonal_eg, 0, sign);
                }

                let bishop_squares = if (Bitboard::from(sq) & Bitboard::LIGHT_SQUARES).any() {
                    Bitboard::LIGHT_SQUARES
                } else {
                    Bitboard::DARK_SQUARES
                };
                let bad_count = infra::to_i32((own_pawns & bishop_squares).count());
                if bad_count != 0 {
                    *mg -= sign * bad_count * self.params.bad_bishop_mg[0];
                    *eg -= sign * bad_count * self.params.bad_bishop_eg[0];
                    tr_mg!(self, bad_bishop_mg, 0, -sign * bad_count);
                    tr_eg!(self, bad_bishop_eg, 0, -sign * bad_count);
                }
            }

            // Connected rooks (Phase 3.10): both own rooks on the same rank
            // or file with nothing between them.
            let color_rooks = board.pieces(color, Piece::Rook);
            if color_rooks.more_than_one() {
                let r1 = color_rooks.lsb();
                let r2 = color_rooks.msb();
                let aligned = SQUARE_FILE[r1.index()] == SQUARE_FILE[r2.index()]
                    || SQUARE_RANK[r1.index()] == SQUARE_RANK[r2.index()];
                if aligned && (movegen::between(r1, r2) & occupied).is_empty() {
                    *mg += sign * self.params.rook_connected_mg[0];
                    *eg += sign * self.params.rook_connected_eg[0];
                    tr_mg!(self, rook_connected_mg, 0, sign);
                    tr_eg!(self, rook_connected_eg, 0, sign);
                }
            }

            let own_king_sq = board.king_sq(color);
            let own_castling_all = match color {
                Color::White => CastlingRights::WHITE_ALL,
                Color::Black => CastlingRights::BLACK_ALL,
            };
            let own_lost_castling = !board.castling().has(own_castling_all);
            let home_rank_corner = match color {
                Color::White => [Square(0), Square(7)],
                Color::Black => [Square(56), Square(63)],
            };

            let mut rooks = board.pieces(color, Piece::Rook);
            while rooks.any() {
                let sq = rooks.pop_lsb();
                let file = SQUARE_FILE[sq.index()];
                let own_file_empty = (own_pawns & FILE_BBS[file]).is_empty();
                let their_file_empty = (their_pawns & FILE_BBS[file]).is_empty();
                if own_file_empty && their_file_empty {
                    *mg += sign * self.params.rook_open_mg[0];
                    *eg += sign * self.params.rook_open_eg[0];
                    tr_mg!(self, rook_open_mg, 0, sign);
                    tr_eg!(self, rook_open_eg, 0, sign);
                } else if own_file_empty {
                    *mg += sign * self.params.rook_semiopen_mg[0];
                    *eg += sign * self.params.rook_semiopen_eg[0];
                    tr_mg!(self, rook_semiopen_mg, 0, sign);
                    tr_eg!(self, rook_semiopen_eg, 0, sign);
                }
                if relative_rank(color, sq) == 6 {
                    *mg += sign * self.params.rook_7th_mg[0];
                    *eg += sign * self.params.rook_7th_eg[0];
                    tr_mg!(self, rook_7th_mg, 0, sign);
                    tr_eg!(self, rook_7th_eg, 0, sign);
                }

                // Trapped rook (Phase 3.10): own rook stuck in its starting
                // corner behind an uncastled king with very low mobility.
                if own_lost_castling
                    && own_king_sq == Square([4u8, 60u8][color as usize])
                    && home_rank_corner.contains(&sq)
                {
                    let mobility = infra::to_i32((atk.rook(sq, occupied) & !own_occ).count());
                    if mobility <= 3 {
                        *mg -= sign * self.params.rook_trapped_mg[0];
                        *eg -= sign * self.params.rook_trapped_eg[0];
                        tr_mg!(self, rook_trapped_mg, 0, -sign);
                        tr_eg!(self, rook_trapped_eg, 0, -sign);
                    }
                }
            }

            let mut knights = board.pieces(color, Piece::Knight);
            while knights.any() {
                let sq = knights.pop_lsb();
                if relative_rank(color, sq) >= 4
                    && (atk.pawn(them, sq) & own_pawns).any()
                    && (atk.pawn(color, sq) & their_pawns).is_empty()
                {
                    *mg += sign * self.params.knight_outpost_mg[0];
                    *eg += sign * self.params.knight_outpost_eg[0];
                    tr_mg!(self, knight_outpost_mg, 0, sign);
                    tr_eg!(self, knight_outpost_eg, 0, sign);
                }
            }

            let mobility_area = maps.mobility_area[color as usize];
            // One-hot per-count tables (Phase 3.7). Index clamped to the table
            // length for safety; the count never exceeds it in practice
            // (N≤8, B≤13, R≤14, Q≤27).
            //
            // 9.7.5(d): each piece type gets its OWN loop, so the table pair is
            // a compile-time constant rather than a `match piece` re-evaluated
            // for every piece on the board. The match was loop-invariant across
            // the whole `while pieces.any()` — the same duplicated-dispatch
            // shape 8.12(g2) found in move scoring, and the Rust equivalent of
            // Basilisk's `if constexpr` specialisation (+0.89% there).
            macro_rules! mob_loop {
                ($piece:expr, $mgf:ident, $egf:ident) => {{
                    let mut pieces = board.pieces(color, $piece);
                    while pieces.any() {
                        let sq = pieces.pop_lsb();
                        let mobility =
                            (maps.attacks_from(color, sq) & mobility_area).count() as usize;
                        let i = mobility.min(self.params.$mgf.len() - 1);
                        mobility_mg[color as usize] += self.params.$mgf[i];
                        *mg += sign * self.params.$mgf[i];
                        *eg += sign * self.params.$egf[i];
                        tr_mg!(self, $mgf, i, sign);
                        tr_eg!(self, $egf, i, sign);
                    }
                }};
            }
            mob_loop!(Piece::Knight, mob_n_mg, mob_n_eg);
            mob_loop!(Piece::Bishop, mob_b_mg, mob_b_eg);
            mob_loop!(Piece::Rook, mob_r_mg, mob_r_eg);
            mob_loop!(Piece::Queen, mob_q_mg, mob_q_eg);

            self.eval_threats(
                board,
                atk,
                color,
                sign,
                maps,
                pawn_attacks,
                &pawns,
                occupied,
                mg,
                eg,
            );

            // ---- Gauntlet-driven additions (Phase 3.12), all seeded 0 (bench
            // unchanged), tuned in Phase 4. ----
            let own_king = board.king_sq(color);
            let minors = board.pieces(color, Piece::Knight) | board.pieces(color, Piece::Bishop);

            // Minor behind pawn: a knight/bishop with a friendly pawn directly
            // in front of it (toward the enemy).
            let shield = if color == Color::White {
                own_pawns.south()
            } else {
                own_pawns.north()
            };
            let behind = infra::to_i32((minors & shield).count());
            if behind != 0 {
                *mg += sign * behind * self.params.minor_behind_pawn_mg[0];
                *eg += sign * behind * self.params.minor_behind_pawn_eg[0];
                tr_mg!(self, minor_behind_pawn_mg, 0, sign * behind);
                tr_eg!(self, minor_behind_pawn_eg, 0, sign * behind);
            }

            // King protector: penalty proportional to each own minor's distance
            // from our king (minors far from the king shelter it less).
            let mut protector = 0i32;
            let mut mb = minors;
            while mb.any() {
                let m = mb.pop_lsb();
                protector += KING_DISTANCE[own_king.index()][m.index()] as i32;
            }
            if protector != 0 {
                *mg -= sign * protector * self.params.king_protector_mg[0];
                *eg -= sign * protector * self.params.king_protector_eg[0];
                tr_mg!(self, king_protector_mg, 0, -sign * protector);
                tr_eg!(self, king_protector_eg, 0, -sign * protector);
            }

            // Queen infiltration: our queen safely deep in the enemy half
            // (relative rank >= 4) on a square no enemy pawn attacks.
            let mut queens = board.pieces(color, Piece::Queen);
            let mut infiltration = 0i32;
            while queens.any() {
                let qs = queens.pop_lsb();
                if relative_rank(color, qs) >= 4
                    && (pawn_attacks[them as usize] & Bitboard::from(qs)).is_empty()
                {
                    infiltration += 1;
                }
            }
            if infiltration != 0 {
                *mg += sign * infiltration * self.params.queen_infiltration_mg[0];
                *eg += sign * infiltration * self.params.queen_infiltration_eg[0];
                tr_mg!(self, queen_infiltration_mg, 0, sign * infiltration);
                tr_eg!(self, queen_infiltration_eg, 0, sign * infiltration);
            }

            self.eval_unstoppable_passers(board, color, sign, passed, occupied, eg);

            self.eval_rooks_behind_passers(board, color, sign, passed, mg, eg);
            self.eval_passer_blockade(board, color, sign, passed, mg, eg);
            self.eval_hanging_pieces(board, color, sign, mg, eg, &maps.attacked);
            self.eval_xray_and_battery(board, color, sign, occupied, &pawns, mg, eg);
        }

        for color in [Color::White, Color::Black] {
            let ks_maps = KsMaps::new(maps, color, occupied, &color_occ);
            self.eval_king_safety(
                board,
                color,
                color_sign(color),
                mg,
                eg,
                &ks_maps,
                &mobility_mg,
                &shelters[color as usize],
            );
        }

        self.eval_passed_pawn_king_proximity(board, passed, eg);
        self.eval_space(board, pawn_attacks, mg);
        if phase < TOTAL_PHASE / 2 {
            self.eval_trapped_bishops(board, atk, mg, eg);
        }
        self.eval_closedness(board, mg);
        self.eval_king_centrality_danger(board, mg);
        self.eval_initiative(board, eg);
    }

    /// Deferred §3.12 trio (Phase 6.2.1, all seeded 0): bishop x-ray on enemy
    /// pawns, queen batteries, and sliders x-raying the enemy queen. X-rays are
    /// slider attacks computed with pawns-only occupancy (seeing through
    /// pieces), the cheap standard formulation.
    fn eval_xray_and_battery(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        occupied: Bitboard,
        pawns: &[Bitboard; 2],
        mg: &mut i32,
        eg: &mut i32,
    ) {
        // Resolve the LazyLock once for the whole feature group. Each slider
        // accessor is hot and inlined; spelling `ATTACKS.*` inside the loops
        // otherwise asks the compiler to rediscover that initialization state
        // repeatedly.
        let atk = &*ATTACKS;
        let them = !color;
        let pawns_only = pawns[0] | pawns[1];
        let enemy_pawns = pawns[them as usize];
        let own_rooks = board.pieces(color, Piece::Rook);
        let own_bishops = board.pieces(color, Piece::Bishop);

        // Bishop x-ray on enemy pawns: pawns on the bishop's diagonals seen
        // through any pieces (long-term pressure the plain attack map misses).
        let mut bishops = own_bishops;
        let mut xray = 0i32;
        while bishops.any() {
            let sq = bishops.pop_lsb();
            xray += infra::to_i32((atk.bishop(sq, pawns_only) & enemy_pawns).count());
        }
        if xray != 0 {
            *mg += sign * xray * self.params.bishop_xray_pawns_mg[0];
            *eg += sign * xray * self.params.bishop_xray_pawns_eg[0];
            tr_mg!(self, bishop_xray_pawns_mg, 0, sign * xray);
            tr_eg!(self, bishop_xray_pawns_eg, 0, sign * xray);
        }

        // Queen battery: own rook (file/rank) or bishop (diagonal) directly
        // aligned with the queen — doubled-force lines.
        let mut queens = board.pieces(color, Piece::Queen);
        let mut battery = 0i32;
        while queens.any() {
            let q = queens.pop_lsb();
            battery += infra::to_i32((atk.rook(q, occupied) & own_rooks).count());
            battery += infra::to_i32((atk.bishop(q, occupied) & own_bishops).count());
        }
        if battery != 0 {
            *mg += sign * battery * self.params.queen_battery_mg[0];
            *eg += sign * battery * self.params.queen_battery_eg[0];
            tr_mg!(self, queen_battery_mg, 0, sign * battery);
            tr_eg!(self, queen_battery_eg, 0, sign * battery);
        }
    }

    fn eval_trapped_bishops(&self, board: &Board, atk: &AttackTables, mg: &mut i32, eg: &mut i32) {
        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let mut bishops = board.pieces(color, Piece::Bishop);
            while bishops.any() {
                let sq = bishops.pop_lsb();
                if (atk.bishop(sq, board.occupied()) & !board.color_occ(color)).is_empty() {
                    *mg -= sign * self.params.trapped_bishop_mg[0];
                    *eg -= sign * self.params.trapped_bishop_eg[0];
                    tr_mg!(self, trapped_bishop_mg, 0, -sign);
                    tr_eg!(self, trapped_bishop_eg, 0, -sign);
                }
            }
        }
    }

    /// Closedness (Phase 3.10): value swing for knights/rooks as the centre
    /// locks (rammed pawn count). Per-count-mobility (3.7) already penalises
    /// a knight's reduced mobility in closed positions, so the only marginal
    /// lever here is the material-value swing itself — kept as one small
    /// weight per piece type rather than a full table.
    fn eval_closedness(&self, board: &Board, mg: &mut i32) {
        let wp = board.pieces(Color::White, Piece::Pawn);
        let bp = board.pieces(Color::Black, Piece::Pawn);
        let rammed = infra::to_i32((wp.north() & bp).count());
        if rammed == 0 {
            return;
        }
        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let knights = infra::to_i32(board.pieces(color, Piece::Knight).count());
            let rooks = infra::to_i32(board.pieces(color, Piece::Rook).count());
            if knights != 0 {
                *mg += sign * rammed * knights * self.params.closedness_knight_mg[0];
                tr_mg!(self, closedness_knight_mg, 0, sign * rammed * knights);
            }
            if rooks != 0 {
                *mg += sign * rammed * rooks * self.params.closedness_rook_mg[0];
                tr_mg!(self, closedness_rook_mg, 0, sign * rammed * rooks);
            }
        }
    }
}
