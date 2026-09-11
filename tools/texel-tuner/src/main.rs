//! Rarog Texel eval tuner (Phase 3.3).
//!
//! A faithful Rust port of `tools/texel/reference/basilisk_tuner.cpp`:
//! golden-section K-fit, full-batch Adam, staged group masks, the
//! `linear_delta_scale` that captures Rarog's frozen non-linear factors, the
//! reconstruction `--verify` gate, and the `name index value` output format
//! that the engine's `RAROG_EVAL_FILE` loader (Phase 3.2) reads back.
//!
//! Dataset format: one `FEN;target` per line; `target` is the White-POV
//! expected score (`1-0`/`0-1`/`1/2-1/2`, or a float in `[0,1]`). With
//! `--from-cp` (Phase 6.1 SF-distillation), `target` is instead a White-POV
//! centipawn integer (e.g. Hydra's `sf_train.csv`), squashed at load with
//! `1/(1+10^(-cp/400))`.
//!
//! Build/run (from repo root):
//!   cargo run --release -p texel-tuner -- --verify tools/texel/data/holdout.csv
//!   cargo run --release -p texel-tuner -- --tune material \
//!       tools/texel/data/train.csv tools/texel/data/holdout.csv out.txt

use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;

use rarog::board::{Board, Color, Piece};
use rarog::eval::{EVAL_PARAM_NAMES, EvalParams, Evaluator, linear_delta_scale};

// ---------------------------------------------------------------------------
// Flat-parameter / group helpers
// ---------------------------------------------------------------------------

/// (flat offset, length) of a named field in `EVAL_PARAM_NAMES` order.
fn field_offset(field: &str) -> (usize, usize) {
    let mut off = 0;
    for &(name, len) in EVAL_PARAM_NAMES {
        if name == field {
            return (off, len);
        }
        off += len;
    }
    panic!("unknown eval field '{field}'");
}

fn push_field(out: &mut Vec<usize>, field: &str) {
    let (off, len) = field_offset(field);
    out.extend(off..off + len);
}

fn push_field_indices(out: &mut Vec<usize>, field: &str, lo: usize, hi: usize) {
    let (off, _) = field_offset(field);
    out.extend(off + lo..off + hi);
}

const PAWNSTRUCT: &[&str] = &[
    "pawn_doubled_mg",
    "pawn_doubled_eg",
    "pawn_isolated_mg",
    "pawn_isolated_eg",
    "pawn_connected_mg",
    "pawn_connected_eg",
    "pawn_backward_mg",
    "pawn_backward_eg",
    // Phase 3.8 additions.
    "pawn_lever_mg",
    "pawn_lever_eg",
    "pawn_doubled_isolated_mg",
    "pawn_doubled_isolated_eg",
    // Phase 7.4: same-rank phalanx (seeded 0, activated by this refit).
    "pawn_phalanx_mg",
    "pawn_phalanx_eg",
];
const PASSERS: &[&str] = &[
    "passed_mg",
    "passed_eg",
    "passed_supported_mg",
    "passed_supported_eg_base",
    "passed_supported_eg_per_rank",
    "passed_freestop_mg_per_rank",
    "passed_freestop_eg_per_rank",
    "passed_safestop_eg_per_rank",
    // Phase 6.2.1 whole-path weighting.
    "passed_freepath_mg_per_rank",
    "passed_freepath_eg_per_rank",
    "passed_safepath_eg_per_rank",
    "passed_candidate_mg",
    "passed_candidate_eg",
    "passer_proximity_base",
    // Phase 3.8 passer detail.
    "blocked_passer_mg",
    "blocked_passer_eg",
    "ideal_blockader_mg",
    "ideal_blockader_eg",
];
const ROOKS: &[&str] = &[
    "rook_open_mg",
    "rook_open_eg",
    "rook_semiopen_mg",
    "rook_semiopen_eg",
    "rook_7th_mg",
    "rook_7th_eg",
    "rook_behind_passer_mg",
    "rook_behind_passer_eg",
    "enemy_rook_behind_passer_mg",
    "enemy_rook_behind_passer_eg",
];
const MINORS: &[&str] = &[
    "bishop_pair_mg",
    "bishop_pair_eg",
    "knight_outpost_mg",
    "knight_outpost_eg",
    "trapped_bishop_mg",
    "trapped_bishop_eg",
];
const MOBILITY: &[&str] = &[
    "mob_n_mg", "mob_n_eg", "mob_b_mg", "mob_b_eg", "mob_r_mg", "mob_r_eg", "mob_q_mg", "mob_q_eg",
];
const THREATS: &[&str] = &[
    "threat_minor_mg",
    "threat_minor_eg",
    "threat_rook_mg",
    "threat_rook_eg",
    "threat_queen_mg",
    "threat_queen_eg",
    // Phase 3.6 threats v2 (per-victim and count terms; all Texel-tunable).
    "threat_by_minor_mg",
    "threat_by_minor_eg",
    "threat_by_rook_mg",
    "threat_by_rook_eg",
    "threat_hanging_refined_mg",
    "threat_hanging_refined_eg",
    "threat_safe_pawn_push_mg",
    "threat_safe_pawn_push_eg",
    "threat_weak_piece_mg",
    "threat_weak_piece_eg",
    "threat_restricted_mg",
    "threat_restricted_eg",
];
const HANGING: &[&str] = &["hanging_minor", "hanging_rook", "hanging_queen"];
const MISC: &[&str] = &["passer_proximity_base", "space_weight", "tempo"];
const IMBALANCE: &[&str] = &["imbalance_ours", "imbalance_theirs"];
// Phase 3.10 small positional terms.
const SMALLPOS: &[&str] = &[
    "bishop_pair_pawn_mg",
    "bishop_pair_pawn_eg",
    "bishop_outpost_mg",
    "bishop_outpost_eg",
    "rook_trapped_mg",
    "rook_trapped_eg",
    "rook_connected_mg",
    "rook_connected_eg",
    "bishop_long_diagonal_mg",
    "bishop_long_diagonal_eg",
    "bad_bishop_mg",
    "bad_bishop_eg",
    "initiative_weight",
    "closedness_knight_mg",
    "closedness_rook_mg",
    "king_centrality_danger_mg",
];
// Phase 3.12 gauntlet-driven additions.
const GAUNTLET: &[&str] = &[
    "unstoppable_passer_eg",
    "minor_behind_pawn_mg",
    "minor_behind_pawn_eg",
    "pawn_islands_mg",
    "pawn_islands_eg",
    "queen_infiltration_mg",
    "queen_infiltration_eg",
    "king_protector_mg",
    "king_protector_eg",
    "space_piece_mg",
    // Phase 6.2.1 refresh structure.
    "space_behind_piece_mg",
    "bishop_xray_pawns_mg",
    "bishop_xray_pawns_eg",
    "queen_battery_mg",
    "queen_battery_eg",
    "slider_on_queen_mg",
    "slider_on_queen_eg",
];
const KINGSAFETY: &[&str] = &[
    "king_safety_table",
    "shelter_missing_file_mg",
    "shelter_missing_adjacent_mg",
    "shelter_dist1_mg",
    "shelter_dist2_mg",
    "storm_file_weight",
    "storm_adjacent_weight",
];

/// Material = mg/eg values for pawn..queen (indices 0..=4; king index 5 has a
/// net-zero feature count and stays 0).
fn push_material(out: &mut Vec<usize>) {
    push_field_indices(out, "mg_val", 0, 5);
    push_field_indices(out, "eg_val", 0, 5);
}

/// One PST square per non-king piece and phase is pinned to remove the exact
/// material/PST gauge: adding C to a piece value and subtracting C from all 64
/// of its PST entries represents the same evaluator. Pinning square 0 retains
/// all 64 identifiable piece-square scores while keeping material interpretable.
fn pst_gauge_anchors() -> Vec<usize> {
    let mut out = Vec::with_capacity(10);
    for field in ["pst_mg", "pst_eg"] {
        let (off, _) = field_offset(field);
        for piece in 0..5 {
            out.push(off + piece * 64);
        }
    }
    out
}

fn active_indices_for_group(group: &str) -> Vec<usize> {
    let mut active = Vec::new();
    let push_all = |a: &mut Vec<usize>, fields: &[&str]| {
        for f in fields {
            push_field(a, f);
        }
    };
    let push_scalars = |a: &mut Vec<usize>| {
        push_all(a, PAWNSTRUCT);
        push_all(a, PASSERS);
        push_all(a, ROOKS);
        push_all(a, MINORS);
        push_all(a, MOBILITY);
        push_all(a, THREATS);
        push_all(a, HANGING);
        push_all(a, MISC);
        push_all(a, SMALLPOS);
        push_all(a, GAUNTLET);
    };
    match group {
        "material" => push_material(&mut active),
        "pawnstruct" | "pawns" => push_all(&mut active, PAWNSTRUCT),
        "passers" => push_all(&mut active, PASSERS),
        "rooks" => push_all(&mut active, ROOKS),
        "minors" => push_all(&mut active, MINORS),
        "mobility" => push_all(&mut active, MOBILITY),
        "threats" => push_all(&mut active, THREATS),
        // Phase 7.4: narrow affected-family refit for the HCE semantics bundle
        // — the pawn tables (support + new phalanx), passers, the rook-behind
        // pair, the attacked2-affected threats, and the square-rule scalar.
        // Nothing else (lesson 1: no all-parameter fit).
        "p74" => {
            push_all(&mut active, PAWNSTRUCT);
            push_all(&mut active, PASSERS);
            push_all(&mut active, ROOKS);
            push_all(&mut active, THREATS);
            push_field(&mut active, "unstoppable_passer_eg");
        }
        // Stage 4.4: the remaining positional scalars — pawn structure, passers,
        // rook files/7th, minors (bishop pair, outposts), space/tempo, small
        // positional terms, and the gauntlet additions. Excludes mobility /
        // threats / hanging (tuned in 4.2–4.3) and material/PST/imbalance (later
        // stages). Freezes the three feature-support sparse pairs (pawn_lever,
        // trapped_bishop, rook_trapped — too few observations to fit, Step 4.0).
        "scalars44" => {
            push_all(&mut active, PAWNSTRUCT);
            push_all(&mut active, PASSERS);
            push_all(&mut active, ROOKS);
            push_all(&mut active, MINORS);
            push_all(&mut active, MISC);
            push_all(&mut active, SMALLPOS);
            push_all(&mut active, GAUNTLET);
            let mut frozen = Vec::new();
            for f in [
                "pawn_lever_mg",
                "pawn_lever_eg",
                "trapped_bishop_mg",
                "trapped_bishop_eg",
                "rook_trapped_mg",
                "rook_trapped_eg",
            ] {
                push_field(&mut frozen, f);
            }
            active.retain(|i| !frozen.contains(i));
        }
        // Stage 4.2: threats + the old flat hanging term together, so the fit
        // resolves their overlap (the refined hanging term generalises the flat
        // one) — the data drives the flat penalty toward 0 rather than us
        // dropping it blind.
        "threats42" => {
            push_all(&mut active, THREATS);
            push_all(&mut active, HANGING);
        }
        "hanging" => push_all(&mut active, HANGING),
        "misc" => push_all(&mut active, MISC),
        "kingsafety" | "king" => push_all(&mut active, KINGSAFETY),
        "imbalance" => push_all(&mut active, IMBALANCE),
        "smallpos" => push_all(&mut active, SMALLPOS),
        "gauntlet" => push_all(&mut active, GAUNTLET),
        "scalars" => push_scalars(&mut active),
        "pst" => {
            push_material(&mut active);
            push_field(&mut active, "pst_mg");
            push_field(&mut active, "pst_eg");
        }
        "all" => {
            push_material(&mut active);
            push_field(&mut active, "pst_mg");
            push_field(&mut active, "pst_eg");
            push_scalars(&mut active);
            push_all(&mut active, KINGSAFETY);
            push_all(&mut active, IMBALANCE);
        }
        // Phase 4.8 complete existing-surface fit. This is `all` with the ten
        // exact material/PST gauge anchors removed. The two king material
        // values are invariant (both kings are always present), and the twelve
        // danger-index selectors use the nonlinear re-evaluation instrument.
        "complete" => {
            active = active_indices_for_group("all");
            let anchors = pst_gauge_anchors();
            active.retain(|i| !anchors.contains(i));
        }
        // Stage 4.7 global polish: everything linearly tunable, but the three
        // feature-support sparse pairs stay frozen (Step 4.0) — "everything
        // unfrozen" predates that audit. The nonlinear king-danger inputs are
        // not in any linear group anyway (fit via --tune-kingsafety in 4.1).
        "all47" => {
            push_material(&mut active);
            push_field(&mut active, "pst_mg");
            push_field(&mut active, "pst_eg");
            push_scalars(&mut active);
            push_all(&mut active, KINGSAFETY);
            push_all(&mut active, IMBALANCE);
            let mut frozen = Vec::new();
            for f in [
                "pawn_lever_mg",
                "pawn_lever_eg",
                "trapped_bishop_mg",
                "trapped_bishop_eg",
                "rook_trapped_mg",
                "rook_trapped_eg",
            ] {
                push_field(&mut frozen, f);
            }
            active.retain(|i| !frozen.contains(i));
        }
        _ => {
            eprintln!("Unknown tune group '{group}'.");
            print_groups();
            exit(1);
        }
    }
    active.sort_unstable();
    active.dedup();
    if active.is_empty() {
        eprintln!("Tune group '{group}' has no active params.");
        exit(1);
    }
    active
}

