//! The compact kernel's decisions, each a pure function of an input struct
//! and the seed table.
//!
//! A rule reads nothing but its arguments, so the parity instrument can call
//! it on the donor's recorded inputs and compare its output with the donor's
//! recorded decision, and a probe can count its firings without sitting
//! inside the node. The seeds are the donor's fitted values in its units;
//! the fold into Rarog's units rewrites this table.
//!
//! A score is valid unless it is `VALUE_NONE`, the evaluation of a node in
//! check and the score of an absent table entry; a decisive score is a win
//! or a loss in the tablebase band or beyond.
#![expect(
    dead_code,
    reason = "the kernel's nodes, which call the rules, land in later commits"
)]

use crate::eval::{INF_SCORE, VALUE_NONE};
use crate::tt::{Bound, TB_WIN_SCORE};

/// Every constant the kernel's rules read, in kernel units. A fraction
/// written `*_t` is in ten-thousandths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Seeds {
    // Evaluation wrapper: `(raw·(raw_base + material) + optimism·
    // (optimism_base + material)) / divisor`, damped linearly to zero at
    // `fifty_horizon` plies of the fifty-move clock.
    pub(crate) wrap_raw_base: i32,
    pub(crate) wrap_optimism_base: i32,
    pub(crate) wrap_divisor: i32,
    pub(crate) wrap_fifty_horizon: i32,

    // Table cutoff: a bound that would cut against the node's expected type
    // needs more than this depth, and no cutoff is taken from this many plies
    // of the fifty-move clock on.
    pub(crate) tt_cut_typed_depth: i32,
    pub(crate) tt_cut_fifty: i32,
    // The cutoff rewards a quiet TT move reached through one of the parent's
    // first moves, before this one.
    // The cutoff's reward to a quiet TT move: `min(slope·d − offset, cap)`.
    pub(crate) tt_bonus_parent_moves: i32,
    pub(crate) tt_bonus_quiet_slope: i32,
    pub(crate) tt_bonus_quiet_offset: i32,
    pub(crate) tt_bonus_quiet_cap: i32,
    pub(crate) tt_bonus_cont_slope: i32,
    pub(crate) tt_bonus_cont_offset: i32,
    pub(crate) tt_bonus_cont_cap: i32,

    // The evaluation swing across the parent's quiet move, as its history,
    // below this depth or without a stored entry.
    pub(crate) eval_bonus_depth: i32,
    pub(crate) eval_bonus_scale: i32,
    pub(crate) eval_bonus_min: i32,
    pub(crate) eval_bonus_max: i32,

    // Hindsight: deeper after a parent reduction this large and an
    // evaluation that fell, shallower past this rise.
    pub(crate) hindsight_reduction: i32,
    pub(crate) hindsight_rise: i32,

    // Razoring below `alpha − base − square·d²`, while alpha is below the
    // limit.
    pub(crate) razor_base: i32,
    pub(crate) razor_square: i32,
    pub(crate) razor_alpha_limit: i32,

    // Reverse futility's margin and the fraction of the way to beta it
    // returns.
    pub(crate) rfp_square: i32,
    pub(crate) rfp_improvement: i32,
    pub(crate) rfp_linear: i32,
    pub(crate) rfp_correction: i32,
    pub(crate) rfp_unthreatened: i32,
    pub(crate) rfp_constant: i32,
    pub(crate) rfp_t: i32,

    // The null move's margin, material floor, reduction and verification.
    pub(crate) nmp_depth: i32,
    pub(crate) nmp_tt_pv: i32,
    pub(crate) nmp_improvement: i32,
    pub(crate) nmp_cutoffs: i32,
    pub(crate) nmp_base: i32,
    pub(crate) nmp_material: i32,
    pub(crate) nmp_r_base: i32,
    pub(crate) nmp_r_improving: i32,
    pub(crate) nmp_r_depth: i32,
    pub(crate) nmp_r_surplus: i32,
    pub(crate) nmp_r_surplus_cap: i32,
    pub(crate) nmp_verify_depth: i32,

    // ProbCut's raised beta, its depth and what it returns.
    pub(crate) probcut_margin: i32,
    pub(crate) probcut_improving: i32,
    pub(crate) probcut_depth_base: i32,
    pub(crate) probcut_depth_per: i32,
    pub(crate) probcut_raise: i32,
    pub(crate) probcut_t: i32,

    // Singular search: the candidate, the margin, the extension bars and the
    // multi-cut's return.
    pub(crate) singular_depth: i32,
    pub(crate) singular_tt_depth: i32,
    pub(crate) singular_exact_div: i32,
    pub(crate) double_pv: i32,
    pub(crate) double_new_pv: i32,
    pub(crate) double_quiet: i32,
    pub(crate) double_correction: i32,
    pub(crate) triple_pv: i32,
    pub(crate) triple_new_pv: i32,
    pub(crate) triple_quiet: i32,
    pub(crate) triple_correction: i32,
    pub(crate) triple_base: i32,
    pub(crate) multicut_t: i32,
    pub(crate) negative_extension: i32,
    pub(crate) ldse_depth: i32,
    pub(crate) ldse_margin: i32,

    // Late-move pruning's count limit, in 1024ths of a move.
    pub(crate) lmp_base: i32,
    pub(crate) lmp_improvement: i32,
    pub(crate) lmp_square: i32,
    pub(crate) lmp_history: i32,

    // Quiet futility.
    pub(crate) fp_linear: i32,
    pub(crate) fp_history: i32,
    pub(crate) fp_above_beta: i32,
    pub(crate) fp_correction: i32,
    pub(crate) fp_constant: i32,
    pub(crate) fp_depth: i32,

    // Bad-noisy futility.
    pub(crate) bnfp_linear: i32,
    pub(crate) bnfp_history: i32,
    pub(crate) bnfp_constant: i32,
    pub(crate) bnfp_depth: i32,

    // History pruning.
    pub(crate) hp_slope: i32,
    pub(crate) hp_depth: i32,

    // The SEE allowances.
    pub(crate) see_quiet_square: i32,
    pub(crate) see_quiet_linear: i32,
    pub(crate) see_quiet_history: i32,
    pub(crate) see_quiet_constant: i32,
    pub(crate) see_noisy_square: i32,
    pub(crate) see_noisy_linear: i32,
    pub(crate) see_noisy_history: i32,
    pub(crate) see_noisy_constant: i32,

    // The late-move reduction, in 1024ths of a ply.
    pub(crate) lmr_log: i32,
    pub(crate) lmr_improvement: i32,
    pub(crate) lmr_improvement_min: i32,
    pub(crate) lmr_improvement_max: i32,
    pub(crate) lmr_correction: i32,
    pub(crate) lmr_exact: i32,
    pub(crate) lmr_tt_below: i32,
    pub(crate) lmr_tt_shallow: i32,
    pub(crate) lmr_win_beta: i32,
    pub(crate) lmr_quiet: i32,
    pub(crate) lmr_quiet_history: i32,
    pub(crate) lmr_alpha_gap: i32,
    pub(crate) lmr_alpha_gap_min: i32,
    pub(crate) lmr_alpha_gap_max: i32,
    pub(crate) lmr_noisy: i32,
    pub(crate) lmr_noisy_history: i32,
    pub(crate) lmr_critical: i32,
    pub(crate) lmr_pv: i32,
    pub(crate) lmr_pv_window: i32,
    pub(crate) lmr_non_pv: i32,
    pub(crate) lmr_laterality: i32,
    pub(crate) lmr_tt_pv: i32,
    pub(crate) lmr_tt_pv_above: i32,
    pub(crate) lmr_tt_pv_deep: i32,
    pub(crate) lmr_cut: i32,
    pub(crate) lmr_cut_no_tt_move: i32,
    pub(crate) lmr_check: i32,
    pub(crate) lmr_cutoffs: i32,
    pub(crate) lmr_cutoffs_all: i32,
    pub(crate) lmr_singular: i32,
    pub(crate) lmr_singular_offset: i32,
    pub(crate) lmr_singular_cap: i32,
    pub(crate) lmr_parent_gap: i32,
    pub(crate) lmr_parent: i32,
    pub(crate) lmr_spread_thread: i32,
    pub(crate) lmr_spread_centre: i32,

    // The re-search after a reduced move beat alpha.
    pub(crate) research_deeper: i32,
    pub(crate) research_shallower: i32,

    // The reduction of a search that LMR did not reduce, in 1024ths.
    pub(crate) fds_log: i32,
    pub(crate) fds_improvement: i32,
    pub(crate) fds_improvement_min: i32,
    pub(crate) fds_improvement_max: i32,
    pub(crate) fds_correction: i32,
    pub(crate) fds_quiet: i32,
    pub(crate) fds_quiet_history: i32,
    pub(crate) fds_noisy: i32,
    pub(crate) fds_noisy_history: i32,
    pub(crate) fds_tt_pv: i32,
    pub(crate) fds_tt_pv_deep: i32,
    pub(crate) fds_cut: i32,
    pub(crate) fds_cut_no_tt_move: i32,
    pub(crate) fds_cutoffs: i32,
    pub(crate) fds_cutoffs_all: i32,
    pub(crate) fds_singular: i32,
    pub(crate) fds_singular_offset: i32,
    pub(crate) fds_singular_cap: i32,
    pub(crate) fds_tt_move: i32,
    pub(crate) fds_parent_gap: i32,
    pub(crate) fds_parent: i32,
    pub(crate) fds_spread_thread: i32,
    pub(crate) fds_spread_centre: i32,
    pub(crate) fds_one_ply: i32,
    pub(crate) fds_two_plies: i32,

    // History amounts after a best move: `min(slope·d, cap) − offset −
    // per·n` for each table.
    pub(crate) noisy_bonus_slope: i32,
    pub(crate) noisy_bonus_cap: i32,
    pub(crate) noisy_bonus_offset: i32,
    pub(crate) noisy_bonus_cut: i32,
    pub(crate) noisy_malus_slope: i32,
    pub(crate) noisy_malus_cap: i32,
    pub(crate) noisy_malus_offset: i32,
    pub(crate) noisy_malus_per: i32,
    pub(crate) quiet_bonus_slope: i32,
    pub(crate) quiet_bonus_cap: i32,
    pub(crate) quiet_bonus_offset: i32,
    pub(crate) quiet_bonus_cut: i32,
    pub(crate) quiet_malus_slope: i32,
    pub(crate) quiet_malus_cap: i32,
    pub(crate) quiet_malus_offset: i32,
    pub(crate) quiet_malus_per: i32,
    pub(crate) cont_bonus_slope: i32,
    pub(crate) cont_bonus_cap: i32,
    pub(crate) cont_bonus_offset: i32,
    pub(crate) cont_bonus_cut: i32,
    pub(crate) cont_malus_slope: i32,
    pub(crate) cont_malus_cap: i32,
    pub(crate) cont_malus_offset: i32,
    pub(crate) cont_malus_per: i32,
    pub(crate) malus_fade: i32,
    pub(crate) parent_malus_slope: i32,
    pub(crate) parent_malus_offset: i32,
    pub(crate) parent_malus_cap: i32,
    pub(crate) research_bonus_slope: i32,
    pub(crate) research_bonus_offset: i32,
    pub(crate) research_bonus_cap: i32,

    // The parent's reward after this node failed low.
    pub(crate) reward_base: i32,
    pub(crate) reward_count: i32,
    pub(crate) reward_count_cap: i32,
    pub(crate) reward_tt_move: i32,
    pub(crate) reward_below_eval: i32,
    pub(crate) reward_below_eval_margin: i32,
    pub(crate) reward_below_parent: i32,
    pub(crate) reward_below_parent_margin: i32,
    pub(crate) reward_slope: i32,
    pub(crate) reward_offset: i32,
    pub(crate) reward_cap: i32,
    pub(crate) reward_grand_slope: i32,
    pub(crate) reward_grand_offset: i32,
    pub(crate) reward_grand_cap: i32,
    pub(crate) reward_noisy_slope: i32,
    pub(crate) reward_noisy_cap: i32,

    // The correction's update.
    pub(crate) corr_slope: i32,
    pub(crate) corr_min: i32,
    pub(crate) corr_max: i32,

    // Quiescence.
    pub(crate) q_stand_pat_t: i32,
    pub(crate) q_move_limit: i32,
    pub(crate) q_see_divisor: i32,
    pub(crate) q_see_correction_cap: i32,
    pub(crate) q_see_constant: i32,
    pub(crate) q_see_history: i32,
    pub(crate) q_cutoff_bonus: i32,
    pub(crate) q_tail_t: i32,

    // The root window.
    pub(crate) window_base: i32,
    pub(crate) window_stability_cap: i32,
    pub(crate) window_square: i32,
    pub(crate) window_low_growth: i32,
    pub(crate) window_high_growth: i32,
    pub(crate) window_stable: i32,
    pub(crate) optimism_scale: i32,
    pub(crate) optimism_offset: i32,

    // Move ordering: a noisy move by its victim's exchange value, its
    // history, a queen promotion and, in check, the cheapness of the piece
    // that moves; a quiet one by its histories and its squares.
    pub(crate) pick_victim: i32,
    pub(crate) pick_queen_promotion: i32,
    pub(crate) pick_check_base: i32,
    pub(crate) pick_check_piece: i32,
    pub(crate) pick_quiet: i32,
    pub(crate) pick_pawn: i32,
    pub(crate) pick_cont: [i32; 4],
    pub(crate) pick_escape: [i32; 6],
    pub(crate) pick_check_square: i32,
    pub(crate) pick_threatened: i32,
    pub(crate) pick_offense: i32,
    pub(crate) pick_wall: i32,
    // A noisy move is good when its SEE clears `−score/divisor + offset`, or
    // past this many good ones under a quiet TT move, when it wins material.
    pub(crate) pick_see_divisor: i32,
    pub(crate) pick_see_offset: i32,
    pub(crate) pick_late_good: i32,
}

