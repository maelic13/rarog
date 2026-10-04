use rarog::board::{Board, Move};
use rarog::eval::MATE_SCORE;
use rarog::search::{SearchEvent, SearchResult, Searcher};
use rarog::search_options::SearchOptions;

#[test]
fn search_finds_fools_mate_in_one() {
    let board =
        Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2").unwrap();

    let result = search_at_depth(board, 1);
    assert_eq!(result.to_string(), "d8h4");
}

#[test]
fn threaded_search_finds_fools_mate_in_one() {
    let board =
        Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2").unwrap();

    let result = search_at_depth_with_threads(board, 1, 2);
    assert_eq!(result.to_string(), "d8h4");
}

/// KQ vs K must be PROVEN as a forced mate of sane length.
///
/// Rewritten 2026-07-19 after it turned out to be a zero-margin tripwire. It
/// asserted `score >= MATE_SCORE - 9`; production scored exactly 31,991 —
/// mate in 9 plies, passing with NO headroom at all. Any eval change costing
/// a single ply of mate distance broke the build. Disabling lazy eval (what
/// `--features texel` does) scores 31,989, mate in 11 plies — still a
/// perfectly correct forced win, one move slower — and the test failed.
///
/// That is exactly the shape-vs-correctness mistake the 2026-07-16 Basilisk
/// canary post-mortem called out; the earlier pass relaxed the `depth ==`
/// assertion but left the mate-distance one just as tight. The real property
/// is "the search proves a forced mate and does not wander": KQvK is at most
/// 10 moves from any position with correct play, so a bound of 8 moves still
/// catches genuine breakage (no mate proven, or a mate 20 moves out) while
/// tolerating the ±1-move wobble any legitimate eval change can cause.
///
/// Measured at depth 18: 5 moves with lazy eval on (shipped), 6 with it off.
#[test]
fn search_continues_to_resolve_shorter_mate() {
    let board = Board::from_fen("4K3/2Q5/6k1/8/8/8/8/8 w - - 0 1").unwrap();

    let result = search_result_at_depth_with_threads(board, 18, 1);

    assert!(
        result.depth >= 9,
        "search aborted early: depth {}",
        result.depth
    );
    assert!(
        result.score > MATE_SCORE - 1_000,
        "no forced mate proven in KQ vs K: score {}",
        result.score
    );
    let mate_in = mate_in_from_score(result.score);
    assert!(
        mate_in <= 8,
        "mate found but implausibly distant for KQ vs K: mate in {mate_in}"
    );
    assert!(!result.bestmove.is_null());
}

/// Robust mate-recognition canary (Basilisk search-doc §14 lesson, adopted
/// 2026-07-16): legal positions a few moves from mate must return a mate score
/// at a moderate depth. Unlike a fixed-depth *conversion trajectory*, this does
/// not depend on the exact search path, so it stays stable across benign
/// eval/search/TT changes while still catching lost mate-finding or — crucially
/// — a stalemate blunder (the winning side accepting a draw it could avoid).
#[test]
fn near_mate_recognition_canary() {
    // (fen, max mate plies the score must prove, note)
    let cases: &[(&str, i32, &str)] = &[
        ("k7/7Q/2K5/8/8/8/8/8 w - - 0 1", 2, "KQK: mate in 1"),
        // The winning side must find the mate and NOT drift into a stalemate:
        // with Ka8/Kc6/Qb1, Qb6?? stalemates but Qb7# mates.
        (
            "k7/8/2K5/8/8/8/8/1Q6 w - - 0 1",
            4,
            "KQK stalemate trap: mate, not stalemate",
        ),
        ("k7/2K5/8/8/8/8/8/7R w - - 0 1", 2, "KRK: mate in 1"),
        ("7k/8/6K1/8/8/8/8/R7 w - - 0 1", 2, "KRK: mate in 1 (Ra8#)"),
        (
            "k7/2K5/8/3N4/4B3/8/8/8 w - - 0 1",
            24,
            "KBNK: mate within a few moves",
        ),
    ];
    for &(fen, max_plies, note) in cases {
        let board = Board::from_fen(fen).unwrap_or_else(|e| panic!("[{note}] bad FEN: {e}"));
        let result = search_result_at_depth_with_threads(board, 20, 1);
        assert!(
            result.score >= MATE_SCORE - max_plies,
            "[{note}] expected a mate score within {max_plies} plies, got {} (depth {}) — \
             lost mate-finding or a stalemate blunder",
            result.score,
            result.depth
        );
        assert!(!result.bestmove.is_null(), "[{note}] null best move");
    }
}

#[test]
fn search_prefers_winning_hanging_queen() {
    let board = Board::from_fen("4k3/8/8/8/3q4/2N1B3/8/4K3 w - - 0 1").unwrap();

    let result = search_at_depth(board, 2);
    assert_eq!(result.to_string(), "e3d4");
}

#[test]
fn search_prefers_safer_interposition_from_sampled_loss_line() {
    let board =
        Board::from_fen("rnbqkbnr/ppp2ppp/8/1B1P4/4p3/5N2/PPPP1PPP/RNBQK2R b KQkq - 1 4").unwrap();

    let result = search_at_depth(board, 4);
    assert_eq!(result.to_string(), "c7c6");
}

#[test]
fn fixed_depth_single_thread_search_is_repeatable() {
    let board = Board::default();
    let first = search_result_at_depth_with_threads(board.clone(), 4, 1);
    let second = search_result_at_depth_with_threads(board, 4, 1);

    assert_eq!(first.bestmove, second.bestmove);
    assert_eq!(first.score, second.score);
    assert_eq!(first.depth, second.depth);
    assert_eq!(first.nodes, second.nodes);
}

#[test]
fn searcher_handles_thread_count_changes() {
    let board =
        Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2").unwrap();
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(4);

    options.board = board.clone();
    options.engine.threads = 4;
    let threaded = searcher.search(board.clone(), &options, false, || SearchEvent::None);

    options.board = board.clone();
    options.engine.threads = 1;
    let single = searcher.search(board, &options, false, || SearchEvent::None);

    assert_eq!(threaded.bestmove.to_string(), "d8h4");
    assert_eq!(single.bestmove.to_string(), "d8h4");
}

fn search_at_depth(board: Board, depth: u32) -> Move {
    search_at_depth_with_threads(board, depth, 1)
}

fn search_result_at_depth_with_threads(board: Board, depth: u32, threads: usize) -> SearchResult {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions {
        board: board.clone(),
        ..SearchOptions::default()
    };
    options.limits.depth = Some(depth);
    options.engine.threads = threads;
    searcher.search(board, &options, false, || SearchEvent::None)
}

fn search_at_depth_with_threads(board: Board, depth: u32, threads: usize) -> Move {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions {
        board: board.clone(),
        ..SearchOptions::default()
    };
    options.limits.depth = Some(depth);
    options.engine.threads = threads;
    let result = searcher.search(board, &options, false, || SearchEvent::None);
    result.bestmove
}

fn mate_in_from_score(score: i32) -> i32 {
    (MATE_SCORE - score.abs() + 1) / 2
}
