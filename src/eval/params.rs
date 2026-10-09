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
            passed_mg: 8 = [0, 0, 0, 11, 51, 111, 152, 0];
            passed_eg: 8 = [0, 0, 0, 19, 56, 105, 128, 0];
            passed_supported_mg: 1 = [11];
            passed_supported_eg_base: 1 = [0];
            passed_supported_eg_per_rank: 1 = [0];
            passed_freestop_mg_per_rank: 1 = [0];
            passed_freestop_eg_per_rank: 1 = [1];
            passed_safestop_eg_per_rank: 1 = [12];
            passed_candidate_mg: 1 = [0];
            passed_candidate_eg: 1 = [0];
            pawn_doubled_mg: 1 = [3];
            pawn_doubled_eg: 1 = [12];
            pawn_isolated_mg: 1 = [1];
            pawn_isolated_eg: 1 = [6];
            // Rank-scaled pawn *support* (Phase 3.8), Phase 4.4 fitted; indexed by the
            // pawn's relative rank (0..7). NB despite the historical name, this term
            // fires only for a pawn defended diagonally from behind by an own pawn
            // (`atk.pawn(them, sq) & our_pawns`) — the *phalanx* case (pawns abreast on
            // the same rank, which do not defend each other) is the separate
            // `pawn_phalanx_*` table below. (Rename to `pawn_supported_*` deferred: it
            // churns the tuner's string-keyed param list; see PLAN 7.4.)
            pawn_connected_mg: 8 = [7, 7, 31, 29, 31, 57, 175, 7];
            pawn_connected_eg: 8 = [5, 5, 10, 0, 14, 24, 22, 5];
            // Same-rank phalanx (Phase 7.4, seeded 0): a pawn with an own pawn on an
            // adjacent file *on the same rank* (d4+e4). Rank-scaled; the refit activates
            // it. Was entirely unrepresented before 7.4.
            pawn_phalanx_mg: 8 = [0, 1, 8, 21, 34, 6, 0, 0];
            pawn_phalanx_eg: 8 = [0, 0, 3, 12, 33, 16, 1, 0];
            pawn_backward_mg: 1 = [0];
            pawn_backward_eg: 1 = [13];
            // Pawn-structure / passer detail (Phase 3.8), Phase 4.4 fitted. pawn_lever
            // stayed frozen at 0 (feature-support: too sparse to fit reliably).
            pawn_lever_mg: 1 = [0];
            pawn_lever_eg: 1 = [0];
            pawn_doubled_isolated_mg: 1 = [3];
            pawn_doubled_isolated_eg: 1 = [13];
            blocked_passer_mg: 1 = [43];
            blocked_passer_eg: 1 = [10];
            ideal_blockader_mg: 1 = [17];
            ideal_blockader_eg: 1 = [0];
            // Minors & rooks. A weight the fit holds at 0 adds nothing atop mobility,
            // threats and the open-file terms.
            bishop_pair_mg: 1 = [24];
            bishop_pair_eg: 1 = [57];
            rook_open_mg: 1 = [46];
            rook_open_eg: 1 = [15];
            rook_semiopen_mg: 1 = [21];
            rook_semiopen_eg: 1 = [14];
            rook_7th_mg: 1 = [0];
            rook_7th_eg: 1 = [13];
            rook_behind_passer_mg: 1 = [0];
            rook_behind_passer_eg: 1 = [64];
            enemy_rook_behind_passer_mg: 1 = [27];
            enemy_rook_behind_passer_eg: 1 = [33];
            knight_outpost_mg: 1 = [50];
            knight_outpost_eg: 1 = [8];
            // Per-count mobility tables. Each is non-decreasing in the count (a trapped
            // piece is worst); low entries can go negative (e.g. a 0-mobility bishop).
            mob_n_mg: 9 = [-17, -8, 17, 28, 33, 37, 45, 51, 52];
            mob_n_eg: 9 = [-20, 11, 20, 35, 55, 75, 81, 81, 81];
            mob_b_mg: 14 = [10, 22, 32, 40, 47, 50, 54, 57, 60, 63, 70, 70, 70, 70];
            mob_b_eg: 14 = [-36, -3, 37, 50, 65, 76, 83, 88, 90, 91, 91, 91, 91, 91];
            mob_r_mg: 15 = [26, 35, 48, 54, 54, 60, 63, 63, 66, 72, 79, 86, 89, 90, 90];
            mob_r_eg: 15 = [4, 41, 41, 50, 65, 77, 86, 96, 104, 108, 111, 115, 115, 115, 119];
            mob_q_mg: 28 = [-45, 18, 61, 63, 65, 69, 69, 69, 74, 76, 82, 85, 87, 93, 93, 97, 97, 97, 97, 101, 101, 101, 101, 101, 101, 101, 101, 101];
            mob_q_eg: 28 = [-13, 8, 25, 25, 33, 33, 86, 94, 97, 101, 101, 101, 108, 111, 111, 118, 118, 121, 121, 125, 125, 125, 125, 125, 125, 125, 125, 125];
            // Threats. The base threat scalars share one value per phase; the per-victim
            // `threat_by_*` tables below carry the attacker/victim-specific signal.
            threat_minor_mg: 1 = [68];
            threat_minor_eg: 1 = [44];
            threat_rook_mg: 1 = [68];
            threat_rook_eg: 1 = [44];
            threat_queen_mg: 1 = [68];
            threat_queen_eg: 1 = [44];
            // Threats package v2 (Phase 3.6), seeded 0; fitted in Phase 4.2. Per-victim
            // arrays indexed by `Piece as usize` (0=pawn..5=king). The refined hanging
            // term absorbed the old flat hanging penalty, which the joint fit drove to
            // ~0 (see hanging_* below).
            threat_by_minor_mg: 6 = [0, 42, 77, 79, 70, 0];
            threat_by_minor_eg: 6 = [6, 28, 0, 0, 0, 0];
            threat_by_rook_mg: 6 = [0, 24, 39, 4, 68, 0];
            threat_by_rook_eg: 6 = [11, 21, 30, 1, 18, 0];
            threat_hanging_refined_mg: 6 = [3, 23, 40, 30, 0, 0];
            threat_hanging_refined_eg: 6 = [48, 28, 14, 3, 0, 0];
            threat_safe_pawn_push_mg: 1 = [34];
            threat_safe_pawn_push_eg: 1 = [2];
            threat_weak_piece_mg: 1 = [38];
            threat_weak_piece_eg: 1 = [0];
            threat_restricted_mg: 1 = [8];
            threat_restricted_eg: 1 = [0];
            // King-danger index coordinates, in index units (seeded on the donor's
            // scale, where a pawn is 206). The index reaches the score only through
            // the quadratic map, so the linear trace cannot see them; the fit's
            // coordinate stage re-evaluates positions to fit them.
            // Ring attackers' weight by attacker: knight, bishop, rook, queen.
            kd_attacker_weight: 4 = [45, 33, 20, 43];
            // Safe checks, single then multiple: knight, bishop, rook, queen.
            kd_safe_check: 8 = [648, 611, 645, 951, 588, 1161, 412, 623];
            kd_weak_ring: 1 = [73];
            kd_unsafe_check: 1 = [171];
            kd_blockers: 1 = [194];
            kd_king_attacks: 1 = [87];
            // Per 100 cp of the enemy's mobility (mg) over ours.
            kd_mobility: 1 = [0];
            kd_no_queen: 1 = [1113];
            kd_knight_defender: 1 = [0];
            kd_constant: 1 = [55];
            // The map from index to score, in hundredths of a centipawn per index
            // unit of `index²/4096` (mg) and `index/16` (eg): 49 is the donor's map
            // converted at 100/206. Coordinates too: the map's output is untraced.
            ks_map_mg: 1 = [73];
            ks_map_eg: 1 = [41];
            // Per 100 cp of our shelter's mg score, taken off the index.
            kd_shelter: 1 = [243];
            // Shelter and storm over the three files around the king, indexed
            // `edge_distance * 7 + relative_rank` (0 = no pawn on the file): our
            // pawn nearest our side, then theirs, unless it is blocked by ours.
            shelter_strength: 28 = [-2, 38, 42, 26, 17, 9, 12, -26, 31, 18, -19, -14, -5, -31, -2, 34, 14, 0, 13, -1, -22, -18, -6, -12, -23, -21, -33, -81];
            unblocked_storm: 28 = [43, -140, -81, 45, 26, 22, 26, 27, -12, 57, 22, 21, -4, 3, 2, 25, 80, 15, 0, -9, -11, -9, -5, 49, 2, 5, -9, -14];
            // An enemy pawn standing on our pawn, by its relative rank.
            blocked_storm_mg: 7 = [0, 0, 37, -5, -3, -2, 1];
            blocked_storm_eg: 7 = [0, 0, 38, 11, 9, 5, 1];
            shelter_constant_mg: 1 = [2];
            shelter_constant_eg: 1 = [2];
            // No pawn of either colour on the king's flank.
            pawnless_flank_mg: 1 = [8];
            pawnless_flank_eg: 1 = [52];
            // Old flat hanging penalty (Phase 3.6). Phase 4.2 dropped it data-driven:
            // the refined hanging term (`threat_hanging_refined`) generalises and fully
            // absorbed it, so the joint fit drove these to ~0. Kept (not deleted) so the
            // term stays available; the values are now near-inert.
            hanging_minor: 1 = [0];
            hanging_rook: 1 = [1];
            hanging_queen: 1 = [1];
            passer_proximity_base: 1 = [11];
            space_weight: 1 = [0];
            tempo: 1 = [34];
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
            imbalance_ours: 36 = [25, 0, 0, 0, 0, 0, 6, 7, 0, 0, 0, 0, -11, 45, -23, 0, 0, 0, 26, 38, -32, -36, 0, 0, -5, 53, -51, -38, -44, 0, -1, 97, -104, -71, -139, -101];
            imbalance_theirs: 36 = [0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, -9, 45, 0, 0, 0, 0, 6, 50, -10, 0, 0, 0, -14, 65, 7, 10, 0, 0, 13, 127, 10, 35, 0, 0];
            // Small positional terms (Phase 3.10), all seeded 0 (bench unchanged),
            // tuned in Phase 4.4/4.5.
            // Small positional terms (Phase 3.10), Phase 4.4 fitted. rook_trapped frozen
            // (feature-support: too sparse).
            bishop_pair_pawn_mg: 1 = [1];
            bishop_pair_pawn_eg: 1 = [-3];
            bishop_outpost_mg: 1 = [43];
            bishop_outpost_eg: 1 = [1];
            rook_trapped_mg: 1 = [0];
            rook_trapped_eg: 1 = [0];
            rook_connected_mg: 1 = [8];
            rook_connected_eg: 1 = [47];
            bishop_long_diagonal_mg: 1 = [21];
            bishop_long_diagonal_eg: 1 = [0];
            bad_bishop_mg: 1 = [0];
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
            unstoppable_passer_eg: 1 = [70];
            minor_behind_pawn_mg: 1 = [16];
            minor_behind_pawn_eg: 1 = [0];
            pawn_islands_mg: 1 = [8];
            pawn_islands_eg: 1 = [0];
            queen_infiltration_mg: 1 = [45];
            queen_infiltration_eg: 1 = [85];
            king_protector_mg: 1 = [6];
            king_protector_eg: 1 = [5];
            space_piece_mg: 1 = [1];
            // Phase 6.2.1 refresh structure — all seeded 0 (inert, bench-identical);
            // activated by the 6.2.2 on-policy joint refit.
            // SF-style space refinement: safe central squares BEHIND own pawns,
            // weighted by piece count (rides next to space_piece_mg).
            space_behind_piece_mg: 1 = [0];
            // Passed-pawn whole-path weighting: the entire path to promotion is empty
            // ("free path") / never attacked by the enemy ("safe path"), scaled by
            // relative rank like the existing free/safe-stop terms.
            passed_freepath_mg_per_rank: 1 = [-3];
            passed_freepath_eg_per_rank: 1 = [8];
            passed_safepath_eg_per_rank: 1 = [24];
            // Deferred §3.12 trio.
            bishop_xray_pawns_mg: 1 = [-7];
            bishop_xray_pawns_eg: 1 = [4];
            queen_battery_mg: 1 = [14];
            queen_battery_eg: 1 = [25];
            slider_on_queen_mg: 1 = [41];
            slider_on_queen_eg: 1 = [10];
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
