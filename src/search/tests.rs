/// The evaluator mirrors `MAX_PLY` to bound its mop-up drive below the mate
/// band without taking a dependency on the search. This is the assertion
/// that keeps the mirror honest, and it lives here because this module is
/// the one that legitimately sees both constants.
#[test]
fn mopup_mirror_matches_the_real_ply_horizon() {
    assert_eq!(
        crate::eval::MOPUP_ASSUMED_MAX_PLY as usize,
        super::MAX_PLY,
        "eval.rs mirrors MAX_PLY to bound the mop-up drive; the two have \
             drifted, so the compile-time bound in eval.rs is now checking \
             the wrong number"
    );
}

use super::*;
use crate::eval::VALUE_NONE;
use crate::tt::{Bound, TtStore};

/// A tuning build's compact seeds reach the compact search: a seed set
/// through the options changes its tree, its own value leaves the tree
/// alone, and a searcher given new values and then the old ones searches
/// the old tree again.
#[cfg(feature = "tune")]
#[test]
fn a_compact_seed_set_in_the_options_reaches_the_compact_search() {
    const FEN: &str = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
    let options = |value: &str| {
        let mut options = SearchOptions::default();
        options.limits.depth = Some(8);
        assert!(options.engine.set_compact_seed("KLmrLog", value));
        options
    };
    let tree = |searcher: &mut Searcher, options: &SearchOptions| {
        let board = Board::from_fen(FEN).expect("valid FEN");
        searcher
            .search(board, options, false, || SearchEvent::None)
            .nodes
    };
    let mut donor = SearchOptions::default();
    donor.limits.depth = Some(8);
    let base = tree(&mut Searcher::default(), &donor);
    let own = compact::rules::SEEDS.lmr_log.to_string();
    assert_eq!(tree(&mut Searcher::default(), &options(&own)), base);
    let moved = tree(&mut Searcher::default(), &options("400"));
    assert_ne!(
        moved, base,
        "a longer reduction table searches another tree"
    );
    let mut reused = Searcher::default();
    assert_eq!(tree(&mut reused, &options("400")), moved);
    reused.new_game();
    assert_eq!(tree(&mut reused, &donor), base);
}

/// Every search parameter is live: set to either end of its declared
/// range, it moves the tree of a fixed-depth search on a named position,
/// or, for a clock parameter, the time limits or the soft stop on a named
/// clock. The SPSA coverage audit reads the source; this reads behaviour,
/// so a parameter the search no longer consults, or consults only where no
/// named position reaches, fails here by name. The kernel's seeds are not
/// search parameters: a tuning build's sets them (see the seed test above).
/// Release only: the sweep is about 25 searches at depth 8.
#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "about 25 searches at depth 8; the release suite runs it"
)]
fn every_search_parameter_moves_what_it_tunes() {
    const DEPTH: u32 = 8;
    const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    const SEMI_SLAV: &str = "rnbqkb1r/pp3ppp/2p1pn2/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq - 0 5";
    // A knight up.
    const KNIGHT_UP: &str = "r1bqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // Two pieces a side, and a rule-50 clock under way.
    const TWO_PIECES: &str = "2r3k1/pp1n1ppp/4p3/3pP3/3P1P2/2RB2P1/PP5P/6K1 w - - 40 30";
    const PAWN_ENDING: &str = "8/5pk1/6p1/8/4P3/5PK1/8/8 w - - 10 50";
    const POSITIONS: [&str; 5] = [KIWIPETE, SEMI_SLAV, KNIGHT_UP, TWO_PIECES, PAWN_ENDING];
    // Proved live by a test of its own: a root fail-high stops the search
    // at the low end of the range and not at the high end.
    const ELSEWHERE: [(&str, &str); 1] = [(
        "TmFailHighTarget",
        "a_root_fail_high_past_the_soft_target_plays_the_move_with_its_lower_bound",
    )];
    let tree = |options: &SearchOptions, fen: &str, depth: u32| {
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut options = options.clone();
        options.limits.depth = Some(depth);
        let mut searcher = Searcher::default();
        let result = searcher.search(board, &options, false, || SearchEvent::None);
        (result.nodes, result.score)
    };
    let clocks = [
        (60_000, 600, 0, 0_u32),
        (2_000, 20, 0, 60),
        (30_000, 0, 20, 20),
    ];
    // Early in a search with the score inside the trend's clamps; late,
    // with the best move and score long held, most of the nodes on the
    // move and the score risen; and a spread search after a large fall.
    let signals = [
        IterationSignals {
            node_share: 0.6,
            score_fall: 2.0,
            pv_stability: 3,
            eval_stability: 2,
            best_move_changes: 1,
        },
        IterationSignals {
            node_share: 0.97,
            score_fall: -10.0,
            pv_stability: 20,
            eval_stability: 20,
            best_move_changes: 0,
        },
        IterationSignals {
            node_share: 0.3,
            score_fall: 25.0,
            pv_stability: 0,
            eval_stability: 0,
            best_move_changes: 3,
        },
    ];
    let clock = |options: &SearchOptions| -> Vec<u64> {
        let mut view = Vec::new();
        for (time, increment, movestogo, game_ply) in clocks {
            let limits = SearchLimits {
                white_time: Some(time),
                black_time: Some(time),
                white_increment: increment,
                black_increment: increment,
                movestogo,
                ..SearchLimits::default()
            };
            let runtime = time::compute_runtime_limits(
                &limits,
                &options.engine,
                Color::White,
                game_ply,
                MAX_DEPTH,
            );
            for value in [
                runtime.optimum_ms,
                runtime.maximum_ms,
                runtime.tb_extension_latest_start_ms,
            ] {
                view.push(value.to_bits());
            }
            for signals in signals {
                let soft = time::soft_stop(&options.engine.search_params, &runtime, signals);
                view.push(soft.target_ms.to_bits());
            }
        }
        view
    };
    let base = SearchOptions::default();
    let base_trees: Vec<_> = POSITIONS
        .iter()
        .map(|fen| tree(&base, fen, DEPTH))
        .collect();
    let base_clock = clock(&base);
    let mut visited = Vec::new();
    let mut dead = Vec::new();
    let mut visit = |name: &str, min: i32, max: i32, set: &dyn Fn(&mut SearchOptions, i32)| {
        visited.push(name.to_string());
        if ELSEWHERE.iter().any(|(elsewhere, _)| *elsewhere == name) {
            return;
        }
        let live = [min, max].into_iter().any(|value| {
            let mut options = base.clone();
            set(&mut options, value);
            if name.starts_with("Tm") {
                clock(&options) != base_clock
            } else {
                POSITIONS
                    .iter()
                    .zip(&base_trees)
                    .any(|(fen, base_tree)| tree(&options, fen, DEPTH) != *base_tree)
            }
        });
        if !live {
            dead.push(name.to_string());
        }
    };
    for &(name, min, max, field) in SearchParams::FIELDS {
        visit(name, min, max, &|o, v| {
            *field(&mut o.engine.search_params) = v;
        });
    }
    assert!(
        dead.is_empty(),
        "moved nothing at either end of its range: {dead:?}"
    );
    for name in ELSEWHERE.iter().map(|e| e.0) {
        assert!(
            visited.iter().any(|v| v == name),
            "{name} is no longer a parameter"
        );
    }
}

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const LAZY_MARGIN_REGRESSION_FEN: &str = "5k2/5p1p/p3B1p1/Pp6/1P6/5P1P/4K1P1/8 b - - 0 1";

