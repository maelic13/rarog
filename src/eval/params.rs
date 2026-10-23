//! Every tunable evaluation weight (`EvalParams`), the material and
//! piece-square tables built from them, and the `tune` build's loader and
//! dumper.

use super::material::{EG_VAL, MG_VAL, build_default_pst};
use crate::board::Color;

/// Every tunable eval weight. Uniform `[i32; N]` shape (scalars as `[i32; 1]`)
/// so the tune-time loader and the Texel tuner can address every field by
/// `(name, index)` through `EVAL_PARAM_NAMES`/`get`/`set` below.
macro_rules! define_eval_params {
    ( $( $field:ident : $len:literal = $default:expr; )* ) => {
        #[derive(Clone)]
        pub struct EvalParams {
            $( pub $field: [i32; $len], )*
        }

        impl EvalParams {
            /// The compiled-in weights, as a constant so a production build
            /// can fold every weight into the code that reads it.
            pub const DEFAULT: Self = Self {
                $( $field: $default, )*
            };
        }

        impl Default for EvalParams {
            fn default() -> Self {
                Self::DEFAULT
            }
        }

        /// (name, length) for every field. Consumed by the Phase 3.2/3.3
        /// tune-time loader/dumper and by `tools/texel-tuner`, which imports
        /// it directly — so this is live, not reserved.
        pub const EVAL_PARAM_NAMES: &[(&str, usize)] = &[
            $( (stringify!($field), $len), )*
        ];

        impl EvalParams {
            pub fn get(&self, name: &str, idx: usize) -> i32 {
                match name {
                    $( stringify!($field) => self.$field[idx], )*
                    _ => panic!("unknown eval param: {name}"),
                }
            }

            pub fn set(&mut self, name: &str, idx: usize, value: i32) {
                match name {
                    $( stringify!($field) => self.$field[idx] = value, )*
                    _ => panic!("unknown eval param: {name}"),
                }
            }
        }
    };
}

