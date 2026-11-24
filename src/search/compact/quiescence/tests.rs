use super::super::super::node::{NonPv, Pv};
use super::*;

fn search<NODE: NodeType>(fen: &str, alpha: i32, beta: i32) -> (Searcher, Board, i32) {
    let mut searcher = Searcher::default();
    let mut board = Board::from_fen(fen).expect("a legal FEN");
    let score = searcher
        .compact_quiescence::<NODE, _>(&mut board, alpha, beta, 1, &mut || SearchEvent::None);
    (searcher, board, score)
}

#[test]
fn a_hanging_queen_is_taken_and_a_mate_is_scored() {
    // White to move takes the queen on d5.
    let fen = "4k3/8/8/3q4/8/8/3R4/4K3 w - - 0 1";
    let (mut searcher, board, score) = search::<Pv>(fen, -INF_SCORE, INF_SCORE);
    let raw = bounded_raw(searcher.raw_eval(&board));
    let material = kernel_material(&board, searcher.exchange_values());
    let (stand_pat, _) = searcher.compact_eval(&board, raw, material, 1);
    // The capture wins the queen: a lost position becomes a won one, a
    // swing well past the deficit, whatever scale the wrapper gives.
    assert!(
        stand_pat < 0 && score > 0 && score - stand_pat > 3 * -stand_pat / 2,
        "{score} after Rxd5 from {stand_pat}"
    );
    assert_eq!(searcher.td.pv_table[1][1].to_string(), "d2d5");

    // Black is mated: in check with no move.
    let (_, _, mated) = search::<NonPv>(
        "4k3/4Q3/4K3/8/8/8/8/8 b - - 0 1",
        -INF_SCORE,
        -INF_SCORE + 1,
    );
    assert_eq!(mated, -MATE_SCORE + 1);
}

#[test]
fn a_stand_pat_at_beta_is_softened_and_stored_as_a_lower_bound() {
    // White, two queens up, stands far above a beta of -100.
    let fen = "k7/8/8/8/8/8/2QQ4/4K3 w - - 0 1";
    let (mut searcher, board, score) = search::<NonPv>(fen, -101, -100);
    let raw = bounded_raw(searcher.raw_eval(&board));
    let material = kernel_material(&board, searcher.exchange_values());
    let (stand_pat, _) = searcher.compact_eval(&board, raw, material, 1);
    assert_eq!(
        score,
        rules::q_stand_pat_value(rules::SEEDS, stand_pat, -100)
    );
    assert!(
        score > -100 && score < stand_pat,
        "{score} between beta and {stand_pat}"
    );
    let entry = searcher.shared.tt.probe(board.hash()).expect("stored");
    assert_eq!(entry.bound(), Some(Bound::Lower));
    assert_eq!(i32::from(entry.depth), Q_DEPTH);
}

/// Quiescence searches a quiet queen promotion as a noisy move: with no
/// capture on the board, a push to a free queening square raises the
/// score above the stand pat and heads the line, and a push to a square a
/// rook guards does neither.
#[test]
fn a_free_queen_push_is_searched_and_a_guarded_one_gains_nothing() {
    let stand_pat_of = |searcher: &mut Searcher, board: &Board| {
        let raw = bounded_raw(searcher.raw_eval(board));
        let material = kernel_material(board, searcher.exchange_values());
        searcher.compact_eval(board, raw, material, 1).0
    };

    let free = "8/P6k/8/8/8/8/8/K7 w - - 0 1";
    let (mut searcher, board, score) = search::<Pv>(free, -INF_SCORE, INF_SCORE);
    let stand_pat = stand_pat_of(&mut searcher, &board);
    assert_eq!(searcher.td.pv_table[1][1].to_string(), "a7a8q");
    assert!(score > stand_pat, "{score} after a8=Q from {stand_pat}");

    let guarded = "7r/P6k/8/8/8/8/8/K7 w - - 0 1";
    let (mut searcher, board, score) = search::<Pv>(guarded, -INF_SCORE, INF_SCORE);
    let stand_pat = stand_pat_of(&mut searcher, &board);
    assert_ne!(searcher.td.pv_table[1][1].to_string(), "a7a8q");
    assert!(
        score <= stand_pat,
        "{score} with the push guarded, from {stand_pat}"
    );
}

/// Quiescence after `uci` is played from `fen`: the move is put on the
/// stack as the search puts it, so the child sees what it captured.
fn after(fen: &str, uci: &str) -> (Searcher, Board, i32) {
    let mut searcher = Searcher::default();
    let mut board = Board::from_fen(fen).expect("a legal FEN");
    let mv = board.parse_move(uci).expect("a legal move");
    let piece = board.moving_piece(mv);
    searcher.compact_push_move(&board, 0, mv, piece, 1);
    board.make_move(mv);
    let score =
        searcher.compact_quiescence::<Pv, _>(&mut board, -INF_SCORE, INF_SCORE, 1, &mut || {
            SearchEvent::None
        });
    (searcher, board, score)
}

/// A side left with king and blocked pawns once its last rook is taken
/// is stalemated, and quiescence reads the draw, not the material; with
/// a pawn free to push it is not, and quiescence keeps its stand pat.
#[test]
fn a_stalemate_after_losing_the_last_piece_reads_as_a_draw() {
    // Rxh2: Black's king on a8 has no square and its a7 pawn is blocked.
    let (_, board, score) = after("k7/p1K5/P7/8/8/8/7r/7R w - - 0 1", "h1h2");
    assert!(board.generate_legal_movelist().is_empty() && !board.is_in_check());
    assert!(score.abs() <= 1, "a stalemate read as {score}");

    // The same with a black pawn free on g7: Black can move, so no draw.
    let (_, board, score) = after("k7/p1K3p1/P7/8/8/8/7r/7R w - - 0 1", "h1h2");
    assert!(!board.generate_legal_movelist().is_empty());
    assert!(score < -100, "the stand pat was {score}");
}

#[test]
fn quiescence_stops_before_ply_stack_overflow() {
    let mut searcher = Searcher::default();
    let mut board = Board::from_fen("4k3/8/8/8/3q4/8/8/4KQ2 w - - 0 1").expect("valid FEN");
    let before = board.to_fen();
    let score = searcher.compact_quiescence::<Pv, _>(
        &mut board,
        -INF_SCORE,
        INF_SCORE,
        MAX_PLY - 1,
        &mut || SearchEvent::None,
    );
    assert_eq!(board.to_fen(), before);
    assert!(score.abs() < INF_SCORE);
}
