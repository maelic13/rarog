//! Phase 7.6 search diagnostics — compile-time gated counters.
//!
//! Enabled only with `--features diag`. The default build contains **no**
//! counter code at all (`diag_count!` expands to nothing), so `bench` stays a
//! stable fingerprint — the gate for this feature is *bench identical with diag
//! off*. When enabled, counters are process-global atomics (the search may run
//! several worker threads), reset at each `go`, and dumped as `info string diag
//! <name> <value>` lines when the search completes.
//!
//! Purpose (search audit §12): size the Phase-8 opportunities before they spend
//! SPRT slots — the check-node share (8.2), the stale-`tt_pv` prune-veto share
//! (8.3), the per-family prune rates, the LMR re-search rate, and the
//! history/correction event coverage.

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
    );
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
    ($name:ident) => {
        $crate::diag::counters::$name.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    };
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
