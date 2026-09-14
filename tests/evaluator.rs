//! Evaluator behaviour visible through the public API: perspective, rule-50 damping, passers, drawish endgames, terminal results.

use rarog::board::{Board, Color, GameResult, Piece};
use rarog::eval::{Evaluator, MATE_SCORE, piece_value};

#[test]
fn evaluator_scores_material_from_side_to_move_perspective() {
    let mut evaluator = Evaluator::default();
    let white_to_move = Board::from_fen("4k3/8/8/8/8/8/8/Q3K3 w - - 0 1").expect("valid FEN");
    let black_to_move = Board::from_fen("4k3/8/8/8/8/8/8/Q3K3 b - - 0 1").expect("valid FEN");

    assert!(evaluator.evaluate(&white_to_move) > piece_value(Piece::Queen) - 100);
    assert!(evaluator.evaluate(&black_to_move) < -piece_value(Piece::Queen) + 100);
}

#[test]
fn evaluator_dampens_static_advantage_near_fifty_move_draw() {
    let mut evaluator = Evaluator::default();
    let fresh = Board::from_fen("4k3/8/8/8/8/8/8/Q3K3 w - - 0 1").expect("valid FEN");
    let stale = Board::from_fen("4k3/8/8/8/8/8/8/Q3K3 w - - 90 46").expect("valid FEN");

    assert!(evaluator.evaluate(&stale).abs() < evaluator.evaluate(&fresh).abs());
}

#[test]
fn evaluator_rewards_advanced_protected_passers_over_back_rank_pawns() {
    let mut evaluator = Evaluator::default();
    // Enemy king parked on the a-file so it is far from *both* pawn pairs — the
    // comparison isolates pawn advancement. (With the king on e8 it sits right
    // next to the advanced e6 pawn, and the Phase-4.6 eval correctly discounts
    // the under-pressure advanced pawns, which is the more accurate read.)
    let advanced_connected =
        Board::from_fen("k7/8/4P3/3P4/8/8/8/4K3 w - - 0 1").expect("valid FEN");
    let undeveloped = Board::from_fen("k7/8/8/8/8/3P4/4P3/4K3 w - - 0 1").expect("valid FEN");

    let a = evaluator.evaluate(&advanced_connected);
    let u = evaluator.evaluate(&undeveloped);
    assert!(a > u, "advanced={a} undeveloped={u}");
}

#[test]
fn evaluator_scales_known_drawish_minor_endgames() {
    let mut evaluator = Evaluator::default();
    let two_knights_vs_bare_king =
        Board::from_fen("7k/8/8/8/8/8/8/NN2K3 w - - 0 1").expect("valid FEN");

    assert_eq!(evaluator.evaluate(&two_knights_vs_bare_king), 0);
}

#[test]
fn evaluator_reports_terminal_results_with_distance_to_mate() {
    let evaluator = Evaluator::default();

    assert_eq!(
        evaluator.evaluate_result(GameResult::WhiteCheckmates, Color::White, 3),
        MATE_SCORE - 3
    );
    assert_eq!(
        evaluator.evaluate_result(GameResult::WhiteCheckmates, Color::Black, 3),
        -MATE_SCORE + 3
    );
    assert_eq!(
        evaluator.evaluate_result(GameResult::BlackCheckmates, Color::Black, 11),
        MATE_SCORE - 11
    );
    assert_eq!(
        evaluator.evaluate_result(GameResult::Draw, Color::White, 99),
        0
    );
}