impl Seeds {
    /// The donor's values.
    pub(crate) const DONOR: Self = Self {
        wrap_raw_base: 21_032,
        wrap_optimism_base: 1_548,
        wrap_divisor: 27_015,
        wrap_fifty_horizon: 200,

        tt_cut_typed_depth: 5,
        tt_cut_fifty: 90,
        tt_bonus_parent_moves: 4,
        tt_bonus_quiet_slope: 190,
        tt_bonus_quiet_offset: 81,
        tt_bonus_quiet_cap: 1_691,
        tt_bonus_cont_slope: 96,
        tt_bonus_cont_offset: 73,
        tt_bonus_cont_cap: 1_206,

        eval_bonus_depth: 6,
        eval_bonus_scale: 812,
        eval_bonus_min: -144,
        eval_bonus_max: 324,

        hindsight_reduction: 2_249,
        hindsight_rise: 57,

        razor_base: 237,
        razor_square: 254,
        razor_alpha_limit: 2_048,

        rfp_square: 1_140,
        rfp_improvement: 120,
        rfp_linear: 22,
        rfp_correction: 669,
        rfp_unthreatened: 54,
        rfp_constant: 19,
        rfp_t: 6_945,

        nmp_depth: 9,
        nmp_tt_pv: 110,
        nmp_improvement: 94,
        nmp_cutoffs: 21,
        nmp_base: 337,
        nmp_material: 491,
        nmp_r_base: 4_407,
        nmp_r_improving: 917,
        nmp_r_depth: 265,
        nmp_r_surplus: 477,
        nmp_r_surplus_cap: 1_187,
        nmp_verify_depth: 16,

        probcut_margin: 254,
        probcut_improving: 85,
        probcut_depth_base: 4,
        probcut_depth_per: 319,
        probcut_raise: 197,
        probcut_t: 2_695,

        singular_depth: 5,
        singular_tt_depth: 3,
        singular_exact_div: 4,
        double_pv: 195,
        double_new_pv: 48,
        double_quiet: 16,
        double_correction: 16,
        triple_pv: 230,
        triple_new_pv: 56,
        triple_quiet: 19,
        triple_correction: 15,
        triple_base: 36,
        multicut_t: 4_027,
        negative_extension: -3,
        ldse_depth: 7,
        ldse_margin: 25,

        lmp_base: 2_818,
        lmp_improvement: 78,
        lmp_square: 1_351,
        lmp_history: 74,

        fp_linear: 79,
        fp_history: 55,
        fp_above_beta: 77,
        fp_correction: 555,
        fp_constant: 127,
        fp_depth: 14,

        bnfp_linear: 84,
        bnfp_history: 82,
        bnfp_constant: 24,
        bnfp_depth: 11,

        hp_slope: 948,
        hp_depth: 5,

        see_quiet_square: 12,
        see_quiet_linear: 56,
        see_quiet_history: 27,
        see_quiet_constant: 27,
        see_noisy_square: 7,
        see_noisy_linear: 36,
        see_noisy_history: 39,
        see_noisy_constant: 14,

        lmr_log: 269,
        lmr_improvement: 425,
        lmr_improvement_min: -241,
        lmr_improvement_max: 1_155,
        lmr_correction: 3_417,
        lmr_exact: 1_412,
        lmr_tt_below: 464,
        lmr_tt_shallow: 326,
        lmr_win_beta: 1_024,
        lmr_quiet: 2_432,
        lmr_quiet_history: 179,
        lmr_alpha_gap: 418,
        lmr_alpha_gap_min: -65,
        lmr_alpha_gap_max: 91,
        lmr_noisy: 1_687,
        lmr_noisy_history: 130,
        lmr_critical: 128,
        lmr_pv: 519,
        lmr_pv_window: 437,
        lmr_non_pv: 96,
        lmr_laterality: 32,
        lmr_tt_pv: 333,
        lmr_tt_pv_above: 611,
        lmr_tt_pv_deep: 685,
        lmr_cut: 1_852,
        lmr_cut_no_tt_move: 2_204,
        lmr_check: 955,
        lmr_cutoffs: 1_151,
        lmr_cutoffs_all: 400,
        lmr_singular: 496,
        lmr_singular_offset: 185,
        lmr_singular_cap: 2_021,
        lmr_parent_gap: 414,
        lmr_parent: 136,
        lmr_spread_thread: 27,
        lmr_spread_centre: 59,

        research_deeper: 57,
        research_shallower: 9,

        fds_log: 207,
        fds_improvement: 366,
        fds_improvement_min: -206,
        fds_improvement_max: 1_370,
        fds_correction: 2_255,
        fds_quiet: 1_468,
        fds_quiet_history: 118,
        fds_noisy: 940,
        fds_noisy_history: 63,
        fds_tt_pv: 844,
        fds_tt_pv_deep: 1_129,
        fds_cut: 1_260,
        fds_cut_no_tt_move: 2_168,
        fds_cutoffs: 1_394,
        fds_cutoffs_all: 258,
        fds_singular: 351,
        fds_singular_offset: 188,
        fds_singular_cap: 2_167,
        fds_tt_move: 3_002,
        fds_parent_gap: 590,
        fds_parent: 130,
        fds_spread_thread: 26,
        fds_spread_centre: 56,
        fds_one_ply: 2_621,
        fds_two_plies: 5_579,

        noisy_bonus_slope: 96,
        noisy_bonus_cap: 885,
        noisy_bonus_offset: 43,
        noisy_bonus_cut: 87,
        noisy_malus_slope: 175,
        noisy_malus_cap: 1_252,
        noisy_malus_offset: 58,
        noisy_malus_per: 16,
        quiet_bonus_slope: 184,
        quiet_bonus_cap: 1_742,
        quiet_bonus_offset: 72,
        quiet_bonus_cut: 42,
        quiet_malus_slope: 171,
        quiet_malus_cap: 1_099,
        quiet_malus_offset: 46,
        quiet_malus_per: 31,
        cont_bonus_slope: 97,
        cont_bonus_cap: 1_098,
        cont_bonus_offset: 74,
        cont_bonus_cut: 48,
        cont_malus_slope: 414,
        cont_malus_cap: 949,
        cont_malus_offset: 49,
        cont_malus_per: 17,
        malus_fade: 45,
        parent_malus_slope: 93,
        parent_malus_offset: 52,
        parent_malus_cap: 935,
        research_bonus_slope: 233,
        research_bonus_offset: 86,
        research_bonus_cap: 1_550,

        reward_base: 88,
        reward_count: 17,
        reward_count_cap: 229,
        reward_tt_move: 110,
        reward_below_eval: 144,
        reward_below_eval_margin: 97,
        reward_below_parent: 306,
        reward_below_parent_margin: 136,
        reward_slope: 180,
        reward_offset: 37,
        reward_cap: 2_414,
        reward_grand_slope: 152,
        reward_grand_offset: 47,
        reward_grand_cap: 1_379,
        reward_noisy_slope: 50,
        reward_noisy_cap: 654,

        corr_slope: 148,
        corr_min: -4_678,
        corr_max: 2_496,

        q_stand_pat_t: 8_256,
        q_move_limit: 3,
        q_see_divisor: 8,
        q_see_correction_cap: 68,
        q_see_constant: 74,
        q_see_history: 48,
        q_cutoff_bonus: 100,
        q_tail_t: 5_072,

        window_base: 23,
        window_stability_cap: 7,
        window_square: 26_394,
        window_low_growth: 26,
        window_high_growth: 60,
        window_stable: 12,
        optimism_scale: 113,
        optimism_offset: 201,

        pick_victim: 14_232,
        pick_queen_promotion: 4_558,
        pick_check_base: 200_000,
        pick_check_piece: 20_000,
        pick_quiet: 1_763,
        pick_pawn: 1_024,
        pick_cont: [1_614, 1_066, 1_086, 1_051],
        pick_escape: [0, 8_854, 8_170, 14_051, 20_357, 0],
        pick_check_square: 10_723,
        pick_threatened: 8_875,
        pick_offense: 3_446,
        pick_wall: 4_494,
        pick_see_divisor: 47,
        pick_see_offset: 116,
        pick_late_good: 2,
    };
}

