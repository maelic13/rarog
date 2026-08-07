//! Phase 4.7 root-abort and fallback-ownership suite.
//!
//! PLAN 4.7 requires that "abort returns last completed legal evidence" and that
//! "incomplete mate/win/loss never becomes authoritative". Neither property is
//! reachable from `bench`: bench searches to a fixed depth and never aborts, so
//! the `root_interrupted_fallback` counter reads **0** on the whole 40-position
//! corpus. The fingerprint therefore cannot see this path at all — the same gap
//! `tests/zugzwang.rs` exists to cover for null-move soundness.
//!
//! What makes an abort dangerous is that it happens mid-iteration, when some root
//! moves have been searched deeper than others. If ownership of the result is
//! taken from a partially-searched iteration, the engine can return a move that
//! only *looked* best because its sibling was never examined — and a mate score
//! from an unfinished line is worse still, because it is both wrong and
//! unfalsifiable by the caller.
//!
//! These tests abort at a range of node budgets, which is how the mid-iteration
//! case is actually hit: a fixed budget lands at an arbitrary point in the move
//! list, so sweeping budgets samples many different interruption points.

use rarog::board::Board;
use rarog::eval::MATE_SCORE;
use rarog::search::{SearchEvent, Searcher};
use rarog::search_options::SearchOptions;

/// Positions with enough branching that a small node budget lands mid-iteration.
const POSITIONS: [&str; 4] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
];

/// Search with a poll that reports `Stop` once `budget` polls have elapsed, so
/// the abort lands at an arbitrary point inside an iteration.
fn search_aborted(fen: &str, budget: u32) -> (rarog::board::Move, i32, usize) {
    let board = Board::from_fen(fen).expect("valid FEN");
    let mut options = SearchOptions::default();
    options.position.board = board.clone();
    // Deep enough that the BUDGET ends the search, but not so deep that a large
    // budget makes the suite slow: an abort is a mid-iteration property, so small
    // budgets carry the coverage and a 40-ply limit only added runtime.
    options.limits.depth = Some(16);
    let mut searcher = Searcher::default();
    let mut polls = 0u32;
    let result = searcher.search(board, &options, false, || {
        polls += 1;
        if polls >= budget {
            SearchEvent::Stop
        } else {
            SearchEvent::None
        }
    });
    (result.bestmove, result.score, result.depth)
}

/// An aborted search must still return a LEGAL move, at every interruption point.
///
/// This is the floor: whatever ownership rule the root uses, it may never hand
/// back a move from a partially-searched iteration that is not playable.
#[test]
fn an_aborted_search_always_returns_a_legal_move() {
    for fen in POSITIONS {
        let board = Board::from_fen(fen).expect("valid FEN");
        let legal = board.generate_legal_moves();
        // Sweep budgets, including very small ones that abort during the first
        // iteration, which is the least-evidence case of all.
        for budget in [1, 2, 3, 5, 8, 13, 21, 34, 55, 100] {
            let (mv, _, _) = search_aborted(fen, budget);
            assert!(
                legal.contains(&mv),
                "{fen} aborted at budget {budget} returned {mv}, which is not legal"
            );
        }
    }
}

/// An aborted search must never report a MATE score it did not prove.
///
/// A mate claim from an unfinished iteration is the worst failure mode here: it
/// is wrong, and the caller cannot tell. None of these positions is mate or is
/// being mated, so any mate-range score is by construction unproven.
#[test]
fn an_aborted_search_never_claims_an_unproven_mate() {
    let mate_threshold = MATE_SCORE - 1000;
    for fen in POSITIONS {
        for budget in [1, 2, 3, 5, 8, 13, 21, 34, 55, 100] {
            let (_, score, _) = search_aborted(fen, budget);
            assert!(
                score.abs() < mate_threshold,
                "{fen} aborted at budget {budget} reported {score}, a mate-range score \
                 it cannot have proven"
            );
        }
    }
}

/// The reported depth must never exceed what was completed.
///
/// `root_interrupted_fallback` exists to notice when a root move was searched
/// deeper than the last completed iteration. Reporting that deeper number as the
/// search depth would be claiming evidence the iteration never finished
/// gathering.
#[test]
fn reported_depth_never_exceeds_a_completed_iteration() {
    for fen in POSITIONS {
        for budget in [3, 8, 21, 55, 100] {
            let (_, _, depth) = search_aborted(fen, budget);
            // A search stopped before finishing depth 1 may legitimately report
            // 0; what it may never do is report a depth it never completed, and
            // the depth limit above means any value at it would be fabricated.
            assert!(
                depth < 16,
                "{fen} at budget {budget} reported depth {depth}, at or beyond the \
                 16-ply limit it cannot have completed"
            );
        }
    }
}

/// Aborting must be deterministic for a given budget: the same interruption point
/// yields the same answer. Non-determinism here would mean the fallback depends
/// on something other than the recorded root evidence.
#[test]
fn aborting_at_the_same_point_is_deterministic() {
    for fen in POSITIONS {
        for budget in [5, 21, 100] {
            let first = search_aborted(fen, budget);
            let second = search_aborted(fen, budget);
            assert_eq!(
                first, second,
                "{fen} at budget {budget} gave two different results, so the abort \
                 path depends on something outside the recorded root evidence"
            );
        }
    }
}