/// The weight list, `name: length = default;`, handed to a callback macro so
/// the parameter struct here and the texel trace types in `trace.rs` are
/// generated from one list. `tools/texel/bake_params.py` rewrites the defaults
/// in place.
macro_rules! eval_params {
    ($callback:ident) => {
        $callback! {
            mg_val: 6 = MG_VAL;
            eg_val: 6 = EG_VAL;
            pst_mg: 384 = build_default_pst(true);
            pst_eg: 384 = build_default_pst(false);
            // Passers & pawn structure (Phase 4.4 fitted). passed_*/connected per-rank
            // tables; passed bonuses stay monotonic (rank 1/8 pinned 0).
            passed_mg: 8 = [0, 2, 2, 12, 49, 102, 147, 0];
            passed_eg: 8 = [0, 1, 1, 15, 56, 121, 136, 0];
            passed_supported_mg: 1 = [9];
            passed_supported_eg_base: 1 = [0];
            passed_supported_eg_per_rank: 1 = [3];
            passed_freestop_mg_per_rank: 1 = [0];
            passed_freestop_eg_per_rank: 1 = [2];
            passed_safestop_eg_per_rank: 1 = [8];
            pawn_doubled_mg: 1 = [2];
            pawn_doubled_eg: 1 = [16];
            pawn_isolated_mg: 1 = [0];
            pawn_isolated_eg: 1 = [0];
            // Rank-scaled pawn *support* (Phase 3.8), Phase 4.4 fitted; indexed by the
            // pawn's relative rank (0..7). NB despite the historical name, this term
            // fires only for a pawn defended diagonally from behind by an own pawn
            // (`atk.pawn(them, sq) & our_pawns`) — the *phalanx* case (pawns abreast on
            // the same rank, which do not defend each other) is the separate
            // `pawn_phalanx_*` table below. (Rename to `pawn_supported_*` deferred: it
            // churns the tuner's string-keyed param list; see PLAN 7.4.)
            pawn_connected_mg: 8 = [7, 7, 27, 27, 33, 53, 175, 7];
            pawn_connected_eg: 8 = [5, 5, 9, 0, 10, 29, 22, 5];
            // Same-rank phalanx (Phase 7.4, seeded 0): a pawn with an own pawn on an
            // adjacent file *on the same rank* (d4+e4). Rank-scaled; the refit activates
            // it. Was entirely unrepresented before 7.4.
            pawn_phalanx_mg: 8 = [0, 1, 9, 20, 40, 13, 0, 0];
            pawn_phalanx_eg: 8 = [0, 0, 4, 11, 41, 26, 1, 0];
            pawn_backward_mg: 1 = [0];
            pawn_backward_eg: 1 = [11];
            pawn_doubled_isolated_mg: 1 = [0];
            pawn_doubled_isolated_eg: 1 = [15];
            // An isolated or backward pawn with no enemy pawn ahead on its
            // file: open to attack down the file. Seed the donor's (13, 27) at
            // 100/206.
            pawn_weak_unopposed_mg: 1 = [13];
            pawn_weak_unopposed_eg: 1 = [5];
            blocked_passer_mg: 1 = [35];
            blocked_passer_eg: 1 = [0];
            ideal_blockader_mg: 1 = [13];
            ideal_blockader_eg: 1 = [4];
            // Minors & rooks. A weight the fit holds at 0 adds nothing atop mobility,
            // threats and the open-file terms.
            bishop_pair_mg: 1 = [24];
            bishop_pair_eg: 1 = [57];
            rook_open_mg: 1 = [45];
            rook_open_eg: 1 = [10];
            rook_semiopen_mg: 1 = [18];
            rook_semiopen_eg: 1 = [14];
            rook_7th_mg: 1 = [0];
            rook_7th_eg: 1 = [7];
            rook_behind_passer_mg: 1 = [0];
            rook_behind_passer_eg: 1 = [53];
            enemy_rook_behind_passer_mg: 1 = [31];
            enemy_rook_behind_passer_eg: 1 = [49];
            knight_outpost_mg: 1 = [39];
            knight_outpost_eg: 1 = [9];
            // Per-count mobility tables. Each is non-decreasing in the count (a trapped
            // piece is worst); low entries can go negative (e.g. a 0-mobility bishop).
            mob_n_mg: 9 = [-17, 1, 22, 29, 33, 35, 37, 44, 48];
            mob_n_eg: 9 = [-20, 9, 20, 38, 60, 81, 86, 86, 86];
            mob_b_mg: 14 = [10, 26, 36, 43, 47, 49, 55, 55, 58, 59, 67, 70, 70, 70];
            mob_b_eg: 14 = [-36, -1, 37, 50, 66, 77, 83, 88, 90, 91, 91, 91, 91, 91];
            mob_r_mg: 15 = [35, 43, 54, 60, 60, 64, 65, 66, 66, 68, 74, 80, 82, 90, 90];
            mob_r_eg: 15 = [4, 41, 42, 52, 65, 78, 86, 97, 106, 110, 112, 115, 115, 115, 115];
            mob_q_mg: 28 = [-45, 18, 55, 60, 61, 68, 68, 68, 73, 77, 82, 86, 91, 95, 97, 97, 98, 99, 99, 101, 101, 101, 101, 101, 101, 101, 101, 101];
            mob_q_eg: 28 = [-13, 8, 25, 25, 33, 33, 84, 93, 96, 101, 101, 101, 108, 111, 112, 118, 118, 121, 121, 125, 125, 125, 125, 125, 125, 125, 125, 125];
            // Threats. The base threat scalars share one value per phase; the per-victim
            // `threat_by_*` tables below carry the attacker/victim-specific signal.
            threat_minor_mg: 1 = [67];
            threat_minor_eg: 1 = [44];
            threat_rook_mg: 1 = [69];
            threat_rook_eg: 1 = [44];
            threat_queen_mg: 1 = [69];
            threat_queen_eg: 1 = [44];
            // Threats package v2 (Phase 3.6), seeded 0; fitted in Phase 4.2. Per-victim
            // arrays indexed by `Piece as usize` (0=pawn..5=king). The refined hanging
            // term absorbed the old flat hanging penalty, which the joint fit drove to
            // ~0 (see hanging_* below).
            threat_by_minor_mg: 6 = [0, 37, 66, 72, 71, 0];
            threat_by_minor_eg: 6 = [6, 28, 2, 0, 0, 0];
            threat_by_rook_mg: 6 = [0, 19, 38, 4, 68, 0];
            threat_by_rook_eg: 6 = [9, 24, 30, 1, 18, 0];
            threat_hanging_refined_mg: 6 = [7, 25, 33, 30, 0, 0];
            threat_hanging_refined_eg: 6 = [36, 23, 14, 3, 0, 0];
            threat_safe_pawn_push_mg: 1 = [28];
            threat_safe_pawn_push_eg: 1 = [7];
            threat_weak_piece_mg: 1 = [35];
            threat_weak_piece_eg: 1 = [0];
            threat_restricted_mg: 1 = [8];
            threat_restricted_eg: 1 = [0];
            // The king attacking a weak enemy piece, and bishops or rooks attacking
            // the safe squares around the enemy's only queen; seeded from the
            // donor's values on its 206-per-pawn scale.
            threat_by_king_mg: 1 = [17];
            threat_by_king_eg: 1 = [24];
            threat_slider_on_queen_mg: 1 = [20];
            threat_slider_on_queen_eg: 1 = [4];
            // King-danger index coordinates, in index units (seeded on the donor's
            // scale, where a pawn is 206). The index reaches the score only through
            // the quadratic map, so the linear trace cannot see them; the fit's
            // coordinate stage re-evaluates positions to fit them.
            // Ring attackers' weight by attacker: knight, bishop, rook, queen.
            kd_attacker_weight: 4 = [33, 26, 13, 20];
            // Safe checks, single then multiple: knight, bishop, rook, queen.
            kd_safe_check: 8 = [444, 333, 327, 265, 495, 743, 338, 493];
            kd_weak_ring: 1 = [57];
            kd_unsafe_check: 1 = [107];
            kd_blockers: 1 = [149];
            kd_king_attacks: 1 = [67];
            // Per 100 cp of the enemy's mobility (mg) over ours.
            kd_mobility: 1 = [32];
            kd_no_queen: 1 = [769];
            kd_knight_defender: 1 = [36];
            kd_constant: 1 = [151];
            // The map from index to score, in hundredths of a centipawn per index
            // unit of `index²/4096` (mg) and `index/16` (eg): 49 is the donor's map
            // converted at 100/206. Coordinates too: the map's output is untraced.
            ks_map_mg: 1 = [102];
            ks_map_eg: 1 = [18];
            // Per 100 cp of our shelter's mg score, taken off the index.
            kd_shelter: 1 = [207];
            // Shelter and storm over the three files around the king, indexed
            // `edge_distance * 7 + relative_rank` (0 = no pawn on the file): our
            // pawn nearest our side, then theirs, unless it is blocked by ours.
            shelter_strength: 28 = [-4, 33, 40, 20, 11, 9, 12, -29, 35, 16, -14, -14, -5, -31, 0, 40, 12, 1, 8, -5, -22, -23, -6, -8, -16, -14, -33, -81];
            unblocked_storm: 28 = [38, -140, -75, 43, 32, 28, 27, 29, -12, 51, 22, 21, -2, 2, 4, 25, 76, 11, 1, -9, -6, -11, -5, 49, 7, 4, -17, -14];
            // An enemy pawn standing on our pawn, by its relative rank.
            blocked_storm_mg: 7 = [0, 0, 31, -3, -6, 0, 6];
            blocked_storm_eg: 7 = [0, 0, 38, 19, 19, 12, 3];
            shelter_constant_mg: 1 = [2];
            shelter_constant_eg: 1 = [2];
            // No pawn of either colour on the king's flank.
            pawnless_flank_mg: 1 = [8];
            pawnless_flank_eg: 1 = [60];
            // Old flat hanging penalty (Phase 3.6). Phase 4.2 dropped it data-driven:
            // the refined hanging term (`threat_hanging_refined`) generalises and fully
            // absorbed it, so the joint fit drove these to ~0. Kept (not deleted) so the
            // term stays available; the values are now near-inert.
            hanging_minor: 1 = [0];
            hanging_rook: 1 = [1];
            hanging_queen: 1 = [1];
            // Kings' distances to the square in front of a passer (and our
            // king's to the square after it), per unit of distance × rank
            // weight; eg-only. Seeds are the donor's 19/4, 2 and 1 at 100/206.
            passer_king_them_eg: 1 = [4];
            passer_king_us_eg: 1 = [3];
            passer_king_next_eg: 1 = [1];
            tempo: 1 = [33];
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
            imbalance_ours: 36 = [25, 0, 0, 0, 0, 0, 6, 7, 0, 0, 0, 0, -9, 41, -24, 0, 0, 0, 25, 36, -34, -36, 0, 0, -7, 47, -58, -44, -49, 0, -4, 89, -113, -82, -156, -105];
            imbalance_theirs: 36 = [0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, -8, 45, 0, 0, 0, 0, 6, 48, -11, 0, 0, 0, -13, 64, 7, 9, 0, 0, 12, 125, 16, 37, 0, 0];
            // Small positional terms (Phase 3.10), all seeded 0 (bench unchanged),
            // tuned in Phase 4.4/4.5.
            // Small positional terms (Phase 3.10), Phase 4.4 fitted.
            bishop_pair_pawn_mg: 1 = [-1];
            bishop_pair_pawn_eg: 1 = [-4];
            bishop_outpost_mg: 1 = [37];
            bishop_outpost_eg: 1 = [1];
            rook_connected_mg: 1 = [9];
            rook_connected_eg: 1 = [47];
            bishop_long_diagonal_mg: 1 = [10];
            bishop_long_diagonal_eg: 1 = [0];
            bad_bishop_mg: 1 = [2];
            bad_bishop_eg: 1 = [15];
            initiative_weight: 1 = [1];
            // Closedness (rammed-pawn count) value swing: per own-piece-count, added
            // for knights (expected positive when tuned) and rooks (expected
            // negative). mg-only — see eval_closedness for the caveat that the
            // marginal lever beyond 3.7's per-count mobility is the material-value
            // swing alone, so this is deliberately kept as a single small weight.
            closedness_knight_mg: 1 = [7];
            closedness_rook_mg: 1 = [-10];
            // Central-king / lost-castling danger: fires only when the king is still
            // on its home square, on a central file, with all castling rights for
            // that side gone.
            king_centrality_danger_mg: 1 = [65];
            // Gauntlet-driven additions (Phase 3.12), Phase 4.4 fitted.
            unstoppable_passer_eg: 1 = [88];
            minor_behind_pawn_mg: 1 = [15];
            minor_behind_pawn_eg: 1 = [0];
            pawn_islands_mg: 1 = [6];
            pawn_islands_eg: 1 = [0];
            queen_infiltration_mg: 1 = [41];
            queen_infiltration_eg: 1 = [87];
            king_protector_mg: 1 = [6];
            king_protector_eg: 1 = [5];
            // Phase 6.2.1 refresh structure — all seeded 0 (inert, bench-identical);
            // activated by the 6.2.2 on-policy joint refit.
            // Passed-pawn whole-path weighting: the entire path to promotion is empty
            // ("free path") / never attacked by the enemy ("safe path"), scaled by
            // relative rank like the existing free/safe-stop terms.
            passed_freepath_mg_per_rank: 1 = [-2];
            passed_freepath_eg_per_rank: 1 = [4];
            passed_safepath_eg_per_rank: 1 = [21];
            // Bishop x-rays on enemy pawns and queen batteries.
            bishop_xray_pawns_mg: 1 = [-5];
            bishop_xray_pawns_eg: 1 = [6];
            queen_battery_mg: 1 = [9];
            queen_battery_eg: 1 = [37];
        }
    };
}

