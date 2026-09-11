//! Phase 7.1 draw-semantics regression tests.
//!
//! Only fix (a) — rule-50/mate precedence — survived Phase 7.1: clock >= 100
//! is a draw only if the position is not checkmate (a mate delivered on the
//! 100th-clock move wins). It is bench-identical to the accepted head and
//! fires ~never, so it ships without an SPRT.
//!
//! The other three parts (null clock, cross-null repetition fence,
//! root-aware repetition) collectively lost two `[-3,3]` SPRTs (−7.21, then
//! −11.91 with the root-aware part removed) and were reverted — the whole
//! repetition/null-clock rework is anti-Elo at Rarog's level (PLAN.md lesson
//! 14). The aggressive twofold-is-a-draw behavior it tried to change is
//! kept, and asserted below.

use rarog::board::{Board, GameResult, Move};
use rarog::eval::MATE_SCORE;
use rarog::search::{SearchEvent, Searcher};
use rarog::search_options::SearchOptions;

/// Find the legal move with the given UCI string.
fn mv(board: &Board, uci: &str) -> Move {
    board
        .generate_legal_moves()
        .into_iter()
        .find(|m| m.to_string() == uci)
        .unwrap_or_else(|| panic!("move {uci} should be legal"))
}

fn search_to_depth(board: Board, depth: u32) -> (i32, String) {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.position.board = board.clone();
    options.limits.depth = Some(depth);
    let result = searcher.search(board, &options, false, || SearchEvent::None);
    (result.score, result.bestmove.to_string())
}

// ---------------------------------------------------------------- (a) ----

#[test]
fn mate_on_the_100th_clock_move_beats_the_rule50_draw() {
    // Ra8# is a quiet rook move: it pushes the clock from 99 to exactly 100.
    let mut board = Board::from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 99 80").unwrap();
    board.make_move(mv(&board, "a1a8"));
    assert_eq!(board.halfmove_clock, 100);
    assert_eq!(board.game_result(), Some(GameResult::WhiteCheckmates));
    assert!(!board.can_declare_draw());
    assert!(!board.can_declare_draw_in_search());
}

#[test]
fn check_with_an_escape_at_clock_100_is_still_a_draw() {
    // Same mating pattern but g7 is empty, so Kg7 escapes: the rule-50 claim
    // stands (only checkmate outranks it).
    let mut board = Board::from_fen("6k1/5p1p/8/8/8/8/8/R5K1 w - - 99 80").unwrap();
    board.make_move(mv(&board, "a1a8"));
    assert_eq!(board.halfmove_clock, 100);
    assert_eq!(board.game_result(), Some(GameResult::Draw));
    assert!(board.can_declare_draw_in_search());
}

#[test]
fn stalemate_at_clock_100_is_a_draw() {
    let board = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 100 80").unwrap();
    assert_eq!(board.game_result(), Some(GameResult::Draw));
    assert!(board.can_declare_draw_in_search());
}

#[test]
fn search_finds_mate_in_one_at_clock_99() {
    // Pre-7.1 the mating child (clock 100) returned draw 0 from negamax's
    // rule-50 check, so the search scored the position ~cp 0.
    let board = Board::from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 99 80").unwrap();
    let (score, bestmove) = search_to_depth(board, 4);
    assert_eq!(bestmove, "a1a8");
    assert!(
        score >= MATE_SCORE - 100,
        "expected a mate score, got {score}"
    );
}

// -------------------- aggressive twofold heuristic (7.1d reverted) --------

#[test]
fn a_single_prior_twofold_is_a_search_draw() {
    // The kept heuristic: one prior occurrence within the scan bound scores a
    // draw in search, whether or not it predates the search root. The
    // root-aware variant that suppressed the pre-root case (7.1d) lost 7 Elo
    // and was reverted (lesson 14).
    let mut board = Board::from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 10 40").unwrap();
    board.make_move(mv(&board, "a1a2"));
    board.make_move(mv(&board, "g8f8"));
    board.make_move(mv(&board, "a2a1"));
    board.make_move(mv(&board, "f8g8"));
    assert!(board.can_declare_draw_in_search());
    // The arbiter's threefold rule is unaffected: twice is not three times.
    assert!(!board.can_declare_draw());
}

#[test]
fn three_occurrences_are_a_threefold_for_the_arbiter() {
    let mut board = Board::from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 10 40").unwrap();
    for _ in 0..2 {
        board.make_move(mv(&board, "a1a2"));
        board.make_move(mv(&board, "g8f8"));
        board.make_move(mv(&board, "a2a1"));
        board.make_move(mv(&board, "f8g8"));
    }
    assert!(board.can_declare_draw_in_search());
    assert!(board.can_declare_draw());
}

/// 4.11b.15: the repetition identity is the position hash and NOTHING else.
///
/// PLAN forbids putting rule-50 buckets into the repetition identity merely
/// because a transposition-table key might want them. This pins the separation
/// executably: two positions that differ only in the halfmove clock must share
/// one hash, so the clock cannot leak into repetition matching. If a future
/// change mixes the clock into the Zobrist key, repetition detection silently
/// stops finding real repetitions and this test fails first.
#[test]
fn the_halfmove_clock_is_not_part_of_the_position_identity() {
    let cases = [
        (
            "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 40",
            "6k1/5ppp/8/8/8/8/8/R5K1 w - - 99 40",
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 3 12",
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 87 12",
        ),
    ];
    for (low, high) in cases {
        let low_board = Board::from_fen(low).expect("valid FEN");
        let high_board = Board::from_fen(high).expect("valid FEN");
        assert_ne!(
            low_board.halfmove_clock, high_board.halfmove_clock,
            "the two FENs must actually differ in the clock"
        );
        assert_eq!(
            low_board.hash, high_board.hash,
            "halfmove clock leaked into the position hash for {low}"
        );
    }
}

/// The rule-50 window bounds the repetition scan for COST, not correctness.
///
/// `is_repetition` stops at `halfmove_clock` plies back, and null moves advance
/// that clock, so the bound can reach past an irreversible move. That is safe:
/// an irreversible move changes piece placement permanently, so those older
/// positions carry a different hash and can never match. This test pins the
/// underlying property — a capture makes the prior position unreachable by
/// hash — so the scan bound stays a performance choice.
#[test]
fn an_irreversible_move_makes_the_earlier_position_hash_unreachable() {
    let mut board = Board::from_fen("4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1").expect("valid FEN");
    let before = board.hash;
    board.make_move(mv(&board, "e4d5"));
    assert_ne!(
        board.hash, before,
        "a capture must change the position hash"
    );
    // Shuffle kings back and forth: the pre-capture hash must never reappear.
    for uci in ["e8d8", "e1d1", "d8e8", "d1e1"] {
        board.make_move(mv(&board, uci));
        assert_ne!(
            board.hash, before,
            "pre-capture hash reappeared after {uci}; the rule-50 scan bound \
             would no longer be a pure cost bound"
        );
    }
}
