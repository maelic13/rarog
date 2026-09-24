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
    aspiration_delta = 21, "AspirationDelta", 5..=100;  // was 25 → 29 → 31 → 30 → 21

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

    /// Futility pruning base margin.
    /// Formula: `(base + not_improving_coeff * not_improving_i) * depth`.
    futility_base = 52, "FutilityBase", 20..=200;  // was 70 → 82 → 86 → 60 → 52
    /// Extra futility margin added when *not* improving (multiplied by
    /// `not_improving_i`). Larger value → prune less when not improving.
    futility_not_improving = 51, "FutilityNotImproving", 0..=120;  // was 20 → 51 → 49 → 42 → 51

    /// Razoring coefficient. Prune if `eval + coeff * depth < alpha`.
    razoring_coeff = 274, "RazoringCoeff", 50..=300;  // was 150 → 194 → 191 → 193 → 274

    /// Null-move pruning depth coefficient.
    /// Allow NMP when `eval >= beta - coeff * depth - improving_bonus * improving`.
    nm_depth_coeff = 12, "NullMoveDepthCoeff", 2..=40;  // was 12 → 14 → 15 → 10 → 12
    /// Null-move pruning improving bonus.
    nm_improving_bonus = 35, "NullMoveImprovingBonus", 0..=80;  // was 24 → 25 → 32 → 35

    /// LMP prune-margin base.
    /// Formula: `(base + not_improving_coeff * not_improving_i) * depth`.
    lmp_base = 80, "LmpBase", 30..=200;  // was 90 → 115 → 88 → 80
    /// Extra LMP prune-margin added when *not* improving (multiplied by
    /// `not_improving_i`). Larger value → prune less when not improving.
    lmp_not_improving = 64, "LmpNotImproving", 0..=120;  // was 25 → 53 → 57 → 63 → 64

    /// Quiet-history pruning coefficient (stored positive; applied as `-(coeff * depth)`).
    ///
    quiet_hist_prune_coeff = 5_617, "QuietHistPruneCoeff", 1000..=10000;  // was 4000 → 4372 → 4419 → 5069 → 5617

    /// SEE bad-capture threshold coefficient (stored positive; applied as `-(coeff * depth)`).
    ///
    see_pruning_coeff = 66, "SeePruningCoeff", 20..=200;  // was 83 → 51 → 66
    /// SEE bad-capture threshold maximum magnitude (floor of `-(coeff * depth)`).
    see_pruning_max = 955, "SeePruningMax", 200..=1600;

    // ── Qsearch SEE thresholds (Phase 7.2 SEE bundle) ────────────────────────
    // Exposed so the `config_see` SPSA can re-tune SEE's consumers alongside
    // the pin-aware `see_ge` (lesson 15: a more accurate SEE de-tunes the
    // constants fitted around the old one). Defaults reproduce the prior
    // hardcoded literals exactly → bench-identical until re-tuned.
    /// Qsearch capture SEE-prune margin: search a capture only if
    /// `see_ge(alpha − stand_pat − qs_see_margin)` (clamped). Seed 200.
    qs_see_margin = 265, "QsSeeMargin", 0..=600;  // was 200 → 251 → 265
    /// Lower clamp on the qsearch SEE-prune threshold. Seed −800.
    qs_see_clamp_lo = -722, "QsSeeClampLo", -1600..=-100;  // was -800 → -661 → -722
    /// Upper clamp on the qsearch SEE-prune threshold. Seed 200.
    qs_see_clamp_hi = 212, "QsSeeClampHi", 0..=600;  // was 200 → 218 → 212
    /// Qsearch bad-capture SEE floor: an ordering-SEE-negative capture is
    /// skipped unless `see_ge(qs_see_bad_floor)`. Seed −50.
    qs_see_bad_floor = -55, "QsSeeBadFloor", -400..=0;  // was -50 → -119 → -55

    /// Singular-extension beta multiplier. `singular_beta = tt_score - mult * depth`.
    singular_beta_mult = 4, "SingularBetaMult", 1..=8;  // was 2 → 4 → 6 → 4

    /// 4.3 arm B — how far below the node depth a TT entry may sit and still
    /// seed a singular verification window (`ev.depth >= depth - margin`).
    ///
    /// 3 = current behaviour, and 3 is exactly the depth a same-node ProbCut
    /// writes (`depth - 3`), so that signature is admitted at the boundary:
    /// RAR-S22 measured 32 of 101 sampled attempts there. Margin 2 excludes the
    /// whole `depth - 3` band, including legitimate full-search entries, but it
    /// is not a provenance guarantee: a ProbCut entry produced by an earlier
    /// deeper search can still qualify at a shallower consumer. RAR-S31 found
    /// value 2 positive on a tune binary, but the ~3 Elo knob was parked under
    /// the material-gain policy. B.1 removed 4.3c's persisted provenance.
    singular_tt_depth_margin = 3, "SingularTtDepthMargin", 0..=4;

    /// Ordering bonus for a quiet move that gives check.
    ///
    /// 32000 = the historical flat `DIRECT_CHECK_BONUS`, promoted from a bare
    /// constant to a coordinate so 4.10 can fit it. The 4.3 audit wanted this
    /// split into safe/losing classes; that split was implemented, measured
    /// NON-FUNCTIONAL (RAR-S44: zero losing-check population because
    /// `see_ge` is trivially true for a non-capture) and reverted. A correct
    /// classifier needs a different predicate and is 4.10 ordering work.
    check_bonus_safe = 32000, "CheckBonusSafe", 0..=32000;

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

    /// Margin below `singular_beta` required for a DOUBLE extension.
    ///
    /// 20 = accepted baseline, previously a bare literal. PLAN 4.4 asks for
    /// "separate single/double rules"; making the double rule's own margin a
    /// coordinate is what separates them, and the 4.1 census measured double
    /// extensions at 21 of 101 sampled attempts against 25 single — a high share
    /// for the more aggressive branch. Larger values make doubles rarer without
    /// removing them outright.
    singular_double_margin = 20, "SingularDoubleMargin", 0..=200;

    /// LMP count base. `count = base + 2 * depth * depth / 3`.
    lmp_count_base = 1, "LmpCountBase", 1..=12;  // was 4 → 2 → 1 (10.4.6 lower rail; active)

    // ── LMR weighted adjustments (all in 1024ths of a ply) ──────────────────
    // Applied to the 1024x-scaled LMR table base; `>> 10` gives integer ply.
    // The default-equivalent seed set was 1024 / 0 / 1024 / 1024; current
    // defaults are the Phase 2.5.1 clock-TC SPSA candidate pending SPRT.
    /// PV / TT-PV nodes: reduce less (stored positive; subtracted).
    lmr_tt_pv_adj = 887, "LmrTtPvAdj", 0..=2048;  // was 1024; Phase 2.4 candidate was 1110
    /// Exact TT bound: additional reduction.
    lmr_exact_bound = 109, "LmrExactBound", 0..=2048;  // was 0; Phase 2.4 candidate was 98
    /// Late-move reduction bump applied when a **TT move is present** and the
    /// move is late in the list (`!tt_move.is_null() && searched >= 4`). NB the
    /// name is a misnomer — the live condition (`search/node.rs`, LMR block) fires
    /// on TT-move *presence*, not absence, and never checks TT depth. The value
    /// (656) was SPSA'd under this live condition; the "TT-absent / depth-aware"
    /// polarity the name implies is a deliberate 10.4-menu A/B, not a bug.
    lmr_shallow_tt = 656, "LmrShallowTt", 0..=2048;  // was 1024; Phase 2.4 candidate was 880
    /// Cut node: reduce more.
    lmr_cut_node = 780, "LmrCutNode", 0..=2048;  // was 1024; Phase 2.4 candidate was 1138

    // ── LMR table formula coefficients (in 1024ths) ──────────────────────────
    // Table formula: 1024 * (base/1024 + ln(depth)*ln(move_idx) / (div/1024))
    // The default-equivalent seed formula was 0.75 + ln*ln/2.25; current
    // defaults are the Phase 2.5.1 clock-TC SPSA candidate pending SPRT.
    /// Additive base constant (1024ths).
    lmr_table_base = 646, "LmrTableBase", 384..=1536;  // was 768 (0.75 * 1024); Phase 2.4 was 738
    /// Logarithm divisor (1024ths).
    lmr_table_div = 2335, "LmrTableDiv", 1536..=3072;  // was 2304 (2.25 * 1024); Phase 2.4 was 2334
    /// History divisor in the per-move history adjustment. Default = 8192.
    /// Applied as: `r -= quiet_hist * 1024 / lmr_hist_div`.
    lmr_hist_div = 8395, "LmrHistDiv", 4096..=16384;  // was 8192; Phase 2.4 was 8268

    // ── Per-move quiet futility pruning (Phase 2.7) ──────────────────────────
    // Skip a quiet move when `eval_for_pruning + fp_base + fp_coeff*depth <= alpha`
    // (depth <= 8, not in check, move doesn't give check). Centipawn-scaled —
    // re-tuned in the Phase 4 SPSA wave after the eval re-fit.
    /// Quiet futility base margin (cp).
    fp_base = 211, "FpBase", 0..=400;
    /// Quiet futility per-depth coefficient (cp).
    fp_coeff = 135, "FpCoeff", 0..=300;

    /// ProbCut beta margin (cp). `probcut_beta = beta + margin`.
    /// Re-tuned in the Phase 5 SPSA wave after the Phase 4 eval re-fit changed
    /// what a centipawn means; the flat-margin form is the current accepted
    /// shape (an earlier improving-aware 3-parameter port was tried in Phase 2
    /// and dropped, H0 -24.5 Elo — see tools/spsa_configs/README.md).
    probcut_margin = 180, "ProbCutMargin", 60..=400;

    /// Paired-ablation mask. Bit per mechanism, matching the oracle's:
    /// 0 razoring, 1 futility-child, 2 nullmove, 3 probcut, 4 iir,
    /// 5 shallow-pruning, 6 extensions, 7 lmr. 0 = shipped behaviour.
    /// Under `b2core` the bits name the selectivity core's mechanisms: 0
    /// razoring, 1 reverse futility, 2 null move, 3 ProbCut, 4 IIR and
    /// hindsight reductions, 5 move-loop pruning (late-move, quiet and
    /// bad-noisy futility, history and SEE pruning), 6 singular extensions,
    /// 7 late-move reductions.
    /// Only consulted when built with `--features ablate`.
    ablation_mask = 0, "AblationMask", 0..=255;
    /// 4.6.7 ROOT REDUCTION RELIEF, in 1024ths of a ply. 0 = off = accepted
    /// behaviour.
    ///
    /// `lmr_reduction_units` is not passed the ply and `reducible` has no
    /// `ply == 0` term, so the root is reduced exactly like an interior node
    /// from the third move onward. An alternative root move is therefore
    /// searched at REDUCED depth and can only displace the incumbent by
    /// beating alpha while reduced. The answer harness measures the
    /// consequence: Rarog revises its root move 1.50 times against the
    /// oracle's 2.16 and lands on a different move a third of the time.
    lmr_root_relief = 1536, "LmrRootRelief", 0..=2048;
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
    /// Base cap on captures searched at one ProbCut node, before the cut-node
    /// bonus. Replaces a flat 8 that had no stated derivation.
    probcut_move_cap_base = 2, "ProbCutMoveCapBase", 1..=8;
    /// Extra captures allowed at an expected cut node, where a fail-high is the
    /// predicted outcome and the speculative search is likeliest to pay.
    probcut_move_cap_cut_bonus = 2, "ProbCutMoveCapCutBonus", 0..=8;

    /// Lazy-eval margin (Phase 5.1b; mirrors `eval::LAZY_MARGIN` = 600). If the
    /// tapered material + PST + pawn score already exceeds this, the expensive
    /// positional block is skipped [eval.rs lazy path]. Pushed into the evaluator
    /// at every search start. A *safety* knob first (Phase 4 grew the positional
    /// weights, so the seeded-0 margin may now be too tight — widen + confirm
    /// `[-3,3]` no-regression before tuning for NPS), then an SPSA speed knob.
    /// Disabled under `--features texel` (the tuner fits the full eval).
    lazy_margin = 600, "LazyMargin", 200..=2000;

    // ── History bonus/malus split (Phase 8.1) ────────────────────────────────
    // Replaces the symmetric `history_bonus(depth) = (d² + 2d).min(1200)` used
    // for both reward and penalty. Cutoff move gets
    // `min(bonus_mul·d − bonus_sub, bonus_max)`; searched-but-failed moves get
    // `−min(malus_mul·d − malus_sub, malus_max)`. SF-shaped linear formulas —
    // they reach the HISTORY_MAX gravity equilibrium much faster than the old
    // quadratic (d=10: 1610 vs 120), and the split lets SPSA make penalties
    // stronger/weaker than rewards independently. Ported from the parked
    // `phase-8.1-history-split` branch onto the p75-tm head; seeds re-tuned by
    // the 8.1 SPSA on this head before the `[0,3]` bake gate.
    /// Bonus per-depth slope. Seed 170.
    hist_bonus_mul = 174, "HistBonusMul", 40..=400;  // 8.4 histcov fit (was 156)
    /// Bonus subtractor. Seed 90.
    hist_bonus_sub = 264, "HistBonusSub", 0..=500;  // 8.4 histcov fit (was 125)
    /// Bonus cap. Seed 1700.
    hist_bonus_max = 2_491, "HistBonusMax", 400..=4000;  // 8.4 histcov fit (was 2162) → rewards saturate high
    /// Malus per-depth slope. Seed 180.
    hist_malus_mul = 210, "HistMalusMul", 40..=400;  // 8.4 histcov fit (was 218)
    /// Malus subtractor. Seed 100.
    hist_malus_sub = 0, "HistMalusSub", 0..=500;  // 8.4 histcov fit (was 27; drifted to the 0 bound)
    /// Malus cap. Seed 1500.
    hist_malus_max = 1_877, "HistMalusMax", 400..=4000;  // 8.4 histcov fit (was 937) → penalties saturate low

    // ── Phase 8.4: history update-coverage bundle ────────────────────────────
    // Storage is rich, learning events are sparse. Every new coverage source
    // sits behind its OWN percentage knob seeded NEUTRAL, so the bundle is
    // bench-identical until the config_histcov SPSA moves a knob — and the
    // SPSA can independently drive any component back to zero. That is the
    // designed answer to "Basilisk's lessons may not translate": Rarog's own
    // tuning data decides per component.
    /// 8.4(b) — reward the QUIET best move of an Exact (PV) node, as a % of
    /// `history_bonus`. Seed 0 = today. REWARD-ONLY by design: Basilisk's
    /// reward-only form passed +4.90 while the sibling-malus form lost
    /// −84.21, so there is deliberately no malus, no killer/countermove
    /// write, and no capture reward at exact nodes.
    exact_bonus_pct = 31, "ExactBonusPct", 0..=150;  // 8.4 histcov fit (seed 0)
    /// 8.4(c) — cross-category malus on a CAPTURE cutoff, as a % of
    /// `history_malus`, applied to the searched quiets and bad captures that
    /// failed to cut (today only earlier good captures are penalized). Seed
    /// 0 = today. Good-SEE captures keep their existing malus only —
    /// Basilisk's all-searched-capture malus was bench-vetoed (+30%).
    capture_malus_pct = 25, "CaptureMalusPct", 0..=150;  // 8.4 histcov fit (seed 0)
    /// 8.4(e) — surprise scale on the cutoff REWARD (both quiet and capture
    /// cutoffs), applied when the node's static eval was below beta: the
    /// search found a good move the eval did not credit. Seed 100 = neutral.
    /// Basilisk's accepted value is 125 (+2.50) — a bounded nudge on a
    /// subset of cutoffs, not a table-wide shift. Maluses stay unscaled.
    surprise_bonus_pct = 119, "SurpriseBonusPct", 50..=250;  // 8.4 histcov fit (seed 100; Basilisk's accepted 125 — independent agreement)
    // ── Phase 8.5: correction-history semantics + magnitude margins + blend ──
    // The continuous margins/weights were included in the accepted 10.4.6(a)
    // joint selectivity fit. Preserve even off-valued mechanisms through the
    // NNUE transition: the post-NNUE retune may reactivate them.
    //
    /// 4.5 — weight (percent) applied to a correction update whose residual came
    /// from a CAPTURE-caused cutoff, instead of dropping it.
    ///
    /// 100 = accepted baseline and exactly inert (`diff * 100 / 100 == diff`).
    ///
    /// This graded coordinate is retained because RAR-S16 measured binary
    /// capture exclusion at **−55.98 Elo**: the guard
    /// discarded 59.7% of training, so the run measured a crippled signal rather
    /// than the mechanism's value. The 4.1 census puts capture-attributed updates
    /// at **145,372 of 283,590 (51.3%)**, which is far too much to throw away and
    /// is precisely why exclusion failed.
    ///
    /// Scaling keeps the coverage while down-weighting evidence the positional
    /// eval arguably should not learn to predict. 0 degenerates to the exclusion
    /// that already lost, so useful values are strictly between — and whether
    /// ANY down-weighting is justified is an open question the new
    /// `correction_resid_*` diagnostics answer: if capture-caused residuals are
    /// no noisier than quiet ones, the whole premise is wrong and this knob
    /// should stay at 100. Final weight enters the 4.10 fit; no dedicated SPSA.
    corr_capture_weight_pct = 100, "CorrCaptureWeightPct", 0..=100;

    // 8.5(b) — magnitude margins: scale forward pruning / LMR by |correction|.
    // A large correction means the raw static eval is being heavily adjusted
    // and is less trustworthy, so prune/reduce LESS (conservative-when-
    // uncertain, the Reckless form). `|corr| = |static_eval − raw_static_eval|`,
    // already computed per node. Each knob adds `|corr| · knob / 128` to a
    // margin (or subtracts it from the LMR reduction in 1024ths). Seed 0 = off.
    //
    // ⚠ These three are NOT inert. A stale comment in the search claimed the
    // seeds left them at 0; the fitted values below are live in the accepted
    // baseline, so `corr_abs` actively widens margins and shrinks reductions.
    corr_rfp_scale = 3, "CorrRfpScale", 0..=512;
    corr_fut_scale = 3, "CorrFutScale", 0..=512;
    corr_lmr_scale = 27, "CorrLmrScale", 0..=512;
    /// 8.5(c) — blend weights for the five correction sources, previously the
    /// fixed `(pawn+minor+own_np+their_np+cont/2)/128`. Now `Σ src·W / 16384`
    /// with the continuation term keeping its inherent `/2`. Seed 128 on every
    /// source reproduces the old blend bit-for-bit (`Σsrc·128/16384 = Σsrc/128`).
    /// SPSA re-weights the sources.
    corr_w_pawn = 135, "CorrWeightPawn", 0..=384;
    corr_w_minor = 80, "CorrWeightMinor", 0..=384;
    corr_w_own_np = 104, "CorrWeightOwnNp", 0..=384;
    corr_w_their_np = 160, "CorrWeightTheirNp", 0..=384;
    corr_w_cont = 152, "CorrWeightCont", 0..=384;

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
#[cfg(feature = "b2core")]
search_params! {
    struct CoreParams, generated_core_param_checks;

    // Correction update and blend.
    /// Update slope in 128ths: `bonus = slope * depth * residual / 128`.
    corr_update_slope = 203, "CoreCorrUpdateSlope", 32..=512;
    /// Clamps of one update, in table units (64 per evaluation unit).
    corr_update_min = -2_047, "CoreCorrUpdateMin", -8192..=-256;
    corr_update_max = 1_184, "CoreCorrUpdateMax", 128..=8192;
    /// Blend weights of the six tables, in 128ths.
    corr_weight_pawn = 136, "CoreCorrWeightPawn", 0..=384;
    corr_weight_minor = 63, "CoreCorrWeightMinor", 0..=384;
    corr_weight_non_pawn_white = 131, "CoreCorrWeightNonPawnWhite", 0..=384;
    corr_weight_non_pawn_black = 110, "CoreCorrWeightNonPawnBlack", 0..=384;
    corr_weight_cont2 = 138, "CoreCorrWeightCont2", 0..=384;
    corr_weight_cont4 = 138, "CoreCorrWeightCont4", 0..=384;
    /// Categorical, never an SPSA coordinate. 1 trains the correction on a
    /// decisive (mate-range or tablebase-range) result as the donors do; 0
    /// refuses a result at or beyond the tablebase-win band at both training
    /// sites, as the accepted search does.
    corr_train_decisive = 1, "CoreCorrTrainDecisive", 0..=1;
    /// Categorical, never an SPSA coordinate. 1 trains the correction at a
    /// singular-exclusion node as the donors do; 0 refuses training whenever
    /// a move is excluded, at both training sites, as the accepted search
    /// does.
    corr_train_excluded = 1, "CoreCorrTrainExcluded", 0..=1;
    // Corrected-eval formula, neutral at zero.
    /// Material scaling of the raw eval, in 64ths per starting-material unit.
    eval_material_scale = 27, "CoreEvalMaterialScale", -64..=64;
    /// Rule-50 damping, in percent of `eval * min(clock, 100) / 199`. The
    /// evaluator's own damping is compiled out under the core, so the eval the
    /// table stores does not depend on the clock and the search damps it here.
    eval_rule50_damping = 100, "CoreEvalRule50Damping", 0..=150;

    // Node-level pruning.
    /// Razoring margin `base + square * depth^2`, evaluation units.
    razor_base = 288, "CoreRazorBase", 50..=800;
    razor_square = 143, "CoreRazorSquare", 20..=400;
    /// Categorical, never an SPSA coordinate. 1 keeps razoring off a node on
    /// a PV line (`tt_pv`) and above depth 3, as the accepted search does; 0
    /// razors as the donor does.
    razor_guards = 0, "CoreRazorGuards", 0..=1;
    /// Reverse-futility margin: `square/16 * depth^2 + linear * depth
    /// - improvement * improvement/1024 + correction * |corr|/1024 - threat *
    /// unthreatened + constant`, floored at 2.
    rfp_square = 135, "CoreRfpSquare", 0..=320;
    rfp_linear = 17, "CoreRfpLinear", 0..=200;
    rfp_improvement = 75, "CoreRfpImprovement", 0..=512;
    rfp_correction = 212, "CoreRfpCorrection", 0..=2048;
    rfp_threat = 28, "CoreRfpThreat", 0..=120;
    rfp_constant = -26, "CoreRfpConstant", -100..=100;
    /// Reverse-futility return, in 1024ths of the way from the estimate to beta.
    rfp_lerp = 648, "CoreRfpLerp", 0..=1024;
    /// Hindsight: a parent reduction (1024ths of a ply) at or above this
    /// deepens a child whose eval says the parent's opponent got worse.
    hindsight_deepen_reduction = 1_924, "CoreHindsightDeepenReduction", 512..=6144;
    /// Hindsight: an eval swing above this reduces a reduced child one ply.
    hindsight_reduce_margin = 26, "CoreHindsightReduceMargin", 0..=200;
    /// Internal iterative reduction: the least depth at which a missing or
    /// shallow TT move costs a ply.
    iir_min_depth = 4, "CoreIirMinDepth", 2..=10;

    // Move-loop pruning.
    /// Late-move pruning count, in 1024ths: `(base + improvement *
    /// improvement/16 + square * depth^2 + history * history/1024) / 1024`.
    lmp_base = 2_407, "CoreLmpBase", 0..=8192;
    lmp_improvement = 62, "CoreLmpImprovement", 0..=400;
    lmp_square = 615, "CoreLmpSquare", 256..=4096;
    lmp_history = 59, "CoreLmpHistory", 0..=400;
    /// Quiet futility value: `eval + base + linear * depth + history *
    /// history/1024 + above_beta * (eval >= beta) + correction * |corr|/1024`.
    fp_base = 129, "CoreFpBase", -100..=500;
    fp_linear = 54, "CoreFpLinear", 10..=300;
    fp_history = 51, "CoreFpHistory", 0..=200;
    fp_eval_above_beta = 45, "CoreFpEvalAboveBeta", 0..=200;
    fp_correction = 138, "CoreFpCorrection", 0..=2048;
    /// Bad-noisy futility value: `eval + base + linear * depth + history *
    /// history/1024 + victim value`.
    bnfp_base = 51, "CoreBnfpBase", -100..=400;
    bnfp_linear = 54, "CoreBnfpLinear", 10..=300;
    bnfp_history = 40, "CoreBnfpHistory", 0..=200;
    /// History pruning below `-slope * depth`.
    hp_slope = 963, "CoreHpSlope", 100..=4000;
    /// SEE-pruning allowance for quiets: `min(0, -square * depth^2 + linear *
    /// depth - history * history/1024 + constant)`, SEE units.
    see_quiet_square = 8, "CoreSeeQuietSquare", 0..=40;
    see_quiet_linear = 34, "CoreSeeQuietLinear", 0..=200;
    see_quiet_history = 19, "CoreSeeQuietHistory", 0..=120;
    see_quiet_constant = 10, "CoreSeeQuietConstant", -100..=100;
    /// SEE-pruning allowance for noisy moves: `min(0, -square * depth^2 -
    /// linear * depth - history * history/1024 + constant)`.
    see_noisy_square = 6, "CoreSeeNoisySquare", 0..=40;
    see_noisy_linear = 6, "CoreSeeNoisyLinear", 0..=200;
    see_noisy_history = 36, "CoreSeeNoisyHistory", 0..=120;
    see_noisy_constant = 22, "CoreSeeNoisyConstant", -100..=100;

    // Late-move reductions, in 1024ths of a ply.
    lmr_log = 188, "CoreLmrLog", 0..=1024;
    lmr_improvement = 395, "CoreLmrImprovement", 0..=2048;
    /// Bounds of the improvement term, in reduction units. Seeded at the
    /// donor's values; converting them to Rarog's evaluation scale measured
    /// worse (RAR-S74), so the fit decides them.
    lmr_improvement_clamp_lo = -189, "CoreLmrImprovementClampLo", -1024..=0;
    lmr_improvement_clamp_hi = 974, "CoreLmrImprovementClampHi", 0..=4096;
    lmr_correction = 3_774, "CoreLmrCorrection", 0..=8192;
    lmr_exact = 1_433, "CoreLmrExact", 0..=4096;
    lmr_tt_score_below_alpha = 409, "CoreLmrTtScoreBelowAlpha", 0..=2048;
    lmr_tt_shallow = 261, "CoreLmrTtShallow", 0..=2048;
    lmr_quiet = 1_808, "CoreLmrQuiet", 0..=6144;
    lmr_quiet_history = 302, "CoreLmrQuietHistory", 0..=1024;
    lmr_alpha_gap = 352, "CoreLmrAlphaGap", 0..=2048;
    /// Bounds of `alpha - estimated score` in the quiet term, evaluation
    /// units, seeded at the donor's values like the improvement bounds.
    lmr_alpha_gap_lo = -79, "CoreLmrAlphaGapLo", -512..=0;
    lmr_alpha_gap_hi = 102, "CoreLmrAlphaGapHi", 0..=512;
    lmr_noisy = 1_063, "CoreLmrNoisy", 0..=6144;
    lmr_noisy_history = 117, "CoreLmrNoisyHistory", 0..=1024;
    lmr_critical_ply = 145, "CoreLmrCriticalPly", 0..=512;
    lmr_pv = 525, "CoreLmrPv", 0..=2048;
    lmr_pv_window = 340, "CoreLmrPvWindow", 0..=2048;
    lmr_non_pv = 272, "CoreLmrNonPv", -1024..=1024;
    lmr_laterality = 27, "CoreLmrLaterality", 0..=256;
    lmr_tt_pv = 322, "CoreLmrTtPv", 0..=2048;
    lmr_tt_pv_score = 624, "CoreLmrTtPvScore", 0..=2048;
    lmr_tt_pv_depth = 675, "CoreLmrTtPvDepth", 0..=2048;
    lmr_cut_node = 2_182, "CoreLmrCutNode", 0..=4096;
    lmr_cut_node_no_tt_move = 2_263, "CoreLmrCutNodeNoTtMove", 0..=4096;
    lmr_gives_check = 968, "CoreLmrGivesCheck", 0..=4096;
    lmr_child_cutoffs = 1_053, "CoreLmrChildCutoffs", 0..=4096;
    lmr_child_cutoffs_all_node = 340, "CoreLmrChildCutoffsAllNode", 0..=2048;
    lmr_parent = 128, "CoreLmrParent", 0..=1024;
    /// Re-search a reduced move one ply deeper when it beats the best score
    /// by more than this, one ply shallower when by less than this.
    lmr_research_deeper = 27, "CoreLmrResearchDeeper", 0..=200;
    lmr_research_shallower = -3, "CoreLmrResearchShallower", -50..=50;
    /// Categorical, never an SPSA coordinate. 1 searches the first move of a
    /// non-PV, non-root node out of check through the donor's full-depth
    /// branch, which may take one or two plies off it; 0 searches it at full
    /// depth.
    lmr_full_depth = 0, "CoreLmrFullDepth", 0..=1;
    /// Categorical, never an SPSA coordinate. 1 gives late-move reductions
    /// the donor's scope, the root and nodes in check included, with the
    /// first move still unreduced and the one-ply floor kept; 0 never
    /// reduces at the root or in check.
    lmr_check_root = 0, "CoreLmrCheckRoot", 0..=1;

    // History update policy, history units.
    /// Quiet best-move bonus `min(slope * depth, cap) - 72 - 42 * cut_node`.
    hist_quiet_bonus_slope = 184, "CoreHistQuietBonusSlope", 32..=512;
    hist_quiet_bonus_cap = 1_686, "CoreHistQuietBonusCap", 256..=4096;
    /// Quiet malus `min(slope * depth, cap) - 46 - 31 * quiets searched`.
    hist_quiet_malus_slope = 175, "CoreHistQuietMalusSlope", 32..=512;
    hist_quiet_malus_cap = 1_141, "CoreHistQuietMalusCap", 256..=4096;
    /// Search-order fade of the quiet malus: the i-th quiet gets
    /// `1024^2 / (1024 + scale * i)^2` of it.
    hist_malus_index_scale = 36, "CoreHistMalusIndexScale", 0..=256;
    hist_cont_bonus_cap = 1_085, "CoreHistContBonusCap", 256..=4096;
    hist_noisy_bonus_cap = 875, "CoreHistNoisyBonusCap", 256..=4096;
    /// Quiet bonus to a TT move that cuts: `min(190 * depth - 81, cap)`.
    hist_tt_cutoff_bonus_cap = 1_720, "CoreHistTtCutoffBonusCap", 256..=4096;
    /// Base of the fail-low reward factor for the parent's quiet move.
    hist_fail_low_base = 93, "CoreHistFailLowBase", 0..=400;
}