struct Recorder(Arc<Mutex<Vec<String>>>);

impl InfoSink for Recorder {
    fn line(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_string());
    }
    fn report(&self, report: &InfoReport<'_>) {
        self.line(&crate::uci::info::report_line(report));
    }
}

/// A `Hash` the allocator refuses keeps the table it had and is announced
/// every time it is sent, and never when another option is.
/// `usize::MAX` MiB is a reservation `try_reserve_exact` rejects before any
/// memory is touched, so the refusal is the same on every platform.
#[test]
fn a_hash_the_allocator_refuses_keeps_the_table_and_is_announced_each_time_it_is_sent() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    let refused = EngineOptions {
        hash_mb: usize::MAX,
        ..EngineOptions::default()
    };
    let refusals = || {
        lines
            .lock()
            .unwrap()
            .iter()
            .filter(|line| line.contains("Unable to allocate Hash"))
            .count()
    };
    searcher.configure(&refused, HashRequest::Sent);
    assert_eq!(refusals(), 1);
    let threads = EngineOptions {
        threads: 2,
        ..refused.clone()
    };
    searcher.configure(&threads, HashRequest::NotSent);
    assert_eq!(refusals(), 1, "another option reports nothing about Hash");
    searcher.configure(&threads, HashRequest::Sent);
    assert_eq!(refusals(), 2, "the same Hash sent again is tried again");
    assert_eq!(searcher.shared.hash_mb, 64);
}

/// A `Hash` change lands in the form the next search uses, at the size
/// that form gives that `Hash`; a refused one keeps the table and what it
/// holds, at any thread count.
#[test]
fn a_hash_change_lands_in_the_searchs_form_and_a_refusal_keeps_the_table() {
    let board = Board::default();
    for threads in [1, 4] {
        let mut searcher = Searcher::default();
        let mut options = EngineOptions {
            hash_mb: 2,
            threads,
            ..EngineOptions::default()
        };
        searcher.configure(&options, HashRequest::Sent);
        assert_eq!(searcher.shared.tt.is_shared(), threads > 1);
        assert_eq!(searcher.shared.tt.allocated_bytes(), 2 << 20);

        store_static_eval(&mut searcher, &board, 17);
        options.hash_mb = usize::MAX;
        searcher.configure(&options, HashRequest::Sent);
        assert_eq!(searcher.shared.hash_mb, 2);
        assert_eq!(searcher.shared.tt.is_shared(), threads > 1);
        assert!(
            searcher.shared.tt.probe(board.hash()).is_some(),
            "a refused Hash kept the entry at Threads {threads}"
        );
    }
}

/// The same `Hash` sent again keeps the table and what it holds, at any
/// thread count, where a different size starts an empty one.
#[test]
fn the_same_hash_sent_again_keeps_the_tables_contents() {
    let board = Board::default();
    for threads in [1, 4] {
        let mut searcher = Searcher::default();
        let options = EngineOptions {
            hash_mb: 2,
            threads,
            ..EngineOptions::default()
        };
        searcher.configure(&options, HashRequest::Sent);
        store_static_eval(&mut searcher, &board, 17);
        searcher.configure(&options, HashRequest::Sent);
        assert!(
            searcher.shared.tt.probe(board.hash()).is_some(),
            "the same Hash kept the entry at Threads {threads}"
        );
        let resized = EngineOptions {
            hash_mb: 3,
            ..options.clone()
        };
        searcher.configure(&resized, HashRequest::Sent);
        assert!(
            searcher.shared.tt.probe(board.hash()).is_none(),
            "a new size starts empty at Threads {threads}"
        );
    }
}

/// `bestmove` must be the move the last `info` line names. A stopped
/// iteration can leave another move's bound line last, as can a vote for
/// a helper: the result's own line is printed then, and only then.
#[test]
fn the_result_line_is_printed_only_when_the_last_line_names_another_move() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    let result = SearchResult {
        bestmove: d2d4,
        pondermove: Move::NULL,
        score: 31,
        depth: 13,
        pv: vec![d2d4],
        seldepth: 20,
        nodes: 1,
        tb_hits: 0,
        elapsed_ms: 0,
        exit: SearchExit::Stop,
        ponderhit: false,
    };
    // The last line is a fail-high on e2e4 that the iteration never
    // confirmed before it was stopped.
    searcher.send_shown_info_line(14, 20, 41, RootBound::Lower, &[e2e4]);
    searcher.report_result_line(&result);
    let printed = lines.lock().unwrap().clone();
    assert_eq!(printed.len(), 2, "{printed:?}");
    assert!(printed[1].ends_with(" pv d2d4"), "{printed:?}");
    // The last line names d2d4, so nothing more is printed.
    searcher.report_result_line(&result);
    assert_eq!(lines.lock().unwrap().len(), 2);
}

/// A search stopped inside its first iteration has no score; its line
/// printed the marker as `mate -0`.
#[test]
fn a_move_the_search_never_scored_prints_zero() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    let a2a3 = Move::from_uci("a2a3").expect("valid move");
    let result = SearchResult {
        bestmove: a2a3,
        pondermove: Move::NULL,
        score: -INF_SCORE,
        depth: 0,
        pv: vec![a2a3],
        seldepth: 0,
        nodes: 1,
        tb_hits: 0,
        elapsed_ms: 0,
        exit: SearchExit::Stop,
        ponderhit: false,
    };
    searcher.report_line(&result);
    let printed = lines.lock().unwrap().clone();
    assert_eq!(printed.len(), 1, "{printed:?}");
    assert!(printed[0].contains(" score cp 0 nodes "), "{printed:?}");
}

/// A node budget smaller than the first iteration still completes it, so
/// the move played was searched and the score is one the search found.
#[test]
fn a_node_budget_waits_for_the_first_iteration() {
    let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .expect("valid FEN");
    let mut first = SearchOptions::default();
    first.limits.depth = Some(1);
    let first_iteration = Searcher::default()
        .search(board.clone(), &first, false, || SearchEvent::None)
        .nodes;
    let mut options = SearchOptions::default();
    options.limits.nodes = 1;
    let result = Searcher::default().search(board, &options, false, || SearchEvent::None);
    assert_eq!(result.depth, 1);
    assert!(
        result.score.abs() < MATE_SCORE - infra::to_i32(MAX_PLY),
        "{}",
        result.score
    );
    // The first iteration's windows, and the first node of depth 2 that
    // stops it.
    assert_eq!(result.nodes, first_iteration + 1);
}

