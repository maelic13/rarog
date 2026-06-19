#[cfg(feature = "texel")]
use std::cell::RefCell;

use crate::board::attacks::AttackTables;
use crate::board::movegen;
use crate::board::{ATTACKS, Bitboard, Board, CastlingRights, Color, GameResult, Piece, Square};

pub const MATE_SCORE: i32 = 32_000;
pub const INF_SCORE: i32 = 32_001;
pub const VALUE_NONE: i32 = 32_002;

const PAWN_TABLE_SIZE: usize = 16_384;
const EVAL_TABLE_SIZE: usize = 32_768;
const TOTAL_PHASE: i32 = 24;
/// Lazy-eval threshold (Phase 3.16): if the tapered material + PST + pawn score
/// already exceeds this, the expensive positional block is skipped. Chosen so
/// the skipped terms cannot flip the sign at the current (seeded-0) eval; it is
/// an SPRT-tunable knob and should be re-checked once Phase 4 grows the
/// positional weights.
const LAZY_MARGIN: i32 = 600;

// Material values (Phase 4.6 fitted). King pinned 0. mg values rescaled up ~×1.1
// vs the old PeSTO seeds to match the lower fitted K (1.70) — ratios-to-pawn are
// essentially unchanged, so this is a benign scale shift, not a distortion.
const MG_VAL: [i32; 6] = [89, 396, 419, 538, 1130, 0];
const EG_VAL: [i32; 6] = [105, 229, 285, 482, 927, 0];
const PHASE_W: [i32; 6] = [0, 1, 1, 2, 4, 0];
const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, MATE_SCORE];

const MG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -51, -23, -11, 16, -12, 53, 64, -57, -55, -45, -23, 2, -14, -11, 10,
    -44, -56, -43, -8, -17, 5, -10, -49, -49, -36, -11, -7, -2, 0, 2, -9, -17, -5, 24, 54, 38, 85,
    77, 102, 33, 221, 256, 180, 216, 192, 228, 139, -29, 0, 0, 0, 0, 0, 0, 0, 0,
];
const EG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 17, 19, 13, -32, 5, -7, -17, -11, 31, 21, 3, -10, -1, 3, -9, -5, 39,
    35, -3, -1, -15, -11, 10, 2, 57, 31, 1, -23, -22, -17, 5, -3, 82, 62, 34, 11, -16, 2, 5, 28,
    84, 63, 92, 36, 23, 27, 71, 88, 0, 0, 0, 0, 0, 0, 0, 0,
];
const MG_KNIGHT_PST: [i32; 64] = [
    -142, -150, -112, -21, -20, -23, 26, -56, -45, -108, -19, 6, 15, -7, -47, 40, -86, 26, 7, 40,
    89, 54, 41, -34, 30, 31, 64, 48, 87, 92, 122, 64, 11, -12, 47, 68, 46, 95, 99, 87, -101, -29,
    38, 60, 120, 107, 119, -35, -26, 20, 62, 95, 80, 48, 39, 16, -184, -43, -62, 35, 24, -17, -6,
    19,
];
const EG_KNIGHT_PST: [i32; 64] = [
    -98, 17, -6, -31, -21, -59, -39, -40, -73, 3, -24, -23, -27, -13, 5, -73, 17, -37, -18, -21,
    -29, -49, -44, -7, -5, -17, -6, -9, -16, -33, -36, -24, 25, -18, -7, -13, -15, -36, -44, -4,
    23, -18, -6, -14, -28, -33, -57, 13, 21, 3, -24, -35, -36, -26, -7, 10, -102, -54, 1, -14, -8,
    -49, 11, -47,
];
const MG_BISHOP_PST: [i32; 64] = [
    -65, -45, -8, -71, 3, -36, -32, -68, 63, -16, -13, -3, 8, 20, 15, -58, 2, 41, 30, 31, 28, 52,
    8, 11, 40, 13, 88, 54, 63, 35, 31, 12, 51, 27, 19, 77, 48, 52, 17, -11, -14, 11, 22, 33, 66,
    68, 26, 4, 14, 37, 6, -26, -60, 40, -35, -80, -6, 4, -45, -1, -60, -36, 40, -79,
];
const EG_BISHOP_PST: [i32; 64] = [
    1, -10, -16, -18, -22, -28, 0, -50, -9, 6, -7, -4, -21, -22, -43, -10, -1, -15, 6, -17, 4, -35,
    -6, -21, 3, 18, -12, 10, -16, -9, -14, -5, 17, 8, 11, -8, 5, -1, 0, 3, 17, 25, 12, 11, -11, 3,
    19, 23, 30, 11, 15, 9, 38, 12, 31, 10, 64, 24, 20, -3, 23, 14, -4, 19,
];
const MG_ROOK_PST: [i32; 64] = [
    -17, 3, -7, 7, 21, 35, 16, -30, -35, -6, -17, 3, 23, 24, 32, 5, -79, -77, -43, -10, -19, -3,
    -32, -36, -86, -4, -22, -15, 12, 24, 65, -63, -66, 25, 6, 68, 34, 98, 47, 4, -28, 23, -9, 47,
    72, 127, 88, 50, -15, 14, 63, 66, 83, 108, 119, 110, -32, 15, 55, 45, 25, 53, 58, 82,
];
const EG_ROOK_PST: [i32; 64] = [
    -28, -34, -19, -25, -46, -51, -32, -51, -13, -9, -3, -5, -26, -25, -30, -29, 21, 12, 5, -12,
    -6, -9, 10, 12, 45, 6, 12, 4, -10, -11, -7, 18, 37, -1, -2, -24, -23, -36, -15, 2, 22, -4, 1,
    -21, -27, -37, -17, -22, 33, 17, -8, -10, -8, -16, -17, -15, 54, 41, 19, 10, 23, 15, 17, 22,
];
const MG_QUEEN_PST: [i32; 64] = [
    40, 0, 4, 19, 25, 8, -24, 2, -3, 4, -15, 3, 14, 55, -8, 52, 5, 11, -20, -24, 14, 66, 32, -10,
    15, -2, -4, 47, 41, 28, 34, 19, -17, -55, -54, -21, 44, 7, 58, -20, 8, -7, -24, -52, 34, 63,
    67, 55, -72, -69, -42, -22, -25, 16, 5, 11, -27, -51, -24, 0, 22, 51, 33, 28,
];
const EG_QUEEN_PST: [i32; 64] = [
    -37, -1, -15, -10, -13, -28, -25, -32, 10, 48, 47, 23, 20, -28, 16, -23, -38, 22, 74, 83, 45,
    18, 44, 55, 52, 64, 82, 65, 51, 55, 85, 64, -38, 16, -3, -7, -20, 17, -35, 12, -87, -60, -25,
    15, 22, -15, -48, -48, -37, -31, 13, 12, 48, 25, 1, -17, -38, 2, 1, 10, -4, 19, 3, 6,
];
const MG_KING_PST: [i32; 64] = [
    -37, 45, -6, -122, -39, -80, 42, 43, 29, 17, -53, -106, -70, -33, 33, 31, -21, -42, -26, -39,
    -72, 16, 64, -8, -60, 35, -93, -143, -135, -54, -16, -44, 24, 19, -116, -144, -126, -42, -6,
    -34, 19, 61, 48, -59, -66, 43, 61, 21, 83, 51, 42, 49, 20, 63, 24, 20, -14, 75, 74, 40, -9, 31,
    66, 32,
];
const EG_KING_PST: [i32; 64] = [
    -14, -39, -6, 25, 12, -6, -46, -66, -6, -3, 20, 35, 23, 1, -14, -22, 13, 14, 25, 32, 35, 8,
    -13, -1, 21, 10, 48, 62, 55, 38, 23, 16, 19, 53, 65, 62, 66, 67, 61, 32, 36, 61, 55, 45, 61,
    73, 81, 57, 43, 19, 56, 39, 40, 86, 102, 48, -67, 17, 26, 18, 28, 64, 52, -3,
];

/// Flatten the six per-piece PST consts into one `[i32; 384]` array in
/// `Piece::ALL` order (Pawn,Knight,Bishop,Rook,Queen,King), white POV — the
/// uniform `[i32; N]` shape every `EvalParams` field uses (Phase 3.1).
fn build_default_pst(mg: bool) -> [i32; 384] {
    let tables: [&[i32; 64]; 6] = if mg {
        [
            &MG_PAWN_PST,
            &MG_KNIGHT_PST,
            &MG_BISHOP_PST,
            &MG_ROOK_PST,
            &MG_QUEEN_PST,
            &MG_KING_PST,
        ]
    } else {
        [
            &EG_PAWN_PST,
            &EG_KNIGHT_PST,
            &EG_BISHOP_PST,
            &EG_ROOK_PST,
            &EG_QUEEN_PST,
            &EG_KING_PST,
        ]
    };
    let mut out = [0i32; 384];
    for (piece, table) in tables.iter().enumerate() {
        out[piece * 64..piece * 64 + 64].copy_from_slice(table.as_slice());
    }
    out
}