#[cfg(feature = "texel")]
pub(super) use eval_params;

eval_params!(define_eval_params);
/// Material + PST combined per (color, piece, square), built from
/// `EvalParams`: once at compile time from the compiled-in weights, and at
/// run time in the `tune` and `texel` builds, whose weights change.
#[derive(Clone)]
pub(super) struct EvalTables {
    pub(super) mg: [[[i32; 64]; 6]; 2],
    pub(super) eg: [[[i32; 64]; 6]; 2],
}

pub(super) const fn build_tables(params: &EvalParams) -> EvalTables {
    let mut mg = [[[0i32; 64]; 6]; 2];
    let mut eg = [[[0i32; 64]; 6]; 2];
    let mut piece = 0;
    while piece < 6 {
        let mut sq = 0;
        while sq < 64 {
            mg[Color::White as usize][piece][sq] =
                params.mg_val[piece] + params.pst_mg[piece * 64 + sq];
            mg[Color::Black as usize][piece][sq] =
                params.mg_val[piece] + params.pst_mg[piece * 64 + (sq ^ 56)];
            eg[Color::White as usize][piece][sq] =
                params.eg_val[piece] + params.pst_eg[piece * 64 + sq];
            eg[Color::Black as usize][piece][sq] =
                params.eg_val[piece] + params.pst_eg[piece * 64 + (sq ^ 56)];
            sq += 1;
        }
        piece += 1;
    }
    EvalTables { mg, eg }
}