/// The seeds the kernel searches with.
pub(crate) const SEEDS: &Seeds = &Seeds::DONOR;

#[inline(always)]
const fn is_valid(score: i32) -> bool {
    score != VALUE_NONE
}

#[inline(always)]
const fn is_win(score: i32) -> bool {
    score >= TB_WIN_SCORE
}

#[inline(always)]
const fn is_loss(score: i32) -> bool {
    score <= -TB_WIN_SCORE
}

#[inline(always)]
pub(crate) const fn is_decisive(score: i32) -> bool {
    is_win(score) || is_loss(score)
}

/// `from` moved the fraction `t` (in ten-thousandths) of the way to `to`.
/// Single precision with a fused multiply-add, truncated toward zero: the
/// one form whose rounding the seeds were fitted with.
#[inline(always)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "a score moved toward another score stays between the two"
)]
pub(crate) fn lerp(from: i32, to: i32, t: i32) -> i32 {
    let t = t as f32 / 10_000.0;
    t.mul_add((to - from) as f32, from as f32) as i32
}

/// `min(slope·depth − offset, cap)`, the shape of most history amounts.
#[inline(always)]
const fn capped(slope: i32, depth: i32, offset: i32, cap: i32) -> i32 {
    let value = slope * depth - offset;
    if value < cap { value } else { cap }
}

/// `ilog2(depth)` for a depth of at least one.
#[inline(always)]
fn log2(depth: i32) -> i32 {
    debug_assert!(depth >= 1);
    depth.ilog2().cast_signed()
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

/// The evaluation wrapper's inputs.
#[derive(Copy, Clone, Debug)]
pub(crate) struct WrapperInput {
    /// The raw evaluation in kernel units.
    pub(crate) raw: i32,
    /// Material on the board in the kernel's exchange values.
    pub(crate) material: i32,
    /// The side to move's optimism.
    pub(crate) optimism: i32,
    /// The fifty-move clock in plies.
    pub(crate) fifty: i32,
    /// The correction tables' value for the position.
    pub(crate) correction: i32,
}

/// The corrected evaluation: the raw evaluation weighed up with the material
/// on the board, optimism added the same way, both damped toward the
/// fifty-move horizon, then corrected and kept out of the decisive band.
#[inline(always)]
pub(crate) fn wrapped_eval(s: &Seeds, i: &WrapperInput) -> i32 {
    let raw = i64::from(i.raw) * i64::from(s.wrap_raw_base + i.material);
    let optimism = i64::from(i.optimism) * i64::from(s.wrap_optimism_base + i.material);
    let mut eval = (raw + optimism) / i64::from(s.wrap_divisor);
    eval = eval * i64::from(s.wrap_fifty_horizon - i.fifty) / i64::from(s.wrap_fifty_horizon);
    let bound = i64::from(TB_WIN_SCORE - 1);
    i32::try_from((eval + i64::from(i.correction)).clamp(-bound, bound))
        .unwrap_or_else(|_| unreachable!("clamped into i32 bounds"))
}

/// The side to move's optimism for a running root score.
#[inline(always)]
pub(crate) fn optimism(s: &Seeds, average: i32) -> i32 {
    s.optimism_scale * average / (average.abs() + s.optimism_offset)
}

/// What a stored score says about the node's value, given its evaluation.
#[derive(Copy, Clone, Debug)]
pub(crate) struct EstimateInput {
    pub(crate) in_check: bool,
    pub(crate) excluded: bool,
    pub(crate) tt_score: i32,
    pub(crate) bound: Option<Bound>,
    pub(crate) eval: i32,
}

/// The estimated score: the stored score where its bound says the value
/// lies beyond the evaluation on its side, else the evaluation.
#[inline(always)]
pub(crate) fn estimated_score(i: &EstimateInput) -> i32 {
    let tightens = match i.bound {
        Some(Bound::Upper) => i.tt_score < i.eval,
        Some(Bound::Lower) => i.tt_score > i.eval,
        _ => true,
    };
    if !i.in_check && !i.excluded && is_valid(i.tt_score) && tightens {
        i.tt_score
    } else {
        i.eval
    }
}

/// The parent's quiet move's history reward from the evaluation swing across
/// it: the side that moved gains what the side to move lost.
#[inline(always)]
pub(crate) fn eval_difference_bonus(s: &Seeds, eval: i32, parent_eval: i32) -> i32 {
    (s.eval_bonus_scale * -(eval + parent_eval) / 128).clamp(s.eval_bonus_min, s.eval_bonus_max)
}

// ---------------------------------------------------------------------------
// Table use
// ---------------------------------------------------------------------------

/// A table probe's cutoff test.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TtCutInput {
    pub(crate) pv: bool,
    pub(crate) excluded: bool,
    pub(crate) tt_depth: i32,
    pub(crate) depth: i32,
    pub(crate) tt_score: i32,
    pub(crate) alpha: i32,
    pub(crate) beta: i32,
    pub(crate) bound: Option<Bound>,
    pub(crate) cut_node: bool,
}

