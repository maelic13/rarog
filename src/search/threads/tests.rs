use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};

use super::*;
use crate::eval::MATE_SCORE;
use crate::search::options::{HashRequest, SearchOptions};
use crate::search::{InfoReport, InfoSink};
use crate::uci::info::report_line;

const SELF_STOP_LIMIT: Duration = Duration::from_secs(30);

/// A helper's previous search scores are cleared at a new game, as the
/// main thread's are: its first window would otherwise open round the
/// last game's score.
#[test]
fn a_helpers_previous_scores_are_cleared_at_a_new_game() {
    let mut helper = Searcher::helper();
    let board = Board::default();
    let legal = board.generate_legal_moves();
    let limits = crate::search::options::SearchLimits {
        depth: Some(6),
        ..Default::default()
    };
    helper.reset_search_state(
        &limits,
        &EngineOptions::default(),
        board.side_to_move(),
        0,
        true,
    );
    helper.search_root(board, &legal, false, &mut || SearchEvent::None);
    helper.td.compact_previous_score = 250;
    helper.reset_thread_for_new_game();
    assert_eq!(helper.td.compact_previous_score, 0);
}

struct Recorder(Arc<Mutex<Vec<String>>>);

impl InfoSink for Recorder {
    fn line(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_string());
    }
    fn report(&self, report: &InfoReport<'_>) {
        self.line(&report_line(report));
    }
}

#[test]
fn parallel_result_selection_uses_weighted_helper_votes() {
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    let g1f3 = Move::from_uci("g1f3").expect("valid move");
    let results = vec![
        test_search_result(e2e4, 20, 5),
        test_search_result(d2d4, 18, 5),
        test_search_result(d2d4, 16, 5),
    ];

    let (index, selected) =
        select_parallel_result(&results, &[e2e4, d2d4, g1f3]).expect("selected result");

    assert_eq!(selected.bestmove, d2d4);
    assert_ne!(
        index, 0,
        "the winner here is a helper, and the caller must know"
    );
}

#[test]
fn parallel_result_selection_prefers_decisive_win() {
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    let results = vec![
        test_search_result(e2e4, 900, 12),
        test_search_result(d2d4, TB_WIN_SCORE, 4),
    ];

    let (index, selected) =
        select_parallel_result(&results, &[e2e4, d2d4]).expect("selected result");

    assert_eq!(selected.bestmove, d2d4);
    assert_eq!(index, 1);
}

/// A helper that takes `delay` over its start-up and each `NewGame`,
/// counting each one it finishes.
fn slow_worker(delay: Duration, finished: &Arc<AtomicUsize>) -> SearchWorkerHandle {
    let finished = Arc::clone(finished);
    let (sender, receiver) = mpsc::channel();
    let (ack, acks) = mpsc::channel();
    let handle = thread::spawn(move || {
        let finish = || {
            thread::sleep(delay);
            finished.fetch_add(1, Ordering::SeqCst);
            let _ = ack.send(());
        };
        finish();
        while let Ok(message) = receiver.recv() {
            match message {
                WorkerMessage::NewGame => finish(),
                WorkerMessage::Shutdown => break,
                WorkerMessage::Search(_) => {}
            }
        }
    });
    SearchWorkerHandle {
        sender,
        acks,
        handle: Some(handle),
    }
}

/// `readyok` follows these calls, so a slow helper must hold them back.
#[test]
fn the_pool_waits_for_slow_helpers_to_start_and_to_clear() {
    const DELAY: Duration = Duration::from_millis(100);
    let finished = Arc::new(AtomicUsize::new(0));
    let mut pool = WorkerPool::default();

    let helpers = pool.set_helper_count_with(3, |_| Some(slow_worker(DELAY, &finished)));
    assert_eq!(helpers, 3);
    assert_eq!(finished.load(Ordering::SeqCst), 3, "started");

    pool.start_new_game();
    pool.finish_new_game();
    assert_eq!(finished.load(Ordering::SeqCst), 6, "cleared");

    pool.set_helper_count_with(4, |_| Some(slow_worker(DELAY, &finished)));
    assert_eq!(
        finished.load(Ordering::SeqCst),
        7,
        "only the new helper is waited for"
    );
}

