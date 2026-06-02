use crate::board::attacks::AttackTables;
use crate::board::{ATTACKS, Bitboard, Board, Color, GameResult, Piece, Square};

pub const MATE_SCORE: i32 = 32_000;
pub const INF_SCORE: i32 = 32_001;
pub const VALUE_NONE: i32 = 32_002;

const PAWN_TABLE_SIZE: usize = 16_384;
const EVAL_TABLE_SIZE: usize = 32_768;
// Conservative lazy-eval margin: if the cheap material+PST score is this many
// centipawns outside the search window, skip the expensive pawn-structure,
// mobility, and king-safety evaluation.  Must be large enough to absorb all
// evaluation terms not included in the cheap score (~200-400 cp typical range).
const LAZY_MARGIN: i32 = 600;
const TOTAL_PHASE: i32 = 24;

const MG_VAL: [i32; 6] = [82, 337, 365, 477, 1025, 0];
const EG_VAL: [i32; 6] = [94, 281, 297, 512, 936, 0];
const PHASE_W: [i32; 6] = [0, 1, 1, 2, 4, 0];
const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, MATE_SCORE];

const MG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -35, -1, -20, -23, -15, 24, 38, -22, -26, -4, -4, -10, 3, 3, 33, -12,
    -27, -2, -5, 12, 17, 6, 10, -25, -14, 13, 6, 21, 23, 12, 17, -23, -6, 7, 26, 31, 65, 56, 25,
    -20, 98, 134, 61, 95, 68, 126, 34, -11, 0, 0, 0, 0, 0, 0, 0, 0,
];
const EG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -10, -6, 10, 0, 14, 7, -5, -19, -8, -4, 7, 22, 17, 16, 3, -14, 13, 0,
    -13, 1, -1, -16, 3, -6, 32, 24, 13, 5, -2, 4, 17, 17, 56, 35, 41, 22, 26, 51, 56, 20, 134, 108,
    109, 107, 105, 104, 112, 108, 0, 0, 0, 0, 0, 0, 0, 0,
];
const MG_KNIGHT_PST: [i32; 64] = [
    -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84,
    129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10,
    19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23,
];
const EG_KNIGHT_PST: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10, 9,
    -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23, -3, -1,
    15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15, -22, -18, -50,
    -64,
];
const MG_BISHOP_PST: [i32; 64] = [
    -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50,
    37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18,
    10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
];
const EG_BISHOP_PST: [i32; 64] = [
    -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6, 0,
    4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7, -15,
    -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
];
const MG_ROOK_PST: [i32; 64] = [
    -19, -13, 1, 17, 16, 7, -37, -26, -44, -16, -20, -9, -1, 11, -6, -71, -45, -25, -16, -17, 3, 0,
    -5, -33, -36, -26, -12, -1, 9, -7, 6, -23, -24, -11, 7, 26, 24, 35, -8, -20, -5, 19, 26, 36,
    17, 45, 61, 16, 27, 32, 58, 62, 80, 67, 26, 44, 32, 42, 32, 51, 63, 9, 31, 43,
];
const EG_ROOK_PST: [i32; 64] = [
    -9, 2, 3, -1, -5, -13, 4, -20, -6, -6, 0, 2, -9, -9, -11, -3, -4, 0, -5, -1, -7, -12, -8, -16,
    3, 5, 8, 4, -5, -6, -8, -11, 4, 3, 13, 1, 2, 1, -1, 2, 7, 7, 7, 5, 4, -3, -5, -3, 11, 13, 13,
    11, -3, 3, 8, 3, 13, 10, 18, 15, 12, 12, 8, 5,
];
const MG_QUEEN_PST: [i32; 64] = [
    -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47,
    57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2,
    14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
];
const EG_QUEEN_PST: [i32; 64] = [
    -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19, 9,
    3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6, 9, 17, 10, 5,
    -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20, -41,
];
const MG_KING_PST: [i32; 64] = [
    -15, 36, 12, -54, 8, -28, 24, 14, 1, 7, -8, -64, -43, -16, 9, 8, -14, -14, -22, -46, -44, -30,
    -15, -27, -49, -1, -27, -39, -46, -44, -33, -51, -17, -20, -12, -27, -30, -25, -14, -36, -9,
    24, 2, -16, -20, 6, 22, -22, 29, -1, -20, -7, -8, -4, -38, -29, -65, 23, 16, -15, -56, -34, 2,
    13,
];
const EG_KING_PST: [i32; 64] = [
    -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20, 45,
    44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11, 21, 23,
    16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
];

const FILE_BBS: [Bitboard; 8] = [
    Bitboard::FILE_A,
    Bitboard::FILE_B,
    Bitboard(0x0404_0404_0404_0404),
    Bitboard(0x0808_0808_0808_0808),
    Bitboard(0x1010_1010_1010_1010),
    Bitboard(0x2020_2020_2020_2020),
    Bitboard::FILE_G,
    Bitboard::FILE_H,
];
const ADJACENT_FILES: [Bitboard; 8] = init_adjacent_files();
const FORWARD_RANKS: [[Bitboard; 8]; 2] = init_forward_ranks();
const PASSED_PAWN_MASKS: [[Bitboard; 64]; 2] = init_passed_pawn_masks();
const MG_TABLE: [[[i32; 64]; 6]; 2] = init_eval_table(true);
const EG_TABLE: [[[i32; 64]; 6]; 2] = init_eval_table(false);
const SQUARE_FILE: [usize; 64] = init_square_file();
const SQUARE_RANK: [usize; 64] = init_square_rank();
const RELATIVE_RANKS: [[u8; 64]; 2] = init_relative_ranks();
const KING_DISTANCE: [[u8; 64]; 64] = init_king_distance();

const fn init_square_file() -> [usize; 64] {
    let mut table = [0usize; 64];
    let mut sq = 0usize;
    while sq < 64 {
        table[sq] = sq & 7;
        sq += 1;
    }
    table
}

const fn init_square_rank() -> [usize; 64] {
    let mut table = [0usize; 64];
    let mut sq = 0usize;
    while sq < 64 {
        table[sq] = sq >> 3;
        sq += 1;
    }
    table
}

const fn init_relative_ranks() -> [[u8; 64]; 2] {
    let mut table = [[0u8; 64]; 2];
    let mut sq = 0usize;
    while sq < 64 {
        let rank = (sq >> 3) as u8;
        table[Color::White as usize][sq] = rank;
        table[Color::Black as usize][sq] = 7 - rank;
        sq += 1;
    }
    table
}

