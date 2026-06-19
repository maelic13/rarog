//! Phase 3.14 regression guard: the whole-eval cache must be a true
//! memoisation — a cached evaluation must equal a cold recompute.
//!
//! The bug this protects against: a term whose value depends on state *not*
//! captured by the cache key (e.g. the passed-pawn free-stop / safe-stop bonus,
//! which depends on non-pawn occupancy, once lived inside the pawn-structure
//! cache keyed by pawns only). Such a term makes `evaluate()` impure, so a
//! long-lived evaluator returns stale values for positions that collide on the
//! key. Here we walk many positions through one long-lived evaluator (which
//! exercises the pawn cache and the whole-eval cache) and assert every result
//! matches a fresh evaluator's cold computation of the same position.

use rarog::board::Board;
use rarog::eval::Evaluator;

const SEEDS: [&str; 10] = [
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "8/pp2k3/8/2p5/2P5/1P2K3/P7/8 w - - 0 1",
    "5k2/5p1p/p3B1p1/Pp6/1P6/5P1P/4K1P1/8 b - - 0 1",
    "8/8/p1p5/1p5p/1P5P/P1P5/8/K1k5 w - - 0 1",
    "1k6/1b6/8/5p2/p1p2p2/P7/1P3P2/K7 b - - 0 1",
    "6k1/p3q2p/1nr3p1/8/3Q4/7P/PP4P1/4R1K1 b - - 0 1",
    "2r3k1/1q2Rp1p/p2p2p1/1p1P4/1Pp1P3/2Q5/1P4PP/6K1 w - - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "4k3/pp1p1p1p/8/2P1P3/3P1P2/8/PP4PP/4K3 w - - 0 1",
    "8/1P6/2K5/8/4k3/8/6p1/8 w - - 0 1",
];

#[test]
fn eval_cache_equals_cold_recompute() {
    // One long-lived evaluator: its pawn cache and whole-eval cache accumulate
    // entries as we walk, so later positions exercise cache hits.
    let mut cached_eval = Evaluator::default();
    let mut checked = 0usize;

    for seed in SEEDS {
        let mut board = Board::from_fen(seed).unwrap_or_else(|e| panic!("bad FEN {seed}: {e}"));
        for _ in 0..80 {
            let cached = cached_eval.evaluate(&board);
            // A fresh evaluator's first call is always a cold (cache-miss) compute.
            let cold = Evaluator::default().evaluate(&board);
            assert_eq!(
                cached,
                cold,
                "eval cache disagrees with a cold recompute at {} (cached {cached} != cold {cold})",
                board.to_fen()
            );
            checked += 1;

            let mut moves = board.generate_legal_moves();
            if moves.is_empty() {
                break;
            }
            moves.sort_by_key(|m| m.to_string());
            let mv = moves[0];
            board.make_move(mv);
        }
    }

    assert!(
        checked > 200,
        "expected to check many positions, got {checked}"
    );
}