#[test]
fn a_helper_that_died_does_not_hang_the_pool() {
    let dead = || {
        let (sender, _) = mpsc::channel();
        let (_, acks) = mpsc::channel();
        Some(SearchWorkerHandle {
            sender,
            acks,
            handle: None,
        })
    };
    let mut pool = WorkerPool::default();

    assert_eq!(pool.set_helper_count_with(2, |_| dead()), 2);
    pool.start_new_game();
    pool.finish_new_game();
}

#[test]
fn a_helper_keeps_only_the_smallest_table() {
    assert_eq!(Searcher::helper().shared.tt.allocated_bytes(), 1 << 20);
}

/// Three threads on a longer mate outvote the one that found the shortest,
/// which decides all the same; so does the nearest mate when all are lost.
#[test]
fn the_shortest_decisive_result_wins_whatever_the_vote_says() {
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    for sign in [1, -1] {
        let results = vec![
            test_search_result(d2d4, sign * (MATE_SCORE - 9), 12),
            test_search_result(d2d4, sign * (MATE_SCORE - 9), 12),
            test_search_result(d2d4, sign * (MATE_SCORE - 9), 12),
            test_search_result(e2e4, sign * (MATE_SCORE - 3), 8),
        ];

        let (index, selected) =
            select_parallel_result(&results, &[e2e4, d2d4]).expect("selected result");

        assert_eq!(index, 3, "sign {sign}");
        assert_eq!(selected.bestmove, e2e4);
    }
}

/// The winner's own line is what the pool reports, so a result carries the
/// PV and seldepth of the thread that produced it.
#[test]
fn a_helper_result_carries_the_line_the_pool_would_report() {
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    let mut helper = test_search_result(d2d4, 30, 9);
    helper.pv = vec![d2d4, e2e4];
    helper.seldepth = 17;
    let results = vec![test_search_result(e2e4, 10, 9), helper];

    let (index, selected) =
        select_parallel_result(&results, &[e2e4, d2d4]).expect("selected result");

    assert_eq!(index, 1);
    assert_eq!(selected.pv, vec![d2d4, e2e4]);
    assert_eq!(selected.seldepth, 17);
}

fn depth_limit(depth: u32) -> SearchLimits {
    SearchLimits {
        depth: Some(depth),
        ..SearchLimits::default()
    }
}

/// Helpers ahead of the main thread and behind it both outvote it, and
/// under a depth limit neither answers.
#[test]
fn a_depth_limit_is_answered_by_the_main_thread_whatever_the_vote_says() {
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    let root_moves = [e2e4, d2d4];
    for helper_depth in [3, 7] {
        let results = vec![
            test_search_result(e2e4, 10, 6),
            test_search_result(d2d4, 300, helper_depth),
            test_search_result(d2d4, 300, helper_depth),
            test_search_result(d2d4, 300, helper_depth),
        ];
        let (voted, _) =
            pool_answer(&results, &root_moves, 1, &SearchLimits::default()).expect("answered");
        assert_ne!(voted, 0, "on the clock the vote goes to a helper");

        let (index, selected) =
            pool_answer(&results, &root_moves, 1, &depth_limit(6)).expect("answered");
        assert_eq!(index, 0);
        assert_eq!((selected.bestmove, selected.depth), (e2e4, 6));
    }
}

#[test]
fn more_than_one_line_is_answered_by_the_main_thread() {
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    let results = vec![
        test_search_result(e2e4, 10, 6),
        test_search_result(d2d4, 300, 9),
    ];

    let (index, _) =
        pool_answer(&results, &[e2e4, d2d4], 2, &SearchLimits::default()).expect("answered");

    assert_eq!(index, 0);
}

/// The depth limit decides alone, as in Stockfish: a main thread stopped
/// inside its first iteration still answers, with the first root move.
#[test]
fn a_depth_limit_is_the_main_threads_even_before_its_first_iteration_ends() {
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    let results = vec![
        test_search_result(e2e4, -INF_SCORE, 0),
        test_search_result(d2d4, 20, 2),
    ];

    let (index, selected) =
        pool_answer(&results, &[e2e4, d2d4], 1, &depth_limit(6)).expect("answered");

    assert_eq!(index, 0);
    assert_eq!(selected.bestmove, e2e4);
}