const fn init_king_distance() -> [[u8; 64]; 64] {
    let mut table = [[0u8; 64]; 64];
    let mut a = 0usize;
    while a < 64 {
        let af = (a & 7) as i32;
        let ar = (a >> 3) as i32;
        let mut b = 0usize;
        while b < 64 {
            let bf = (b & 7) as i32;
            let br = (b >> 3) as i32;
            let df = if af > bf { af - bf } else { bf - af };
            let dr = if ar > br { ar - br } else { br - ar };
            table[a][b] = if df > dr { df as u8 } else { dr as u8 };
            b += 1;
        }
        a += 1;
    }
    table
}

const fn init_adjacent_files() -> [Bitboard; 8] {
    let mut table = [Bitboard::EMPTY; 8];
    let mut file = 0usize;
    while file < 8 {
        let mut mask = 0u64;
        if file > 0 {
            mask |= FILE_BBS[file - 1].0;
        }
        if file < 7 {
            mask |= FILE_BBS[file + 1].0;
        }
        table[file] = Bitboard(mask);
        file += 1;
    }
    table
}

const fn init_forward_ranks() -> [[Bitboard; 8]; 2] {
    let mut table = [[Bitboard::EMPTY; 8]; 2];
    let mut rank = 0usize;
    while rank < 8 {
        let mut white = 0u64;
        let mut r = rank + 1;
        while r < 8 {
            white |= 0xFFu64 << (r * 8);
            r += 1;
        }
        table[Color::White as usize][rank] = Bitboard(white);

        let mut black = 0u64;
        r = 0;
        while r < rank {
            black |= 0xFFu64 << (r * 8);
            r += 1;
        }
        table[Color::Black as usize][rank] = Bitboard(black);
        rank += 1;
    }
    table
}

const fn init_passed_pawn_masks() -> [[Bitboard; 64]; 2] {
    let mut table = [[Bitboard::EMPTY; 64]; 2];
    let mut color = 0usize;
    while color < 2 {
        let mut sq = 0usize;
        while sq < 64 {
            let file = (sq % 8) as i32;
            let rank = (sq / 8) as i32;
            let mut mask = 0u64;
            let mut df = -1i32;
            while df <= 1 {
                let f = file + df;
                if f >= 0 && f < 8 {
                    if color == Color::White as usize {
                        let mut r = rank + 1;
                        while r < 8 {
                            mask |= 1u64 << (r * 8 + f);
                            r += 1;
                        }
                    } else {
                        let mut r = 0;
                        while r < rank {
                            mask |= 1u64 << (r * 8 + f);
                            r += 1;
                        }
                    }
                }
                df += 1;
            }
            table[color][sq] = Bitboard(mask);
            sq += 1;
        }
        color += 1;
    }
    table
}

const fn init_eval_table(mg: bool) -> [[[i32; 64]; 6]; 2] {
    let mut table = [[[0i32; 64]; 6]; 2];
    let mut piece = 0usize;
    while piece < 6 {
        let mut sq = 0usize;
        while sq < 64 {
            table[Color::White as usize][piece][sq] =
                piece_base(piece, mg) + pst_value(piece, sq, mg);
            table[Color::Black as usize][piece][sq] =
                piece_base(piece, mg) + pst_value(piece, sq ^ 56, mg);
            sq += 1;
        }
        piece += 1;
    }
    table
}

const fn piece_base(piece: usize, mg: bool) -> i32 {
    if mg { MG_VAL[piece] } else { EG_VAL[piece] }
}

const fn pst_value(piece: usize, sq: usize, mg: bool) -> i32 {
    if mg {
        match piece {
            0 => MG_PAWN_PST[sq],
            1 => MG_KNIGHT_PST[sq],
            2 => MG_BISHOP_PST[sq],
            3 => MG_ROOK_PST[sq],
            4 => MG_QUEEN_PST[sq],
            _ => MG_KING_PST[sq],
        }
    } else {
        match piece {
            0 => EG_PAWN_PST[sq],
            1 => EG_KNIGHT_PST[sq],
            2 => EG_BISHOP_PST[sq],
            3 => EG_ROOK_PST[sq],
            4 => EG_QUEEN_PST[sq],
            _ => EG_KING_PST[sq],
        }
    }
}

/// All hand-crafted evaluation weights in one place.
/// The default values reproduce the original inline constants exactly, so this
/// type can be introduced without changing playing strength or the bench
/// fingerprint.  Steps 8 and 9 tune these values with SPSA and Texel tuning.
#[derive(Copy, Clone, Debug)]
pub struct EvalParams {
    // Tempo bonus (added to mg for the side to move).
    pub tempo: i32,

    // Passed-pawn rank bonus (index = relative rank 0..7).
    pub passed_mg: [i32; 8],
    pub passed_eg: [i32; 8],
    // Defended passed pawn extras.
    pub passed_defended_mg: i32,
    pub passed_defended_eg_base: i32,
    pub passed_defended_eg_rank: i32, // × rel_rank
    // Free-path (stop square empty) extras.
    pub passed_free_mg_rank: i32,     // × rel_rank
    pub passed_free_eg_rank: i32,     // × rel_rank
    pub passed_free_safe_eg_rank: i32,// × rel_rank, stop also unattacked

    // Candidate passer (can potentially become passed).
    pub candidate_mg: i32,
    pub candidate_eg: i32,

    // Pawn structure bonuses / penalties.
    pub doubled_mg: i32,
    pub doubled_eg: i32,
    pub isolated_mg: i32,
    pub isolated_eg: i32,
    pub defended_mg: i32,  // defended by own pawn (non-passer)
    pub defended_eg: i32,
    pub backward_mg: i32,
    pub backward_eg: i32,

    // Piece bonuses.
    pub bishop_pair_mg: i32,
    pub bishop_pair_eg: i32,
    pub rook_open_mg: i32,
    pub rook_open_eg: i32,
    pub rook_semi_mg: i32,
    pub rook_semi_eg: i32,
    pub rook_seventh_mg: i32,
    pub rook_seventh_eg: i32,
    pub knight_outpost_mg: i32,
    pub knight_outpost_eg: i32,

    // Mobility bonus per safe square, indexed by Piece as usize (Pawn=0 … King=5).
    pub mob_mg: [i32; 6],
    pub mob_eg: [i32; 6],