/// The printed score and its rates are output only: a search showing
/// them makes the same tree, move, internal score and number of lines as
/// one that does not, and only its lines carry `wdl`.
#[test]
fn showing_the_rates_changes_no_search() {
    for fen in [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        LAZY_MARGIN_REGRESSION_FEN,
    ] {
        let run = |show_wdl| {
            let lines = Arc::new(Mutex::new(Vec::new()));
            let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
            let mut options = SearchOptions::default();
            options.limits.depth = Some(9);
            options.engine.show_wdl = show_wdl;
            let board = Board::from_fen(fen).expect("valid FEN");
            let result = searcher.search(board, &options, true, || SearchEvent::None);
            let printed = lines.lock().unwrap().clone();
            (result, printed)
        };
        let (plain, plain_lines) = run(false);
        let (shown, shown_lines) = run(true);
        assert_eq!(
            (
                plain.nodes,
                plain.bestmove,
                plain.score,
                plain.depth,
                &plain.pv
            ),
            (
                shown.nodes,
                shown.bestmove,
                shown.score,
                shown.depth,
                &shown.pv
            ),
            "{fen}"
        );
        assert_eq!(plain_lines.len(), shown_lines.len(), "{fen}");
        let info = |lines: &[String]| -> Vec<bool> {
            lines
                .iter()
                .filter(|l| l.contains(" score "))
                .map(|l| l.contains(" wdl "))
                .collect()
        };
        assert!(info(&plain_lines).iter().all(|wdl| !wdl), "{fen}");
        assert!(info(&shown_lines).iter().all(|wdl| *wdl), "{fen}");
        assert!(!info(&shown_lines).is_empty(), "{fen}");
    }
}

/// A tablebase score is exact however the search's window failed, so its
/// line carries no bound; a searched score keeps the bound it has.
#[test]
fn a_tablebase_root_score_prints_without_a_bound() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    searcher.td.tb_root_scores = vec![(e2e4, TB_VALUE)];
    searcher.send_full_info_line(5, 11, 41, RootBound::Upper, &[e2e4]);
    searcher.send_full_info_line(5, 11, 41, RootBound::Upper, &[d2d4]);
    // A mate the search proved is its own score and keeps its bound.
    searcher.send_full_info_line(5, 11, MATE_SCORE - 3, RootBound::Lower, &[e2e4]);
    let printed = lines.lock().unwrap().clone();
    assert!(printed[0].contains(" score cp 20000 nodes "), "{printed:?}");
    // No search ran, so the line prints at the start position's material.
    let searched = format!(
        " score {} upperbound nodes ",
        crate::uci::info::format_score(41, 78)
    );
    assert!(printed[1].contains(&searched), "{printed:?}");
    assert!(
        printed[2].contains(" score mate 2 lowerbound nodes "),
        "{printed:?}"
    );
}

#[test]
fn search_writes_one_info_line_per_iteration_through_its_sink() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    let mut options = SearchOptions::default();
    options.limits.depth = Some(3);

    let result = searcher.search(options.board.clone(), &options, true, || SearchEvent::None);

    let lines = lines.lock().unwrap();
    assert_eq!(result.depth, 3);
    // A window that fails prints its bound on the way; each iteration
    // ends on one exact line.
    let exact: Vec<&String> = lines
        .iter()
        .filter(|line| !line.contains("lowerbound") && !line.contains("upperbound"))
        .collect();
    assert_eq!(exact.len(), 3, "{lines:?}");
    for (index, line) in exact.iter().enumerate() {
        assert!(
            line.starts_with(&format!("info depth {} ", index + 1)),
            "{line}"
        );
    }
    for line in lines.iter() {
        assert!(line.contains(" pv "), "{line}");
    }

    drop(lines);

    let quiet_lines = Arc::new(Mutex::new(Vec::new()));
    let mut quiet = Searcher::with_sink(Box::new(Recorder(Arc::clone(&quiet_lines))));
    quiet.search(options.board.clone(), &options, false, || SearchEvent::None);
    assert!(
        quiet_lines.lock().unwrap().is_empty(),
        "emit_info = false writes nothing"
    );
}