/// Every tunable eval weight, hoisted out of inline literals (Phase 3.1).
/// Uniform `[i32; N]` shape (scalars as `[i32; 1]`) so the Phase 3.2/3.3
/// tune-time loader and Texel tuner can address every field by
/// `(name, index)` through `EVAL_PARAM_NAMES`/`get`/`set` below. This step is
/// a default-equivalence refactor only: every default here reproduces the
/// constant it replaces exactly, so `bench 13` is unchanged.
macro_rules! eval_params {
    ( $( $field:ident : $len:literal = $default:expr; )* ) => {
        #[derive(Clone)]
        pub struct EvalParams {
            $( pub $field: [i32; $len], )*
        }

        impl Default for EvalParams {
            fn default() -> Self {
                Self {
                    $( $field: $default, )*
                }
            }
        }

        /// (name, length) for every field — addresses the Phase 3.2/3.3
        /// tune-time loader/dumper and Texel tuner (not wired up yet).
        #[allow(dead_code)]
        pub const EVAL_PARAM_NAMES: &[(&str, usize)] = &[
            $( (stringify!($field), $len), )*
        ];

        impl EvalParams {
            #[allow(dead_code)]
            pub fn get(&self, name: &str, idx: usize) -> i32 {
                match name {
                    $( stringify!($field) => self.$field[idx], )*
                    _ => panic!("unknown eval param: {name}"),
                }
            }

            #[allow(dead_code)]
            pub fn set(&mut self, name: &str, idx: usize, value: i32) {
                match name {
                    $( stringify!($field) => self.$field[idx] = value, )*
                    _ => panic!("unknown eval param: {name}"),
                }
            }
        }

        // ---- Texel trace machinery (Phase 3.3) — `--features texel` only ----
        // EvalCounts mirrors EvalParams field-for-field but holds *net feature
        // counts* (white − black). The trace records, per scoring site, how
        // many times each weight entered the mg and/or eg accumulator, so the
        // raw tapered eval can be reconstructed as Σ count·weight and the Texel
        // tuner can differentiate the loss w.r.t. each weight analytically.
        #[cfg(feature = "texel")]
        #[derive(Clone)]
        pub struct EvalCounts {
            $( pub $field: [i32; $len], )*
        }

        #[cfg(feature = "texel")]
        impl Default for EvalCounts {
            fn default() -> Self {
                Self { $( $field: [0; $len], )* }
            }
        }

        #[cfg(feature = "texel")]
        #[derive(Clone, Default)]
        pub struct EvalTrace {
            pub mg: EvalCounts,
            pub eg: EvalCounts,
            pub phase: i32,
            /// Untraced (frozen, non-tunable) contributions to the pre-taper mg
            /// and eg accumulators — the mate-drive mop-up and the passer-king
            /// proximity `rel_rank` constant. They are excluded from the linear
            /// reconstruction (they end up in the tuner's per-position `rest`).
            pub frozen_mg: i32,
            pub frozen_eg: i32,
            /// White-POV raw tapered score of the *linear* part only (mg/eg with
            /// the frozen contributions removed). The reconstruction gate asserts
            /// `reconstruct(defaults) == raw`, validating every traced count.
            pub raw: i32,
        }

        #[cfg(feature = "texel")]
        impl EvalTrace {
            pub fn reset(&mut self) {
                *self = Self::default();
            }

            /// White-POV raw tapered eval reconstructed from the counts:
            /// `(Σcount_mg·w · phase + Σcount_eg·w · (24−phase)) / 24`. At the
            /// default weights this must equal the pre-scaling tapered score
            /// `evaluate()` computed (the reconstruction acceptance gate).
            pub fn reconstruct(&self, p: &EvalParams) -> i32 {
                let mut mg: i64 = 0;
                let mut eg: i64 = 0;
                $( for i in 0..$len {
                    mg += self.mg.$field[i] as i64 * p.$field[i] as i64;
                    eg += self.eg.$field[i] as i64 * p.$field[i] as i64;
                } )*
                ((mg * self.phase as i64 + eg * (TOTAL_PHASE as i64 - self.phase as i64))
                    / TOTAL_PHASE as i64) as i32
            }

            /// Per-flat-index tapered coefficient `(count_mg·phase +
            /// count_eg·(24−phase))/24` — the linear sensitivity of the raw
            /// eval to each weight, in `EVAL_PARAM_NAMES` flat order.
            pub fn flat_coeffs(&self) -> Vec<f64> {
                let ph = self.phase as f64;
                let mut out = Vec::with_capacity(EvalParams::FLAT_SIZE);
                $( for i in 0..$len {
                    out.push(
                        (self.mg.$field[i] as f64 * ph
                            + self.eg.$field[i] as f64 * (TOTAL_PHASE as f64 - ph))
                            / TOTAL_PHASE as f64,
                    );
                } )*
                out
            }
        }

        #[cfg(feature = "texel")]
        impl EvalParams {
            /// Total number of scalar weights across all fields.
            pub const FLAT_SIZE: usize = 0 $( + $len )*;

            /// Flatten the weights into a contiguous `f64` vector in
            /// `EVAL_PARAM_NAMES` order (the tuner's working representation).
            pub fn to_flat(&self) -> Vec<f64> {
                let mut out = Vec::with_capacity(Self::FLAT_SIZE);
                $( for i in 0..$len { out.push(self.$field[i] as f64); } )*
                out
            }

            /// Inverse of `to_flat` (rounds to nearest integer weight).
            pub fn set_from_flat(&mut self, w: &[f64]) {
                let mut k = 0usize;
                $( for i in 0..$len { self.$field[i] = w[k].round() as i32; k += 1; } )*
            }
        }
    };
}

eval_params! {
    mg_val: 6 = MG_VAL;
    eg_val: 6 = EG_VAL;
    pst_mg: 384 = build_default_pst(true);
    pst_eg: 384 = build_default_pst(false);
    // Passers & pawn structure (Phase 4.4 fitted). passed_*/connected per-rank
    // tables; passed bonuses stay monotonic (rank 1/8 pinned 0).
    passed_mg: 8 = [0, 0, 0, 0, 54, 135, 152, 0];
    passed_eg: 8 = [0, 0, 0, 39, 76, 98, 98, 0];
    passed_supported_mg: 1 = [0];
    passed_supported_eg_base: 1 = [0];
    passed_supported_eg_per_rank: 1 = [0];
    passed_freestop_mg_per_rank: 1 = [0];
    passed_freestop_eg_per_rank: 1 = [0];
    passed_safestop_eg_per_rank: 1 = [16];
    passed_candidate_mg: 1 = [2];
    passed_candidate_eg: 1 = [1];
    pawn_doubled_mg: 1 = [3];
    pawn_doubled_eg: 1 = [18];
    pawn_isolated_mg: 1 = [6];
    pawn_isolated_eg: 1 = [13];
    // Rank-scaled connected/phalanx (Phase 3.8), Phase 4.4 fitted; indexed by
    // the pawn's relative rank (0..7).
    pawn_connected_mg: 8 = [7, 7, 36, 17, 18, 61, 175, 7];
    pawn_connected_eg: 8 = [5, 5, 0, 0, 12, 22, 21, 5];
    pawn_backward_mg: 1 = [0];
    pawn_backward_eg: 1 = [18];
    // Pawn-structure / passer detail (Phase 3.8), Phase 4.4 fitted. pawn_lever
    // stayed frozen at 0 (feature-support: too sparse to fit reliably).
    pawn_lever_mg: 1 = [0];
    pawn_lever_eg: 1 = [0];
    pawn_doubled_isolated_mg: 1 = [0];
    pawn_doubled_isolated_eg: 1 = [5];
    blocked_passer_mg: 1 = [46];
    blocked_passer_eg: 1 = [0];
    ideal_blockader_mg: 1 = [23];
    ideal_blockader_eg: 1 = [0];
    // Minors & rooks (Phase 4.4 fitted). rook_7th and a few others fitted to 0 —
    // the data verdict that they add nothing atop mobility/threats/open-file.
    bishop_pair_mg: 1 = [25];
    bishop_pair_eg: 1 = [55];
    rook_open_mg: 1 = [42];
    rook_open_eg: 1 = [0];
    rook_semiopen_mg: 1 = [2];
    rook_semiopen_eg: 1 = [24];
    rook_7th_mg: 1 = [0];
    rook_7th_eg: 1 = [0];
    rook_behind_passer_mg: 1 = [0];
    rook_behind_passer_eg: 1 = [76];
    enemy_rook_behind_passer_mg: 1 = [13];
    enemy_rook_behind_passer_eg: 1 = [0];
    knight_outpost_mg: 1 = [55];
    knight_outpost_eg: 1 = [5];
    // Per-count mobility tables (Phase 3.7 structure; Phase 4.3 fitted). Each is
    // non-decreasing in the count (a trapped piece is worst); low entries can go
    // negative (e.g. a 0-mobility bishop). Fitted at 250 epochs — the clean point
    // where every holdout bucket still improves (a fuller fit overvalued rook
    // activity in drawish rook endings, regressing that bucket).
    mob_n_mg: 9 = [-12, -4, 7, 21, 33, 39, 49, 54, 54];
    mob_n_eg: 9 = [-20, 16, 21, 30, 51, 70, 71, 71, 71];
    mob_b_mg: 14 = [24, 27, 30, 34, 41, 46, 51, 58, 60, 63, 70, 70, 70, 70];
    mob_b_eg: 14 = [-36, -3, 43, 49, 64, 74, 78, 81, 86, 86, 86, 86, 86, 86];
    mob_r_mg: 15 = [12, 33, 46, 47, 52, 59, 61, 65, 70, 73, 82, 88, 88, 88, 88];
    mob_r_eg: 15 = [4, 41, 41, 50, 66, 77, 87, 99, 105, 105, 105, 107, 107, 107, 107];
    mob_q_mg: 28 = [-45, 18, 65, 65, 67, 68, 69, 70, 74, 74, 82, 85, 85, 87, 89, 97, 97, 97, 97, 101, 101, 101, 101, 101, 101, 101, 101, 101];
    mob_q_eg: 28 = [-13, 8, 25, 25, 33, 33, 95, 97, 97, 101, 101, 101, 108, 111, 111, 118, 118, 121, 121, 125, 125, 125, 125, 125, 125, 125, 125, 125];
    // Threats (Phase 3.6 structure; Phase 4.2 fitted). The base threat scalars
    // converged to a common (38, 25) — the per-victim `threat_by_*` tables below
    // now carry the attacker/victim-specific signal.
    threat_minor_mg: 1 = [60];
    threat_minor_eg: 1 = [44];
    threat_rook_mg: 1 = [60];
    threat_rook_eg: 1 = [44];
    threat_queen_mg: 1 = [60];
    threat_queen_eg: 1 = [44];
    // Threats package v2 (Phase 3.6), seeded 0; fitted in Phase 4.2. Per-victim
    // arrays indexed by `Piece as usize` (0=pawn..5=king). The refined hanging
    // term absorbed the old flat hanging penalty, which the joint fit drove to
    // ~0 (see hanging_* below).
    threat_by_minor_mg: 6 = [0, 50, 82, 83, 72, 0];
    threat_by_minor_eg: 6 = [2, 22, 0, 0, 0, 0];
    threat_by_rook_mg: 6 = [0, 38, 32, 4, 71, 0];
    threat_by_rook_eg: 6 = [13, 19, 24, 1, 18, 0];
    threat_hanging_refined_mg: 6 = [0, 16, 51, 33, 0, 0];
    threat_hanging_refined_eg: 6 = [53, 28, 14, 2, 0, 0];
    threat_safe_pawn_push_mg: 1 = [27];
    threat_safe_pawn_push_eg: 1 = [1];
    threat_weak_piece_mg: 1 = [46];
    threat_weak_piece_eg: 1 = [0];
    threat_restricted_mg: 1 = [11];
    threat_restricted_eg: 1 = [0];
    king_safety_unit_minor: 1 = [2];
    king_safety_unit_rook: 1 = [2];
    king_safety_unit_queen: 1 = [5];
    // King-danger conversion table (Phase 3.5; Phase 4.1 fitted). Lengthened
    // 16 -> 40 and the hard `.min(15)` cap removed. Phase 4.1 co-tuned this
    // table with the danger-index inputs below by re-evaluating the 2.19M set
    // (`--tune-kingsafety`): the tail rose well above the old 118 cap into the
    // danger² curve strong engines use, staying monotonic non-decreasing.
    king_safety_table: 40 = [29, 29, 60, 60, 83, 83, 121, 121, 172, 172, 216, 216, 217, 217, 217, 217, 217, 217, 217, 217, 217, 217, 217, 259, 259, 288, 288, 288, 288, 288, 288, 288, 288, 288, 288, 288, 288, 288, 288, 369];
    // King-danger inputs (Phase 3.5). Seeded 0 (danger == the old attacker-unit
    // sum); they select the danger bucket non-linearly, so they are invisible
    // to the linear Texel trace and were fitted in Phase 4.1 by re-evaluation
    // (`--tune-kingsafety`). ks_weak_ring / ks_flank_attack stayed at 0 in the
    // fit.
    ks_weak_ring: 1 = [0];
    ks_safe_check_knight: 1 = [2];
    ks_safe_check_bishop: 1 = [4];
    ks_safe_check_rook: 1 = [4];
    ks_safe_check_queen: 1 = [16];
    ks_queen_relief: 1 = [2];
    ks_flank_attack: 1 = [0];
    ks_pawnless_flank: 1 = [12];
    shelter_missing_file_mg: 1 = [0];
    shelter_missing_adjacent_mg: 1 = [0];
    shelter_dist1_mg: 1 = [23];
    shelter_dist2_mg: 1 = [13];
    storm_file_weight: 1 = [0];
    storm_adjacent_weight: 1 = [0];
    // Old flat hanging penalty (Phase 3.6). Phase 4.2 dropped it data-driven:
    // the refined hanging term (`threat_hanging_refined`) generalises and fully
    // absorbed it, so the joint fit drove these to ~0. Kept (not deleted) so the
    // term stays available; the values are now near-inert.
    hanging_minor: 1 = [0];
    hanging_rook: 1 = [1];
    hanging_queen: 1 = [1];
    passer_proximity_base: 1 = [9];
    space_weight: 1 = [0];
    tempo: 1 = [40];
    // trapped_bishop frozen at hand value (feature-support: too sparse to fit).
    trapped_bishop_mg: 1 = [60];
    trapped_bishop_eg: 1 = [40];
    // Material imbalance (Phase 3.9), SF-style symmetric quadratic form, all
    // coefficients seeded 0 (bench unchanged). Two 6x6 matrices indexed
    // `pt1*6 + pt2` over the imbalance "piece" order
    // [bishop_pair, pawn, knight, bishop, rook, queen]; only the lower triangle
    // (pt2 <= pt1) is used. `imbalance_ours[pt1][pt2]` weights our-pt1 × our-pt2
    // count products; `imbalance_theirs[pt1][pt2]` weights our-pt1 × their-pt2.
    // Phase-independent (added equally to mg and eg). No SF `/16` divisor — the
    // coefficients are direct per-count-product weights so the term is exactly
    // linear and Texel-tunable; the scale is the tuner's to find (Phase 4.5).
    // Phase 4.5 fitted (lower triangle; upper entries never fire). Rows/cols in
    // the imbalance "piece" order [bishop_pair, pawn, knight, bishop, rook, queen].
    imbalance_ours: 36 = [25, 0, 0, 0, 0, 0, 2, 8, 0, 0, 0, 0, -11, 42, -19, 0, 0, 0, 26, 41, -24, -39, 0, 0, -4, 50, -48, -43, -49, 0, 6, 92, -95, -61, -127, -101];
    imbalance_theirs: 36 = [0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, -6, 40, 0, 0, 0, 0, 6, 45, -15, 0, 0, 0, -14, 63, 17, 10, 0, 0, 11, 114, -3, 30, -3, 0];
    // Small positional terms (Phase 3.10), all seeded 0 (bench unchanged),
    // tuned in Phase 4.4/4.5.
    // Small positional terms (Phase 3.10), Phase 4.4 fitted. rook_trapped frozen
    // (feature-support: too sparse).
    bishop_pair_pawn_mg: 1 = [-2];
    bishop_pair_pawn_eg: 1 = [-4];
    bishop_outpost_mg: 1 = [47];
    bishop_outpost_eg: 1 = [0];
    rook_trapped_mg: 1 = [0];
    rook_trapped_eg: 1 = [0];
    rook_connected_mg: 1 = [10];
    rook_connected_eg: 1 = [38];
    bishop_long_diagonal_mg: 1 = [24];
    bishop_long_diagonal_eg: 1 = [2];
    bad_bishop_mg: 1 = [0];
    bad_bishop_eg: 1 = [15];
    initiative_weight: 1 = [2];
    // Closedness (rammed-pawn count) value swing: per own-piece-count, added
    // for knights (expected positive when tuned) and rooks (expected
    // negative). mg-only — see eval_closedness for the caveat that the
    // marginal lever beyond 3.7's per-count mobility is the material-value
    // swing alone, so this is deliberately kept as a single small weight.
    closedness_knight_mg: 1 = [12];
    closedness_rook_mg: 1 = [-10];
    // Central-king / lost-castling danger: fires only when the king is still
    // on its home square, on a central file, with all castling rights for
    // that side gone.
    king_centrality_danger_mg: 1 = [65];
    // Gauntlet-driven additions (Phase 3.12), Phase 4.4 fitted. king_protector /
    // space_piece fitted to 0 (no marginal value atop the rest).
    unstoppable_passer_eg: 1 = [52];
    minor_behind_pawn_mg: 1 = [13];
    minor_behind_pawn_eg: 1 = [0];
    pawn_islands_mg: 1 = [6];
    pawn_islands_eg: 1 = [0];
    queen_infiltration_mg: 1 = [47];
    queen_infiltration_eg: 1 = [73];
    king_protector_mg: 1 = [8];
    king_protector_eg: 1 = [4];
    space_piece_mg: 1 = [0];
}