    // Pawn-threat bonuses on enemy pieces.
    pub threat_minor_mg: i32,
    pub threat_minor_eg: i32,
    pub threat_rook_mg: i32,
    pub threat_rook_eg: i32,
    pub threat_queen_mg: i32,
    pub threat_queen_eg: i32,

    // King-safety: attacker unit weights.
    pub ks_minor_weight: i32, // knight / bishop
    pub ks_rook_weight: i32,
    pub ks_queen_weight: i32,
    // Per ring-square attacked (in addition to attacker weight).
    pub ks_ring_attack: i32,
    // Safe check bonuses (piece can give check on a square not defended by king/pawns).
    pub ks_safe_check_queen: i32,
    pub ks_safe_check_rook: i32,
    pub ks_safe_check_bishop: i32,
    pub ks_safe_check_knight: i32,
    // Relief: subtract this from danger when the attacker has no queen.
    pub ks_no_queen: i32,
    // Quadratic conversion: penalty = (danger² / divisor).min(max_penalty).
    pub ks_divisor: i32,
    pub ks_max_penalty: i32,

    // Pawn shelter (positive = king-safety bonus from own pawn in front).
    pub shelter_open_king: i32, // penalty when own pawn absent on king file
    pub shelter_open_adj: i32,  // penalty when absent on adjacent file
    pub shelter_close1: i32,    // bonus when pawn is 1 step ahead
    pub shelter_close2: i32,    // bonus when pawn is 2 steps ahead

    // Pawn-storm penalty per relative rank of enemy pawn approaching the king.
    pub storm_king_file: i32,
    pub storm_adj_file: i32,

    // Rooks behind passed pawns.
    pub rook_passer_mg: i32,
    pub rook_passer_eg: i32,
    pub enemy_rook_passer_mg: i32,
    pub enemy_rook_passer_eg: i32,

    // Hanging piece penalties (undefended + attacked).
    pub hanging_minor: i32,
    pub hanging_rook: i32,
    pub hanging_queen: i32,

    // Passed-pawn king-proximity scale (endgame).
    pub king_prox_base: i32, // (enemy_dist − own_dist) × (base + rel_rank)

    // Endgame king-centralisation bonus (applied when one side is clearly winning).
    pub king_push_weight: i32,     // × (file_push + rank_push)
    pub king_prox_weight: i32,     // × (max_dist − king_distance)
    pub king_prox_max_dist: i32,   // maximum Chebyshev distance on an 8×8 board

    // Space bonus per safe central square not occupied by own pawn.
    pub space: i32,

    // Trapped bishop penalty.
    pub trapped_bishop_mg: i32,
    pub trapped_bishop_eg: i32,

    // Opposite-colour bishop endgame scaling.
    pub ocb_base: i32,
    pub ocb_per_pawn: i32,
    pub ocb_cap: i32,

    // New threat terms (7b).
    pub threat_attack_minor_mg: i32, // our minor attacks enemy minor
    pub threat_attack_minor_eg: i32,
    pub threat_rook_queen_mg: i32,   // our rook attacks enemy queen
    pub threat_rook_queen_eg: i32,
    pub threat_push_mg: i32,         // safe pawn push threatens enemy piece
    pub threat_push_eg: i32,

    // Restricted mobility: penalty for pieces with at most one safe move.
    pub restricted_mobility_mg: i32,
    pub restricted_mobility_eg: i32,

    // Bishop outpost: bishop on a square not attacked by enemy pawns, supported by own pawn.
    pub bishop_outpost_mg: i32,
    pub bishop_outpost_eg: i32,

    // Phalanx / connected pawn bonus (indexed by relative rank 0..7).
    pub phalanx_mg: [i32; 8],
    pub phalanx_eg: [i32; 8],
}

impl EvalParams {
    /// Default values — identical to the original inline constants.
    /// This is the ground truth used by both the compile-time const path
    /// and the runtime-loading path of the `tune` feature.
    pub const DEFAULT: EvalParams = EvalParams {
    tempo: 10,
    passed_mg: [0, 5, 10, 20, 35, 60, 100, 0],
    passed_eg: [0, 10, 17, 35, 62, 100, 170, 0],
    passed_defended_mg: 8,
    passed_defended_eg_base: 6,
    passed_defended_eg_rank: 4,
    passed_free_mg_rank: 2,
    passed_free_eg_rank: 6,
    passed_free_safe_eg_rank: 8,
    candidate_mg: 6,
    candidate_eg: 10,
    doubled_mg: 10,
    doubled_eg: 20,
    isolated_mg: 15,
    isolated_eg: 20,
    defended_mg: 7,
    defended_eg: 5,
    backward_mg: 10,
    backward_eg: 15,
    bishop_pair_mg: 30,
    bishop_pair_eg: 50,
    rook_open_mg: 25,
    rook_open_eg: 10,
    rook_semi_mg: 12,
    rook_semi_eg: 8,
    rook_seventh_mg: 20,
    rook_seventh_eg: 40,
    knight_outpost_mg: 25,
    knight_outpost_eg: 15,
    mob_mg: [0, 4, 5, 2, 1, 0],
    mob_eg: [0, 4, 5, 4, 2, 0],
    threat_minor_mg: 18,
    threat_minor_eg: 12,
    threat_rook_mg: 28,
    threat_rook_eg: 18,
    threat_queen_mg: 45,
    threat_queen_eg: 30,
    ks_minor_weight: 2,
    ks_rook_weight: 3,
    ks_queen_weight: 5,
    ks_ring_attack: 3,
    ks_safe_check_queen: 35,
    ks_safe_check_rook: 18,
    ks_safe_check_bishop: 10,
    ks_safe_check_knight: 10,
    ks_no_queen: 20,
    ks_divisor: 12,
    ks_max_penalty: 200,
    shelter_open_king: 20,
    shelter_open_adj: 10,
    shelter_close1: 15,
    shelter_close2: 7,
    storm_king_file: 7,
    storm_adj_file: 4,
    rook_passer_mg: 15,
    rook_passer_eg: 25,
    enemy_rook_passer_mg: 10,
    enemy_rook_passer_eg: 20,
    hanging_minor: 45,
    hanging_rook: 60,
    hanging_queen: 80,
    king_prox_base: 2,
    king_push_weight: 5,
    king_prox_weight: 4,
    king_prox_max_dist: 14,
    space: 2,
    trapped_bishop_mg: 60,
    trapped_bishop_eg: 40,
    ocb_base: 32,
    ocb_per_pawn: 4,
    ocb_cap: 48,
    threat_attack_minor_mg: 8,
    threat_attack_minor_eg: 5,
    threat_rook_queen_mg: 10,
    threat_rook_queen_eg: 8,
    threat_push_mg: 5,
    threat_push_eg: 4,
    restricted_mobility_mg: 5,
    restricted_mobility_eg: 4,
    bishop_outpost_mg: 8,
    bishop_outpost_eg: 5,
        phalanx_mg: [0, 3, 5, 7, 10, 13, 16, 0],
        phalanx_eg: [0, 3, 5, 7, 10, 13, 16, 0],
    };
}