fn print_groups() {
    eprintln!(
        "Groups: material pawnstruct passers rooks minors mobility threats \
         threats42 hanging misc kingsafety imbalance smallpos gauntlet scalars scalars44 pst all complete"
    );
}

// ---------------------------------------------------------------------------
// Domain clamps (priors): applied after every Adam step so candidates stay
// sane (penalties non-negative magnitudes, bonuses bounded, passer/threat
// tables monotone). SPRT still decides whether a fit transfers.
// ---------------------------------------------------------------------------

fn clamp_field(w: &mut [f64], field: &str, lo: f64, hi: f64) {
    let (off, len) = field_offset(field);
    for x in &mut w[off..off + len] {
        *x = x.clamp(lo, hi);
    }
}

fn enforce_non_decreasing(w: &mut [f64], field: &str, first: usize, last: usize) {
    let (off, _) = field_offset(field);
    for i in first + 1..=last {
        if w[off + i] < w[off + i - 1] {
            w[off + i] = w[off + i - 1];
        }
    }
}

fn clamp_weights(w: &mut [f64]) {
    // Material: pawn..queen positive; king value pinned at 0.
    let (mg, _) = field_offset("mg_val");
    let (eg, _) = field_offset("eg_val");
    for pt in 0..5 {
        w[mg + pt] = w[mg + pt].clamp(1.0, 2000.0);
        w[eg + pt] = w[eg + pt].clamp(1.0, 2000.0);
    }
    w[mg + 5] = 0.0;
    w[eg + 5] = 0.0;

    // Pawn-structure penalties are stored as positive magnitudes (subtracted).
    for f in [
        "pawn_doubled_mg",
        "pawn_doubled_eg",
        "pawn_isolated_mg",
        "pawn_isolated_eg",
        "pawn_backward_mg",
        "pawn_backward_eg",
    ] {
        clamp_field(w, f, 0.0, 200.0);
    }
    clamp_field(w, "pawn_connected_mg", 0.0, 200.0);
    clamp_field(w, "pawn_connected_eg", 0.0, 200.0);
    // Phase 7.4: phalanx bonus (same range as support).
    clamp_field(w, "pawn_phalanx_mg", 0.0, 200.0);
    clamp_field(w, "pawn_phalanx_eg", 0.0, 200.0);
    // Phase 3.8: lever bonus and doubled-isolated penalty magnitude.
    clamp_field(w, "pawn_lever_mg", 0.0, 100.0);
    clamp_field(w, "pawn_lever_eg", 0.0, 100.0);
    clamp_field(w, "pawn_doubled_isolated_mg", 0.0, 200.0);
    clamp_field(w, "pawn_doubled_isolated_eg", 0.0, 200.0);

    // Passers: rank 0 and 7 stay 0, middle ranks non-decreasing bonuses.
    for f in ["passed_mg", "passed_eg"] {
        let (off, _) = field_offset(f);
        w[off] = 0.0;
        w[off + 7] = 0.0;
        for x in &mut w[off + 1..off + 7] {
            *x = x.clamp(0.0, 400.0);
        }
        enforce_non_decreasing(w, f, 1, 6);
    }
    clamp_field(w, "passed_supported_mg", 0.0, 200.0);
    clamp_field(w, "passed_supported_eg_base", 0.0, 200.0);
    clamp_field(w, "passed_supported_eg_per_rank", 0.0, 50.0);
    clamp_field(w, "passed_freestop_mg_per_rank", 0.0, 100.0);
    clamp_field(w, "passed_freestop_eg_per_rank", 0.0, 100.0);
    clamp_field(w, "passed_safestop_eg_per_rank", 0.0, 100.0);
    clamp_field(w, "passed_candidate_mg", 0.0, 200.0);
    clamp_field(w, "passed_candidate_eg", 0.0, 200.0);
    clamp_field(w, "passer_proximity_base", 0.0, 50.0);
    // Phase 3.8: blocked-passer penalty magnitude, ideal-blockader bonus.
    clamp_field(w, "blocked_passer_mg", 0.0, 200.0);
    clamp_field(w, "blocked_passer_eg", 0.0, 200.0);
    clamp_field(w, "ideal_blockader_mg", 0.0, 200.0);
    clamp_field(w, "ideal_blockader_eg", 0.0, 200.0);

    for f in [
        "bishop_pair_mg",
        "bishop_pair_eg",
        "knight_outpost_mg",
        "knight_outpost_eg",
        "trapped_bishop_mg",
        "trapped_bishop_eg",
        "rook_open_mg",
        "rook_open_eg",
        "rook_semiopen_mg",
        "rook_semiopen_eg",
        "rook_7th_mg",
        "rook_7th_eg",
        "rook_behind_passer_mg",
        "rook_behind_passer_eg",
        "enemy_rook_behind_passer_mg",
        "enemy_rook_behind_passer_eg",
    ] {
        clamp_field(w, f, 0.0, 200.0);
    }

    // Per-count mobility tables: bounded, and non-decreasing in the count
    // (more safe squares should not score worse). Index 0 may go negative
    // (trapped piece), like SF's low-mobility entries.
    for f in [
        "mob_n_mg", "mob_n_eg", "mob_b_mg", "mob_b_eg", "mob_r_mg", "mob_r_eg", "mob_q_mg",
        "mob_q_eg",
    ] {
        clamp_field(w, f, -150.0, 400.0);
        let (_, len) = field_offset(f);
        enforce_non_decreasing(w, f, 0, len - 1);
    }

    for f in THREATS {
        clamp_field(w, f, 0.0, 200.0);
    }
    // Threat magnitude grows with victim value.
    let (tmg, _) = field_offset("threat_minor_mg");
    let (trk, _) = field_offset("threat_rook_mg");
    let (tq, _) = field_offset("threat_queen_mg");
    w[trk] = w[trk].max(w[tmg]);
    w[tq] = w[tq].max(w[trk]);
    let (tmge, _) = field_offset("threat_minor_eg");
    let (trke, _) = field_offset("threat_rook_eg");
    let (tqe, _) = field_offset("threat_queen_eg");
    w[trke] = w[trke].max(w[tmge]);
    w[tqe] = w[tqe].max(w[trke]);

    // Hanging penalties grow with piece value.
    for f in HANGING {
        clamp_field(w, f, 0.0, 200.0);
    }
    let (hm, _) = field_offset("hanging_minor");
    let (hr, _) = field_offset("hanging_rook");
    let (hq, _) = field_offset("hanging_queen");
    w[hr] = w[hr].max(w[hm]);
    w[hq] = w[hq].max(w[hr]);

    clamp_field(w, "space_weight", 0.0, 50.0);
    clamp_field(w, "tempo", 0.0, 50.0);

    // King safety: a non-decreasing danger table, positive shelter/storm.
    clamp_field(w, "king_safety_table", 0.0, 600.0);
    let (kst, len) = field_offset("king_safety_table");
    let _ = kst;
    enforce_non_decreasing(w, "king_safety_table", 0, len - 1);
    for f in [
        "shelter_missing_file_mg",
        "shelter_missing_adjacent_mg",
        "shelter_dist1_mg",
        "shelter_dist2_mg",
        "storm_file_weight",
        "storm_adjacent_weight",
    ] {
        clamp_field(w, f, 0.0, 100.0);
    }
    // Nonlinear danger-index inputs (SPSA/finite-difference path, Phase 4.0).
    // All are danger *contributions* (more attack = more danger) or, for
    // queen_relief, a danger *reduction* stored as a positive magnitude — so
    // every one is bounded non-negative. The bucket index they feed is clamped
    // to the table length in eval, so generous upper bounds are safe.
    for f in [
        "king_safety_unit_minor",
        "king_safety_unit_rook",
        "king_safety_unit_queen",
        "ks_weak_ring",
        "ks_safe_check_knight",
        "ks_safe_check_bishop",
        "ks_safe_check_rook",
        "ks_safe_check_queen",
        "ks_flank_attack",
        "ks_pawnless_flank",
        "ks_shelter_storm",
        "ks_queen_relief",
    ] {
        clamp_field(w, f, 0.0, 20.0);
    }

    // Imbalance coefficients are signed; just bound the magnitude.
    clamp_field(w, "imbalance_ours", -300.0, 300.0);
    clamp_field(w, "imbalance_theirs", -300.0, 300.0);

    // Phase 3.10 small positional terms.
    clamp_field(w, "bishop_pair_pawn_mg", -20.0, 20.0);
    clamp_field(w, "bishop_pair_pawn_eg", -20.0, 20.0);
    for f in [
        "bishop_outpost_mg",
        "bishop_outpost_eg",
        "rook_trapped_mg",
        "rook_trapped_eg",
        "rook_connected_mg",
        "rook_connected_eg",
        "bishop_long_diagonal_mg",
        "bishop_long_diagonal_eg",
        "bad_bishop_mg",
        "bad_bishop_eg",
    ] {
        clamp_field(w, f, 0.0, 200.0);
    }
    clamp_field(w, "initiative_weight", 0.0, 30.0);
    // Closedness: knight swing expected positive, rook swing expected
    // negative as the centre locks; signed, just bound the magnitude.
    clamp_field(w, "closedness_knight_mg", -30.0, 30.0);
    clamp_field(w, "closedness_rook_mg", -30.0, 30.0);
    clamp_field(w, "king_centrality_danger_mg", 0.0, 100.0);

    // Phase 3.12 gauntlet additions.
    clamp_field(w, "unstoppable_passer_eg", 0.0, 600.0);
    for f in [
        "minor_behind_pawn_mg",
        "minor_behind_pawn_eg",
        "queen_infiltration_mg",
        "queen_infiltration_eg",
    ] {
        clamp_field(w, f, 0.0, 100.0);
    }
    // Penalty magnitudes (subtracted in the eval).
    clamp_field(w, "pawn_islands_mg", 0.0, 60.0);
    clamp_field(w, "pawn_islands_eg", 0.0, 60.0);
    // Per-distance-unit / per-product weights: summed over many units, so small.
    clamp_field(w, "king_protector_mg", 0.0, 30.0);
    clamp_field(w, "king_protector_eg", 0.0, 30.0);
    clamp_field(w, "space_piece_mg", 0.0, 20.0);
}

