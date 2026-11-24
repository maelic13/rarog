//! Search diagnostics — compile-time gated counters.
//!
//! Enabled only with `--features diag`. The default build contains **no**
//! counter code at all (`diag_count!` expands to nothing), so `bench` stays a
//! stable fingerprint — the gate for this feature is *bench identical with diag
//! off*. When enabled, counters are process-global atomics (the search may run
//! several worker threads), reset at each `go`, and dumped as `info string diag
//! <name> <value>` lines when the search completes. Every counter is exact and
//! observational only: none may steer the search. The kernel's per-node rule
//! counts are its probes (`search/compact/probes.rs`), not these.

#[cfg(feature = "diag")]
// Counter statics are deliberately lower_snake_case: the name is emitted
// verbatim as the `info string diag <name>` label.
#[expect(non_upper_case_globals, reason = "each counter's name is its label")]
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
        // SMP quality: where does helper work go when more threads buy nodes
        // but little depth?
        //
        // Aspiration churn. A thread re-centres its window on the
        // POOL's deepest Exact score; if the pool disagrees with what the
        // thread then finds, it pays fail-high/low re-searches. A re-search
        // rate that climbs with thread count indicts pool-seeded windows.
        asp_fail_high,
        asp_fail_low,
        // The aspiration loop's cost, all EXACT and Rarog-only: nodes of every
        // root window search from depth 4, and of those that failed low or
        // high and were searched again.
        asp_search_nodes,
        asp_fail_low_nodes,
        asp_fail_high_nodes,
        // Completed single-PV root iterations, EXACT: those that prove a
        // win; later ones in the same search that no longer do; and of those,
        // the ones whose move differs from the last proving iteration's.
        iter_mate_win,
        iter_mate_win_lost,
        iter_mate_win_lost_move,
        // Single-PV searches, EXACT: those that proved a win in a completed
        // iteration and end without one, and of those, the ones whose final
        // move differs from the last proving iteration's.
        search_mate_win_dropped,
        search_mate_win_dropped_move,
        // Root iteration census.
        root_iterations,
        root_best_changes,
        root_interrupted_fallback,
        // A single-PV search stopped inside an iteration, and what the
        // fallback to the last completed one discards: a move that raised
        // alpha in the interrupted window, a move an earlier window of the
        // iteration failed high on, either.
        root_searches,
        root_stop_mid_iteration,
        root_stop_partial_new_best,
        root_stop_after_fail_high_new,
        root_stop_discards,
        // Clock stops: a proved mate held for three iterations (or a loss in
        // two plies), a root fail-high past the soft target, and hard-limit
        // polls held because no iteration had completed.
        tm_mate_stop,
        tm_fail_high_stop,
        tm_first_iteration_guard,
        // Endgame-family occurrence in the SEARCH TREE, which differs from
        // occurrence on the board in real games: a bare-king minor mate drive
        // can leave `bench 13` byte-identical (no bench tree reaches one at
        // depth 13) while a rook-ending scale moves it by 14%. Counted on every
        // evaluation call, a unit apart from the node count (moves made).
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
        // Board cost census. These are exact call/work counters from
        // real search paths. They observe only and compile away completely
        // without `--features diag`.
        board_gen_vec_calls,
        board_gen_vec_moves,
        board_gen_full_calls,
        board_gen_full_moves,
        board_gen_staged_capture_calls,
        board_gen_staged_capture_moves,
        board_gen_staged_quiet_calls,
        board_gen_staged_quiet_moves,
        board_compute_pinned_calls,
        board_check_info_calls,
        board_gives_check_full_calls,
        board_calculate_checkers_calls,
        board_see_full_calls,
        board_see_threshold_calls,
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
        // twice. KPKP is the only one in this table.
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
    }
}

#[cfg(all(test, feature = "diag"))]
mod tests {
    use std::sync::atomic::Ordering;

    use crate::board::{Board, MoveList};

    use super::counters;

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

        let _ = board.check_info();
        let mv = board.parse_move("e2e4").expect("legal test move");
        let gives_check = board.gives_check(mv);
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
            counters::board_gives_check_full_calls.load(Ordering::Relaxed),
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
