//! Phase 4.1 search diagnostics — compile-time gated counters and sampled traces.
//!
//! Enabled only with `--features diag`. The default build contains **no**
//! counter code at all (`diag_count!` expands to nothing), so `bench` stays a
//! stable fingerprint — the gate for this feature is *bench identical with diag
//! off*. When enabled, counters are process-global atomics (the search may run
//! several worker threads), reset at each `go`, and dumped as `info string diag
//! <name> <value>` lines when the search completes.
//!
//! The legacy event counters remain exact. Phase 4 adds a deterministic 1/1024
//! position sample for the wider interaction map; this bounds diagnostic cost
//! while making repeated runs on the same tree directly comparable. Sampled
//! counters are observational only and may never steer search.

#[cfg(feature = "diag")]
// Counter statics are deliberately lower_snake_case: the name is emitted
// verbatim as the `info string diag <name>` label.
#[expect(non_upper_case_globals)]
pub mod counters {
    use std::sync::atomic::{AtomicU64, Ordering};

    macro_rules! declare {
        ($($name:ident),+ $(,)?) => {
            $( pub static $name: AtomicU64 = AtomicU64::new(0); )+

            /// Zero every counter (called at the start of each search).
            pub fn reset() { $( $name.store(0, Ordering::Relaxed); )+ }

            /// Emit one `info string diag <name> <value>` line per counter.
            pub fn dump() {
                $(
                    crate::info_string!(
                        "diag {} {}",
                        stringify!($name),
                        $name.load(Ordering::Relaxed)
                    );
                )+
            }
        };
    }