/// Whether a stored result ends a node off the PV: deep enough (one ply
/// less is enough for a score below beta), on the window's side, and for a
/// bound against the node's expected type, deeper than a few plies.
#[inline(always)]
pub(crate) fn tt_cuts(s: &Seeds, i: &TtCutInput) -> bool {
    let typed = |expected: bool| expected || i.depth > s.tt_cut_typed_depth;
    !i.pv
        && !i.excluded
        && i.tt_depth > i.depth - i32::from(i.tt_score < i.beta)
        && is_valid(i.tt_score)
        && match i.bound {
            Some(Bound::Upper) => i.tt_score <= i.alpha && typed(!i.cut_node),
            Some(Bound::Lower) => i.tt_score >= i.beta && typed(i.cut_node),
            _ => true,
        }
}

/// The history rewards of a quiet TT move whose cutoff ends the node: its
/// quiet history's and its continuations'.
#[inline(always)]
pub(crate) fn tt_cut_bonus(s: &Seeds, depth: i32) -> (i32, i32) {
    (
        capped(
            s.tt_bonus_quiet_slope,
            depth,
            s.tt_bonus_quiet_offset,
            s.tt_bonus_quiet_cap,
        ),
        capped(
            s.tt_bonus_cont_slope,
            depth,
            s.tt_bonus_cont_offset,
            s.tt_bonus_cont_cap,
        ),
    )
}

/// The node's depth after hindsight: one ply more when the parent reduced
/// this move a lot and the evaluation fell across it, one less off a PV
/// line when it rose past a margin after any reduction.
#[inline(always)]
pub(crate) fn hindsight_depth(
    s: &Seeds,
    depth: i32,
    parent_reduction: i32,
    eval_delta: i32,
    tt_pv: bool,
) -> i32 {
    let mut depth = depth;
    if parent_reduction >= s.hindsight_reduction && eval_delta < 0 {
        depth += 1;
    }
    if !tt_pv && depth >= 2 && parent_reduction > 0 && eval_delta > s.hindsight_rise {
        depth -= 1;
    }
    depth
}

// ---------------------------------------------------------------------------
// Before the moves
// ---------------------------------------------------------------------------

/// Razoring's inputs.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RazorInput {
    pub(crate) pv: bool,
    pub(crate) in_check: bool,
    pub(crate) estimate: i32,
    pub(crate) alpha: i32,
    pub(crate) depth: i32,
    pub(crate) tt_quiet: bool,
    pub(crate) bound: Option<Bound>,
}

/// Whether the node hands over to quiescence: off the PV, out of check, so
/// far below alpha that only a tactic helps, alpha short of decisive, and
/// neither a quiet TT move nor a lower bound saying there is more here.
#[inline(always)]
pub(crate) fn razors(s: &Seeds, i: &RazorInput) -> bool {
    !i.pv
        && !i.in_check
        && i.estimate < i.alpha - s.razor_base - s.razor_square * i.depth * i.depth
        && i.alpha < s.razor_alpha_limit
        && !i.tt_quiet
        && i.bound != Some(Bound::Lower)
}

/// Reverse futility's inputs.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RfpInput {
    pub(crate) tt_pv: bool,
    pub(crate) in_check: bool,
    pub(crate) excluded: bool,
    pub(crate) estimate: i32,
    pub(crate) beta: i32,
    pub(crate) depth: i32,
    pub(crate) improvement: i32,
    pub(crate) correction: i32,
    /// No piece of the side to move stands attacked.
    pub(crate) unthreatened: bool,
}

/// How far above beta the estimate must stand, never below 2.
#[inline(always)]
pub(crate) fn rfp_margin(s: &Seeds, i: &RfpInput) -> i32 {
    (s.rfp_square * i.depth * i.depth / 128 - s.rfp_improvement * i.improvement / 1024
        + s.rfp_linear * i.depth
        + s.rfp_correction * i.correction.abs() / 1024
        - s.rfp_unthreatened * i32::from(i.unthreatened)
        - s.rfp_constant)
        .max(2)
}

/// What reverse futility returns: the estimate moved part of the way to beta.
#[inline(always)]
pub(crate) fn rfp_value(s: &Seeds, estimate: i32, beta: i32) -> i32 {
    lerp(estimate, beta, s.rfp_t)
}

/// Whether reverse futility ends the node: off a PV line, out of check, not
/// in an exclusion search, and above beta by the margin, short of decisive
/// scores.
#[inline(always)]
pub(crate) fn rfp_cuts(s: &Seeds, i: &RfpInput) -> bool {
    !i.tt_pv
        && !i.in_check
        && !i.excluded
        && i.estimate >= i.beta + rfp_margin(s, i)
        && !is_loss(i.beta)
        && !is_win(i.estimate)
}

/// The null move's gate.
#[derive(Copy, Clone, Debug)]
pub(crate) struct NullGateInput {
    pub(crate) cut_node: bool,
    pub(crate) in_check: bool,
    pub(crate) excluded: bool,
    /// The node may search a singular extension.
    pub(crate) singular: bool,
    pub(crate) estimate: i32,
    pub(crate) beta: i32,
    pub(crate) depth: i32,
    pub(crate) tt_pv: bool,
    pub(crate) improvement: i32,
    /// Beta cutoffs among the children so far.
    pub(crate) child_cutoffs: i32,
    pub(crate) ply: i32,
    /// Inside a verification, the first ply that may pass again.
    pub(crate) nmp_min_ply: i32,
    /// Material on the board in exchange values.
    pub(crate) material: i32,
    /// A lower bound won by the TT move capturing a knight or more.
    pub(crate) tt_capture_refutes: bool,
}

/// How far above beta the estimate must stand for a null move, never below
/// 2: less with depth, improvement and few child cutoffs, more on a PV line.
#[inline(always)]
pub(crate) fn nmp_margin(s: &Seeds, i: &NullGateInput) -> i32 {
    (-s.nmp_depth * i.depth + s.nmp_tt_pv * i32::from(i.tt_pv)
        - s.nmp_improvement * i.improvement / 1024
        - s.nmp_cutoffs * i32::from(i.child_cutoffs < 2)
        + s.nmp_base)
        .max(2)
}

/// Whether the node passes: at an expected cut node out of check, outside a
/// verification's region, with material on the board, far enough above
/// beta, and neither a singular candidate nor a capture refutation stored.
#[inline(always)]
pub(crate) fn nmp_gate(s: &Seeds, i: &NullGateInput) -> bool {
    i.cut_node
        && !i.in_check
        && !i.excluded
        && !i.singular
        && i.estimate >= i.beta + nmp_margin(s, i)
        && i.ply >= i.nmp_min_ply
        && i.material > s.nmp_material
        && !is_loss(i.beta)
        && !is_win(i.estimate)
        && !i.tt_capture_refutes
}

/// The null move's reduction in plies.
#[inline(always)]
pub(crate) fn nmp_reduction(
    s: &Seeds,
    improving: bool,
    depth: i32,
    estimate: i32,
    beta: i32,
) -> i32 {
    (s.nmp_r_base
        + s.nmp_r_improving * i32::from(improving)
        + s.nmp_r_depth * depth
        + s.nmp_r_surplus * (estimate - beta).clamp(0, s.nmp_r_surplus_cap) / 128)
        / 1024
}

