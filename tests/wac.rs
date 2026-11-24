//! WAC tactical-suite floor test — guards against gross tactical breakage.
//!
//! Runs the full 300-position suite on a fixed node budget and asserts a
//! conservative pass-count floor (SaberTooth's pattern: a floor, not
//! all-pass, so normal search evolution doesn't flap the suite — only a
//! collapse fails it). The floor sits well under the calibrated solved
//! count; `wac [depth]` (the engine command) is the fine-grained per-step
//! diagnostic, this test is only the tripwire.
//!
//! PRODUCTION-FEATURES ONLY. This suite asserts SHIPPED behaviour, so it is
//! compiled out under `texel` (whose evaluator records a trace and bypasses
//! the caches), and CI runs the engine suites on default features
//! (`-p rarog`) for the same reason.
//!
//! The lazy gate matters here: when material, piece-square and pawn terms
//! already decide a position by more than the margin, the positional block is
//! skipped, so a build without the gate scores lopsided positions differently,
//! searches a different tree and fails this floor. Verified by experiment:
//! forcing `lazy = false` in a default-feature build fails it. The caches are
//! not a factor: both are exact memoisations (`tests/eval_cache.rs` proves a
//! hit equals a cold recompute).
#![cfg(not(feature = "texel"))]

use rarog::board::Board;
use rarog::search::options::SearchOptions;
use rarog::search::{SearchEvent, Searcher};
use rarog::tools::wac::{move_matches_any, wac_positions};

/// Nodes a position: a budget, not a depth, so the count reads tactics per
/// unit of work, which a change of per-ply effort does not move by itself.
const TEST_NODES: u64 = 50_000;
/// A band around the calibrated count, not the count itself, so ordinary
/// search changes do not flap it: about 8% each way. Below it a tactical
/// regression has likely landed; above it the band is stale, and both bounds
/// move to the new count, in the commit that moved it. Calibrated at 218/300
/// on the compact kernel with quiescence's stalemate rule.
const CALIBRATED: usize = 218;
const FLOOR: usize = 201;
const CEILING: usize = 235;

#[test]
fn aspiration_terminates_on_sudden_mate_scores() {
    // WAC.005: depth 3 scores cp -78, depth 4 finds mate-in-3 — the shape
    // that would make a window widening loop fail high forever if the window
    // could never reach the mate score. The node cap exists only so a
    // regression fails the assert instead of hanging the test runner; the
    // aspiration window must terminate by construction on mate-magnitude
    // scores.
    let board = Board::from_fen("5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - - 0 1").unwrap();
    let mut searcher = Searcher::default();
    let mut options = SearchOptions {
        board: board.clone(),
        ..SearchOptions::default()
    };
    options.limits.depth = Some(8);
    options.limits.nodes = 2_000_000;
    let result = searcher.search(board, &options, false, || SearchEvent::None);
    assert!(
        result.nodes < 500_000,
        "aspiration re-search explosion: {} nodes for a depth-8 mate-in-2 search",
        result.nodes
    );
    assert_eq!(
        result.bestmove.to_string(),
        "c6c4",
        "expected the WAC.005 mate Qc4+"
    );
}

#[test]
#[ignore = "long tier: 300-position tactics floor at fixed depth; run by name with --ignored in release (PROCESS, Common commands)"]
fn wac_solved_count_stays_in_its_band() {
    let positions = wac_positions();
    assert_eq!(positions.len(), 300);

    let mut searcher = Searcher::default();
    let mut solved = 0usize;
    for pos in &positions {
        let board = Board::from_fen(&pos.fen).expect("suite FENs are legal");
        searcher.new_game();
        let mut options = SearchOptions {
            board: board.clone(),
            ..SearchOptions::default()
        };
        options.limits.nodes = TEST_NODES;
        let result = searcher.search(board, &options, false, || SearchEvent::None);
        if move_matches_any(&options.board, result.bestmove, &pos.best_moves) {
            solved += 1;
        }
    }

    assert!(
        solved >= FLOOR,
        "WAC solved {solved}/300 at {TEST_NODES} nodes — below the floor {FLOOR}; \
         a tactical regression has likely landed"
    );
    assert!(
        solved <= CEILING,
        "WAC solved {solved}/300 at {TEST_NODES} nodes, above the band's ceiling {CEILING} \
         around {CALIBRATED}: recalibrate both bounds to the new count"
    );
}