// Texel trace recording (Phase 3.3). `tr_mg!`/`tr_eg!` add a net feature count
// to the current `Evaluator::trace` at every `mg += sign·W·n` / `eg += ...`
// site. Without `--features texel` they expand to nothing, so production builds
// compile to byte-identical code and `self.trace` is never referenced.
#[cfg(feature = "texel")]
macro_rules! tr_mg {
    ($self:ident, $field:ident, $idx:expr, $v:expr) => {
        $self.trace.borrow_mut().mg.$field[$idx] += ($v) as i32;
    };
}
#[cfg(not(feature = "texel"))]
macro_rules! tr_mg {
    ($($t:tt)*) => {};
}
#[cfg(feature = "texel")]
macro_rules! tr_eg {
    ($self:ident, $field:ident, $idx:expr, $v:expr) => {
        $self.trace.borrow_mut().eg.$field[$idx] += ($v) as i32;
    };
}
#[cfg(not(feature = "texel"))]
macro_rules! tr_eg {
    ($($t:tt)*) => {};
}

/// Tune-time loader/dumper (Phase 3.2) — `--features tune` only, so release
/// builds expose neither the env-var load path nor the `dumpeval` command.
/// File format: one `name index value` line per scalar, in `EVAL_PARAM_NAMES`
/// order — the same format `dump` writes, so a tuner's output file loads
/// straight back in. A line naming an unknown field is a hard error (catches
/// stale/typo'd tuner output instead of silently keeping a default); a file
/// that omits some fields is valid (those fields keep their default).
#[cfg(feature = "tune")]
impl EvalParams {
    pub fn load_from_str(text: &str) -> Self {
        let mut params = Self::default();
        for (line_no, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            let name = parts.next().unwrap_or_else(|| {
                panic!("RAROG_EVAL_FILE line {}: missing field name", line_no + 1)
            });
            let idx: usize = parts
                .next()
                .unwrap_or_else(|| panic!("RAROG_EVAL_FILE line {}: missing index", line_no + 1))
                .parse()
                .unwrap_or_else(|err| {
                    panic!("RAROG_EVAL_FILE line {}: bad index: {err}", line_no + 1)
                });
            let value: i32 = parts
                .next()
                .unwrap_or_else(|| panic!("RAROG_EVAL_FILE line {}: missing value", line_no + 1))
                .parse()
                .unwrap_or_else(|err| {
                    panic!("RAROG_EVAL_FILE line {}: bad value: {err}", line_no + 1)
                });
            params.set(name, idx, value);
        }
        params
    }

    pub fn load_from_env() -> Self {
        match std::env::var("RAROG_EVAL_FILE") {
            Ok(path) => {
                let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
                    panic!("RAROG_EVAL_FILE: failed to read '{path}': {err}")
                });
                Self::load_from_str(&text)
            }
            Err(_) => Self::default(),
        }
    }

    pub fn dump(&self) -> String {
        let mut out = String::new();
        for &(name, len) in EVAL_PARAM_NAMES {
            for idx in 0..len {
                out.push_str(&format!("{name} {idx} {}\n", self.get(name, idx)));
            }
        }
        out
    }
}

#[cfg(all(test, feature = "tune"))]
mod tune_tests {
    use super::EvalParams;

    #[test]
    fn dump_load_dump_round_trip_is_byte_identical() {
        let dumped = EvalParams::default().dump();
        let reloaded = EvalParams::load_from_str(&dumped);
        assert_eq!(dumped, reloaded.dump());
    }

    #[test]
    fn partial_file_keeps_defaults_for_omitted_fields() {
        let reloaded = EvalParams::load_from_str("tempo 0 99\n");
        let mut expected = EvalParams::default();
        expected.tempo[0] = 99;
        assert_eq!(reloaded.dump(), expected.dump());
    }

    #[test]
    #[should_panic(expected = "unknown eval param")]
    fn unknown_field_name_is_a_hard_error() {
        EvalParams::load_from_str("not_a_real_field 0 1\n");
    }
}

// Phase 3.3 reconstruction acceptance gate: for a large, diverse set of
// positions, the trace must reconstruct the raw tapered eval `evaluate()`
// computed *exactly* (integer-for-integer). A mismatch means a trace count is
// wrong; that must be fixed before any tuning run, since the tuner's gradients
// are built from these counts.
#[cfg(all(test, feature = "texel"))]
mod texel_tests {
    use super::{EvalParams, Evaluator};
    use crate::board::Board;

    // Deterministic xorshift so the test is reproducible.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    #[test]
    fn trace_reconstructs_eval_exactly_over_random_playouts() {
        let defaults = EvalParams::default();
        let mut evaluator = Evaluator::default();
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let mut checked = 0usize;

        for _ in 0..400 {
            let mut board = Board::starting_position();
            for _ in 0..40 {
                let _ = evaluator.evaluate(&board);
                let trace = evaluator.last_trace();
                let recon = trace.reconstruct(&defaults);
                assert_eq!(
                    recon,
                    trace.raw,
                    "trace reconstruction mismatch in {} (recon {recon} != raw {})",
                    board.to_fen(),
                    trace.raw,
                );
                checked += 1;

                let moves = board.generate_legal_movelist();
                if moves.is_empty() {
                    break;
                }
                let mv = moves.as_slice()[(rng.next() as usize) % moves.len()];
                board.make_move(mv);
            }
        }
        assert!(
            checked > 5000,
            "expected a broad sample, only checked {checked}"
        );
    }

    fn trace_of(ev: &mut Evaluator, fen: &str) -> super::EvalTrace {
        let board = Board::from_fen(fen).unwrap_or_else(|e| panic!("bad FEN {fen}: {e}"));
        let _ = ev.evaluate(&board);
        ev.last_trace()
    }