/// A failed window's bound line is printed only past the floor, and
/// keeps the exact line's shape when it is; the exact lines are the
/// same either way.
#[test]
fn bound_lines_are_printed_past_the_node_floor_only() {
    let limits = SearchLimits {
        depth: Some(10),
        ..SearchLimits::default()
    };
    let fen = FAIL_HIGH_CANDIDATES[0];
    let options = EngineOptions::default();
    let (short, lines) = clock_search(fen, &limits, &options, None, usize::MAX);
    assert!(short.nodes < BOUND_LINES_FROM_NODES);
    assert!(
        lines.iter().all(|line| line.bound.is_empty()),
        "no bound line"
    );
    let (all, every) = clock_search_with_bounds(fen, &limits, &options, None, usize::MAX, 0);
    assert_eq!(all.nodes, short.nodes, "printing changes no search");
    assert!(
        every.iter().any(|line| !line.bound.is_empty()),
        "a window fails"
    );
    let exact = |lines: &[InfoLine]| {
        lines
            .iter()
            .filter(|line| line.bound.is_empty())
            .map(|line| {
                (
                    line.depth,
                    line.score.clone(),
                    line.nodes,
                    line.first.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        exact(&lines),
        exact(&every),
        "the exact lines are unchanged"
    );
}

/// The budget is measured from the instant `go` was
/// parsed, not from the engine thread's start. A `go movetime 200`
/// parsed 300 ms ago has already spent its budget and must return at
/// once with a legal move from the first completed iteration.
#[test]
fn search_clock_starts_when_go_was_parsed() {
    let board = Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).unwrap();
    let legal = board.generate_legal_moves();
    let limits = SearchLimits {
        move_time: 200,
        issued: Some(Instant::now() - Duration::from_millis(300)),
        ..SearchLimits::default()
    };
    let mut searcher = Searcher::default();
    searcher.reset_search_state(
        &limits,
        &EngineOptions::default(),
        board.side_to_move(),
        0,
        true,
    );
    let started = Instant::now();
    let result = searcher.search_root(board, &legal, false, &mut || SearchEvent::None);
    let took = started.elapsed();
    assert!(
        took < Duration::from_millis(100),
        "a search whose budget expired before it began must stop at once, took {took:?}"
    );
    assert!(
        result.depth >= 1,
        "at least one iteration completes: {}",
        result.depth
    );
    assert!(legal.contains(&result.bestmove), "bestmove must be legal");
    assert!(
        result.elapsed_ms >= 300,
        "elapsed is measured from the parse instant: {} ms",
        result.elapsed_ms
    );
}

fn store_static_eval(searcher: &mut Searcher, board: &Board, static_eval: i32) {
    let mv = board
        .generate_legal_moves()
        .into_iter()
        .next()
        .expect("regression position has a legal move");
    searcher.shared.tt.store(TtStore {
        key: board.hash(),
        depth: 1,
        score: static_eval,
        bound: Bound::Exact,
        mv,
        ply: 0,
        static_eval,
        is_pv: false,
    });
}

#[test]
fn lazy_margin_change_invalidates_local_and_shared_tt_evals() {
    const LOW_MARGIN: i32 = 200;
    const HIGH_MARGIN: i32 = 2_000;
    let board =
        Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).expect("valid lazy-eval regression FEN");
    let limits = SearchLimits::default();

    for shared in [false, true] {
        let mut searcher = Searcher::default();
        let mut options = EngineOptions::default();
        options.search_params.lazy_margin = LOW_MARGIN;
        searcher.reset_search_state(&limits, &options, board.side_to_move(), 0, true);
        let low_margin_score = searcher.raw_eval(&board);

        if shared {
            searcher.shared.tt.make_shared(searcher.shared.hash_mb);
        }
        store_static_eval(&mut searcher, &board, low_margin_score);
        assert_eq!(
            i32::from(
                searcher
                    .shared
                    .tt
                    .probe(board.hash())
                    .expect("stored TT entry")
                    .static_eval
            ),
            low_margin_score
        );

        options.search_params.lazy_margin = HIGH_MARGIN;
        searcher.reset_search_state(&limits, &options, board.side_to_move(), 0, true);
        assert!(
            searcher.shared.tt.probe(board.hash()).is_none(),
            "LazyMargin change retained a {} TT evaluation",
            if shared { "shared" } else { "local" }
        );
        assert_ne!(searcher.raw_eval(&board), low_margin_score);
    }
}

/// A search takes the tuning options' exchange values and pawn-table
/// limit when it begins.
#[cfg(all(feature = "tune", not(feature = "pricing")))]
#[test]
fn a_search_takes_the_exchange_values_and_pawn_limit_from_the_options() {
    use crate::board::{Piece, Square};

    let mut searcher = Searcher::default();
    let mut options = EngineOptions::default();
    options.compact_seeds.pawn_history_max = 4_096;
    options.compact_seeds.exchange_value[3] = 600;
    searcher.reset_search_state(&SearchLimits::default(), &options, Color::White, 0, false);
    assert_eq!(searcher.exchange_values().as_array()[3], 600);
    let row = history::HistoryTables::pawn_row(0);
    searcher
        .td
        .hist
        .update_pawn(row, Color::White, Piece::Knight, Square::F3, 100_000);
    assert_eq!(
        searcher
            .td
            .hist
            .pawn(row, Color::White, Piece::Knight, Square::F3),
        4_096,
        "one clamped bonus saturates the table at its limit"
    );
}

#[test]
fn helper_lazy_margin_sync_does_not_clear_live_shared_tt() {
    let board =
        Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).expect("valid lazy-eval regression FEN");
    let mut main = Searcher::default();
    main.shared.tt.make_shared(main.shared.hash_mb);
    store_static_eval(&mut main, &board, 123);

    let mut helper = Searcher::default();
    helper.shared.tt =
        TranspositionTable::attached(main.shared.tt.make_shared(main.shared.hash_mb));
    let mut options = EngineOptions::default();
    options.search_params.lazy_margin = 2_000;
    helper.reset_search_state(
        &SearchLimits::default(),
        &options,
        board.side_to_move(),
        0,
        false,
    );

    assert!(
        main.shared.tt.probe(board.hash()).is_some(),
        "helper startup cleared the shared TT after another thread made it live"
    );
}

#[test]
fn configure_puts_the_table_in_the_form_the_next_search_uses() {
    let mut searcher = Searcher::default();
    let mut options = EngineOptions {
        hash_mb: 1,
        threads: 4,
        ..EngineOptions::default()
    };
    searcher.configure(&options, HashRequest::Sent);
    assert!(
        searcher.shared.tt.is_shared(),
        "Threads > 1 must convert the table at configure, not in the first search"
    );
    options.threads = 1;
    searcher.configure(&options, HashRequest::Sent);
    assert!(
        !searcher.shared.tt.is_shared(),
        "Threads = 1 must restore the local table at configure"
    );
}

/// A `LargePages` change allocates the tables again: the transposition
/// table comes back empty at the same `Hash` and form, and a notice says
/// how they are backed. Any other option change keeps the table.
#[test]
fn a_large_pages_change_allocates_the_tables_again() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    let board = Board::default();
    let mut options = EngineOptions {
        hash_mb: 2,
        threads: 2,
        ..EngineOptions::default()
    };
    searcher.configure(&options, HashRequest::Sent);
    store_static_eval(&mut searcher, &board, 17);

    options.multi_pv = 2;
    searcher.configure(&options, HashRequest::Sent);
    assert!(
        searcher.shared.tt.probe(board.hash()).is_some(),
        "an unrelated option change kept the table"
    );
    assert!(lines.lock().unwrap().is_empty());

    options.large_pages = false;
    searcher.configure(&options, HashRequest::Sent);
    assert!(
        searcher.shared.tt.probe(board.hash()).is_none(),
        "the table was allocated again"
    );
    assert_eq!(searcher.shared.hash_mb, 2);
    let mut reference = TranspositionTable::new(2);
    reference.make_shared(2);
    assert!(searcher.shared.tt.is_shared());
    assert_eq!(
        searcher.shared.tt.capacity_entries(),
        reference.capacity_entries(),
        "the shared table at the same Hash"
    );
    assert_eq!(
        lines.lock().unwrap().as_slice(),
        ["info string Large pages: disabled"]
    );

    options.large_pages = true;
    searcher.configure(&options, HashRequest::Sent);
    assert_eq!(
        lines.lock().unwrap().len(),
        2,
        "the change back is reported"
    );
    assert!(!lines.lock().unwrap()[1].ends_with("disabled"));
}

#[test]
fn search_root_respects_restricted_root_moves() {
    let mut searcher = Searcher::default();
    let board = Board::default();
    let forced = board.parse_move("a2a3").expect("legal root move");
    let engine_options = EngineOptions::default();
    let limits = SearchLimits {
        depth: Some(1),
        ..SearchLimits::default()
    };
    searcher.reset_search_state(&limits, &engine_options, board.side_to_move(), 0, true);

    let result = searcher.search_root(board, &[forced], false, &mut || SearchEvent::None);

    assert_eq!(result.bestmove, forced);
}

