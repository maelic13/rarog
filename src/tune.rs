//! Tuning infrastructure for SPSA (search constants) and Texel (eval weights).
//!
//! This module is only compiled when the `tune` feature is enabled:
//!
//! ```text
//! cargo build --features tune
//! ```
//!
//! # Eval tuning
//!
//! Set `RAROG_TUNE_FILE` to the path of a plain-text parameter file, then run the
//! engine normally.  Every call to `crate::eval::PARAMS` (auto-dereffed from its
//! `LazyLock`) will use the loaded values instead of the compile-time defaults.
//!
//! ## Parameter file format
//!
//! One parameter per line, name followed by space-separated value(s).
//! Lines starting with `#` and blank lines are ignored.
//!
//! ```text
//! # Rarog eval parameter file
//! tempo 12
//! passed_mg  0  6 11 21 36 62 102  0
//! mob_mg     0  5  6  3  2  0
//! ks_divisor 10
//! phalanx_mg 0 3 5 7 10 13 16 0
//! ```
//!
//! Array fields are written as a space-separated sequence of values on the same
//! line.  Only named fields are overridden; everything else keeps its DEFAULT.
//!
//! # Search tuning
//!
//! The key search constants targeted for SPSA (with their current defaults) are
//! listed below.  To tune them, extract each into a `SearchParams` struct
//! (similar to `EvalParams`) and load from `RAROG_SEARCH_TUNE_FILE`.
//!
//! ## Current search constants and default values (from src/search.rs)
//!
//! ### LMR base formula
//! `LMR_TABLE[d][m] = (1024 * (0.75 + ln(d) * ln(m) / 2.25)) as i32`
//! Tunable: base_offset = 0.75, divisor = 2.25 (fractional, requires recompile).
//!
//! ### LMR weighted terms (all in 1024ths)
//! | Name              | Default | Description                           |
//! |-------------------|---------|---------------------------------------|
//! | lmr_tt_pv         | -463    | reduction for PV / TT-PV nodes        |
//! | lmr_exact_bound   | +1405   | TT bound is Exact                     |
//! | lmr_shallow_tt    | +286    | TT entry depth < depth − 1            |
//! | lmr_cut_node      | +1810   | cut node                              |
//! | lmr_cut_no_tt     | +2113   | cut node AND no TT move               |
//! | lmr_improving     | -1024   | position is improving                 |
//! | lmr_quiet_base    | +2171   | quiet move base                       |
//! | lmr_quiet_hist    | -179    | quiet history divisor (per 1024)      |
//! | lmr_noisy_base    | +1724   | bad-capture base                      |
//! | lmr_noisy_hist    | -107    | bad-capture history divisor           |
//! | lmr_corr_scale    | -3403   | correction magnitude (per 1024)       |
//! | lmr_cutoff_count  | +992    | cutoff_count[ply] > 2                 |
//! | lmr_killer_counter| -1024   | move is killer or countermove         |
//!
//! ### do-deeper / do-shallower
//! | Name              | Default | Description                           |
//! |-------------------|---------|---------------------------------------|
//! | do_deeper_margin  | 54      | score > best_score + margin: +1 depth |
//! | do_shallower_margin| 8      | score < best_score + margin: -1 depth |
//!
//! ### Reverse Futility Pruning (RFP)
//! `margin = (rfp_base + rfp_not_impr * not_improving) * depth + |corr| * rfp_corr_scale / 1024`
//! | Name              | Default |
//! |-------------------|---------|
//! | rfp_base          | 70      |
//! | rfp_not_impr      | 20      |
//! | rfp_corr_scale    | 60      |
//!
//! ### Null Move Pruning (NMP)
//! `reduction = 4 + depth/4 + ((eval − beta) / nmp_eval_div).clamp(0, 3) + improving`
//! | Name              | Default |
//! |-------------------|---------|
//! | nmp_base          | 4       |
//! | nmp_depth_div     | 4       |
//! | nmp_eval_div      | 200     |
//!
//! ### Razoring
//! `threshold = eval + razor_base + razor_quad * depth * depth`
//! | Name              | Default |
//! |-------------------|---------|
//! | razor_base        | 200     |
//! | razor_quad        | 250     |
//!
//! ### ProbCut
//! `probcut_beta = beta + probcut_base - probcut_impr * improving`
//! | Name              | Default |
//! |-------------------|---------|
//! | probcut_base      | 200     |
//! | probcut_impr      | 80      |
//!
//! ### Futility (move loop)
//! `prune_margin = (futility_base + futility_not_impr * not_improving) * depth + hist / 128`
//! | Name              | Default |
//! |-------------------|---------|
//! | futility_base     | 90      |
//! | futility_not_impr | 25      |
//!
//! ### Aspiration windows
//! `initial_delta = asp_base + best_score^2 / asp_score_div`
//! Fail-low: `delta += delta * asp_fail_low_num / 128`
//! Fail-high: `delta += delta * asp_fail_high_num / 128`
//! | Name              | Default |
//! |-------------------|---------|
//! | asp_base          | 12      |
//! | asp_score_div     | 16000   |
//! | asp_fail_low_num  | 26      |
//! | asp_fail_high_num | 60      |
//!
//! ### History bonus cap
//! `history_bonus(depth) = (depth*depth + 2*depth).min(history_bonus_cap)`
//! Default cap: 1200.
//!
//! ### Correction history
//! `scaled = (146 * depth * diff / 128).clamp(-4449, 2659)`
//! Divisor on read side: `/128` (currently; Reckless uses /69).