// ---------------------------------------------------------------------------
// PARAMS: compile-time constant (default) or runtime-loaded (tune feature).
// ---------------------------------------------------------------------------
//
// Normal builds: PARAMS is a const — zero runtime cost, fully inlined.
// `--features tune` builds: PARAMS is a LazyLock that reads from the file
// path in the RAROG_TUNE_FILE environment variable on first access.
// All call sites use `PARAMS.field_name` identically in both cases because
// LazyLock<EvalParams> auto-derefs to EvalParams.

#[cfg(not(feature = "tune"))]
pub const PARAMS: EvalParams = EvalParams::DEFAULT;

#[cfg(feature = "tune")]
pub static PARAMS: std::sync::LazyLock<EvalParams> =
    std::sync::LazyLock::new(|| match std::env::var("RAROG_TUNE_FILE") {
        Ok(path) => crate::tune::load_eval_params(&path),
        Err(_) => EvalParams::DEFAULT,
    });

#[derive(Copy, Clone, Default)]
struct PawnEntry {
    key: u64,
    mg: i32,
    eg: i32,
    passed: [Bitboard; 2],
    attacks: [Bitboard; 2],
}

#[derive(Copy, Clone, Default)]
struct EvalEntry {
    key: u64,
    halfmove_clock: u8,
    value: i32,
    occupied: bool,
}

#[derive(Clone)]
pub struct Evaluator {
    pawn_table: Vec<PawnEntry>,
    eval_table: Vec<EvalEntry>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            pawn_table: vec![PawnEntry::default(); PAWN_TABLE_SIZE],
            eval_table: vec![EvalEntry::default(); EVAL_TABLE_SIZE],
        }
    }
}

impl Evaluator {
    pub fn clear_pawn_table(&mut self) {
        self.pawn_table.fill(PawnEntry::default());
        self.eval_table.fill(EvalEntry::default());
    }

    pub fn evaluate_result(&self, result: GameResult, color: Color, ply: usize) -> i32 {
        let mate = MATE_SCORE - ply as i32;
        match (result, color) {
            (GameResult::WhiteCheckmates, Color::White)
            | (GameResult::BlackCheckmates, Color::Black) => mate,
            (GameResult::WhiteCheckmates, Color::Black)
            | (GameResult::BlackCheckmates, Color::White) => -mate,
            (GameResult::Stalemate, _) | (GameResult::Draw, _) => 0,
        }
    }

    pub fn evaluate(&mut self, board: &Board) -> i32 {
        let eval_slot = board.hash as usize & (EVAL_TABLE_SIZE - 1);
        let cached = self.eval_table[eval_slot];
        if cached.occupied
            && cached.key == board.hash
            && cached.halfmove_clock == board.halfmove_clock
        {
            return cached.value;
        }

        let atk = &*ATTACKS;
        let mut mg = 0;
        let mut eg = 0;
        let mut phase = 0;

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            for piece in Piece::ALL {
                let mut bb = board.pieces(color, piece);
                phase += bb.count() as i32 * PHASE_W[piece as usize];
                while bb.any() {
                    let sq = bb.pop_lsb();
                    mg += sign * MG_TABLE[color as usize][piece as usize][sq.index()];
                    eg += sign * EG_TABLE[color as usize][piece as usize][sq.index()];
                }
            }
        }
        phase = phase.min(TOTAL_PHASE);

        let mut passed = [Bitboard::EMPTY; 2];
        let mut pawn_attacks = [Bitboard::EMPTY; 2];
        let (pawn_mg, pawn_eg) = self.eval_pawns(board, atk, &mut passed, &mut pawn_attacks);
        mg += pawn_mg;
        eg += pawn_eg;

        self.eval_piece_activity(board, atk, &mut mg, &mut eg, &passed, &pawn_attacks, phase);

        let tempo = if board.side_to_move() == Color::White {
            PARAMS.tempo
        } else {
            -PARAMS.tempo
        };
        mg += tempo;

        let mut score = (mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
        score = scale_drawish_endgames(board, score);
        let rule50 = board.halfmove_clock.min(100) as i32;
        score -= score * rule50 / 199;
        let value = if board.side_to_move() == Color::White {
            score
        } else {
            -score
        };
        self.eval_table[eval_slot] = EvalEntry {
            key: board.hash,
            halfmove_clock: board.halfmove_clock,
            value,
            occupied: true,
        };
        value
    }

    /// Like [`evaluate`] but returns a cheap material+PST estimate when the
    /// position is clearly outside `[alpha, beta]` by at least `LAZY_MARGIN`,
    /// skipping the expensive pawn-structure, mobility, and king-safety loops.
    /// The full-eval cache is still consulted first; the lazy path is only taken
    /// on a cache miss where the cheap score is far from the window.
    pub fn evaluate_lazy(&mut self, board: &Board, alpha: i32, beta: i32) -> i32 {
        let eval_slot = board.hash as usize & (EVAL_TABLE_SIZE - 1);
        let cached = self.eval_table[eval_slot];
        if cached.occupied
            && cached.key == board.hash
            && cached.halfmove_clock == board.halfmove_clock
        {
            return cached.value;
        }

        // Cheap eval: material + PSQT + tempo (phase-interpolated).
        let mut mg = 0i32;
        let mut eg = 0i32;
        let mut phase = 0i32;
        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            for piece in Piece::ALL {
                let mut bb = board.pieces(color, piece);
                phase += bb.count() as i32 * PHASE_W[piece as usize];
                while bb.any() {
                    let sq = bb.pop_lsb();
                    mg += sign * MG_TABLE[color as usize][piece as usize][sq.index()];
                    eg += sign * EG_TABLE[color as usize][piece as usize][sq.index()];
                }
            }
        }
        phase = phase.min(TOTAL_PHASE);
        let tempo = if board.side_to_move() == Color::White { PARAMS.tempo } else { -PARAMS.tempo };
        mg += tempo;
        let cheap_score = (mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
        let cheap_value = if board.side_to_move() == Color::White {
            cheap_score
        } else {
            -cheap_score
        };

        // Lazy gate: skip the expensive loops when the position is far outside
        // the window regardless of pawn structure, mobility, or king safety.
        if cheap_value + LAZY_MARGIN < alpha || cheap_value - LAZY_MARGIN > beta {
            return cheap_value;
        }

        // Full evaluation (computes and caches the complete result).
        self.evaluate(board)
    }