    /// Phase 3 gate — **nonzero activation**: every new term must actually fire
    /// on a position designed to trigger it, otherwise the Phase-4 feature-support
    /// diagnostics would tune a dead term blind.
    #[test]
    fn new_terms_activate_on_curated_positions() {
        let mut ev = Evaluator::default();

        // Passers (3.8) + free-stop / safe-stop (3.14): a clear, unattacked passer.
        let t = trace_of(&mut ev, "4k3/8/8/3P4/8/8/8/4K3 w - - 0 1");
        assert!(t.eg.passed_eg.iter().any(|&c| c != 0), "passed_eg dead");
        assert_ne!(t.eg.passed_freestop_eg_per_rank[0], 0, "free-stop dead");
        assert_ne!(t.eg.passed_safestop_eg_per_rank[0], 0, "safe-stop dead");

        // Per-count knight mobility one-hot (3.7).
        let t = trace_of(&mut ev, "4k3/8/8/8/4N3/8/8/4K3 w - - 0 1");
        assert!(t.mg.mob_n_mg.iter().any(|&c| c != 0), "mob_n_mg dead");

        // Material imbalance (3.9): 3 white pawns vs none → nonzero net products.
        let t = trace_of(&mut ev, "4k3/8/8/8/8/8/PPP5/4K3 w - - 0 1");
        assert!(
            t.mg.imbalance_ours.iter().any(|&c| c != 0),
            "imbalance dead"
        );

        // Threat-by-minor (3.6): white knight e4 attacks the black rook on d6.
        let t = trace_of(&mut ev, "4k3/8/3r4/8/4N3/8/8/4K3 w - - 0 1");
        assert!(
            t.mg.threat_by_minor_mg.iter().any(|&c| c != 0),
            "threat_by_minor dead"
        );

        // Minor-behind-pawn (3.12): white knight d3 shielded by the pawn on d4.
        let t = trace_of(&mut ev, "4k3/8/8/8/3P4/3N4/8/4K3 w - - 0 1");
        assert_ne!(t.mg.minor_behind_pawn_mg[0], 0, "minor_behind_pawn dead");
    }
}

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
const SQUARE_FILE: [usize; 64] = init_square_file();
const SQUARE_RANK: [usize; 64] = init_square_rank();
const RELATIVE_RANKS: [[u8; 64]; 2] = init_relative_ranks();
const KING_DISTANCE: [[u8; 64]; 64] = init_king_distance();
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
// KBNK mate (Phase 3.11): the bare king is driven to a corner the winning
// bishop can actually reach — i.e. a corner of the bishop's own square colour.
// A bishop can only reach corners that share its colour, so these sets are
// exactly the corner squares contained in `Bitboard::LIGHT_SQUARES` /
// `DARK_SQUARES`. NB this engine's colour convention puts a1 in LIGHT_SQUARES
// (see bitboard.rs), so the "light" corners are a1(0) and h8(63).
const KBNK_LIGHT_CORNERS: [usize; 2] = [0, 63]; // a1, h8 — on LIGHT_SQUARES
const KBNK_DARK_CORNERS: [usize; 2] = [7, 56]; // h1, a8 — on DARK_SQUARES
/// Endgame scale-factor framework (Phase 3.11). A scale of `SCALE_NORMAL`
/// leaves the tapered endgame score untouched; specialised functions return a
/// smaller scale (down to 0 = dead draw) for known material patterns. Kept as
/// a `/64` basis (SF convention) for the new patterns; the pre-existing OCB
/// scaling retains its own exact `/48` arithmetic so the bench fingerprint is
/// unchanged.
const SCALE_NORMAL: i32 = 64;
/// Opposite-coloured-bishop scaling keeps its own `/48` basis, preserved
/// verbatim from the pre-3.11 code so passer-free OCB positions are unchanged
/// (the bench fingerprint is unaffected).
const OCB_SCALE_NORMAL: i32 = 48;

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

/// Material + PST combined per (color, piece, square), rebuilt from
/// `EvalParams` whenever params change (Phase 3.1 — these used to be
/// `const`-baked `MG_TABLE`/`EG_TABLE`; now `params.mg_val`/`params.pst_mg`
/// are tunable data, so the table must be a runtime-built `Evaluator` field).
#[derive(Clone)]
pub struct EvalTables {
    mg: [[[i32; 64]; 6]; 2],
    eg: [[[i32; 64]; 6]; 2],
}

fn build_tables(params: &EvalParams) -> EvalTables {
    let mut mg = [[[0i32; 64]; 6]; 2];
    let mut eg = [[[0i32; 64]; 6]; 2];
    for piece in 0..6 {
        for sq in 0..64 {
            mg[Color::White as usize][piece][sq] =
                params.mg_val[piece] + params.pst_mg[piece * 64 + sq];
            mg[Color::Black as usize][piece][sq] =
                params.mg_val[piece] + params.pst_mg[piece * 64 + (sq ^ 56)];
            eg[Color::White as usize][piece][sq] =
                params.eg_val[piece] + params.pst_eg[piece * 64 + sq];
            eg[Color::Black as usize][piece][sq] =
                params.eg_val[piece] + params.pst_eg[piece * 64 + (sq ^ 56)];
        }
    }
    EvalTables { mg, eg }
}

// Under `texel` the caches are written but never read (hits are bypassed so
// every position re-emits its trace), so the fields look dead to that build.
#[cfg_attr(feature = "texel", allow(dead_code))]
#[derive(Copy, Clone, Default)]
struct PawnEntry {
    key: u64,
    mg: i32,
    eg: i32,
    passed: [Bitboard; 2],
    attacks: [Bitboard; 2],
}

#[cfg_attr(feature = "texel", allow(dead_code))]
#[derive(Copy, Clone, Default)]
struct EvalEntry {
    key: u64,
    halfmove_clock: u8,
    value: i32,
    occupied: bool,
}

/// Attack-map slices the king-danger model (Phase 3.5) reads, bundled to keep
/// `eval_king_safety`'s signature small. All come from the 3.0 substrate.
struct KsMaps<'a> {
    /// `attacks_from_sq` for the *attacking* side (the enemy of the king).
    their_from_sq: &'a [Bitboard; 64],
    /// Union of squares attacked by each colour (incl. pawns + king).
    attacked: &'a [Bitboard; 2],
    /// Squares attacked ≥2 times by each colour.
    attacked2: &'a [Bitboard; 2],
    /// Per-piece-type attack union for the attacking side.
    attacked_by_them: &'a [Bitboard; 6],
    occupied: Bitboard,
    own_occ: Bitboard,
    their_occ: Bitboard,
}

#[derive(Clone)]
pub struct Evaluator {
    pawn_table: Vec<PawnEntry>,
    eval_table: Vec<EvalEntry>,
    params: EvalParams,
    tables: Box<EvalTables>,
    /// Per-call feature trace, recorded only under `--features texel`. Held in
    /// a `RefCell` so the `&self` eval helpers can append to it; the field does
    /// not exist in production builds.
    #[cfg(feature = "texel")]
    trace: RefCell<EvalTrace>,
}

impl Default for Evaluator {
    fn default() -> Self {
        #[cfg(feature = "tune")]
        let params = EvalParams::load_from_env();
        #[cfg(not(feature = "tune"))]
        let params = EvalParams::default();
        let tables = Box::new(build_tables(&params));
        Self {
            pawn_table: vec![PawnEntry::default(); PAWN_TABLE_SIZE],
            eval_table: vec![EvalEntry::default(); EVAL_TABLE_SIZE],
            params,
            tables,
            #[cfg(feature = "texel")]
            trace: RefCell::new(EvalTrace::default()),
        }
    }
}

#[cfg(feature = "texel")]
impl Evaluator {
    /// Read-only view of the evaluation parameters (the tuner's defaults).
    pub fn params(&self) -> &EvalParams {
        &self.params
    }

    /// Swap in new parameters, rebuilding the derived tables so a changed
    /// material/PST/king-safety weight is fully reflected. Used by the
    /// nonlinear king-safety fit, which re-evaluates the dataset many times with
    /// perturbed danger-index weights (those weights select a table bucket
    /// nonlinearly, so the linear trace cannot see them). The whole-eval cache
    /// is never *read* under `texel` (hits are bypassed), so stale entries from
    /// a previous parameter set are harmless and need no clearing.
    pub fn set_params(&mut self, params: EvalParams) {
        self.tables = Box::new(build_tables(&params));
        self.params = params;
    }

    /// A clone of the trace captured by the most recent `evaluate()` call.
    pub fn last_trace(&self) -> EvalTrace {
        self.trace.borrow().clone()
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
        // The whole-eval cache must be bypassed under `texel`: a cache hit
        // returns without re-emitting trace counts, which would poison the
        // per-position trace the tuner records.
        let eval_slot = board.hash as usize & (EVAL_TABLE_SIZE - 1);
        #[cfg(not(feature = "texel"))]
        {
            let cached = self.eval_table[eval_slot];
            if cached.occupied
                && cached.key == board.hash
                && cached.halfmove_clock == board.halfmove_clock
            {
                return cached.value;
            }
        }
        #[cfg(feature = "texel")]
        self.trace.borrow_mut().reset();

        let atk = &*ATTACKS;
        let mut mg = 0;
        let mut eg = 0;
        let mut phase = 0;

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            for piece in Piece::ALL {
                let mut bb = board.pieces(color, piece);
                let phase_weight = PHASE_W[piece as usize];
                while bb.any() {
                    let sq = bb.pop_lsb();
                    phase += phase_weight;
                    mg += sign * self.tables.mg[color as usize][piece as usize][sq.index()];
                    eg += sign * self.tables.eg[color as usize][piece as usize][sq.index()];
                    // Material and PST are separate tunable params; the cooked
                    // table folds them, so trace each separately. The PST index
                    // mirrors build_tables: sq for white, sq^56 for black.
                    #[cfg_attr(not(feature = "texel"), allow(unused_variables))]
                    let pst_sq = if color == Color::White {
                        sq.index()
                    } else {
                        sq.index() ^ 56
                    };
                    tr_mg!(self, mg_val, piece as usize, sign);
                    tr_eg!(self, eg_val, piece as usize, sign);
                    tr_mg!(self, pst_mg, piece as usize * 64 + pst_sq, sign);
                    tr_eg!(self, pst_eg, piece as usize * 64 + pst_sq, sign);
                }
            }
        }
        phase = phase.min(TOTAL_PHASE);
        #[cfg(feature = "texel")]
        {
            self.trace.borrow_mut().phase = phase;
        }

        let mut passed = [Bitboard::EMPTY; 2];
        let mut pawn_attacks = [Bitboard::EMPTY; 2];
        let (pawn_mg, pawn_eg) = self.eval_pawns(board, atk, &mut passed, &mut pawn_attacks);
        mg += pawn_mg;
        eg += pawn_eg;
        // Passed-pawn free-stop / safe-stop bonuses (Phase 3.14): occupancy- and
        // attack-dependent, so they run every evaluation rather than living in
        // the pawn-structure cache. Applied here — immediately after `eval_pawns`
        // and before `eval_piece_activity` — so the running `mg`/`eg` totals seen
        // by downstream nonlinear terms (e.g. the mop-up's `(mg+eg)/2` test)
        // match the pre-3.14 ordering exactly; only the cache key changes.
        self.eval_passed_pawn_advance(board, &passed, &mut mg, &mut eg);

