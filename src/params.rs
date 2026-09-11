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
    /// Categorical and excluded from SPSA. Root aspiration stays off now;
    /// reconsider only in the registered post-NNUE Phase-7.3 review.
    asp_fail_high_reduction = 0, "AspFailHighReduction", 0..=3;

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
    see_pruning_max = 955, "SeePruningMax", 200..=1600;

    /// 4.6.4 QUIET SEE PRUNING. `depth` 0 = OFF = accepted behaviour.
    ///
    /// Rarog's only SEE prune lives in the CAPTURE branch, behind `see < 0`.
    /// Its quiet branch has move-count and futility pruning and nothing else,
    /// so a quiet move that simply hangs material is never pruned for that
    /// reason. The reference prunes exactly that case, and annotates it at
    /// ~20 Elo. This is the one gap in Step 13 that the counters could not
    /// explain as a population effect: `see_prune` runs at 0.20x the oracle's
    /// per-node rate, five times down, against a 30% shrinkage in the late-move
    /// population.
    ///
    /// Threshold is quadratic in the prospective depth, as the reference's is:
    /// `-coeff * d * d`, floored by `see_pruning_max`. `coeff` is seeded from
    /// Rarog's own capture-side `see_pruning_coeff` (66), not from the
    /// reference's constants.
    quiet_see_prune_depth = 0, "QuietSeePruneDepth", 0..=12;
    quiet_see_prune_coeff = 25, "QuietSeePruneCoeff", 1..=200;

    /// 4.6.5 SKIP QUIETS once move-count pruning fires. 0 = off = accepted.
    ///
    /// The reference sets a `moveCountPruning` flag and passes it to the move
    /// picker, which then stops emitting quiets entirely. Rarog generated,
    /// scored and individually rejected every remaining quiet instead — the
    /// same decision, paid for once per move rather than once per node.
    skip_quiets_on_move_count = 0, "SkipQuietsOnMoveCount", 0..=1;  // was 804 → 869 → 955

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
    /// 3 = current behaviour, and 3 is exactly the depth a same-node ProbCut
    /// writes (`depth - 3`), so that signature is admitted at the boundary:
    /// RAR-S22 measured 32 of 101 sampled attempts there. Margin 2 excludes the
    /// whole `depth - 3` band, including legitimate full-search entries, but it
    /// is not a provenance guarantee: a ProbCut entry produced by an earlier
    /// deeper search can still qualify at a shallower consumer. RAR-S31 found
    /// value 2 positive on a tune binary, but the ~3 Elo knob was parked under
    /// the material-gain policy; explicit provenance is implemented in 4.3c.
    singular_tt_depth_margin = 3, "SingularTtDepthMargin", 0..=4;

    // ── 4.4a — NMP/IIR/singular cooperation, all landed INERT ───────────────
    // Every switch here defaults to the accepted baseline, so `bench 13` stays
    // 6,502,902. They exist so 4.4 can size each mechanism with cheap
    // deterministic ablations and then gate ONE coherent bundle, instead of
    // repeating 4.3's five standalone gates that banked nothing.

    /// Suppress NMP for the WHOLE null-verification subtree, not just its root.
    ///
    /// 0 = accepted baseline. Verification calls `negamax` with
    /// `allow_null = false`, but that only covers its own root — descendants
    /// re-enable null, so the search meant to check a null cutoff can itself
    /// null-prune. The 4.1 census measured this happening (`nmp_nested_attempt`
    /// = 42 sampled). 1 makes the suppression cover the subtree via
    /// `Searcher::nmp_verify_nesting`.
    ///
    /// PLAN 4.4 states nested verification nulls are forbidden unless proven, so
    /// 1 is the intended direction — but it is a strength change and rides the
    /// 4.4 bundle gate, not a standalone one.
    nmp_suppress_null_in_verification = 0, "NmpSuppressNullInVerification", 0..=1;

    /// Per-mechanism `tt_pv` eligibility (4.4). One inherited PV bit currently
    /// vetoes RFP, razoring, NMP and ProbCut **together** — 24,361 exact vetoes
    /// on `bench 13`, and the bit may have been inherited from a search that
    /// proved nothing about this window. 0 keeps the shared veto (baseline); 1
    /// lets that one mechanism run at a `tt_pv` node.
    ///
    /// Separate knobs because the four have different risk: RFP and razoring
    /// return immediately on a margin, NMP spends a reduced search first, and
    /// ProbCut spends a qsearch plus a reduced search. They should not be
    /// forced to share one eligibility rule just because they happen to share
    /// one `if`.
    rfp_allow_tt_pv = 0, "RfpAllowTtPv", 0..=1;
    /// See `rfp_allow_tt_pv`.
    razor_allow_tt_pv = 0, "RazorAllowTtPv", 0..=1;
    /// See `rfp_allow_tt_pv`.
    nmp_allow_tt_pv = 0, "NmpAllowTtPv", 0..=1;
    /// See `rfp_allow_tt_pv`.
    probcut_allow_tt_pv = 0, "ProbCutAllowTtPv", 0..=1;

    // ── 4.4b — NMP and singular guards, all landed INERT ────────────────────

    /// Require a cut node before attempting NMP.
    ///
    /// 0 = accepted baseline (any non-PV node may try). 1 restricts NMP to nodes
    /// the caller expects to fail high, which is where a null refutation is
    /// actually cheap information; at an all-node a failed null costs a reduced
    /// search and tells us little. PLAN 4.4's "cut-node guard".
    nmp_require_cut_node = 0, "NmpRequireCutNode", 0..=1;

    /// Which eval the NMP threshold reads.
    ///
    /// 0 = accepted baseline, `eval_for_pruning` — i.e. TT-refined when a bound
    /// is available, which the 4.1 census measured at 78 of 190 sampled
    /// attempts (41%). 1 reads the corrected static eval instead, so a null
    /// decision never rests on a bound some other window produced. This is
    /// PLAN 4.4's "compare raw vs TT-adjusted null windows"; note RAR-S29/S30
    /// showed TT refinement is *earning* strength elsewhere, so the expected
    /// sign here is genuinely unknown.
    nmp_use_static_eval = 0, "NmpUseStaticEval", 0..=1;

    // ── 4.9c SMP depth diversity ──────────────────────────────

    /// Let helper threads SKIP iterations, so the pool is diversified in depth
    /// and not only in width.
    ///
    /// 0 = accepted baseline: every thread walks `depth = 1, 2, 3, …` in
    /// lockstep. `thread_id` seeds only the LMR jitter and the root-move
    /// rotation, both of which vary WIDTH; nothing has ever varied depth.
    ///
    /// RAR-R08 measured what that costs. At 16T the per-thread depths read
    /// 22,21,20,22,22,22,21,21,21,22,20,22,21,22,22,22 — **no thread ever
    /// exceeds the depth one thread reaches alone**. Lazy SMP's depth gain
    /// comes from threads running AHEAD and leaving deeper entries for the
    /// laggards to skip to; with no stagger there is nothing to skip to, which
    /// is also why the shallow-rejection deficit sits at 2.4 plies whatever the
    /// thread count: every hit comes from that thread's own last iteration.
    ///
    /// The headroom is arithmetic: at EBF 2.449, 10.34x the nodes is 2.61
    /// plies at 16T (2.27 at 8T, 1.48 at 4T) against a realized gain of zero.
    ///
    /// ⚠ Diversification has LOST twice in this engine — RAR-R06's history
    /// blending and jitter, and 9.7.5(j)'s pool-view instability at −5.54.
    /// PLAN 4.9 therefore forbids reopening it "without a specific measured
    /// independent-work failure"; RAR-R08 is that measurement, which is why
    /// this exists at all. It still needs a 4T/8T gate, not a depth reading.
    ///
    /// Inert at `Threads = 1` twice over: a serial search has no shared state,
    /// and thread 0 never skips regardless.
    smp_iteration_skip = 0, "SmpIterationSkip", 0..=1;

    // ── 4.7b root confidence ──────────────────────────────────
    // ONE completed-iteration snapshot (`RootConfidence` in search.rs), two
    // consumers, each behind its own switch. Aspiration is separate from time
    // on purpose: it has been changed twice and LOST twice (RAR-S17 −4.52,
    // RAR-S20's rejected fit), so it is the highest-risk consumer of any
    // confidence model and must be ablatable on its own.

    /// Let the root-confidence snapshot drive the clock instead of effort alone.
    ///
    /// 0 = accepted baseline: the between-iteration soft target scales by
    /// `best-move instability × effort factor`. 1 keeps the instability factor
    /// and swaps the EFFORT factor for the confidence factor — the same
    /// `TmEffortHigh`/`TmEffortLow` interpolation, evaluated on the whole
    /// confidence model instead of on effort alone.
    ///
    /// That is what makes it double-count-free: effort is an input to the
    /// confidence scalar, so it is charged exactly once, and the reachable
    /// range of the multiplier is unchanged. The arm adds no TM coordinate.
    ///
    /// ⚠ Invisible to `bench`, which is depth-limited: the soft target is never
    /// binding there. Only a timed gate can resolve it.
    root_conf_time = 0, "RootConfTime", 0..=1;

    /// Let the root-confidence snapshot size the aspiration window.
    ///
    /// 0 = accepted baseline: the initial half-width is symmetric and depends
    /// only on `AspirationDelta` (plus `AspMagnitudeDiv`). 1 scales it between
    /// `AspConfWidePct` (no confidence) and `AspConfNarrowPct` (full
    /// confidence) and widens each side by `AspConfFailPct` per fail the
    /// PREVIOUS iteration took on that side, capped at three bumps.
    ///
    /// ⚠ Highest-risk arm in 4.7. Keep it separable from `RootConfTime` in
    /// every gate.
    root_conf_aspiration = 0, "RootConfAspiration", 0..=1;

    /// Feed the POOL's mean best-move instability into the confidence snapshot
    /// instead of this thread's own.
    ///
    /// 0 = accepted baseline: each thread reads its own instability, which
    /// 9.7.5(j) confirmed is the better signal for RESULT purposes (the pool
    /// view lost at −5.54 ± 8.15 over 2,760 games at 4T; RAR-R05 rejected raw
    /// pool instability as a direct time multiplier at −5.54 as well).
    ///
    /// 1 pools it, and pooling can only ever reach the CLOCK: instability is an
    /// input to `RootConfidence::time_factor` and to nothing else, and result
    /// ownership is decided by `select_parallel_result` from `SearchResult`
    /// values that carry no confidence at all. That is Plan 4.7's "pool worker
    /// instability for time, not result ownership", enforced by construction
    /// rather than by convention.
    ///
    /// Inert at `Threads = 1`: there is no shared state to pool.
    root_conf_pool_instability = 0, "RootConfPoolInstability", 0..=1;

    /// Weight of the STEADINESS term (the best move's score deviation across
    /// completed iterations, from its mean/mean-square pair).
    ///
    /// The live weights were seeded EQUAL — the assumption-free seed. The
    /// Phase-4 fit tunes deviation/window while effort stays fixed as the
    /// normalized scalar's reference, avoiding an unidentifiable common scale.
    /// Steadiness is the best-populated of the three: the measured deviation
    /// spreads 18.1 / 26.2 / 50.8 / 5.0 % across its four buckets.
    root_conf_w_deviation = 100, "RootConfWeightDeviation", 0..=1000;
    /// Weight of the EFFORT term (share of the iteration spent on the best
    /// move), normalised by the same `effort_term` the baseline clock uses.
    ///
    /// ⚠ Sparse, and knowably so: the term is above its floor on only
    /// **47 of 520** bench iterations (9.0%, RAR-S47), because effort averages
    /// 37.9% while the band starts at 79%. It is seeded live anyway, unlike
    /// the rejected root-gap term, because a sparse term is not a degenerate one — its
    /// zeros are a real reading of "the best move did not dominate the
    /// iteration", whereas the gap's zeros are an artefact of the search window.
    root_conf_w_effort = 100, "RootConfWeightEffort", 0..=1000;
    /// Weight of the WINDOW term (aspiration re-searches this iteration).
    /// Well populated: 297 of 520 iterations take at least one re-search.
    root_conf_w_window = 100, "RootConfWeightWindow", 0..=1000;

    /// Half-confidence point of the steadiness term, in centipawns of
    /// deviation. Seeded just inside the MEASURED median bucket (32–127 holds
    /// 50.8% of iterations with 44.2% below it, RAR-S47) rather than at a round
    /// number, so the term spans its range on real data instead of saturating.
    root_conf_dev_scale = 36, "RootConfDevScale", 1..=400;

    /// Clock multiplier at ZERO confidence, in ten-thousandths — the endpoint
    /// `RootConfTime` interpolates DOWN from as confidence rises.
    ///
    /// These two exist rather than reusing `TmEffortHigh`/`TmEffortLow`, which
    /// would have been free, because reusing them made the arm a LEVEL change
    /// instead of a shape change. Measured (RAR-S47): the baseline effort
    /// factor sits at its 0.924 endpoint on 91% of iterations, so any
    /// confidence above zero interpolates strictly below it — the arm cut the
    /// total budget by **8.85%** and ran shorter on 507 of 520 iterations. A
    /// uniform 9% time cut is worth several Elo on its own, so a gate on that
    /// candidate would have measured the cut, not the redistribution.
    ///
    /// Seeded so that it does not. The span is the baseline's own (0.214), and
    /// the endpoints are shifted up until the MEASURED mean multiplier over the
    /// bench corpus matches the baseline's — i.e. the arm at its seed spends
    /// the same total time and only moves it between positions.
    ///
    /// ⚠ Calibrated on `bench`: fixed depth 13, one thread, 40 positions. The
    /// confidence distribution under a real clock is not required to match, so
    /// this is level-neutral on the proxy, not proven level-neutral in play.
    /// The gate is what settles that, and `TmOptScale` is the coordinate that
    /// absorbs any residue.
    tm_conf_high = 10_060, "TmConfHigh", 6000..=14000;  // 1.006
    /// Clock multiplier at FULL confidence. See `tm_conf_high`.
    tm_conf_low = 7_920, "TmConfLow", 4000..=12000;  // 0.792

    /// Aspiration half-width at ZERO confidence, percent of the baseline
    /// delta. Excluded from Phase 4 after two aspiration losses and expensive
    /// sizing; retained only for the registered post-NNUE Phase-7.3 review.
    asp_conf_wide_pct = 200, "AspConfWidePct", 50..=400;
    /// Aspiration half-width at FULL confidence, percent of the baseline delta.
    asp_conf_narrow_pct = 50, "AspConfNarrowPct", 10..=200;
    /// Extra width, in percent, per fail the previous iteration took on THIS
    /// side of the window. Capped at three bumps, so the widening is bounded
    /// however the scores behave.
    asp_conf_fail_pct = 50, "AspConfFailPct", 0..=200;

    // ── 4.6c ──────────────────────────────────────────────────

    /// Ordering bonus for a quiet move that gives check.
    ///
    /// 32000 = the historical flat `DIRECT_CHECK_BONUS`, promoted from a bare
    /// constant to a coordinate so 4.10 can fit it. The 4.3 audit wanted this
    /// split into safe/losing classes; that split was implemented, measured
    /// NON-FUNCTIONAL (RAR-S44: zero losing-check population because
    /// `see_ge` is trivially true for a non-capture) and reverted. A correct
    /// classifier needs a different predicate and is 4.10 ordering work.
    check_bonus_safe = 32000, "CheckBonusSafe", 0..=32000;


    // ── 4.6b ──────────────────────────────────────────────────

    /// Derive LMP, futility and SEE pruning from the same PROSPECTIVE depth LMR
    /// will search the move at, instead of from raw `depth`.
    ///
    /// 0 = accepted baseline: every consumer reads raw `depth`, which is the
    /// incoherence the 4.3 audit recorded — a move about to be reduced by 3 plies
    /// was still judged for pruning as if it were not. 1 switches all three onto
    /// `prospective_depth = depth - 1 - lmr_reduction(...)`, floored at 1.
    ///
    /// All three move together on purpose. Switching them one at a time would
    /// recreate precisely the mixed-depth incoherence this step exists to remove,
    /// so there is no per-consumer knob.
    ///
    /// The shared depth excludes two terms, both documented at
    /// `lmr_reduction_units`: the per-thread jitter (drawn once, at the real
    /// reduction site, and not drawn at all at `Threads = 1`) and the singular
    /// extension (not yet known when pruning runs). A `debug_assert` checks the
    /// two callers derive identical reduction units, so they cannot drift.
    ///
    /// ⚠ Expect this to PRUNE MORE, since a reduced depth passes `depth <= N`
    /// guards more often and shrinks the margins. The 0.47% measured pruning
    /// overlap (RAR-S21) says deduplication is not the prize here — the prize, if
    /// any, is that the decisions become coherent with the depth actually
    /// searched.
    selectivity_prospective_depth = 0, "SelectivityProspectiveDepth", 0..=1;

    // ── 4.6a ──────────────────────────────────────────────────

    // ── 4.4c ────────────────────────────────────────────────────────────────

    /// Refuse NMP at a node whose TT move may be singular.
    ///
    /// 0 = accepted baseline. A node that hinges on one move is the worst place
    /// to trust a null refutation: passing concedes nothing *because* the
    /// position's value lives in a single reply, so a null cutoff there reports
    /// safety the position does not have. 1 skips NMP when this node's evidence
    /// would already admit a singular verification.
    ///
    /// This is PLAN 4.4's "potential-singularity protection". Note the
    /// predicate is evaluated BEFORE the move loop, so it uses node evidence
    /// only — it cannot know the TT move is legal yet, which makes it a slight
    /// over-approximation and therefore the conservative direction.
    nmp_singular_guard = 0, "NmpSingularGuard", 0..=1;

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

    /// 4.3c contract switch — whether singular verification refuses a seed from
    /// a persisted **speculative** producer (ProbCut).
    ///
    /// 0 = off, which reproduces the accepted baseline exactly; the speculative
    /// bit is still persisted, and `bench 13` stays 6,502,902 (RAR-S34 measured
    /// the bit itself as free: 0.00% nodes, +0.10% NPS). 1 enables the contract,
    /// which was measured *cheaper* than baseline — −0.19% nodes, +0.97% NPS,
    /// −1.15% time-to-depth.
    ///
    /// It lands OFF not because it is doubtful but because it is a strength
    /// change with no passing gate: the one gate it had (RAR-S34) tested it
    /// bundled with `probcut_store_actual_score`, which alone costs +5.55%
    /// time-to-depth and sank the pair to neutral. Its own prior is RAR-S31's
    /// ~5 nElo, which cannot clear `[3,10]` standalone, so **Phase 4.4 turns it
    /// on inside the evidence-bound-singularity bundle** where the combined
    /// effect can resolve. Do not flip it alone without a gate.
    singular_reject_speculative = 0, "SingularRejectSpeculative", 0..=1;

    /// 4.3c ablation — whether ProbCut persists its ACTUAL fail-high (1) or the
    /// conservative margin-shifted value the node returns to its caller (0).
    ///
    /// 0 = off and measured cheapest. RAR-S34 attributed the entire ~4.3%
    /// time-to-depth headwind of the first 4.3c candidate to this one change:
    /// +1.62% nodes and −3.73% NPS on its own, i.e. **+5.55% time-to-depth**.
    /// The likely mechanism is RAR-S30's: a higher stored `Lower` raises
    /// `eval_for_pruning`, refinement acts almost purely as an upward correction
    /// that *prevents* razoring, so more nodes survive and the node mix shifts
    /// to expensive interior nodes.
    ///
    /// The singular contract does **not** need it — the persisted speculative
    /// bit delivers that guarantee, not the stored value — so the expensive form
    /// stays inert and is retained only for the post-NNUE Phase-7.3 review.
    probcut_store_actual_score = 0, "ProbCutStoreActualScore", 0..=1;

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
    /// 4.10 CANDIDATE: single-thread LMR-reduction jitter, in 1024ths of a
    /// ply. 0 = off = accepted behaviour. The multi-thread path is unaffected
    /// and keeps its own magnitude of 64.
    /// 4.10 CANDIDATE: unconditional LMR-reduction relief, in 1024ths of a
    /// ply, subtracted from every reduction. 0 = off = accepted behaviour.
    ///
    /// This is the DIRECTIONAL form of what RAR-S54 and RAR-S64 measured.
    /// RAR-S67 built the symmetric form (jitter) and it failed: symmetric noise
    /// has zero mean effect on the reduction, so it cannot reproduce an effect
    /// that is about reducing LESS.
    lmr_relief = 0, "LmrRelief", 0..=512;
    lmr_jitter_1t = 0, "LmrJitter1t", 0..=512;
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
    /// 4.8.1 MINIMUM REDUCED DEPTH, in plies. 0 = off = accepted behaviour.
    ///
    /// `lmr_reduction` clamps to `new_depth`, so a reduced search may run at
    /// depth 0 and be answered by quiescence. Measured at bench 13: that is
    /// **46.7%** of all applied reductions -- a prune wearing a reduction's
    /// name, counted in no pruning family, and able to fail high off a
    /// stand-pat. The reference guarantees the reduced search is at least one
    /// ply. 1 reproduces that contract.
    lmr_min_reduced_depth = 0, "LmrMinReducedDepth", 0..=2;
    /// Paired-ablation mask. Bit per mechanism, matching the oracle's:
    /// 0 razoring, 1 futility-child, 2 nullmove, 3 probcut, 4 iir,
    /// 5 shallow-pruning, 6 extensions, 7 lmr. 0 = shipped behaviour.
    /// Only consulted when built with `--features ablate`.
    ablation_mask = 0, "AblationMask", 0..=255;
    lmr_root_relief = 1536, "LmrRootRelief", 0..=2048;
    /// 4.5.5 MOVE-INDEX SEMANTICS. 0 = accepted behaviour, 1 = count every
    /// move CONSIDERED.
    ///
    /// Rarog increments `searched` at the END of the move loop, after the LMP,
    /// quiet-futility and SEE `continue`s, so a pruned move never advances it.
    /// The reference increments before its Step 13 pruning, so its count
    /// includes every legal move it looked at. Both then feed the SAME
    /// log(depth)*log(count) reduction — whose two engines' constants agree to
    /// within 2% — and the SAME move-count pruning threshold.
    ///
    /// So Rarog applies the identical formula to a systematically smaller
    /// argument, and its move-count pruning is self-limiting: pruning a move
    /// withholds the increment that would trigger more pruning. `LmpCountBase`
    /// sitting pinned at its lower rail is what that looks like from the
    /// tuner's side.
    selectivity_count_considered = 0, "SelectivityCountConsidered", 0..=1;
    /// 4.5.1 reduction-contract additions, in 1024ths of a ply. All 0 =
    /// accepted behaviour; the accepted fingerprint holds until they are
    /// fitted. The MECHANISMS come from the reference, the constants do not:
    /// its thresholds are in its own history units, and the independence
    /// boundary forbids importing them regardless.
    ///
    /// Extra reduction for a quiet move when the TT move is a capture.
    lmr_tt_capture = 0, "LmrTtCapture", 0..=2048;
    /// Relief when the TT move was proved singular at this node.
    lmr_singular_relief = 0, "LmrSingularRelief", 0..=2048;
    /// Relief when the parent was still searching late; `min` is the move
    /// count at which it applies and is inert while the relief is 0.
    lmr_parent_movecount_relief = 0, "LmrParentMoveCountRelief", 0..=2048;
    lmr_parent_movecount_min = 0, "LmrParentMoveCountMin", 0..=64;
    /// Swing from comparing this move's history with the parent move's.
    /// `margin` is the dead zone in history units.
    lmr_stat_swing = 0, "LmrStatSwing", 0..=2048;
    lmr_stat_swing_margin = 0, "LmrStatSwingMargin", 0..=32768;
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
    /// 8.5(b) — magnitude margins: scale forward pruning / LMR by |correction|.
    /// A large correction means the raw static eval is being heavily adjusted
    /// and is less trustworthy, so prune/reduce LESS (conservative-when-
    /// uncertain, the Reckless form). `|corr| = |static_eval − raw_static_eval|`,
    /// already computed per node. Each knob adds `|corr| · knob / 128` to a
    /// margin (or subtracts it from the LMR reduction in 1024ths). Seed 0 = off.
    /// 4.5c — drop the correction-uncertainty term when a TT bound has REPLACED
    /// the corrected eval.
    ///
    /// 0 = accepted baseline. The three `Corr*Scale` knobs below widen margins
    /// and shrink reductions in proportion to `|static_eval − raw_eval|`, i.e.
    /// how much the correction moved the eval. But `eval_for_pruning` can be
    /// replaced wholesale by a TT bound — RAR-S30 measured that at 28.5% of
    /// sampled hits — and when it is, the corrected eval is **discarded** while
    /// the margins are still widened by the discarded correction's magnitude.
    /// The uncertainty is charged for an adjustment no longer present in the
    /// number being tested.
    ///
    /// 1 zeroes the term in exactly that case. This is PLAN 4.5's "prevent
    /// correction double-counting across eval/pruning/reduction"; the
    /// `corr_applied_to_replaced_eval` counter sizes the affected population.
    ///
    /// Note the direction is not obviously a gain: RAR-S30 showed TT refinement
    /// is *earning* strength, and a wider margin may be doing useful work for
    /// reasons unrelated to its stated rationale. Gate it, do not assume it.
    corr_skip_when_tt_refined = 0, "CorrSkipWhenTtRefined", 0..=1;

    // ⚠ These three are NOT inert. A stale comment in `search.rs` claimed the
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

    /// 4.5b — continuation-correction weights at 2- and 4-ply distance.
    ///
    /// 0 = inert, and inert at zero *cost*: `corrected_eval_from_raw` and
    /// `update_correction` both check the weight before touching the table, so
    /// the seeded default performs neither a read nor a write.
    ///
    /// The pre-4.5b model had a single continuation slot keyed on the
    /// 1-ply-previous `(piece, to)`, which the 4.3 audit flagged as standing in
    /// for a pair structure it did not have. These add the same compact keying at
    /// distance 2 and 4; all three tables age through one loop so they cannot
    /// drift out of scale with each other and invalidate their fitted weights.
    ///
    /// Deliberately NOT given a plausible-looking default. Adding a new eval
    /// term with a guessed weight is how RAR-S13 went wrong; these are 4.10
    /// coordinates, and the fit decides whether either distance carries unique
    /// signal beyond the 1-ply term. If neither does, both stay at 0 and the
    /// tables cost nothing.
    corr_w_cont2 = 0, "CorrWeightCont2", 0..=384;
    /// See `corr_w_cont2`.
    corr_w_cont4 = 0, "CorrWeightCont4", 0..=384;

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
        // 4.3c landed its infrastructure but neither consumer change: RAR-S34's
        // gate did not promote them, so both stay off and 4.4 owns the gate.
        assert_eq!(
            p.singular_reject_speculative, 0,
            "4.3c contract must land inert until 4.4 gates it"
        );
        assert_eq!(
            p.probcut_store_actual_score, 0,
            "4.3c score ablation must land inert (RAR-S34: +5.55% TTD)"
        );
        // 4.4a switches all land inert; the bundle gate flips them together.
        for (name, value) in [
            (
                "NmpSuppressNullInVerification",
                p.nmp_suppress_null_in_verification,
            ),
            ("RfpAllowTtPv", p.rfp_allow_tt_pv),
            ("RazorAllowTtPv", p.razor_allow_tt_pv),
            ("NmpAllowTtPv", p.nmp_allow_tt_pv),
            ("ProbCutAllowTtPv", p.probcut_allow_tt_pv),
            ("NmpRequireCutNode", p.nmp_require_cut_node),
            ("NmpUseStaticEval", p.nmp_use_static_eval),
            ("NmpSingularGuard", p.nmp_singular_guard),
        ] {
            assert_eq!(value, 0, "4.4 switch {name} must land inert");
        }
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
        assert!(p.tt_cutoff_bonus_pct >= 0);
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
