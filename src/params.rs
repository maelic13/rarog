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
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            aspiration_delta: 31,          // was 25 → 29 → 31
            futility_base: 86,             // was 70 → 82 → 86
            futility_not_improving: 49,    // was 20 → 51 → 49
            razoring_coeff: 191,           // was 150 → 194 → 191
            nm_depth_coeff: 15,            // was 12 → 14 → 15
            nm_improving_bonus: 25,        // was 24 (unchanged)
            lmp_base: 115,                 // was 90 (unchanged)
            lmp_not_improving: 57,         // was 25 → 53 → 57
            quiet_hist_prune_coeff: 4_419, // was 4000 → 4372 → 4419
            see_pruning_coeff: 81,         // was 80 → 75 → 81
            see_pruning_max: 811,          // was 800 → 801 → 811
            singular_beta_mult: 4,         // was 2 (unchanged)
            lmp_count_base: 2,             // was 4 (unchanged)
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
    }
}
