/// Tunable search parameters.
///
/// Every field maps 1-to-1 to a UCI `spin` option declared in
/// `search_options.rs`.
///
/// Defaults are the current accepted integration-head values. The original
/// hand-coded values are shown in comments for reference where useful.
///
/// For re-tuning, copy the three weather-factory config files from
/// `tools/spsa_configs/` into your weather-factory root and run `python
/// main.py`.  See `tools/spsa_configs/README.md` for full setup.
#[derive(Clone, Debug)]
pub struct SearchParams {
    /// Initial aspiration window half-width (centipawns). [search.rs:615]
    pub aspiration_delta: i32,

    /// Futility pruning base margin.
    /// Formula: `(base + not_improving_coeff * not_improving_i) * depth`. [search.rs:1003]
    pub futility_base: i32,
    /// Extra futility margin added when *not* improving (multiplied by
    /// `not_improving_i`). Larger value → prune less when not improving.
    pub futility_not_improving: i32,

    /// Razoring coefficient. Prune if `eval + coeff * depth < alpha`. [search.rs:1007]
    pub razoring_coeff: i32,

    /// Null-move pruning depth coefficient. [search.rs:1012]
    /// Allow NMP when `eval >= beta - coeff * depth - improving_bonus * improving`.
    pub nm_depth_coeff: i32,
    /// Null-move pruning improving bonus. [search.rs:1012]
    pub nm_improving_bonus: i32,

    /// LMP prune-margin base.
    /// Formula: `(base + not_improving_coeff * not_improving_i) * depth`. [search.rs:1182]
    pub lmp_base: i32,
    /// Extra LMP prune-margin added when *not* improving (multiplied by
    /// `not_improving_i`). Larger value → prune less when not improving.
    pub lmp_not_improving: i32,

    /// Quiet-history pruning coefficient (stored positive; applied as `-(coeff * depth)`).
    /// [search.rs:1186]
    pub quiet_hist_prune_coeff: i32,

    /// SEE bad-capture threshold coefficient (stored positive; applied as `-(coeff * depth)`).
    /// [search.rs:1195]
    pub see_pruning_coeff: i32,
    /// SEE bad-capture threshold maximum magnitude (floor of `-(coeff * depth)`). [search.rs:1195]
    pub see_pruning_max: i32,

    /// Singular-extension beta multiplier. `singular_beta = tt_score - mult * depth`. [search.rs:1215]
    pub singular_beta_mult: i32,

    /// LMP count base. `count = base + 2 * depth * depth / 3`. [search.rs:2394]
    pub lmp_count_base: i32,

    // ── LMR weighted adjustments (all in 1024ths of a ply) ──────────────────
    // Applied to the 1024x-scaled LMR table base; `>> 10` gives integer ply.
    // The default-equivalent seed set was 1024 / 0 / 1024 / 1024; current
    // defaults are the Phase 2.5.1 clock-TC SPSA candidate pending SPRT.
    /// PV / TT-PV nodes: reduce less (stored positive; subtracted).
    pub lmr_tt_pv_adj: i32,
    /// Exact TT bound: additional reduction.
    pub lmr_exact_bound: i32,
    /// Shallow / absent TT entry: searched >= 4 and no tt_move.
    pub lmr_shallow_tt: i32,
    /// Cut node: reduce more.
    pub lmr_cut_node: i32,

    // ── LMR table formula coefficients (in 1024ths) ──────────────────────────
    // Table formula: 1024 * (base/1024 + ln(depth)*ln(move_idx) / (div/1024))
    // The default-equivalent seed formula was 0.75 + ln*ln/2.25; current
    // defaults are the Phase 2.5.1 clock-TC SPSA candidate pending SPRT.
    /// Additive base constant (1024ths).
    pub lmr_table_base: i32,
    /// Logarithm divisor (1024ths).
    pub lmr_table_div: i32,
    /// History divisor in the per-move history adjustment. Default = 8192.
    /// Applied as: `r -= quiet_hist * 1024 / lmr_hist_div`.
    pub lmr_hist_div: i32,

