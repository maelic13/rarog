//! Search diagnostics — compile-time gated counters and sampled traces.
//!
//! Enabled only with `--features diag`. The default build contains **no**
//! counter code at all (`diag_count!` expands to nothing), so `bench` stays a
//! stable fingerprint — the gate for this feature is *bench identical with diag
//! off*. When enabled, counters are process-global atomics (the search may run
//! several worker threads), reset at each `go`, and dumped as `info string diag
//! <name> <value>` lines when the search completes.
//!
//! Event counters are exact. A deterministic 1/1024 position sample feeds the
//! wider interaction map; this bounds diagnostic cost
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
        // oracle differential needs a qsearch denominator collected the same
        // way the oracle collects it, and a 1/1024 sample cannot be joined
        // against an exact count (analysis/phase4_counter_spec.md).
        nodes,
        qnodes,
        // Check-node cost.
        nodes_in_check,
        // A stale PV bit vetoes pruning at a non-PV node.
        tt_pv_veto,
        // Forward-pruning families (successful cutoffs / skips).
        rfp_cut,
        // Reverse-futility cutoffs by depth band (selectivity core).
        rfp_cut_d1_3,
        rfp_cut_d4_7,
        rfp_cut_d8_plus,
        // Hindsight reductions: a reduced child searched deeper or shallower.
        hindsight_up,
        hindsight_down,
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
        // Selectivity core: nodes whose quiets were skipped by late-move or
        // quiet-futility pruning, and the per-move prunes it adds.
        skip_quiets_nodes,
        // Quiet TT moves rewarded at a TT cutoff (selectivity core).
        tt_cutoff_quiet_bonus,
        quiet_see_prune,
        history_pruned,
        bad_noisy_futility,
        // LMR reduction and its verification re-search.
        lmr_applied,
        // Late moves whose reduction rounds to zero plies.
        node_lmr_zero_reduction,
        // Audit of the reduction floor. `lmr_reduction` is
        // `(r >> 10).clamp(0, new_depth)`, so both ends silently discard
        // information:
        //   node_lmr_qs_clamped -- reduction reached new_depth, so the "reduced
        //     search" ran at depth 0 and was answered by quiescence. That
        //     is a prune wearing a reduction's name, and it is counted
        //     nowhere in the pruning family.
        node_lmr_qs_clamped,
        lmr_research,
        // Selectivity core: reductions clamped to one ply, reductions that
        // extend, and re-searches made deeper or shallower.
        lmr_floor_hits,
        lmr_extended,
        lmr_research_deeper,
        lmr_research_shallower,
        // History / correction learning events. `cutoff_quiet + cutoff_capture`
        // is also the count of every beta cutoff at a real (non-excluded)
        // interior node, i.e. the DENOMINATOR of the ordering metric below.
        cutoff_quiet,
        cutoff_capture,
        // FIRST-MOVE CUTOFF RATE, the standard move-ordering readout:
        // `cutoff_first_move / (cutoff_quiet + cutoff_capture)`. Counted where
        // the move that failed high was the FIRST move the node searched.
        //
        // The over-reduction ratio (`lmr_research / lmr_applied`) reads the
        // PRUNING-DEPTH side of how the search converts nodes into decisions;
        // this counter reads the ORDERING side, and the two imply opposite
        // fixes. Healthy engines sit ~90%+; materially below implicates
        // ordering rather than the selectivity surface.
        //
        // Excluded-move (singular-verification) searches do NOT count: their
        // best move is deliberately withheld, so a first-move cutoff there
        // measures the exclusion, not the ordering. That is automatic — this
        // sits inside the same `excluded.is_null()` guard as the two above, so
        // numerator and denominator always cover the same node set.
        cutoff_first_move,
        correction_updates,
        corr_on_capture,
        // Continuation-correction training admitted at the context two and
        // four plies back (selectivity core; trained before it is read).
        corr_cont2_admitted,
        corr_cont4_admitted,
        // Residual MAGNITUDE by attribution class, exact.
        //
        // The premise behind capture-weighted correction updates is that a
        // capture-caused residual is less trustworthy evidence for a
        // positional correction. These give the mean
        // |residual| for each class; if the two means are close, the premise is
        // wrong and neither knob should move off its baseline.
        corr_resid_capture_n,
        corr_resid_capture_sum,
        corr_resid_quiet_n,
        corr_resid_quiet_sum,
        // Residual by HALFMOVE-CLOCK context. A new correction context needs
        // held-out unique signal, so the measurement comes before any proposal.
        // Rule-50 proximity is the
        // plausible mechanism: near the horizon a position's value stops being a
        // function of its structure.
        //
        // A check/evasion context is NOT measured, because
        // it is structurally unreachable: correction only trains where
        // `static_eval != VALUE_NONE`, which IS the not-in-check condition, so
        // its population is zero by construction rather than by observation.
        corr_resid_hm_low_n,
        corr_resid_hm_low_sum,
        corr_resid_hm_mid_n,
        corr_resid_hm_mid_sum,
        corr_resid_hm_high_n,
        corr_resid_hm_high_sum,
        // SMP quality: where does helper work go when more threads buy nodes
        // but little depth?
        //
        // Aspiration churn. A thread re-centres its window on the
        // POOL's deepest Exact score; if the pool disagrees with what the
        // thread then finds, it pays fail-high/low re-searches. A re-search
        // rate that climbs with thread count indicts pool-seeded windows.
        asp_fail_high,
        asp_fail_low,
        // Does helper work actually REACH the main thread? Probe/hit counted
        // on thread 0 only. If helpers contribute, main's hit rate should rise
        // with thread count; if it is flat, the helpers are searching in vain.
        main_tt_probes,
        main_tt_hits,
        // Lazy-eval safety audit. On every lazy skip the full eval is
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
        // Sampled node and TT outcome map.
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
        // Why an entry that HIT could not be used, split by cause,
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
        // SIZING a staged in-check qsearch.
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
        nmp_verify_attempt,
        nmp_verify_pass,
        nmp_verify_fail,
        // An NMP cutoff whose RETURNED SCORE is mate-range, i.e. a mate
        // this node never proved by a real line. The search clamps it to beta
        // (as Stockfish does); this counts how often the clamp fires.
        nmp_cut_unproven_mate,
        // Per-NODE: nodes passing the ProbCut entry gate, counted before
        // capture generation, so nodes with no eligible capture are included.
        // Per-MOVE: `probcut_attempt` -- a ProbCut search was actually started,
        // which is what the spec says and what the oracle counts. The two are
        // different units; never difference one against the other.
        probcut_nodes,
        probcut_attempt,
        probcut_qpass,
        probcut_tt_store,
        singular_attempt,
        singular_extend_one,
        singular_extend_two,
        singular_multicut,
        singular_negative_extension,
        iir_applied,
        iir_pv,
        iir_no_tt_move,
        iir_shallow_tt,
        // Move-stage recall and pruning overlap. Counts cover sampled nodes only.
        move_seen_tt,
        move_seen_good_capture,
        move_seen_quiet,
        move_seen_bad_capture,
        // Core (comparable): rank at which a beta cutoff occurred. Exact, and
        // counted in the same block as cutoff_quiet/cutoff_capture so the
        // buckets sum to that denominator.
        best_rank_1,
        best_rank_2_3,
        best_rank_4_7,
        best_rank_8_plus,
        prune_shadow_moves,
        prune_shadow_lmp,
        prune_shadow_futility,
        prune_shadow_see,
        prune_shadow_check_exempt,
        prune_shadow_overlap_two_plus,
        prospective_depth_sum,
        reduction_depth_sum,
        // Correction attribution and hashed-table quality.
        corr_sample_updates,
        corr_sample_abs_sum,
        corr_slot_first,
        corr_slot_repeat,
        corr_slot_collision,
        corr_slot_near_saturation,
        // Root iteration census.
        root_iterations,
        root_best_changes,
        root_interrupted_fallback,
        worker_best_disagreement,
        worker_depth_spread_sum,
        worker_score_spread_sum,
        // Endgame-family occurrence in the SEARCH TREE, which differs from
        // occurrence on the board in real games: a bare-king minor mate drive
        // can leave `bench 13` byte-identical (no bench tree reaches one at
        // depth 13) while a rook-ending scale moves it by 14%. EXACT, not
        // sampled -- these are cheap and a ratio against
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