#[test]
fn root_move_record_keeps_iteration_and_distribution_state() {
    let mv = Move::from_uci("e2e4").expect("valid move");
    let mut root_move = RootMove::new(mv);

    root_move.record_search(1, 20, 100, 7, RootBound::Exact);
    root_move.complete_iteration();
    root_move.begin_iteration();
    root_move.record_search(2, 40, 250, 9, RootBound::Upper);
    root_move.record_search(2, -30, 50, 8, RootBound::Lower);
    root_move.complete_iteration();

    assert_eq!(root_move.mv, mv);
    assert_eq!(root_move.previous_score, 20);
    assert_eq!(root_move.score, -30);
    assert_eq!(root_move.samples, 2);
    assert!((root_move.average_score + 5.0).abs() < f64::EPSILON);
    assert!((root_move.mean_squared_score - 650.0).abs() < f64::EPSILON);
    assert_eq!(root_move.nodes, 400);
    assert_eq!(root_move.seldepth, 9);
    assert_eq!(root_move.fail_highs, 1);
    assert_eq!(root_move.fail_lows, 1);
    assert_eq!(root_move.pv_len, 1);
    assert_eq!(root_move.pv[0], mv);
}

#[test]
fn search_populates_persistent_root_move_records() {
    let mut searcher = Searcher::default();
    let board = Board::default();
    let legal = board.generate_legal_moves();
    let limits = SearchLimits {
        depth: Some(3),
        ..SearchLimits::default()
    };
    searcher.reset_search_state(
        &limits,
        &EngineOptions::default(),
        board.side_to_move(),
        0,
        true,
    );

    let result = searcher.search_root(board, &legal, false, &mut || SearchEvent::None);

    assert_eq!(searcher.td.root_move_records.len(), legal.len());
    assert!(searcher.td.root_move_records.iter().all(|rm| {
        rm.samples >= 3
            && rm.previous_score > -INF_SCORE
            && rm.score > -INF_SCORE
            && rm.nodes > 0
            && rm.seldepth > 0
            && rm.pv[0] == rm.mv
            && rm.mean_squared_score + 1e-9 >= rm.average_score * rm.average_score
    }));
    let best = searcher
        .td
        .root_move_records
        .iter()
        .find(|rm| rm.mv == result.bestmove)
        .expect("best move has a persistent record");
    assert_eq!(best.last_best_depth, result.depth);
    // At least the move: after a fail high the window searches one ply
    // less, and a line from depth 1 is the move alone.
    assert!(best.pv_len >= 1);
    assert!(
        searcher
            .td
            .root_move_records
            .iter()
            .map(|rm| rm.nodes)
            .sum::<u64>()
            <= result.nodes
    );
}

/// The SMP machinery (pool root scores, stop voting, reduction
/// jitter, pool-seeded aspiration) must be INERT at Threads=1.
///
/// Every SMP feature gates on the thread count, which only a parallel
/// search raises above one. This guards the property every 1-thread gate
/// and the bench fingerprint rely on: a serial search must be
/// deterministic and free of any pool machinery. If a gate ever leaks into
/// the serial path, the run-to-run identity below breaks.
#[test]
fn smp_machinery_is_inert_on_a_single_thread() {
    let board = Board::default();
    let legal = board.generate_legal_moves();

    let run = || {
        let mut searcher = Searcher::default();
        let limits = SearchLimits {
            depth: Some(7),
            ..SearchLimits::default()
        };
        searcher.reset_search_state(
            &limits,
            &EngineOptions::default(),
            board.side_to_move(),
            0,
            true,
        );
        assert!(
            searcher.shared.pool().is_none(),
            "a serial search must have every pool gate closed"
        );
        let result = searcher.search_root(board.clone(), &legal, false, &mut || SearchEvent::None);
        (result.nodes, result.bestmove, result.score)
    };

    assert_eq!(
        run(),
        run(),
        "a serial search must be deterministic with all SMP gates closed"
    );
}

/// `searchmoves` with one move is the caller's line, not a forced move:
/// the search runs to its depth instead of stopping at depth 2.
#[test]
fn a_one_move_searchmoves_is_not_a_forced_move() {
    let board = Board::starting_position();
    let mv = board.parse_move("e2e4").expect("legal move");
    let limits = SearchLimits {
        depth: Some(6),
        search_moves: vec![mv.into()],
        ..SearchLimits::default()
    };
    let mut searcher = Searcher::default();
    searcher.reset_search_state(
        &limits,
        &EngineOptions::default(),
        board.side_to_move(),
        0,
        true,
    );
    let depth = searcher
        .search_root(board, &[mv], false, &mut || SearchEvent::None)
        .depth;
    assert_eq!(
        depth, 6,
        "a one-move searchmoves stopped at the forced-move shortcut"
    );
}

/// The single-legal-move shortcut must save clock time WITHOUT truncating
/// an analysis search.
///
/// On `1k3Q1r/pPpP2p1/P1P3P1/8/8/1p6/1P6/K6N b` only `Rxf8` is legal.
/// `search_root` breaks after depth 2 on a single root move — correct under
/// a clock, since that move gets played regardless of score, but under
/// `go infinite`/`go ponder` it would freeze a GUI's analysis at depth 2 on
/// a meaningless score. Both halves are asserted: the shortcut fires for a
/// move request and not in analysis mode.
#[test]
fn single_legal_move_shortcut_skips_analysis_but_not_move_requests() {
    const FORCED: &str = "1k3Q1r/pPpP2p1/P1P3P1/8/8/1p6/1P6/K6N b - - 0 1";
    let board = Board::from_fen(FORCED).expect("valid fen");
    let legal = board.generate_legal_moves();
    assert_eq!(legal.len(), 1, "position must have exactly one legal move");

    let run = |infinite: bool| {
        let mut searcher = Searcher::default();
        let limits = SearchLimits {
            // Generous depth cap either way, so the STOP REASON under test
            // is the shortcut rather than the depth limit.
            depth: Some(8),
            infinite,
            ..SearchLimits::default()
        };
        searcher.reset_search_state(
            &limits,
            &EngineOptions::default(),
            board.side_to_move(),
            0,
            true,
        );
        searcher
            .search_root(board.clone(), &legal, false, &mut || SearchEvent::None)
            .depth
    };

    // Move request: stops at the shortcut, well short of the depth cap.
    let move_request_depth = run(false);
    assert_eq!(
        move_request_depth, 2,
        "clock/move-request search should stop at the single-move shortcut"
    );

    // Analysis: must keep going and honour the depth request instead.
    let analysis_depth = run(true);
    assert!(
        analysis_depth > move_request_depth,
        "analysis search must not be truncated by the single-move shortcut              (got depth {analysis_depth}, move-request depth {move_request_depth})"
    );
}