    declare!(
        // Denominators. `qnodes` is EXACT, unlike `sampled_qnodes`: the
        // Phase-4 differential needs a qsearch denominator collected the same
        // way the oracle collects it, and a 1/1024 sample cannot be joined
        // against an exact count (analysis/phase4_counter_spec.md).
        nodes,
        qnodes,
        // 8.2 — check-node cost.
        nodes_in_check,
        check_extensions,
        // 8.3 — stale PV bit vetoes pruning at a non-PV node.
        tt_pv_veto,
        // Forward-pruning families (successful cutoffs / skips).
        rfp_cut,
        razor_drop,
        nmp_cut,
        probcut_cut,
        // Per-MOVE: every quiet skipped by move-count/history pruning.
        // Rarog-only -- the oracle cannot count this without generating the
        // quiets it is declining to generate.
        lmp_prune,
        // Per-NODE: nodes at which at least one quiet was suppressed. THIS is
        // the comparable one; see analysis/phase4_counter_spec.md.
        lmp_nodes,
        quiet_futility_prune,
        see_prune,
        // 4.6.4: quiet moves pruned for hanging material. Rarog had no
        // such population before; the capture-side `see_prune` is separate.
        quiet_see_prune,
        // 4.6.5: nodes where the picker was told to stop emitting quiets.
        skip_quiets_nodes,
        // LMR reduction and its verification re-search.
        lmr_applied,
        // 10.2.5 — late moves whose confidence estimate removes the old
        // mandatory one-ply reduction.
        lmr_zero_reduction,
        // 4.8.1 AUDIT of the reduction floor, which 4.5.4 named and never
        // measured. `lmr_reduction` is `(r >> 10).clamp(0, new_depth)`, so
        // both ends silently discard information:
        //   lmr_floor_clamped -- r was NEGATIVE. The formula asked for an
        //     extension and the floor refused it. Every relief term
        //     (tt_pv, improving, corr, and 4.6.7's root relief) is eaten
        //     here once the accumulated r crosses zero.
        //   lmr_qs_clamped -- reduction reached new_depth, so the "reduced
        //     search" ran at depth 0 and was answered by quiescence. That
        //     is a prune wearing a reduction's name, and it is counted
        //     nowhere in the pruning family.
        lmr_floor_clamped,
        lmr_qs_clamped,
        // Root-only reduction census: the denominator 4.6.7 needs to know
        // whether a root relief can move anything at all.
        lmr_root_applied,
        lmr_root_reduction_sum,
        lmr_research,
        // History / correction learning events. `cutoff_quiet + cutoff_capture`
        // is also the count of every beta cutoff at a real (non-excluded)
        // interior node, i.e. the DENOMINATOR of the ordering metric below.
        cutoff_quiet,
        cutoff_capture,
        // 10.0(a) — FIRST-MOVE CUTOFF RATE, the standard move-ordering readout:
        // `cutoff_first_move / (cutoff_quiet + cutoff_capture)`. Counted where
        // the move that failed high was the FIRST move the node searched.
        //
        // Why it is the missing metric: 10.0 established that Rarog's eval and
        // NPS match Basilisk 1.9.1 while it plays ~38-55 Elo weaker at 1T, at
        // any time control, so the deficit is in how the search converts nodes
        // into decisions. Two sub-causes remain, and they imply opposite fixes.
        // The over-reduction ratio (`lmr_research / lmr_applied`) reads the
        // PRUNING-DEPTH side; this counter reads the ORDERING side. Healthy
        // engines sit ~90%+; materially below implicates ordering, in which
        // case re-tuning the selectivity surface (10.4.6) is aimed at the wrong
        // half of the problem.
        //
        // Excluded-move (singular-verification) searches do NOT count: their
        // best move is deliberately withheld, so a first-move cutoff there
        // measures the exclusion, not the ordering. That is automatic — this
        // sits inside the same `excluded.is_null()` guard as the two above, so
        // numerator and denominator always cover the same node set.
        cutoff_first_move,
        correction_updates,
        correction_on_capture,
        // 4.5 — residual MAGNITUDE by attribution class, exact.
        //
        // The premise behind capture-weighted correction updates is that a
        // capture-caused residual is less trustworthy evidence for a
        // positional correction. Nobody has measured that. These give the mean
        // |residual| for each class; if the two means are close, the premise is
        // wrong and neither knob should move off its baseline.
        // 4.5c — nodes where the margin/reduction knobs are widened by a
        // correction magnitude that is NOT in the eval they test, because a TT
        // bound replaced the corrected eval. Exact. This sizes the mismatch
        // `CorrSkipWhenTtRefined` exists to remove.
        corr_applied_to_replaced_eval,
        correction_resid_capture_n,
        correction_resid_capture_sum,
        correction_resid_quiet_n,
        correction_resid_quiet_sum,
        // 4.6c — safe versus losing quiet checks in the ordering path.
        //
        // `CheckBonusLosing` measured 0.00% node change even at 0, which is
        // indistinguishable from dead code. These decide which it is: a zero
        // `losing` count means the population is genuinely empty, while a
        // non-zero one means the switch is wired but its effect is masked.
        // Counted unconditionally, so the answer does not depend on the
        // switch being enabled.
        check_order_safe,
        check_order_losing,
        // 4.5d — residual by HALFMOVE-CLOCK context. PLAN 4.5 allows a new
        // correction context only where held-out unique signal is shown, so the
        // measurement comes before any proposal. Rule-50 proximity is the
        // plausible mechanism: near the horizon a position's value stops being a
        // function of its structure.
        //
        // The check/evasion context the plan also lists is NOT measured, because
        // it is structurally unreachable: correction only trains where
        // `static_eval != VALUE_NONE`, which IS the not-in-check condition, so
        // its population is zero by construction rather than by observation.
        correction_resid_hm_low_n,
        correction_resid_hm_low_sum,
        correction_resid_hm_mid_n,
        correction_resid_hm_mid_sum,
        correction_resid_hm_high_n,
        correction_resid_hm_high_sum,
        // 9.7.5(b) — SMP quality. The question these answer: 16 threads give
        // 13x the nodes but +0 depth and +2 seldepth, so where does the work
        // go? Four hypotheses imply opposite fixes, hence measure first.
        //
        // Aspiration churn. 8.13 made a thread re-centre its window on the
        // POOL's deepest Exact score; if the pool disagrees with what the
        // thread then finds, it pays fail-high/low re-searches. A re-search
        // rate that climbs with thread count indicts pool-seeded windows.
        asp_fail_high,
        asp_fail_low,
        // TT store duplication. `same_key` means the slot already held THIS
        // position; `fresh` means it did not. If threads are re-deriving each
        // other's work, the same_key share rises with thread count. Counted on
        // both backends so 1T (local) and NT (shared) are comparable.
        tt_store_same_key,
        tt_store_fresh,
        // 4.2 — EXACT producer census, keyed by the `OutcomeKind` the store site
        // declares. The 4.1 producer counters are sampled and sit at the call
        // sites; these are unsampled and sit in the store path, so they both
        // cross-check the sampler's producer mix and catch a store site that
        // stops being reached at all. `Null`/`Incomplete` have no counter
        // because no path stores them — `debug_assert_outcome` fires instead.
        store_kind_full,
        store_kind_verified_reduced,
        store_kind_qsearch_move,
        store_kind_qsearch_tail,
        store_kind_stand_pat,
        store_kind_probcut,
        store_kind_tablebase,
        // 4.3 — provenance HAZARDS in the store path, both exact.
        //
        // `tt_move_inherited` counts moveless stores that adopted the resident
        // move; the `_stand_pat` subset is the one that matters, because it
        // turns a static estimate into an entry indistinguishable from a
        // searched qmove. If that subset is large, "depth 0 + Lower + no move"
        // is NOT a usable stand-pat test and 4.3 cannot lean on it.
        //
        // `tt_horizon_overwrote_searched` counts depth-0 stores that replaced a
        // deeper same-position entry, which the depth-preservation rule only
        // blocks beyond 3 plies.
        tt_move_inherited,
        tt_move_inherited_stand_pat,
        tt_horizon_overwrote_searched,
        // 4.3 — ATTEMPTED versus COMMITTED stores.
        //
        // The `store_kind_*` census above runs before the backend dispatch, so
        // it counts ATTEMPTS and reconciles with `fresh + same_key`. The hazard
        // counters run after the depth-preservation `return`, so they count
        // COMMITTED stores. Dividing one by the other mismatches denominators
        // and understates every hazard rate, which is exactly the error the
        // first RAR-S25 figures carried. These give the matched denominators.
        //
        // A store is skipped when it lands on a same-position entry more than 3
        // plies deeper, is not exact, and is the current generation — so horizon
        // producers are by far the likeliest to be skipped.
        store_skipped_depth_rule,
        store_committed_stand_pat,
        store_committed_qsearch_move,
        store_committed_horizon,
        // Does helper work actually REACH the main thread? Probe/hit counted
        // on thread 0 only. If helpers contribute, main's hit rate should rise
        // with thread count; if it is flat, the helpers are searching in vain.
        main_tt_probes,
        main_tt_hits,
        // 9.6(b) — lazy-eval safety audit. On every lazy skip the full eval is
        // ALSO computed (served score unchanged) and the two are compared.
        // `lazy_delta_sum / lazy_fires` = mean |full − cheap| in internal cp;
        // `lazy_delta_max` is a running maximum (fetch_max, not fetch_add).
        lazy_fires,
        lazy_delta_sum,
        lazy_delta_max,
        // The cheap score exceeded LazyMargin by construction; a sign flip
        // means the full eval DISAGREES ABOUT WHO IS BETTER — the failure
        // lazy eval promises cannot happen. A margin crossing is the softer
        // event: |full| <= LazyMargin, i.e. the position was not actually
        // decided. Both bucketed by the max king-danger index seen in the
        // full pass (low 0-9 / mid 10-19 / high 20-29 / extreme 30+) and by
        // game-phase quartile (q1 = endgame .. q4 = middlegame) as the
        // material signature.
        lazy_sign_flips,
        lazy_margin_crossings,
        lazy_flip_danger_low,
        lazy_flip_danger_mid,
        lazy_flip_danger_high,
        lazy_flip_danger_extreme,
        lazy_flip_phase_q1,
        lazy_flip_phase_q2,
        lazy_flip_phase_q3,
        lazy_flip_phase_q4,
        lazy_cross_danger_low,
        lazy_cross_danger_mid,
        lazy_cross_danger_high,
        lazy_cross_danger_extreme,
        lazy_cross_phase_q1,
        lazy_cross_phase_q2,
        lazy_cross_phase_q3,
        lazy_cross_phase_q4,
        // 4.1 sampled node/TT provenance and contradiction map.
        sampled_main_nodes,
        sampled_qnodes,
        tt_sample_hit,
        tt_sample_miss,
        tt_cut_exact,
        tt_cut_lower,
        tt_cut_upper,
        // Deep enough at an eligible node, but the stored bound resolves some
        // OTHER window — a store/window question.
        tt_bound_not_usable,
        // 4.9b — why an entry that HIT could not be used, split by cause,
        // because the three imply different fixes and only one of them can
        // grow with thread count.
        //
        // `_pv` and `_excluded` are structural: those nodes refuse the entry
        // however deep it is, and their populations do not depend on how many
        // threads are searching. `_shallow` is the one the "helpers add entries
        // that cannot cut" hypothesis predicts should rise with threads, and
        // `_deficit` sums how many plies short each one was, so a marginal miss
        // (a replacement-policy question) is distinguishable from a hopeless
        // one (not worth chasing).
        tt_reject_pv,
        tt_reject_excluded,
        tt_reject_shallow,
        tt_reject_shallow_deficit,
        tt_bound_contradicts_window,
        tt_eval_refined,
        tt_eval_delta_sum,
        main_store_lower,
        main_store_exact,
        main_store_upper,
        // qsearch authority: distinguish unsearched stand pat from searched moves.
        q_in_check,
        q_tt_hit,
        q_tt_cut,
        q_stand_pat_cut,
        q_stand_pat_store,
        q_move_cut,
        q_move_store,
        q_tail_exact_store,
        q_tail_upper_store,
        // 4.9d — SIZING the in-check qsearch staging that 4.6 deferred here.
        //
        // An in-check qnode generates every evasion and scores ALL of them
        // before picking any, so a node that cuts on its first move paid for
        // the rest. Staging would emit the TT move before scoring anything —
        // order-identical, since `score_moves` already gives it a dominating
        // score — but it is only worth building if the scoring is genuinely
        // wasted. `scored - tried` is exactly that waste, and these are EXACT
        // (in-check qnodes are a small population; sampling them would add
        // noise to the one number that decides whether to build it).
        q_check_nodes,
        q_check_moves_scored,
        q_check_moves_tried,
        // NMP/ProbCut/singular/IIR cooperation.
        nmp_attempt,
        nmp_sample_cut,
        nmp_nested_attempt,
        nmp_eval_raw,
        nmp_eval_corrected,
        nmp_eval_tt,
        nmp_verify_attempt,
        nmp_verify_pass,
        nmp_verify_fail,
        // 4.10a — two DIFFERENT questions the decisive guard has been conflated
        // with, counted apart because they imply opposite decisions.
        //
        // `nmp_decisive_population` is an NMP-eligible node whose WINDOW is
        // already at mate range. This was `NmpDecisiveGuard`'s predicate; 4.10a
        // removed that switch (efficiency only, 0.004% of nodes) and kept the
        // count, because it is the context for the question below.
        //
        // `nmp_cut_unproven_mate` is the correctness question, and it is NOT
        // the same condition: an NMP cutoff whose RETURNED SCORE is mate-range,
        // i.e. a mate this node never proved by a real line. Rarog returns the
        // raw null score with no clamp, where Stockfish forces it back to beta
        // precisely so unproven mates cannot propagate. A non-zero count here
        // is a defect the guard does not fix.
        nmp_decisive_population,
        nmp_cut_unproven_mate,
        // Per-NODE: nodes passing the ProbCut entry gate, counted before
        // capture generation, so nodes with no eligible capture are included.
        // Per-MOVE: `probcut_attempt` -- a ProbCut search was actually started,
        // which is what the spec says and what the oracle counts. Until 4.7c
        // prep the per-node figure carried the `probcut_attempt` name and was
        // differenced against the oracle's per-move one; see the RAR-S55
        // correction in EXPERIMENTS.md.
        probcut_nodes,
        probcut_attempt,
        probcut_qpass,
        probcut_tt_store,
        singular_attempt,
        singular_probcut_depth_match,
        singular_speculative_seed_blocked,
        singular_extend_one,
        singular_extend_two,
        singular_multicut,
        singular_negative_extension,
        iir_applied,
        iir_pv,
        iir_no_tt_move,
        iir_shallow_tt,
        iir_extension_debt,
        // Move-stage recall and pruning overlap. Counts cover sampled nodes only.
        move_seen_tt,
        move_seen_good_capture,
        move_seen_quiet,
        move_seen_bad_capture,
        // Rarog-only: rank of the BEST MOVE at any node, including PV nodes
        // where it merely raised alpha. Strictly larger population than a beta
        // cutoff, so it must never be differenced against the oracle's
        // cutoff-rank counters -- see analysis/phase4_counter_spec.md.
        best_move_rank_1,
        best_move_rank_2_3,
        best_move_rank_4_7,
        best_move_rank_8_plus,
        // Core (comparable): rank at which a beta cutoff occurred. Exact, and
        // counted in the same block as cutoff_quiet/cutoff_capture so the
        // buckets sum to that denominator.
        best_rank_1,
        best_rank_2_3,
        best_rank_4_7,
        best_rank_8_plus,
        best_stage_tt,
        best_stage_good_capture,
        best_stage_quiet,
        best_stage_bad_capture,
        best_was_reduced,
        prune_shadow_moves,
        prune_shadow_lmp,
        prune_shadow_futility,
        prune_shadow_see,
        prune_shadow_check_exempt,
        prune_shadow_overlap_two_plus,
        prospective_depth_sum,
        reduction_depth_sum,
        // Correction attribution and hashed-table quality.
        correction_sample_updates,
        correction_sample_abs_sum,
        correction_slot_first,
        correction_slot_repeat,
        correction_slot_collision,
        correction_slot_near_saturation,
        // Root confidence/SMP observations use fixed-point sums (ppm/cp).
        root_iterations,
        root_gap_sum,
        root_deviation_sum,
        root_effort_ppm_sum,
        root_best_changes,
        root_interrupted_fallback,
        // 4.7b — the completed-iteration confidence model, measured before any
        // consumer is switched on.
        //
        // The scalar histogram answers the first question a confidence model
        // has to answer: does it DISCRIMINATE? A model that reads ~500 on every
        // iteration is a constant wearing a model's clothes, and multiplying
        // the clock by a constant is a re-scale of `TmOptScale`, not a new
        // mechanism.
        rootconf_scalar_sum,
        rootconf_scalar_q1,
        rootconf_scalar_q2,
        rootconf_scalar_q3,
        rootconf_scalar_q4,
        // Input distributions. The deviation scale is seeded at its measured
        // half-confidence point instead of a round number that happens to
        // saturate its term; gap remains diagnostic-only.
        // The gap gets a ZERO bucket of its own, because the separation term
        // stands or falls on it: every root move but the best is searched on a
        // null window, so a move that fails low reports a bound just under
        // alpha rather than a value. If the gap is mostly exactly 0, the term
        // is measuring PVS bookkeeping and not separation.
        // Splits the zero bucket by CAUSE. "No rival" means no other root move
        // was searched to this depth at all, so the gap is zero by absence of
        // evidence; the remainder is a rival that genuinely scored level.
        rootconf_gap_no_rival,
        rootconf_gap_0,
        rootconf_gap_1_7,
        rootconf_gap_8_127,
        rootconf_gap_128_plus,
        rootconf_dev_lt_8,
        rootconf_dev_8_31,
        rootconf_dev_32_127,
        rootconf_dev_128_plus,
        // Is the term the confidence factor REPLACES even live? The baseline
        // effort factor interpolates over `0.79..=1.0` of the iteration spent
        // on the best move, and reads its endpoint for everything below 0.79.
        // This counts the iterations that actually land inside the band — i.e.
        // the ones where the shipped effort factor is a function rather than a
        // constant.
        rootconf_effort_term_live,
        // Iterations that took at least one aspiration re-search: the WINDOW
        // term's population.
        rootconf_window_fail_iters,
        // The redundancy check behind leaving best-move AGE out of the scalar.
        // If age and instability track each other, charging both would price
        // one fact twice; these two sums are what makes that claim falsifiable
        // rather than asserted.
        rootconf_best_age_sum,
        rootconf_instab_milli_sum,
        rootconf_pool_instab_milli_sum,
        // PV truncation: the population a PV term would have. 4.5d's lesson —
        // a term needs a distinct signal AND a population, so measure the
        // population before adding the term.
        rootconf_pv_truncated,
        // 4.7b TM SHADOW — the two clock multipliers side by side, in
        // ten-thousandths. `RootConfTime` cannot be sized by `bench` (which is
        // depth-limited, so the soft target never binds), but the multiplier it
        // would apply is computed from data `bench` does produce. Sum ratio =
        // does the arm move the total budget or only its distribution;
        // longer/shorter = how often it disagrees at all.
        rootconf_tm_baseline_sum,
        rootconf_tm_candidate_sum,
        rootconf_tm_longer,
        rootconf_tm_shorter,
        worker_best_disagreement,
        worker_depth_spread_sum,
        worker_score_spread_sum,
        // 4.2b SHADOW TEST — inexact bounds that CONTRADICT the current window.
        //
        // A `Lower` at or below alpha, or an `Upper` at or above beta, resolved
        // some OTHER window and says nothing about this one. It cannot produce a
        // cutoff (proved by a unit test in `evidence.rs`), but every consumer
        // that does not test the bound direction still admits it at full
        // nominal depth. The registered question is whether it should carry a
        // confidence/depth penalty. These counters measure what a penalty WOULD
        // change; no consumer branches on any of them.
        //
        // `contradict_hits` is UNGATED, unlike `tt_bound_contradicts_window`
        // above, which only counts the cutoff-eligible subset (deep enough, at a
        // non-PV non-excluded node). The consumers below have their own, looser
        // depth rules, so the gated figure understates their exposure.
        contradict_hits,
        // eval_for_pruning: the highest-volume consumer. `slack` is
        // `ev.depth` relative to the accepted floor of zero. A hypothetical
        // penalty of P plies blocks exactly the cases with slack < P — one
        // histogram answers every P.
        contradict_refined_eval,
        contradict_refine_slack_0,
        contradict_refine_slack_1,
        contradict_refine_slack_2_3,
        contradict_refine_slack_4_7,
        contradict_refine_slack_8_plus,
        contradict_refine_delta_sum,
        // Singular seeds its verification window from this stored score.
        contradict_singular_attempt,
        contradict_singular_changed_depth,
        // The multi-cut arm RETURNS, so it cannot be counted alongside the
        // extension outcomes above; it needs its own counter at its own site.
        contradict_singular_multicut,
        // A DEEP contradicting entry suppresses IIR, i.e. it is trusted to
        // order the node even though it resolved a different window.
        contradict_iir_suppressed,
        // Control pair. If a contradicting entry's move is best about as often
        // as an agreeing one's, the penalty belongs on the SCORE consumers only
        // and must not touch ordering or IIR. This is the measurement that
        // decides the shape of the 4.3 change, so it has its own denominator.
        contradict_move_present,
        contradict_move_was_best,
        agree_move_present,
        agree_move_was_best,
        // 4.4a SIZING — how much work does each candidate switch actually reach?
        //
        // All exact, all measured with the switches OFF, so they size the arms
        // BEFORE the bundle is chosen rather than after a gate has been spent.
        // `tt_pv_veto` above is the shared denominator: the nodes where one
        // inherited PV bit currently blocks all four mechanisms at once. These
        // count, per mechanism, how many of those vetoed nodes would additionally
        // satisfy that mechanism's own depth precondition — i.e. the population
        // its `*AllowTtPv` switch would hand back.
        tt_pv_veto_rfp_eligible,
        tt_pv_veto_razor_eligible,
        tt_pv_veto_nmp_eligible,
        tt_pv_veto_probcut_eligible,
        // 4.3 SHADOW — is TT eval refinement SELF-CANCELLING?
        //
        // Two depth-floor arms (1 and 2) both measured ~0 Elo while
        // moving 15-44% of the tree, which has two very different explanations:
        // the margins absorb it (lesson 2), or the refinement helps as often as
        // it hurts. Those imply opposite fixes, so measure rather than guess.
        //
        // PART 1 - decision flips, at the pruning site. For each consumer, does
        // the predicate evaluated on `eval_for_pruning` differ from the same
        // predicate on `static_eval`? `_on` = refinement CAUSED the prune,
        // `_off` = refinement PREVENTED one static would have taken. Roughly
        // balanced on/off is the precise form of "self-cancelling", and this
        // half is unbiased: it is recorded before any of the three can return.
        refine_flip_nodes,
        refine_flip_rfp_on,
        refine_flip_rfp_off,
        refine_flip_razor_on,
        refine_flip_razor_off,
        refine_flip_nmp_on,
        refine_flip_nmp_off,
        // PART 2 - did refinement move the eval TOWARD the value the node went
        // on to report? Recorded at the node tail.
        //
        // ⚠ BIASED, and knowably so: a node that pruned never reaches the tail,
        // so the cases where refinement mattered MOST are exactly the ones
        // missing. Part 1 sizes that excluded population. The comparison is also
        // against what the node REPORTED, not against truth — on a fail-low the
        // reported score is an upper bound, not a value. Read it as "did
        // refinement agree with the search's own conclusion", nothing stronger.
        refine_report_nodes,
        refine_report_closer,
        refine_report_farther,
        refine_report_gain_sum,
        refine_report_loss_sum,
        // Coverage proof for the shadow consumers planned in 4.2--4.7.
        shadow_4_2_evidence,
        shadow_4_3_qsearch,
        shadow_4_4_selectivity,
        shadow_4_5_correction,
        shadow_4_6_prospective_depth,
        shadow_4_7_root_confidence,
        // 4.9a search-tree occurrence. RAR-M15 measured how often each
        // reference endgame family occurs ON THE BOARD in real games, and 4.9a
        // is ordered on that. These count how often each family is reached in
        // the SEARCH TREE, which is a different quantity: 4.9a.4's mate drive
        // left `bench 13` byte-identical (no bench tree reaches a bare-king
        // minor mate at depth 13) while 4.9a.7's rook-ending scale moved it
        // 14%. EXACT, not sampled -- these are cheap and a ratio against
        // `nodes` must be joinable (analysis/phase4_counter_spec.md).
        eg_krpkr,
        eg_krpkb,
        eg_kpsk,
        eg_kpk,
        eg_krkp,
        eg_kbpsk,
        eg_kpkp,
        eg_kqkp,
        eg_kbpkb,
        eg_kbppkb,
        eg_krkn,
        eg_krkb,
        eg_kbpkn,
        eg_knnkp,
        eg_knnk,
        eg_kqkr,
        eg_kqkrps,
        eg_krppkrp,
        eg_kxk,
        eg_kbnk,
        // Denominator: evaluations of positions with at most 7 men, the set
        // the classifier looks at. Without it an occurrence count cannot be
        // read as a rate.
        eg_classified,
        // 4.11b.7 board cost census. These are exact call/work counters from
        // real search paths. They observe only and compile away completely
        // without `--features diag`.
        board_gen_vec_calls,
        board_gen_vec_moves,
        board_gen_full_calls,
        board_gen_full_moves,
        board_gen_capture_calls,
        board_gen_capture_moves,
        board_gen_staged_capture_calls,
        board_gen_staged_capture_moves,
        board_gen_staged_quiet_calls,
        board_gen_staged_quiet_moves,
        board_compute_pinned_calls,
        board_check_info_calls,
        board_gives_check_fast_calls,
        board_gives_check_full_calls,
        board_calculate_checkers_calls,
        board_see_full_calls,
        board_see_threshold_calls,
        board_see_quiet_threshold_calls,
        board_make_plain_calls,
        board_make_with_check_calls,
        board_unmake_calls,
        board_make_null_calls,
        board_unmake_null_calls,
        board_history_pushes,
        board_history_growths,
    );
}

