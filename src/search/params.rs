/// Declares every tunable search parameter **once**.
///
/// Before 9.0a each tunable lived in FOUR hand-synchronised places: the struct
/// field, the `Default` value, the tune-gated UCI `option` string (carrying its
/// own copy of default/min/max) and the setter arm (carrying its own copy of
/// the clamp range). Nothing checked that the four agreed — and on 2026-07-19
/// they did not: **12 UCI declarations advertised stale defaults** (e.g.
/// `SeePruningCoeff` said 83 while the baked value was 51, and all six `Hist*`
/// still advertised their pre-8.1 seeds). This macro makes that class of drift
/// unrepresentable: one line per parameter generates all four, so a bake can
/// only ever change one number.
///
/// Syntax: `struct Name, checks_module; field = default, "UciName", min..=max;`
/// — doc comments and plain `//` section comments pass through as normal.
macro_rules! search_params {
    (
        struct $name:ident, $checks:ident;
        $(
            $(#[$meta:meta])*
            $field:ident = $default:literal, $uci:literal, $min:literal ..= $max:literal;
        )+
    ) => {
        /// Tunable search parameters — every field is a UCI `spin` option in
        /// tune builds. Defaults are the current accepted integration-head
        /// values; the trailing comment on each declaration records its bake
        /// history. To re-tune, copy the weather-factory configs from
        /// `tools/spsa_configs/` and run `./tools/spsa.ps1` (see that
        /// directory's README).
        #[derive(Clone, Debug)]
        pub struct $name {
            $( $(#[$meta])* pub $field: i32, )+
        }

        impl Default for $name {
            fn default() -> Self {
                Self { $( $field: $default, )+ }
            }
        }

        impl $name {
            /// UCI `spin` declarations for every tunable, generated from the
            /// same literals as the defaults and clamps — they cannot disagree.
            /// Tune builds only (production must not advertise these).
            #[cfg(feature = "tune")]
            pub fn uci_option_strings() -> Vec<String> {
                vec![$(
                    format!(
                        "option name {} type spin default {} min {} max {}",
                        $uci, $default, $min, $max
                    ),
                )+]
            }

            /// Applies `setoption name <UciName> value <v>` (name matched
            /// case-insensitively), clamping to the declared range. Returns
            /// `false` if the name is not a tunable, so the caller can fall
            /// through to the engine options.
            #[cfg(feature = "tune")]
            pub fn set_uci_option(&mut self, name: &str, value: &str) -> bool {
                $(
                    if name.eq_ignore_ascii_case($uci) {
                        if let Ok(v) = value.parse::<i32>() {
                            self.$field = v.clamp($min, $max);
                        }
                        return true;
                    }
                )+
                false
            }
        }

        #[cfg(test)]
        mod $checks {
            use super::*;

            /// Every default must sit inside its own declared range. Cheap, but
            /// it is the invariant the four-way duplication used to break.
            #[test]
            fn defaults_are_within_declared_ranges() {
                let p = $name::default();
                $(
                    assert!(
                        ($min..=$max).contains(&p.$field),
                        "{} default {} is outside [{}, {}]",
                        $uci, p.$field, $min, $max
                    );
                )+
            }

            /// Ranges must be non-empty and ordered.
            #[test]
            fn declared_ranges_are_sane() {
                $( assert!($min < $max, "{} has min >= max", $uci); )+
            }
        }
    };
}

// The single source of truth for every tunable: `field = default, "UciName",
// min..=max;`. Struct field, Default value, UCI option string and setter clamp
// are all generated from these lines — see the `search_params!` docs above.
search_params! {
    struct SearchParams, generated_param_checks;

    /// Initial aspiration window half-width (centipawns).
    aspiration_delta = 13, "AspirationDelta", 5..=100;  // was 25 → 29 → 31 → 30 → 21

    // ── 10.2(a) aspiration shape ─────────────────────────────────────────────
    // The widening loop is parameterised so its shape can be SPSA'd rather than
    // hardcoded. EVERY default below reproduces the pre-10.2 behaviour exactly,
    // so this lands bench-identical and the tune activates it (principle #5).
    // That staging is deliberate: lesson 13 records that adopting a modern
    // aspiration shape WITHOUT re-tuning its constants measured −4.52, because
    // `AspirationDelta` and the pruning group were fitted around the old
    // dynamics. Ship the mechanism inert, fit it, then gate it.
    /// Delta growth per fail-LOW, in percent of the current delta.
    /// 150 reproduces the old `d + d/2` (both are `floor(3d/2)`).
    asp_growth_pct = 150, "AspGrowthPct", 100..=400;
    /// Delta growth per fail-HIGH, in percent. Separate from the fail-low side
    /// so the growth can become asymmetric — fail-highs and fail-lows carry
    /// different information, and nothing forced them to share a rate except
    /// that the old code was written as one expression. 150 = old behaviour.
    asp_growth_high_pct = 150, "AspGrowthHighPct", 100..=400;
    /// Additive term applied with the growth, so a small delta still escapes
    /// its own rounding. 5 = old behaviour.
    asp_growth_add = 5, "AspGrowthAdd", 0..=50;
    /// ⛔ TERMINATION BY CONSTRUCTION. After this many consecutive fails on one
    /// side, that side opens to ±INF unconditionally, so it cannot fail again;
    /// the loop is bounded at `2 × asp_max_fails` iterations regardless of
    /// score magnitude. This is what lets 10.2(a) retire the 7.0b hang guard,
    /// which special-cased mate scores and delta saturation instead.
    ///
    /// Seeded at 20 because the delta needs ~18 growth steps to saturate from
    /// the seed, so the counter never fires first and behaviour is unchanged.
    /// The interesting direction is DOWN — engines that re-search a bounded
    /// number of times before opening fully spend far fewer nodes on a
    /// runaway iteration — which is exactly what the tune explores.
    asp_max_fails = 20, "AspMaxFails", 1..=32;

    // ── Qsearch SEE thresholds (Phase 7.2 SEE bundle) ────────────────────────
    // Exposed so the `config_see` SPSA can re-tune SEE's consumers alongside
    // the pin-aware `see_ge` (lesson 15: a more accurate SEE de-tunes the
    // constants fitted around the old one). Defaults reproduce the prior
    // hardcoded literals exactly → bench-identical until re-tuned.
    /// Qsearch capture SEE-prune margin: search a capture only if
    /// `see_ge(alpha − stand_pat − qs_see_margin)` (clamped). Seed 200.
    qs_see_margin = 266, "QsSeeMargin", 0..=600;  // was 200 → 251 → 265
    /// Upper clamp on the qsearch SEE-prune threshold. Seed 200.
    qs_see_clamp_hi = 208, "QsSeeClampHi", 0..=600;  // was 200 → 218 → 212
    /// Qsearch bad-capture SEE floor: an ordering-SEE-negative capture is
    /// skipped unless `see_ge(qs_see_bad_floor)`. Seed −50.
    qs_see_bad_floor = -43, "QsSeeBadFloor", -400..=0;  // was -50 → -119 → -55

    /// Minimum non-pawn pieces the side to move must have for NMP.
    ///
    /// 1 = accepted baseline, i.e. exactly the existing
    /// `has_non_pawn_material` test. Zugzwang risk concentrates where the mover
    /// has almost nothing left to move: with one minor piece and pawns, "pass"
    /// and "move" can differ by the whole game. 2 or 3 demand progressively more
    /// material before a null is trusted.
    ///
    /// The existing guard is not removed — this tightens the same test, so 1
    /// reproduces it exactly and the `tests/zugzwang.rs` pawn-only assertions
    /// keep covering the boundary.
    nmp_min_non_pawn_pieces = 1, "NmpMinNonPawnPieces", 1..=3;

    /// 4.7c PROBCUT MOVE FILTER. Two constants for the entry contract of the
    /// speculative capture search; both are categorical-frozen defaults awaiting
    /// the cluster fit, not tuned values.
    ///
    /// `probcut_see_gap_scale` is a percentage applied to the gap the capture
    /// must bridge, `probcut_beta - static_eval`. At 100 the move must win at
    /// least the whole gap by SEE; at 0 the gap is ignored and the filter
    /// degenerates to the pre-4.7c `see_ge(mv, 0)`. The threshold is floored at
    /// 0, so this can only ever tighten the old contract, never loosen it —
    /// RAR-S55 v3 measured Rarog searching 5.17x the reference's normalised
    /// ProbCut moves and converting 32.6% of them against 71.9%.
    probcut_see_gap_scale = 100, "ProbCutSeeGapScale", 0..=100;

    /// Lazy-eval margin: if the tapered material + PST + pawn score already
    /// exceeds it, the positional block is skipped. Pushed into the evaluator
    /// at every search start; the default must equal `eval::LAZY_MARGIN`, the
    /// evaluator's own seed and the margin `texel` fits use.
    lazy_margin = 414, "LazyMargin", 200..=2000;

    // ── Time-management dynamic multipliers (Phase 5.1 TM group) ─────────────
    // The clock-mode between-iteration soft-stop scales `optimum_ms` by
    // falling-eval × best-move-instability × effort (`search_root` soft-stop block);
    // these are the 2.2 SF-seeded constants, exposed for the TM SPSA group.
    // Stored in ten-thousandths so the float defaults reconstruct bit-exactly
    // (`x / 10000.0` is correctly-rounded, identical to the original literal).
    // TM affects only clock play, never the depth-limited `bench` fingerprint.
    /// Overall multiplier on `optimum_ms` (10000 = ×1.0). The single
    /// highest-leverage TM knob; lets SPSA scale base time allocation.
    tm_opt_scale = 10_000, "TmOptScale", 5000..=20000;  // ×1.0
    /// Falling-eval base term. Seed 1187 (0.1187).
    tm_fall_base = 1_187, "TmFallBase", 0..=5000;  // 0.1187
    /// Falling-eval slope on `(prev_avg_score - score)`. Seed 221 (0.0221).
    tm_fall_slope = 221, "TmFallSlope", 0..=1000;  // 0.0221
    /// Best-move-instability base. Seed 11000 (1.10).
    tm_instab_base = 11_000, "TmInstabBase", 8000..=16000;  // 1.10
    /// Best-move-instability slope on `tot_best_move_changes`. Seed 22900 (2.29).
    tm_instab_slope = 22_900, "TmInstabSlope", 0..=50000;  // 2.29
    /// Effort factor at low effort (interp endpoint at t=0). Seed 9240 (0.924).
    tm_effort_high = 9_240, "TmEffortHigh", 6000..=12000;  // 0.924
    /// Effort factor at high effort (interp endpoint at t=1). Seed 7100 (0.71).
    tm_effort_low = 7_100, "TmEffortLow", 4000..=10000;  // 0.71
}

// The selectivity core's own coordinates. Seeds follow the handoff's seed
// rule: the donor's value converted to Rarog's scale (evaluation units x0.457,
// SEE units x0.75, plies, counts and history units unchanged), or the
// geometric mean of the classical-oracle and Rarog-fitted values where the
// donor's margin is sized for a far more accurate evaluation.
search_params! {
    struct CoreParams, generated_core_param_checks;

    // Correction update and blend.
    /// Update slope in 128ths: `bonus = slope * depth * residual / 128`.
    corr_update_slope = 217, "CoreCorrUpdateSlope", 32..=512;
    /// Clamps of one update, in table units (64 per evaluation unit).
    corr_update_min = -1_742, "CoreCorrUpdateMin", -8192..=-256;
    corr_update_max = 735, "CoreCorrUpdateMax", 128..=8192;
    /// Blend weights of the six tables, in 128ths.
    corr_weight_pawn = 121, "CoreCorrWeightPawn", 0..=384;
    corr_weight_minor = 42, "CoreCorrWeightMinor", 0..=384;
    corr_weight_non_pawn_white = 122, "CoreCorrWeightNonPawnWhite", 0..=384;
    corr_weight_non_pawn_black = 127, "CoreCorrWeightNonPawnBlack", 0..=384;
    corr_weight_cont2 = 165, "CoreCorrWeightCont2", 0..=384;
    corr_weight_cont4 = 147, "CoreCorrWeightCont4", 0..=384;
    // Corrected-eval formula, neutral at zero.
    /// Material scaling of the raw eval, in 64ths per starting-material unit.
    eval_material_scale = 21, "CoreEvalMaterialScale", -64..=64;
    /// Rule-50 damping, in percent of `eval * min(clock, 100) / 199`. The
    /// evaluator's own damping is compiled out under the core, so the eval the
    /// table stores does not depend on the clock and the search damps it here.
    eval_rule50_damping = 100, "CoreEvalRule50Damping", 0..=150;

    // Node-level pruning.
    /// Razoring margin `base + square * depth^2`, evaluation units.
    razor_base = 289, "CoreRazorBase", 50..=800;
    razor_square = 184, "CoreRazorSquare", 20..=400;
    /// Reverse-futility margin: `square/16 * depth^2 + linear * depth
    /// - improvement * improvement/1024 + correction * |corr|/1024 - threat *
    /// unthreatened + constant`, floored at 2.
    rfp_square = 143, "CoreRfpSquare", 0..=320;
    rfp_linear = 19, "CoreRfpLinear", -100..=200;
    rfp_improvement = 50, "CoreRfpImprovement", 0..=512;
    rfp_correction = 88, "CoreRfpCorrection", 0..=2048;
    rfp_threat = 20, "CoreRfpThreat", 0..=120;
    rfp_constant = -31, "CoreRfpConstant", -100..=100;
    /// Reverse-futility return, in 1024ths of the way from the estimate to beta.
    rfp_lerp = 586, "CoreRfpLerp", 0..=1024;
    /// Hindsight: a parent reduction (1024ths of a ply) at or above this
    /// deepens a child whose eval says the parent's opponent got worse.
    hindsight_deepen_reduction = 1_676, "CoreHindsightDeepenReduction", 512..=6144;
    /// Hindsight: an eval swing above this reduces a reduced child one ply.
    hindsight_reduce_margin = 31, "CoreHindsightReduceMargin", 0..=200;
    /// Internal iterative reduction: the least depth at which a missing or
    /// shallow TT move costs a ply.
    iir_min_depth = 4, "CoreIirMinDepth", 2..=10;

    // Move-loop pruning.
    /// Late-move pruning count, in 1024ths: `(base + improvement *
    /// improvement/16 + square * depth^2 + history * history/1024) / 1024`.
    lmp_base = 2_888, "CoreLmpBase", 0..=8192;
    lmp_improvement = 76, "CoreLmpImprovement", 0..=400;
    lmp_square = 402, "CoreLmpSquare", 256..=4096;
    lmp_history = 75, "CoreLmpHistory", 0..=400;
    /// Quiet futility value: `eval + base + linear * depth + history *
    /// history/1024 + above_beta * (eval >= beta) + correction * |corr|/1024`.
    fp_base = 147, "CoreFpBase", -100..=500;
    fp_linear = 35, "CoreFpLinear", 10..=300;
    fp_history = 44, "CoreFpHistory", 0..=200;
    fp_eval_above_beta = 39, "CoreFpEvalAboveBeta", 0..=200;
    fp_correction = 166, "CoreFpCorrection", 0..=2048;
    /// Bad-noisy futility value: `eval + base + linear * depth + history *
    /// history/1024 + victim value`.
    bnfp_base = 37, "CoreBnfpBase", -100..=400;
    bnfp_linear = 44, "CoreBnfpLinear", 10..=300;
    bnfp_history = 36, "CoreBnfpHistory", 0..=200;
    /// History pruning below `-slope * depth`.
    hp_slope = 1_019, "CoreHpSlope", 100..=4000;
    /// SEE-pruning allowance for quiets: `min(0, -square * depth^2 + linear *
    /// depth - history * history/1024 + constant)`, SEE units.
    see_quiet_square = 7, "CoreSeeQuietSquare", 0..=40;
    see_quiet_linear = 31, "CoreSeeQuietLinear", 0..=200;
    see_quiet_history = 18, "CoreSeeQuietHistory", 0..=120;
    see_quiet_constant = 20, "CoreSeeQuietConstant", -100..=100;
    /// SEE-pruning allowance for noisy moves: `min(0, -square * depth^2 -
    /// linear * depth - history * history/1024 + constant)`.
    see_noisy_square = 5, "CoreSeeNoisySquare", 0..=40;
    see_noisy_linear = 12, "CoreSeeNoisyLinear", 0..=200;
    see_noisy_history = 33, "CoreSeeNoisyHistory", 0..=120;
    see_noisy_constant = 29, "CoreSeeNoisyConstant", -100..=100;

    // Late-move reductions, in 1024ths of a ply.
    lmr_log = 150, "CoreLmrLog", 0..=1024;
    lmr_improvement = 411, "CoreLmrImprovement", 0..=2048;
    /// Bounds of the improvement term, in reduction units. Seeded at the
    /// donor's values; converting them to Rarog's evaluation scale measured
    /// worse (RAR-S74), so the fit decides them.
    lmr_improvement_clamp_lo = -162, "CoreLmrImprovementClampLo", -1024..=0;
    lmr_improvement_clamp_hi = 1_065, "CoreLmrImprovementClampHi", 0..=4096;
    lmr_correction = 3_505, "CoreLmrCorrection", 0..=8192;
    lmr_exact = 1_748, "CoreLmrExact", 0..=4096;
    lmr_tt_score_below_alpha = 526, "CoreLmrTtScoreBelowAlpha", 0..=2048;
    lmr_tt_shallow = 58, "CoreLmrTtShallow", 0..=2048;
    lmr_quiet = 1_932, "CoreLmrQuiet", 0..=6144;
    lmr_quiet_history = 316, "CoreLmrQuietHistory", 0..=1024;
    lmr_alpha_gap = 195, "CoreLmrAlphaGap", 0..=2048;
    /// Bounds of `alpha - estimated score` in the quiet term, evaluation
    /// units, seeded at the donor's values like the improvement bounds.
    lmr_alpha_gap_lo = -69, "CoreLmrAlphaGapLo", -512..=0;
    lmr_alpha_gap_hi = 84, "CoreLmrAlphaGapHi", 0..=512;
    lmr_noisy = 830, "CoreLmrNoisy", 0..=6144;
    lmr_noisy_history = 182, "CoreLmrNoisyHistory", 0..=1024;
    lmr_critical_ply = 171, "CoreLmrCriticalPly", 0..=512;
    lmr_pv = 399, "CoreLmrPv", 0..=2048;
    lmr_pv_window = 306, "CoreLmrPvWindow", 0..=2048;
    lmr_non_pv = 358, "CoreLmrNonPv", -1024..=1024;
    lmr_laterality = 24, "CoreLmrLaterality", 0..=256;
    lmr_tt_pv = 179, "CoreLmrTtPv", 0..=2048;
    lmr_tt_pv_score = 571, "CoreLmrTtPvScore", 0..=2048;
    lmr_tt_pv_depth = 633, "CoreLmrTtPvDepth", 0..=2048;
    lmr_cut_node = 2_071, "CoreLmrCutNode", 0..=4096;
    lmr_cut_node_no_tt_move = 2_073, "CoreLmrCutNodeNoTtMove", 0..=4096;
    lmr_gives_check = 1_030, "CoreLmrGivesCheck", 0..=4096;
    lmr_child_cutoffs = 1_334, "CoreLmrChildCutoffs", 0..=4096;
    lmr_child_cutoffs_all_node = 253, "CoreLmrChildCutoffsAllNode", 0..=2048;
    lmr_parent = 130, "CoreLmrParent", 0..=1024;
    /// Re-search a reduced move one ply deeper when it beats the best score
    /// by more than this, one ply shallower when by less than this.
    lmr_research_deeper = 35, "CoreLmrResearchDeeper", 0..=200;
    lmr_research_shallower = -1, "CoreLmrResearchShallower", -50..=50;

    // History update policy, history units.
    /// Quiet best-move bonus `min(slope * depth, cap) - 72 - 42 * cut_node`.
    hist_quiet_bonus_slope = 173, "CoreHistQuietBonusSlope", 32..=512;
    hist_quiet_bonus_cap = 1_532, "CoreHistQuietBonusCap", 256..=4096;
    /// Quiet malus `min(slope * depth, cap) - 46 - 31 * quiets searched`.
    hist_quiet_malus_slope = 197, "CoreHistQuietMalusSlope", 32..=512;
    hist_quiet_malus_cap = 1_405, "CoreHistQuietMalusCap", 256..=4096;
    /// Search-order fade of the quiet malus: the i-th quiet gets
    /// `1024^2 / (1024 + scale * i)^2` of it.
    hist_malus_index_scale = 42, "CoreHistMalusIndexScale", 0..=256;
    hist_cont_bonus_cap = 1_051, "CoreHistContBonusCap", 256..=4096;
    hist_noisy_bonus_cap = 894, "CoreHistNoisyBonusCap", 256..=4096;
    /// Quiet bonus to a TT move that cuts: `min(190 * depth - 81, cap)`.
    hist_tt_cutoff_bonus_cap = 1_727, "CoreHistTtCutoffBonusCap", 256..=4096;
    /// Base of the fail-low reward factor for the parent's quiet move.
    hist_fail_low_base = 92, "CoreHistFailLowBase", 0..=400;

    // Draw score.
}

// The proof-search cluster's coordinates. Seeds follow the core's rule: the
// donor's shape, Rarog's own fitted magnitude where one exists, otherwise the
// donor's value converted (evaluation units x0.457). Categoricals are
// switches, never SPSA coordinates.
search_params! {
    struct ProofParams, generated_proof_param_checks;

    // Null move.
    /// Entry margin above beta: `max(2, base - depth_term * depth/4 +
    /// tt_pv_term * tt_pv - improvement_term * improvement/1024 - cutoff *
    /// (child cutoffs < 2))`, evaluation units; the depth term is in
    /// quarters of a unit per ply.
    nmp_base = 180, "CoreNmpBase", 0..=400;
    nmp_depth = 16, "CoreNmpDepth", 0..=80;
    nmp_tt_pv = 48, "CoreNmpTtPv", 0..=200;
    nmp_improvement = 43, "CoreNmpImprovement", 0..=200;
    nmp_cutoff = 11, "CoreNmpCutoff", 0..=60;
    /// Reduction in 1024ths of a ply: `base + improving_term * improving +
    /// depth_term * depth + eval * clamp(estimate - beta, 0, clamp)/128`.
    nmp_r_base = 4_203, "CoreNmpRBase", 2048..=8192;
    nmp_r_improving = 949, "CoreNmpRImproving", 0..=2048;
    nmp_r_depth = 253, "CoreNmpRDepth", 64..=640;
    nmp_r_eval = 1_092, "CoreNmpREval", 0..=3072;
    nmp_r_clamp = 534, "CoreNmpRClamp", 128..=1536;
    /// From this depth a null fail-high outside a verification region is
    /// verified by a search with the null move disabled over the region.
    nmp_verify_depth = 16, "CoreNmpVerifyDepth", 4..=32;

    // ProbCut.
    /// `probcut_beta = beta + base - improving_term * improving`, evaluation
    /// units; the base is Rarog's fitted `ProbCutMargin`.
    probcut_base = 172, "CoreProbcutBase", 50..=400;
    probcut_improving = 43, "CoreProbcutImproving", 0..=150;
    /// Captures searched at most per node.
    probcut_move_cap = 5, "CoreProbcutMoveCap", 1..=16;
    /// The verification depth falls one ply below `depth - 4 - improving`
    /// for every `div` units the qsearch pass cleared `probcut_beta` by.
    probcut_depth_div = 131, "CoreProbcutDepthDiv", 48..=512;
    /// A shallower verification must clear `probcut_beta` plus this per ply
    /// it saved.
    probcut_adjust = 105, "CoreProbcutAdjust", 0..=300;
    /// A cut returns this many 1024ths of the way from its score to beta.
    probcut_lerp = 240, "CoreProbcutLerp", 0..=1024;

    // Singular extensions, multi-cut, low-depth singular extension.
    /// The singular margin is `(margin * span + margin * depth * (PV-line
    /// node searched with a null window)) / 16`: sixteenths of an
    /// evaluation unit per ply of span. 64 is Rarog's fitted `4 * depth`;
    /// both donors convert to about 8.
    singular_margin = 17, "CoreSingularMargin", 4..=192;
    /// The span of the margin when the stored bound is exact, in sixteenths
    /// of the depth, rounded up; a non-exact bound spans the whole depth.
    sing_exact_span = 4, "CoreSingExactSpan", 2..=16;
    /// A stored lower bound seeds a singular candidate only when it is at
    /// most this many plies shallower than the node; 2 here, not the
    /// accepted search's `SingularTtDepthMargin` of 3, because 2 measured
    /// +12.9 ± 9.4 Elo on this arm in 2,000 games (RAR-S80).
    singular_tt_depth_margin = 2, "CoreSingularTtDepthMargin", 0..=4;
    /// A singular move extends twice when its exclusion score falls this far
    /// below the singular beta: `pv_term * PV + not_tt_pv * (PV and not
    /// stored on a PV line) - quiet * quiet TT move - corr * |correction|/128
    /// + base`; the base, zero as in the donor, sets the bar off the PV.
    sing_double_base = -2, "CoreSingDoubleBase", -100..=200;
    sing_double_pv = 101, "CoreSingDoublePv", 0..=300;
    sing_double_not_tt_pv = 26, "CoreSingDoubleNotTtPv", 0..=100;
    sing_double_quiet = 8, "CoreSingDoubleQuiet", 0..=50;
    sing_double_corr = 10, "CoreSingDoubleCorr", 0..=50;
    /// Three times at this margin, the same terms plus a base.
    sing_triple_pv = 119, "CoreSingTriplePv", 0..=350;
    sing_triple_not_tt_pv = 23, "CoreSingTripleNotTtPv", 0..=100;
    sing_triple_quiet = 7, "CoreSingTripleQuiet", 0..=50;
    sing_triple_corr = 7, "CoreSingTripleCorr", 0..=50;
    sing_triple_base = 15, "CoreSingTripleBase", 0..=80;
    /// A multi-cut returns this many 1024ths of the way from its score to
    /// beta.
    sing_multicut_lerp = 383, "CoreSingMulticutLerp", 0..=1024;
    /// A cut node at depth 7 or less with no singular candidate extends its
    /// first move when the estimate is this far below alpha.
    ldse_margin = 8, "CoreLdseMargin", 0..=100;
    /// Late moves at a node whose TT move beat the exclusion search reduce
    /// more: `clamp(slope * (tt_move_score - singular_score - offset)/128, 0,
    /// cap)`, in 1024ths of a ply.
    lmr_singular_slope = 1_123, "CoreLmrSingularSlope", 0..=3072;
    lmr_singular_offset = 78, "CoreLmrSingularOffset", 0..=300;
    lmr_singular_cap = 2_051, "CoreLmrSingularCap", 0..=4096;
}

// The quiescence cluster's coordinates and switches. The interpolations are in 1024ths of the way from a fail-high score
// to beta: 0 keeps the fail-soft score, 1024 is fail-hard.
search_params! {
    struct QuietParams, generated_quiet_param_checks;

    /// A stand-pat fail-high is stored and returned this far toward beta.
    qs_stand_pat_lerp = 565, "QsStandPatLerp", 0..=1024;
    /// A capture's fail-high is stored and returned this far toward beta.
    qs_cutoff_lerp = 586, "QsCutoffLerp", 0..=1024;
    /// Captures after this many at a node are skipped unless they give check,
    /// recapture on the previous move's square or promote, while the stand pat
    /// is not a loss.
    qs_count_limit = 3, "QsCountLimit", 2..=8;
    /// A capture is skipped when the stand pat, its victim and this margin
    /// cannot reach alpha; evaluation units over the search's piece scale.
    qs_futility_margin = 208, "QsFutilityMargin", 0..=400;
    /// With more than eight pieces on the board, the stand pat, a queen and
    /// this margin below alpha end the node.
    qs_delta_margin = 212, "QsDeltaMargin", 0..=800;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_params_defaults_are_sane() {
        let p = SearchParams::default();
        assert!(p.aspiration_delta > 0);
        assert!(p.qs_see_margin >= 0);
        assert!(p.qs_see_bad_floor <= 0);
        assert_eq!(
            p.nmp_min_non_pawn_pieces, 1,
            "4.4c material guard must reproduce has_non_pawn_material"
        );
        assert!(p.lazy_margin > 0);
        assert!(p.tm_opt_scale > 0);
        assert!(p.tm_fall_base > 0);
        assert!(p.tm_fall_slope > 0);
        assert!(p.tm_instab_base > 0);
        assert!(p.tm_instab_slope > 0);
        assert!(p.tm_effort_high > 0);
        assert!(p.tm_effort_low > 0);
    }
}
