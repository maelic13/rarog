/// Tunable search parameters.
///
/// Every field maps 1-to-1 to a UCI `spin` option declared in
/// `search_options.rs`.
///
/// Defaults are the Phase 1 SPSA-tuned values (weather-factory + fastchess,
/// tc=1+0.01, SuperGM_4mvs, 2271 iterations / 72672 games).  The original
/// hand-coded values are shown in comments for reference.
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

    // ── LMR weighted adjustments (all in 1024ths of a ply) ──────────────────
    // Applied to the 1024x-scaled LMR_TABLE base, then `>> 10` gives integer
    // ply reduction.  Negative values reduce less; positive values reduce more.

    /// PV / TT-PV nodes: reduce less (stored positive; subtracted).
    pub lmr_tt_pv_adj: i32,
    /// Exact TT bound: solid score on file, later moves less likely to flip it.
    pub lmr_exact_bound: i32,
    /// Shallow or absent TT entry: less guidance available.
    pub lmr_shallow_tt: i32,
    /// Cut node: expected to fail high; later moves are typically bad.
    pub lmr_cut_node: i32,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            aspiration_delta:       29,  // was 25
            futility_base:          82,  // was 70
            futility_improving:     51,  // was 20
            razoring_coeff:        194,  // was 150
            nm_depth_coeff:         14,  // was 12
            nm_improving_bonus:     25,  // was 24
            lmp_base:              115,  // was 90
            lmp_improving:          53,  // was 25
            quiet_hist_prune_coeff: 4_372, // was 4000
            see_pruning_coeff:      75,  // was 80
            see_pruning_max:       801,  // was 800
            singular_beta_mult:      4,  // was 2
            lmp_count_base:          2,  // was 4
            // LMR weighted adjustments (1024ths) — defaults from v2.1.0-claude:src/tune.rs.
            lmr_tt_pv_adj:         463,
            lmr_exact_bound:      1405,
            lmr_shallow_tt:        286,
            lmr_cut_node:         1810,
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
        assert!(p.lmr_tt_pv_adj > 0);
        assert!(p.lmr_exact_bound > 0);
        assert!(p.lmr_shallow_tt > 0);
        assert!(p.lmr_cut_node > p.lmr_tt_pv_adj, "cut node must reduce more than pv relief");
    }
}