/// Count one evaluation against the 4.9a reference-family table.
///
/// Called from `evaluate()` and compiled out entirely without `--features
/// diag`, so the production fingerprint is untouched -- which is this feature's
/// own acceptance gate.
///
/// The table is the 20 families of the final pre-NNUE Stockfish dispatcher, in
/// the order 4.9a works them. `counts` is (pawns, knights, bishops, rooks,
/// queens) per side; each family is tried in both orientations, so a Black
/// strong side counts the same as a White one.
#[cfg(feature = "diag")]
pub fn record_endgame_family(w: [u32; 5], b: [u32; 5]) {
    // At most 7 men total (2 kings + 5 pieces) can match any listed family.
    let men: u32 = w.iter().sum::<u32>() + b.iter().sum::<u32>();
    if men > 5 {
        return;
    }
    crate::diag_count!(eg_classified);

    // Each arm is (strong, weak) as (P, N, B, R, Q). `None` in a slot means
    // "one or more", used by the families whose reference name carries `Ps`.
    let hit = |s: [u32; 5], k: [Option<u32>; 5]| -> bool {
        (0..5).all(|i| match k[i] {
            Some(n) => s[i] == n,
            None => s[i] >= 1,
        })
    };
    let e = |n: u32| Some(n);
    let bare = [e(0), e(0), e(0), e(0), e(0)];

    for (i, (strong, weak)) in [(w, b), (b, w)].into_iter().enumerate() {
        // A SYMMETRIC family matches in both orientations and would be counted
        // twice. KPKP is the only one in this table, and it read 370 instead of
        // 185 before this guard.
        if i == 1 && w == b {
            break;
        }
        // Order matters only for readability; the arms are disjoint.
        if hit(strong, [e(1), e(0), e(0), e(1), e(0)]) && hit(weak, [e(0), e(0), e(0), e(1), e(0)])
        {
            crate::diag_count!(eg_krpkr);
        } else if hit(strong, [e(1), e(0), e(0), e(1), e(0)])
            && hit(weak, [e(0), e(0), e(1), e(0), e(0)])
        {
            crate::diag_count!(eg_krpkb);
        } else if hit(strong, [e(2), e(0), e(0), e(1), e(0)])
            && hit(weak, [e(1), e(0), e(0), e(1), e(0)])
        {
            crate::diag_count!(eg_krppkrp);
        } else if hit(strong, [e(1), e(0), e(0), e(0), e(0)]) && hit(weak, bare) {
            crate::diag_count!(eg_kpk);
            crate::diag_count!(eg_kpsk);
        } else if hit(strong, [None, e(0), e(0), e(0), e(0)]) && hit(weak, bare) {
            crate::diag_count!(eg_kpsk);
        } else if hit(strong, [e(0), e(0), e(0), e(1), e(0)])
            && hit(weak, [e(1), e(0), e(0), e(0), e(0)])
        {
            crate::diag_count!(eg_krkp);
        } else if hit(strong, [e(1), e(0), e(0), e(0), e(0)])
            && hit(weak, [e(1), e(0), e(0), e(0), e(0)])
        {
            crate::diag_count!(eg_kpkp);
        } else if hit(strong, [None, e(0), e(1), e(0), e(0)]) && hit(weak, bare) {
            crate::diag_count!(eg_kbpsk);
        } else if hit(strong, [e(0), e(0), e(0), e(0), e(1)])
            && hit(weak, [e(1), e(0), e(0), e(0), e(0)])
        {
            crate::diag_count!(eg_kqkp);
        } else if hit(strong, [e(2), e(0), e(1), e(0), e(0)])
            && hit(weak, [e(0), e(0), e(1), e(0), e(0)])
        {
            crate::diag_count!(eg_kbppkb);
        } else if hit(strong, [e(1), e(0), e(1), e(0), e(0)])
            && hit(weak, [e(0), e(0), e(1), e(0), e(0)])
        {
            crate::diag_count!(eg_kbpkb);
        } else if hit(strong, [e(1), e(0), e(1), e(0), e(0)])
            && hit(weak, [e(0), e(1), e(0), e(0), e(0)])
        {
            crate::diag_count!(eg_kbpkn);
        } else if hit(strong, [e(0), e(0), e(0), e(1), e(0)])
            && hit(weak, [e(0), e(1), e(0), e(0), e(0)])
        {
            crate::diag_count!(eg_krkn);
        } else if hit(strong, [e(0), e(0), e(0), e(1), e(0)])
            && hit(weak, [e(0), e(0), e(1), e(0), e(0)])
        {
            crate::diag_count!(eg_krkb);
        } else if hit(strong, [e(0), e(2), e(0), e(0), e(0)])
            && hit(weak, [e(1), e(0), e(0), e(0), e(0)])
        {
            crate::diag_count!(eg_knnkp);
        } else if hit(strong, [e(0), e(2), e(0), e(0), e(0)]) && hit(weak, bare) {
            crate::diag_count!(eg_knnk);
        } else if hit(strong, [e(0), e(0), e(0), e(0), e(1)])
            && hit(weak, [e(0), e(0), e(0), e(1), e(0)])
        {
            crate::diag_count!(eg_kqkr);
        } else if hit(strong, [e(0), e(0), e(0), e(0), e(1)])
            && hit(weak, [None, e(0), e(0), e(1), e(0)])
        {
            crate::diag_count!(eg_kqkrps);
        } else if hit(strong, [e(0), e(1), e(1), e(0), e(0)]) && hit(weak, bare) {
            crate::diag_count!(eg_kbnk);
            crate::diag_count!(eg_kxk);
        } else if hit(weak, bare) && strong.iter().sum::<u32>() == 1 && strong[0] == 0 {
            // KXK: one non-pawn piece against a bare king.
            crate::diag_count!(eg_kxk);
        }
    }
}

