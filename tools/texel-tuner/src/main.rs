//! Rarog Texel eval tuner (Phase 3.3).
//!
//! A faithful Rust port of `tools/texel/reference/basilisk_tuner.cpp`:
//! golden-section K-fit, full-batch Adam, staged group masks, the
//! `linear_delta_scale` that captures Rarog's frozen non-linear factors, the
//! reconstruction `--verify` gate, and the `name index value` output format
//! that the engine's `RAROG_EVAL_FILE` loader (Phase 3.2) reads back.
//!
//! Dataset format: one `FEN;target` per line; `target` is the White-POV
//! expected score (`1-0`/`0-1`/`1/2-1/2`, or a float in `[0,1]`).
//!
//! Build/run (from repo root):
//!   cargo run --release -p texel-tuner -- --verify tools/texel/data/holdout.csv
//!   cargo run --release -p texel-tuner -- --tune material \
//!       tools/texel/data/train.csv tools/texel/data/holdout.csv out.txt

use std::fs;
use std::process::exit;
use std::sync::OnceLock;
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
         threats42 hanging misc kingsafety imbalance smallpos gauntlet scalars scalars44 pst all"
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
struct FamilyRanges {
    passer: (usize, usize),
    threat: (usize, usize),
    ksafe: (usize, usize),
}

