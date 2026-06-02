//! Texel tuner for Rarog evaluation weights.
//!
//! # Usage
//!
//! ```text
//! cargo run --release -p tune -- <dataset> [options]
//! ```
//!
//! ## Arguments
//!
//! | Argument | Default | Description |
//! |---|---|---|
//! | `<dataset>` | required | path to FEN dataset file |
//! | `--out <file>` | `tuned_params.txt` | write final params to this file |
//! | `--in <file>` | defaults | load starting params from this file |
//! | `--iters <n>` | 100 | number of full coordinate-descent passes |
//! | `--k <f>` | auto | fixed scaling constant K (auto-calibrates if 0) |
//! | `--step <n>` | 1 | step size for coordinate descent |
//!
//! ## Dataset format
//!
//! One position per line: `<FEN>;<result>`
//! where result is `1.0` (white wins), `0.5` (draw), or `0.0` (black wins).
//!
//! ```text
//! rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1;0.5
//! r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3;1.0
//! ```
//!
//! Positions are automatically filtered: only positions where the best move is
//! not a capture or promotion are used (quiet positions), to reduce tactical
//! noise in the eval signal.  You can skip this filter if your dataset is
//! already quiet-filtered.
//!
//! ## Algorithm
//!
//! 1. **K calibration**: golden-section search for the scaling constant K that
//!    minimises MSE over the dataset.  K converts engine centipawn scores to
//!    win probabilities via `sigmoid(K * eval / 400)`.
//!
//! 2. **Coordinate descent**: for each parameter, try value + step and value −
//!    step.  Keep the direction that reduces MSE; repeat until no improvement.
//!    Each complete pass over all parameters is one iteration.
//!
//! ## Expanding the eval coverage
//!
//! `tune_eval` in `src/tune.rs` currently evaluates material + PSQT + tempo.
//! To tune pawn-structure, mobility, king-safety and other EvalParams fields:
//! expand `tune_eval` to call the same logic as `Evaluator::eval_pawns` and
//! `eval_piece_activity` but with explicit `params` instead of global `PARAMS`.
//!
//! Alternatively, refactor `Evaluator::evaluate` into an
//! `evaluate_with(params: &EvalParams, board: &Board)` method (keep the public
//! `evaluate` delegating to it), and call that from the tuner.

use rarog::board::Board;
use rarog::eval::{EvalParams, PARAMS};
use rarog::tune::{load_eval_params, save_eval_params, tune_eval};

// ---------------------------------------------------------------------------
// Dataset
// ---------------------------------------------------------------------------

struct Position {
    board: Board,
    result: f64, // 1.0 = white wins, 0.5 = draw, 0.0 = black wins
}

fn load_dataset(path: &str) -> Vec<Position> {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("Cannot read dataset {path}: {e}"));
    let mut positions = Vec::new();
    for (ln, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (fen, result_str) = match line.split_once(';') {
            Some(parts) => parts,
            None => {
                eprintln!("line {}: missing ';', skipping: {line}", ln + 1);
                continue;
            }
        };
        let result: f64 = match result_str.trim().parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!(
                    "line {}: cannot parse result '{}', skipping",
                    ln + 1,
                    result_str.trim()
                );
                continue;
            }
        };
        match Board::from_fen(fen.trim()) {
            Ok(board) => positions.push(Position { board, result }),
            Err(e) => eprintln!("line {}: bad FEN — {e}, skipping", ln + 1),
        }
    }
    eprintln!("Loaded {} positions from {path}", positions.len());
    positions
}

// ---------------------------------------------------------------------------
// Objective
// ---------------------------------------------------------------------------

/// Win-probability sigmoid: P(white wins) = 1 / (1 + 10^(-K * eval / 400)).
#[inline]
fn sigmoid(k: f64, eval_cp: f64) -> f64 {
    1.0 / (1.0 + 10.0f64.powf(-k * eval_cp / 400.0))
}

/// Mean squared error of `sigmoid(K * eval)` vs game results.
fn mse(positions: &[Position], params: &EvalParams, k: f64) -> f64 {
    let mut total = 0.0f64;
    for pos in positions {
        let eval = tune_eval(&pos.board, params) as f64;
        let pred = sigmoid(k, eval);
        let diff = pred - pos.result;
        total += diff * diff;
    }
    total / positions.len() as f64
}

// ---------------------------------------------------------------------------
// K calibration
// ---------------------------------------------------------------------------