    fn eval_pawns(
        &mut self,
        board: &Board,
        atk: &AttackTables,
        passed: &mut [Bitboard; 2],
        attacks: &mut [Bitboard; 2],
    ) -> (i32, i32) {
        let key = board.pawn_key();
        let slot = key as usize & (PAWN_TABLE_SIZE - 1);
        let cached = self.pawn_table[slot];
        if cached.key == key {
            *passed = cached.passed;
            *attacks = cached.attacks;
            return (cached.mg, cached.eg);
        }

        let mut mg = 0;
        let mut eg = 0;

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let us = color;
            let them = !us;
            let our_pawns = board.pieces(us, Piece::Pawn);
            let their_pawns = board.pieces(them, Piece::Pawn);
            let occupied = board.occupied();
            attacks[us as usize] = if us == Color::White {
                our_pawns.north_east() | our_pawns.north_west()
            } else {
                our_pawns.south_east() | our_pawns.south_west()
            };

            let mut tmp = our_pawns;
            passed[us as usize] = Bitboard::EMPTY;
            while tmp.any() {
                let sq = tmp.pop_lsb();
                let file = SQUARE_FILE[sq.index()];
                let rel_rank = relative_rank(us, sq) as usize;
                let adjacent = ADJACENT_FILES[file];

                if (PASSED_PAWN_MASKS[us as usize][sq.index()] & their_pawns).is_empty() {
                    passed[us as usize] |= Bitboard::from(sq);
                    mg += sign * PARAMS.passed_mg[rel_rank];
                    eg += sign * PARAMS.passed_eg[rel_rank];

                    if (atk.pawn(them, sq) & our_pawns).any() {
                        mg += sign * PARAMS.passed_defended_mg;
                        eg += sign * (PARAMS.passed_defended_eg_base
                            + rel_rank as i32 * PARAMS.passed_defended_eg_rank);
                    }

                    if let Some(stop) = forward_square(us, sq)
                        && (occupied & Bitboard::from(stop)).is_empty()
                    {
                        mg += sign * (rel_rank as i32 * PARAMS.passed_free_mg_rank);
                        eg += sign * (rel_rank as i32 * PARAMS.passed_free_eg_rank);
                        if board.attackers_to_color(stop, occupied, them).is_empty() {
                            eg += sign * (rel_rank as i32 * PARAMS.passed_free_safe_eg_rank);
                        }
                    }
                } else if rel_rank >= 3
                    && (atk.pawn(them, sq) & our_pawns).any()
                    && (their_pawns
                        & adjacent
                        & FORWARD_RANKS[us as usize][SQUARE_RANK[sq.index()]])
                    .is_empty()
                {
                    mg += sign * PARAMS.candidate_mg;
                    eg += sign * PARAMS.candidate_eg;
                }

                let file_bb = FILE_BBS[file];
                if (our_pawns & file_bb).more_than_one() {
                    mg -= sign * PARAMS.doubled_mg;
                    eg -= sign * PARAMS.doubled_eg;
                }
                if (our_pawns & adjacent).is_empty() {
                    mg -= sign * PARAMS.isolated_mg;
                    eg -= sign * PARAMS.isolated_eg;
                }
                if (atk.pawn(them, sq) & our_pawns).any() {
                    mg += sign * PARAMS.defended_mg;
                    eg += sign * PARAMS.defended_eg;
                }
                // Phalanx: a friendly pawn directly to the left or right on the same rank.
                let west = sq.0.checked_sub(1).filter(|&s| SQUARE_FILE[s as usize] == file.wrapping_sub(1));
                let east = sq.0.checked_add(1).filter(|&s| SQUARE_FILE[s as usize] == file + 1);
                let is_phalanx =
                    west.is_some_and(|s| (our_pawns & Bitboard::from(Square(s))).any())
                    || east.is_some_and(|s| (our_pawns & Bitboard::from(Square(s))).any());
                if is_phalanx {
                    mg += sign * PARAMS.phalanx_mg[rel_rank];
                    eg += sign * PARAMS.phalanx_eg[rel_rank];
                }

                let stop_sq = if us == Color::White {
                    sq.0.checked_add(8)
                } else {
                    sq.0.checked_sub(8)
                };
                if (our_pawns & PASSED_PAWN_MASKS[them as usize][sq.index()] & adjacent).is_empty()
                    && let Some(stop) = stop_sq.filter(|sq| *sq < 64)
                    && (atk.pawn(us, Square(stop)) & their_pawns).any()
                {
                    mg -= sign * PARAMS.backward_mg;
                    eg -= sign * PARAMS.backward_eg;
                }
            }
        }