use crate::board::{Board, Color, Piece};
use crate::eval::{EvalParams, EG_TABLE, MG_TABLE, PHASE_W, TOTAL_PHASE};

/// Standalone positional evaluation for tuning.
///
/// Evaluates the board using explicit `params` (no global `PARAMS` reference).
/// Returns a score in centipawns from the side-to-move's perspective.
///
/// The evaluation covers material + PSQT (fixed tables) plus the EvalParams
/// bonus and penalty terms.  The full-engine eval also includes pawn structure,
/// mobility, king safety, and passed-pawn terms — those are **not** included
/// here yet; add them incrementally as the tuning coverage expands.
///
/// NOTE: The phase-weighting tables (MG_TABLE, EG_TABLE) embed the base
/// material values (MG_VAL/EG_VAL) plus PST offsets.  Those are currently
/// hardcoded rather than in EvalParams; adding them to EvalParams is the next
/// expansion step for full PSQT tuning.
pub fn tune_eval(board: &Board, params: &EvalParams) -> i32 {
    let mut mg = 0i32;
    let mut eg = 0i32;
    let mut phase = 0i32;

    // Material + PSQT (fixed tables derived from hardcoded MG_VAL/EG_VAL + PSTs)
    for color in [Color::White, Color::Black] {
        let sign: i32 = if color == Color::White { 1 } else { -1 };
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

    // Tempo
    let tempo = if board.side_to_move() == Color::White {
        params.tempo
    } else {
        -params.tempo
    };
    mg += tempo;

    let score = (mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
    if board.side_to_move() == Color::White { score } else { -score }
}

/// Load an `EvalParams` from `path`, starting from `EvalParams::DEFAULT`.
/// Unknown or malformed lines are silently skipped so a partial file is valid.
pub fn load_eval_params(path: &str) -> EvalParams {
    let mut p = EvalParams::DEFAULT;
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("info string tune: cannot read {path}: {e}");
            return p;
        }
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, ' ');
        let name = match parts.next() {
            Some(n) => n.trim(),
            None => continue,
        };
        let rest = parts.next().unwrap_or("").trim();
        let mut vals = rest.split_whitespace().filter_map(|s| s.parse::<i32>().ok());

        macro_rules! scalar {
            ($field:ident) => {
                if let Some(v) = vals.next() {
                    p.$field = v;
                }
            };
        }
        macro_rules! array {
            ($field:ident, $len:expr) => {
                for i in 0..$len {
                    if let Some(v) = vals.next() {
                        p.$field[i] = v;
                    }
                }
            };
        }

        match name {
            "tempo" => scalar!(tempo),
            "passed_mg" => array!(passed_mg, 8),
            "passed_eg" => array!(passed_eg, 8),
            "passed_defended_mg" => scalar!(passed_defended_mg),
            "passed_defended_eg_base" => scalar!(passed_defended_eg_base),
            "passed_defended_eg_rank" => scalar!(passed_defended_eg_rank),
            "passed_free_mg_rank" => scalar!(passed_free_mg_rank),
            "passed_free_eg_rank" => scalar!(passed_free_eg_rank),
            "passed_free_safe_eg_rank" => scalar!(passed_free_safe_eg_rank),
            "candidate_mg" => scalar!(candidate_mg),
            "candidate_eg" => scalar!(candidate_eg),
            "doubled_mg" => scalar!(doubled_mg),
            "doubled_eg" => scalar!(doubled_eg),
            "isolated_mg" => scalar!(isolated_mg),
            "isolated_eg" => scalar!(isolated_eg),
            "defended_mg" => scalar!(defended_mg),
            "defended_eg" => scalar!(defended_eg),
            "backward_mg" => scalar!(backward_mg),
            "backward_eg" => scalar!(backward_eg),
            "bishop_pair_mg" => scalar!(bishop_pair_mg),
            "bishop_pair_eg" => scalar!(bishop_pair_eg),
            "rook_open_mg" => scalar!(rook_open_mg),
            "rook_open_eg" => scalar!(rook_open_eg),
            "rook_semi_mg" => scalar!(rook_semi_mg),
            "rook_semi_eg" => scalar!(rook_semi_eg),
            "rook_seventh_mg" => scalar!(rook_seventh_mg),
            "rook_seventh_eg" => scalar!(rook_seventh_eg),
            "knight_outpost_mg" => scalar!(knight_outpost_mg),
            "knight_outpost_eg" => scalar!(knight_outpost_eg),
            "mob_mg" => array!(mob_mg, 6),
            "mob_eg" => array!(mob_eg, 6),
            "threat_minor_mg" => scalar!(threat_minor_mg),
            "threat_minor_eg" => scalar!(threat_minor_eg),
            "threat_rook_mg" => scalar!(threat_rook_mg),
            "threat_rook_eg" => scalar!(threat_rook_eg),
            "threat_queen_mg" => scalar!(threat_queen_mg),
            "threat_queen_eg" => scalar!(threat_queen_eg),
            "ks_minor_weight" => scalar!(ks_minor_weight),
            "ks_rook_weight" => scalar!(ks_rook_weight),
            "ks_queen_weight" => scalar!(ks_queen_weight),
            "ks_ring_attack" => scalar!(ks_ring_attack),
            "ks_safe_check_queen" => scalar!(ks_safe_check_queen),
            "ks_safe_check_rook" => scalar!(ks_safe_check_rook),
            "ks_safe_check_bishop" => scalar!(ks_safe_check_bishop),
            "ks_safe_check_knight" => scalar!(ks_safe_check_knight),
            "ks_no_queen" => scalar!(ks_no_queen),
            "ks_divisor" => scalar!(ks_divisor),
            "ks_max_penalty" => scalar!(ks_max_penalty),
            "shelter_open_king" => scalar!(shelter_open_king),
            "shelter_open_adj" => scalar!(shelter_open_adj),
            "shelter_close1" => scalar!(shelter_close1),
            "shelter_close2" => scalar!(shelter_close2),
            "storm_king_file" => scalar!(storm_king_file),
            "storm_adj_file" => scalar!(storm_adj_file),
            "rook_passer_mg" => scalar!(rook_passer_mg),
            "rook_passer_eg" => scalar!(rook_passer_eg),
            "enemy_rook_passer_mg" => scalar!(enemy_rook_passer_mg),
            "enemy_rook_passer_eg" => scalar!(enemy_rook_passer_eg),
            "hanging_minor" => scalar!(hanging_minor),
            "hanging_rook" => scalar!(hanging_rook),
            "hanging_queen" => scalar!(hanging_queen),
            "king_prox_base" => scalar!(king_prox_base),
            "king_push_weight" => scalar!(king_push_weight),
            "king_prox_weight" => scalar!(king_prox_weight),
            "king_prox_max_dist" => scalar!(king_prox_max_dist),
            "space" => scalar!(space),
            "trapped_bishop_mg" => scalar!(trapped_bishop_mg),
            "trapped_bishop_eg" => scalar!(trapped_bishop_eg),
            "ocb_base" => scalar!(ocb_base),
            "ocb_per_pawn" => scalar!(ocb_per_pawn),
            "ocb_cap" => scalar!(ocb_cap),
            "threat_attack_minor_mg" => scalar!(threat_attack_minor_mg),
            "threat_attack_minor_eg" => scalar!(threat_attack_minor_eg),
            "threat_rook_queen_mg" => scalar!(threat_rook_queen_mg),
            "threat_rook_queen_eg" => scalar!(threat_rook_queen_eg),
            "threat_push_mg" => scalar!(threat_push_mg),
            "threat_push_eg" => scalar!(threat_push_eg),
            "restricted_mobility_mg" => scalar!(restricted_mobility_mg),
            "restricted_mobility_eg" => scalar!(restricted_mobility_eg),
            "bishop_outpost_mg" => scalar!(bishop_outpost_mg),
            "bishop_outpost_eg" => scalar!(bishop_outpost_eg),
            "phalanx_mg" => array!(phalanx_mg, 8),
            "phalanx_eg" => array!(phalanx_eg, 8),
            _ => {
                eprintln!("info string tune: unknown parameter '{name}'");
            }
        }
    }
    p
}