/// Golden-section search for the K in [lo, hi] minimising MSE.
fn calibrate_k(positions: &[Position], params: &EvalParams, lo: f64, hi: f64) -> f64 {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let mut a = lo;
    let mut b = hi;
    for _ in 0..64 {
        let m1 = b - (b - a) / phi;
        let m2 = a + (b - a) / phi;
        if mse(positions, params, m1) < mse(positions, params, m2) {
            b = m2;
        } else {
            a = m1;
        }
    }
    (a + b) / 2.0
}

// ---------------------------------------------------------------------------
// Coordinate descent
// ---------------------------------------------------------------------------

/// A single named scalar parameter: getter and setter.
struct ParamEntry {
    name: &'static str,
    get: fn(&EvalParams) -> i32,
    set: fn(&mut EvalParams, i32),
    /// Practical min / max bounds (soft — the tuner won't go outside these).
    min: i32,
    max: i32,
}

/// Build the list of all tunable scalar parameters.
///
/// Array parameters (passed_mg, mob_mg, etc.) are expanded into individual
/// per-element entries with names like `passed_mg_1`, `mob_mg_3`.
fn build_param_list() -> Vec<ParamEntry> {
    macro_rules! scalar {
        ($name:ident, $min:expr, $max:expr) => {
            ParamEntry {
                name: stringify!($name),
                get: |p| p.$name,
                set: |p, v| p.$name = v,
                min: $min,
                max: $max,
            }
        };
    }
    // For array elements we use a leaked string for the name.
    macro_rules! arr_elem {
        ($arr:ident, $idx:literal, $min:expr, $max:expr) => {
            ParamEntry {
                name: Box::leak(format!("{}_{}", stringify!($arr), $idx).into_boxed_str()),
                get: |p| p.$arr[$idx],
                set: |p, v| p.$arr[$idx] = v,
                min: $min,
                max: $max,
            }
        };
    }

    let mut v: Vec<ParamEntry> = Vec::new();

    // Tempo
    v.push(scalar!(tempo, 0, 30));

    // Pawn structure
    v.push(scalar!(passed_defended_mg, 0, 30));
    v.push(scalar!(passed_defended_eg_base, 0, 20));
    v.push(scalar!(passed_defended_eg_rank, 0, 12));
    v.push(scalar!(passed_free_mg_rank, 0, 10));
    v.push(scalar!(passed_free_eg_rank, 0, 15));
    v.push(scalar!(passed_free_safe_eg_rank, 0, 20));
    v.push(scalar!(candidate_mg, 0, 20));
    v.push(scalar!(candidate_eg, 0, 20));
    v.push(scalar!(doubled_mg, 0, 30));
    v.push(scalar!(doubled_eg, 0, 40));
    v.push(scalar!(isolated_mg, 0, 30));
    v.push(scalar!(isolated_eg, 0, 35));
    v.push(scalar!(defended_mg, 0, 20));
    v.push(scalar!(defended_eg, 0, 15));
    v.push(scalar!(backward_mg, 0, 25));
    v.push(scalar!(backward_eg, 0, 30));

    // Passed pawn rank bonuses (ranks 1..6; ranks 0 and 7 are always 0)
    v.push(arr_elem!(passed_mg, 1, 0, 150));
    v.push(arr_elem!(passed_mg, 2, 0, 150));
    v.push(arr_elem!(passed_mg, 3, 0, 150));
    v.push(arr_elem!(passed_mg, 4, 0, 150));
    v.push(arr_elem!(passed_mg, 5, 0, 150));
    v.push(arr_elem!(passed_mg, 6, 0, 150));
    v.push(arr_elem!(passed_eg, 1, 0, 250));
    v.push(arr_elem!(passed_eg, 2, 0, 250));
    v.push(arr_elem!(passed_eg, 3, 0, 250));
    v.push(arr_elem!(passed_eg, 4, 0, 250));
    v.push(arr_elem!(passed_eg, 5, 0, 250));
    v.push(arr_elem!(passed_eg, 6, 0, 250));

    // Piece bonuses
    v.push(scalar!(bishop_pair_mg, 10, 60));
    v.push(scalar!(bishop_pair_eg, 20, 80));
    v.push(scalar!(rook_open_mg, 5, 50));
    v.push(scalar!(rook_open_eg, 0, 30));
    v.push(scalar!(rook_semi_mg, 0, 30));
    v.push(scalar!(rook_semi_eg, 0, 20));
    v.push(scalar!(rook_seventh_mg, 5, 40));
    v.push(scalar!(rook_seventh_eg, 10, 60));
    v.push(scalar!(knight_outpost_mg, 5, 45));
    v.push(scalar!(knight_outpost_eg, 0, 30));

    // Mobility (pieces N=1, B=2, R=3, Q=4; skip Pawn=0 and King=5)
    v.push(arr_elem!(mob_mg, 1, 0, 10));
    v.push(arr_elem!(mob_mg, 2, 0, 10));
    v.push(arr_elem!(mob_mg, 3, 0, 10));
    v.push(arr_elem!(mob_mg, 4, 0, 10));
    v.push(arr_elem!(mob_eg, 1, 0, 12));
    v.push(arr_elem!(mob_eg, 2, 0, 12));
    v.push(arr_elem!(mob_eg, 3, 0, 12));
    v.push(arr_elem!(mob_eg, 4, 0, 12));

    // Threats
    v.push(scalar!(threat_minor_mg, 5, 40));
    v.push(scalar!(threat_minor_eg, 0, 25));
    v.push(scalar!(threat_rook_mg, 10, 55));
    v.push(scalar!(threat_rook_eg, 5, 35));
    v.push(scalar!(threat_queen_mg, 20, 70));
    v.push(scalar!(threat_queen_eg, 10, 50));
    v.push(scalar!(threat_attack_minor_mg, 0, 30));
    v.push(scalar!(threat_attack_minor_eg, 0, 20));
    v.push(scalar!(threat_rook_queen_mg, 0, 35));
    v.push(scalar!(threat_rook_queen_eg, 0, 25));
    v.push(scalar!(threat_push_mg, 0, 20));
    v.push(scalar!(threat_push_eg, 0, 15));

    // King safety
    v.push(scalar!(ks_minor_weight, 1, 5));
    v.push(scalar!(ks_rook_weight, 1, 7));
    v.push(scalar!(ks_queen_weight, 3, 10));
    v.push(scalar!(ks_ring_attack, 0, 8));
    v.push(scalar!(ks_safe_check_queen, 10, 100));
    v.push(scalar!(ks_safe_check_rook, 5, 60));
    v.push(scalar!(ks_safe_check_bishop, 0, 40));
    v.push(scalar!(ks_safe_check_knight, 0, 40));
    v.push(scalar!(ks_no_queen, 0, 60));
    v.push(scalar!(ks_divisor, 4, 64));
    v.push(scalar!(ks_max_penalty, 100, 500));

    // Shelter / storm
    v.push(scalar!(shelter_open_king, 5, 40));
    v.push(scalar!(shelter_open_adj, 0, 25));
    v.push(scalar!(shelter_close1, 5, 30));
    v.push(scalar!(shelter_close2, 0, 20));
    v.push(scalar!(storm_king_file, 2, 15));
    v.push(scalar!(storm_adj_file, 1, 10));

    // Rook behind passers
    v.push(scalar!(rook_passer_mg, 5, 35));
    v.push(scalar!(rook_passer_eg, 10, 50));
    v.push(scalar!(enemy_rook_passer_mg, 0, 25));
    v.push(scalar!(enemy_rook_passer_eg, 5, 40));

    // Hanging pieces
    v.push(scalar!(hanging_minor, 20, 70));
    v.push(scalar!(hanging_rook, 30, 90));
    v.push(scalar!(hanging_queen, 40, 120));

    // King proximity / centralisation
    v.push(scalar!(king_prox_base, 1, 5));
    v.push(scalar!(king_push_weight, 2, 10));
    v.push(scalar!(king_prox_weight, 2, 8));

    // Misc
    v.push(scalar!(space, 1, 5));
    v.push(scalar!(trapped_bishop_mg, 20, 100));
    v.push(scalar!(trapped_bishop_eg, 10, 70));
    v.push(scalar!(restricted_mobility_mg, 0, 20));
    v.push(scalar!(restricted_mobility_eg, 0, 15));
    v.push(scalar!(bishop_outpost_mg, 0, 30));
    v.push(scalar!(bishop_outpost_eg, 0, 20));

    // Phalanx ranks 1..6 (rank 0 and 7 are always 0)
    v.push(arr_elem!(phalanx_mg, 1, 0, 40));
    v.push(arr_elem!(phalanx_mg, 2, 0, 40));
    v.push(arr_elem!(phalanx_mg, 3, 0, 40));
    v.push(arr_elem!(phalanx_mg, 4, 0, 40));
    v.push(arr_elem!(phalanx_mg, 5, 0, 40));
    v.push(arr_elem!(phalanx_mg, 6, 0, 40));
    v.push(arr_elem!(phalanx_eg, 1, 0, 40));
    v.push(arr_elem!(phalanx_eg, 2, 0, 40));
    v.push(arr_elem!(phalanx_eg, 3, 0, 40));
    v.push(arr_elem!(phalanx_eg, 4, 0, 40));
    v.push(arr_elem!(phalanx_eg, 5, 0, 40));
    v.push(arr_elem!(phalanx_eg, 6, 0, 40));

    v
}