        self.pawn_table[slot] = PawnEntry {
            key,
            mg,
            eg,
            passed: *passed,
            attacks: *attacks,
        };
        (mg, eg)
    }

    fn eval_piece_activity(
        &self,
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

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let them = !color;
            let own_pawns = pawns[color as usize];
            let their_pawns = pawns[them as usize];
            let own_occ = color_occ[color as usize];

            if board.pieces(color, Piece::Bishop).more_than_one() {
                *mg += sign * PARAMS.bishop_pair_mg;
                *eg += sign * PARAMS.bishop_pair_eg;
            }

            let mut rooks = board.pieces(color, Piece::Rook);
            while rooks.any() {
                let sq = rooks.pop_lsb();
                let file = SQUARE_FILE[sq.index()];
                let own_file_empty = (own_pawns & FILE_BBS[file]).is_empty();
                let their_file_empty = (their_pawns & FILE_BBS[file]).is_empty();
                if own_file_empty && their_file_empty {
                    *mg += sign * PARAMS.rook_open_mg;
                    *eg += sign * PARAMS.rook_open_eg;
                } else if own_file_empty {
                    *mg += sign * PARAMS.rook_semi_mg;
                    *eg += sign * PARAMS.rook_semi_eg;
                }
                if relative_rank(color, sq) == 6 {
                    *mg += sign * PARAMS.rook_seventh_mg;
                    *eg += sign * PARAMS.rook_seventh_eg;
                }
            }

            let mut knights = board.pieces(color, Piece::Knight);
            while knights.any() {
                let sq = knights.pop_lsb();
                if relative_rank(color, sq) >= 4
                    && (atk.pawn(them, sq) & own_pawns).any()
                    && (atk.pawn(color, sq) & their_pawns).is_empty()
                {
                    *mg += sign * PARAMS.knight_outpost_mg;
                    *eg += sign * PARAMS.knight_outpost_eg;
                }
            }

            let safe = !pawn_attacks[them as usize];
            for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
                let mut pieces = board.pieces(color, piece);
                while pieces.any() {
                    let sq = pieces.pop_lsb();
                    let attacks = attacks_for(atk, piece, sq, occupied);
                    let mobility = (attacks & safe & !own_occ).count() as i32;
                    *mg += sign * mobility * mobility_mg(piece);
                    *eg += sign * mobility * mobility_eg(piece);
                    // Restricted mobility: extra penalty for pieces with ≤ 1 safe move.
                    if mobility <= 1 {
                        *mg -= sign * PARAMS.restricted_mobility_mg;
                        *eg -= sign * PARAMS.restricted_mobility_eg;
                    }
                }
            }

            let mut threats = pawn_attacks[color as usize] & board.color_occ(them);
            while threats.any() {
                let sq = threats.pop_lsb();
                match board.piece_on(sq) {
                    Some(Piece::Knight | Piece::Bishop) => {
                        *mg += sign * PARAMS.threat_minor_mg;
                        *eg += sign * PARAMS.threat_minor_eg;
                    }
                    Some(Piece::Rook) => {
                        *mg += sign * PARAMS.threat_rook_mg;
                        *eg += sign * PARAMS.threat_rook_eg;
                    }
                    Some(Piece::Queen) => {
                        *mg += sign * PARAMS.threat_queen_mg;
                        *eg += sign * PARAMS.threat_queen_eg;
                    }
                    _ => {}
                }
            }

            // Minor piece attacks enemy minor.
            let our_minors =
                board.pieces(color, Piece::Knight) | board.pieces(color, Piece::Bishop);
            let their_minors =
                board.pieces(them, Piece::Knight) | board.pieces(them, Piece::Bishop);
            let mut minor_iter = our_minors;
            while minor_iter.any() {
                let sq = minor_iter.pop_lsb();
                let piece = board.piece_on(sq).unwrap_or(Piece::Knight);
                let attacks = attacks_for(atk, piece, sq, occupied);
                if (attacks & their_minors).any() {
                    *mg += sign * PARAMS.threat_attack_minor_mg;
                    *eg += sign * PARAMS.threat_attack_minor_eg;
                }
            }

            // Rook attacks enemy queen.
            let enemy_queens = board.pieces(them, Piece::Queen);
            let mut our_rooks = board.pieces(color, Piece::Rook);
            while our_rooks.any() && enemy_queens.any() {
                let sq = our_rooks.pop_lsb();
                if (atk.rook(sq, occupied) & enemy_queens).any() {
                    *mg += sign * PARAMS.threat_rook_queen_mg;
                    *eg += sign * PARAMS.threat_rook_queen_eg;
                }
            }

            // Safe pawn push threats: push one step and attack an enemy non-pawn.
            let enemy_non_pawns = board.color_occ(them) & !board.pieces(them, Piece::Pawn);
            let our_pawns_push_src = board.pieces(color, Piece::Pawn)
                & !(if color == Color::White { Bitboard::RANK_7 } else { Bitboard::RANK_2 });
            let push_dest = if color == Color::White {
                our_pawns_push_src.north()
            } else {
                our_pawns_push_src.south()
            } & !occupied;
            let push_attacks = if color == Color::White {
                push_dest.north_east() | push_dest.north_west()
            } else {
                push_dest.south_east() | push_dest.south_west()
            };
            let push_threat_count = (push_attacks & enemy_non_pawns).count() as i32;
            if push_threat_count > 0 {
                *mg += sign * push_threat_count * PARAMS.threat_push_mg;
                *eg += sign * push_threat_count * PARAMS.threat_push_eg;
            }

            // Bishop outpost: bishop on a hole (not attacked by enemy pawns) supported
            // by own pawn.  Uses the same pawn-attack geometry as knight outpost.
            let mut bishops_iter = board.pieces(color, Piece::Bishop);
            while bishops_iter.any() {
                let sq = bishops_iter.pop_lsb();
                let not_attacked = (atk.pawn(color, sq) & their_pawns).is_empty();
                let supported = (atk.pawn(them, sq) & own_pawns).any();
                if not_attacked && supported {
                    *mg += sign * PARAMS.bishop_outpost_mg;
                    *eg += sign * PARAMS.bishop_outpost_eg;
                }
            }

            self.eval_king_safety(board, atk, color, sign, mg, occupied, &pawns);
            self.eval_rooks_behind_passers(board, color, sign, passed, mg, eg);
            self.eval_hanging_pieces(board, color, sign, mg, eg, occupied);
        }

        self.eval_passed_pawn_king_proximity(board, passed, eg);
        self.eval_space(board, pawn_attacks, mg);
        if phase < TOTAL_PHASE / 2 {
            self.eval_trapped_bishops(board, atk, mg, eg);
        }

        let approximate = (*mg + *eg) / 2;
        if approximate.abs() > 200 {
            let winning = if approximate > 0 {
                Color::White
            } else {
                Color::Black
            };
            let losing = !winning;
            let sign = color_sign(winning);
            let lksq = board.king_sq(losing);
            let wksq = board.king_sq(winning);
            let lfile = SQUARE_FILE[lksq.index()] as i32;
            let lrank = SQUARE_RANK[lksq.index()] as i32;
            let file_push = (3 - lfile).max(lfile - 4);
            let rank_push = (3 - lrank).max(lrank - 4);
            let king_distance = KING_DISTANCE[wksq.index()][lksq.index()] as i32;
            *eg += sign
                * (PARAMS.king_push_weight * (file_push + rank_push)
                    + (PARAMS.king_prox_max_dist - king_distance) * PARAMS.king_prox_weight);
        }
    }

    fn eval_space(&self, board: &Board, pawn_attacks: &[Bitboard; 2], mg: &mut i32) {
        let center_files = FILE_BBS[2] | FILE_BBS[3] | FILE_BBS[4] | FILE_BBS[5];
        let white_space_ranks = Bitboard::RANK_2 | Bitboard::RANK_3 | Bitboard::RANK_4;
        let black_space_ranks = Bitboard::RANK_5 | Bitboard::RANK_6 | Bitboard::RANK_7;
        let white_space = center_files
            & white_space_ranks
            & !board.pieces(Color::White, Piece::Pawn)
            & !pawn_attacks[Color::Black as usize];
        let black_space = center_files
            & black_space_ranks
            & !board.pieces(Color::Black, Piece::Pawn)
            & !pawn_attacks[Color::White as usize];
        *mg += (white_space.count() as i32 - black_space.count() as i32) * PARAMS.space;
    }

    fn eval_king_safety(
        &self,
        board: &Board,
        atk: &AttackTables,
        color: Color,
        sign: i32,
        mg: &mut i32,
        occupied: Bitboard,
        pawns: &[Bitboard; 2],
    ) {
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

        // Squares defended by the side whose king we're evaluating (own king + own pawns).
        // A "safe check" square is one NOT defended by king or pawns.
        let own_pawn_bb = board.pieces(color, Piece::Pawn);
        let own_pawn_attacks = if color == Color::White {
            own_pawn_bb.north_east() | own_pawn_bb.north_west()
        } else {
            own_pawn_bb.south_east() | own_pawn_bb.south_west()
        };
        let defended_by_king_and_pawns = king_attacks | own_pawn_attacks;

        let mut king_danger = 0i32;
        for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
            let piece_weight = match piece {
                Piece::Knight | Piece::Bishop => PARAMS.ks_minor_weight,
                Piece::Rook => PARAMS.ks_rook_weight,
                Piece::Queen => PARAMS.ks_queen_weight,
                _ => 0,
            };
            let safe_check_bonus = match piece {
                Piece::Knight => PARAMS.ks_safe_check_knight,
                Piece::Bishop => PARAMS.ks_safe_check_bishop,
                Piece::Rook => PARAMS.ks_safe_check_rook,
                Piece::Queen => PARAMS.ks_safe_check_queen,
                _ => 0,
            };
            // Squares this piece type attacks from the king square (for check detection).
            let king_check_squares = match piece {
                Piece::Knight => atk.knight(king),
                Piece::Bishop => atk.bishop(king, occupied),
                Piece::Rook => atk.rook(king, occupied),
                Piece::Queen => atk.queen(king, occupied),
                _ => Bitboard::EMPTY,
            };

            let mut pieces = board.pieces(them, piece);
            while pieces.any() {
                let sq = pieces.pop_lsb();
                let attacks = attacks_for(atk, piece, sq, occupied);
                let zone_attacks = attacks & zone;
                if zone_attacks.any() {
                    king_danger += piece_weight;
                    king_danger += zone_attacks.count() as i32 * PARAMS.ks_ring_attack;
                }
                // Safe check: can this piece move to a square that checks the king
                // and is not defended by the king's own pawns/king?
                let safe_checks = attacks
                    & king_check_squares
                    & !defended_by_king_and_pawns
                    & !board.color_occ(them);
                if safe_checks.any() {
                    king_danger += safe_check_bonus;
                }
            }
        }

        // No-queen relief: attacking without a queen is far less dangerous.
        if board.pieces(them, Piece::Queen).is_empty() {
            king_danger -= PARAMS.ks_no_queen;
        }

        // Convert to a penalty quadratically; only meaningful when positive.
        if king_danger > 0 {
            let penalty =
                (king_danger * king_danger / PARAMS.ks_divisor).min(PARAMS.ks_max_penalty);
            *mg -= sign * penalty;
        }

        let king_file = SQUARE_FILE[king.index()] as i32;
        if king_file <= 2 || king_file >= 5 {
            let king_rank = SQUARE_RANK[king.index()] as i32;
            for df in -1..=1 {
                let file = king_file + df;
                if !(0..8).contains(&file) {
                    continue;
                }
                let file_pawns = pawns[color as usize] & FILE_BBS[file as usize];
                let in_front = file_pawns & FORWARD_RANKS[color as usize][king_rank as usize];
                if in_front.is_empty() {
                    *mg -= sign * if df == 0 { PARAMS.shelter_open_king } else { PARAMS.shelter_open_adj };
                } else {
                    let pawn_sq = if color == Color::White {
                        in_front.lsb()
                    } else {
                        in_front.msb()
                    };
                    let distance = if color == Color::White {
                        SQUARE_RANK[pawn_sq.index()] as i32 - king_rank
                    } else {
                        king_rank - SQUARE_RANK[pawn_sq.index()] as i32
                    };
                    if distance == 1 {
                        *mg += sign * PARAMS.shelter_close1;
                    } else if distance == 2 {
                        *mg += sign * PARAMS.shelter_close2;
                    }
                }
            }
        }

        let enemy_pawns = pawns[them as usize];
        let mut storm_files = Bitboard::EMPTY;
        let king_file = SQUARE_FILE[king.index()] as i32;
        for df in -1..=1 {
            let file = king_file + df;
            if (0..8).contains(&file) {
                storm_files |= FILE_BBS[file as usize];
            }
        }
        let mut storm = enemy_pawns & storm_files;
        while storm.any() {
            let pawn = storm.pop_lsb();
            let rel = relative_rank(them, pawn) as i32;
            if rel >= 3 {
                *mg -= sign
                    * (rel
                        * if SQUARE_FILE[pawn.index()] == SQUARE_FILE[king.index()] {
                            PARAMS.storm_king_file
                        } else {
                            PARAMS.storm_adj_file
                        });
            }
        }
    }

    fn eval_rooks_behind_passers(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        passed: &[Bitboard; 2],
        mg: &mut i32,
        eg: &mut i32,
    ) {
        let them = !color;
        let mut rooks = board.pieces(color, Piece::Rook);
        while rooks.any() {
            let rook = rooks.pop_lsb();
            let file = SQUARE_FILE[rook.index()];
            let file_passers = passed[color as usize] & FILE_BBS[file];
            if file_passers.any() {
                let passer = if color == Color::White {
                    file_passers.lsb()
                } else {
                    file_passers.msb()
                };
                let behind = if color == Color::White {
                    SQUARE_RANK[rook.index()] < SQUARE_RANK[passer.index()]
                } else {
                    SQUARE_RANK[rook.index()] > SQUARE_RANK[passer.index()]
                };
                if behind {
                    *mg += sign * PARAMS.rook_passer_mg;
                    *eg += sign * PARAMS.rook_passer_eg;
                }
            }

            let mut enemy_rooks = board.pieces(them, Piece::Rook) & FILE_BBS[file];
            while enemy_rooks.any() && file_passers.any() {
                let enemy = enemy_rooks.pop_lsb();
                let passer = if color == Color::White {
                    file_passers.lsb()
                } else {
                    file_passers.msb()
                };
                let behind = if color == Color::White {
                    SQUARE_RANK[enemy.index()] < SQUARE_RANK[passer.index()]
                } else {
                    SQUARE_RANK[enemy.index()] > SQUARE_RANK[passer.index()]
                };
                if behind {
                    *mg -= sign * PARAMS.enemy_rook_passer_mg;
                    *eg -= sign * PARAMS.enemy_rook_passer_eg;
                }
            }
        }
    }

    fn eval_hanging_pieces(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        mg: &mut i32,
        eg: &mut i32,
        occupied: Bitboard,
    ) {
        let them = !color;
        let mut pieces = board.color_occ(color)
            & !board.pieces(color, Piece::Pawn)
            & !board.pieces(color, Piece::King);
        while pieces.any() {
            let sq = pieces.pop_lsb();
            let Some(piece) = board.piece_on(sq) else {
                continue;
            };
            let attackers = board.attackers_to_color(sq, occupied, them);
            let defenders = board.attackers_to_color(sq, occupied, color);
            if attackers.is_empty() || defenders.any() {
                continue;
            }
            let penalty = hanging_piece_penalty(piece);
            *mg -= sign * penalty;
            *eg -= sign * penalty;
        }
    }

    fn eval_passed_pawn_king_proximity(&self, board: &Board, passed: &[Bitboard; 2], eg: &mut i32) {
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
                *eg += sign * (enemy_dist - own_dist) * (PARAMS.king_prox_base + rel_rank);
            }
        }
    }

    fn eval_trapped_bishops(&self, board: &Board, atk: &AttackTables, mg: &mut i32, eg: &mut i32) {
        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let mut bishops = board.pieces(color, Piece::Bishop);
            while bishops.any() {
                let sq = bishops.pop_lsb();
                if (atk.bishop(sq, board.occupied()) & !board.color_occ(color)).is_empty() {
                    *mg -= sign * PARAMS.trapped_bishop_mg;
                    *eg -= sign * PARAMS.trapped_bishop_eg;
                }
            }
        }
    }
}