// ---------------------------------------------------------------------------
// Dataset records
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Holdout buckets (Phase 4.0) — a single global loss can fall while a critical
// eval domain silently regresses. We tag every position with the buckets it
// belongs to and report loss per bucket each fit; a bucket that regresses while
// global loss drops is the signal to investigate *before* the SPRT. Material
// classes come from the board; king-attack / passer / threat reuse the trace
// activation already computed for the row (a reporting layer, not new
// instrumentation).
const BUCKET_NAMES: &[&str] = &[
    "opening",     // phase >= 16
    "middlegame",  // 6 <= phase < 16
    "endgame",     // phase < 6
    "no-queens",   // both sides queenless
    "ocb",         // one bishop each, opposite colours, no knights
    "rook-ending", // rooks only (no queens, no minors)
    "pawn-ending", // kings + pawns only
    "king-attack", // king-safety table active (enemy pressure on a king)
    "passer",      // a passed pawn present
    "threat",      // a static threat present
];

/// Flat-index ranges of the trace families used for the king-attack / passer /
/// threat buckets, computed once from `EVAL_PARAM_NAMES`.
///
/// Each family is a LIST of ranges, not one min..max span. The span form was a
/// silent instrument failure: `passed*` is 13 fields totalling 27 slots but
/// they are **not contiguous**, so min..max covered [780,1211) = 431 slots, of
/// which 404 belonged to 113 other fields -- every pawn-structure term, the
/// threats, king safety, all of it. The bucket then fired whenever any of those
/// foreign slots was nonzero, which is nearly always: it selected 127,777 of
/// 127,778 positions and therefore measured nothing at all, while looking like
/// a working cohort. `threat` (48 slots) and `king_safety_table` (40) happen to
/// be contiguous, so those two buckets were correct -- which is exactly why the
/// bug survived: two thirds of the instrument worked.
struct FamilyRanges {
    passer: Vec<(usize, usize)>,
    threat: Vec<(usize, usize)>,
    ksafe: Vec<(usize, usize)>,
}

fn family_ranges() -> &'static FamilyRanges {
    static R: OnceLock<FamilyRanges> = OnceLock::new();
    R.get_or_init(|| {
        let (mut passer, mut threat, mut ksafe) = (Vec::new(), Vec::new(), Vec::new());
        let mut off = 0usize;
        for &(name, len) in EVAL_PARAM_NAMES {
            let end = off + len;
            if name.starts_with("passed") {
                passer.push((off, end));
            } else if name.starts_with("threat") {
                threat.push((off, end));
            } else if name == "king_safety_table" {
                ksafe.push((off, end));
            }
            off = end;
        }
        FamilyRanges {
            passer,
            threat,
            ksafe,
        }
    })
}

/// Bucket-membership bitmask for one position (bit i ⇔ `BUCKET_NAMES[i]`).
/// `coeffs` is the position's full linear trace (`trace.flat_coeffs()`).
fn position_buckets(board: &Board, phase: i32, coeffs: &[f64]) -> u32 {
    let mut m = 0u32;
    if phase >= 16 {
        m |= 1 << 0;
    } else if phase >= 6 {
        m |= 1 << 1;
    } else {
        m |= 1 << 2;
    }

    let cnt = |c: Color, p: Piece| board.pieces(c, p).count();
    let (wq, bq) = (
        cnt(Color::White, Piece::Queen),
        cnt(Color::Black, Piece::Queen),
    );
    let (wr, br) = (
        cnt(Color::White, Piece::Rook),
        cnt(Color::Black, Piece::Rook),
    );
    let (wn, bn) = (
        cnt(Color::White, Piece::Knight),
        cnt(Color::Black, Piece::Knight),
    );
    let (wb, bb) = (
        cnt(Color::White, Piece::Bishop),
        cnt(Color::Black, Piece::Bishop),
    );
    let queens = wq + bq;
    let rooks = wr + br;
    let minors = wn + bn + wb + bb;
    if queens == 0 {
        m |= 1 << 3;
    }
    if wb == 1 && bb == 1 && (wn + bn) == 0 {
        let sq_colour = |bbm: rarog::board::Bitboard| {
            let i = bbm.lsb().index();
            (i / 8 + i % 8) & 1
        };
        if sq_colour(board.pieces(Color::White, Piece::Bishop))
            != sq_colour(board.pieces(Color::Black, Piece::Bishop))
        {
            m |= 1 << 4;
        }
    }
    if queens == 0 && minors == 0 && rooks > 0 {
        m |= 1 << 5;
    }
    if queens == 0 && minors == 0 && rooks == 0 {
        m |= 1 << 6;
    }

    let fr = family_ranges();
    let any_nz = |ranges: &[(usize, usize)]| {
        ranges
            .iter()
            .any(|&(s, e)| s < e && coeffs[s..e].iter().any(|&c| c != 0.0))
    };
    if any_nz(&fr.ksafe) {
        m |= 1 << 7;
    }
    if any_nz(&fr.passer) {
        m |= 1 << 8;
    }
    if any_nz(&fr.threat) {
        m |= 1 << 9;
    }
    m
}

struct TuneSet {
    result: Vec<f32>,
    base_score: Vec<f32>,
    /// CSR row offsets plus parallel active-coordinate/value arrays. A complete
    /// 1,194-coordinate row is overwhelmingly zero; the old dense layout used
    /// about 11 GiB for the 2.30M-position corpus and multiplied every zero in
    /// every epoch. `u16` is sufficient because FLAT_SIZE is 1,218.
    row_offsets: Vec<usize>,
    coeff_indices: Vec<u16>,
    coeff_values: Vec<f32>,
    /// Per-position bucket bitmask (see `BUCKET_NAMES`).
    buckets: Vec<u32>,
}

impl TuneSet {
    fn len(&self) -> usize {
        self.result.len()
    }
    fn row(&self, i: usize) -> (&[u16], &[f32]) {
        let start = self.row_offsets[i];
        let end = self.row_offsets[i + 1];
        (
            &self.coeff_indices[start..end],
            &self.coeff_values[start..end],
        )
    }

    fn print_storage(&self, label: &str) {
        let nnz = self.coeff_values.len();
        let bytes = nnz * (std::mem::size_of::<u16>() + std::mem::size_of::<f32>())
            + self.row_offsets.len() * std::mem::size_of::<usize>();
        println!(
            "  {label}: {nnz} nonzero coefficients ({:.1}/position), {:.2} GiB CSR",
            nnz as f64 / self.len() as f64,
            bytes as f64 / 1024.0f64.powi(3)
        );
    }
}

/// `--from-cp` mode (Phase 6.1 SF-distillation): targets are White-POV
/// centipawns (Hydra `annotate_sf.py` output), squashed to `[0,1]` at load.
static FROM_CP: AtomicBool = AtomicBool::new(false);

/// `--fix-k <v>` (Phase 6.1): pin the sigmoid K instead of golden-section
/// fitting it. Stored as f64 bits; 0 = unset (K=0 is never a valid pin).
static FIX_K: AtomicU64 = AtomicU64::new(0);

fn fixed_k() -> Option<f64> {
    let bits = FIX_K.load(Ordering::Relaxed);
    if bits == 0 {
        None
    } else {
        Some(f64::from_bits(bits))
    }
}

fn k_label(k: f64) -> String {
    if fixed_k().is_some() {
        format!("{k:.5} (fixed via --fix-k)")
    } else {
        format!("{k:.5} (fitted)")
    }
}

fn parse_target(text: &str) -> Option<f32> {
    if FROM_CP.load(Ordering::Relaxed) {
        // Raw centipawn label (already White-POV; Hydra clamps mates/limits to
        // ±2000, but clamp again defensively). Squash with the standard Texel
        // logistic at cp/400 — the same shape the tuner's own sigmoid uses.
        let cp: f64 = text.trim().parse().ok()?;
        let cp = cp.clamp(-2000.0, 2000.0);
        return Some((1.0 / (1.0 + 10f64.powf(-cp / 400.0))) as f32);
    }
    match text {
        "1-0" => return Some(1.0),
        "0-1" => return Some(0.0),
        "1/2-1/2" => return Some(0.5),
        _ => {}
    }
    let v: f32 = text.trim().parse().ok()?;
    if (0.0..=1.0).contains(&v) {
        Some(v)
    } else {
        None
    }
}

fn read_lines(path: &str) -> Vec<String> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Cannot read {path}: {e}");
        exit(1);
    });
    text.lines().map(str::to_string).collect()
}

/// Atomically consume a frozen test set immediately before its first read.
/// `create_new` makes a second run fail across processes and restarts rather
/// than quietly converting the test set into another validation set.
fn claim_frozen_test(marker: &str, test_path: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(marker)
        .unwrap_or_else(|e| {
            eprintln!("Cannot claim frozen test via {marker}: {e}");
            exit(1)
        });
    writeln!(file, "rarog-frozen-test-v1\ntest={test_path}").unwrap_or_else(|e| {
        eprintln!("Cannot record frozen-test claim in {marker}: {e}");
        exit(1)
    });
    file.sync_all().unwrap_or_else(|e| {
        eprintln!("Cannot persist frozen-test claim in {marker}: {e}");
        exit(1)
    });
    println!("Frozen test claimed atomically via {marker}");
}

/// Load a complete `name index value` vector. Production stage chaining must
/// never accept a partial file: an omitted field would silently fall back to
/// source defaults and discard an earlier fit.
fn load_eval_file(path: &str) -> EvalParams {
    let lines = read_lines(path);
    let mut params = EvalParams::default();
    let mut seen = vec![false; EvalParams::FLAT_SIZE];
    for (line_no, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() != 3 {
            eprintln!("{path}:{}: expected 'name index value'", line_no + 1);
            exit(1);
        }
        let name = parts[0];
        let Some(&(_, len)) = EVAL_PARAM_NAMES.iter().find(|&&(field, _)| field == name) else {
            eprintln!("{path}:{}: unknown eval field '{name}'", line_no + 1);
            exit(1);
        };
        let idx: usize = parts[1].parse().unwrap_or_else(|_| {
            eprintln!("{path}:{}: bad index '{}'", line_no + 1, parts[1]);
            exit(1)
        });
        if idx >= len {
            eprintln!("{path}:{}: {name}[{idx}] is outside 0..{len}", line_no + 1);
            exit(1);
        }
        let value: i32 = parts[2].parse().unwrap_or_else(|_| {
            eprintln!("{path}:{}: bad value '{}'", line_no + 1, parts[2]);
            exit(1)
        });
        let (off, _) = field_offset(name);
        let flat = off + idx;
        if seen[flat] {
            eprintln!("{path}:{}: duplicate {name}[{idx}]", line_no + 1);
            exit(1);
        }
        seen[flat] = true;
        params.set(name, idx, value);
    }
    let missing = seen.iter().filter(|&&present| !present).count();
    if missing != 0 {
        eprintln!(
            "{path}: incomplete eval vector: {missing}/{} slots missing",
            EvalParams::FLAT_SIZE
        );
        exit(1);
    }
    params
}

fn n_threads() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Evaluate one line into (result, base_score_white, coeff row, bucket mask)
/// for the given active indices. Returns None for blank/malformed lines.
struct TuneRow {
    result: f32,
    base_score: f32,
    indices: Vec<u16>,
    values: Vec<f32>,
    buckets: u32,
}

fn process_line(
    evaluator: &mut Evaluator,
    base_params: &EvalParams,
    active: &[usize],
    line: &str,
) -> Option<TuneRow> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let sep = line.rfind(';')?;
    let fen = &line[..sep];
    let result = parse_target(&line[sep + 1..])?;
    let board = Board::from_fen(fen).ok()?;

    let score = evaluator.evaluate(&board);
    let trace = evaluator.last_trace();
    let score_white = if board.side_to_move() == rarog::board::Color::White {
        score
    } else {
        -score
    };
    let recon = trace.reconstruct(base_params);
    let rest = score_white - recon;
    let base_white = recon + rest; // == score_white, kept explicit for clarity

    let scale = linear_delta_scale(&board) as f32;
    let coeffs = trace.flat_coeffs();
    let mut row_indices = Vec::new();
    let mut row_values = Vec::new();
    for (active_index, &flat_index) in active.iter().enumerate() {
        let value = coeffs[flat_index] as f32 * scale;
        if value != 0.0 {
            row_indices.push(active_index as u16);
            row_values.push(value);
        }
    }
    let buckets = position_buckets(&board, trace.phase, &coeffs);
    Some(TuneRow {
        result,
        base_score: base_white as f32,
        indices: row_indices,
        values: row_values,
        buckets,
    })
}