#[test]
fn ponderhit_preserves_elapsed_time_budget() {
    let mut searcher = Searcher::default();
    // Mid-search: the second iteration is running.
    searcher.td.root_depth = 2;
    searcher.td.pondering = true;
    searcher.cfg.start = Instant::now() - Duration::from_millis(10);
    searcher.cfg.limits = RuntimeLimits {
        depth: 64,
        optimum_ms: 1.0,
        maximum_ms: 1.0,
        ..RuntimeLimits::default()
    };
    searcher.td.nodes = 2047;

    searcher.count_node(&mut || SearchEvent::PonderHit);

    assert!(searcher.td.stopped);
    assert!(searcher.td.ponderhit);
    assert!(!searcher.td.pondering);
}

/// A search's `info depth` lines, parsed: depth, the score as printed,
/// its bound (empty when exact), the node count and the first PV move.
struct InfoLine {
    depth: usize,
    score: String,
    bound: &'static str,
    nodes: u64,
    first: String,
}

fn parse_info(line: &str) -> InfoLine {
    let field = |name: &str| {
        line.split_once(&format!(" {name} "))
            .and_then(|(_, rest)| rest.split_whitespace().next())
            .unwrap_or_else(|| panic!("no {name} in {line}"))
            .to_string()
    };
    let score = line
        .split_once(" score ")
        .map(|(_, rest)| {
            rest.split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_else(|| panic!("no score in {line}"));
    InfoLine {
        depth: field("depth").parse().expect("numeric depth"),
        score,
        bound: if line.contains(" lowerbound ") {
            "lowerbound"
        } else if line.contains(" upperbound ") {
            "upperbound"
        } else {
            ""
        },
        nodes: field("nodes").parse().expect("numeric nodes"),
        first: field("pv"),
    }
}

/// A one-thread search of `fen` with `test_clock` standing in for the
/// clock, ended by the GUI after `poll_budget` polls of 2,048 nodes;
/// returns the result and its `info depth` lines.
fn clock_search(
    fen: &str,
    limits: &SearchLimits,
    options: &EngineOptions,
    test_clock: Option<(u64, f64)>,
    poll_budget: usize,
) -> (SearchResult, Vec<InfoLine>) {
    clock_search_with_bounds(
        fen,
        limits,
        options,
        test_clock,
        poll_budget,
        BOUND_LINES_FROM_NODES,
    )
}

/// `clock_search` with failed windows' bound lines printed past
/// `bound_lines_from_nodes` nodes.
fn clock_search_with_bounds(
    fen: &str,
    limits: &SearchLimits,
    options: &EngineOptions,
    test_clock: Option<(u64, f64)>,
    poll_budget: usize,
    bound_lines_from_nodes: u64,
) -> (SearchResult, Vec<InfoLine>) {
    let board = Board::from_fen(fen).expect("valid fen");
    let legal = board.generate_legal_moves();
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    let options = EngineOptions {
        bound_lines_from_nodes,
        ..options.clone()
    };
    searcher.reset_search_state(limits, &options, board.side_to_move(), 0, true);
    assert!(searcher.shared.pool().is_none());
    searcher.td.test_clock = test_clock;
    let mut polls = 0;
    let result = searcher.search_root(board, &legal, true, &mut || {
        polls += 1;
        if polls >= poll_budget {
            SearchEvent::Stop
        } else {
            SearchEvent::None
        }
    });
    assert!(legal.contains(&result.bestmove));
    let lines = lines
        .lock()
        .unwrap()
        .iter()
        .filter(|line| line.starts_with("info depth "))
        .map(|line| parse_info(line))
        .collect();
    (result, lines)
}

/// A clock that never moves: no soft or hard limit is ever reached.
const FROZEN_CLOCK: Option<(u64, f64)> = Some((u64::MAX, 0.0));
const SCHOLARS_MATE: &str = "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4";

fn minute_clock() -> SearchLimits {
    SearchLimits {
        white_time: Some(60_000),
        black_time: Some(60_000),
        ..SearchLimits::default()
    }
}

#[test]
fn a_proved_mate_ends_a_clock_search_after_three_iterations_but_not_analysis() {
    const BUDGET: usize = 500;
    let (result, lines) = clock_search(
        SCHOLARS_MATE,
        &minute_clock(),
        &EngineOptions::default(),
        FROZEN_CLOCK,
        BUDGET,
    );
    // A window that fails prints its bound on the way; each iteration
    // ends on one exact line.
    let exact: Vec<&InfoLine> = lines.iter().filter(|line| line.bound.is_empty()).collect();
    let first_mate = exact
        .iter()
        .position(|line| line.score == "mate 1")
        .expect("the mate is found");
    assert_eq!(
        exact.len(),
        first_mate + 3,
        "three mate iterations, then the stop"
    );
    for line in &exact[first_mate..] {
        assert!(line.score == "mate 1" && line.first == "h5f7");
    }
    assert_eq!(result.depth, exact[first_mate].depth + 2);
    assert_eq!(result.bestmove.to_string(), "h5f7");
    assert!(
        result.nodes < (BUDGET * 2048) as u64,
        "the stop came from the mate"
    );

    let (analysis, analysis_lines) = clock_search(
        SCHOLARS_MATE,
        &SearchLimits {
            infinite: true,
            ..minute_clock()
        },
        &EngineOptions::default(),
        FROZEN_CLOCK,
        BUDGET,
    );
    assert!(analysis.depth > result.depth + 3, "{}", analysis.depth);
    assert!(analysis_lines.len() > lines.len());
}

/// Positions in which some iteration from depth 4 fails high at the root.
const FAIL_HIGH_CANDIDATES: [&str; 4] = [
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "r1bq1rk1/ppp2ppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQ1RK1 w - - 0 7",
    "2rq1rk1/pb1nbppp/1p2pn2/2pp4/2PP4/1PN1PN2/PB2BPPP/2RQ1RK1 w - - 0 11",
    LAZY_MARGIN_REGRESSION_FEN,
];

#[test]
fn a_root_fail_high_past_the_soft_target_plays_the_move_with_its_lower_bound() {
    const DEPTH: u32 = 10;
    // The fail-high, and the node count at which the iteration before it
    // completed, read from a search with no clock: a serial tree does not
    // depend on the clock until something stops it.
    let (fen, fail_high, completed_nodes) = FAIL_HIGH_CANDIDATES
        .iter()
        .find_map(|&fen| {
            let probe = SearchLimits {
                depth: Some(DEPTH),
                ..SearchLimits::default()
            };
            // Every failed window's line, so the fail-high shows.
            let (_, mut lines) = clock_search_with_bounds(
                fen,
                &probe,
                &EngineOptions::default(),
                None,
                usize::MAX,
                0,
            );
            // The first fail-high from depth 4: an earlier one is not
            // what this test reads, and the seeds decide whether a
            // position has one.
            let at = lines
                .iter()
                .position(|line| line.bound == "lowerbound" && line.depth >= 4)?;
            let completed = lines[..at]
                .iter()
                .rev()
                .find(|line| line.bound.is_empty())?;
            assert_eq!(completed.depth + 1, lines[at].depth);
            let completed_nodes = completed.nodes;
            Some((fen, lines.swap_remove(at), completed_nodes))
        })
        .expect("a candidate fails high at the root");
    assert!(fail_high.depth >= 4);

    // The score trend pinned at a half keeps every soft target below 99%
    // of the maximum; the stop is read at the target itself.
    let mut options = EngineOptions::default();
    options.search_params.tm_trend_low = 5_000;
    options.search_params.tm_trend_high = 5_000;
    options.search_params.tm_fail_high_target = 10_000;
    let board = Board::from_fen(fen).expect("valid fen");
    let clock = SearchLimits {
        depth: Some(DEPTH),
        ..minute_clock()
    };
    let maximum_ms =
        compute_runtime_limits(&clock, &options, board.side_to_move(), 0, MAX_DEPTH).maximum_ms;

    // The clock passes the soft target, not the maximum, during the
    // iteration that fails high. The search is short of the bound-line
    // floor, so the line it stops on is printed because it stops.
    let late = Some((completed_nodes + 1, 0.99 * maximum_ms));
    let (stopped, lines) = clock_search(fen, &clock, &options, late, usize::MAX);
    assert!(stopped.nodes < BOUND_LINES_FROM_NODES);
    assert_eq!(
        lines.iter().filter(|line| !line.bound.is_empty()).count(),
        1,
        "only the line it stops on is a bound"
    );
    let last = lines.last().expect("a line was printed");
    assert_eq!(last.bound, "lowerbound");
    assert_eq!(
        (last.depth, &last.score, last.nodes, &last.first),
        (
            fail_high.depth,
            &fail_high.score,
            fail_high.nodes,
            &fail_high.first
        )
    );
    assert_eq!(stopped.bestmove.to_string(), fail_high.first);
    assert_eq!(
        crate::uci::info::format_score(stopped.score, score_material(&board)),
        fail_high.score
    );
    assert_eq!(stopped.depth, fail_high.depth - 1);

    // Short of the soft target the fail-high is searched again.
    let (unstopped, lines) = clock_search(fen, &clock, &options, FROZEN_CLOCK, usize::MAX);
    assert_eq!(unstopped.depth, DEPTH as usize);
    assert!(lines.last().expect("a line was printed").bound.is_empty());

    // With the stop's threshold past every maximum the same late clock
    // completes the fail-high's iteration, and the soft stop ends the
    // search between iterations on an exact line.
    let mut never = options;
    never.search_params.tm_fail_high_target = 10_000_000;
    let (unswitched, lines) = clock_search(fen, &clock, &never, late, usize::MAX);
    assert_eq!(unswitched.depth, fail_high.depth);
    assert!(lines.last().expect("a line was printed").bound.is_empty());
}

/// A position is counted once, when a move reaches it: a depth-1 root
/// search of the start position in an open window makes its twenty root
/// moves, and neither the root nor a child's hand-off to quiescence adds
/// a count.
#[test]
fn a_depth_one_search_counts_each_root_move_once() {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .expect("valid FEN");
    let mut searcher = Searcher::default();
    searcher.search_root_window(&mut board, 1, -INF_SCORE, INF_SCORE, &mut || {
        SearchEvent::None
    });
    assert_eq!(searcher.td.nodes, 20);
}

#[test]
fn the_hard_limit_waits_for_the_first_iteration() {
    let mut searcher = Searcher::default();
    searcher.cfg.start = Instant::now() - Duration::from_millis(10);
    searcher.cfg.limits = RuntimeLimits {
        depth: 64,
        optimum_ms: 1.0,
        maximum_ms: 1.0,
        ..RuntimeLimits::default()
    };
    searcher.td.root_depth = 1;
    searcher.td.nodes = 2047;
    searcher.count_node(&mut || SearchEvent::None);
    assert!(!searcher.td.stopped);
    searcher.td.root_depth = 2;
    searcher.td.nodes = 4095;
    searcher.count_node(&mut || SearchEvent::None);
    assert!(searcher.td.stopped);
}

#[test]
fn a_clock_spent_before_the_search_began_still_returns_a_first_iteration_move() {
    let limits = SearchLimits {
        issued: Some(Instant::now() - Duration::from_secs(10)),
        ..minute_clock()
    };
    let (result, lines) = clock_search(
        SCHOLARS_MATE,
        &limits,
        &EngineOptions::default(),
        None,
        usize::MAX,
    );
    assert_eq!(result.depth, 1);
    // The first iteration's windows, ending on its one exact line.
    assert!(lines.iter().all(|line| line.depth == 1));
    let last = lines.last().expect("a line was printed");
    assert!(last.bound.is_empty());
    assert_eq!(last.first, result.bestmove.to_string());
}

/// A castling move that fails belongs to the quiet tables: no search
/// writes the noisy slot of a king moving to its castled square without a
/// capture, and the quiet table does hold the castle.
#[test]
fn a_search_writes_castling_to_the_quiet_tables_only() {
    use crate::board::{Bitboard, Color, Move, Piece, Square};

    let engine = EngineOptions::default();
    let mut searcher = Searcher::default();
    let limits = SearchLimits {
        depth: Some(9),
        ..SearchLimits::default()
    };
    let board =
        Board::from_fen("r3k2r/pppq1ppp/2n1bn2/3pp3/3PP3/2N1BN2/PPPQ1PPP/R3K2R w KQkq - 0 1")
            .expect("valid fen");
    let legal = board.generate_legal_moves();
    searcher.reset_search_state(&limits, &engine, board.side_to_move(), 0, true);
    searcher.search_root(board, &legal, false, &mut || SearchEvent::None);

    let castles = [
        (Color::White, Square::E1, Square::G1),
        (Color::White, Square::E1, Square::C1),
        (Color::Black, Square::E8, Square::G8),
        (Color::Black, Square::E8, Square::C8),
    ];
    let mut written = 0;
    for (color, from, to) in castles {
        let hist = &searcher.td.hist;
        assert_eq!(
            hist.noisy(Bitboard::EMPTY, color, Piece::King, to, None),
            0,
            "the noisy slot for a castle to {to}"
        );
        written += usize::from(hist.quiet(Bitboard::EMPTY, color, Move::new(from, to, 0)) != 0);
    }
    assert!(written > 0, "the search never reached a castling move");
}

/// Each search leaves its root score for the next one's score trend; a
/// new game forgets it, and before any search the trend reads from zero,
/// as the donor's does.
#[test]
fn a_search_leaves_its_score_for_the_next_move_until_a_new_game() {
    let engine = EngineOptions::default();
    let mut searcher = Searcher::default();
    assert_eq!(searcher.td.compact_previous_score, 0);
    let depth_limited = SearchLimits {
        depth: Some(4),
        ..SearchLimits::default()
    };
    let board = Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).expect("valid fen");
    let legal = board.generate_legal_moves();
    searcher.reset_search_state(&depth_limited, &engine, board.side_to_move(), 0, true);
    let first = searcher.search_root(board.clone(), &legal, false, &mut || SearchEvent::None);
    assert_eq!(searcher.td.compact_previous_score, first.score);

    let mut next = board;
    next.make_move(first.bestmove);
    let legal = next.generate_legal_moves();
    searcher.reset_search_state(&depth_limited, &engine, next.side_to_move(), 1, true);
    let second = searcher.search_root(next, &legal, false, &mut || SearchEvent::None);
    assert_eq!(searcher.td.compact_previous_score, second.score);

    searcher.new_game();
    assert_eq!(searcher.td.compact_previous_score, 0);
}

/// `go` parsed 60 ms before the search starts, at a 150 ms clock: the
/// hard limit is measured from the parse and the move still leaves the
/// `2 × Move Overhead` reserve.
#[test]
fn a_late_go_at_a_150_ms_clock_leaves_the_reserve() {
    let board = Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).expect("valid fen");
    let legal = board.generate_legal_moves();
    let engine = EngineOptions::default();
    // Built first: allocating the table is not part of this move.
    let mut searcher = Searcher::default();
    let limits = SearchLimits {
        white_time: Some(150),
        black_time: Some(150),
        white_increment: 30,
        black_increment: 30,
        issued: Some(Instant::now() - Duration::from_millis(60)),
        ..SearchLimits::default()
    };
    searcher.reset_search_state(&limits, &engine, board.side_to_move(), 120, true);
    let ceiling = 150.0 - 2.0 * engine.move_overhead;
    assert!(searcher.cfg.limits.maximum_ms <= ceiling);
    let result = searcher.search_root(board, &legal, false, &mut || SearchEvent::None);
    assert!(result.depth >= 1 && legal.contains(&result.bestmove));
    assert!(
        (result.elapsed_ms as f64) < ceiling,
        "{} ms of a 150 ms clock",
        result.elapsed_ms
    );
}