/// Stable domains keep independent samples from accidentally selecting exactly
/// the same positions. Public constants make call sites self-documenting.
#[cfg(feature = "diag")]
pub const SAMPLE_MAIN: u64 = 0x4D41_494E_5F34_2E31;
#[cfg(feature = "diag")]
pub const SAMPLE_QSEARCH: u64 = 0x5153_4541_5243_4831;
#[cfg(feature = "diag")]
pub const SAMPLE_CORRECTION: u64 = 0x434F_5252_5F34_2E31;

/// Sampling stride mask, read once from `RAROG_DIAG_SAMPLE_STRIDE`.
///
/// Phase 4.2: the differential suite needs the CORE counters exact, because the
/// oracle collects them exactly and a 1/1024 sample cannot be joined against an
/// exact count — the ratio reads 1024x off while looking plausible. Rather than
/// lift seventeen counters out of their sampling guards in the hottest file in
/// the engine, the stride itself is configurable, so `RAROG_DIAG_SAMPLE_STRIDE=1`
/// makes every sampled counter exact in one place.
///
/// The stride must be a power of two; anything else falls back to the 1024
/// default, which keeps every historical reading (RAR-S21/S22/S24) reproducible
/// by simply not setting the variable.
#[cfg(feature = "diag")]
fn sample_mask() -> u64 {
    use std::sync::OnceLock;
    static MASK: OnceLock<u64> = OnceLock::new();
    *MASK.get_or_init(|| {
        std::env::var("RAROG_DIAG_SAMPLE_STRIDE")
            .ok()
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .filter(|stride| *stride >= 1 && stride.is_power_of_two())
            .map_or(1023, |stride| stride - 1)
    })
}