/// The bound the null search must hold: a stored lower bound below beta and
/// deep enough, which already says the node is worth that much, else beta.
#[inline(always)]
pub(crate) fn null_bound(
    depth: i32,
    beta: i32,
    tt_score: i32,
    bound: Option<Bound>,
    tt_depth: i32,
) -> i32 {
    if is_valid(tt_score) && beta > tt_score && bound == Some(Bound::Lower) && depth - 2 <= tt_depth
    {
        tt_score
    } else {
        beta
    }
}

/// What the null search's score decides.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum NullOutcome {
    /// It did not hold its bound: search the moves.
    Fail,
    /// It holds beta: return the null search's score.
    Cut,
    /// Search the node again at this depth, without passing, before trusting it.
    Verify(i32),
}

/// The null search's score against its bound (beta, or a stored lower bound
/// below it): a non-decisive score at or above the bound cuts at once when
/// shallow or inside a verification, and is verified otherwise.
#[inline(always)]
pub(crate) fn null_outcome(
    s: &Seeds,
    score: i32,
    null_beta: i32,
    beta: i32,
    depth: i32,
    reduction: i32,
    nmp_min_ply: i32,
) -> NullOutcome {
    if score < null_beta || is_decisive(score) {
        return NullOutcome::Fail;
    }
    if (nmp_min_ply > 0 || depth < s.nmp_verify_depth) && score >= beta {
        return NullOutcome::Cut;
    }
    NullOutcome::Verify(if score < beta {
        depth / 2
    } else {
        depth - reduction
    })
}

/// ProbCut's raised beta.
#[inline(always)]
pub(crate) fn probcut_beta(s: &Seeds, beta: i32, improving: bool) -> i32 {
    beta + s.probcut_margin - s.probcut_improving * i32::from(improving)
}

/// ProbCut's gate.
#[derive(Copy, Clone, Debug)]
pub(crate) struct ProbcutGateInput {
    pub(crate) cut_node: bool,
    pub(crate) beta: i32,
    pub(crate) probcut_beta: i32,
    pub(crate) tt_score: i32,
    pub(crate) eval: i32,
    pub(crate) tt_quiet: bool,
}

/// Whether ProbCut runs: at an expected cut node short of a winning beta,
/// where a stored score clears the raised beta and is not decisive (or,
/// with none, the evaluation reaches beta), and the TT move is not quiet.
#[inline(always)]
pub(crate) fn probcut_gate(i: &ProbcutGateInput) -> bool {
    let promising = if is_valid(i.tt_score) {
        i.tt_score >= i.probcut_beta && !is_decisive(i.tt_score)
    } else {
        i.eval >= i.beta
    };
    i.cut_node && !is_win(i.beta) && promising && !i.tt_quiet
}

/// ProbCut's verification after a capture's quiescence score.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProbcutDepth {
    /// The full verification depth.
    pub(crate) base: i32,
    /// The depth after the quiescence score's surplus.
    pub(crate) depth: i32,
    /// The bound the shallower verification must clear.
    pub(crate) adjusted_beta: i32,
}

/// The verification is shallower the more the quiescence score cleared the
/// raised beta, and must then clear a beta raised per ply saved.
#[inline(always)]
pub(crate) fn probcut_depth(
    s: &Seeds,
    depth: i32,
    improving: bool,
    score: i32,
    probcut_beta: i32,
) -> ProbcutDepth {
    let base = (depth - s.probcut_depth_base - i32::from(improving)).max(0);
    let shallower = (base - (score - probcut_beta) / s.probcut_depth_per).clamp(0, base);
    ProbcutDepth {
        base,
        depth: shallower,
        adjusted_beta: (probcut_beta + s.probcut_raise * (base - shallower)).min(INF_SCORE),
    }
}

/// What a ProbCut cutoff returns.
#[inline(always)]
pub(crate) fn probcut_value(s: &Seeds, score: i32, beta: i32) -> i32 {
    if is_decisive(score) {
        score
    } else {
        lerp(score, beta, s.probcut_t)
    }
}

// ---------------------------------------------------------------------------
// Singular search
// ---------------------------------------------------------------------------

/// Whether the TT move is a singular candidate: deep enough, from an entry
/// nearly as deep that is not an upper bound, with a score short of
/// decisive. A PV line asks one ply more.
#[inline(always)]
pub(crate) fn singular_candidate(
    s: &Seeds,
    depth: i32,
    tt_pv: bool,
    tt_depth: i32,
    bound: Option<Bound>,
    tt_score: i32,
) -> bool {
    depth >= s.singular_depth + i32::from(tt_pv)
        && tt_depth >= depth - s.singular_tt_depth
        && bound != Some(Bound::Upper)
        && is_valid(tt_score)
        && !is_decisive(tt_score)
}

/// How far below the stored score the exclusion search's beta sits: the
/// depth (a quarter of it, rounded up, when exact), and the depth again at a
/// PV-line node searched with a null window.
#[inline(always)]
pub(crate) fn singular_margin(s: &Seeds, depth: i32, exact: bool, tt_pv_off_pv: bool) -> i32 {
    let span = if exact {
        (depth + s.singular_exact_div - 1) / s.singular_exact_div
    } else {
        depth
    };
    span + depth * i32::from(tt_pv_off_pv)
}

/// The exclusion search's result and the node it was asked at.
#[derive(Copy, Clone, Debug)]
pub(crate) struct SingularInput {
    pub(crate) singular_score: i32,
    pub(crate) singular_beta: i32,
    pub(crate) beta: i32,
    pub(crate) tt_score: i32,
    pub(crate) pv: bool,
    pub(crate) tt_was_pv: bool,
    pub(crate) tt_quiet: bool,
    pub(crate) correction: i32,
    pub(crate) cut_node: bool,
    /// The exclusion search reached a move.
    pub(crate) searched_a_move: bool,
}

/// What the exclusion search decides about the TT move.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum SingularOutcome {
    /// Nothing changes.
    None,
    /// The TT move is searched this many plies deeper (less when negative).
    Extend(i32),
    /// Another move holds beta too: the node returns this score.
    MultiCut(i32),
    /// Another move beat the stored score: the TT move loses its slot.
    Demote,
}

/// Every other move failing low makes the TT move singular, extended by one
/// to three plies; another move at beta is a multi-cut; one above the stored
/// score demotes it; otherwise a stored score at beta, or a cut node,
/// searches it shallower.
#[inline(always)]
pub(crate) fn singular_outcome(s: &Seeds, i: &SingularInput) -> SingularOutcome {
    if i.singular_score < i.singular_beta {
        let new_pv = i32::from(i.pv && !i.tt_was_pv);
        let pv = i32::from(i.pv);
        let quiet = i32::from(i.tt_quiet);
        let correction = i.correction.abs();
        let double = s.double_pv * pv + s.double_new_pv * new_pv
            - s.double_quiet * quiet
            - s.double_correction * correction / 128;
        let triple = s.triple_pv * pv + s.triple_new_pv * new_pv
            - s.triple_quiet * quiet
            - s.triple_correction * correction / 128
            + s.triple_base;
        return SingularOutcome::Extend(
            1 + i32::from(i.singular_score < i.singular_beta - double)
                + i32::from(i.singular_score < i.singular_beta - triple),
        );
    }
    if i.singular_score >= i.beta && !is_decisive(i.singular_score) {
        return SingularOutcome::MultiCut(lerp(i.singular_score, i.beta, s.multicut_t));
    }
    if i.singular_score > i.tt_score && i.searched_a_move {
        return SingularOutcome::Demote;
    }
    if i.tt_score >= i.beta || i.cut_node {
        return SingularOutcome::Extend(s.negative_extension);
    }
    SingularOutcome::None
}

/// Whether a node with no singular search extends its first move: a shallow
/// expected cut node out of check, well below alpha.
#[inline(always)]
pub(crate) fn ldse_extends(
    s: &Seeds,
    singular_searched: bool,
    depth: i32,
    in_check: bool,
    cut_node: bool,
    estimate: i32,
    alpha: i32,
) -> bool {
    !singular_searched
        && depth <= s.ldse_depth
        && !in_check
        && cut_node
        && estimate <= alpha - s.ldse_margin
}

// ---------------------------------------------------------------------------
// The move loop's pruning
// ---------------------------------------------------------------------------