#[inline(always)]
pub fn piece_value(piece: Piece) -> i32 {
    unsafe { *PIECE_VALUES.get_unchecked(piece as usize) }
}

#[inline(always)]
fn hanging_piece_penalty(piece: Piece) -> i32 {
    match piece {
        Piece::Knight | Piece::Bishop => PARAMS.hanging_minor,
        Piece::Rook => PARAMS.hanging_rook,
        Piece::Queen => PARAMS.hanging_queen,
        _ => 0,
    }
}

#[inline(always)]
fn color_sign(color: Color) -> i32 {
    if color == Color::White { 1 } else { -1 }
}

#[inline(always)]
fn attacks_for(atk: &AttackTables, piece: Piece, sq: Square, occ: Bitboard) -> Bitboard {
    match piece {
        Piece::Pawn => Bitboard::EMPTY,
        Piece::Knight => atk.knight(sq),
        Piece::Bishop => atk.bishop(sq, occ),
        Piece::Rook => atk.rook(sq, occ),
        Piece::Queen => atk.queen(sq, occ),
        Piece::King => atk.king(sq),
    }
}

#[inline(always)]
fn mobility_mg(piece: Piece) -> i32 {
    PARAMS.mob_mg[piece as usize]
}

#[inline(always)]
fn mobility_eg(piece: Piece) -> i32 {
    PARAMS.mob_eg[piece as usize]
}

