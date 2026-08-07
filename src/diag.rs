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
#[allow(non_upper_case_globals)]
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
        // Denominators.
        nodes,
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
        lmp_prune,
        quiet_futility_prune,
        see_prune,
        // LMR reduction and its verification re-search.
        lmr_applied,
        // 10.2.5 — late moves whose confidence estimate removes the old
        // mandatory one-ply reduction.
        lmr_zero_reduction,
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
        tt_bound_not_usable,
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
        // Root confidence/SMP observations use fixed-point sums (ppm/cp²).
        root_iterations,
        root_gap_sum,
        root_variance_sum,
        root_effort_ppm_sum,
        root_best_changes,
        root_interrupted_fallback,
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
        // `ev.depth - EvalPruneTtMinDepth`, so a penalty of P plies blocks
        // exactly the cases with slack < P — one histogram answers every P.
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
        // 4.3 SHADOW — is TT eval refinement SELF-CANCELLING?
        //
        // Two arms of `EvalPruneTtMinDepth` (1 and 2) both measured ~0 Elo while
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
    );
}

/// Stable domains keep independent samples from accidentally selecting exactly
/// the same positions. Public constants make call sites self-documenting.
#[cfg(feature = "diag")]
pub const SAMPLE_MAIN: u64 = 0x4D41_494E_5F34_2E31;
#[cfg(feature = "diag")]
pub const SAMPLE_QSEARCH: u64 = 0x5153_4541_5243_4831;
#[cfg(feature = "diag")]
pub const SAMPLE_CORRECTION: u64 = 0x434F_5252_5F34_2E31;

/// Deterministic 1/1024 position sampler. It is deliberately available only in
/// diagnostic builds: production code must contain neither the mix nor a branch.
#[cfg(feature = "diag")]
#[inline]
pub fn sampled(hash: u64, ply: usize, domain: u64) -> bool {
    let mut value = hash ^ domain ^ (ply as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((value ^ (value >> 31)) & 1023) == 0
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
            .unwrap_or_else(|error| error.into_inner())
            .clear();
    }

    pub fn record(source: u8, index: usize, key: u64, value: i16) {
        use crate::diag::counters;
        let mut owners = owners().lock().unwrap_or_else(|error| error.into_inner());
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
        1 => &counters::best_rank_1,
        2 | 3 => &counters::best_rank_2_3,
        4..=7 => &counters::best_rank_4_7,
        _ => &counters::best_rank_8_plus,
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

/// Root statistics are cold and diagnostic-only. Floating-point conversion is
/// intentionally lossy because these are aggregate telemetry units, not search
/// inputs (effort in ppm and variance in cp²).
#[cfg(feature = "diag")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn record_root_iteration(gap: i32, variance: f64, effort: f64, changed: bool) {
    use std::sync::atomic::Ordering;

    counters::root_iterations.fetch_add(1, Ordering::Relaxed);
    counters::root_gap_sum.fetch_add(u64::from(gap.unsigned_abs()), Ordering::Relaxed);
    counters::root_variance_sum.fetch_add(
        variance.max(0.0).min(u64::MAX as f64) as u64,
        Ordering::Relaxed,
    );
    counters::root_effort_ppm_sum.fetch_add(
        (effort.clamp(0.0, 1.0) * 1_000_000.0) as u64,
        Ordering::Relaxed,
    );
    if changed {
        counters::root_best_changes.fetch_add(1, Ordering::Relaxed);
    }
    counters::shadow_4_7_root_confidence.fetch_add(1, Ordering::Relaxed);
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
    use super::{SAMPLE_MAIN, SAMPLE_QSEARCH, sampled};

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