/// The clock's stops read only this thread's state: one thread stays
/// deterministic and never builds a pool.
#[test]
fn the_clock_stops_are_deterministic_and_poolless_on_one_thread() {
    let run = || {
        let (result, _) = clock_search(
            SCHOLARS_MATE,
            &minute_clock(),
            &EngineOptions::default(),
            FROZEN_CLOCK,
            usize::MAX,
        );
        (result.nodes, result.bestmove, result.score, result.depth)
    };
    assert_eq!(run(), run());
}

#[test]
fn malformed_tt_move_is_not_searched_or_reported_in_pv() {
    let root = Board::from_fen("2k5/pp3pp1/5n2/2P5/bPP2P2/P3K3/6Pp/3Q1B1R w - - 0 23")
        .expect("valid tournament-derived FEN");
    let illegal = Move::from_uci("e3f4").expect("valid UCI move shape");
    let mut board = root.clone();
    let before_fen = board.to_fen();
    let before_hash = board.hash();
    let mut searcher = Searcher::default();
    let engine_options = EngineOptions::default();
    let limits = SearchLimits {
        depth: Some(3),
        ..SearchLimits::default()
    };
    searcher.reset_search_state(&limits, &engine_options, board.side_to_move(), 0, true);
    searcher.shared.tt.store(TtStore {
        key: board.hash(),
        depth: 8,
        score: 0,
        bound: Bound::Exact,
        mv: illegal,
        ply: 0,
        static_eval: VALUE_NONE,
        is_pv: false,
    });

    let _ = searcher.search_root_window(&mut board, 3, -INF_SCORE, INF_SCORE, &mut || {
        SearchEvent::None
    });

    assert_eq!(board.to_fen(), before_fen);
    assert_eq!(board.hash(), before_hash);
    assert!(
        !searcher.td.pv_table[0][..searcher.td.pv_len[0].min(MAX_PLY)]
            .iter()
            .any(|&mv| mv.same_uci_move(illegal)),
        "malformed TT move must not appear in the root PV"
    );
    assert_legal_pv(
        &root,
        &searcher.td.pv_table[0][..searcher.td.pv_len[0].min(MAX_PLY)],
    );
}

