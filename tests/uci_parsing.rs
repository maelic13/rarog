//! The protocol's parsers: `position`, `go` and `setoption`, and the option
//! list `uci` advertises.

use rarog::board::{Board, Color, Piece, Square};
use rarog::search::options::{EngineOptions, SearchOptions};
use rarog::uci::go::{self, GoRequest};
use rarog::uci::options::{self, OptionUpdate};
use rarog::uci::position;

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

fn piece_at(board: &Board, square: Square) -> Option<(Color, Piece)> {
    board.piece_at(square)
}

fn board_of(parts: &[&str]) -> Board {
    position::parse(&args(parts))
        .expect("a readable position")
        .expect("a position named")
}

fn rejected(name: &str) -> OptionUpdate {
    OptionUpdate::Rejected {
        notice: format!("Invalid {name} value."),
    }
}

/// A clock `go` sends is present whatever its value: a zero, negative or
/// unreadable one is 1 ms, never the absent clock that searches without limit.
#[test]
fn a_sent_clock_is_present_and_at_least_one_millisecond() {
    for (wtime, want) in [("0", 1), ("-50", 1), ("none", 1), ("2500", 2500)] {
        let go = go::parse(&args(&["wtime", wtime, "btime", "1000"]));
        assert_eq!(go.limits.white_time, Some(want), "wtime {wtime}");
        assert_eq!(go.limits.black_time, Some(1000));
    }
    let go = go::parse(&args(&["depth", "3"]));
    assert_eq!(go.limits.white_time, None);
    assert_eq!(go.limits.black_time, None);
}

