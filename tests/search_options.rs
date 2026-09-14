//! `SearchOptions`: `position`, `go` and `setoption` parsing, and the UCI option list.

use rarog::board::{Board, Color, Piece, Square};
use rarog::search_options::{GoRequest, OptionUpdate, SearchOptions};

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

fn piece_at(board: &Board, square: Square) -> Option<(Color, Piece)> {
    board.piece_at(square)
}

#[test]
fn search_options_parse_startpos_moves_and_go_limits() {
    let mut options = SearchOptions::default();

    options
        .set_position(&args(&["startpos", "moves", "e2e4", "e7e5", "g1f3"]))
        .expect("valid startpos moves");

    assert_eq!(options.board.side_to_move(), Color::Black);
    assert_eq!(
        piece_at(&options.board, Square::E4),
        Some((Color::White, Piece::Pawn))
    );
    assert_eq!(
        piece_at(&options.board, Square::E5),
        Some((Color::Black, Piece::Pawn))
    );
    assert_eq!(
        piece_at(&options.board, Square::F3),
        Some((Color::White, Piece::Knight))
    );

    options.set_search_parameters(&args(&[
        "wtime",
        "10000",
        "btime",
        "9000",
        "winc",
        "100",
        "binc",
        "200",
        "movestogo",
        "20",
        "nodes",
        "12345",
        "ponder",
        "depth",
        "7",
    ]));

    assert_eq!(options.limits.white_time, 10_000);
    assert_eq!(options.limits.black_time, 9_000);
    assert_eq!(options.limits.white_increment, 100);
    assert_eq!(options.limits.black_increment, 200);
    assert_eq!(options.limits.movestogo, 20);
    assert_eq!(options.limits.nodes, 12_345);
    assert_eq!(options.limits.depth, Some(7));
    assert!(options.limits.search_moves.is_empty());
    assert!(options.limits.ponder);
    assert!(!options.limits.infinite);

    options.set_search_parameters(&args(&["ponder", "infinite"]));
    assert!(options.limits.ponder);
    assert!(options.limits.infinite);
    assert_eq!(options.limits.depth, None);

    options.set_search_parameters(&args(&["depth", "3"]));
    assert!(!options.limits.ponder);
    assert!(!options.limits.infinite);
    assert_eq!(options.limits.depth, Some(3));

    options.set_search_parameters(&args(&["mate", "2", "searchmoves", "e2e4", "g1f3"]));
    assert_eq!(options.limits.depth, Some(3));
    assert_eq!(options.limits.search_moves.len(), 2);
}

#[test]
fn search_options_accept_uppercase_uci_move_text() {
    let mut options = SearchOptions::default();

    options
        .set_position(&args(&["startpos", "moves", "E2E4"]))
        .expect("uppercase UCI move text should be normalized");

    assert_eq!(options.board.side_to_move(), Color::Black);
    assert_eq!(
        piece_at(&options.board, Square::E4),
        Some((Color::White, Piece::Pawn))
    );
}

#[test]
fn search_options_default_go_and_invalid_limits_are_bounded() {
    let mut options = SearchOptions::default();

    assert_eq!(options.set_search_parameters(&[]), GoRequest::Search);

    assert_eq!(options.limits.depth, None);
    assert_eq!(options.limits.nodes, 0);
    assert!(!options.limits.infinite);
    assert!(!options.limits.ponder);

    let request = options.set_search_parameters(&args(&[
        "depth",
        "not-a-number",
        "nodes",
        "bad",
        "movetime",
        "none",
        "wtime",
        "missing",
        "btime",
        "also-bad",
        "movestogo",
        "oops",
    ]));

    assert_eq!(request, GoRequest::Search);
    assert_eq!(options.limits.depth, Some(2));
    assert_eq!(options.limits.nodes, 0);
    assert_eq!(options.limits.move_time, 0);
    assert_eq!(options.limits.white_time, 0);
    assert_eq!(options.limits.black_time, 0);
    assert_eq!(options.limits.movestogo, 0);
}

#[test]
fn search_options_parse_uci_go_perft() {
    let mut options = SearchOptions::default();

    assert_eq!(
        options.set_search_parameters(&args(&["perft", "3"])),
        GoRequest::Perft(3)
    );
    assert_eq!(options.limits.depth, None);
}