/// An illegal move stored as the root's TT move, deep and exact, never
/// reaches the answer or any line a whole iterative deepening prints.
#[test]
fn a_deep_exact_illegal_tt_move_never_reaches_a_line_or_the_answer() {
    let root = Board::from_fen("2k5/pp3pp1/5n2/2P5/bPP2P2/P3K3/6Pp/3Q1B1R w - - 0 23")
        .expect("valid tournament-derived FEN");
    let illegal = Move::from_uci("e3f4").expect("valid UCI move shape");
    assert!(!root.is_legal(illegal));
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    searcher.shared.tt.store(TtStore {
        key: root.hash(),
        depth: 10,
        score: 500,
        bound: Bound::Exact,
        mv: illegal,
        ply: 0,
        static_eval: VALUE_NONE,
        is_pv: true,
    });
    let mut options = SearchOptions::default();
    options.limits.depth = Some(6);
    let result = searcher.search(root.clone(), &options, true, || SearchEvent::None);
    assert!(root.is_legal(result.bestmove), "{}", result.bestmove);
    let lines = lines.lock().unwrap();
    let mut printed = 0;
    for line in lines.iter().filter(|line| line.contains(" pv ")) {
        let mut board = root.clone();
        for token in line.split(" pv ").nth(1).expect("a pv").split_whitespace() {
            let mv = board
                .parse_move(token)
                .unwrap_or_else(|| panic!("{token} is illegal in `{line}`"));
            board.make_move(mv);
        }
        printed += 1;
    }
    assert!(printed >= 6, "{printed} lines with a pv");
}

#[test]
fn ponder_move_can_be_recovered_from_tt_child() {
    let mut searcher = Searcher::default();
    let root = Board::default();
    let bestmove = root.parse_move("a2a3").expect("legal root move");
    let mut child = root.clone();
    child.make_move(bestmove);
    let ponder = child.parse_move("a7a6").expect("legal child move");
    searcher.shared.tt.store(TtStore {
        key: child.hash(),
        depth: 4,
        score: 0,
        bound: Bound::Exact,
        mv: ponder,
        ply: 1,
        static_eval: VALUE_NONE,
        is_pv: false,
    });

    assert_eq!(searcher.ponder_from_tt(&root, bestmove), ponder);
}

fn assert_legal_pv(root: &Board, pv: &[Move]) {
    let mut board = root.clone();
    for &mv in pv {
        if mv.is_null() {
            break;
        }
        let legal = board
            .parse_move(&mv.to_string())
            .unwrap_or_else(|| panic!("PV move {mv} is illegal in {}", board.to_fen()));
        board.make_move(legal);
    }
}