/// The weights a production build evaluates with: a zero-sized handle to
/// the compiled-in constants. Reading a weight through it is a load from a
/// constant the compiler sees, so it folds into the code; a weight read
/// through `Evaluator` memory is a load at every use. The `tune` and `texel`
/// builds keep a runtime `EvalParams`, because their weights change.
#[cfg(not(any(feature = "tune", feature = "texel", test)))]
#[derive(Clone, Copy)]
pub(super) struct CompiledParams;

#[cfg(not(any(feature = "tune", feature = "texel", test)))]
static COMPILED_PARAMS: EvalParams = EvalParams::DEFAULT;

#[cfg(not(any(feature = "tune", feature = "texel", test)))]
impl std::ops::Deref for CompiledParams {
    type Target = EvalParams;

    #[inline(always)]
    fn deref(&self) -> &EvalParams {
        &COMPILED_PARAMS
    }
}

/// The material and piece-square tables of the compiled-in weights, built at
/// compile time; see [`CompiledParams`].
#[cfg(not(any(feature = "tune", feature = "texel", test)))]
#[derive(Clone, Copy)]
pub(super) struct CompiledTables;

#[cfg(not(any(feature = "tune", feature = "texel", test)))]
static COMPILED_TABLES: EvalTables = build_tables(&EvalParams::DEFAULT);

#[cfg(not(any(feature = "tune", feature = "texel", test)))]
impl std::ops::Deref for CompiledTables {
    type Target = EvalTables;

    #[inline(always)]
    fn deref(&self) -> &EvalTables {
        &COMPILED_TABLES
    }
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
    fn load_from_str(text: &str) -> Self {
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

    pub(crate) fn load_from_env() -> Self {
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

    pub(crate) fn dump(&self) -> String {
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