#[test]
fn search_options_setoption_and_reset_cover_engine_configuration() {
    let mut options = SearchOptions::default();

    assert_eq!(
        options.set_option(&args(&["name", "Hash", "value", "256"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Move", "Overhead", "value", "25"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Threads", "value", "99"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Ponder", "value", "true"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "SyzygyPath", "value", "C:\\TB\\WDL"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "SyzygyProbeDepth", "value", "6"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "SyzygyProbeLimit", "value", "5"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Syzygy50MoveRule", "value", "false"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Clear", "Hash"])),
        OptionUpdate::ClearHash
    );

    assert_eq!(options.engine.hash_mb, 256);
    assert_eq!(options.engine.move_overhead, 25.0);
    assert_eq!(options.engine.threads, 99);
    assert!(options.engine.ponder);
    assert_eq!(options.engine.syzygy.path, "C:\\TB\\WDL");
    assert_eq!(options.engine.syzygy.probe_depth, 6);
    assert_eq!(options.engine.syzygy.probe_limit, 5);
    assert!(!options.engine.syzygy.fifty_move_rule);

    assert_eq!(
        options.set_option(&args(&["name", "Threads", "value", "9999"])),
        OptionUpdate::Engine
    );
    assert_eq!(options.engine.threads, 1024);

    options.set_search_parameters(&args(&[
        "depth", "4", "nodes", "99", "movetime", "500", "ponder",
    ]));
    options.reset();

    assert_eq!(options.limits.depth, None);
    assert_eq!(options.limits.nodes, 0);
    assert_eq!(options.limits.move_time, 0);
    assert!(options.limits.search_moves.is_empty());
    assert!(!options.limits.ponder);
    assert!(!options.limits.infinite);
    assert_eq!(options.engine.hash_mb, 256);
    assert_eq!(options.engine.move_overhead, 25.0);
    assert_eq!(options.engine.syzygy.path, "C:\\TB\\WDL");

    let names = SearchOptions::get_uci_options().join("\n");
    assert!(names.contains("option name Hash"));
    assert!(names.contains("option name Move Overhead"));
    assert!(names.contains("option name Threads type spin default 1 min 1 max 1024"));
    assert!(names.contains("option name Clear Hash"));
    assert!(names.contains("option name Ponder type check default false"));
    assert!(names.contains("option name SyzygyPath type string default <empty>"));
    assert!(names.contains("option name SyzygyProbeDepth type spin default 1 min 1 max 100"));
    assert!(names.contains("option name SyzygyProbeLimit"));
    assert!(names.contains("option name Syzygy50MoveRule"));
}

#[test]
fn search_options_invalid_setoption_values_preserve_previous_values() {
    let mut options = SearchOptions::default();

    options.set_option(&args(&["name", "Hash", "value", "128"]));
    options.set_option(&args(&["name", "Move", "Overhead", "value", "35"]));
    options.set_option(&args(&["name", "Threads", "value", "4"]));
    options.set_option(&args(&["name", "SyzygyProbeDepth", "value", "8"]));
    options.set_option(&args(&["name", "SyzygyProbeLimit", "value", "5"]));
    options.set_option(&args(&["name", "Syzygy50MoveRule", "value", "false"]));

    assert_eq!(
        options.set_option(&args(&["name", "Hash", "value", "bad"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Move", "Overhead", "value", "nan"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Move", "Overhead", "value", "5001"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Threads", "value", "bad"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "SyzygyProbeDepth", "value", "bad"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "SyzygyProbeLimit", "value", "bad"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Syzygy50MoveRule", "value", "maybe"])),
        OptionUpdate::Engine
    );
    assert_eq!(
        options.set_option(&args(&["name", "Unknown", "Option", "value", "1"])),
        OptionUpdate::Unknown
    );

    assert_eq!(options.engine.hash_mb, 128);
    assert_eq!(options.engine.move_overhead, 35.0);
    assert_eq!(options.engine.threads, 4);
    assert_eq!(options.engine.syzygy.probe_depth, 8);
    assert_eq!(options.engine.syzygy.probe_limit, 5);
    assert!(!options.engine.syzygy.fifty_move_rule);
}

#[test]
fn search_options_clamp_syzygy_values_and_preserve_raw_path() {
    let mut options = SearchOptions::default();
    assert_eq!(options.engine.syzygy.probe_depth, 1);

    options.set_option(&args(&[
        "name",
        "SyzygyPath",
        "value",
        "C:\\TB",
        "Mixed Case",
    ]));
    options.set_option(&args(&["name", "SyzygyProbeDepth", "value", "0"]));
    options.set_option(&args(&["name", "SyzygyProbeLimit", "value", "99"]));
    options.set_option(&args(&["name", "Syzygy50MoveRule", "value", "false"]));

    assert_eq!(options.engine.syzygy.path, "C:\\TB Mixed Case");
    assert_eq!(options.engine.syzygy.probe_depth, 1);
    assert_eq!(options.engine.syzygy.probe_limit, 7);
    assert!(!options.engine.syzygy.fifty_move_rule);

    options.set_option(&args(&["name", "SyzygyProbeDepth", "value", "250"]));
    options.set_option(&args(&["name", "SyzygyProbeLimit", "value", "0"]));
    options.set_option(&args(&["name", "Syzygy50MoveRule", "value", "maybe"]));

    assert_eq!(options.engine.syzygy.probe_depth, 100);
    assert_eq!(options.engine.syzygy.probe_limit, 0);
    assert!(
        !options.engine.syzygy.fifty_move_rule,
        "invalid boolean value must leave the previous setting unchanged"
    );

    options.set_option(&args(&["name", "Syzygy50MoveRule", "value", "true"]));
    assert!(options.engine.syzygy.fifty_move_rule);
}

#[test]
fn search_options_reject_illegal_position_move_without_losing_current_board() {
    let mut options = SearchOptions::default();

    options
        .set_position(&args(&["startpos", "moves", "e2e4"]))
        .expect("valid startpos move");
    let expected = options.board.clone();

    let err = options
        .set_position(&args(&["startpos", "moves", "e2e5"]))
        .expect_err("illegal position move should be reported");

    assert_eq!(err, "Illegal move: e2e5");
    assert_eq!(options.board.hash(), expected.hash());
    assert_eq!(options.board.to_fen(), expected.to_fen());
}

#[test]
fn search_options_accept_little_blitzer_fullmove_zero_fen() {
    let mut options = SearchOptions::default();

    options
        .set_position(&args(&[
            "fen",
            "r1bqkb1r/pppn1ppp/3p1n2/4p1B1/3PP3/2N5/PPP2PPP/R2QKBNR",
            "w",
            "KQkq",
            "e6",
            "0",
            "0",
            "moves",
            "d4d5",
        ]))
        .expect("compatible fullmove-zero FEN");

    assert_eq!(options.board.side_to_move(), Color::Black);
    assert_eq!(
        options.board.piece_at(Square::D5),
        Some((Color::White, Piece::Pawn))
    );
    assert_eq!(options.board.fullmove(), 1);
}
