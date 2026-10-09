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

        impl Default for EvalParams {
            fn default() -> Self {
                Self {
                    $( $field: $default, )*
                }
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
            passed_mg: 8 = [0, 0, 0, 9, 51, 116, 152, 0];
            passed_eg: 8 = [0, 0, 0, 21, 60, 106, 121, 0];
            passed_supported_mg: 1 = [9];
            passed_supported_eg_base: 1 = [0];
            passed_supported_eg_per_rank: 1 = [0];
            passed_freestop_mg_per_rank: 1 = [0];
            passed_freestop_eg_per_rank: 1 = [1];
            passed_safestop_eg_per_rank: 1 = [11];
            passed_candidate_mg: 1 = [0];
            passed_candidate_eg: 1 = [0];
            pawn_doubled_mg: 1 = [0];
            pawn_doubled_eg: 1 = [12];
            pawn_isolated_mg: 1 = [0];
            pawn_isolated_eg: 1 = [6];
            // Rank-scaled pawn *support* (Phase 3.8), Phase 4.4 fitted; indexed by the
            // pawn's relative rank (0..7). NB despite the historical name, this term
            // fires only for a pawn defended diagonally from behind by an own pawn
            // (`atk.pawn(them, sq) & our_pawns`) — the *phalanx* case (pawns abreast on
            // the same rank, which do not defend each other) is the separate
            // `pawn_phalanx_*` table below. (Rename to `pawn_supported_*` deferred: it
            // churns the tuner's string-keyed param list; see PLAN 7.4.)
            pawn_connected_mg: 8 = [7, 7, 34, 26, 27, 57, 175, 7];
            pawn_connected_eg: 8 = [5, 5, 9, 0, 14, 24, 22, 5];
            // Same-rank phalanx (Phase 7.4, seeded 0): a pawn with an own pawn on an
            // adjacent file *on the same rank* (d4+e4). Rank-scaled; the refit activates
            // it. Was entirely unrepresented before 7.4.
            pawn_phalanx_mg: 8 = [0, 0, 6, 20, 31, 4, 0, 0];
            pawn_phalanx_eg: 8 = [0, 0, 2, 11, 29, 14, 1, 0];
            pawn_backward_mg: 1 = [0];
            pawn_backward_eg: 1 = [14];
            // Pawn-structure / passer detail (Phase 3.8), Phase 4.4 fitted. pawn_lever
            // stayed frozen at 0 (feature-support: too sparse to fit reliably).
            pawn_lever_mg: 1 = [0];
            pawn_lever_eg: 1 = [0];
            pawn_doubled_isolated_mg: 1 = [4];
            pawn_doubled_isolated_eg: 1 = [13];
            blocked_passer_mg: 1 = [44];
            blocked_passer_eg: 1 = [10];
            ideal_blockader_mg: 1 = [19];
            ideal_blockader_eg: 1 = [0];
            // Minors & rooks. A weight the fit holds at 0 adds nothing atop mobility,
            // threats and the open-file terms.
            bishop_pair_mg: 1 = [24];
            bishop_pair_eg: 1 = [57];
            rook_open_mg: 1 = [51];
            rook_open_eg: 1 = [17];
            rook_semiopen_mg: 1 = [20];
            rook_semiopen_eg: 1 = [14];
            rook_7th_mg: 1 = [0];
            rook_7th_eg: 1 = [15];
            rook_behind_passer_mg: 1 = [0];
            rook_behind_passer_eg: 1 = [66];
            enemy_rook_behind_passer_mg: 1 = [25];
            enemy_rook_behind_passer_eg: 1 = [29];
            knight_outpost_mg: 1 = [55];
            knight_outpost_eg: 1 = [8];
            // Per-count mobility tables. Each is non-decreasing in the count (a trapped
            // piece is worst); low entries can go negative (e.g. a 0-mobility bishop).
            mob_n_mg: 9 = [-17, -10, 15, 27, 33, 38, 46, 53, 54];
            mob_n_eg: 9 = [-20, 11, 20, 35, 53, 75, 79, 79, 79];
            mob_b_mg: 14 = [11, 20, 30, 39, 46, 50, 54, 58, 61, 63, 70, 70, 70, 70];
            mob_b_eg: 14 = [-36, -3, 37, 49, 65, 76, 83, 87, 90, 91, 91, 91, 91, 91];
            mob_r_mg: 15 = [22, 32, 47, 54, 54, 59, 63, 63, 67, 74, 81, 88, 90, 90, 90];
            mob_r_eg: 15 = [4, 41, 41, 50, 65, 77, 86, 96, 104, 108, 111, 115, 115, 115, 121];
            mob_q_mg: 28 = [-45, 18, 63, 64, 66, 69, 69, 69, 74, 76, 82, 85, 87, 91, 91, 97, 97, 97, 97, 101, 101, 101, 101, 101, 101, 101, 101, 101];
            mob_q_eg: 28 = [-13, 8, 25, 25, 33, 33, 88, 96, 97, 101, 101, 101, 108, 111, 111, 118, 118, 121, 121, 125, 125, 125, 125, 125, 125, 125, 125, 125];
            // Threats. The base threat scalars share one value per phase; the per-victim
            // `threat_by_*` tables below carry the attacker/victim-specific signal.
            threat_minor_mg: 1 = [66];
            threat_minor_eg: 1 = [44];
            threat_rook_mg: 1 = [66];
            threat_rook_eg: 1 = [44];
            threat_queen_mg: 1 = [66];
            threat_queen_eg: 1 = [44];
            // Threats package v2 (Phase 3.6), seeded 0; fitted in Phase 4.2. Per-victim
            // arrays indexed by `Piece as usize` (0=pawn..5=king). The refined hanging
            // term absorbed the old flat hanging penalty, which the joint fit drove to
            // ~0 (see hanging_* below).
            threat_by_minor_mg: 6 = [0, 45, 80, 81, 71, 0];
            threat_by_minor_eg: 6 = [7, 28, 0, 0, 0, 0];
            threat_by_rook_mg: 6 = [0, 26, 41, 4, 70, 0];
            threat_by_rook_eg: 6 = [12, 21, 30, 1, 18, 0];
            threat_hanging_refined_mg: 6 = [3, 22, 42, 30, 0, 0];
            threat_hanging_refined_eg: 6 = [48, 28, 14, 2, 0, 0];
            threat_safe_pawn_push_mg: 1 = [36];
            threat_safe_pawn_push_eg: 1 = [2];
            threat_weak_piece_mg: 1 = [40];
            threat_weak_piece_eg: 1 = [0];
            threat_restricted_mg: 1 = [9];
            threat_restricted_eg: 1 = [0];
            king_safety_unit_minor: 1 = [2];
            king_safety_unit_rook: 1 = [3];
            king_safety_unit_queen: 1 = [4];
            // King-danger conversion table, indexed by the clamped danger index and
            // kept monotonic non-decreasing by the tuner.
            king_safety_table: 40 = [102, 102, 116, 116, 123, 138, 152, 163, 198, 222, 236, 263, 263, 263, 263, 263, 263, 263, 263, 263, 263, 282, 284, 358, 358, 376, 376, 376, 376, 376, 376, 376, 376, 376, 376, 376, 376, 384, 414, 499];
            // King-danger inputs. They select the table bucket non-linearly, so the
            // linear trace cannot see them; the fit's coordinate stage re-evaluates
            // positions to fit them. A weight at 0 leaves its input out of the index.
            ks_weak_ring: 1 = [1];
            ks_safe_check_knight: 1 = [16];
            ks_safe_check_bishop: 1 = [13];
            ks_safe_check_rook: 1 = [8];
            ks_safe_check_queen: 1 = [16];
            ks_queen_relief: 1 = [8];
            ks_flank_attack: 1 = [0];
            ks_pawnless_flank: 1 = [19];
            shelter_missing_file_mg: 1 = [27];
            shelter_missing_adjacent_mg: 1 = [7];
            shelter_dist1_mg: 1 = [25];
            shelter_dist2_mg: 1 = [15];
            storm_file_weight: 1 = [0];
            storm_adjacent_weight: 1 = [2];
            // Old flat hanging penalty (Phase 3.6). Phase 4.2 dropped it data-driven:
            // the refined hanging term (`threat_hanging_refined`) generalises and fully
            // absorbed it, so the joint fit drove these to ~0. Kept (not deleted) so the
            // term stays available; the values are now near-inert.
            hanging_minor: 1 = [0];
            hanging_rook: 1 = [0];
            hanging_queen: 1 = [0];
            passer_proximity_base: 1 = [11];
            space_weight: 1 = [0];
            tempo: 1 = [36];
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
            imbalance_ours: 36 = [25, 0, 0, 0, 0, 0, 3, 7, 0, 0, 0, 0, -11, 46, -23, 0, 0, 0, 26, 41, -30, -36, 0, 0, -5, 54, -50, -38, -41, 0, 1, 100, -101, -67, -135, -99];
            imbalance_theirs: 36 = [0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, -9, 45, 0, 0, 0, 0, 6, 50, -10, 0, 0, 0, -14, 66, 9, 10, 0, 0, 13, 130, 7, 35, 1, 0];
            // Small positional terms (Phase 3.10), all seeded 0 (bench unchanged),
            // tuned in Phase 4.4/4.5.
            // Small positional terms (Phase 3.10), Phase 4.4 fitted. rook_trapped frozen
            // (feature-support: too sparse).
            bishop_pair_pawn_mg: 1 = [1];
            bishop_pair_pawn_eg: 1 = [-3];
            bishop_outpost_mg: 1 = [45];
            bishop_outpost_eg: 1 = [1];
            rook_trapped_mg: 1 = [0];
            rook_trapped_eg: 1 = [0];
            rook_connected_mg: 1 = [6];
            rook_connected_eg: 1 = [47];
            bishop_long_diagonal_mg: 1 = [26];
            bishop_long_diagonal_eg: 1 = [1];
            bad_bishop_mg: 1 = [1];
            bad_bishop_eg: 1 = [19];
            initiative_weight: 1 = [1];
            // Closedness (rammed-pawn count) value swing: per own-piece-count, added
            // for knights (expected positive when tuned) and rooks (expected
            // negative). mg-only — see eval_closedness for the caveat that the
            // marginal lever beyond 3.7's per-count mobility is the material-value
            // swing alone, so this is deliberately kept as a single small weight.
            closedness_knight_mg: 1 = [10];
            closedness_rook_mg: 1 = [-11];
            // Central-king / lost-castling danger: fires only when the king is still
            // on its home square, on a central file, with all castling rights for
            // that side gone.
            king_centrality_danger_mg: 1 = [65];
            // Gauntlet-driven additions (Phase 3.12), Phase 4.4 fitted. king_protector /
            // space_piece fitted to 0 (no marginal value atop the rest).
            unstoppable_passer_eg: 1 = [66];
            minor_behind_pawn_mg: 1 = [16];
            minor_behind_pawn_eg: 1 = [0];
            pawn_islands_mg: 1 = [9];
            pawn_islands_eg: 1 = [1];
            queen_infiltration_mg: 1 = [48];
            queen_infiltration_eg: 1 = [85];
            king_protector_mg: 1 = [6];
            king_protector_eg: 1 = [4];
            space_piece_mg: 1 = [1];
            // Phase 6.2.1 refresh structure — all seeded 0 (inert, bench-identical);
            // activated by the 6.2.2 on-policy joint refit.
            // SF-style space refinement: safe central squares BEHIND own pawns,
            // weighted by piece count (rides next to space_piece_mg).
            space_behind_piece_mg: 1 = [0];
            // Passed-pawn whole-path weighting: the entire path to promotion is empty
            // ("free path") / never attacked by the enemy ("safe path"), scaled by
            // relative rank like the existing free/safe-stop terms.
            passed_freepath_mg_per_rank: 1 = [-7];
            passed_freepath_eg_per_rank: 1 = [8];
            passed_safepath_eg_per_rank: 1 = [24];
            // Deferred §3.12 trio.
            bishop_xray_pawns_mg: 1 = [-8];
            bishop_xray_pawns_eg: 1 = [4];
            queen_battery_mg: 1 = [15];
            queen_battery_eg: 1 = [21];
            slider_on_queen_mg: 1 = [35];
            slider_on_queen_eg: 1 = [8];
            // Shelter/storm folded into the king-danger index (nonlinear — selects the
            // safety-table bucket, so it is invisible to the linear trace and is fit by
            // the --tune-kingsafety re-eval path, like the other ks_* inputs). The
            // linear shelter/storm terms above stay; the Phase-4 fit zeroed the storm
            // weights because a LINEAR term cannot express "exposed king x piece
            // pressure" — this input is where that interaction lives.
            ks_shelter_storm: 1 = [0];
        }
    };
}

#[cfg(feature = "texel")]
pub(super) use eval_params;

eval_params!(define_eval_params);
/// Material + PST combined per (color, piece, square), rebuilt from
/// `EvalParams` whenever params change (Phase 3.1 — these used to be
/// `const`-baked `MG_TABLE`/`EG_TABLE`; now `params.mg_val`/`params.pst_mg`
/// are tunable data, so the table must be a runtime-built `Evaluator` field).
#[derive(Clone)]
pub(super) struct EvalTables {
    pub(super) mg: [[[i32; 64]; 6]; 2],
    pub(super) eg: [[[i32; 64]; 6]; 2],
}

pub(super) fn build_tables(params: &EvalParams) -> EvalTables {
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