fn run_coordinate_descent(
    positions: &[Position],
    mut params: EvalParams,
    k: f64,
    iters: usize,
    step: i32,
) -> EvalParams {
    let param_list = build_param_list();
    let mut current_mse = mse(positions, &params, k);
    eprintln!("Starting MSE: {current_mse:.8}  K: {k:.4}");

    for iter in 0..iters {
        let mut improved = 0usize;
        for entry in &param_list {
            let orig = (entry.get)(&params);

            // Try +step
            let v_up = (orig + step).clamp(entry.min, entry.max);
            if v_up != orig {
                let mut p = params;
                (entry.set)(&mut p, v_up);
                let m = mse(positions, &p, k);
                if m < current_mse {
                    current_mse = m;
                    params = p;
                    improved += 1;
                    continue;
                }
            }

            // Try -step
            let v_dn = (orig - step).clamp(entry.min, entry.max);
            if v_dn != orig {
                let mut p = params;
                (entry.set)(&mut p, v_dn);
                let m = mse(positions, &p, k);
                if m < current_mse {
                    current_mse = m;
                    params = p;
                    improved += 1;
                }
            }
        }
        eprintln!(
            "Iter {:3}  MSE: {current_mse:.8}  improved: {improved}/{} params",
            iter + 1,
            param_list.len()
        );
        if improved == 0 {
            eprintln!("No improvement — stopping early.");
            break;
        }
    }
    params
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Parse options
    let mut dataset_path: Option<String> = None;
    let mut out_path = "tuned_params.txt".to_string();
    let mut in_path: Option<String> = None;
    let mut iters: usize = 100;
    let mut k_fixed: f64 = 0.0;
    let mut step: i32 = 1;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                i += 1;
                out_path = args[i].clone();
            }
            "--in" => {
                i += 1;
                in_path = Some(args[i].clone());
            }
            "--iters" => {
                i += 1;
                iters = args[i].parse().expect("--iters requires a number");
            }
            "--k" => {
                i += 1;
                k_fixed = args[i].parse().expect("--k requires a float");
            }
            "--step" => {
                i += 1;
                step = args[i].parse().expect("--step requires an integer");
            }
            s if !s.starts_with("--") && dataset_path.is_none() => {
                dataset_path = Some(s.to_string());
            }
            s => {
                eprintln!("Unknown argument: {s}");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let dataset_path = dataset_path.unwrap_or_else(|| {
        eprintln!("Usage: tune <dataset> [--options]");
        std::process::exit(1);
    });

    // Load dataset
    let positions = load_dataset(&dataset_path);
    if positions.is_empty() {
        eprintln!("Empty dataset — aborting.");
        std::process::exit(1);
    }

    // Load starting params
    let mut params: EvalParams = if let Some(ref path) = in_path {
        load_eval_params(path)
    } else {
        // Use whatever PARAMS was initialised to (respects RAROG_TUNE_FILE)
        *PARAMS
    };

    // K calibration
    let k = if k_fixed > 0.0 {
        eprintln!("Using fixed K = {k_fixed}");
        k_fixed
    } else {
        eprint!("Calibrating K ... ");
        let k = calibrate_k(&positions, &params, 0.5, 10.0);
        eprintln!("K = {k:.6}");
        k
    };

    // Coordinate descent
    params = run_coordinate_descent(&positions, params, k, iters, step);

    // Write result
    save_eval_params(&params, &out_path)
        .unwrap_or_else(|e| eprintln!("Cannot write {out_path}: {e}"));
    eprintln!("Tuned parameters written to {out_path}");

    // Print diff vs defaults
    let defaults = EvalParams::DEFAULT;
    let param_list = build_param_list();
    let mut changed = 0usize;
    for entry in &param_list {
        let orig = (entry.get)(&defaults);
        let tuned = (entry.get)(&params);
        if tuned != orig {
            println!(
                "{:35} default={:5}  tuned={:5}  delta={:+}",
                entry.name,
                orig,
                tuned,
                tuned - orig
            );
            changed += 1;
        }
    }
    eprintln!(
        "{changed}/{} parameters changed from defaults.",
        param_list.len()
    );
}