        // Lazy eval (Phase 3.16): if the cheap material + PST + pawn margin
        // already decides the position by more than any positional term could
        // flip, skip the expensive block (piece activity = mobility / threats /
        // king-safety / hanging / small-terms, plus imbalance). The mop-up still
        // runs, so mating technique (KBNK, KXK) survives a lazy skip. Disabled
        // under `--features texel` so the tuner traces and fits the *full* eval;
        // the eval stays a pure function of the position, so the eval cache and
        // `tests/eval_cache.rs` remain exact. `LAZY_MARGIN` is SPRT-tunable.
        #[cfg(not(feature = "texel"))]
        let lazy = ((mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE).abs() > LAZY_MARGIN;
        #[cfg(feature = "texel")]
        let lazy = false;

        if lazy {
            self.apply_mop_up(board, &mut mg, &mut eg);
        } else {
            self.eval_piece_activity(board, atk, &mut mg, &mut eg, &passed, &pawn_attacks, phase);
            // Mop-up keeps its pre-3.16 position (after activity, before
            // imbalance) so the full-eval path is byte-identical.
            self.apply_mop_up(board, &mut mg, &mut eg);
            self.eval_imbalance(board, &mut mg, &mut eg);
        }

        let tempo_sign = if board.side_to_move() == Color::White {
            1
        } else {
            -1
        };
        mg += tempo_sign * self.params.tempo[0];
        tr_mg!(self, tempo, 0, tempo_sign);

        let mut score = (mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
        #[cfg(feature = "texel")]
        {
            // Reconstruct the *linear* tapered score (frozen mop-up / passer
            // proximity constants removed) so it matches `reconstruct()` exactly.
            let (fmg, feg) = {
                let t = self.trace.borrow();
                (t.frozen_mg, t.frozen_eg)
            };
            let lin = ((mg - fmg) * phase + (eg - feg) * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
            self.trace.borrow_mut().raw = lin;
        }
        score = scale_endgame(board, score);
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

    fn eval_pawns(
        &mut self,
        board: &Board,
        atk: &AttackTables,
        passed: &mut [Bitboard; 2],
        attacks: &mut [Bitboard; 2],
    ) -> (i32, i32) {
        let key = board.pawn_key();
        let slot = key as usize & (PAWN_TABLE_SIZE - 1);
        // Bypass the pawn cache under `texel`: a hit skips the trace counts.
        #[cfg(not(feature = "texel"))]
        {
            let cached = self.pawn_table[slot];
            if cached.key == key {
                *passed = cached.passed;
                *attacks = cached.attacks;
                return (cached.mg, cached.eg);
            }
        }

        let mut mg = 0;
        let mut eg = 0;

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let us = color;
            let them = !us;
            let our_pawns = board.pieces(us, Piece::Pawn);
            let their_pawns = board.pieces(them, Piece::Pawn);
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
                    mg += sign * self.params.passed_mg[rel_rank];
                    eg += sign * self.params.passed_eg[rel_rank];
                    tr_mg!(self, passed_mg, rel_rank, sign);
                    tr_eg!(self, passed_eg, rel_rank, sign);

                    if (atk.pawn(them, sq) & our_pawns).any() {
                        mg += sign * self.params.passed_supported_mg[0];
                        eg += sign
                            * (self.params.passed_supported_eg_base[0]
                                + rel_rank as i32 * self.params.passed_supported_eg_per_rank[0]);
                        tr_mg!(self, passed_supported_mg, 0, sign);
                        tr_eg!(self, passed_supported_eg_base, 0, sign);
                        tr_eg!(
                            self,
                            passed_supported_eg_per_rank,
                            0,
                            sign * rel_rank as i32
                        );
                    }

                    // NB: the passed-pawn "free stop / safe stop" bonuses depend
                    // on non-pawn occupancy and enemy attacks, so they are NOT
                    // computed here — this function's result is cached by a
                    // pawn-structure-only key (Phase 3.14 fix). They are scored
                    // per-evaluation in `eval_passed_pawn_advance` instead.
                } else if rel_rank >= 3
                    && (atk.pawn(them, sq) & our_pawns).any()
                    && (their_pawns
                        & adjacent
                        & FORWARD_RANKS[us as usize][SQUARE_RANK[sq.index()]])
                    .is_empty()
                {
                    mg += sign * self.params.passed_candidate_mg[0];
                    eg += sign * self.params.passed_candidate_eg[0];
                    tr_mg!(self, passed_candidate_mg, 0, sign);
                    tr_eg!(self, passed_candidate_eg, 0, sign);
                }

                let file_bb = FILE_BBS[file];
                let is_doubled = (our_pawns & file_bb).more_than_one();
                let is_isolated = (our_pawns & adjacent).is_empty();
                if is_doubled {
                    mg -= sign * self.params.pawn_doubled_mg[0];
                    eg -= sign * self.params.pawn_doubled_eg[0];
                    tr_mg!(self, pawn_doubled_mg, 0, -sign);
                    tr_eg!(self, pawn_doubled_eg, 0, -sign);
                }
                if is_isolated {
                    mg -= sign * self.params.pawn_isolated_mg[0];
                    eg -= sign * self.params.pawn_isolated_eg[0];
                    tr_mg!(self, pawn_isolated_mg, 0, -sign);
                    tr_eg!(self, pawn_isolated_eg, 0, -sign);
                }
                // Doubled *and* isolated — an extra penalty on top (Phase 3.8,
                // seeded 0).
                if is_doubled && is_isolated {
                    mg -= sign * self.params.pawn_doubled_isolated_mg[0];
                    eg -= sign * self.params.pawn_doubled_isolated_eg[0];
                    tr_mg!(self, pawn_doubled_isolated_mg, 0, -sign);
                    tr_eg!(self, pawn_doubled_isolated_eg, 0, -sign);
                }
                // Connected (defended by an own pawn) — now rank-scaled.
                if (atk.pawn(them, sq) & our_pawns).any() {
                    mg += sign * self.params.pawn_connected_mg[rel_rank];
                    eg += sign * self.params.pawn_connected_eg[rel_rank];
                    tr_mg!(self, pawn_connected_mg, rel_rank, sign);
                    tr_eg!(self, pawn_connected_eg, rel_rank, sign);
                }
                // Pawn lever: our pawn that attacks an enemy pawn (Phase 3.8,
                // seeded 0).
                if (atk.pawn(us, sq) & their_pawns).any() {
                    mg += sign * self.params.pawn_lever_mg[0];
                    eg += sign * self.params.pawn_lever_eg[0];
                    tr_mg!(self, pawn_lever_mg, 0, sign);
                    tr_eg!(self, pawn_lever_eg, 0, sign);
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
                    mg -= sign * self.params.pawn_backward_mg[0];
                    eg -= sign * self.params.pawn_backward_eg[0];
                    tr_mg!(self, pawn_backward_mg, 0, -sign);
                    tr_eg!(self, pawn_backward_eg, 0, -sign);
                }
            }

            // Pawn islands (Phase 3.12, seeded 0): number of maximal groups of
            // own pawns on consecutive files. Penalty grows with fragmentation.
            let mut file_mask = 0u16;
            for f in 0..8 {
                if (our_pawns & FILE_BBS[f]).any() {
                    file_mask |= 1 << f;
                }
            }
            let islands = (file_mask & !(file_mask << 1)).count_ones() as i32;
            if islands != 0 {
                mg -= sign * islands * self.params.pawn_islands_mg[0];
                eg -= sign * islands * self.params.pawn_islands_eg[0];
                tr_mg!(self, pawn_islands_mg, 0, -sign * islands);
                tr_eg!(self, pawn_islands_eg, 0, -sign * islands);
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

    /// Passed-pawn advance bonuses that depend on non-pawn occupancy: a clear
    /// stop square ("free stop") and an unattacked stop square ("safe stop").
    /// These are deliberately kept out of `eval_pawns` — whose result is cached
    /// by a pawn-structure-only key — so the whole evaluation stays a pure
    /// function of the position and the eval cache is exact (Phase 3.14).
    fn eval_passed_pawn_advance(
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
                    if board.attackers_to_color(stop, occupied, them).is_empty() {
                        *eg += sign * rel_rank * self.params.passed_safestop_eg_per_rank[0];
                        tr_eg!(self, passed_safestop_eg_per_rank, 0, sign * rel_rank);
                    }
                }
            }
        }
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

        // Attack-map substrate (Phase 3.0): compute every piece's attack
        // bitboard once per evaluate() call, then reuse it for mobility,
        // king safety, and hanging-piece detection below instead of
        // recomputing attacks_for()/attackers_to_color() per consumer.
        // attacked[color] is the union over all of color's pieces (incl.
        // pawns and king) of squares they attack with the current
        // occupancy — equivalent to attackers_to_color(sq, occupied, color)
        // being non-empty for any sq, by the same symmetric attack-table
        // argument attackers_to_color itself relies on.
        let mut attacks_from_sq = [[Bitboard::EMPTY; 64]; 2];
        let mut attacked_by = [[Bitboard::EMPTY; 6]; 2];
        let mut attacked = [Bitboard::EMPTY; 2];
        let mut attacked2 = [Bitboard::EMPTY; 2];
        for color in [Color::White, Color::Black] {
            let ci = color as usize;
            attacked_by[ci][Piece::Pawn as usize] = pawn_attacks[ci];
            attacked[ci] |= pawn_attacks[ci];

            let king_atk = atk.king(board.king_sq(color));
            attacked_by[ci][Piece::King as usize] = king_atk;
            attacked2[ci] |= attacked[ci] & king_atk;
            attacked[ci] |= king_atk;

            for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
                let mut bb = board.pieces(color, piece);
                while bb.any() {
                    let sq = bb.pop_lsb();
                    let atks = attacks_for(atk, piece, sq, occupied);
                    attacks_from_sq[ci][sq.index()] = atks;
                    attacked_by[ci][piece as usize] |= atks;
                    attacked2[ci] |= attacked[ci] & atks;
                    attacked[ci] |= atks;
                }
            }
        }

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
                let pawn_term = 8 - (own_pawns | their_pawns).count() as i32;
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
                    && (attacks_from_sq[color as usize][sq.index()] & king_zone).any()
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
                let bad_count = (own_pawns & bishop_squares).count() as i32;
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
            let own_lost_castling = !board.castling.has(own_castling_all);
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
                    let mobility = (atk.rook(sq, occupied) & !own_occ).count() as i32;
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