#[inline(always)]
fn relative_rank(color: Color, sq: Square) -> u8 {
    RELATIVE_RANKS[color as usize][sq.index()]
}

fn forward_square(color: Color, sq: Square) -> Option<Square> {
    match color {
        Color::White => sq.0.checked_add(8).filter(|to| *to < 64).map(Square),
        Color::Black => sq.0.checked_sub(8).map(Square),
    }
}

fn scale_drawish_endgames(board: &Board, mut score: i32) -> i32 {
    let white_bishops = board.pieces(Color::White, Piece::Bishop);
    let black_bishops = board.pieces(Color::Black, Piece::Bishop);
    if white_bishops.any()
        && !white_bishops.more_than_one()
        && black_bishops.any()
        && !black_bishops.more_than_one()
    {
        let white_dark = (white_bishops & Bitboard::DARK_SQUARES).any();
        let black_dark = (black_bishops & Bitboard::DARK_SQUARES).any();
        if white_dark != black_dark {
            let pawns = (board.pieces(Color::White, Piece::Pawn)
                | board.pieces(Color::Black, Piece::Pawn))
            .count() as i32;
            let scale = PARAMS.ocb_base + pawns * PARAMS.ocb_per_pawn;
            score = score * scale.min(PARAMS.ocb_cap) / PARAMS.ocb_cap;
        }
    }

    if has_only_king(board, Color::White) && has_only_knights(board, Color::Black, 2) {
        return 0;
    }
    if has_only_king(board, Color::Black) && has_only_knights(board, Color::White, 2) {
        return 0;
    }

    score
}

fn has_only_king(board: &Board, color: Color) -> bool {
    board.color_occ(color) == Bitboard::from(board.king_sq(color))
}

fn has_only_knights(board: &Board, color: Color, count: u32) -> bool {
    board.pieces(color, Piece::Pawn).is_empty()
        && board.pieces(color, Piece::Bishop).is_empty()
        && board.pieces(color, Piece::Rook).is_empty()
        && board.pieces(color, Piece::Queen).is_empty()
        && board.pieces(color, Piece::Knight).count() == count
}
