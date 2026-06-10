/// Tunable search parameters.
///
/// Every field maps 1-to-1 to a UCI `spin` option declared in
/// `search_options.rs`.
///
/// Defaults are the Phase 1 SPSA-tuned values (weather-factory + fastchess,
/// tc=1+0.01, SuperGM_4mvs).  The original hand-coded values are shown in
/// comments for reference.
///
/// For re-tuning, copy the three weather-factory config files from
/// `tools/spsa_configs/` into your weather-factory root and run `python
/// main.py`.  See `tools/spsa_configs/README.md` for full setup.
#[derive(Clone, Debug)]
pub struct SearchParams {
    /// Initial aspiration window half-width (centipawns). [search.rs:615]
    pub aspiration_delta: i32,

    /// Futility pruning base margin. Formula: `(base + improving * not_improving_i) * depth`.
    /// [search.rs:1003]
    pub futility_base: i32,
    /// Futility improving coefficient (added when *not* improving). [search.rs:1003]
    pub futility_improving: i32,

    /// Razoring coefficient. Prune if `eval + coeff * depth < alpha`. [search.rs:1007]
    pub razoring_coeff: i32,

    /// Null-move pruning depth coefficient. [search.rs:1012]
    /// Allow NMP when `eval >= beta - coeff * depth - improving_bonus * improving`.
    pub nm_depth_coeff: i32,
    /// Null-move pruning improving bonus. [search.rs:1012]
    pub nm_improving_bonus: i32,

    /// LMP prune-margin base. Formula: `(base + improving * not_improving_i) * depth`. [search.rs:1182]
    pub lmp_base: i32,
    /// LMP prune-margin improving coefficient (added when *not* improving). [search.rs:1182]
    pub lmp_improving: i32,

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

    /// ProbCut base margin. Formula: `base + depth_margin * depth - improving_bonus * improving`.
    pub probcut_base_margin: i32,
    /// ProbCut depth margin coefficient.
    pub probcut_depth_margin: i32,
    /// ProbCut margin reduction when improving.
    pub probcut_improving_bonus: i32,

    // ── LMR weighted adjustments (all in 1024ths of a ply) ──────────────────
    // Applied to the 1024x-scaled LMR_TABLE base; `>> 10` gives integer ply.
    // Defaults of 1024 reproduce the original ±1 ply behavior exactly, so
    // bench 13 is unchanged and SPSA tunes from a correct baseline.
    /// PV / TT-PV nodes: reduce less (stored positive; subtracted). Default = 1024 (1 ply).
    pub lmr_tt_pv_adj: i32,
    /// Exact TT bound: additional reduction. Default = 0 (not in original code; new term).
    pub lmr_exact_bound: i32,
    /// Shallow / absent TT entry: searched >= 4 and no tt_move. Default = 1024 (1 ply).
    pub lmr_shallow_tt: i32,
    /// Cut node: reduce more. Default = 1024 (1 ply).
    pub lmr_cut_node: i32,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            aspiration_delta: 31,          // was 25 → 29 → 31
            futility_base: 86,             // was 70 → 82 → 86
            futility_improving: 49,        // was 20 → 51 → 49
            razoring_coeff: 191,           // was 150 → 194 → 191
            nm_depth_coeff: 15,            // was 12 → 14 → 15
            nm_improving_bonus: 25,        // was 24 (unchanged)
            lmp_base: 115,                 // was 90 (unchanged)
            lmp_improving: 57,             // was 25 → 53 → 57
            quiet_hist_prune_coeff: 4_419, // was 4000 → 4372 → 4419
            see_pruning_coeff: 81,         // was 80 → 75 → 81
            see_pruning_max: 811,          // was 800 → 801 → 811
            singular_beta_mult: 4,         // was 2 (unchanged)
            lmp_count_base: 2,             // was 4 (unchanged)
            // Phase 2 ProbCut seed values ported from v2.1.0-codex; tune before keeping.
            probcut_base_margin: 188,    // codex seed
            probcut_depth_margin: 4,     // codex seed
            probcut_improving_bonus: 28, // codex seed
            // LMR adjustments — defaults reproduce original ±1-ply behavior exactly.
            // Group A SPSA candidate (914 / 136 / 1073 / 834) was rejected:
            // [0,3] SPRT stayed inconclusive after ~58k games (nElo ~+1.7).
            lmr_tt_pv_adj: 1024,  // 1 ply (original: -1)
            lmr_exact_bound: 0,   // 0 = not in original code
            lmr_shallow_tt: 1024, // 1 ply (original: +1 when !tt_move && searched>=4)
            lmr_cut_node: 1024,   // 1 ply (original: +1)
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
        assert!(p.futility_improving >= 0);
        assert!(p.razoring_coeff > 0);
        assert!(p.nm_depth_coeff > 0);
        assert!(p.nm_improving_bonus >= 0);
        assert!(p.lmp_base > 0);
        assert!(p.lmp_improving >= 0);
        assert!(p.quiet_hist_prune_coeff > 0);
        assert!(p.see_pruning_coeff > 0);
        assert!(p.see_pruning_max > 0);
        assert!(p.singular_beta_mult > 0);
        assert!(p.lmp_count_base > 0);
        assert!(p.probcut_base_margin > p.probcut_improving_bonus);
        assert!(p.probcut_depth_margin >= 0);
        assert!(p.lmr_tt_pv_adj >= 0);
        assert!(p.lmr_exact_bound >= 0);
        assert!(p.lmr_shallow_tt >= 0);
        assert!(p.lmr_cut_node >= 0);
    }
}
