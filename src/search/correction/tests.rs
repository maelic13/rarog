use super::*;
use crate::board::{Move, Piece};
use crate::score::TB_WIN_SCORE;

const FEN: &str = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";

/// The table stores the raw eval under a key that ignores the halfmove
/// clock, so the raw eval must not depend on the clock: a value stored at
/// clock 0 and read at clock 80 evaluates to exactly what a fresh
/// evaluation at clock 80 does, and that is damped toward the draw.
#[test]
fn a_stored_raw_eval_serves_any_halfmove_clock() {
    let mut searcher = Searcher::default();
    let early = Board::from_fen("4k3/8/8/8/8/8/3Q4/4K3 w - - 0 60").expect("valid FEN");
    let late = Board::from_fen("4k3/8/8/8/8/8/3Q4/4K3 w - - 80 60").expect("valid FEN");
    assert_eq!(early.hash(), late.hash(), "the table key ignores the clock");
    let raw = searcher.raw_eval(&early);
    searcher.shared.tt.store_eval(early.hash(), raw, false);
    let stored = crate::tt::TtProbe::from_entry(
        searcher.shared.tt.probe(late.hash()),
        0,
        late.halfmove_clock(),
    )
    .raw_static_eval;
    let fresh = searcher.raw_eval(&late);
    assert_eq!(stored, fresh);
    let material = 1_242;
    let (late_eval, _) = searcher.compact_eval(&late, stored, material, 0);
    assert_eq!(
        late_eval,
        searcher.compact_eval(&late, fresh, material, 0).0
    );
    let (early_eval, _) = searcher.compact_eval(&early, raw, material, 0);
    assert!(
        late_eval.abs() < early_eval.abs(),
        "{late_eval} vs {early_eval}"
    );
}

#[test]
fn rule50_buckets_cover_the_clock() {
    let bucket = |clock: u8| {
        let fen = format!("4k3/8/8/8/8/8/8/4K3 w - - {clock} 60");
        rule50_bucket(&Board::from_fen(&fen).expect("valid FEN"))
    };
    assert_eq!(bucket(0), 0);
    assert_eq!(bucket(15), 0);
    assert_eq!(bucket(16), 1);
    assert_eq!(bucket(23), 1);
    assert_eq!(bucket(100), 11);
    assert_eq!(bucket(255), RULE50_BUCKETS - 1);
}

/// A positive bonus trained at a position raises the correction read
/// back there and a negative one lowers it, and the corrected eval moves
/// by the correction.
#[test]
fn training_moves_the_correction_toward_the_residual() {
    let mut searcher = Searcher::default();
    let board = Board::from_fen(FEN).expect("valid FEN");
    let (raw, material) = (37, 8_000);
    assert_eq!(searcher.compact_correction(&board, 0), 0);
    let (plain, _) = searcher.compact_eval(&board, raw, material, 0);
    for _ in 0..20 {
        searcher.compact_train_correction(&board, 0, 600);
    }
    let correction = searcher.compact_correction(&board, 0);
    assert!(correction > 0, "{correction}");
    assert_eq!(
        searcher.compact_eval(&board, raw, material, 0),
        (plain + correction, correction)
    );
    for _ in 0..200 {
        searcher.compact_train_correction(&board, 0, -4_000);
    }
    assert!(searcher.compact_correction(&board, 0) < 0);
}

/// Continuation corrections are keyed by the earlier context and the
/// previous move, and read only when both moves exist.
#[test]
fn continuation_corrections_need_both_moves() {
    let mut searcher = Searcher::default();
    let board = Board::from_fen(FEN).expect("valid FEN");
    let slots = searcher.compact_correction_slots(&board, 4);
    assert_eq!(slots.continuation, [None, None]);
    let mv = Move::from_uci("e7e5").expect("valid move");
    searcher.td.stack[3].mv = mv;
    searcher.td.stack[3].piece = Piece::Pawn;
    searcher.td.stack[2].mv = Move::from_uci("g1f3").expect("valid move");
    searcher.td.stack[2].cont_key = 1234 * super::super::history::PIECE_TO_SIZE;
    let slots = searcher.compact_correction_slots(&board, 4);
    assert!(slots.continuation[0].is_some());
    assert_eq!(slots.continuation[1], None, "no move four plies back");
}

/// A draw verdict is the evaluator's classification: tables trained on its
/// own keys leave it at exactly 0, while the same tables move an ordinary
/// raw 0, so the guard is the verdict and not the value.
#[test]
fn a_draw_verdict_takes_no_correction() {
    let mut searcher = Searcher::default();
    // The wrong-bishop rook-pawn draw: a light bishop cannot cover h8.
    let verdict = Board::from_fen("7k/8/6KP/8/8/8/8/5B2 w - - 0 1").expect("valid FEN");
    assert!(crate::eval::endgame::is_draw_verdict(&verdict));
    assert_eq!(searcher.raw_eval(&verdict), 0, "the recogniser's verdict");
    for _ in 0..50 {
        searcher.compact_train_correction(&verdict, 0, 600);
    }
    assert_ne!(
        searcher.compact_correction(&verdict, 0),
        0,
        "the tables are trained"
    );
    assert_eq!(searcher.compact_eval(&verdict, 0, 545, 0), (0, 0));

    let ordinary = Board::from_fen(FEN).expect("valid FEN");
    assert!(!crate::eval::endgame::is_draw_verdict(&ordinary));
    for _ in 0..50 {
        searcher.compact_train_correction(&ordinary, 0, 600);
    }
    let correction = searcher.compact_correction(&ordinary, 0);
    assert_ne!(correction, 0);
    assert_eq!(searcher.compact_eval(&ordinary, 0, 8_000, 0).1, correction);
}

#[test]
fn corrected_eval_stays_out_of_the_tablebase_band() {
    let mut searcher = Searcher::default();
    let board = Board::from_fen(FEN).expect("valid FEN");
    for _ in 0..500 {
        searcher.compact_train_correction(&board, 0, 30_000);
    }
    let raw = crate::search::compact::bounded_raw(TB_WIN_SCORE);
    let (eval, _) = searcher.compact_eval(&board, raw, 8_000, 0);
    assert!(eval < TB_WIN_SCORE);
}