/// A move's pruning inputs.
#[derive(Copy, Clone, Debug)]
pub(crate) struct MovePruneInput {
    pub(crate) in_check: bool,
    pub(crate) direct_check: bool,
    pub(crate) quiet: bool,
    pub(crate) depth: i32,
    /// The move's history: quiet, or noisy.
    pub(crate) history: i32,
    pub(crate) eval: i32,
    pub(crate) alpha: i32,
    pub(crate) beta: i32,
    pub(crate) improvement: i32,
    pub(crate) correction: i32,
    pub(crate) move_count: i32,
    /// The picker is down to the losing noisy moves.
    pub(crate) bad_stage: bool,
}

/// Late-move pruning's move-count limit.
#[inline(always)]
pub(crate) fn lmp_limit(s: &Seeds, i: &MovePruneInput) -> i32 {
    (s.lmp_base
        + s.lmp_improvement * i.improvement / 16
        + s.lmp_square * i.depth * i.depth
        + s.lmp_history * i.history / 1024)
        / 1024
}

/// Whether late-move pruning skips the remaining quiets.
#[inline(always)]
pub(crate) fn lmp_prunes(s: &Seeds, i: &MovePruneInput) -> bool {
    !i.in_check && !i.direct_check && i.quiet && !is_win(i.beta) && i.move_count >= lmp_limit(s, i)
}

/// Quiet futility's value: what the move might lift the evaluation to.
#[inline(always)]
pub(crate) fn futility_value(s: &Seeds, i: &MovePruneInput) -> i32 {
    i.eval
        + s.fp_linear * i.depth
        + s.fp_history * i.history / 1024
        + s.fp_above_beta * i32::from(i.eval >= i.beta)
        + s.fp_correction * i.correction.abs() / 1024
        - s.fp_constant
}

/// Whether quiet futility skips the remaining quiets.
#[inline(always)]
pub(crate) fn futility_prunes(s: &Seeds, i: &MovePruneInput) -> bool {
    !i.in_check
        && !i.direct_check
        && i.quiet
        && i.depth < s.fp_depth
        && futility_value(s, i) <= i.alpha
}

/// Bad-noisy futility's value.
#[inline(always)]
pub(crate) fn bad_noisy_value(s: &Seeds, i: &MovePruneInput) -> i32 {
    i.eval + s.bnfp_linear * i.depth + s.bnfp_history * i.history / 1024 + s.bnfp_constant
}

/// Whether bad-noisy futility ends the move loop.
#[inline(always)]
pub(crate) fn bad_noisy_prunes(s: &Seeds, i: &MovePruneInput) -> bool {
    !i.in_check
        && !i.direct_check
        && i.depth < s.bnfp_depth
        && i.bad_stage
        && bad_noisy_value(s, i) <= i.alpha
}

/// Whether history pruning skips a quiet.
#[inline(always)]
pub(crate) fn history_prunes(s: &Seeds, i: &MovePruneInput) -> bool {
    !i.in_check && i.quiet && i.depth < s.hp_depth && i.history < -s.hp_slope * i.depth
}

/// The SEE a move must reach to be searched.
#[inline(always)]
pub(crate) fn see_threshold(s: &Seeds, quiet: bool, depth: i32, history: i32) -> i32 {
    if quiet {
        (-s.see_quiet_square * depth * depth + s.see_quiet_linear * depth
            - s.see_quiet_history * history / 1024
            + s.see_quiet_constant)
            .min(0)
    } else {
        (-s.see_noisy_square * depth * depth
            - s.see_noisy_linear * depth
            - s.see_noisy_history * history / 1024
            + s.see_noisy_constant)
            .min(0)
    }
}

// ---------------------------------------------------------------------------
// Reductions
// ---------------------------------------------------------------------------

/// What both reduction formulas read about the node and the move.
#[derive(Copy, Clone, Debug)]
pub(crate) struct ReductionInput {
    pub(crate) depth: i32,
    pub(crate) improvement: i32,
    pub(crate) correction: i32,
    /// Alpha has been raised at this node.
    pub(crate) exact: bool,
    pub(crate) tt_score: i32,
    pub(crate) tt_depth: i32,
    pub(crate) alpha: i32,
    pub(crate) beta: i32,
    pub(crate) quiet: bool,
    pub(crate) history: i32,
    pub(crate) estimate: i32,
    /// Plies since the last node searched without a reduction, at most 8.
    pub(crate) since_critical: i32,
    pub(crate) pv: bool,
    pub(crate) root_delta: i32,
    /// The line's accumulated lateness to this move.
    pub(crate) laterality: i32,
    pub(crate) tt_pv: bool,
    pub(crate) cut_node: bool,
    pub(crate) tt_move_null: bool,
    pub(crate) gives_check: bool,
    pub(crate) child_cutoffs: i32,
    /// The TT move's score at this node, or `VALUE_NONE`.
    pub(crate) tt_move_score: i32,
    /// The exclusion search's score, or `VALUE_NONE`.
    pub(crate) singular_score: i32,
    pub(crate) parent_reduction: i32,
    /// The node count's phase for this formula, `0..128`.
    pub(crate) spread: i32,
    /// This move is the TT move.
    pub(crate) is_tt_move: bool,
}

/// The node count's phase for a reduction's spread: a per-thread offset
/// keeps threads from reducing in step.
#[inline(always)]
pub(crate) fn spread(nodes: u64, thread: u64, per_thread: i32) -> i32 {
    let per_thread = u64::try_from(per_thread).unwrap_or(0);
    i32::try_from(nodes.wrapping_add(thread * per_thread) % 128)
        .unwrap_or_else(|_| unreachable!("below 128"))
}

/// The late-move reduction in 1024ths of a ply.
#[inline(always)]
pub(crate) fn late_move_reduction(s: &Seeds, i: &ReductionInput) -> i32 {
    let tt_valid = is_valid(i.tt_score);
    let mut r = s.lmr_log * log2(i.depth);
    r -= (s.lmr_improvement * i.improvement / 128)
        .clamp(s.lmr_improvement_min, s.lmr_improvement_max);
    r -= s.lmr_correction * i.correction.abs() / 1024;
    r += s.lmr_exact * i32::from(i.exact);
    r += s.lmr_tt_below * i32::from(tt_valid && i.tt_score <= i.alpha);
    r += s.lmr_tt_shallow * i32::from(tt_valid && i.tt_depth < i.depth);
    r += s.lmr_win_beta * i32::from(is_win(i.beta));
    if i.quiet {
        r += s.lmr_quiet;
        r -= s.lmr_quiet_history * i.history / 1024;
        r += s.lmr_alpha_gap
            * (i.alpha - i.estimate).clamp(s.lmr_alpha_gap_min, s.lmr_alpha_gap_max)
            / 128;
    } else {
        r += s.lmr_noisy;
        r -= s.lmr_noisy_history * i.history / 1024;
    }
    r -= s.lmr_critical * i.since_critical;
    if i.pv {
        r -= s.lmr_pv + s.lmr_pv_window * (i.beta - i.alpha) / i.root_delta;
    } else {
        r -= s.lmr_non_pv - s.lmr_laterality * i.laterality;
    }
    if i.tt_pv {
        r -= s.lmr_tt_pv;
        r -= s.lmr_tt_pv_above * i32::from(tt_valid && i.tt_score > i.alpha);
        r -= s.lmr_tt_pv_deep * i32::from(tt_valid && i.tt_depth >= i.depth);
    } else if i.cut_node {
        r += s.lmr_cut;
        r += s.lmr_cut_no_tt_move * i32::from(i.tt_move_null);
    }
    if i.gives_check {
        r -= s.lmr_check;
    }
    if i.child_cutoffs > 2 {
        r += s.lmr_cutoffs;
        r += s.lmr_cutoffs_all * i32::from(!i.pv && !i.cut_node);
    }
    if is_valid(i.tt_move_score) && is_valid(i.singular_score) {
        let margin = i.tt_move_score - i.singular_score;
        r += (s.lmr_singular * (margin - s.lmr_singular_offset) / 128).clamp(0, s.lmr_singular_cap);
    }
    if !i.pv && i.parent_reduction > r + s.lmr_parent_gap {
        r += s.lmr_parent;
    }
    r + i.spread - s.lmr_spread_centre
}

/// The depth a reduced move is searched at: never below one ply, never more
/// than two above the unreduced depth, two plies deeper on a PV line.
#[inline(always)]
pub(crate) fn lmr_depth(new_depth: i32, reduction: i32, pv: bool) -> i32 {
    (new_depth - reduction / 1024).clamp(1, new_depth + 2) + 2 * i32::from(pv)
}