/// Write `params` to `path` in the load format (one parameter per line).
pub fn save_eval_params(params: &EvalParams, path: &str) -> std::io::Result<()> {
    use std::fmt::Write as FmtWrite;
    let mut out = String::with_capacity(4096);
    macro_rules! w {
        ($name:ident) => {
            let _ = writeln!(out, "{} {}", stringify!($name), params.$name);
        };
        ($name:ident[$len:expr]) => {
            let vals: Vec<String> = (0..$len).map(|i| params.$name[i].to_string()).collect();
            let _ = writeln!(out, "{} {}", stringify!($name), vals.join(" "));
        };
    }
    w!(tempo);
    w!(passed_mg[8]); w!(passed_eg[8]);
    w!(passed_defended_mg); w!(passed_defended_eg_base); w!(passed_defended_eg_rank);
    w!(passed_free_mg_rank); w!(passed_free_eg_rank); w!(passed_free_safe_eg_rank);
    w!(candidate_mg); w!(candidate_eg);
    w!(doubled_mg); w!(doubled_eg);
    w!(isolated_mg); w!(isolated_eg);
    w!(defended_mg); w!(defended_eg);
    w!(backward_mg); w!(backward_eg);
    w!(bishop_pair_mg); w!(bishop_pair_eg);
    w!(rook_open_mg); w!(rook_open_eg);
    w!(rook_semi_mg); w!(rook_semi_eg);
    w!(rook_seventh_mg); w!(rook_seventh_eg);
    w!(knight_outpost_mg); w!(knight_outpost_eg);
    w!(mob_mg[6]); w!(mob_eg[6]);
    w!(threat_minor_mg); w!(threat_minor_eg);
    w!(threat_rook_mg); w!(threat_rook_eg);
    w!(threat_queen_mg); w!(threat_queen_eg);
    w!(ks_minor_weight); w!(ks_rook_weight); w!(ks_queen_weight);
    w!(ks_ring_attack);
    w!(ks_safe_check_queen); w!(ks_safe_check_rook);
    w!(ks_safe_check_bishop); w!(ks_safe_check_knight);
    w!(ks_no_queen); w!(ks_divisor); w!(ks_max_penalty);
    w!(shelter_open_king); w!(shelter_open_adj); w!(shelter_close1); w!(shelter_close2);
    w!(storm_king_file); w!(storm_adj_file);
    w!(rook_passer_mg); w!(rook_passer_eg);
    w!(enemy_rook_passer_mg); w!(enemy_rook_passer_eg);
    w!(hanging_minor); w!(hanging_rook); w!(hanging_queen);
    w!(king_prox_base); w!(king_push_weight); w!(king_prox_weight); w!(king_prox_max_dist);
    w!(space);
    w!(trapped_bishop_mg); w!(trapped_bishop_eg);
    w!(ocb_base); w!(ocb_per_pawn); w!(ocb_cap);
    w!(threat_attack_minor_mg); w!(threat_attack_minor_eg);
    w!(threat_rook_queen_mg); w!(threat_rook_queen_eg);
    w!(threat_push_mg); w!(threat_push_eg);
    w!(restricted_mobility_mg); w!(restricted_mobility_eg);
    w!(bishop_outpost_mg); w!(bishop_outpost_eg);
    w!(phalanx_mg[8]); w!(phalanx_eg[8]);
    std::fs::write(path, out)
}