            let safe = !pawn_attacks[them as usize];
            for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
                let mut pieces = board.pieces(color, piece);
                while pieces.any() {
                    let sq = pieces.pop_lsb();
                    let attacks = attacks_from_sq[color as usize][sq.index()];
                    let mobility = (attacks & safe & !own_occ).count() as usize;
                    // One-hot per-count tables (Phase 3.7). Index clamped to the
                    // table length for safety; the count never exceeds it in
                    // practice (N≤8, B≤13, R≤14, Q≤27).
                    macro_rules! mob_term {
                        ($mgf:ident, $egf:ident) => {{
                            let i = mobility.min(self.params.$mgf.len() - 1);
                            *mg += sign * self.params.$mgf[i];
                            *eg += sign * self.params.$egf[i];
                            tr_mg!(self, $mgf, i, sign);
                            tr_eg!(self, $egf, i, sign);
                        }};
                    }
                    match piece {
                        Piece::Knight => mob_term!(mob_n_mg, mob_n_eg),
                        Piece::Bishop => mob_term!(mob_b_mg, mob_b_eg),
                        Piece::Rook => mob_term!(mob_r_mg, mob_r_eg),
                        Piece::Queen => mob_term!(mob_q_mg, mob_q_eg),
                        _ => {}
                    }
                }
            }

            let mut threats = pawn_attacks[color as usize] & board.color_occ(them);
            while threats.any() {
                let sq = threats.pop_lsb();
                match board.piece_on(sq) {
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
            let our_minor_att =
                attacked_by[ci][Piece::Knight as usize] | attacked_by[ci][Piece::Bishop as usize];
            let mut tb = our_minor_att & enemy_occ;
            while tb.any() {
                let sq = tb.pop_lsb();
                if let Some(v) = board.piece_on(sq) {
                    *mg += sign * self.params.threat_by_minor_mg[v as usize];
                    *eg += sign * self.params.threat_by_minor_eg[v as usize];
                    tr_mg!(self, threat_by_minor_mg, v as usize, sign);
                    tr_eg!(self, threat_by_minor_eg, v as usize, sign);
                }
            }
            let mut tb = attacked_by[ci][Piece::Rook as usize] & enemy_occ;
            while tb.any() {
                let sq = tb.pop_lsb();
                if let Some(v) = board.piece_on(sq) {
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
                let att1 = (attacked[ci] & bb).any();
                let att2 = (attacked2[ci] & bb).any();
                let def1 = (attacked[ti] & bb).any();
                let def2 = (attacked2[ti] & bb).any();
                if ((att1 && !def1) || (att2 && def1 && !def2))
                    && let Some(v) = board.piece_on(sq)
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
            let push_targets = (push_attacks & enemy_occ & !their_pawns).count() as i32;
            if push_targets != 0 {
                *mg += sign * push_targets * self.params.threat_safe_pawn_push_mg[0];
                *eg += sign * push_targets * self.params.threat_safe_pawn_push_eg[0];
                tr_mg!(self, threat_safe_pawn_push_mg, 0, sign * push_targets);
                tr_eg!(self, threat_safe_pawn_push_eg, 0, sign * push_targets);
            }

            // Weak piece: our piece attacked by a strictly lower-valued enemy
            // piece (penalty for us).
            let their_minor_att =
                attacked_by[ti][Piece::Knight as usize] | attacked_by[ti][Piece::Bishop as usize];
            let weak_minor = (board.pieces(color, Piece::Knight)
                | board.pieces(color, Piece::Bishop))
                & pawn_attacks[ti];
            let weak_rook = board.pieces(color, Piece::Rook) & (pawn_attacks[ti] | their_minor_att);
            let weak_queen = board.pieces(color, Piece::Queen)
                & (pawn_attacks[ti] | their_minor_att | attacked_by[ti][Piece::Rook as usize]);
            let weak_cnt = (weak_minor | weak_rook | weak_queen).count() as i32;
            if weak_cnt != 0 {
                *mg -= sign * weak_cnt * self.params.threat_weak_piece_mg[0];
                *eg -= sign * weak_cnt * self.params.threat_weak_piece_eg[0];
                tr_mg!(self, threat_weak_piece_mg, 0, -sign * weak_cnt);
                tr_eg!(self, threat_weak_piece_eg, 0, -sign * weak_cnt);
            }

            // Restricted squares: squares both sides attack that the enemy does
            // not strongly protect (no enemy pawn attack, not doubly attacked).
            let strongly_protected = pawn_attacks[ti] | attacked2[ti];
            let restricted = (attacked[ti] & attacked[ci] & !strongly_protected).count() as i32;
            if restricted != 0 {
                *mg += sign * restricted * self.params.threat_restricted_mg[0];
                *eg += sign * restricted * self.params.threat_restricted_eg[0];
                tr_mg!(self, threat_restricted_mg, 0, sign * restricted);
                tr_eg!(self, threat_restricted_eg, 0, sign * restricted);
            }

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
            let behind = (minors & shield).count() as i32;
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

            // Unstoppable passer (rule of the square): a passed pawn with a clear
            // path whose promotion the enemy king cannot reach in time. eg-only.
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
                let promo_sq = Square(promo as u8);
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

            let ks_maps = KsMaps {
                their_from_sq: &attacks_from_sq[them as usize],
                attacked: &attacked,
                attacked2: &attacked2,
                attacked_by_them: &attacked_by[them as usize],
                occupied,
                own_occ: color_occ[color as usize],
                their_occ: color_occ[them as usize],
            };
            self.eval_king_safety(board, color, sign, mg, &pawns, &ks_maps);
            self.eval_rooks_behind_passers(board, color, sign, passed, mg, eg);
            self.eval_passer_blockade(board, color, sign, passed, mg, eg);
            self.eval_hanging_pieces(board, color, sign, mg, eg, &attacked);
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

    /// Mate-drive "mop-up": when one side is clearly winning, nudge the losing
    /// king toward the edge/corner (the bishop's corner for KBNK). Extracted
    /// from `eval_piece_activity` so it also runs on the lazy-eval early-return
    /// path (Phase 3.16) — mating technique must survive a lazy skip. Frozen
    /// (non-tunable) term, so it lands in the tuner's `rest`.
    fn apply_mop_up(&self, board: &Board, mg: &mut i32, eg: &mut i32) {
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
            let king_distance = KING_DISTANCE[wksq.index()][lksq.index()] as i32;
            // KBNK (Phase 3.11): the generic corner-drive cannot win K+B+N vs K
            // because it pushes the bare king to the nearest corner, not the
            // bishop-coloured one. For that exact material pattern, drive the
            // losing king to a corner matching the winning bishop's colour
            // instead; keep the generic drive for every other won ending.
            let mopup = if let Some(light_bishop) = kbnk_winner_bishop(board, winning) {
                let corners = if light_bishop {
                    KBNK_LIGHT_CORNERS
                } else {
                    KBNK_DARK_CORNERS
                };
                let corner_distance = (KING_DISTANCE[lksq.index()][corners[0]]
                    .min(KING_DISTANCE[lksq.index()][corners[1]]))
                    as i32;
                sign * (8 * (7 - corner_distance) + (14 - king_distance) * 4)
            } else {
                let lfile = SQUARE_FILE[lksq.index()] as i32;
                let lrank = SQUARE_RANK[lksq.index()] as i32;
                let file_push = (3 - lfile).max(lfile - 4);
                let rank_push = (3 - lrank).max(lrank - 4);
                sign * (5 * (file_push + rank_push) + (14 - king_distance) * 4)
            };
            *eg += mopup;
            // Frozen mate-drive term — not a tunable weight; goes into `rest`.
            #[cfg(feature = "texel")]
            {
                self.trace.borrow_mut().frozen_eg += mopup;
            }
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
        let space_net = white_space.count() as i32 - black_space.count() as i32;
        *mg += space_net * self.params.space_weight[0];
        tr_mg!(self, space_weight, 0, space_net);

        // Space weighted by piece count (Phase 3.12, SF-style shape, seeded 0):
        // space matters more when more pieces remain to exploit it. The flat
        // `space_weight` term above stays active; Phase 4 retires it if this
        // shaped term earns its fit.
        let piece_count = |c: Color| -> i32 {
            (board.pieces(c, Piece::Knight)
                | board.pieces(c, Piece::Bishop)
                | board.pieces(c, Piece::Rook)
                | board.pieces(c, Piece::Queen))
            .count() as i32
        };
        let space_weighted = white_space.count() as i32 * piece_count(Color::White)
            - black_space.count() as i32 * piece_count(Color::Black);
        *mg += space_weighted * self.params.space_piece_mg[0];
        tr_mg!(self, space_piece_mg, 0, space_weighted);
    }

    fn eval_king_safety(
        &self,
        board: &Board,
        color: Color,
        sign: i32,
        mg: &mut i32,
        pawns: &[Bitboard; 2],
        maps: &KsMaps,
    ) {
        let them = !color;
        let king = board.king_sq(color);
        let king_bb = Bitboard::from(king);
        let king_attacks = ATTACKS.king(king);
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
            let mut pieces = board.pieces(them, piece);
            while pieces.any() {
                let sq = pieces.pop_lsb();
                if (maps.their_from_sq[sq.index()] & zone).any() {
                    danger += match piece {
                        Piece::Knight | Piece::Bishop => self.params.king_safety_unit_minor[0],
                        Piece::Rook => self.params.king_safety_unit_rook[0],
                        Piece::Queen => self.params.king_safety_unit_queen[0],
                        _ => 0,
                    };
                }
            }
        }

        // Weak king-ring squares: zone squares the enemy attacks but we do not
        // defend (or defend only once while doubly attacked).
        let weak = zone
            & maps.attacked[them as usize]
            & (!maps.attacked[color as usize] | maps.attacked2[them as usize]);
        danger += self.params.ks_weak_ring[0] * weak.count() as i32;

        // Safe checks: squares from which an enemy piece type could check our
        // king, that the enemy actually attacks with that type and we do not
        // defend (and are not occupied by an enemy piece).
        let occ = maps.occupied;
        let safe = !maps.attacked[color as usize] & !maps.their_occ;
        let knight_from = ATTACKS.knight(king);
        let bishop_from = ATTACKS.bishop(king, occ);
        let rook_from = ATTACKS.rook(king, occ);
        let knight_checks = knight_from & maps.attacked_by_them[Piece::Knight as usize] & safe;
        let bishop_checks = bishop_from & maps.attacked_by_them[Piece::Bishop as usize] & safe;
        let rook_checks = rook_from & maps.attacked_by_them[Piece::Rook as usize] & safe;
        let queen_checks =
            (bishop_from | rook_from) & maps.attacked_by_them[Piece::Queen as usize] & safe;
        danger += self.params.ks_safe_check_knight[0] * knight_checks.count() as i32;
        danger += self.params.ks_safe_check_bishop[0] * bishop_checks.count() as i32;
        danger += self.params.ks_safe_check_rook[0] * rook_checks.count() as i32;
        danger += self.params.ks_safe_check_queen[0] * queen_checks.count() as i32;

        // King-flank pressure: enemy attacks minus our defenses over the three
        // files around the king (clamped non-negative).
        let king_file = SQUARE_FILE[king.index()] as i32;
        let mut flank = Bitboard::EMPTY;
        for df in -1..=1 {
            let f = king_file + df;
            if (0..8).contains(&f) {
                flank |= FILE_BBS[f as usize];
            }
        }
        let flank_attack = (maps.attacked[them as usize] & flank).count() as i32;
        let flank_defense = (maps.attacked[color as usize] & flank).count() as i32;
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

        // Non-linear table lookup: trace one-hot on the bucket actually read.
        let safety_idx = danger.clamp(0, self.params.king_safety_table.len() as i32 - 1) as usize;
        *mg -= sign * self.params.king_safety_table[safety_idx];
        tr_mg!(self, king_safety_table, safety_idx, -sign);

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
                        SQUARE_RANK[pawn_sq.index()] as i32 - king_rank
                    } else {
                        king_rank - SQUARE_RANK[pawn_sq.index()] as i32
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
                    *mg += sign * self.params.rook_behind_passer_mg[0];
                    *eg += sign * self.params.rook_behind_passer_eg[0];
                    tr_mg!(self, rook_behind_passer_mg, 0, sign);
                    tr_eg!(self, rook_behind_passer_eg, 0, sign);
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
                    *mg -= sign * self.params.enemy_rook_behind_passer_mg[0];
                    *eg -= sign * self.params.enemy_rook_behind_passer_eg[0];
                    tr_mg!(self, enemy_rook_behind_passer_mg, 0, -sign);
                    tr_eg!(self, enemy_rook_behind_passer_eg, 0, -sign);
                }
            }
        }
    }

    /// Passer/blockade detail needing piece squares (Phase 3.8; seeded 0):
    /// a penalty when our own passed pawn is blocked by an enemy piece on its
    /// stop square, and a bonus for our knight as the ideal blockader directly
    /// in front of an enemy passed pawn.
    fn eval_passer_blockade(
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

    fn eval_hanging_pieces(
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
            let Some(piece) = board.piece_on(sq) else {
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

    /// SF-style quadratic material imbalance (Phase 3.9). White-POV net value,
    /// added phase-independently to mg and eg. Imbalance "pieces" are indexed
    /// `[0]=bishop pair, [1]=pawn, [2]=knight, [3]=bishop, [4]=rook, [5]=queen`;
    /// the coefficient matrices use the lower triangle (`pt2 <= pt1`). All
    /// coefficients are seeded 0, so this contributes nothing today.
    fn eval_imbalance(&self, board: &Board, mg: &mut i32, eg: &mut i32) {
        let count = |c: Color| -> [i32; 6] {
            let bishops = board.pieces(c, Piece::Bishop).count() as i32;
            [
                (bishops >= 2) as i32,
                board.pieces(c, Piece::Pawn).count() as i32,
                board.pieces(c, Piece::Knight).count() as i32,
                bishops,
                board.pieces(c, Piece::Rook).count() as i32,
                board.pieces(c, Piece::Queen).count() as i32,
            ]
        };
        let cnt = [count(Color::White), count(Color::Black)];

        let mut imb = 0i32; // white − black
        for c in [Color::White, Color::Black] {
            let sign = color_sign(c);
            let us = cnt[c as usize];
            let them = cnt[!c as usize];
            for pt1 in 0..6 {
                if us[pt1] == 0 {
                    continue;
                }
                for pt2 in 0..=pt1 {
                    let k = pt1 * 6 + pt2;
                    imb += sign
                        * us[pt1]
                        * (self.params.imbalance_ours[k] * us[pt2]
                            + self.params.imbalance_theirs[k] * them[pt2]);
                }
            }
        }
        *mg += imb;
        *eg += imb;

        // Trace each coefficient's net (white − black) count product, in both
        // mg and eg (phase-independent weight). `imbalance_ours[pt1][pt2]`
        // multiplies count[pt1]*count[pt2]; `imbalance_theirs[pt1][pt2]`
        // multiplies our[pt1]*their[pt2].
        #[cfg(feature = "texel")]
        {
            let w = &cnt[Color::White as usize];
            let b = &cnt[Color::Black as usize];
            for pt1 in 0..6 {
                for pt2 in 0..=pt1 {
                    let k = pt1 * 6 + pt2;
                    let ours = w[pt1] * w[pt2] - b[pt1] * b[pt2];
                    let theirs = w[pt1] * b[pt2] - b[pt1] * w[pt2];
                    tr_mg!(self, imbalance_ours, k, ours);
                    tr_eg!(self, imbalance_ours, k, ours);
                    tr_mg!(self, imbalance_theirs, k, theirs);
                    tr_eg!(self, imbalance_theirs, k, theirs);
                }
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
        let rammed = (wp.north() & bp).count() as i32;
        if rammed == 0 {
            return;
        }
        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let knights = board.pieces(color, Piece::Knight).count() as i32;
            let rooks = board.pieces(color, Piece::Rook).count() as i32;
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

    /// Central-king / lost-castling danger (Phase 3.10): a king still on its
    /// home square, on a central file, with all castling rights for that
    /// side gone — separate from the king-ring attack model (3.5), which
    /// scores nothing until attackers actually arrive.
    fn eval_king_centrality_danger(&self, board: &Board, mg: &mut i32) {
        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            let ksq = board.king_sq(color);
            let home_sq = Square([4u8, 60u8][color as usize]);
            let own_castling_all = match color {
                Color::White => CastlingRights::WHITE_ALL,
                Color::Black => CastlingRights::BLACK_ALL,
            };
            if ksq == home_sq && !board.castling.has(own_castling_all) {
                *mg -= sign * self.params.king_centrality_danger_mg[0];
                tr_mg!(self, king_centrality_danger_mg, 0, -sign);
            }
        }
    }

    /// Initiative / complexity (Phase 3.10): nudges the endgame score away
    /// from (or toward) a draw based on a cheap complexity proxy — total
    /// pawns, king-file separation, and pawns on both flanks — mirroring
    /// SF's `Initiative` adjustment. Seeded 0, so it is a no-op until tuned.
    fn eval_initiative(&self, board: &Board, eg: &mut i32) {
        let pawns =
            board.pieces(Color::White, Piece::Pawn) | board.pieces(Color::Black, Piece::Pawn);
        let pawn_count = pawns.count() as i32;
        let kf_w = SQUARE_FILE[board.king_sq(Color::White).index()] as i32;
        let kf_b = SQUARE_FILE[board.king_sq(Color::Black).index()] as i32;
        let king_file_distance = (kf_w - kf_b).abs();
        let queenside = FILE_BBS[0] | FILE_BBS[1] | FILE_BBS[2] | FILE_BBS[3];
        let kingside = FILE_BBS[4] | FILE_BBS[5] | FILE_BBS[6] | FILE_BBS[7];
        let both_flanks = (pawns & queenside).any() && (pawns & kingside).any();
        let complexity = pawn_count + king_file_distance + i32::from(both_flanks);
        let outcome_sign = (*eg > 0) as i32 - (*eg < 0) as i32;
        let n = outcome_sign * complexity;
        *eg += n * self.params.initiative_weight[0];
        tr_eg!(self, initiative_weight, 0, n);
    }
}

#[inline(always)]
pub fn piece_value(piece: Piece) -> i32 {
    unsafe { *PIECE_VALUES.get_unchecked(piece as usize) }
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
fn relative_rank(color: Color, sq: Square) -> u8 {
    RELATIVE_RANKS[color as usize][sq.index()]
}

fn forward_square(color: Color, sq: Square) -> Option<Square> {
    match color {
        Color::White => sq.0.checked_add(8).filter(|to| *to < 64).map(Square),
        Color::Black => sq.0.checked_sub(8).map(Square),
    }
}

/// Endgame scale-factor framework dispatch (Phase 3.11). Specialised material
/// patterns are checked first (they are absent from the bench suite, so the
/// fingerprint is unchanged); everything else falls through to the pre-existing
/// opposite-coloured-bishop scaling and the KNNK draw, whose exact integer
/// arithmetic is preserved verbatim.
fn scale_endgame(board: &Board, mut score: i32) -> i32 {
    if let Some(sf) = specialized_endgame_scale(board) {
        return score * sf / SCALE_NORMAL;
    }

    if let Some(scale) = opposite_bishop_scale(board) {
        score = score * scale / OCB_SCALE_NORMAL;
    }

    if has_only_king(board, Color::White) && has_only_knights(board, Color::Black, 2) {
        return 0;
    }
    if has_only_king(board, Color::Black) && has_only_knights(board, Color::White, 2) {
        return 0;
    }

    score
}

/// Specialised known-endgame scale factor (Phase 3.11), in `0..=SCALE_NORMAL`.
/// Returns `Some(sf)` for a recognised material pattern (the caller multiplies
/// the tapered score by `sf / SCALE_NORMAL`), or `None` to fall through to the
/// general scaling. Each pattern fires only on exact material absent from the
/// bench suite, so `bench 13` is unchanged.
///
/// Implemented: KPK bitbase, KBP wrong-corner draw, pawnless
/// insufficient-mating-material draws (KK/KNK/KBK/minor-vs-minor), and the
/// Phase-3.11c patterns — the KQKP fortress draw (rook/bishop pawn only) and a
/// conservative KRKP partial scale. KNNK keeps its existing handling in
/// `scale_endgame`. Deliberately omitted: KQ-vs-KR (a win — never scaled toward
/// draw) and broad rook-endgame drawishness (a tunable Phase-4 term, not a
/// hardcoded rule that could wrongly draw a won R+P vs R).
fn specialized_endgame_scale(board: &Board) -> Option<i32> {
    // KPK (king + single pawn vs lone king): exact bitbase verdict. A drawn KPK
    // is forced to 0; a won one falls through (`None`) so normal eval scores it.
    if let Some((white_to_move, wk, bk, p)) = kpk_normalized(board) {
        return if crate::kpk::probe(white_to_move, wk, bk, p) {
            None
        } else {
            Some(0)
        };
    }
    // KBP with a wrong-coloured bishop and a rook pawn, defender on the corner:
    // a textbook dead draw the bishop cannot break.
    if kbp_wrong_corner_draw(board) {
        return Some(0);
    }
    // KQ vs KP fortress (Phase 3.11c): only the textbook drawn case — a rook or
    // bishop pawn on the 7th, its king guarding the queening square, the queen's
    // king too far to break through. Knight/centre pawns are wins and are left
    // untouched.
    if let Some(sf) = kqkp_fortress_scale(board) {
        return Some(sf);
    }
    // KR vs KP (Phase 3.11c): conservative *partial* scale toward draw in the
    // clear draw zone (pawn on the 7th, escorted by its king, rook's king far).
    // Never a forced draw, so an actually-won KRKP keeps a clearly winning score.
    if let Some(sf) = krkp_drawish_scale(board) {
        return Some(sf);
    }

    let no_pawns = board.pieces(Color::White, Piece::Pawn).is_empty()
        && board.pieces(Color::Black, Piece::Pawn).is_empty();
    if !no_pawns {
        return None;
    }
    let majors = board.pieces(Color::White, Piece::Rook)
        | board.pieces(Color::Black, Piece::Rook)
        | board.pieces(Color::White, Piece::Queen)
        | board.pieces(Color::Black, Piece::Queen);
    if majors.any() {
        return None;
    }
    let white_minors = board.pieces(Color::White, Piece::Knight).count()
        + board.pieces(Color::White, Piece::Bishop).count();
    let black_minors = board.pieces(Color::Black, Piece::Knight).count()
        + board.pieces(Color::Black, Piece::Bishop).count();
    // KK / KNK / KBK (at most one minor on the board) and minor-vs-minor are
    // dead draws. K+B+N vs K (one side has two minors, the other none) is a
    // WIN and is deliberately *not* matched here — it falls through so the
    // KBNK corner-drive scores it.
    if white_minors + black_minors <= 1 || (white_minors == 1 && black_minors == 1) {
        return Some(0);
    }
    None
}

/// If `board` is exactly K+P vs K, return the KPK bitbase probe arguments
/// normalised so the pawn is White's (mirroring vertically when the pawn is
/// Black's). Returns `None` for any other material.
fn kpk_normalized(board: &Board) -> Option<(bool, usize, usize, usize)> {
    let wp = board.pieces(Color::White, Piece::Pawn);
    let bp = board.pieces(Color::Black, Piece::Pawn);
    if wp.count() + bp.count() != 1 {
        return None;
    }
    for color in [Color::White, Color::Black] {
        for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
            if board.pieces(color, piece).any() {
                return None;
            }
        }
    }
    let wk = board.king_sq(Color::White).index();
    let bk = board.king_sq(Color::Black).index();
    if wp.any() {
        let p = wp.lsb().index();
        Some((board.side_to_move() == Color::White, wk, bk, p))
    } else {
        // Black has the pawn: mirror vertically (square ^ 56) so the pawn is
        // White's, swapping the kings' roles and the side to move.
        let p = bp.lsb().index();
        Some((
            board.side_to_move() == Color::Black,
            bk ^ 56,
            wk ^ 56,
            p ^ 56,
        ))
    }
}

/// True for the wrong-bishop rook-pawn draw: the strong side has K + one bishop
/// + one or more pawns all on a single rook file, the bishop is the wrong colour
/// to control the queening square, and the bare defending king holds that corner
/// (within one square of it). Such positions are dead draws.
fn kbp_wrong_corner_draw(board: &Board) -> bool {
    for strong in [Color::White, Color::Black] {
        let weak = !strong;
        if board.color_occ(weak) != Bitboard::from(board.king_sq(weak)) {
            continue;
        }
        if board.pieces(strong, Piece::Knight).any()
            || board.pieces(strong, Piece::Rook).any()
            || board.pieces(strong, Piece::Queen).any()
        {
            continue;
        }
        let bishops = board.pieces(strong, Piece::Bishop);
        if bishops.count() != 1 {
            continue;
        }
        let pawns = board.pieces(strong, Piece::Pawn);
        if pawns.is_empty() {
            continue;
        }
        let on_a = (pawns & FILE_BBS[0]) == pawns;
        let on_h = (pawns & FILE_BBS[7]) == pawns;
        if !on_a && !on_h {
            continue;
        }
        let file = if on_a { 0 } else { 7 };
        let queening = if strong == Color::White {
            file + 56
        } else {
            file
        };
        let queening_sq = Square(queening as u8);
        // A bishop can only guard the queening square if it shares that square's
        // colour. Wrong colour → it can never evict the king from the corner.
        let bishop_light = (bishops & Bitboard::LIGHT_SQUARES).any();
        let queen_light = (Bitboard::from(queening_sq) & Bitboard::LIGHT_SQUARES).any();
        if bishop_light == queen_light {
            continue;
        }
        let weak_king = board.king_sq(weak);
        if KING_DISTANCE[weak_king.index()][queening_sq.index()] as i32 <= 1 {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Phase 3.11c — narrow, high-confidence endgame knowledge. Each fires only on
// an exact material pattern (verified absent from the bench suite, so the
// fingerprint is unchanged). Two deliberate omissions vs the original plan list:
//   * KQ-vs-KR is a *win*, so it is never scaled toward draw; and
//   * general rook-endgame drawishness is a tunable Phase-4 term, not a
//     hardcoded rule that could wrongly draw a won R+P vs R.
// ---------------------------------------------------------------------------

/// KQ vs KP fortress draw. The weak side has K+P with the pawn on the 7th, and
/// — crucially — it is a **rook or bishop pawn** (a/c/f/h), the only files where
/// KQ-vs-KP is a fortress draw. Its king must guard the queening square and the
/// queen's king be too far to break through. Knight and centre pawns are wins
/// and return `None`. Returns `Some(0)` (dead draw) for the recognised fortress.
fn kqkp_fortress_scale(board: &Board) -> Option<i32> {
    for strong in [Color::White, Color::Black] {
        let weak = !strong;
        if !has_exact_material(board, strong, 0, 0, 0, 0, 1)
            || !has_exact_material(board, weak, 1, 0, 0, 0, 0)
        {
            continue;
        }
        let pawn = board.pieces(weak, Piece::Pawn).lsb();
        if relative_rank(weak, pawn) != 6 || !is_rook_or_bishop_pawn(pawn) {
            continue;
        }
        let queening = promotion_square(weak, pawn);
        let weak_king = board.king_sq(weak);
        let strong_king = board.king_sq(strong);
        if KING_DISTANCE[weak_king.index()][queening.index()] <= 1
            && KING_DISTANCE[strong_king.index()][queening.index()] > 2
        {
            return Some(0);
        }
    }
    None
}

/// KR vs KP drawish heuristic. When the weak side's pawn is on the 7th escorted
/// by its king and the rook's king is far (> 4 — deliberately conservative), the
/// ending is usually drawn (the rook must give itself for the pawn). A
/// *partial* scale (≈¼) only — never a forced draw — so an actually-won KRKP
/// keeps a clearly winning score and a wrong guess cannot throw the game.
fn krkp_drawish_scale(board: &Board) -> Option<i32> {
    for strong in [Color::White, Color::Black] {
        let weak = !strong;
        if !has_exact_material(board, strong, 0, 0, 0, 1, 0)
            || !has_exact_material(board, weak, 1, 0, 0, 0, 0)
        {
            continue;
        }
        let pawn = board.pieces(weak, Piece::Pawn).lsb();
        if relative_rank(weak, pawn) != 6 {
            continue;
        }
        let queening = promotion_square(weak, pawn);
        let weak_king = board.king_sq(weak);
        let strong_king = board.king_sq(strong);
        if KING_DISTANCE[weak_king.index()][queening.index()] <= 1
            && KING_DISTANCE[strong_king.index()][queening.index()] > 4
        {
            return Some(16); // ×0.25 of SCALE_NORMAL
        }
    }
    None
}

/// Opposite-coloured-bishop scaling (single bishop each, opposite colours), on
/// the `/48` basis. Passed pawns make OCB endings less drawish, so the scale is
/// relaxed upward by them (Phase 3.11c refinement). Passer-free positions keep
/// the exact pre-3.11 value, so the bench fingerprint is unaffected.
fn opposite_bishop_scale(board: &Board) -> Option<i32> {
    let white_bishops = board.pieces(Color::White, Piece::Bishop);
    let black_bishops = board.pieces(Color::Black, Piece::Bishop);
    if white_bishops.is_empty()
        || white_bishops.more_than_one()
        || black_bishops.is_empty()
        || black_bishops.more_than_one()
    {
        return None;
    }
    let white_dark = (white_bishops & Bitboard::DARK_SQUARES).any();
    let black_dark = (black_bishops & Bitboard::DARK_SQUARES).any();
    if white_dark == black_dark {
        return None;
    }
    let pawns = (board.pieces(Color::White, Piece::Pawn) | board.pieces(Color::Black, Piece::Pawn))
        .count() as i32;
    let passers = count_passed_pawns(board);
    Some((32 + pawns * 4 + passers * 4).min(OCB_SCALE_NORMAL))
}

fn count_passed_pawns(board: &Board) -> i32 {
    let mut count = 0;
    for color in [Color::White, Color::Black] {
        let enemy_pawns = board.pieces(!color, Piece::Pawn);
        let mut pawns = board.pieces(color, Piece::Pawn);
        while pawns.any() {
            let sq = pawns.pop_lsb();
            if (PASSED_PAWN_MASKS[color as usize][sq.index()] & enemy_pawns).is_empty() {
                count += 1;
            }
        }
    }
    count
}

fn has_exact_material(
    board: &Board,
    color: Color,
    pawns: u32,
    knights: u32,
    bishops: u32,
    rooks: u32,
    queens: u32,
) -> bool {
    board.pieces(color, Piece::Pawn).count() == pawns
        && board.pieces(color, Piece::Knight).count() == knights
        && board.pieces(color, Piece::Bishop).count() == bishops
        && board.pieces(color, Piece::Rook).count() == rooks
        && board.pieces(color, Piece::Queen).count() == queens
}

fn promotion_square(color: Color, pawn: Square) -> Square {
    let file = SQUARE_FILE[pawn.index()];
    match color {
        Color::White => Square((file + 56) as u8),
        Color::Black => Square(file as u8),
    }
}

/// A rook (a/h) or bishop (c/f) pawn — the files on which KQ-vs-KP is a fortress
/// draw.
fn is_rook_or_bishop_pawn(pawn: Square) -> bool {
    matches!(SQUARE_FILE[pawn.index()], 0 | 2 | 5 | 7)
}

/// True iff `winner` holds exactly king + one bishop + one knight while the
/// loser has a bare king and neither side has pawns/rooks/queens (the KBNK
/// mate). Returns whether the winning bishop is light-squared.
fn kbnk_winner_bishop(board: &Board, winner: Color) -> Option<bool> {
    let loser = !winner;
    if board.color_occ(loser) != Bitboard::from(board.king_sq(loser)) {
        return None;
    }
    if board.pieces(winner, Piece::Pawn).any()
        || board.pieces(winner, Piece::Rook).any()
        || board.pieces(winner, Piece::Queen).any()
    {
        return None;
    }
    let bishops = board.pieces(winner, Piece::Bishop);
    let knights = board.pieces(winner, Piece::Knight);
    if bishops.count() == 1 && knights.count() == 1 {
        Some((bishops & Bitboard::LIGHT_SQUARES).any())
    } else {
        None
    }
}

/// The multiplicative factor `scale_drawish_endgames` + the rule-50 damping
/// apply to the raw tapered score, as an `f64` (Phase 3.3, texel only). The
/// tuner uses it to scale per-position weight *deltas* the same way the engine
/// scales the eval, so a small weight change predicts the right score change.
/// Mirrors `scale_drawish_endgames` and the `score -= score*rule50/199` line.
#[cfg(feature = "texel")]
pub fn linear_delta_scale(board: &Board) -> f64 {
    // Specialised endgame scale factors (Phase 3.11) apply first, mirroring
    // `scale_endgame`. A dead-draw pattern zeroes the delta scale.
    if let Some(sf) = specialized_endgame_scale(board) {
        return sf as f64 / SCALE_NORMAL as f64 * (199.0 - board.halfmove_clock.min(100) as f64)
            / 199.0;
    }

    let mut scale = 1.0f64;

    if let Some(ocb) = opposite_bishop_scale(board) {
        scale *= ocb as f64 / OCB_SCALE_NORMAL as f64;
    }

    if (has_only_king(board, Color::White) && has_only_knights(board, Color::Black, 2))
        || (has_only_king(board, Color::Black) && has_only_knights(board, Color::White, 2))
    {
        return 0.0;
    }

    let rule50 = board.halfmove_clock.min(100) as f64;
    scale *= (199.0 - rule50) / 199.0;
    scale
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

#[cfg(test)]
mod endgame_311c_tests {
    use super::*;

    fn board(fen: &str) -> Board {
        Board::from_fen(fen).unwrap_or_else(|e| panic!("bad test FEN {fen}: {e}"))
    }

    fn static_eval(fen: &str) -> i32 {
        Evaluator::default().evaluate(&board(fen))
    }

    /// The chess rule, not a hardcoded constant: KQ-vs-KP is a fortress draw
    /// only for rook/bishop pawns — knight and centre pawns are wins and must
    /// not be scaled.
    #[test]
    fn kqkp_fortress_only_draws_rook_and_bishop_pawns() {
        // Bishop pawn (c) on the 7th, king guarding c1, queen's king far: drawn.
        assert_eq!(
            kqkp_fortress_scale(&board("8/8/6K1/8/8/8/1kp5/7Q w - - 0 1")),
            Some(0)
        );
        // Knight pawn (b) on the 7th — a win for the queen — must NOT scale.
        assert_eq!(
            kqkp_fortress_scale(&board("8/8/6K1/8/8/8/kp6/7Q w - - 0 1")),
            None
        );
    }

    /// KR-vs-KP draw zone gets a conservative *partial* scale, never a forced 0.
    #[test]
    fn krkp_partial_scale_in_the_draw_zone() {
        assert_eq!(
            krkp_drawish_scale(&board("8/8/6K1/8/8/8/1kp5/7R w - - 0 1")),
            Some(16)
        );
    }

    /// Won endings must keep a clearly winning static score — guards against the
    /// earlier bug of scaling wins (knight-pawn KQKP, and KQ-vs-KR) toward draw.
    #[test]
    fn won_endings_are_not_scaled_toward_draw() {
        let kqkp_knight_pawn = static_eval("8/8/6K1/8/8/8/kp6/7Q w - - 0 1");
        assert!(
            kqkp_knight_pawn > 300,
            "won KQ vs knight-pawn should stay clearly winning, got {kqkp_knight_pawn}"
        );
        let kqkr = static_eval("8/8/6KQ/8/3k4/8/8/3r4 w - - 0 1");
        assert!(
            kqkr > 150,
            "won KQ vs KR should stay clearly winning, got {kqkr}"
        );
    }

    /// Passed pawns relax OCB scaling upward (less drawish); passer-free OCB
    /// keeps the exact pre-3.11 value.
    #[test]
    fn opposite_bishop_scale_relaxed_by_passers() {
        assert_eq!(
            opposite_bishop_scale(&board("4k3/p7/8/3b4/8/8/P7/2B1K3 w - - 0 1")),
            Some(40) // 2 pawns, no passers: 32 + 2*4
        );
        assert_eq!(
            opposite_bishop_scale(&board("4k3/7p/P7/3b4/8/8/8/2B1K3 w - - 0 1")),
            Some(48) // 2 passed pawns relax to the /48 cap
        );
    }
}