/// The unreduced depth after a reduced move beat alpha: a ply deeper past a
/// wide margin over the best score, a ply shallower below a narrow one, not
/// at the root.
#[inline(always)]
pub(crate) fn research_depth(
    s: &Seeds,
    root: bool,
    score: i32,
    best_score: i32,
    new_depth: i32,
) -> i32 {
    if root {
        return new_depth;
    }
    new_depth + i32::from(score > best_score + s.research_deeper)
        - i32::from(score < best_score + s.research_shallower)
}

/// The reduction, in 1024ths, of a move LMR does not reduce.
#[inline(always)]
pub(crate) fn unreduced_reduction(s: &Seeds, i: &ReductionInput) -> i32 {
    let mut r = s.fds_log * log2(i.depth);
    r -= (s.fds_improvement * i.improvement / 128)
        .clamp(s.fds_improvement_min, s.fds_improvement_max);
    r -= s.fds_correction * i.correction.abs() / 1024;
    if i.quiet {
        r += s.fds_quiet;
        r -= s.fds_quiet_history * i.history / 1024;
    } else {
        r += s.fds_noisy;
        r -= s.fds_noisy_history * i.history / 1024;
    }
    if i.tt_pv {
        r -= s.fds_tt_pv;
        r -= s.fds_tt_pv_deep * i32::from(is_valid(i.tt_score) && i.tt_depth >= i.depth);
    } else if i.cut_node {
        r += s.fds_cut;
        r += s.fds_cut_no_tt_move * i32::from(i.tt_move_null);
    }
    if i.child_cutoffs > 2 {
        r += s.fds_cutoffs;
        r += s.fds_cutoffs_all * i32::from(!i.pv && !i.cut_node);
    }
    if is_valid(i.tt_move_score) && is_valid(i.singular_score) {
        let margin = i.tt_move_score - i.singular_score;
        r += (s.fds_singular * (margin - s.fds_singular_offset) / 128).clamp(0, s.fds_singular_cap);
    }
    if i.is_tt_move {
        r -= s.fds_tt_move;
    }
    if i.parent_reduction > r + s.fds_parent_gap {
        r += s.fds_parent;
    }
    r + i.spread - s.fds_spread_centre
}

/// The depth of a move LMR does not reduce: one ply less past one bar, two
/// past a second.
#[inline(always)]
pub(crate) fn unreduced_depth(s: &Seeds, new_depth: i32, reduction: i32) -> i32 {
    new_depth - i32::from(reduction >= s.fds_one_ply) - i32::from(reduction >= s.fds_two_plies)
}

// ---------------------------------------------------------------------------
// Updates
// ---------------------------------------------------------------------------

/// The history amounts after a node found a best move.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct UpdateAmounts {
    pub(crate) noisy_bonus: i32,
    pub(crate) noisy_malus: i32,
    pub(crate) quiet_bonus: i32,
    pub(crate) quiet_malus: i32,
    pub(crate) cont_bonus: i32,
    pub(crate) cont_malus: i32,
    /// The parent's quiet move, refuted by this node's first moves.
    pub(crate) parent_malus: i32,
    /// A quiet best move found only on a re-search.
    pub(crate) research_bonus: i32,
}

/// The amounts for a node at `depth` whose best move came after `noisy` and
/// `quiet` other moves: bonuses grow with depth and shrink at a cut node,
/// maluses grow with depth and shrink with the moves that share them.
#[inline(always)]
pub(crate) fn update_amounts(
    s: &Seeds,
    depth: i32,
    cut_node: bool,
    noisy: i32,
    quiet: i32,
) -> UpdateAmounts {
    let cut = i32::from(cut_node);
    UpdateAmounts {
        noisy_bonus: (s.noisy_bonus_slope * depth).min(s.noisy_bonus_cap)
            - s.noisy_bonus_offset
            - s.noisy_bonus_cut * cut,
        noisy_malus: (s.noisy_malus_slope * depth).min(s.noisy_malus_cap)
            - s.noisy_malus_offset
            - s.noisy_malus_per * noisy,
        quiet_bonus: (s.quiet_bonus_slope * depth).min(s.quiet_bonus_cap)
            - s.quiet_bonus_offset
            - s.quiet_bonus_cut * cut,
        quiet_malus: (s.quiet_malus_slope * depth).min(s.quiet_malus_cap)
            - s.quiet_malus_offset
            - s.quiet_malus_per * quiet,
        cont_bonus: (s.cont_bonus_slope * depth).min(s.cont_bonus_cap)
            - s.cont_bonus_offset
            - s.cont_bonus_cut * cut,
        cont_malus: (s.cont_malus_slope * depth).min(s.cont_malus_cap)
            - s.cont_malus_offset
            - s.cont_malus_per * quiet,
        parent_malus: capped(
            s.parent_malus_slope,
            depth,
            s.parent_malus_offset,
            s.parent_malus_cap,
        ),
        research_bonus: capped(
            s.research_bonus_slope,
            depth,
            s.research_bonus_offset,
            s.research_bonus_cap,
        ),
    }
}

/// How much of its malus the `index`-th earlier quiet takes, in 1024ths:
/// later ones less.
#[inline(always)]
pub(crate) fn malus_fade(s: &Seeds, index: i32) -> i32 {
    let denominator = 1024 + s.malus_fade * index;
    1024 * 1024 / (denominator * denominator / 1024)
}

/// The parent's quiet move after this node failed low.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RewardInput {
    pub(crate) parent_move_count: i32,
    /// The parent's move was its TT move.
    pub(crate) parent_tt: bool,
    pub(crate) in_check: bool,
    pub(crate) best_score: i32,
    pub(crate) eval: i32,
    pub(crate) parent_eval: i32,
    pub(crate) depth: i32,
}

/// The reward to a parent's quiet move that this node failed low against:
/// its quiet history's amount and its factor, and the grandparent's
/// continuation's.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Reward {
    pub(crate) factor: i32,
    pub(crate) bonus: i32,
    pub(crate) grandparent: i32,
}

/// The reward grows with the parent's move order, for its TT move, and
/// when this node fell well below its own or the parent's evaluation.
#[inline(always)]
pub(crate) fn fail_low_reward(s: &Seeds, i: &RewardInput) -> Reward {
    let factor = s.reward_base
        + (s.reward_count * i.parent_move_count).min(s.reward_count_cap)
        + s.reward_tt_move * i32::from(i.parent_tt)
        + s.reward_below_eval
            * i32::from(!i.in_check && i.best_score <= i.eval - s.reward_below_eval_margin)
        + s.reward_below_parent
            * i32::from(
                is_valid(i.parent_eval)
                    && i.best_score <= -i.parent_eval - s.reward_below_parent_margin,
            );
    Reward {
        factor,
        bonus: factor * capped(s.reward_slope, i.depth, s.reward_offset, s.reward_cap) / 128,
        grandparent: capped(
            s.reward_grand_slope,
            i.depth,
            s.reward_grand_offset,
            s.reward_grand_cap,
        ),
    }
}

/// The reward to a parent's noisy move that this node failed low against.
#[inline(always)]
pub(crate) fn noisy_fail_low_reward(s: &Seeds, depth: i32) -> i32 {
    (s.reward_noisy_slope * depth).min(s.reward_noisy_cap)
}

/// The correction's update from a node's result.
#[derive(Copy, Clone, Debug)]
pub(crate) struct CorrectionInput {
    pub(crate) in_check: bool,
    pub(crate) noisy_best: bool,
    pub(crate) bound: Bound,
    pub(crate) best_score: i32,
    pub(crate) eval: i32,
    pub(crate) depth: i32,
}

/// The correction tables learn the result's residual, scaled by depth and
/// clamped, only where the evaluation can be blamed: out of check, with no
/// noisy best move, and with a bound that agrees with the residual's sign.
#[inline(always)]
pub(crate) fn correction_update(s: &Seeds, i: &CorrectionInput) -> Option<i32> {
    let disagrees = match i.bound {
        Bound::Upper => i.best_score >= i.eval,
        Bound::Lower => i.best_score <= i.eval,
        Bound::Exact => false,
    };
    if i.in_check || i.noisy_best || disagrees {
        return None;
    }
    let product =
        i64::from(s.corr_slope) * i64::from(i.depth) * i64::from(i.best_score - i.eval) / 128;
    Some(
        i32::try_from(product.clamp(i64::from(s.corr_min), i64::from(s.corr_max)))
            .unwrap_or_else(|_| unreachable!("clamped into i32 bounds")),
    )
}

/// A history entry after a bonus: the bonus, clamped to the table's limit,
/// less the part that would carry the entry past it.
#[inline(always)]
pub(crate) fn gravity(max: i32, entry: i32, bonus: i32) -> i32 {
    let bonus = bonus.clamp(-max, max);
    entry + bonus - bonus.abs() * entry / max
}