/// `go mate` is not a depth limit: a mate only a helper found is the answer.
#[test]
fn a_mate_search_is_answered_by_the_vote() {
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let d2d4 = Move::from_uci("d2d4").expect("valid move");
    let results = vec![
        test_search_result(e2e4, 40, 5),
        test_search_result(d2d4, TB_WIN_SCORE, 4),
    ];
    let limits = SearchLimits {
        mate: Some(3),
        ..SearchLimits::default()
    };

    let (index, selected) = pool_answer(&results, &[e2e4, d2d4], 1, &limits).expect("answered");

    assert_eq!(index, 1);
    assert_eq!(selected.bestmove, d2d4);
}

const MATE_IN_2: &str = "k7/8/2K5/8/8/8/8/7R w - - 0 1";
const MATED_IN_1: &str = "7k/8/5K2/8/8/8/6Q1/8 b - - 0 1";

/// A search meant to end itself: it stops here instead of hanging the
/// suite, and the elapsed time says which it was.
fn within_a_limit() -> impl FnMut() -> SearchEvent {
    let start = Instant::now();
    move || {
        if start.elapsed() > SELF_STOP_LIMIT {
            SearchEvent::Stop
        } else {
            SearchEvent::None
        }
    }
}

fn mate_search(fen: &str, mate: u32, threads: usize) -> SearchResult {
    let mut options = SearchOptions::default();
    options.engine.hash_mb = 1;
    options.engine.threads = threads;
    options.limits.mate = Some(mate);
    let mut searcher = Searcher::default();
    searcher.configure(&options.engine, HashRequest::Sent);
    let board = Board::from_fen(fen).expect("valid FEN");
    searcher.search(board, &options, false, within_a_limit())
}

#[test]
fn a_mate_search_ends_itself_on_the_proof_for_either_side() {
    for threads in [1, 4] {
        let start = Instant::now();
        assert_eq!(
            mate_search(MATE_IN_2, 2, threads).score,
            MATE_SCORE - 3,
            "Threads {threads}"
        );
        assert_eq!(
            mate_search(MATED_IN_1, 1, threads).score,
            2 - MATE_SCORE,
            "Threads {threads}"
        );
        assert!(start.elapsed() < SELF_STOP_LIMIT, "Threads {threads}");
    }
}

/// Either side's mate within the limit ends the search at once. The same
/// mate beyond a tighter limit ends nothing: the search goes on past it,
/// here as far as a search goes in so small a position.
#[test]
fn only_a_mate_within_the_limit_ends_the_search() {
    let won = mate_search(MATE_IN_2, 2, 1);
    let lost = mate_search(MATED_IN_1, 1, 1);
    let beyond = mate_search(MATE_IN_2, 1, 1);
    assert!(won.depth < 10, "{}", won.depth);
    assert!(lost.depth < 10, "{}", lost.depth);
    assert!(beyond.depth > 50, "{}", beyond.depth);
    assert_eq!(beyond.score, MATE_SCORE - 3);
}

/// A helper's proof ends the pool's search: the main thread need not find
/// the mate itself.
#[test]
fn a_helpers_proof_stops_the_pool() {
    let board = Board::from_fen(MATE_IN_2).expect("valid FEN");
    let moves = board.generate_legal_moves();
    let context = Arc::new(SharedContext::new(0, moves.len(), 2));
    let mut helper = Searcher::default();
    helper.shared.join_pool(Arc::clone(&context));
    helper.td.thread_id = 1;
    let limits = SearchLimits {
        mate: Some(2),
        ..SearchLimits::default()
    };

    let result = helper.search_worker(
        board,
        &limits,
        &EngineOptions::default(),
        &moves,
        TbRootDecision::default(),
        &mut within_a_limit(),
    );

    assert_eq!(result.score, MATE_SCORE - 3);
    assert_eq!(context.stop_state.load(Ordering::Relaxed), STOP_SEARCH);
}