    // ── Per-move quiet futility pruning (Phase 2.7) ──────────────────────────
    // Skip a quiet move when `eval_for_pruning + fp_base + fp_coeff*depth <= alpha`
    // (depth <= 8, not in check, move doesn't give check). Centipawn-scaled —
    // re-tuned in the Phase 4 SPSA wave after the eval re-fit.
    /// Quiet futility base margin (cp).
    pub fp_base: i32,
    /// Quiet futility per-depth coefficient (cp).
    pub fp_coeff: i32,

    /// ProbCut beta margin (cp). `probcut_beta = beta + margin`. [search.rs:1108]
    /// Re-tuned in the Phase 5 SPSA wave after the Phase 4 eval re-fit changed
    /// what a centipawn means; the flat-margin form is the current accepted
    /// shape (an earlier improving-aware 3-parameter port was tried in Phase 2
    /// and dropped, H0 -24.5 Elo — see tools/spsa_configs/README.md).
    pub probcut_margin: i32,

    /// Futility-margin improving-direction selector (Phase 5.1, relocated 2.5.2).
    /// Controls which side of the `improving` flag the `futility_not_improving`
    /// coefficient is added to in the reverse-futility margin [search.rs:1041]:
    /// `0` (default) → added when *not* improving (margin shrinks when improving,
    /// i.e. prunes more — the current/SF-RFP direction); `1` → added when
    /// improving (larger margin when improving — the conventional forward-futility
    /// direction). The no-modulation variant is `futility_not_improving = 0`
    /// (reachable at either setting). A discrete A/B knob, not a continuous SPSA
    /// target — gate each direction `[-3,3]`. Default reproduces current behaviour
    /// exactly (bench-identical).
    pub futility_improving_dir: i32,

    /// Lazy-eval margin (Phase 5.1b; mirrors `eval::LAZY_MARGIN` = 600). If the
    /// tapered material + PST + pawn score already exceeds this, the expensive
    /// positional block is skipped [eval.rs lazy path]. Pushed into the evaluator
    /// at every search start. A *safety* knob first (Phase 4 grew the positional
    /// weights, so the seeded-0 margin may now be too tight — widen + confirm
    /// `[-3,3]` no-regression before tuning for NPS), then an SPSA speed knob.
    /// Disabled under `--features texel` (the tuner fits the full eval).
    pub lazy_margin: i32,