/// Count one evaluation against the reference-family table.
///
/// Called from `evaluate()` and compiled out entirely without `--features
/// diag`, so the production fingerprint is untouched -- which is this feature's
/// own acceptance gate.
///
/// The table is the 20 families of the final pre-NNUE Stockfish dispatcher, in
/// the order the dispatcher tries them. `counts` is (pawns, knights, bishops, rooks,
/// queens) per side; each family is tried in both orientations, so a Black
/// strong side counts the same as a White one.
#[cfg(feature = "diag")]
pub(crate) fn record_endgame_family(w: [u32; 5], b: [u32; 5]) {
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
pub(crate) const SAMPLE_MAIN: u64 = 0x4D41_494E_5F34_2E31;
#[cfg(feature = "diag")]
pub(crate) const SAMPLE_QSEARCH: u64 = 0x5153_4541_5243_4831;
#[cfg(feature = "diag")]
pub(crate) const SAMPLE_CORRECTION: u64 = 0x434F_5252_5F34_2E31;

/// Sampling stride mask, read once from `RAROG_DIAG_SAMPLE_STRIDE`.
///
/// The differential suite needs the CORE counters exact, because the
/// oracle collects them exactly and a 1/1024 sample cannot be joined against an
/// exact count — the ratio reads 1024x off while looking plausible. Rather than
/// lift seventeen counters out of their sampling guards in the hottest file in
/// the engine, the stride itself is configurable, so `RAROG_DIAG_SAMPLE_STRIDE=1`
/// makes every sampled counter exact in one place.
///
/// The stride must be a power of two; anything else falls back to the 1024
/// default, which keeps every historical reading reproducible by simply not
/// setting the variable.
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
/// position is sampled — the exact mode the oracle differential requires.
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
            None => counters::corr_slot_first.fetch_add(1, Ordering::Relaxed),
            Some(old) if old == key => counters::corr_slot_repeat.fetch_add(1, Ordering::Relaxed),
            Some(_) => counters::corr_slot_collision.fetch_add(1, Ordering::Relaxed),
        };
        if value.unsigned_abs() >= 15_000 {
            counters::corr_slot_near_saturation.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(feature = "diag")]
#[inline]
pub(crate) fn record_correction_slot(source: u8, index: usize, key: u64, value: i16) {
    correction_probe::record(source, index, key, value);
}

/// Side-channel: `eval_king_safety` records the danger-table index it
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
/// spawned. Helpers reach `search_root` too, so a reset there would run once
/// per thread and wipe whatever the earlier-starting threads had counted.
#[inline(always)]
pub fn reset() {
    #[cfg(feature = "diag")]
    {
        counters::reset();
        correction_probe::reset();
    }
}

#[cfg(all(test, feature = "diag"))]
mod tests {
    use std::sync::atomic::Ordering;

    use crate::board::{Board, MoveList};

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
        let mut captures = MoveList::new();
        let pinned = board.generate_legal_captures_pinned_into(&mut captures);
        let mut quiets = MoveList::new();
        board.generate_legal_quiets_pinned_into(pinned, &mut quiets);
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
pub(crate) fn dump() {
    #[cfg(feature = "diag")]
    {
        counters::dump();
    }
}