/// A mate search ends on a completed iteration, where each of the helper's
/// three lines holds an exact score at that depth, so the pool holds at
/// least three. A move that raised alpha before a better one overtook it
/// in its line is exact too, so the count can exceed the lines; one line
/// leaves two here. Bounds are not ranked against exact scores: a move
/// that fails low against a mate publishes that mate as its upper bound.
#[test]
fn a_helper_searches_every_line() {
    let board = Board::from_fen(MATE_IN_2).expect("valid FEN");
    let moves = board.generate_legal_moves();
    let context = Arc::new(SharedContext::new(0, moves.len(), 2));
    let mut helper = Searcher::default();
    helper.shared.join_pool(Arc::clone(&context));
    helper.td.thread_id = 1;
    let limits = SearchLimits {
        mate: Some(2),
        ..SearchLimits::default()
    };
    let engine = EngineOptions {
        multi_pv: 3,
        ..EngineOptions::default()
    };

    let result = helper.search_worker(
        board,
        &limits,
        &engine,
        &moves,
        TbRootDecision::default(),
        &mut within_a_limit(),
    );

    let depth = i32::try_from(result.depth).expect("a small depth");
    let scored: Vec<Option<(i32, i32, RootBound)>> = (0..moves.len())
        .map(|index| context.root_score(index))
        .collect();
    let exact = scored
        .iter()
        .filter(|score| matches!(score, Some((at, _, RootBound::Exact)) if *at == depth))
        .count();
    assert!(exact >= 3, "depth {depth}: {scored:?}");
}

#[test]
fn a_helpers_answer_is_printed_even_for_the_move_already_shown() {
    let e2e4 = Move::from_uci("e2e4").expect("valid move");
    let lines = Arc::new(Mutex::new(Vec::new()));
    let searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    searcher.td.last_reported_move.set(e2e4);
    let answer = test_search_result(e2e4, MATE_SCORE - 3, 5);

    searcher.report_pool_answer(0, &answer);
    assert!(lines.lock().unwrap().is_empty(), "the main thread said so");

    searcher.report_pool_answer(2, &answer);
    let printed = lines.lock().unwrap();
    assert_eq!(printed.len(), 1);
    assert!(printed[0].contains("score mate 2"), "{}", printed[0]);
}

/// The whole path at four threads: the result and the last line printed
/// both name the depth asked for, whatever the helpers reached. Helpers
/// race, so this catches a wrong answer only some of the time; the tests
/// on `pool_answer` above are the ones that always do.
#[test]
fn go_depth_at_four_threads_reports_that_depth() {
    const DEPTH: usize = 6;
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut options = SearchOptions::default();
    options.engine.hash_mb = 1;
    options.engine.threads = 4;
    options.limits.depth = Some(6);
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    searcher.configure(&options.engine, HashRequest::Sent);

    for run in 0..30 {
        lines.lock().unwrap().clear();
        searcher.new_game();
        let result = searcher.search(Board::default(), &options, true, || SearchEvent::None);
        assert_eq!(result.depth, DEPTH, "run {run}");
        let printed = lines.lock().unwrap();
        let last = printed
            .iter()
            .rfind(|line| line.starts_with("info depth"))
            .expect("a depth line");
        assert!(
            last.starts_with(&format!("info depth {DEPTH} ")),
            "run {run}: {last}"
        );
    }
}

/// Every helper has let go of the table by the time the search returns,
/// so the main thread's handle is the only one, and a `Hash` change frees
/// the old table at once.
#[test]
fn a_helper_holds_the_table_only_for_its_job() {
    let mut options = SearchOptions::default();
    options.engine.hash_mb = 1;
    options.engine.threads = 4;
    options.limits.depth = Some(4);
    let mut searcher = Searcher::default();
    searcher.configure(&options.engine, HashRequest::Sent);

    for _ in 0..3 {
        let result = searcher.search(Board::default(), &options, false, || SearchEvent::None);
        assert!(!result.bestmove.is_null());
        assert_eq!(searcher.shared.tt.shared_handles(), 1);
    }

    options.engine.hash_mb = 2;
    searcher.configure(&options.engine, HashRequest::Sent);
    let result = searcher.search(Board::default(), &options, false, || SearchEvent::None);
    assert!(!result.bestmove.is_null(), "the resized table searches");
    assert_eq!(searcher.shared.tt.shared_handles(), 1);
}

fn test_search_result(bestmove: Move, score: i32, depth: usize) -> SearchResult {
    SearchResult {
        bestmove,
        pondermove: Move::NULL,
        score,
        depth,
        pv: vec![bestmove],
        seldepth: depth,
        nodes: 0,
        tb_hits: 0,
        elapsed_ms: 0,
        exit: SearchExit::Stop,
        ponderhit: false,
    }
}