    // ── Time-management dynamic multipliers (Phase 5.1 TM group) ─────────────
    // The clock-mode between-iteration soft-stop scales `optimum_ms` by
    // falling-eval × best-move-instability × effort (search.rs soft-stop block);
    // these are the 2.2 SF-seeded constants, exposed for the TM SPSA group.
    // Stored in ten-thousandths so the float defaults reconstruct bit-exactly
    // (`x / 10000.0` is correctly-rounded, identical to the original literal).
    // TM affects only clock play, never the depth-limited `bench` fingerprint.
    /// Overall multiplier on `optimum_ms` (10000 = ×1.0). The single
    /// highest-leverage TM knob; lets SPSA scale base time allocation.
    pub tm_opt_scale: i32,
    /// Falling-eval base term. Seed 1187 (0.1187).
    pub tm_fall_base: i32,
    /// Falling-eval slope on `(prev_avg_score - score)`. Seed 221 (0.0221).
    pub tm_fall_slope: i32,
    /// Best-move-instability base. Seed 11000 (1.10).
    pub tm_instab_base: i32,
    /// Best-move-instability slope on `tot_best_move_changes`. Seed 22900 (2.29).
    pub tm_instab_slope: i32,
    /// Effort factor at low effort (interp endpoint at t=0). Seed 9240 (0.924).
    pub tm_effort_high: i32,
    /// Effort factor at high effort (interp endpoint at t=1). Seed 7100 (0.71).
    pub tm_effort_low: i32,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            // Phase 5.1 pruning SPSA candidate (tc=3+0.03, 2,461 iters / 78,752
            // games, post-Phase-4 eval scale). Pending the [0,3] confirming SPRT.
            aspiration_delta: 30,          // was 25 → 29 → 31 → 30
            futility_base: 61,             // was 70 → 82 → 86 → 61
            futility_not_improving: 42,    // was 20 → 51 → 49 → 42
            razoring_coeff: 193,           // was 150 → 194 → 191 → 193
            nm_depth_coeff: 10,            // was 12 → 14 → 15 → 10
            nm_improving_bonus: 33,        // was 24 → 25 → 33
            lmp_base: 88,                  // was 90 → 115 → 88
            lmp_not_improving: 63,         // was 25 → 53 → 57 → 63
            quiet_hist_prune_coeff: 5_072, // was 4000 → 4372 → 4419 → 5072
            see_pruning_coeff: 84,         // was 80 → 75 → 81 → 84
            see_pruning_max: 808,          // was 800 → 801 → 811 → 808
            singular_beta_mult: 6,         // was 2 → 4 → 6 (interior; ceiling widened 6→8)
            lmp_count_base: 2,             // was 4 → 2 (unchanged this wave)
            // LMR adjustments — Phase 2.5.1 clock-TC SPSA candidate
            // (weather-factory tc=3+0.03, 85,792 games / 2,681 iterations).
            lmr_tt_pv_adj: 887,   // was 1024; Phase 2.4 candidate was 1110
            lmr_exact_bound: 109, // was 0; Phase 2.4 candidate was 98
            lmr_shallow_tt: 656,  // was 1024; Phase 2.4 candidate was 880
            lmr_cut_node: 780,    // was 1024; Phase 2.4 candidate was 1138
            // LMR table formula — Phase 2.5.1 clock-TC SPSA candidate.
            lmr_table_base: 646, // was 768 (0.75 * 1024); Phase 2.4 was 738
            lmr_table_div: 2335, // was 2304 (2.25 * 1024); Phase 2.4 was 2334
            lmr_hist_div: 8395,  // was 8192; Phase 2.4 was 8268
            // Quiet futility pruning — Basilisk seeds, SPSA-tuned on entry.
            fp_base: 184,
            fp_coeff: 117,
            probcut_margin: 180,
            // Futility-direction A/B (relocated 2.5.2): default = current behaviour.
            futility_improving_dir: 0,
            // Lazy-eval margin (mirrors eval::LAZY_MARGIN).
            lazy_margin: 600,
            // Time-management dynamic multipliers (×10000), 2.2 SF seeds.
            tm_opt_scale: 10_000,    // ×1.0
            tm_fall_base: 1_187,     // 0.1187
            tm_fall_slope: 221,      // 0.0221
            tm_instab_base: 11_000,  // 1.10
            tm_instab_slope: 22_900, // 2.29
            tm_effort_high: 9_240,   // 0.924
            tm_effort_low: 7_100,    // 0.71
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_params_defaults_are_sane() {
        let p = SearchParams::default();
        assert!(p.aspiration_delta > 0);
        assert!(p.futility_base > 0);
        assert!(p.futility_not_improving >= 0);
        assert!(p.razoring_coeff > 0);
        assert!(p.nm_depth_coeff > 0);
        assert!(p.nm_improving_bonus >= 0);
        assert!(p.lmp_base > 0);
        assert!(p.lmp_not_improving >= 0);
        assert!(p.quiet_hist_prune_coeff > 0);
        assert!(p.see_pruning_coeff > 0);
        assert!(p.see_pruning_max > 0);
        assert!(p.singular_beta_mult > 0);
        assert!(p.lmp_count_base > 0);
        assert!(p.lmr_tt_pv_adj >= 0);
        assert!(p.lmr_exact_bound >= 0);
        assert!(p.lmr_shallow_tt >= 0);
        assert!(p.lmr_cut_node >= 0);
        assert!(p.lmr_table_base > 0);
        assert!(p.lmr_table_div > 0);
        assert!(p.lmr_hist_div > 0);
        assert!(p.fp_base > 0);
        assert!(p.fp_coeff > 0);
        assert!(p.probcut_margin > 0);
        assert!(p.futility_improving_dir == 0 || p.futility_improving_dir == 1);
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