// ---------------------------------------------------------------------------
// Quiescence
// ---------------------------------------------------------------------------

/// Whether a stored result ends a quiescence node: any bound on the
/// window's side, unless the score is decisive at a PV node.
#[inline(always)]
pub(crate) fn q_tt_cuts(
    pv: bool,
    tt_score: i32,
    bound: Option<Bound>,
    alpha: i32,
    beta: i32,
) -> bool {
    is_valid(tt_score)
        && (!pv || !is_decisive(tt_score))
        && match bound {
            Some(Bound::Upper) => tt_score <= alpha,
            Some(Bound::Lower) => tt_score >= beta,
            _ => true,
        }
}

/// The stand pat: no score in check, else the evaluation, tightened by a
/// stored score on its side (not a decisive one at a PV node).
#[inline(always)]
pub(crate) fn stand_pat(
    in_check: bool,
    pv: bool,
    eval: i32,
    tt_score: i32,
    bound: Option<Bound>,
) -> i32 {
    if in_check {
        return -INF_SCORE;
    }
    let tightens = match bound {
        Some(Bound::Upper) => tt_score < eval,
        Some(Bound::Lower) => tt_score > eval,
        _ => true,
    };
    if is_valid(tt_score) && (!pv || !is_decisive(tt_score)) && tightens {
        tt_score
    } else {
        eval
    }
}

/// A quiescence fail-high moved part of the way to beta, unless either is
/// decisive.
#[inline(always)]
fn q_soften(score: i32, beta: i32, t: i32) -> i32 {
    if score >= beta && !is_decisive(score) && !is_decisive(beta) {
        lerp(score, beta, t)
    } else {
        score
    }
}

/// What a stand pat at or above beta returns.
#[inline(always)]
pub(crate) fn q_stand_pat_value(s: &Seeds, best_score: i32, beta: i32) -> i32 {
    q_soften(best_score, beta, s.q_stand_pat_t)
}

/// What a quiescence node returns from its best score.
#[inline(always)]
pub(crate) fn q_tail_value(s: &Seeds, best_score: i32, beta: i32) -> i32 {
    q_soften(best_score, beta, s.q_tail_t)
}

/// Whether quiescence stops at this move: from the move-count limit on,
/// unless the move gives direct check.
#[inline(always)]
pub(crate) fn q_stops(s: &Seeds, move_count: i32, direct_check: bool) -> bool {
    move_count >= s.q_move_limit && !direct_check
}

/// The SEE a quiescence move must reach: an eighth of alpha's distance above
/// the evaluation, less the correction's size up to a cap, a constant and
/// the move's history.
#[inline(always)]
pub(crate) fn q_see_threshold(
    s: &Seeds,
    alpha: i32,
    eval: i32,
    correction: i32,
    history: i32,
) -> i32 {
    (alpha - eval) / s.q_see_divisor
        - correction.abs().min(s.q_see_correction_cap)
        - s.q_see_constant
        - history / s.q_see_history
}

// ---------------------------------------------------------------------------
// The root window
// ---------------------------------------------------------------------------

/// The aspiration window's half-width at an iteration's start: narrower the
/// longer the score and the best move have held, wider for a large score.
#[inline(always)]
pub(crate) fn window_delta(s: &Seeds, eval_stability: i32, pv_stability: i32, average: i32) -> i32 {
    s.window_base - eval_stability.min(pv_stability).min(s.window_stability_cap)
        + average * average / s.window_square
}

/// The root window and its widening state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Window {
    pub(crate) alpha: i32,
    pub(crate) beta: i32,
    pub(crate) delta: i32,
    /// Plies the next root search is shallower after fail-highs.
    pub(crate) reduction: i32,
}

/// The window round a running average.
#[inline(always)]
pub(crate) fn open_window(average: i32, delta: i32) -> Window {
    Window {
        alpha: (average - delta).max(-INF_SCORE),
        beta: (average + delta).min(INF_SCORE),
        delta,
        reduction: 0,
    }
}

/// After a fail low: alpha from the failing score, beta drawn down to
/// alpha plus the half-width, the half-width grown.
#[inline(always)]
pub(crate) fn window_fail_low(s: &Seeds, w: Window, score: i32) -> Window {
    let alpha = (score - w.delta).max(-INF_SCORE);
    Window {
        alpha,
        beta: (alpha + w.delta).min(w.beta),
        delta: w.delta + s.window_low_growth * w.delta / 128,
        reduction: w.reduction,
    }
}

/// After a fail high: beta from the failing score, alpha raised toward the
/// old beta, one ply shallower unless the score is decisive, the half-width
/// grown more.
#[inline(always)]
pub(crate) fn window_fail_high(s: &Seeds, w: Window, score: i32) -> Window {
    Window {
        alpha: (w.beta - w.delta).max(w.alpha),
        beta: (score + w.delta).min(INF_SCORE),
        delta: w.delta + s.window_high_growth * w.delta / 128,
        reduction: if is_decisive(score) {
            w.reduction.min(1)
        } else {
            w.reduction + 1
        },
    }
}

/// Whether the iteration's score held near the running average.
#[inline(always)]
pub(crate) fn score_held(s: &Seeds, score: i32, average: i32) -> bool {
    (score - average).abs() < s.window_stable
}

// ---------------------------------------------------------------------------
// Draws and move ordering
// ---------------------------------------------------------------------------

/// A draw's score: two either side of zero by the node count, so a line
/// that can draw is not scored exactly level with every other draw.
#[inline(always)]
pub(crate) fn draw_score(nodes: u64) -> i32 {
    i32::try_from(nodes % 5).unwrap_or_else(|_| unreachable!("below 5")) - 2
}

/// A noisy move's ordering inputs.
#[derive(Copy, Clone, Debug)]
pub(crate) struct NoisyOrderInput {
    /// The captured piece's exchange value; zero for a push.
    pub(crate) victim: i32,
    pub(crate) history: i32,
    pub(crate) queen_promotion: bool,
    pub(crate) in_check: bool,
    /// The moving piece's index, pawn 0 to king 5.
    pub(crate) piece: i32,
}

/// A noisy move's ordering score: in check the cheaper evader first.
#[inline(always)]
pub(crate) fn noisy_order(s: &Seeds, i: &NoisyOrderInput) -> i32 {
    s.pick_victim * i.victim / 1024
        + i.history
        + s.pick_queen_promotion * i32::from(i.queen_promotion)
        + (s.pick_check_base - s.pick_check_piece * i.piece) * i32::from(i.in_check)
}

/// A quiet move's ordering inputs.
#[derive(Copy, Clone, Debug)]
pub(crate) struct QuietOrderInput {
    pub(crate) quiet: i32,
    pub(crate) pawn: i32,
    /// The continuation histories one, two, four and six plies back.
    pub(crate) cont: [i32; 4],
    /// The moving piece's index, pawn 0 to king 5.
    pub(crate) piece: usize,
    /// It leaves a square a cheaper enemy piece attacks.
    pub(crate) escape: bool,
    /// It gives direct check.
    pub(crate) check_square: bool,
    /// It lands where a cheaper enemy piece attacks.
    pub(crate) threatened_to: bool,
    /// It lands safely where it attacks an enemy piece.
    pub(crate) offense: bool,
    /// It moves a pawn from the castled king's wall.
    pub(crate) wall: bool,
}

/// A quiet move's ordering score.
#[inline(always)]
pub(crate) fn quiet_order(s: &Seeds, i: &QuietOrderInput) -> i32 {
    let mut score = s.pick_quiet * i.quiet / 1024 + s.pick_pawn * i.pawn / 1024;
    for (weight, value) in s.pick_cont.iter().zip(i.cont) {
        score += weight * value / 1024;
    }
    score
        + s.pick_escape[i.piece] * i32::from(i.escape)
        + s.pick_check_square * i32::from(i.check_square)
        - s.pick_threatened * i32::from(i.threatened_to)
        + s.pick_offense * i32::from(i.offense)
        - s.pick_wall * i32::from(i.wall)
}

/// The SEE a noisy move must reach to be ordered with the good ones: lower
/// the better its score, or past the first few good ones under a quiet TT
/// move, a material gain.
#[inline(always)]
pub(crate) fn good_noisy_threshold(s: &Seeds, score: i32, tt_quiet: bool, good_so_far: i32) -> i32 {
    if tt_quiet && good_so_far > s.pick_late_good {
        1
    } else {
        -score / s.pick_see_divisor + s.pick_see_offset
    }
}
