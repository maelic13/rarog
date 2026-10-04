//! Search behaviour through the public API: root edge cases, `searchmoves`, node limits, quit and ponderhit, disabled tablebases.

use rarog::board::{Board, Move};
use rarog::search::{SearchEvent, SearchExit, Searcher};
use rarog::search_options::SearchOptions;
use rarog::syzygy;

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

#[test]
fn syzygy_disabled_path_leaves_tablebase_probes_unavailable() {
    let board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");

    assert_eq!(syzygy::initialize(""), 0);
    assert_eq!(syzygy::largest(), 0);
    assert!(syzygy::probe_wdl(&board, true).is_none());
    assert!(syzygy::probe_root(&board, true).is_none());
}

#[test]
fn search_returns_null_move_for_stalemate() {
    let board = Board::from_fen("4k3/4P3/4K3/8/8/8/8/8 b - - 0 1").expect("valid FEN");
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(4);

    let result = searcher.search(board, &options, false, || SearchEvent::None);

    assert_eq!(result.bestmove, Move::NULL);
    assert_eq!(result.pondermove, Move::NULL);
    assert_eq!(result.depth, 0);
    assert_eq!(result.score, 0);
    assert_eq!(result.tb_hits, 0);
    assert_eq!(result.exit, SearchExit::Stop);
}

#[test]
fn search_returns_legal_root_move_in_drawn_material_positions() {
    for fen in [
        "8/8/8/8/8/8/4K3/6k1 w - - 0 1",
        "7k/8/8/8/8/8/4KN2/8 w - - 0 1",
        "7k/8/8/8/8/8/4KB2/8 w - - 0 1",
    ] {
        let board = Board::from_fen(fen).expect("valid draw FEN");
        let legal_moves = board.generate_legal_movelist();
        let mut searcher = Searcher::default();
        let mut options = SearchOptions::default();
        options.limits.depth = Some(1);

        let result = searcher.search(board, &options, false, || SearchEvent::None);

        assert_eq!(result.score, 0, "{fen}");
        assert_ne!(result.bestmove, Move::NULL, "{fen}");
        assert!(
            legal_moves
                .iter()
                .any(|&legal_move| legal_move.same_uci_move(result.bestmove)),
            "{} must be legal for {fen}",
            result.bestmove
        );
    }
}

#[test]
fn search_returns_legal_move_in_root_fifty_move_claim_position() {
    let board =
        Board::from_fen("8/8/7k/8/1N6/1K6/4r3/8 w - - 100 1").expect("valid fifty-move claim FEN");
    let legal_moves = board.generate_legal_movelist();
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(1);

    let result = searcher.search(board, &options, false, || SearchEvent::None);

    assert_ne!(result.bestmove, Move::NULL);
    assert_eq!(result.score, 0);
    assert!(
        legal_moves
            .iter()
            .any(|&legal_move| legal_move.same_uci_move(result.bestmove)),
        "{} must be legal",
        result.bestmove
    );
}

#[test]
fn search_returns_legal_moves_from_little_blitzer_illegal_artifacts() {
    for fen in [
        "4k3/p5QR/1p2p3/6p1/8/PP1q4/8/2K5 b - - 4 0",
        "8/6R1/5P1k/8/2PB1KP1/r7/3r4/8 w - - 7 0",
    ] {
        let board = Board::from_fen(fen).expect("valid artifact FEN");
        let legal_moves = board.generate_legal_movelist();
        let mut searcher = Searcher::default();
        let mut options = SearchOptions::default();
        options.limits.depth = Some(4);

        let result = searcher.search(board, &options, false, || SearchEvent::None);

        assert_ne!(result.bestmove, Move::NULL, "{fen}");
        assert!(
            legal_moves
                .iter()
                .any(|&legal_move| legal_move.same_uci_move(result.bestmove)),
            "{} must be legal for {fen}",
            result.bestmove
        );
    }
}

#[test]
fn search_respects_searchmoves_root_filter() {
    let board = Board::default();
    let forced = board.parse_move("a2a3").expect("legal root move");
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.set_search_parameters(&args(&["depth", "2", "searchmoves", "a2a3"]));

    let result = searcher.search(board, &options, false, || SearchEvent::None);

    assert_eq!(result.bestmove, forced);
}

#[test]
fn search_uses_matching_searchmoves_when_some_requested_moves_are_illegal() {
    let board = Board::default();
    let forced = board.parse_move("a2a3").expect("legal root move");
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.set_search_parameters(&args(&["depth", "2", "searchmoves", "a7a6", "a2a3"]));

    let result = searcher.search(board, &options, false, || SearchEvent::None);

    assert_eq!(result.bestmove, forced);
}

#[test]
fn search_falls_back_when_searchmoves_match_no_root_move() {
    let board = Board::default();
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.set_search_parameters(&args(&["depth", "1", "searchmoves", "a7a6"]));

    let result = searcher.search(board, &options, false, || SearchEvent::None);

    assert_ne!(result.bestmove, Move::NULL);
    assert_eq!(result.depth, 1);
}

#[test]
fn search_respects_node_limit() {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(99);
    options.limits.nodes = 512;

    let result = searcher.search(options.board.clone(), &options, false, || SearchEvent::None);

    assert_eq!(result.exit, SearchExit::Stop);
    assert!(result.nodes >= 512, "nodes: {}", result.nodes);
    assert!(result.nodes <= 2_048, "nodes: {}", result.nodes);
    assert!(result.depth < 99);
}

#[test]
fn threaded_search_uses_aggregate_node_limit() {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(99);
    options.limits.nodes = 512;
    options.engine.threads = 8;

    let result = searcher.search(options.board.clone(), &options, false, || SearchEvent::None);

    assert_eq!(result.exit, SearchExit::Stop);
    assert!(result.nodes >= 512, "nodes: {}", result.nodes);
    assert!(
        result.nodes <= 2_048,
        "threaded node-limited search should not multiply the limit: {}",
        result.nodes
    );
    assert!(result.depth < 99);
}

#[test]
fn search_quit_event_exits_search() {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(99);

    let mut polls = 0;
    let result = searcher.search(options.board.clone(), &options, false, || {
        polls += 1;
        SearchEvent::Quit
    });

    assert_eq!(result.exit, SearchExit::Quit);
    assert!(polls > 0);
    assert!(result.nodes >= 512, "nodes: {}", result.nodes);
    assert!(result.depth < 99);
}

#[test]
fn search_result_records_ponderhit_conversion() {
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(99);
    options.limits.nodes = 4_096;
    options.limits.ponder = true;

    let mut polls = 0;
    let result = searcher.search(options.board.clone(), &options, false, || {
        polls += 1;
        if polls == 1 {
            SearchEvent::PonderHit
        } else {
            SearchEvent::None
        }
    });

    assert_eq!(result.exit, SearchExit::Stop);
    assert!(result.ponderhit);
    assert!(polls > 0);
    assert!(result.nodes >= 4_096, "nodes: {}", result.nodes);
    assert!(result.depth < 99);
}