#[test]
fn startpos_moves_and_go_limits_parse() {
    let board = board_of(&["startpos", "moves", "e2e4", "e7e5", "g1f3"]);

    assert_eq!(board.side_to_move(), Color::Black);
    assert_eq!(
        piece_at(&board, Square::E4),
        Some((Color::White, Piece::Pawn))
    );
    assert_eq!(
        piece_at(&board, Square::E5),
        Some((Color::Black, Piece::Pawn))
    );
    assert_eq!(
        piece_at(&board, Square::F3),
        Some((Color::White, Piece::Knight))
    );

    let go = go::parse(&args(&[
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
    let limits = go.limits;
    assert!(go.notices.is_empty(), "{:?}", go.notices);
    assert_eq!(limits.white_time, Some(10_000));
    assert_eq!(limits.black_time, Some(9_000));
    assert_eq!(limits.white_increment, 100);
    assert_eq!(limits.black_increment, 200);
    assert_eq!(limits.movestogo, 20);
    assert_eq!(limits.nodes, 12_345);
    assert_eq!(limits.depth, Some(7));
    assert!(limits.search_moves.is_empty());
    assert!(limits.ponder);
    assert!(!limits.infinite);

    let limits = go::parse(&args(&["ponder", "infinite"])).limits;
    assert!(limits.ponder);
    assert!(limits.infinite);
    assert_eq!(limits.depth, None);

    let limits = go::parse(&args(&["depth", "3"])).limits;
    assert!(!limits.ponder);
    assert!(!limits.infinite);
    assert_eq!(limits.depth, Some(3));

    let limits = go::parse(&args(&["mate", "2", "searchmoves", "e2e4", "g1f3"])).limits;
    assert_eq!(limits.depth, Some(3));
    assert_eq!(limits.search_moves.len(), 2);
}

#[test]
fn uppercase_uci_move_text_is_accepted() {
    let board = board_of(&["startpos", "moves", "E2E4"]);

    assert_eq!(board.side_to_move(), Color::Black);
    assert_eq!(
        piece_at(&board, Square::E4),
        Some((Color::White, Piece::Pawn))
    );
}

#[test]
fn an_empty_go_searches_and_unreadable_limits_are_bounded_with_a_notice_each() {
    let go = go::parse(&[]);
    assert_eq!(go.request, GoRequest::Search);
    assert_eq!(go.limits.depth, None);
    assert_eq!(go.limits.nodes, 0);
    assert!(!go.limits.infinite);
    assert!(!go.limits.ponder);
    assert!(go.notices.is_empty());

    let go = go::parse(&args(&[
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

    assert_eq!(go.request, GoRequest::Search);
    assert_eq!(go.limits.depth, Some(2));
    assert_eq!(go.limits.nodes, 0);
    assert_eq!(go.limits.move_time, 0);
    // A present clock that does not parse moves at once; it is never "no clock".
    assert_eq!(go.limits.white_time, Some(1));
    assert_eq!(go.limits.black_time, Some(1));
    assert_eq!(go.limits.movestogo, 0);
    // In the order the values are applied, as they were printed.
    assert_eq!(
        go.notices,
        [
            "Invalid movetime value.",
            "Invalid wtime value.",
            "Invalid btime value.",
            "Invalid depth value.",
            "Invalid movestogo value.",
            "Invalid nodes value.",
        ]
    );
}

#[test]
fn go_perft_is_a_perft_request() {
    let go = go::parse(&args(&["perft", "3"]));
    assert_eq!(go.request, GoRequest::Perft(3));
    assert_eq!(go.limits.depth, None);
}

#[test]
fn setoption_and_reset_cover_engine_configuration() {
    let mut engine = EngineOptions::default();
    for parts in [
        &["name", "Hash", "value", "256"][..],
        &["name", "Move", "Overhead", "value", "25"],
        &["name", "Threads", "value", "99"],
        &["name", "Ponder", "value", "true"],
        &["name", "SyzygyPath", "value", "C:\\TB\\WDL"],
        &["name", "SyzygyProbeDepth", "value", "6"],
        &["name", "SyzygyProbeLimit", "value", "5"],
        &["name", "Syzygy50MoveRule", "value", "false"],
    ] {
        // `Hash` says it is the table's size, so the engine thread tries it.
        let want = if parts[1] == "Hash" {
            OptionUpdate::HashApplied
        } else {
            OptionUpdate::Applied
        };
        assert_eq!(options::apply(&mut engine, &args(parts)), want, "{parts:?}");
    }
    assert_eq!(
        options::apply(&mut engine, &args(&["name", "Clear", "Hash"])),
        OptionUpdate::ClearHash
    );

    assert_eq!(engine.hash_mb, 256);
    assert_eq!(engine.move_overhead, 25.0);
    assert_eq!(engine.threads, 99);
    assert!(engine.ponder);
    assert_eq!(engine.syzygy.path, "C:\\TB\\WDL");
    assert_eq!(engine.syzygy.probe_depth, 6);
    assert_eq!(engine.syzygy.probe_limit, 5);
    assert!(!engine.syzygy.fifty_move_rule);

    assert_eq!(
        options::apply(&mut engine, &args(&["name", "Threads", "value", "9999"])),
        OptionUpdate::Applied
    );
    assert_eq!(engine.threads, 1024);

    let mut search = SearchOptions {
        engine,
        limits: go::parse(&args(&[
            "depth", "4", "nodes", "99", "movetime", "500", "ponder",
        ]))
        .limits,
        ..SearchOptions::default()
    };
    search.reset();

    assert_eq!(search.limits.depth, None);
    assert_eq!(search.limits.nodes, 0);
    assert_eq!(search.limits.move_time, 0);
    assert!(search.limits.search_moves.is_empty());
    assert!(!search.limits.ponder);
    assert!(!search.limits.infinite);
    assert_eq!(search.engine.hash_mb, 256);
    assert_eq!(search.engine.move_overhead, 25.0);
    assert_eq!(search.engine.syzygy.path, "C:\\TB\\WDL");

    let names = options::advertised().join("\n");
    assert!(names.contains("option name Hash"));
    assert!(names.contains("option name Move Overhead"));
    assert!(names.contains("option name Threads type spin default 1 min 1 max 1024"));
    assert!(names.contains("option name MultiPV type spin default 1 min 1 max 256"));
    assert!(names.contains("option name Clear Hash"));
    assert!(names.contains("option name Ponder type check default false"));
    assert!(names.contains("option name SyzygyPath type string default <empty>"));
    assert!(names.contains("option name SyzygyProbeDepth type spin default 1 min 1 max 100"));
    assert!(names.contains("option name SyzygyProbeLimit"));
    assert!(names.contains("option name Syzygy50MoveRule"));
}

#[test]
fn invalid_setoption_values_keep_the_previous_values_and_say_so() {
    let mut engine = EngineOptions::default();
    for parts in [
        &["name", "Hash", "value", "128"][..],
        &["name", "Move", "Overhead", "value", "35"],
        &["name", "Threads", "value", "4"],
        &["name", "SyzygyProbeDepth", "value", "8"],
        &["name", "SyzygyProbeLimit", "value", "5"],
        &["name", "Syzygy50MoveRule", "value", "false"],
    ] {
        options::apply(&mut engine, &args(parts));
    }

    for (parts, name) in [
        (&["name", "Hash", "value", "bad"][..], "Hash"),
        (&["name", "Hash", "value", "-5"], "Hash"),
        (
            &["name", "Move", "Overhead", "value", "nan"],
            "Move Overhead",
        ),
        (
            &["name", "Move", "Overhead", "value", "5001"],
            "Move Overhead",
        ),
        (&["name", "Threads", "value", "bad"], "Threads"),
        (
            &["name", "SyzygyProbeDepth", "value", "bad"],
            "SyzygyProbeDepth",
        ),
        (
            &["name", "SyzygyProbeLimit", "value", "bad"],
            "SyzygyProbeLimit",
        ),
        (
            &["name", "Syzygy50MoveRule", "value", "maybe"],
            "Syzygy50MoveRule",
        ),
    ] {
        assert_eq!(
            options::apply(&mut engine, &args(parts)),
            rejected(name),
            "{parts:?}"
        );
    }
    assert_eq!(
        options::apply(
            &mut engine,
            &args(&["name", "Unknown", "Option", "value", "1"])
        ),
        OptionUpdate::Unknown {
            name: String::from("Unknown Option")
        }
    );

    assert_eq!(engine.hash_mb, 128);
    assert_eq!(engine.move_overhead, 35.0);
    assert_eq!(engine.threads, 4);
    assert_eq!(engine.syzygy.probe_depth, 8);
    assert_eq!(engine.syzygy.probe_limit, 5);
    assert!(!engine.syzygy.fifty_move_rule);
}

#[test]
fn multipv_clamps_and_ignores_invalid_values() {
    let mut engine = EngineOptions::default();
    assert_eq!(engine.multi_pv, 1);

    assert_eq!(
        options::apply(&mut engine, &args(&["name", "MultiPV", "value", "4"])),
        OptionUpdate::Applied
    );
    assert_eq!(engine.multi_pv, 4);
    options::apply(&mut engine, &args(&["name", "MultiPV", "value", "bad"]));
    assert_eq!(engine.multi_pv, 4);
    options::apply(&mut engine, &args(&["name", "MultiPV", "value", "0"]));
    assert_eq!(engine.multi_pv, 1);
    options::apply(&mut engine, &args(&["name", "MultiPV", "value", "9999"]));
    assert_eq!(engine.multi_pv, 256);
}

#[test]
fn syzygy_values_clamp_and_the_path_keeps_its_case() {
    let mut engine = EngineOptions::default();
    assert_eq!(engine.syzygy.probe_depth, 1);

    options::apply(
        &mut engine,
        &args(&["name", "SyzygyPath", "value", "C:\\TB", "Mixed Case"]),
    );
    options::apply(
        &mut engine,
        &args(&["name", "SyzygyProbeDepth", "value", "0"]),
    );
    options::apply(
        &mut engine,
        &args(&["name", "SyzygyProbeLimit", "value", "99"]),
    );
    options::apply(
        &mut engine,
        &args(&["name", "Syzygy50MoveRule", "value", "false"]),
    );

    assert_eq!(engine.syzygy.path, "C:\\TB Mixed Case");
    assert_eq!(engine.syzygy.probe_depth, 1);
    assert_eq!(engine.syzygy.probe_limit, 7);
    assert!(!engine.syzygy.fifty_move_rule);

    // A signed field holds a negative value to its range.
    options::apply(
        &mut engine,
        &args(&["name", "SyzygyProbeDepth", "value", "-3"]),
    );
    assert_eq!(engine.syzygy.probe_depth, 1);
    options::apply(
        &mut engine,
        &args(&["name", "SyzygyProbeDepth", "value", "250"]),
    );
    options::apply(
        &mut engine,
        &args(&["name", "SyzygyProbeLimit", "value", "0"]),
    );
    options::apply(
        &mut engine,
        &args(&["name", "Syzygy50MoveRule", "value", "maybe"]),
    );

    assert_eq!(engine.syzygy.probe_depth, 100);
    assert_eq!(engine.syzygy.probe_limit, 0);
    assert!(
        !engine.syzygy.fifty_move_rule,
        "invalid boolean value must leave the previous setting unchanged"
    );

    options::apply(
        &mut engine,
        &args(&["name", "Syzygy50MoveRule", "value", "true"]),
    );
    assert!(engine.syzygy.fifty_move_rule);
}

#[test]
fn an_illegal_position_move_is_reported() {
    let err = position::parse(&args(&["startpos", "moves", "e2e5"]))
        .err()
        .expect("illegal position move should be reported");
    assert_eq!(err, "Illegal move: e2e5");
    assert_eq!(
        position::parse(&args(&["fen", "invalid"]))
            .err()
            .expect("an unreadable FEN"),
        "Invalid FEN."
    );
    assert!(position::parse(&[]).expect("nothing to read").is_none());
    assert!(
        position::parse(&args(&["sideways"]))
            .expect("not a position")
            .is_none()
    );
}

#[test]
fn a_little_blitzer_fullmove_zero_fen_is_accepted() {
    let board = board_of(&[
        "fen",
        "r1bqkb1r/pppn1ppp/3p1n2/4p1B1/3PP3/2N5/PPP2PPP/R2QKBNR",
        "w",
        "KQkq",
        "e6",
        "0",
        "0",
        "moves",
        "d4d5",
    ]);

    assert_eq!(board.side_to_move(), Color::Black);
    assert_eq!(
        board.piece_at(Square::D5),
        Some((Color::White, Piece::Pawn))
    );
    assert_eq!(board.fullmove(), 1);
}

/// A GUI echoes back the `<empty>` default the engine advertises for a string
/// option - CuteChess does (GitHub issue maelic13/rarog#1) - and taking it
/// literally made `SyzygyPath` search a folder called `<empty>` and report
/// "loaded no usable tablebases" at the start of every game.
#[test]
fn the_empty_placeholder_sets_a_string_option_to_nothing() {
    let mut engine = EngineOptions::default();

    assert_eq!(
        options::apply(
            &mut engine,
            &args(&["name", "SyzygyPath", "value", r"C:\TB\WDL"])
        ),
        OptionUpdate::Applied
    );
    assert_eq!(engine.syzygy.path, r"C:\TB\WDL");

    assert_eq!(
        options::apply(
            &mut engine,
            &args(&["name", "SyzygyPath", "value", "<empty>"])
        ),
        OptionUpdate::Applied
    );
    assert_eq!(
        engine.syzygy.path, "",
        "the advertised placeholder means no path at all"
    );

    // A path that merely contains the token is still a path.
    assert_eq!(
        options::apply(
            &mut engine,
            &args(&["name", "SyzygyPath", "value", r"C:\<empty>"])
        ),
        OptionUpdate::Applied
    );
    assert_eq!(engine.syzygy.path, r"C:\<empty>");
}

#[cfg(windows)]
#[test]
fn large_pages_is_a_check_option_that_defaults_on() {
    let names = options::advertised().join("\n");
    assert!(names.contains("option name LargePages type check default true"));

    let mut engine = EngineOptions::default();
    assert!(engine.large_pages);
    assert_eq!(
        options::apply(
            &mut engine,
            &args(&["name", "LargePages", "value", "false"])
        ),
        OptionUpdate::Applied
    );
    assert!(!engine.large_pages);
    assert_eq!(
        options::apply(
            &mut engine,
            &args(&["name", "LargePages", "value", "maybe"])
        ),
        rejected("LargePages")
    );
    assert!(!engine.large_pages, "an invalid value keeps the old one");
    options::apply(&mut engine, &args(&["name", "largepages", "value", "TRUE"]));
    assert!(engine.large_pages);
}

#[cfg(not(windows))]
#[test]
fn large_pages_is_not_offered_without_the_page_allocator() {
    assert!(!options::advertised().join("\n").contains("LargePages"));
    let mut engine = EngineOptions::default();
    assert_eq!(
        options::apply(
            &mut engine,
            &args(&["name", "LargePages", "value", "false"])
        ),
        OptionUpdate::Unknown {
            name: String::from("LargePages")
        }
    );
}