/// One worker chunk's contribution to the CSR dataset.
type ChunkColumns = (Vec<f32>, Vec<f32>, Vec<usize>, Vec<u16>, Vec<f32>, Vec<u32>);

fn load_tune_dataset(
    path: &str,
    active: &[usize],
    max_positions: usize,
    base_params: &EvalParams,
) -> TuneSet {
    let mut lines = read_lines(path);
    if max_positions > 0 && lines.len() > max_positions {
        lines.truncate(max_positions);
    }
    assert!(active.len() <= u16::MAX as usize);
    let threads = n_threads().min(lines.len().max(1));
    let chunk = lines.len().div_ceil(threads.max(1));

    let parts: Vec<ChunkColumns> = thread::scope(|s| {
        let handles: Vec<_> = lines
            .chunks(chunk.max(1))
            .map(|slice| {
                s.spawn(move || {
                    let mut evaluator = Evaluator::default();
                    evaluator.set_params(base_params.clone());
                    let mut res = Vec::new();
                    let mut base = Vec::new();
                    let mut offsets = vec![0usize];
                    let mut indices = Vec::new();
                    let mut values = Vec::new();
                    let mut bk = Vec::new();
                    for line in slice {
                        if let Some(row) = process_line(&mut evaluator, base_params, active, line) {
                            res.push(row.result);
                            base.push(row.base_score);
                            indices.extend_from_slice(&row.indices);
                            values.extend_from_slice(&row.values);
                            offsets.push(indices.len());
                            bk.push(row.buckets);
                        }
                    }
                    (res, base, offsets, indices, values, bk)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let mut set = TuneSet {
        result: Vec::new(),
        base_score: Vec::new(),
        row_offsets: vec![0],
        coeff_indices: Vec::new(),
        coeff_values: Vec::new(),
        buckets: Vec::new(),
    };
    for (res, base, offsets, indices, values, bk) in parts {
        let prior_nnz = set.coeff_indices.len();
        set.result.extend(res);
        set.base_score.extend(base);
        set.row_offsets
            .extend(offsets.into_iter().skip(1).map(|offset| prior_nnz + offset));
        set.coeff_indices.extend(indices);
        set.coeff_values.extend(values);
        set.buckets.extend(bk);
    }
    if set.len() == 0 {
        eprintln!("No positions loaded from {path}.");
        exit(1);
    }
    // Guard against a silently mis-read dataset: a cp-labelled file loaded
    // WITHOUT --from-cp keeps only rows whose cp happens to be 0 or 1 (they
    // parse as valid WDL floats) and drops everything else. A WDL file loaded
    // WITH --from-cp squashes 0..1 to ~0.500 (harmless but wrong). The first
    // case shows up as a huge rejected fraction — refuse to continue.
    let rejected = lines.len().saturating_sub(set.len());
    if rejected * 2 > lines.len() {
        eprintln!(
            "ERROR: {rejected}/{} lines of {path} were rejected. If the targets \
             are centipawns (e.g. Hydra sf_*.csv), pass --from-cp.",
            lines.len()
        );
        exit(1);
    }
    set
}

// ---------------------------------------------------------------------------
// Loss / K-fit
// ---------------------------------------------------------------------------

fn sigmoid(score: f64, k: f64) -> f64 {
    1.0 / (1.0 + (-k * score / 400.0).exp())
}

fn default_loss(set: &TuneSet, k: f64) -> f64 {
    let sum: f64 = set
        .base_score
        .iter()
        .zip(&set.result)
        .map(|(&s, &r)| {
            let d = r as f64 - sigmoid(s as f64, k);
            d * d
        })
        .sum();
    sum / set.len() as f64
}

fn score_from_weights(set: &TuneSet, i: usize, active: &[usize], base_w: &[f64], w: &[f64]) -> f64 {
    let mut score = set.base_score[i] as f64;
    let (indices, values) = set.row(i);
    for (&active_index, &value) in indices.iter().zip(values) {
        let idx = active[active_index as usize];
        score += value as f64 * (w[idx] - base_w[idx]);
    }
    score
}

fn traced_loss(set: &TuneSet, active: &[usize], base_w: &[f64], w: &[f64], k: f64) -> f64 {
    // Parallel reduction over positions.
    let threads = n_threads().min(set.len().max(1));
    let chunk = set.len().div_ceil(threads.max(1));
    let total: f64 = thread::scope(|s| {
        let handles: Vec<_> = (0..set.len())
            .step_by(chunk.max(1))
            .map(|start| {
                let end = (start + chunk).min(set.len());
                s.spawn(move || {
                    let mut acc = 0.0;
                    for i in start..end {
                        let sig = sigmoid(score_from_weights(set, i, active, base_w, w), k);
                        let d = set.result[i] as f64 - sig;
                        acc += d * d;
                    }
                    acc
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    });
    total / set.len() as f64
}

fn fit_k(set: &TuneSet) -> f64 {
    if let Some(k) = fixed_k() {
        return k;
    }
    let (mut lo, mut hi) = (0.5f64, 2.5f64);
    for _ in 0..50 {
        let m1 = lo + (hi - lo) / 3.0;
        let m2 = hi - (hi - lo) / 3.0;
        if default_loss(set, m1) < default_loss(set, m2) {
            hi = m2;
        } else {
            lo = m1;
        }
    }
    (lo + hi) / 2.0
}

/// Per-bucket (sum-squared-error, count) under weights `w`, in one parallel
/// pass over the set. A position contributes to every bucket whose bit it sets.
fn bucket_losses(
    set: &TuneSet,
    active: &[usize],
    base_w: &[f64],
    w: &[f64],
    k: f64,
) -> Vec<(f64, u64)> {
    let nb = BUCKET_NAMES.len();
    let threads = n_threads().min(set.len().max(1));
    let chunk = set.len().div_ceil(threads.max(1));
    let parts: Vec<Vec<(f64, u64)>> = thread::scope(|s| {
        let handles: Vec<_> = (0..set.len())
            .step_by(chunk.max(1))
            .map(|start| {
                let end = (start + chunk).min(set.len());
                s.spawn(move || {
                    let mut acc = vec![(0.0f64, 0u64); nb];
                    for i in start..end {
                        let sig = sigmoid(score_from_weights(set, i, active, base_w, w), k);
                        let d = set.result[i] as f64 - sig;
                        let d2 = d * d;
                        let mask = set.buckets[i];
                        for (b, a) in acc.iter_mut().enumerate() {
                            if mask & (1 << b) != 0 {
                                a.0 += d2;
                                a.1 += 1;
                            }
                        }
                    }
                    acc
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut total = vec![(0.0f64, 0u64); nb];
    for p in parts {
        for (t, x) in total.iter_mut().zip(p) {
            t.0 += x.0;
            t.1 += x.1;
        }
    }
    total
}

/// Print a baseline→final per-bucket loss table and flag any bucket that
/// regressed while the global fit was applied. `final_w == base_w` gives a
/// plain snapshot of the current eval.
fn report_bucket_table(set: &TuneSet, active: &[usize], base_w: &[f64], final_w: &[f64], k: f64) {
    let base = bucket_losses(set, active, base_w, base_w, k);
    let fin = bucket_losses(set, active, base_w, final_w, k);
    let changed = base_w
        .iter()
        .zip(final_w)
        .any(|(a, b)| (a - b).abs() > 1e-9);
    println!("\nPer-bucket loss:");
    if changed {
        println!(
            "{:<13} {:>9} {:>11} {:>11} {:>11}",
            "bucket", "n", "base", "final", "delta"
        );
    } else {
        println!("{:<13} {:>9} {:>11}", "bucket", "n", "loss");
    }
    for (b, &name) in BUCKET_NAMES.iter().enumerate() {
        let (bs, bn) = base[b];
        let (fs, _) = fin[b];
        if bn == 0 {
            println!("{name:<13} {:>9} {:>11}", 0, "-");
            continue;
        }
        let base_mean = bs / bn as f64;
        if changed {
            let fin_mean = fs / bn as f64;
            let delta = fin_mean - base_mean;
            let flag = if delta > 1e-7 { "  <-- REGRESSED" } else { "" };
            println!("{name:<13} {bn:>9} {base_mean:>11.7} {fin_mean:>11.7} {delta:>+11.7}{flag}");
        } else {
            println!("{name:<13} {bn:>9} {base_mean:>11.7}");
        }
    }
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

fn write_eval_file(path: &str, w: &[f64]) {
    let mut params = EvalParams::default();
    params.set_from_flat(w);
    let mut out = String::new();
    for &(name, len) in EVAL_PARAM_NAMES {
        for i in 0..len {
            out.push_str(&format!("{name} {i} {}\n", params.get(name, i)));
        }
    }
    if let Some(parent) = std::path::Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, out).unwrap_or_else(|e| {
        eprintln!("Cannot write {path}: {e}");
        exit(1);
    });
}

fn print_active_deltas(active: &[usize], base_w: &[f64], w: &[f64]) {
    // Build a flat (name, index) lookup once.
    let mut info: Vec<(&str, usize)> = Vec::with_capacity(EvalParams::FLAT_SIZE);
    for &(name, len) in EVAL_PARAM_NAMES {
        for i in 0..len {
            info.push((name, i));
        }
    }
    let changed: Vec<usize> = active
        .iter()
        .copied()
        .filter(|&idx| base_w[idx].round() as i32 != w[idx].round() as i32)
        .collect();
    println!(
        "\nActive parameter deltas: {} changed / {} active",
        changed.len(),
        active.len()
    );
    for (printed, &idx) in changed.iter().enumerate() {
        if printed >= 120 {
            println!("... {} more", changed.len() - printed);
            break;
        }
        let (name, i) = info[idx];
        let (o, n) = (base_w[idx].round() as i32, w[idx].round() as i32);
        println!("{name:<28} {i:>3}  {o:>5} -> {n:<5}  {:+}", n - o);
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

fn cmd_verify(path: &str, weights: Option<&str>) {
    const VERIFY_COUNT: usize = 10_000;
    println!("Loading up to {VERIFY_COUNT} positions from {path} ...");
    let mut lines = read_lines(path);
    lines.retain(|l| {
        let t = l.trim();
        !t.is_empty() && !t.starts_with('#')
    });
    lines.truncate(VERIFY_COUNT);

    let params = weights.map_or_else(EvalParams::default, load_eval_file);
    let mut evaluator = Evaluator::default();
    evaluator.set_params(params.clone());
    let mut checked = 0usize;
    let mut mismatches = 0usize;
    let mut max_err = 0i32;

    for line in &lines {
        let Some(sep) = line.rfind(';') else { continue };
        let fen = &line[..sep];
        let Ok(board) = Board::from_fen(fen) else {
            continue;
        };
        let _ = evaluator.evaluate(&board);
        let trace = evaluator.last_trace();
        // `trace.raw` is the independently accumulated linear tapered score.
        // Comparing reconstruction directly against it catches missing/wrong
        // feature counts. The old check derived `rest = score - recon` and
        // then compared `score == recon + rest`, which was true by definition.
        let recon = trace.reconstruct(&params);
        let err = (trace.raw - recon).abs();
        if err != 0 {
            mismatches += 1;
            max_err = max_err.max(err);
            if mismatches <= 5 {
                eprintln!(
                    "MISMATCH: trace.raw={} reconstruct={recon} fen={fen}",
                    trace.raw
                );
            }
        }
        checked += 1;
    }

    if checked == 0 {
        eprintln!("No positions loaded.");
        exit(1);
    }
    if mismatches == 0 {
        println!("PASS: all {checked} positions reconstruct exactly.");
    } else {
        println!("FAIL: {mismatches}/{checked} differ (max error {max_err}).");
        exit(1);
    }
}

struct TuneOpts {
    group: String,
    train: String,
    holdout: String,
    /// 9.6(a): optional FROZEN TEST set. Three data roles: train updates the
    /// weights, the holdout is the VALIDATION set (it selects K and the best
    /// epoch — i.e. it steers the fit), and this set selects NOTHING. It is
    /// loaded after training ends and read exactly once, so the number it
    /// reports is an unbiased estimate. The holdout's own report is
    /// optimistically biased by construction: it is the best of N epochs on
    /// the very data that picked them.
    test: Option<String>,
    /// Complete source vector used as the frozen-test comparator. This is
    /// deliberately separate from `initial`: the final polish starts from the
    /// preceding nonlinear stage, while the production question is whether the
    /// complete rounded candidate beats the source HCE that began the run.
    test_baseline: Option<String>,
    /// Persistent exclusive marker created at the exact moment the frozen test
    /// is first opened. Production fits use it to enforce one-shot use across
    /// separate invocations, not merely within one process.
    test_marker: Option<String>,
    /// Complete vector produced by the preceding stage. Omission means source
    /// defaults. Partial vectors are rejected so stage chaining cannot reset a
    /// family silently.
    initial: Option<String>,
    out: String,
    epochs: usize,
    lr: f64,
    max_positions: usize,
    /// L2-to-prior strength. The prior is the current default (hand-tuned)
    /// value, so this shrinks each active weight back toward where it started
    /// unless the data pulls it away — the standard guard against a broad fit
    /// learning implausible signs/magnitudes off thin signal. 0 disables it.
    l2: f64,
}

fn cmd_tune(opts: &TuneOpts) {
    const BETA1: f64 = 0.9;
    const BETA2: f64 = 0.999;
    const EPS: f64 = 1e-8;

    let active = active_indices_for_group(&opts.group);
    println!(
        "Tune group: {} ({} active params)",
        opts.group,
        active.len()
    );

    let base_params = opts
        .initial
        .as_deref()
        .map_or_else(EvalParams::default, load_eval_file);
    println!(
        "Initial vector: {}",
        opts.initial.as_deref().unwrap_or("source defaults")
    );
    println!("Loading train from {} ...", opts.train);
    let train = load_tune_dataset(&opts.train, &active, opts.max_positions, &base_params);
    println!("  {} train positions", train.len());
    train.print_storage("train storage");
    println!("Loading holdout from {} ...", opts.holdout);
    let holdout = load_tune_dataset(&opts.holdout, &active, opts.max_positions, &base_params);
    println!("  {} holdout positions", holdout.len());
    holdout.print_storage("holdout storage");

    let k = fit_k(&holdout);
    println!("K = {}", k_label(k));

    let base_w = base_params.to_flat();
    let mut w = base_w.clone();

    let mut best_w = w.clone();
    let mut best_holdout = traced_loss(&holdout, &active, &base_w, &w, k);
    let mut best_epoch = 0usize;
    println!(
        "Initial train  loss = {:.8}",
        traced_loss(&train, &active, &base_w, &w, k)
    );
    println!("Initial holdout loss = {best_holdout:.8}");

    let mut m = vec![0.0f64; active.len()];
    let mut v = vec![0.0f64; active.len()];
    let n = train.len() as f64;
    let threads = n_threads().min(train.len().max(1));
    let chunk = train.len().div_ceil(threads.max(1));

    for epoch in 1..=opts.epochs {
        // Full-batch gradient, parallel over position chunks.
        let grad: Vec<f64> = thread::scope(|s| {
            let handles: Vec<_> = (0..train.len())
                .step_by(chunk.max(1))
                .map(|start| {
                    let end = (start + chunk).min(train.len());
                    let active = &active;
                    let base_w = &base_w;
                    let w = &w;
                    let train = &train;
                    s.spawn(move || {
                        let mut g = vec![0.0f64; active.len()];
                        for i in start..end {
                            let score = score_from_weights(train, i, active, base_w, w);
                            let sig = sigmoid(score, k);
                            let err = train.result[i] as f64 - sig;
                            let dsig = sig * (1.0 - sig);
                            let coeff = -2.0 * err * dsig * (k / 400.0);
                            let (indices, values) = train.row(i);
                            for (&active_index, &value) in indices.iter().zip(values) {
                                g[active_index as usize] += coeff * value as f64;
                            }
                        }
                        g
                    })
                })
                .collect();
            let mut total = vec![0.0f64; active.len()];
            for h in handles {
                for (t, p) in total.iter_mut().zip(h.join().unwrap()) {
                    *t += p;
                }
            }
            total
        });

        let t = epoch as f64;
        let bc1 = 1.0 - BETA1.powf(t);
        let bc2 = 1.0 - BETA2.powf(t);
        for (j, &idx) in active.iter().enumerate() {
            // Data gradient + L2-to-prior pull (∂/∂w of λ·(w−prior)² = 2λ(w−prior)).
            let g = grad[j] / n + 2.0 * opts.l2 * (w[idx] - base_w[idx]);
            m[j] = BETA1 * m[j] + (1.0 - BETA1) * g;
            v[j] = BETA2 * v[j] + (1.0 - BETA2) * g * g;
            let m_hat = m[j] / bc1;
            let v_hat = v[j] / bc2;
            w[idx] -= opts.lr * m_hat / (v_hat.sqrt() + EPS);
        }
        clamp_weights(&mut w);

        let holdout_loss = traced_loss(&holdout, &active, &base_w, &w, k);
        if holdout_loss < best_holdout {
            best_holdout = holdout_loss;
            best_epoch = epoch;
            best_w.copy_from_slice(&w);
        }
        if epoch == 1 || epoch % 10 == 0 || epoch == opts.epochs {
            let train_loss = traced_loss(&train, &active, &base_w, &w, k);
            println!("Epoch {epoch:>4}  train={train_loss:.8}  holdout={holdout_loss:.8}");
        }
    }

    w.copy_from_slice(&best_w);
    println!("Best validation epoch {best_epoch} (validation={best_holdout:.8}).");
    println!("NOTE: the validation loss above SELECTED the epoch, so it is an");
    println!("optimistic estimate. Pass --test <frozen.csv> for an unbiased one.");
    report_bucket_table(&holdout, &active, &base_w, &w, k);

    // Persist and reload before any final evidence. `EvalParams` is integer;
    // the optimizer is floating-point. Measuring `w` here would report a model
    // that can never be baked into the engine.
    write_eval_file(&opts.out, &w);
    let persisted = load_eval_file(&opts.out);
    let persisted_w = persisted.to_flat();
    let persisted_holdout = traced_loss(&holdout, &active, &base_w, &persisted_w, k);
    println!("Persisted rounded validation loss = {persisted_holdout:.8}");

    report_frozen_test(opts, k);

    print_active_deltas(&active, &base_w, &w);
    println!("Tuned weights written to {}", opts.out);
}

// ---------------------------------------------------------------------------
// Nonlinear king-safety fit (Phase 4.0)
// ---------------------------------------------------------------------------

/// The danger-index inputs that select the (non-linear) safety-table bucket.
/// They are invisible to the linear trace — a perturbation moves the table
/// *index*, not a coefficient — so they are fit here by re-evaluating positions
/// with perturbed weights instead of through the linear gradient.
const KS_DANGER_INPUTS: &[&str] = &[
    "king_safety_unit_minor",
    "king_safety_unit_rook",
    "king_safety_unit_queen",
    "ks_weak_ring",
    "ks_safe_check_knight",
    "ks_safe_check_bishop",
    "ks_safe_check_rook",
    "ks_safe_check_queen",
    "ks_flank_attack",
    "ks_pawnless_flank",
    // Phase 6.2.1: shelter/storm pawn-cover deficit folded into the danger index.
    "ks_shelter_storm",
    "ks_queen_relief",
];

/// Active flat indices for the king-safety fit: the 12 danger-index inputs plus
/// the 40-entry safety table they index into. The table is co-tuned because its
/// shape only makes sense against the index distribution the inputs produce.
///
/// The count said 11 until 2026-09-01; `ks_shelter_storm` was folded into the
/// danger index at Phase 6.2.1 and the comment was not updated. `KS_DANGER_INPUTS`
/// is the authority and has 12 entries, which is what 4.7.3's 1,194 + 12 + 10 + 2
/// partition of FLAT_SIZE counts.
fn ks_active_indices() -> Vec<usize> {
    let mut a = Vec::new();
    for f in KS_DANGER_INPUTS {
        push_field(&mut a, f);
    }
    push_field(&mut a, "king_safety_table");
    a
}

type RawPos = (Board, f32, u32);

/// Load (board, result, bucket-mask) triples. One eval per position records the
/// bucket mask (king-attack / passer / threat reuse the trace activation).
fn load_raw_dataset(path: &str, max_positions: usize, base_params: &EvalParams) -> Vec<RawPos> {
    let mut lines = read_lines(path);
    if max_positions > 0 && lines.len() > max_positions {
        lines.truncate(max_positions);
    }
    let threads = n_threads().min(lines.len().max(1));
    let chunk = lines.len().div_ceil(threads.max(1));
    let parts: Vec<Vec<RawPos>> = thread::scope(|s| {
        let handles: Vec<_> = lines
            .chunks(chunk.max(1))
            .map(|slice| {
                s.spawn(move || {
                    let mut ev = Evaluator::default();
                    ev.set_params(base_params.clone());
                    let mut out = Vec::new();
                    for line in slice {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        let Some(sep) = line.rfind(';') else { continue };
                        let Some(r) = parse_target(&line[sep + 1..]) else {
                            continue;
                        };
                        let Ok(board) = Board::from_fen(&line[..sep]) else {
                            continue;
                        };
                        let _ = ev.evaluate(&board);
                        let tr = ev.last_trace();
                        let mask = position_buckets(&board, tr.phase, &tr.flat_coeffs());
                        out.push((board, r, mask));
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all = Vec::new();
    for p in parts {
        all.extend(p);
    }
    if all.is_empty() {
        eprintln!("No positions loaded from {path}.");
        exit(1);
    }
    // Same silent-misread guard as load_tune_dataset (see comment there).
    let rejected = lines.len().saturating_sub(all.len());
    if rejected * 2 > lines.len() {
        eprintln!(
            "ERROR: {rejected}/{} lines of {path} were rejected. If the targets \
             are centipawns (e.g. Hydra sf_*.csv), pass --from-cp.",
            lines.len()
        );
        exit(1);
    }
    all
}

#[inline]
fn eval_white(ev: &mut Evaluator, board: &Board) -> f64 {
    let sc = ev.evaluate(board);
    if board.side_to_move() == Color::White {
        sc as f64
    } else {
        -(sc as f64)
    }
}

/// Re-evaluate the whole set with `params` (reusing the evaluator pool) and
/// return the Texel MSE at scaling `k`. Each pool evaluator handles a disjoint
/// board range, so there is no sharing.
fn ks_mse(boards: &[RawPos], evs: &mut [Evaluator], params: &EvalParams, k: f64) -> f64 {
    for e in evs.iter_mut() {
        e.set_params(params.clone());
    }
    let n = boards.len();
    let t = evs.len();
    let chunk = n.div_ceil(t.max(1));
    let total: f64 = thread::scope(|s| {
        let mut handles = Vec::new();
        for (ti, e) in evs.iter_mut().enumerate() {
            let start = ti * chunk;
            let end = ((ti + 1) * chunk).min(n);
            if start >= end {
                continue;
            }
            let slice = &boards[start..end];
            handles.push(s.spawn(move || {
                let mut acc = 0.0;
                for (b, r, _) in slice {
                    let d = *r as f64 - sigmoid(eval_white(e, b), k);
                    acc += d * d;
                }
                acc
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    });
    total / n as f64
}

/// Per-bucket re-eval MSE under `params` (for the final base→final table).
fn ks_bucket_losses(
    boards: &[RawPos],
    evs: &mut [Evaluator],
    params: &EvalParams,
    k: f64,
) -> Vec<(f64, u64)> {
    for e in evs.iter_mut() {
        e.set_params(params.clone());
    }
    let nb = BUCKET_NAMES.len();
    let n = boards.len();
    let t = evs.len();
    let chunk = n.div_ceil(t.max(1));
    let parts: Vec<Vec<(f64, u64)>> = thread::scope(|s| {
        let mut handles = Vec::new();
        for (ti, e) in evs.iter_mut().enumerate() {
            let start = ti * chunk;
            let end = ((ti + 1) * chunk).min(n);
            if start >= end {
                continue;
            }
            let slice = &boards[start..end];
            handles.push(s.spawn(move || {
                let mut acc = vec![(0.0f64, 0u64); nb];
                for (b, r, mask) in slice {
                    let d = *r as f64 - sigmoid(eval_white(e, b), k);
                    let d2 = d * d;
                    for (bi, a) in acc.iter_mut().enumerate() {
                        if mask & (1 << bi) != 0 {
                            a.0 += d2;
                            a.1 += 1;
                        }
                    }
                }
                acc
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut total = vec![(0.0f64, 0u64); nb];
    for p in parts {
        for (tot, x) in total.iter_mut().zip(p) {
            tot.0 += x.0;
            tot.1 += x.1;
        }
    }
    total
}

fn ks_report_buckets(base: &[(f64, u64)], fin: &[(f64, u64)]) {
    println!("\nPer-bucket loss:");
    println!(
        "{:<13} {:>9} {:>11} {:>11} {:>11}",
        "bucket", "n", "base", "final", "delta"
    );
    for (b, &name) in BUCKET_NAMES.iter().enumerate() {
        let (bs, bn) = base[b];
        if bn == 0 {
            println!("{name:<13} {:>9} {:>11}", 0, "-");
            continue;
        }
        let bm = bs / bn as f64;
        let fm = fin[b].0 / bn as f64;
        let delta = fm - bm;
        let flag = if delta > 1e-7 { "  <-- REGRESSED" } else { "" };
        println!("{name:<13} {bn:>9} {bm:>11.7} {fm:>11.7} {delta:>+11.7}{flag}");
    }
}

/// Compare the exact persisted integer candidate with the explicit source
/// vector. The raw evaluator is required: the source and candidate can differ
/// in nonlinear danger-index coordinates, which a linear trace based on the
/// preceding stage cannot reconstruct.
fn report_frozen_test(opts: &TuneOpts, k: f64) {
    let Some(test_path) = &opts.test else {
        return;
    };
    let baseline_path = opts
        .test_baseline
        .as_deref()
        .expect("validated --test-baseline must accompany --test");
    // Validate both vectors before consuming the one-shot marker. A corrupt
    // artifact must fail without burning the test set.
    let baseline = load_eval_file(baseline_path);
    let candidate = load_eval_file(&opts.out);
    if let Some(marker) = &opts.test_marker {
        claim_frozen_test(marker, test_path);
    }
    println!(
        "\nLoading FROZEN TEST set from {test_path} ...\n\
         Exact comparison: source vector {baseline_path}\n\
         Candidate: persisted rounded vector {}",
        opts.out
    );
    // The baseline defines cohort membership, so parameter movement cannot
    // silently move positions between the base and final columns.
    let test = load_raw_dataset(test_path, opts.max_positions, &baseline);
    println!("  {} test positions", test.len());
    let mut evs: Vec<Evaluator> = (0..n_threads()).map(|_| Evaluator::default()).collect();
    let base_test = ks_mse(&test, &mut evs, &baseline, k);
    let test_loss = ks_mse(&test, &mut evs, &candidate, k);
    println!(
        "Frozen test loss = {test_loss:.8} (source baseline {base_test:.8}, delta {:+.8})",
        test_loss - base_test
    );
    println!("(one-shot exact rounded report; this set selected nothing)");
    let base_buckets = ks_bucket_losses(&test, &mut evs, &baseline, k);
    let final_buckets = ks_bucket_losses(&test, &mut evs, &candidate, k);
    ks_report_buckets(&base_buckets, &final_buckets);
}

fn cmd_compare_frozen(test_path: &str, baseline_path: &str, candidate_path: &str, marker: &str) {
    let k = fixed_k().unwrap_or_else(|| {
        eprintln!("--compare-frozen requires --fix-k K.");
        exit(1)
    });
    // Reuse the same exact path as production tuning without constructing a
    // fake optimization stage or reading the confirmation set as train data.
    let opts = TuneOpts {
        group: "confirmation-only".to_string(),
        train: String::new(),
        holdout: String::new(),
        test: Some(test_path.to_string()),
        test_baseline: Some(baseline_path.to_string()),
        test_marker: Some(marker.to_string()),
        initial: None,
        out: candidate_path.to_string(),
        epochs: 1,
        lr: 0.0,
        max_positions: 0,
        l2: 0.0,
    };
    report_frozen_test(&opts, k);
}

#[derive(Default)]
struct EndgameClassStats {
    positions: u64,
    loss: f64,
    predicted: f64,
    result: f64,
    draw_positions: u64,
    draw_predicted: f64,
}

fn material_side(board: &Board, color: Color) -> (String, [u32; 6]) {
    let counts = [
        board.pieces(color, Piece::Queen).count(),
        board.pieces(color, Piece::Rook).count(),
        board.pieces(color, Piece::Bishop).count(),
        board.pieces(color, Piece::Knight).count(),
        board.pieces(color, Piece::Pawn).count(),
    ];
    let mut name = String::from("K");
    for (count, symbol) in counts.iter().zip(['Q', 'R', 'B', 'N', 'P']) {
        for _ in 0..*count {
            name.push(symbol);
        }
    }
    let key = [
        counts[0] * 9 + counts[1] * 5 + (counts[2] + counts[3]) * 3 + counts[4],
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        counts[4],
    ];
    (name, key)
}

fn endgame_class(board: &Board) -> Option<(String, Color)> {
    let (white, white_key) = material_side(board, Color::White);
    let (black, black_key) = material_side(board, Color::Black);
    let pieces = white_key[1..].iter().sum::<u32>() + black_key[1..].iter().sum::<u32>() + 2;
    if pieces > 7 {
        return None;
    }
    let white_strong = white_key >= black_key;
    let (strong, weak, color) = if white_strong {
        (white, black, Color::White)
    } else {
        (black, white, Color::Black)
    };
    Some((format!("{strong}-{weak}"), color))
}

fn cmd_report_endgames(path: &str, weights_path: &str) {
    let k = fixed_k().unwrap_or_else(|| {
        eprintln!("--report-endgames requires --fix-k K.");
        exit(1)
    });
    let params = load_eval_file(weights_path);
    let boards = load_raw_dataset(path, 0, &params);
    let mut evaluator = Evaluator::default();
    evaluator.set_params(params);
    let mut classes: HashMap<String, EndgameClassStats> = HashMap::new();
    let mut global_loss = 0.0;
    for (board, result_white, _) in &boards {
        let score_white = eval_white(&mut evaluator, board);
        let predicted_white = sigmoid(score_white, k);
        let error = *result_white as f64 - predicted_white;
        global_loss += error * error;
        let Some((name, strong)) = endgame_class(board) else {
            continue;
        };
        let (predicted, result) = if strong == Color::White {
            (predicted_white, *result_white as f64)
        } else {
            (1.0 - predicted_white, 1.0 - *result_white as f64)
        };
        let stats = classes.entry(name).or_default();
        stats.positions += 1;
        stats.loss += error * error;
        stats.predicted += predicted;
        stats.result += result;
        if (*result_white - 0.5).abs() < f32::EPSILON {
            stats.draw_positions += 1;
            stats.draw_predicted += predicted;
        }
    }
    let global = global_loss / boards.len() as f64;
    let mut rows: Vec<_> = classes.into_iter().collect();
    rows.sort_by_key(|row| std::cmp::Reverse(row.1.positions));
    println!(
        "# endgame material report: {} positions, K={k:.8}, global loss={global:.8}",
        boards.len()
    );
    println!(
        "class,n,loss,loss_vs_global,mean_predicted,mean_result,draw_n,draw_predicted,draw_bias"
    );
    for (name, stats) in rows {
        let n = stats.positions as f64;
        let draw_predicted = if stats.draw_positions == 0 {
            f64::NAN
        } else {
            stats.draw_predicted / stats.draw_positions as f64
        };
        println!(
            "{name},{},{:.8},{:.4},{:.6},{:.6},{},{:.6},{:+.6}",
            stats.positions,
            stats.loss / n,
            (stats.loss / n) / global,
            stats.predicted / n,
            stats.result / n,
            stats.draw_positions,
            draw_predicted,
            draw_predicted - 0.5,
        );
    }
}

/// Fit K once from base-parameter scores (ternary search), so the K-fit does
/// not re-evaluate the dataset per iteration.
fn ks_fit_k(boards: &[RawPos], evs: &mut [Evaluator], base: &EvalParams) -> f64 {
    if let Some(k) = fixed_k() {
        return k;
    }
    for e in evs.iter_mut() {
        e.set_params(base.clone());
    }
    let n = boards.len();
    let t = evs.len();
    let chunk = n.div_ceil(t.max(1));
    let scores: Vec<(f64, f32)> = thread::scope(|s| {
        let mut handles = Vec::new();
        for (ti, e) in evs.iter_mut().enumerate() {
            let start = ti * chunk;
            let end = ((ti + 1) * chunk).min(n);
            if start >= end {
                continue;
            }
            let slice = &boards[start..end];
            handles.push(s.spawn(move || {
                let mut v = Vec::with_capacity(slice.len());
                for (b, r, _) in slice {
                    v.push((eval_white(e, b), *r));
                }
                v
            }));
        }
        let mut all = Vec::new();
        for h in handles {
            all.extend(h.join().unwrap());
        }
        all
    });
    let loss = |k: f64| -> f64 {
        scores
            .iter()
            .map(|(sc, r)| {
                let d = *r as f64 - sigmoid(*sc, k);
                d * d
            })
            .sum::<f64>()
            / scores.len() as f64
    };
    let (mut lo, mut hi) = (0.5f64, 2.5f64);
    for _ in 0..50 {
        let m1 = lo + (hi - lo) / 3.0;
        let m2 = hi - (hi - lo) / 3.0;
        if loss(m1) < loss(m2) {
            hi = m2;
        } else {
            lo = m1;
        }
    }
    (lo + hi) / 2.0
}

fn cmd_tune_kingsafety(opts: &TuneOpts) {
    let active = ks_active_indices();
    let (_, table_len) = field_offset("king_safety_table");
    println!(
        "King-safety nonlinear fit: {} active params ({} danger inputs + {table_len}-entry table)",
        active.len(),
        KS_DANGER_INPUTS.len(),
    );

    let base_params = opts
        .initial
        .as_deref()
        .map_or_else(EvalParams::default, load_eval_file);
    println!(
        "Initial vector: {}",
        opts.initial.as_deref().unwrap_or("source defaults")
    );
    println!("Loading train from {} ...", opts.train);
    let train = load_raw_dataset(&opts.train, opts.max_positions, &base_params);
    println!("  {} train positions", train.len());
    println!("Loading holdout from {} ...", opts.holdout);
    let holdout = load_raw_dataset(&opts.holdout, opts.max_positions, &base_params);
    println!("  {} holdout positions", holdout.len());

    let mut evs: Vec<Evaluator> = (0..n_threads()).map(|_| Evaluator::default()).collect();

    let k = ks_fit_k(&holdout, &mut evs, &base_params);
    println!("K = {}", k_label(k));

    let base_w = base_params.to_flat();
    let mut w = base_w.clone();
    let mut params = base_params.clone();

    params.set_from_flat(&w);
    let mut cur_train = ks_mse(&train, &mut evs, &params, k);
    let base_holdout = ks_mse(&holdout, &mut evs, &params, k);
    let mut best_w = w.clone();
    let mut best_holdout = base_holdout;
    println!("Initial train  loss = {cur_train:.8}");
    println!("Initial holdout loss = {base_holdout:.8}");

    // Integer coordinate descent with a shrinking step — robust to the table's
    // step-function nonlinearity and the integer parameter grid. A whole-vector
    // snapshot is restored between trials because `clamp_weights` re-monotonises
    // the safety table, which can ripple into neighbouring entries.
    let mut step = 4.0f64;
    let mut epoch = 0usize;
    while step >= 1.0 && epoch < opts.epochs {
        epoch += 1;
        let mut improved = false;
        for &idx in &active {
            let snapshot = w.clone();
            w[idx] = snapshot[idx] + step;
            clamp_weights(&mut w);
            params.set_from_flat(&w);
            let up = ks_mse(&train, &mut evs, &params, k);
            let up_w = w.clone();

            w.copy_from_slice(&snapshot);
            w[idx] = snapshot[idx] - step;
            clamp_weights(&mut w);
            params.set_from_flat(&w);
            let dn = ks_mse(&train, &mut evs, &params, k);

            if up < cur_train && up <= dn {
                w.copy_from_slice(&up_w);
                cur_train = up;
                improved = true;
            } else if dn < cur_train {
                // w already holds the down candidate
                cur_train = dn;
                improved = true;
            } else {
                w.copy_from_slice(&snapshot);
            }
        }
        params.set_from_flat(&w);
        let h = ks_mse(&holdout, &mut evs, &params, k);
        if h < best_holdout {
            best_holdout = h;
            best_w.copy_from_slice(&w);
        }
        println!("Epoch {epoch:>3}  step={step:>3}  train={cur_train:.8}  holdout={h:.8}");
        if !improved {
            step /= 2.0;
        }
    }

    w.copy_from_slice(&best_w);
    println!(
        "Best holdout = {best_holdout:.8} (base {base_holdout:.8}, delta {:+.8}).",
        best_holdout - base_holdout
    );

    params.set_from_flat(&base_w);
    let base_buckets = ks_bucket_losses(&holdout, &mut evs, &params, k);
    params.set_from_flat(&w);
    let fin_buckets = ks_bucket_losses(&holdout, &mut evs, &params, k);
    ks_report_buckets(&base_buckets, &fin_buckets);

    write_eval_file(&opts.out, &w);
    let persisted = load_eval_file(&opts.out);
    let persisted_holdout = ks_mse(&holdout, &mut evs, &persisted, k);
    println!("Persisted rounded validation loss = {persisted_holdout:.8}");

    report_frozen_test(opts, k);

    print_active_deltas(&active, &base_w, &w);
    println!("Tuned weights written to {}", opts.out);
}

// ---------------------------------------------------------------------------
// main / argument parsing
// ---------------------------------------------------------------------------

/// Phase 4.0 readiness gate — **feature support**. For every weight, count the
/// positions whose linear trace gives it a nonzero tapered coefficient (i.e.
/// positions that can supply gradient signal to fit it), broken down by game
/// phase. A weight with very few activations is *underdetermined* and would
/// learn a random sign / giant value off a handful of positions — it should be
/// frozen or merged before staging. Reuses the per-position trace (no new
/// instrumentation).
fn cmd_feature_support(path: &str, max_positions: usize) {
    let mut lines = read_lines(path);
    if max_positions > 0 && lines.len() > max_positions {
        lines.truncate(max_positions);
    }
    let flat = EvalParams::FLAT_SIZE;
    let threads = n_threads().min(lines.len().max(1));
    let chunk = lines.len().div_ceil(threads.max(1));

    // phase buckets: 0 = opening (phase>=16), 1 = middlegame (6..16), 2 = endgame (<6)
    let bucket_of = |phase: i32| -> usize {
        if phase >= 16 {
            0
        } else if phase >= 6 {
            1
        } else {
            2
        }
    };

    type Acc = (Vec<u64>, Vec<f64>, [Vec<u64>; 3], u64, [u64; 3]);
    let parts: Vec<Acc> = thread::scope(|s| {
        let handles: Vec<_> = lines
            .chunks(chunk.max(1))
            .map(|slice| {
                s.spawn(move || {
                    let mut ev = Evaluator::default();
                    let mut act = vec![0u64; flat];
                    let mut sig = vec![0f64; flat];
                    let mut bucket_act = [vec![0u64; flat], vec![0u64; flat], vec![0u64; flat]];
                    let mut total = 0u64;
                    let mut bucket_total = [0u64; 3];
                    for line in slice {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        let Some(sep) = line.rfind(';') else { continue };
                        if parse_target(&line[sep + 1..]).is_none() {
                            continue;
                        }
                        let Ok(board) = Board::from_fen(&line[..sep]) else {
                            continue;
                        };
                        let _ = ev.evaluate(&board);
                        let trace = ev.last_trace();
                        let coeffs = trace.flat_coeffs();
                        let b = bucket_of(trace.phase);
                        total += 1;
                        bucket_total[b] += 1;
                        for i in 0..flat {
                            if coeffs[i] != 0.0 {
                                act[i] += 1;
                                sig[i] += coeffs[i].abs();
                                bucket_act[b][i] += 1;
                            }
                        }
                    }
                    (act, sig, bucket_act, total, bucket_total)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let mut act = vec![0u64; flat];
    let mut sig = vec![0f64; flat];
    let mut bucket_act = [vec![0u64; flat], vec![0u64; flat], vec![0u64; flat]];
    let mut total = 0u64;
    let mut bucket_total = [0u64; 3];
    for (a, sg, ba, t, bt) in parts {
        for i in 0..flat {
            act[i] += a[i];
            sig[i] += sg[i];
            for b in 0..3 {
                bucket_act[b][i] += ba[b][i];
            }
        }
        total += t;
        for b in 0..3 {
            bucket_total[b] += bt[b];
        }
    }
    if total == 0 {
        eprintln!("No positions in {path}.");
        exit(1);
    }

    // "Sparse" = a weight active in fewer than max(SPARSE_ABS, SPARSE_FRAC·N)
    // positions — too few to fit a reliable sign/magnitude.
    const SPARSE_ABS: u64 = 200;
    let sparse_frac = 0.0005_f64; // 0.05%
    let sparse_threshold = (sparse_frac * total as f64) as u64;
    let sparse_cut = SPARSE_ABS.max(sparse_threshold);

    println!("# Feature support — {path}");
    println!(
        "# positions: {total}  (opening {}, middlegame {}, endgame {})",
        bucket_total[0], bucket_total[1], bucket_total[2]
    );
    println!(
        "# sparse cut: < {sparse_cut} activations ({:.3}% of N)\n",
        sparse_frac * 100.0
    );
    println!(
        "{:<32} {:>4} {:>10} {:>7} {:>8} {:>8} {:>8} {:>10}",
        "field", "len", "act(min)", "% N", "open%", "mid%", "end%", "mean|sig|"
    );

    let mut offset = 0usize;
    let mut sparse: Vec<(String, u64)> = Vec::new();
    for &(name, len) in EVAL_PARAM_NAMES {
        // per-field summary over its `len` indices
        let mut min_act = u64::MAX;
        let mut max_act = 0u64;
        let mut sum_sig = 0f64;
        let mut sum_act = 0u64;
        let mut bsum = [0u64; 3];
        for k in 0..len {
            let i = offset + k;
            min_act = min_act.min(act[i]);
            max_act = max_act.max(act[i]);
            sum_act += act[i];
            sum_sig += sig[i];
            for b in 0..3 {
                bsum[b] += bucket_act[b][i];
            }
            if act[i] < sparse_cut {
                sparse.push((format!("{name}[{k}]"), act[i]));
            }
        }
        let pct = |x: u64, d: u64| {
            if d == 0 {
                0.0
            } else {
                100.0 * x as f64 / d as f64
            }
        };
        let mean_sig = if sum_act == 0 {
            0.0
        } else {
            sum_sig / sum_act as f64
        };
        let flag = if max_act < sparse_cut {
            " <-- DEAD/SPARSE"
        } else {
            ""
        };
        println!(
            "{:<32} {:>4} {:>10} {:>6.2}% {:>7.1} {:>7.1} {:>7.1} {:>10.2}{}",
            name,
            len,
            min_act,
            pct(max_act, total),
            pct(bsum[0], bucket_total[0].max(1)) / len as f64,
            pct(bsum[1], bucket_total[1].max(1)) / len as f64,
            pct(bsum[2], bucket_total[2].max(1)) / len as f64,
            mean_sig,
            flag
        );
        offset += len;
    }

    println!(
        "\n# individual weights below the sparse cut ({}):",
        sparse.len()
    );
    sparse.sort_by_key(|(_, a)| *a);
    for (nm, a) in &sparse {
        println!("  {nm}: {a}");
    }
}

/// Exact primary instrument/disposition for every EvalParams scalar. The
/// nonlinear king-safety fit also co-tunes the linearly traced safety table,
/// but its primary owner remains the complete linear surface so this partition
/// totals exactly once to FLAT_SIZE.
fn coverage_partition() -> Vec<&'static str> {
    let mut owner = vec![""; EvalParams::FLAT_SIZE];
    let mut claim = |idx: usize, label: &'static str| {
        assert!(owner[idx].is_empty(), "slot {idx} claimed twice");
        owner[idx] = label;
    };
    for idx in active_indices_for_group("complete") {
        claim(idx, "linear");
    }
    for field in KS_DANGER_INPUTS {
        let (off, len) = field_offset(field);
        for idx in off..off + len {
            claim(idx, "nonlinear");
        }
    }
    for idx in pst_gauge_anchors() {
        claim(idx, "gauge");
    }
    for field in ["mg_val", "eg_val"] {
        let (off, len) = field_offset(field);
        claim(off + len - 1, "invariant");
    }
    assert!(owner.iter().all(|label| !label.is_empty()));
    owner
}

fn cmd_audit_coverage() {
    let owner = coverage_partition();
    let mut flat = 0usize;
    let mut counts = std::collections::BTreeMap::<&str, usize>::new();
    println!("EvalParams instrument coverage (primary owner per scalar):");
    for &(name, len) in EVAL_PARAM_NAMES {
        for index in 0..len {
            let label = owner[flat];
            *counts.entry(label).or_default() += 1;
            println!("{flat:>4} {name:<30} {index:>3} {label}");
            flat += 1;
        }
    }
    println!("Coverage summary:");
    for (label, count) in counts {
        println!("  {label:<10} {count:>4}");
    }
    println!("  total      {flat:>4}");
    assert_eq!(flat, EvalParams::FLAT_SIZE);
    println!(
        "PASS: all {} EvalParams slots have exactly one primary disposition; nonlinear fit additionally co-tunes the 40-entry king_safety_table.",
        EvalParams::FLAT_SIZE
    );
}

/// Phase 4.0 readiness — per-bucket loss snapshot of the *current* eval, no
/// fit. Establishes the baselines a later fit's per-bucket table is judged
/// against.
fn cmd_buckets(path: &str, max_positions: usize) {
    let active: Vec<usize> = Vec::new();
    let base_params = EvalParams::default();
    println!("Loading {path} ...");
    let set = load_tune_dataset(path, &active, max_positions, &base_params);
    println!("  {} positions", set.len());
    let k = fit_k(&set);
    println!("K = {}", k_label(k));
    println!("Aggregate loss = {:.8}", default_loss(&set, k));
    let base_w = base_params.to_flat();
    report_bucket_table(&set, &active, &base_w, &base_w, k);
}

fn usage(exe: &str) {
    eprintln!("Usage:");
    eprintln!("  {exe} --verify <dataset.csv> [--weights complete-vector.txt]");
    eprintln!("  {exe} --audit-coverage");
    eprintln!("  {exe} --feature-support <dataset.csv> [--max-positions N]");
    eprintln!("  {exe} --buckets <dataset.csv> [--max-positions N] [--from-cp] [--fix-k K]");
    eprintln!(
        "  {exe} --tune <group> <train.csv> <validation.csv> [out.txt] [--initial complete-vector.txt] [--test frozen.csv --test-baseline source-vector.txt] [--test-marker FILE] [--epochs N] [--lr X] [--l2 X] [--max-positions N] [--from-cp] [--fix-k K]"
    );
    eprintln!(
        "  {exe} --tune-kingsafety <train.csv> <validation.csv> [out.txt] [--initial complete-vector.txt] [--test frozen.csv --test-baseline source-vector.txt] [--test-marker FILE] [--epochs N] [--max-positions N] [--from-cp] [--fix-k K]"
    );
    eprintln!();
    eprintln!("  --test      FROZEN test set: loaded after the fit, read once,");
    eprintln!("              selects nothing. The validation set picks K and the");
    eprintln!("              best epoch, so its loss is optimistically biased;");
    eprintln!("              only the frozen-test number is an honest residual.");
    eprintln!("  --from-cp   targets are White-POV centipawns (e.g. Hydra sf_*.csv),");
    eprintln!("              squashed 1/(1+10^(-cp/400)) at load (Phase 6.1 SF-distill)");
    eprintln!("  --fix-k K   pin sigmoid K instead of fitting it (e.g. --fix-k 1)");
    eprintln!("  --initial   start from a complete prior-stage vector; partial files fail");
    eprintln!("  --test-marker atomically enforce one-shot frozen-test use across runs");
    eprintln!(
        "  --test-baseline complete source vector for exact source-to-rounded-candidate test"
    );
    eprintln!("  {exe} --write-defaults <complete-vector.txt>");
    eprintln!(
        "  {exe} --compare-frozen <test.csv> <source-vector.txt> <candidate-vector.txt> <marker> --fix-k K"
    );
    eprintln!("  {exe} --report-endgames <dataset.csv> <complete-vector.txt> --fix-k K");
    print_groups();
}

fn validate_test_contract(opts: &TuneOpts) {
    if opts.test_marker.is_some() && opts.test.is_none() {
        eprintln!("--test-marker requires --test.");
        exit(1);
    }
    if opts.test.is_some() != opts.test_baseline.is_some() {
        eprintln!("--test and --test-baseline must be supplied together.");
        exit(1);
    }
}

/// Parse the global `--from-cp` / `--fix-k` flags shared by several
/// subcommands. Returns true (and advances `*i` past any value) if `flag`
/// was consumed.
fn parse_global_flag(flag: &str, args: &[String], i: &mut usize) -> bool {
    match flag {
        "--from-cp" => {
            FROM_CP.store(true, Ordering::Relaxed);
            true
        }
        "--fix-k" => {
            let v: f64 = args
                .get(*i)
                .and_then(|v| v.parse().ok())
                .filter(|v| *v > 0.0)
                .unwrap_or_else(|| {
                    eprintln!("Bad --fix-k (must be a positive number)");
                    exit(1)
                });
            FIX_K.store(v.to_bits(), Ordering::Relaxed);
            *i += 1;
            true
        }
        _ => false,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage(&args[0]);
        exit(1);
    }
    match args[1].as_str() {
        "--verify" => {
            if args.len() != 3 && args.len() != 5 {
                usage(&args[0]);
                exit(1);
            }
            let weights = if args.len() == 5 && args[3] == "--weights" {
                Some(args[4].as_str())
            } else if args.len() == 3 {
                None
            } else {
                usage(&args[0]);
                exit(1)
            };
            cmd_verify(&args[2], weights);
        }
        "--write-defaults" => {
            if args.len() != 3 {
                usage(&args[0]);
                exit(1);
            }
            write_eval_file(&args[2], &EvalParams::default().to_flat());
            println!("Complete source-default vector written to {}", args[2]);
        }
        "--compare-frozen" => {
            if args.len() < 7 {
                usage(&args[0]);
                exit(1);
            }
            let mut i = 6;
            while i < args.len() {
                let flag = args[i].clone();
                i += 1;
                if !parse_global_flag(&flag, &args, &mut i) {
                    eprintln!("Unknown option {flag}");
                    usage(&args[0]);
                    exit(1);
                }
            }
            cmd_compare_frozen(&args[2], &args[3], &args[4], &args[5]);
        }
        "--report-endgames" => {
            if args.len() < 5 {
                usage(&args[0]);
                exit(1);
            }
            let mut i = 4;
            while i < args.len() {
                let flag = args[i].clone();
                i += 1;
                if !parse_global_flag(&flag, &args, &mut i) {
                    eprintln!("Unknown option {flag}");
                    usage(&args[0]);
                    exit(1);
                }
            }
            cmd_report_endgames(&args[2], &args[3]);
        }
        "--audit-coverage" => cmd_audit_coverage(),
        "--tune" => {
            if args.len() < 5 {
                usage(&args[0]);
                exit(1);
            }
            let mut opts = TuneOpts {
                group: args[2].clone(),
                train: args[3].clone(),
                holdout: args[4].clone(),
                test: None,
                test_baseline: None,
                test_marker: None,
                initial: None,
                out: "tools/texel/out/eval_params.txt".to_string(),
                epochs: 200,
                lr: 0.3,
                max_positions: 0,
                l2: 0.0,
            };
            let mut i = 5;
            if i < args.len() && !args[i].starts_with("--") {
                opts.out = args[i].clone();
                i += 1;
            }
            while i < args.len() {
                let flag = args[i].clone();
                i += 1;
                let val = || {
                    args.get(i).unwrap_or_else(|| {
                        eprintln!("Missing value for {flag}");
                        exit(1);
                    })
                };
                match flag.as_str() {
                    "--initial" => {
                        opts.initial = Some(val().clone());
                        i += 1;
                    }
                    "--test" => {
                        opts.test = Some(val().clone());
                        i += 1;
                    }
                    "--test-baseline" => {
                        opts.test_baseline = Some(val().clone());
                        i += 1;
                    }
                    "--test-marker" => {
                        opts.test_marker = Some(val().clone());
                        i += 1;
                    }
                    "--epochs" => {
                        opts.epochs = val().parse().unwrap_or_else(|_| {
                            eprintln!("Bad --epochs");
                            exit(1)
                        });
                        i += 1;
                    }
                    "--lr" => {
                        opts.lr = val().parse().unwrap_or_else(|_| {
                            eprintln!("Bad --lr");
                            exit(1)
                        });
                        i += 1;
                    }
                    "--l2" => {
                        opts.l2 = val().parse().unwrap_or_else(|_| {
                            eprintln!("Bad --l2");
                            exit(1)
                        });
                        i += 1;
                    }
                    "--max-positions" => {
                        opts.max_positions = val().parse().unwrap_or_else(|_| {
                            eprintln!("Bad --max-positions");
                            exit(1)
                        });
                        i += 1;
                    }
                    other => {
                        if !parse_global_flag(other, &args, &mut i) {
                            eprintln!("Unknown option {other}");
                            usage(&args[0]);
                            exit(1);
                        }
                    }
                }
            }
            if opts.epochs == 0 {
                eprintln!("--epochs must be positive.");
                exit(1);
            }
            validate_test_contract(&opts);
            cmd_tune(&opts);
        }
        "--tune-kingsafety" => {
            if args.len() < 4 {
                usage(&args[0]);
                exit(1);
            }
            let mut opts = TuneOpts {
                group: "kingsafety-nonlinear".to_string(),
                train: args[2].clone(),
                holdout: args[3].clone(),
                test: None,
                test_baseline: None,
                test_marker: None,
                initial: None,
                out: "tools/texel/out/king_safety.txt".to_string(),
                epochs: 40,
                lr: 0.0,
                max_positions: 0,
                l2: 0.0,
            };
            let mut i = 4;
            if i < args.len() && !args[i].starts_with("--") {
                opts.out = args[i].clone();
                i += 1;
            }
            while i < args.len() {
                let flag = args[i].clone();
                i += 1;
                let val = || {
                    args.get(i).unwrap_or_else(|| {
                        eprintln!("Missing value for {flag}");
                        exit(1);
                    })
                };
                match flag.as_str() {
                    "--initial" => {
                        opts.initial = Some(val().clone());
                        i += 1;
                    }
                    "--test" => {
                        opts.test = Some(val().clone());
                        i += 1;
                    }
                    "--test-baseline" => {
                        opts.test_baseline = Some(val().clone());
                        i += 1;
                    }
                    "--test-marker" => {
                        opts.test_marker = Some(val().clone());
                        i += 1;
                    }
                    "--epochs" => {
                        opts.epochs = val().parse().unwrap_or_else(|_| {
                            eprintln!("Bad --epochs");
                            exit(1)
                        });
                        i += 1;
                    }
                    "--max-positions" => {
                        opts.max_positions = val().parse().unwrap_or_else(|_| {
                            eprintln!("Bad --max-positions");
                            exit(1)
                        });
                        i += 1;
                    }
                    other => {
                        if !parse_global_flag(other, &args, &mut i) {
                            eprintln!("Unknown option {other}");
                            usage(&args[0]);
                            exit(1);
                        }
                    }
                }
            }
            if opts.epochs == 0 {
                eprintln!("--epochs must be positive.");
                exit(1);
            }
            validate_test_contract(&opts);
            cmd_tune_kingsafety(&opts);
        }
        "--feature-support" => {
            if args.len() < 3 {
                usage(&args[0]);
                exit(1);
            }
            let path = args[2].clone();
            let mut max_positions = 0usize;
            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "--max-positions" => {
                        max_positions = args
                            .get(i + 1)
                            .and_then(|v| v.parse().ok())
                            .unwrap_or_else(|| {
                                eprintln!("Bad --max-positions");
                                exit(1)
                            });
                        i += 2;
                    }
                    "--from-cp" => {
                        FROM_CP.store(true, Ordering::Relaxed);
                        i += 1;
                    }
                    "--fix-k" => {
                        let v: f64 = args
                            .get(i + 1)
                            .and_then(|v| v.parse().ok())
                            .filter(|v| *v > 0.0)
                            .unwrap_or_else(|| {
                                eprintln!("Bad --fix-k (must be a positive number)");
                                exit(1)
                            });
                        FIX_K.store(v.to_bits(), Ordering::Relaxed);
                        i += 2;
                    }
                    other => {
                        eprintln!("Unknown option {other}");
                        usage(&args[0]);
                        exit(1);
                    }
                }
            }
            cmd_feature_support(&path, max_positions);
        }
        "--buckets" => {
            if args.len() < 3 {
                usage(&args[0]);
                exit(1);
            }
            let path = args[2].clone();
            let mut max_positions = 0usize;
            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "--max-positions" => {
                        max_positions = args
                            .get(i + 1)
                            .and_then(|v| v.parse().ok())
                            .unwrap_or_else(|| {
                                eprintln!("Bad --max-positions");
                                exit(1)
                            });
                        i += 2;
                    }
                    "--from-cp" => {
                        FROM_CP.store(true, Ordering::Relaxed);
                        i += 1;
                    }
                    "--fix-k" => {
                        let v: f64 = args
                            .get(i + 1)
                            .and_then(|v| v.parse().ok())
                            .filter(|v| *v > 0.0)
                            .unwrap_or_else(|| {
                                eprintln!("Bad --fix-k (must be a positive number)");
                                exit(1)
                            });
                        FIX_K.store(v.to_bits(), Ordering::Relaxed);
                        i += 2;
                    }
                    other => {
                        eprintln!("Unknown option {other}");
                        usage(&args[0]);
                        exit(1);
                    }
                }
            }
            cmd_buckets(&path, max_positions);
        }
        other => {
            eprintln!("Unknown mode '{other}'.");
            usage(&args[0]);
            exit(1);
        }
    }
}