// The proof-search cluster's coordinates, on the `b3proof` arm only, so the
// accepted search advertises none of them. Seeds follow the core's rule: the
// donor's shape, Rarog's own fitted magnitude where one exists, otherwise the
// donor's value converted (evaluation units x0.457). Categoricals are
// switches, never SPSA coordinates.
#[cfg(feature = "b3proof")]
search_params! {
    struct ProofParams, generated_proof_param_checks;

    // Null move.
    /// Categorical, never an SPSA coordinate. 0 tries the null move at every
    /// node off the PV line (`!tt_pv`), the population Rarog measured; 1 at
    /// expected cut nodes only, PV-line cut nodes included.
    nmp_nodes = 0, "CoreNmpNodes", 0..=1;
    /// Entry margin above beta: `max(2, base - depth_term * depth +
    /// tt_pv_term * tt_pv - improvement_term * improvement/1024 - cutoff *
    /// (child cutoffs < 2))`, evaluation units.
    nmp_base = 150, "CoreNmpBase", 0..=400;
    nmp_depth = 4, "CoreNmpDepth", 0..=20;
    nmp_tt_pv = 50, "CoreNmpTtPv", 0..=200;
    nmp_improvement = 43, "CoreNmpImprovement", 0..=200;
    nmp_cutoff = 10, "CoreNmpCutoff", 0..=60;
    /// Reduction in 1024ths of a ply: `base + improving_term * improving +
    /// depth_term * depth + eval * clamp(estimate - beta, 0, clamp)/128`.
    nmp_r_base = 4_407, "CoreNmpRBase", 2048..=8192;
    nmp_r_improving = 917, "CoreNmpRImproving", 0..=2048;
    nmp_r_depth = 265, "CoreNmpRDepth", 64..=640;
    nmp_r_eval = 1_044, "CoreNmpREval", 0..=3072;
    nmp_r_clamp = 542, "CoreNmpRClamp", 128..=1536;
    /// From this depth a null fail-high outside a verification region is
    /// verified by a search with the null move disabled over the region.
    nmp_verify_depth = 16, "CoreNmpVerifyDepth", 4..=32;

    // ProbCut.
    /// Categorical, never an SPSA coordinate. 0 runs ProbCut at every node
    /// off the PV line (`!tt_pv`), the population Rarog measured; 1 at
    /// expected cut nodes only, and not when the TT move is quiet.
    probcut_nodes = 0, "CoreProbcutNodes", 0..=1;
    /// `probcut_beta = beta + base - improving_term * improving`, evaluation
    /// units; the base is Rarog's fitted `ProbCutMargin`.
    probcut_base = 180, "CoreProbcutBase", 50..=400;
    probcut_improving = 39, "CoreProbcutImproving", 0..=150;
    /// Captures searched at most per node.
    probcut_move_cap = 4, "CoreProbcutMoveCap", 1..=16;
    /// The verification depth falls one ply below `depth - 4 - improving`
    /// for every `div` units the qsearch pass cleared `probcut_beta` by.
    probcut_depth_div = 146, "CoreProbcutDepthDiv", 48..=512;
    /// A shallower verification must clear `probcut_beta` plus this per ply
    /// it saved.
    probcut_adjust = 90, "CoreProbcutAdjust", 0..=300;
    /// A cut returns this many 1024ths of the way from its score to beta.
    probcut_lerp = 276, "CoreProbcutLerp", 0..=1024;
    /// Categorical, never an SPSA coordinate. 1 returns `beta + margin`
    /// before the capture search when a stored lower bound at most four
    /// plies shallower already clears it; 0 does not.
    probcut_tt_served = 0, "CoreProbcutTtServed", 0..=1;
    probcut_tt_margin = 208, "CoreProbcutTtMargin", 50..=600;

    // Singular extensions, multi-cut, low-depth singular extension.
    /// The singular margin is `margin * (exact ? ceil(depth/4) : depth)`,
    /// plus `margin * depth` at a PV-line node searched with a null window;
    /// Rarog's fitted `4 * depth` is the seed.
    singular_margin = 4, "CoreSingularMargin", 1..=12;
    /// Categorical, never an SPSA coordinate. The least depth of a singular
    /// candidate: 0 from depth 4, the accepted search's; 1 from depth 5, 6
    /// on a PV line, the donor's.
    singular_floor = 0, "CoreSingularFloor", 0..=1;
    /// A stored lower bound seeds a singular candidate only when it is at
    /// most this many plies shallower than the node; 2 here, not the
    /// accepted search's `SingularTtDepthMargin` of 3, because 2 measured
    /// +12.9 ± 9.4 Elo on this arm in 2,000 games (RAR-S80).
    singular_tt_depth_margin = 2, "CoreSingularTtDepthMargin", 0..=4;
    /// A singular move extends twice when its exclusion score falls this far
    /// below the singular beta: `pv_term * PV + not_tt_pv * (PV and not
    /// stored on a PV line) - quiet * quiet TT move - corr * |correction|/128
    /// + base`; the base, zero as in the donor, sets the bar off the PV.
    sing_double_base = 0, "CoreSingDoubleBase", -100..=200;
    sing_double_pv = 89, "CoreSingDoublePv", 0..=300;
    sing_double_not_tt_pv = 22, "CoreSingDoubleNotTtPv", 0..=100;
    sing_double_quiet = 7, "CoreSingDoubleQuiet", 0..=50;
    sing_double_corr = 7, "CoreSingDoubleCorr", 0..=50;
    /// Three times at this margin, the same terms plus a base.
    sing_triple_pv = 105, "CoreSingTriplePv", 0..=350;
    sing_triple_not_tt_pv = 26, "CoreSingTripleNotTtPv", 0..=100;
    sing_triple_quiet = 9, "CoreSingTripleQuiet", 0..=50;
    sing_triple_corr = 7, "CoreSingTripleCorr", 0..=50;
    sing_triple_base = 16, "CoreSingTripleBase", 0..=80;
    /// A multi-cut returns this many 1024ths of the way from its score to
    /// beta.
    sing_multicut_lerp = 412, "CoreSingMulticutLerp", 0..=1024;
    /// A cut node at depth 7 or less with no singular candidate extends its
    /// first move when the estimate is this far below alpha.
    ldse_margin = 11, "CoreLdseMargin", 0..=100;
    /// Late moves at a node whose TT move beat the exclusion search reduce
    /// more: `clamp(slope * (tt_move_score - singular_score - offset)/128, 0,
    /// cap)`, in 1024ths of a ply.
    lmr_singular_slope = 1_085, "CoreLmrSingularSlope", 0..=3072;
    lmr_singular_offset = 85, "CoreLmrSingularOffset", 0..=300;
    lmr_singular_cap = 2_021, "CoreLmrSingularCap", 0..=4096;
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
        assert!(p.qs_see_margin >= 0);
        assert!(p.qs_see_clamp_lo < p.qs_see_clamp_hi);
        assert!(p.qs_see_bad_floor <= 0);
        assert!(p.singular_beta_mult > 0);
        // 4.3 arms land INERT: these three defaults must reproduce pre-4.3
        // behaviour exactly, so the bench fingerprint gates the refactor rather
        // than the arm. A bake that moves one of them is changing play and owes
        // an SPRT, so pin the inert values here — this assert is the tripwire.
        assert_eq!(p.singular_tt_depth_margin, 3, "4.3 arm B must land inert");
        // Not on/off switches: these two carry the baseline value itself, so
        // the inert position is the current constant rather than zero.
        assert_eq!(
            p.nmp_min_non_pawn_pieces, 1,
            "4.4c material guard must reproduce has_non_pawn_material"
        );
        assert_eq!(
            p.singular_double_margin, 20,
            "4.4c double-extension margin must land inert"
        );
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
        assert!(p.hist_bonus_mul > 0);
        assert!(p.hist_bonus_sub >= 0);
        assert!(p.hist_bonus_max > 0 && p.hist_bonus_max <= 16_384);
        assert!(p.hist_malus_mul > 0);
        assert!(p.exact_bonus_pct >= 0);
        assert!(p.capture_malus_pct >= 0);
        assert!(p.surprise_bonus_pct > 0);
        assert!(p.hist_malus_sub >= 0);
        assert!(p.hist_malus_max > 0 && p.hist_malus_max <= 16_384);
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
