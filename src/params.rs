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
/// Syntax: `field = default, "UciName", min..=max;` — doc comments and plain
/// `//` section comments pass through as normal.
macro_rules! search_params {
    ($(
        $(#[$meta:meta])*
        $field:ident = $default:literal, $uci:literal, $min:literal ..= $max:literal;
    )+) => {
        /// Tunable search parameters — every field is a UCI `spin` option in
        /// tune builds. Defaults are the current accepted integration-head
        /// values; the trailing comment on each declaration records its bake
        /// history. To re-tune, copy the weather-factory configs from
        /// `tools/spsa_configs/` and run `./tools/spsa.ps1` (see that
        /// directory's README).
        #[derive(Clone, Debug)]
        pub struct SearchParams {
            $( $(#[$meta])* pub $field: i32, )+
        }

        impl Default for SearchParams {
            fn default() -> Self {
                Self { $( $field: $default, )+ }
            }
        }

        impl SearchParams {
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
        mod generated_param_checks {
            use super::*;

            /// Every default must sit inside its own declared range. Cheap, but
            /// it is the invariant the four-way duplication used to break.
            #[test]
            fn defaults_are_within_declared_ranges() {
                let p = SearchParams::default();
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
    /// Initial aspiration window half-width (centipawns). [search.rs:615]
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
    /// Weight (percent) of the running average of completed root scores in the
    /// window centre, against this thread's last completed score. 0 = the old
    /// pure last-score centre. A centre that follows an average is less
    /// whipsawed by a single noisy iteration.
    asp_center_avg_pct = 0, "AspCenterAvgPct", 0..=100;
    /// Magnitude scaling: the initial half-width gains `|centre| / div`, so a
    /// won position opens a proportionally wider window than an equal one.
    /// 0 disables it and reproduces the flat initial delta.
    asp_magnitude_div = 0, "AspMagnitudeDiv", 0..=64;
    /// Depth reduction per consecutive fail-HIGH on the re-search. A fail-high
    /// means the move is better than believed; confirming that at slightly
    /// reduced depth is cheaper and usually sufficient. 0 = old behaviour
    /// (always re-search at full depth).
    ///
    /// ⛔ **NOT in the `aspiration` SPSA group — it is effectively discrete.**
    /// With only four reachable values, the perturbation needed to move it is
    /// a large fraction of its whole range, and the coverage audit rejects it
    /// outright: at `step = 1` the perturbation rounds to zero from iteration
    /// 894 of 5,000, so for 82 % of the run both arms would receive the SAME
    /// integer while the knob kept being updated by the other knobs' gradient —
    /// a random walk that drags the joint fit. Raising the step to keep it
    /// alive would make one perturbation span two thirds of the range, which is
    /// a noisy A/B wearing a tuner's clothes rather than a gradient.
    /// Handled like the project's other discrete knobs (`CorrGuardCapture`,
    /// `FutilityImprovingDir`): gate it on its own AFTER the continuous tune
    /// lands, under the winning vector.
    asp_fail_high_reduction = 0, "AspFailHighReduction", 0..=3;

    /// 9.7.5 lead (found while decomposing 8.11): minimum TT entry depth for
    /// the `eval_for_pruning` refinement, which lets a TT score stand in for
    /// the static eval when deciding RFP / razoring / NMP / LMP.
    ///
    /// negamax hands off to qsearch at `depth <= 0`, so it only ever STORES at
    /// depth >= 1 — depth-0 entries are exactly the qsearch ones. Today any
    /// entry qualifies at any depth, so a **depth-0 qsearch bound can decide
    /// an RFP cut at depth 8**, which is hard to defend on principle. It also
    /// coupled the pruning group to qsearch's old fail-hard inflation: those
    /// bounds were literally `alpha`, the group was SPSA-fitted against them,
    /// and 8.11's honest bounds cost +14.4% nodes purely through this path.
    ///
    /// 0 = no guard (since qsearch stores depth 0). The 10.4.6(a) SPSA final
    /// theta retained 0, so the experimental guard remains disabled. 1
    /// excludes qsearch entries; higher demands progressively deeper evidence
    /// before the TT may override the eval.
    eval_prune_tt_min_depth = 0, "EvalPruneTtMinDepth", 0..=8;

    /// Futility pruning base margin.
    /// Formula: `(base + not_improving_coeff * not_improving_i) * depth`. [search.rs:1003]
    futility_base = 52, "FutilityBase", 20..=200;  // was 70 → 82 → 86 → 60 → 52
    /// Extra futility margin added when *not* improving (multiplied by
    /// `not_improving_i`). Larger value → prune less when not improving.
    futility_not_improving = 51, "FutilityNotImproving", 0..=120;  // was 20 → 51 → 49 → 42 → 51

    /// Razoring coefficient. Prune if `eval + coeff * depth < alpha`. [search.rs:1007]
    razoring_coeff = 274, "RazoringCoeff", 50..=300;  // was 150 → 194 → 191 → 193 → 274

    /// Null-move pruning depth coefficient. [search.rs:1012]
    /// Allow NMP when `eval >= beta - coeff * depth - improving_bonus * improving`.
    nm_depth_coeff = 12, "NullMoveDepthCoeff", 2..=40;  // was 12 → 14 → 15 → 10 → 12
    /// Null-move pruning improving bonus. [search.rs:1012]
    nm_improving_bonus = 35, "NullMoveImprovingBonus", 0..=80;  // was 24 → 25 → 32 → 35

    /// LMP prune-margin base.
    /// Formula: `(base + not_improving_coeff * not_improving_i) * depth`. [search.rs:1182]
    lmp_base = 80, "LmpBase", 30..=200;  // was 90 → 115 → 88 → 80
    /// Extra LMP prune-margin added when *not* improving (multiplied by
    /// `not_improving_i`). Larger value → prune less when not improving.
    lmp_not_improving = 64, "LmpNotImproving", 0..=120;  // was 25 → 53 → 57 → 63 → 64

    /// Quiet-history pruning coefficient (stored positive; applied as `-(coeff * depth)`).
    /// [search.rs:1186]
    quiet_hist_prune_coeff = 5_617, "QuietHistPruneCoeff", 1000..=10000;  // was 4000 → 4372 → 4419 → 5069 → 5617


    /// SEE bad-capture threshold coefficient (stored positive; applied as `-(coeff * depth)`).
    /// [search.rs:1195]
    see_pruning_coeff = 66, "SeePruningCoeff", 20..=200;  // was 83 → 51 → 66
    /// SEE bad-capture threshold maximum magnitude (floor of `-(coeff * depth)`). [search.rs:1195]
    see_pruning_max = 955, "SeePruningMax", 200..=1600;  // was 804 → 869 → 955

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

    /// Singular-extension beta multiplier. `singular_beta = tt_score - mult * depth`. [search.rs:1215]
    singular_beta_mult = 4, "SingularBetaMult", 1..=8;  // was 2 → 4 → 6 → 4

    /// 4.3 arm B — how far below the node depth a TT entry may sit and still
    /// seed a singular verification window (`ev.depth >= depth - margin`).
    ///
    /// 3 = current behaviour, and 3 is exactly the depth ProbCut writes
    /// (`depth - 3`), so today a margin-shifted speculative score is admitted at
    /// the boundary: RAR-S22 measured 32 of 101 sampled attempts sitting on that
    /// signature and RAR-S24 measured 41 of 101 seeded by a window-contradicting
    /// score. Provenance is not persisted, so the only available lever is the
    /// depth band itself — 2 excludes the ProbCut band entirely, at the cost of
    /// also excluding a legitimate full search at `depth - 3`. That trade is
    /// what the registered arm measures; it is not obviously good.
    singular_tt_depth_margin = 3, "SingularTtDepthMargin", 0..=4;

    /// 4.3 arm D — plies subtracted from the node depth when ProbCut stores its
    /// speculative result (`depth - adj`).
    ///
    /// 3 = current behaviour. The verification search actually ran at
    /// `depth - 4`, which makes `depth - 3` the conventional (and defensible)
    /// parent-bound depth, but it also places the entry exactly on
    /// `singular_tt_depth_margin`'s boundary. 4 stores the depth the search
    /// literally measured, which denies the entry singular authority as a side
    /// effect rather than by a special case. Interacts with arm B — gate them
    /// separately before any combination.
    probcut_store_depth_adj = 3, "ProbCutStoreDepthAdj", 3..=4;

    /// 4.3 arm C — minimum stored depth before a TT bound may refine the
    /// QSEARCH stand pat.
    ///
    /// 0 = current behaviour, and provably so: every stored depth is >= 0 and a
    /// post-conversion `ev.score` can never equal `VALUE_NONE`, so the guarded
    /// form at 0 admits exactly what the unguarded form admitted. This knob
    /// exists to make the audited asymmetry against `EvalPruneTtMinDepth`
    /// adjustable rather than structural. 1 excludes all depth-0 entries, which
    /// is 67.5% of stores and would gut most of RAR-S02's accepted +6.5 Elo
    /// mechanism — a real risk, which is why it is gated and not assumed.
    qs_refine_min_depth = 0, "QsRefineMinDepth", 0..=4;

    /// LMP count base. `count = base + 2 * depth * depth / 3`. [search.rs:2394]
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
    /// name is a misnomer — the live condition (`search.rs`, LMR block) fires
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

    /// ProbCut beta margin (cp). `probcut_beta = beta + margin`. [search.rs:1108]
    /// Re-tuned in the Phase 5 SPSA wave after the Phase 4 eval re-fit changed
    /// what a centipawn means; the flat-margin form is the current accepted
    /// shape (an earlier improving-aware 3-parameter port was tried in Phase 2
    /// and dropped, H0 -24.5 Elo — see tools/spsa_configs/README.md).
    probcut_margin = 180, "ProbCutMargin", 60..=400;

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
    futility_improving_dir = 0, "FutilityImprovingDir", 0..=1;

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
    /// 8.4(a) — reward a QUIET TT move on a TT lower-bound cutoff, as a % of
    /// `history_bonus`. Seed 0 = today's behavior (the cutoff returns with
    /// zero feedback). ⚠ Basilisk's unscaled version was bench-vetoed (+82%
    /// nodes: TT cutoffs are so frequent the position-independent main
    /// history saturates). The knob lets SPSA find a small value or stay at
    /// 0; at clock TC a node explosion loses games, so the gradient sees it.
    tt_cutoff_bonus_pct = 0, "TtCutoffBonusPct", 0..=150;  // 8.4 histcov fit: stayed at 0 (SPSA drifted 0→5→~3, i.e. OFF)
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
    /// A/B knob (Phase 8.1b): `1` disables the between-search history halving
    /// (`age_history`). First no-aging attempt was −12.4 — it relied on halving
    /// as its only decay; the 8.1 malus provides in-search decay, so retry.
    /// Gate `[0,3]` as a separate SPRT after 8.1 resolves. Default 0 = halve.
    hist_no_aging = 0, "HistNoAging", 0..=1;  // 8.1b REJECTED (-6.6 Elo): between-search halving is required even with 8.1's malus decay. Do not retry.

    // ── Phase 8.5: correction-history semantics + magnitude margins + blend ──
    // The continuous margins/weights were included in the accepted 10.4.6(a)
    // joint selectivity fit. Preserve even off-valued mechanisms through the
    // NNUE transition: the post-NNUE retune may reactivate them.
    //
    /// 8.5(a) — guard: skip a correction UPDATE whose causing/best move is a
    /// CAPTURE. Today a capture beta-cutoff trains correction, teaching the
    /// evaluator to absorb search tactics that then feed back into pruning. The
    /// wrong-bound-direction half of the guard is ALREADY enforced at both
    /// update sites (Lower needs `score > static_eval`; Upper needs `diff < 0`),
    /// verified in code — so 8.5(a) adds only the capture guard. Seed 0 = today;
    /// it remains a discrete A/B knob and was excluded from 10.4.6(a).
    corr_guard_capture = 0, "CorrGuardCapture", 0..=1;  // 8.5 closed neutral 2026-07-27; retain through post-NNUE retune
    /// 8.5(b) — magnitude margins: scale forward pruning / LMR by |correction|.
    /// A large correction means the raw static eval is being heavily adjusted
    /// and is less trustworthy, so prune/reduce LESS (conservative-when-
    /// uncertain, the Reckless form). `|corr| = |static_eval − raw_static_eval|`,
    /// already computed per node. Each knob adds `|corr| · knob / 128` to a
    /// margin (or subtracts it from the LMR reduction in 1024ths). Seed 0 = off.
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
    // falling-eval × best-move-instability × effort (search.rs soft-stop block);
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
        assert_eq!(p.probcut_store_depth_adj, 3, "4.3 arm D must land inert");
        assert_eq!(p.qs_refine_min_depth, 0, "4.3 arm C must land inert");
        assert_eq!(
            p.eval_prune_tt_min_depth, 0,
            "4.3 arm A must land inert (the knob predates 4.3)"
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
        assert!(p.futility_improving_dir == 0 || p.futility_improving_dir == 1);
        assert!(p.hist_bonus_mul > 0);
        assert!(p.hist_bonus_sub >= 0);
        assert!(p.hist_bonus_max > 0 && p.hist_bonus_max <= 16_384);
        assert!(p.hist_malus_mul > 0);
        assert!(p.tt_cutoff_bonus_pct >= 0);
        assert!(p.exact_bonus_pct >= 0);
        assert!(p.capture_malus_pct >= 0);
        assert!(p.surprise_bonus_pct > 0);
        assert!(p.hist_malus_sub >= 0);
        assert!(p.hist_malus_max > 0 && p.hist_malus_max <= 16_384);
        assert!(p.hist_no_aging == 0 || p.hist_no_aging == 1);
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