fn family_ranges() -> &'static FamilyRanges {
    static R: OnceLock<FamilyRanges> = OnceLock::new();
    R.get_or_init(|| {
        let (mut passer, mut threat, mut ksafe) =
            ((usize::MAX, 0), (usize::MAX, 0), (usize::MAX, 0));
        let mut off = 0usize;
        for &(name, len) in EVAL_PARAM_NAMES {
            let end = off + len;
            if name.starts_with("passed") {
                passer = (passer.0.min(off), passer.1.max(end));
            } else if name.starts_with("threat") {
                threat = (threat.0.min(off), threat.1.max(end));
            } else if name == "king_safety_table" {
                ksafe = (ksafe.0.min(off), ksafe.1.max(end));
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
    let any_nz = |(s, e): (usize, usize)| s < e && coeffs[s..e].iter().any(|&c| c != 0.0);
    if any_nz(fr.ksafe) {
        m |= 1 << 7;
    }
    if any_nz(fr.passer) {
        m |= 1 << 8;
    }
    if any_nz(fr.threat) {
        m |= 1 << 9;
    }
    m
}

struct TuneSet {
    active_count: usize,
    result: Vec<f32>,
    base_score: Vec<f32>,
    /// Row-major `len × active_count`: the tapered per-weight coefficient
    /// `(count_mg·phase + count_eg·(24−phase))/24 · delta_scale`.
    coeffs: Vec<f32>,
    /// Per-position bucket bitmask (see `BUCKET_NAMES`).
    buckets: Vec<u32>,
}

impl TuneSet {
    fn len(&self) -> usize {
        self.result.len()
    }
    fn row(&self, i: usize) -> &[f32] {
        &self.coeffs[i * self.active_count..(i + 1) * self.active_count]
    }
}

fn parse_target(text: &str) -> Option<f32> {
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

fn n_threads() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Evaluate one line into (result, base_score_white, coeff row, bucket mask)
/// for the given active indices. Returns None for blank/malformed lines.
fn process_line(
    evaluator: &mut Evaluator,
    defaults: &EvalParams,
    active: &[usize],
    line: &str,
) -> Option<(f32, f32, Vec<f32>, u32)> {
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
    let recon = trace.reconstruct(defaults);
    let rest = score_white - recon;
    let base_white = recon + rest; // == score_white, kept explicit for clarity

    let scale = linear_delta_scale(&board) as f32;
    let coeffs = trace.flat_coeffs();
    let row: Vec<f32> = active
        .iter()
        .map(|&idx| coeffs[idx] as f32 * scale)
        .collect();
    let buckets = position_buckets(&board, trace.phase, &coeffs);
    Some((result, base_white as f32, row, buckets))
}

fn load_tune_dataset(path: &str, active: &[usize], max_positions: usize) -> TuneSet {
    let mut lines = read_lines(path);
    if max_positions > 0 && lines.len() > max_positions {
        lines.truncate(max_positions);
    }
    let active_count = active.len();
    let threads = n_threads().min(lines.len().max(1));
    let chunk = lines.len().div_ceil(threads.max(1));

    let defaults = EvalParams::default();
    let parts: Vec<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<u32>)> = thread::scope(|s| {
        let handles: Vec<_> = lines
            .chunks(chunk.max(1))
            .map(|slice| {
                let defaults = &defaults;
                s.spawn(move || {
                    let mut evaluator = Evaluator::default();
                    let mut res = Vec::new();
                    let mut base = Vec::new();
                    let mut co = Vec::new();
                    let mut bk = Vec::new();
                    for line in slice {
                        if let Some((r, b, row, mask)) =
                            process_line(&mut evaluator, defaults, active, line)
                        {
                            res.push(r);
                            base.push(b);
                            co.extend_from_slice(&row);
                            bk.push(mask);
                        }
                    }
                    (res, base, co, bk)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let mut set = TuneSet {
        active_count,
        result: Vec::new(),
        base_score: Vec::new(),
        coeffs: Vec::new(),
        buckets: Vec::new(),
    };
    for (res, base, co, bk) in parts {
        set.result.extend(res);
        set.base_score.extend(base);
        set.coeffs.extend(co);
        set.buckets.extend(bk);
    }
    if set.len() == 0 {
        eprintln!("No positions loaded from {path}.");
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
    let row = set.row(i);
    for (j, &idx) in active.iter().enumerate() {
        score += row[j] as f64 * (w[idx] - base_w[idx]);
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
    println!("\nPer-bucket holdout loss:");
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

fn cmd_verify(path: &str) {
    const VERIFY_COUNT: usize = 10_000;
    println!("Loading up to {VERIFY_COUNT} positions from {path} ...");
    let mut lines = read_lines(path);
    lines.retain(|l| {
        let t = l.trim();
        !t.is_empty() && !t.starts_with('#')
    });
    lines.truncate(VERIFY_COUNT);

    let defaults = EvalParams::default();
    let mut evaluator = Evaluator::default();
    let mut checked = 0usize;
    let mut mismatches = 0usize;
    let mut max_err = 0i32;

    for line in &lines {
        let Some(sep) = line.rfind(';') else { continue };
        let fen = &line[..sep];
        let Ok(board) = Board::from_fen(fen) else {
            continue;
        };
        let score = evaluator.evaluate(&board);
        let trace = evaluator.last_trace();
        let score_white = if board.side_to_move() == rarog::board::Color::White {
            score
        } else {
            -score
        };
        let recon = trace.reconstruct(&defaults);
        let rest = score_white - recon;

        // Re-evaluate and confirm reconstruct + rest reproduces the eval.
        let fresh = evaluator.evaluate(&board);
        let fresh_white = if board.side_to_move() == rarog::board::Color::White {
            fresh
        } else {
            -fresh
        };
        let err = (fresh_white - (recon + rest)).abs();
        if err != 0 {
            mismatches += 1;
            max_err = max_err.max(err);
            if mismatches <= 5 {
                eprintln!(
                    "MISMATCH: actual={fresh_white} recon+rest={} fen={fen}",
                    recon + rest
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

    println!("Loading train from {} ...", opts.train);
    let train = load_tune_dataset(&opts.train, &active, opts.max_positions);
    println!("  {} train positions", train.len());
    println!("Loading holdout from {} ...", opts.holdout);
    let holdout = load_tune_dataset(&opts.holdout, &active, opts.max_positions);
    println!("  {} holdout positions", holdout.len());

    let k = fit_k(&holdout);
    println!("Fitted K = {k:.5}");

    let base_w = EvalParams::default().to_flat();
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
                            let row = train.row(i);
                            for (j, gj) in g.iter_mut().enumerate() {
                                *gj += coeff * row[j] as f64;
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
    println!("Best holdout epoch {best_epoch} (holdout={best_holdout:.8}).");
    report_bucket_table(&holdout, &active, &base_w, &w, k);
    print_active_deltas(&active, &base_w, &w);
    write_eval_file(&opts.out, &w);
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
    "ks_queen_relief",
];

/// Active flat indices for the king-safety fit: the 11 danger-index inputs plus
/// the 40-entry safety table they index into. The table is co-tuned because its
/// shape only makes sense against the index distribution the inputs produce.
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
fn load_raw_dataset(path: &str, max_positions: usize) -> Vec<RawPos> {
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
    println!("\nPer-bucket holdout loss:");
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

/// Fit K once from base-parameter scores (ternary search), so the K-fit does
/// not re-evaluate the dataset per iteration.
fn ks_fit_k(boards: &[RawPos], evs: &mut [Evaluator], base: &EvalParams) -> f64 {
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

    println!("Loading train from {} ...", opts.train);
    let train = load_raw_dataset(&opts.train, opts.max_positions);
    println!("  {} train positions", train.len());
    println!("Loading holdout from {} ...", opts.holdout);
    let holdout = load_raw_dataset(&opts.holdout, opts.max_positions);
    println!("  {} holdout positions", holdout.len());

    let mut evs: Vec<Evaluator> = (0..n_threads()).map(|_| Evaluator::default()).collect();

    let base_params = EvalParams::default();
    let k = ks_fit_k(&holdout, &mut evs, &base_params);
    println!("Fitted K = {k:.5}");

    let base_w = base_params.to_flat();
    let mut w = base_w.clone();
    let mut params = EvalParams::default();

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

    print_active_deltas(&active, &base_w, &w);
    write_eval_file(&opts.out, &w);
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

/// Phase 4.0 readiness — per-bucket loss snapshot of the *current* eval, no
/// fit. Establishes the baselines a later fit's per-bucket table is judged
/// against.
fn cmd_buckets(path: &str, max_positions: usize) {
    let active: Vec<usize> = Vec::new();
    println!("Loading {path} ...");
    let set = load_tune_dataset(path, &active, max_positions);
    println!("  {} positions", set.len());
    let k = fit_k(&set);
    println!("Fitted K = {k:.5}");
    println!("Aggregate loss = {:.8}", default_loss(&set, k));
    let base_w = EvalParams::default().to_flat();
    report_bucket_table(&set, &active, &base_w, &base_w, k);
}

fn usage(exe: &str) {
    eprintln!("Usage:");
    eprintln!("  {exe} --verify <dataset.csv>");
    eprintln!("  {exe} --feature-support <dataset.csv> [--max-positions N]");
    eprintln!("  {exe} --buckets <dataset.csv> [--max-positions N]");
    eprintln!(
        "  {exe} --tune <group> <train.csv> <holdout.csv> [out.txt] [--epochs N] [--lr X] [--l2 X] [--max-positions N]"
    );
    eprintln!(
        "  {exe} --tune-kingsafety <train.csv> <holdout.csv> [out.txt] [--epochs N] [--max-positions N]"
    );
    print_groups();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage(&args[0]);
        exit(1);
    }
    match args[1].as_str() {
        "--verify" => {
            if args.len() != 3 {
                usage(&args[0]);
                exit(1);
            }
            cmd_verify(&args[2]);
        }
        "--tune" => {
            if args.len() < 5 {
                usage(&args[0]);
                exit(1);
            }
            let mut opts = TuneOpts {
                group: args[2].clone(),
                train: args[3].clone(),
                holdout: args[4].clone(),
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
                        eprintln!("Unknown option {other}");
                        usage(&args[0]);
                        exit(1);
                    }
                }
            }
            if opts.epochs == 0 {
                eprintln!("--epochs must be positive.");
                exit(1);
            }
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
                        eprintln!("Unknown option {other}");
                        usage(&args[0]);
                        exit(1);
                    }
                }
            }
            if opts.epochs == 0 {
                eprintln!("--epochs must be positive.");
                exit(1);
            }
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