/// Deterministic position sampler, 1/1024 by default. It is deliberately
/// available only in diagnostic builds: production code must contain neither
/// the mix nor a branch.
///
/// With a stride of 1 the mask is 0, so the test is always true and every
/// position is sampled — the exact mode the Phase-4 differential requires.
#[cfg(feature = "diag")]
#[inline]
pub fn sampled(hash: u64, ply: usize, domain: u64) -> bool {
    let mut value = hash ^ domain ^ (ply as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((value ^ (value >> 31)) & sample_mask()) == 0
}

/// Diagnostic-only ownership tags for the deliberately lossy correction
/// tables. A repeated slot/key is normal reuse; a different key in the same
/// slot is an observed collision. The map is sparse because callers invoke it
/// only for sampled updates.
#[cfg(feature = "diag")]
mod correction_probe {
    use std::collections::HashMap;
    use std::sync::atomic::Ordering;
    use std::sync::{Mutex, OnceLock};

    static OWNERS: OnceLock<Mutex<HashMap<(u8, usize), u64>>> = OnceLock::new();

    fn owners() -> &'static Mutex<HashMap<(u8, usize), u64>> {
        OWNERS.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub fn reset() {
        owners()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    pub fn record(source: u8, index: usize, key: u64, value: i16) {
        use crate::diag::counters;
        let mut owners = owners()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match owners.insert((source, index), key) {
            None => counters::correction_slot_first.fetch_add(1, Ordering::Relaxed),
            Some(old) if old == key => {
                counters::correction_slot_repeat.fetch_add(1, Ordering::Relaxed)
            }
            Some(_) => counters::correction_slot_collision.fetch_add(1, Ordering::Relaxed),
        };
        if value.unsigned_abs() >= 15_000 {
            counters::correction_slot_near_saturation.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(feature = "diag")]
#[inline]
pub fn record_correction_slot(source: u8, index: usize, key: u64, value: i16) {
    correction_probe::record(source, index, key, value);
}

#[cfg(feature = "diag")]
pub fn record_best_move(rank: usize, stage: crate::evidence::MoveClass, reduced: bool) {
    use crate::evidence::MoveClass;
    use std::sync::atomic::Ordering;

    let rank_counter = match rank {
        1 => &counters::best_move_rank_1,
        2 | 3 => &counters::best_move_rank_2_3,
        4..=7 => &counters::best_move_rank_4_7,
        _ => &counters::best_move_rank_8_plus,
    };
    rank_counter.fetch_add(1, Ordering::Relaxed);
    // 4.2: takes `MoveClass` rather than a 0..3 integer, so the picker's stage
    // taxonomy is defined in exactly one place.
    let stage_counter = match stage {
        MoveClass::TtMove => &counters::best_stage_tt,
        MoveClass::GoodCapture => &counters::best_stage_good_capture,
        MoveClass::Quiet => &counters::best_stage_quiet,
        MoveClass::BadCapture => &counters::best_stage_bad_capture,
    };
    stage_counter.fetch_add(1, Ordering::Relaxed);
    if reduced {
        counters::best_was_reduced.fetch_add(1, Ordering::Relaxed);
    }
}

/// 4.2b: record how a contradicting entry's MOVE fared for ordering, against
/// the agreeing-entry control.
///
/// Called from both node exits so the numerator and denominator always cover the
/// same node set — the same trap `cutoff_first_move` documents. `hit` without
/// `contradicts` is the control group and includes exact bounds.
#[cfg(feature = "diag")]
#[inline]
pub fn record_contradiction_ordering(contradicts: bool, hit: bool, best_was_tt_move: bool) {
    use std::sync::atomic::Ordering;

    if contradicts {
        counters::contradict_move_present.fetch_add(1, Ordering::Relaxed);
        if best_was_tt_move {
            counters::contradict_move_was_best.fetch_add(1, Ordering::Relaxed);
        }
    } else if hit {
        counters::agree_move_present.fetch_add(1, Ordering::Relaxed);
        if best_was_tt_move {
            counters::agree_move_was_best.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// 4.3 shadow, part 2: did the refined eval sit closer than the plain static
/// eval to the score this node went on to report?
///
/// `gain`/`loss` accumulate the centipawn improvement or worsening so a small
/// number of large disagreements cannot hide behind a majority of tiny ones —
/// the count alone would call that self-cancelling when it is not.
#[cfg(feature = "diag")]
#[inline]
pub fn record_refine_agreement(static_eval: i32, refined: i32, reported: i32) {
    use std::sync::atomic::Ordering;

    counters::refine_report_nodes.fetch_add(1, Ordering::Relaxed);
    let plain_err = i64::from(static_eval - reported).abs();
    let refined_err = i64::from(refined - reported).abs();
    if refined_err < plain_err {
        counters::refine_report_closer.fetch_add(1, Ordering::Relaxed);
        let gain = u64::try_from(plain_err - refined_err).unwrap_or(0);
        counters::refine_report_gain_sum.fetch_add(gain, Ordering::Relaxed);
    } else if refined_err > plain_err {
        counters::refine_report_farther.fetch_add(1, Ordering::Relaxed);
        let loss = u64::try_from(refined_err - plain_err).unwrap_or(0);
        counters::refine_report_loss_sum.fetch_add(loss, Ordering::Relaxed);
    }
}

/// 4.2b: bucket the depth slack a contradicting entry had when it refined
/// `eval_for_pruning`. A penalty of P plies blocks every case with slack < P.
#[cfg(feature = "diag")]
#[inline]
pub fn record_contradiction_refine(slack: i32, delta: u64) {
    use std::sync::atomic::Ordering;

    counters::contradict_refined_eval.fetch_add(1, Ordering::Relaxed);
    counters::contradict_refine_delta_sum.fetch_add(delta, Ordering::Relaxed);
    let bucket = match slack {
        i32::MIN..=0 => &counters::contradict_refine_slack_0,
        1 => &counters::contradict_refine_slack_1,
        2..=3 => &counters::contradict_refine_slack_2_3,
        4..=7 => &counters::contradict_refine_slack_4_7,
        _ => &counters::contradict_refine_slack_8_plus,
    };
    bucket.fetch_add(1, Ordering::Relaxed);
}

/// One completed root iteration, as the 4.7b diagnostics see it.
///
/// A struct rather than nine positional scalars: the recorder takes every field
/// of the same snapshot, and a positional list of `i32`/`f64`/`bool` is exactly
/// the kind of call site that silently transposes two arguments.
#[cfg(feature = "diag")]
pub struct RootConfidenceShadow {
    pub gap: i32,
    pub deviation: i32,
    pub effort: f64,
    pub best_changed: bool,
    /// True when no other root move reached this depth, so `gap` is zero for
    /// want of a rival rather than for want of separation.
    pub no_rival: bool,
    pub scalar: i32,
    pub instability: f64,
    pub pooled_instability: Option<u64>,
    pub best_age: usize,
    pub pv_len: usize,
    pub fails: i32,
    /// Clock multiplier the shipped formula applies.
    pub baseline_time: f64,
    /// Clock multiplier `RootConfTime = 1` would apply instead.
    pub candidate_time: f64,
}

/// Root statistics are cold and diagnostic-only. Floating-point conversion is
/// intentionally lossy because these are aggregate telemetry units, not search
/// inputs (effort in ppm, deviation in cp, multipliers in ten-thousandths).
#[cfg(feature = "diag")]
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn record_root_confidence(shadow: &RootConfidenceShadow) {
    use std::sync::atomic::Ordering;

    let add = |counter: &std::sync::atomic::AtomicU64, value: u64| {
        counter.fetch_add(value, Ordering::Relaxed);
    };
    // Octave buckets. The boundaries are where they are so the MEDIAN of each
    // input is identifiable: a scale coordinate seeded off a mean would be
    // dragged by the decisive tail (a mate-range gap is hundreds of cp), and
    // one seeded off a round number would sit wherever the round number sits.
    let bucket = |value: i32, counters: [&std::sync::atomic::AtomicU64; 4]| {
        let slot = match value {
            i32::MIN..8 => counters[0],
            8..32 => counters[1],
            32..128 => counters[2],
            _ => counters[3],
        };
        slot.fetch_add(1, Ordering::Relaxed);
    };

    add(&counters::root_iterations, 1);
    add(
        &counters::root_gap_sum,
        u64::from(shadow.gap.unsigned_abs()),
    );
    add(
        &counters::root_deviation_sum,
        u64::from(shadow.deviation.unsigned_abs()),
    );
    add(
        &counters::root_effort_ppm_sum,
        (shadow.effort.clamp(0.0, 1.0) * 1_000_000.0) as u64,
    );
    if shadow.best_changed {
        add(&counters::root_best_changes, 1);
    }
    add(&counters::shadow_4_7_root_confidence, 1);

    add(
        &counters::rootconf_scalar_sum,
        u64::from(shadow.scalar.clamp(0, 1000).unsigned_abs()),
    );
    let quartile = match shadow.scalar {
        i32::MIN..250 => &counters::rootconf_scalar_q1,
        250..500 => &counters::rootconf_scalar_q2,
        500..750 => &counters::rootconf_scalar_q3,
        _ => &counters::rootconf_scalar_q4,
    };
    add(quartile, 1);
    if shadow.no_rival {
        add(&counters::rootconf_gap_no_rival, 1);
    }
    let gap_slot = match shadow.gap {
        i32::MIN..1 => &counters::rootconf_gap_0,
        1..8 => &counters::rootconf_gap_1_7,
        8..128 => &counters::rootconf_gap_8_127,
        _ => &counters::rootconf_gap_128_plus,
    };
    add(gap_slot, 1);
    bucket(
        shadow.deviation,
        [
            &counters::rootconf_dev_lt_8,
            &counters::rootconf_dev_8_31,
            &counters::rootconf_dev_32_127,
            &counters::rootconf_dev_128_plus,
        ],
    );
    if shadow.effort > crate::search::EFFORT_TERM_FLOOR {
        add(&counters::rootconf_effort_term_live, 1);
    }
    if shadow.fails > 0 {
        add(&counters::rootconf_window_fail_iters, 1);
    }
    add(
        &counters::rootconf_best_age_sum,
        u64::try_from(shadow.best_age).unwrap_or(u64::MAX),
    );
    add(
        &counters::rootconf_instab_milli_sum,
        (shadow.instability.clamp(0.0, 1_000.0) * 1000.0) as u64,
    );
    if let Some(pooled) = shadow.pooled_instability {
        add(&counters::rootconf_pool_instab_milli_sum, pooled);
    }
    if shadow.pv_len < 2 {
        add(&counters::rootconf_pv_truncated, 1);
    }

    let baseline = (shadow.baseline_time.max(0.0) * 10_000.0) as u64;
    let candidate = (shadow.candidate_time.max(0.0) * 10_000.0) as u64;
    add(&counters::rootconf_tm_baseline_sum, baseline);
    add(&counters::rootconf_tm_candidate_sum, candidate);
    // "Disagrees" means by more than 1% of the baseline: below that the two
    // multipliers are the same decision expressed with different rounding.
    let tolerance = baseline / 100;
    if candidate > baseline.saturating_add(tolerance) {
        add(&counters::rootconf_tm_longer, 1);
    } else if candidate < baseline.saturating_sub(tolerance) {
        add(&counters::rootconf_tm_shorter, 1);
    }
}

/// 9.7.5(b) per-thread completed depth — the counter that distinguishes "the
/// pool is deep but the main thread is shallow" from "every thread is shallow".
/// A plain global counter cannot express it, so this is a small indexed table
/// written once per thread per search.
#[cfg(feature = "diag")]
pub mod smp {
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Threads tracked individually. Far below `MAX_THREADS` (1024) on purpose:
    /// nobody measures SMP quality at 1024 threads, and ids beyond this fold
    /// into the last slot rather than being lost or panicking.
    pub const MAX_TRACKED: usize = 64;

    /// `usize` so the caller's `completed_depth` needs no conversion — the
    /// project bans truncating casts and a lossless one would be noise here.
    pub static THREAD_DEPTH: [AtomicUsize; MAX_TRACKED] =
        [const { AtomicUsize::new(0) }; MAX_TRACKED];

    pub fn record_depth(thread_id: usize, depth: usize) {
        THREAD_DEPTH[thread_id.min(MAX_TRACKED - 1)].store(depth, Ordering::Relaxed);
    }

    pub fn reset() {
        for slot in &THREAD_DEPTH {
            slot.store(0, Ordering::Relaxed);
        }
    }

    /// Emits only threads that completed a depth, so the serial case prints one
    /// line and the dump stays readable.
    pub fn dump() {
        for (id, slot) in THREAD_DEPTH.iter().enumerate() {
            let depth = slot.load(Ordering::Relaxed);
            if depth > 0 {
                crate::info_string!("diag thread_depth_{} {}", id, depth);
            }
        }
    }
}

/// Record a thread's completed depth (no-op without the `diag` feature).
#[inline(always)]
pub fn record_thread_depth(thread_id: usize, depth: usize) {
    #[cfg(feature = "diag")]
    smp::record_depth(thread_id, depth);
    #[cfg(not(feature = "diag"))]
    {
        let _ = (thread_id, depth);
    }
}

/// 9.6(b) side-channel: `eval_king_safety` records the danger-table index it
/// reads, so the dual-eval comparison can bucket its findings by king danger
/// without threading a return value through the whole eval stack. A
/// thread-local (not an atomic) because each `Evaluator` runs on one thread —
/// this keeps worker threads from smearing each other's buckets.
#[cfg(feature = "diag")]
pub mod lazy_probe {
    use std::cell::Cell;

    thread_local! {
        static MAX_DANGER_IDX: Cell<usize> = const { Cell::new(0) };
    }

    pub fn reset() {
        MAX_DANGER_IDX.with(|c| c.set(0));
    }

    pub fn record(idx: usize) {
        MAX_DANGER_IDX.with(|c| c.set(c.get().max(idx)));
    }

    pub fn max() -> usize {
        MAX_DANGER_IDX.with(Cell::get)
    }
}

/// Increment a diagnostic counter by name. Expands to nothing without the
/// `diag` feature, so instrumentation sites cost zero in production builds.
#[cfg(feature = "diag")]
#[macro_export]
macro_rules! diag_count {
    ($name:ident) => {{
        $crate::diag::counters::$name.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }};
}

/// Add an unsigned value to a diagnostic counter. Like `diag_count!`, both the
/// expression and atomic disappear entirely from non-diagnostic builds.
#[cfg(feature = "diag")]
#[macro_export]
macro_rules! diag_add {
    ($name:ident, $value:expr) => {{
        $crate::diag::counters::$name.fetch_add($value, std::sync::atomic::Ordering::Relaxed);
    }};
}

#[cfg(not(feature = "diag"))]
#[macro_export]
macro_rules! diag_add {
    ($name:ident, $value:expr) => {};
}

#[cfg(not(feature = "diag"))]
#[macro_export]
macro_rules! diag_count {
    ($name:ident) => {};
}

/// Reset all counters (no-op without the `diag` feature).
///
/// ⚠ Must be called ONCE per `go`, by the main thread, BEFORE any helper is
/// spawned. Helpers reach `search_root` too, so a reset left there ran once per
/// thread and wiped whatever the earlier-starting threads had already counted —
/// every multi-thread diag number before 9.7.5(b) was junk for this reason.
#[inline(always)]
pub fn reset() {
    #[cfg(feature = "diag")]
    {
        counters::reset();
        smp::reset();
        correction_probe::reset();
    }
}

#[cfg(all(test, feature = "diag"))]
mod tests {
    use std::sync::atomic::Ordering;

    use crate::board::Board;

    use super::{SAMPLE_MAIN, SAMPLE_QSEARCH, counters, sampled};

    #[test]
    fn sampler_is_stable_sparse_and_domain_separated() {
        let first: Vec<_> = (0..65_536_u64)
            .filter(|hash| sampled(*hash, 7, SAMPLE_MAIN))
            .collect();
        let repeated: Vec<_> = (0..65_536_u64)
            .filter(|hash| sampled(*hash, 7, SAMPLE_MAIN))
            .collect();
        let qsearch: Vec<_> = (0..65_536_u64)
            .filter(|hash| sampled(*hash, 7, SAMPLE_QSEARCH))
            .collect();

        assert_eq!(first, repeated);
        assert!(
            (40..=88).contains(&first.len()),
            "sample size {}",
            first.len()
        );
        assert_ne!(first, qsearch);
    }

    #[test]
    fn board_profile_counters_are_live() {
        counters::reset();
        let mut board = Board::starting_position();
        let (captures, pinned) = board.generate_legal_captures_pinned();
        let quiets = board.generate_legal_quiets_pinned(pinned);
        assert!(captures.is_empty());
        assert_eq!(quiets.len(), 20);

        let check_info = board.check_info();
        let mv = board.parse_move("e2e4").expect("legal test move");
        let gives_check = board.gives_check_with(mv, &check_info);
        assert!(!gives_check);
        assert!(board.see_ge(mv, 0));
        board.make_move_with_check(mv, gives_check);
        board.unmake_move(mv);

        assert_eq!(
            counters::board_gen_staged_capture_calls.load(Ordering::Relaxed),
            1
        );
        assert_eq!(
            counters::board_gen_staged_quiet_moves.load(Ordering::Relaxed),
            20
        );
        assert_eq!(
            counters::board_compute_pinned_calls.load(Ordering::Relaxed),
            1
        );
        assert_eq!(counters::board_check_info_calls.load(Ordering::Relaxed), 1);
        assert_eq!(
            counters::board_gives_check_fast_calls.load(Ordering::Relaxed),
            1
        );
        assert_eq!(
            counters::board_see_threshold_calls.load(Ordering::Relaxed),
            1
        );
        assert_eq!(
            counters::board_make_with_check_calls.load(Ordering::Relaxed),
            1
        );
        assert_eq!(counters::board_unmake_calls.load(Ordering::Relaxed), 1);
        assert_eq!(counters::board_history_pushes.load(Ordering::Relaxed), 1);
        assert_eq!(counters::board_history_growths.load(Ordering::Relaxed), 0);

        counters::reset();
        let mut growth = Board::starting_position();
        for _ in 0..129 {
            growth.make_null_move();
        }
        assert_eq!(counters::board_history_pushes.load(Ordering::Relaxed), 129);
        assert_eq!(counters::board_history_growths.load(Ordering::Relaxed), 1);
    }
}

/// Dump all counters as `info string` lines (no-op without the `diag` feature).
///
/// ⚠ Must be called ONCE per `go`, by the main thread, AFTER the helpers have
/// been joined — otherwise the helper tail contributions are missing and, worse,
/// each helper emits its own competing set of lines.
#[inline(always)]
pub fn dump() {
    #[cfg(feature = "diag")]
    {
        counters::dump();
        smp::dump();
    }
}
