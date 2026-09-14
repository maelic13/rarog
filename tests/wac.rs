//! WAC tactical-suite floor test — guards against gross tactical breakage.
//!
//! Runs the full 300-position suite at a shallow fixed depth and asserts a
//! conservative pass-count floor (SaberTooth's pattern: a floor, not
//! all-pass, so normal search evolution doesn't flap the suite — only a
//! collapse fails it). The floor sits well under the calibrated solved
//! count; `wac [depth]` (the engine command) is the fine-grained per-step
//! diagnostic, this test is only the tripwire.
//!
//! PRODUCTION-FEATURES ONLY.
//!
//! Under `--features texel`, LAZY EVAL IS DISABLED (`eval.rs`: `let lazy =
//! false`) so the tuner traces and fits the full eval. Lazy eval is an
//! approximation by design — when material + PST + pawns already decide a
//! position by more than any positional term could flip, the expensive
//! positional block is skipped. Turning it off therefore produces genuinely
//! different eval scores in lopsided positions, a different search path, and
//! different depth-sensitive results.
//!
//! Verified by experiment, not assumed: forcing `lazy = false` in a normal
//! default-feature build reproduces the identical failure. The caches are NOT
//! the cause — `texel` also bypasses them, but both are exact memoisations
//! (`tests/eval_cache.rs` proves a hit equals a cold recompute), so bypassing
//! them costs speed and changes nothing else.
//!
//! This suite asserts SHIPPED behaviour, so under `texel` it would be
//! asserting an engine we never release. CI runs the engine suites on default
//! features (`-p rarog`) for the same reason.
#![cfg(not(feature = "texel"))]

use rarog::board::Board;
use rarog::search::{SearchEvent, Searcher};
use rarog::search_options::SearchOptions;
use rarog::wac::{move_matches_any, wac_positions};

const TEST_DEPTH: u32 = 6;
/// A deliberate FLOOR, not an exact count (Basilisk canary lesson: gate
/// correctness, not shape) — ~8% headroom absorbs benign drift; only a
/// tactical collapse trips it.
///
/// History:
///   - 2026-07-15: calibrated 147/300 at the p5+7.0b head → FLOOR 135.
///     Still 146/300 at the 7.2 SEE-bundle head, so Phase-7 work barely
///     moved it.
///   - 2026-07-19: **RE-BASELINED to 122 for the 8.2(a) head** (133/300).
///     Removing the unconditional in-check extension deliberately shifts the
///     fixed-depth shape: the engine does ~40% less work per nominal depth
///     here (1.81M vs 3.07M nodes at depth 6), so a fixed-depth count drops
///     even though the search got *stronger*. Measured at EQUAL node cost it
///     is decisively better (203/300 in 28.4M nodes vs the old head's
///     185/300 in 29.3M), and the change won its SPRT at **+30.75 ± 8.83,
///     LOS 100%**. The floor was moved only AFTER that evidence, never to
///     make the change pass — it tripped at 133 vs 135 during the gate and
///     was left red on purpose until the SPRT decided.
const FLOOR: usize = 122;

#[test]
fn aspiration_terminates_on_sudden_mate_scores() {
    // WAC.005: depth 3 scores cp -78, depth 4 finds mate-in-3 — the exact
    // shape that made the pre-7.0b widening loop fail high forever (the
    // stale best_score-centered window could never reach the mate score;
    // PLAN.md lesson 13). With the 7.0b guard the whole depth-8 search
    // costs ~1k nodes. The node cap exists only so a regression fails the
    // assert instead of hanging the test runner; any future aspiration
    // rework (Phase 10.2a replaces the widening loop wholesale) must keep
    // this invariant: termination by construction on mate-magnitude scores.
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
fn wac_solved_count_stays_above_floor() {
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
        options.limits.depth = Some(TEST_DEPTH);
        let result = searcher.search(board, &options, false, || SearchEvent::None);
        if move_matches_any(&options.board, result.bestmove, &pos.best_moves) {
            solved += 1;
        }
    }

    assert!(
        solved >= FLOOR,
        "WAC solved {solved}/300 at depth {TEST_DEPTH} — below the floor {FLOOR}; \
         a tactical regression has likely landed"
    );
}
