//! The tablebase root, in-search probes, display and PV extension against the
//! committed KQvK and KRvK tables (`tests/fixtures/syzygy`), so they run in CI.
//! Tablebase state is global to the process, so these tests live in their own
//! binary and share one path.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rarog::board::{Board, Move};
use rarog::search::{InfoSink, SearchEvent, Searcher};
use rarog::search_options::SearchOptions;
use rarog::syzygy::{self, Wdl};

fn fixture_path() -> String {
    format!("{}/tests/fixtures/syzygy", env!("CARGO_MANIFEST_DIR"))
}

struct Recorder(Arc<Mutex<Vec<String>>>);

impl InfoSink for Recorder {
    fn line(&self, line: &str) {
        self.0.lock().expect("recorder lock").push(line.to_string());
    }
}

/// One search over the fixture: the raw output, the best move and the ponder
/// move.
fn search(fen: &str, configure: impl FnOnce(&mut SearchOptions)) -> (Vec<String>, Move, Move) {
    let output = Arc::new(Mutex::new(Vec::new()));
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&output))));
    let board = Board::from_fen(fen).expect("valid FEN");
    let mut options = SearchOptions {
        board: board.clone(),
        ..SearchOptions::default()
    };
    options.engine.syzygy.path = fixture_path();
    configure(&mut options);
    searcher.configure(&options.engine);
    assert_eq!(syzygy::largest(), 3, "the fixture holds 3-man tables");
    let result = searcher.search(board, &options, true, || SearchEvent::None);
    let raw = output.lock().expect("recorder lock").clone();
    (raw, result.bestmove, result.pondermove)
}

fn last_pv_line(raw: &[String]) -> &str {
    raw.iter()
        .rev()
        .find(|line| line.starts_with("info depth") && line.contains(" pv "))
        .expect("an info line with a PV")
}

fn score_of(line: &str) -> String {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let at = tokens.iter().position(|&t| t == "score").expect("score");
    format!("{} {}", tokens[at + 1], tokens[at + 2])
}

/// Play `line`'s PV from `fen`, asserting every move legal; the final board.
fn play_pv(fen: &str, line: &str) -> (Board, usize) {
    let mut board = Board::from_fen(fen).expect("valid FEN");
    let pv = line.split(" pv ").nth(1).expect("pv");
    let mut plies = 0;
    for uci in pv.split_whitespace() {
        let mv = board
            .parse_move(uci)
            .unwrap_or_else(|| panic!("illegal PV move {uci} in {line}"));
        board.make_move(mv);
        plies += 1;
    }
    (board, plies)
}

fn winning_moves(fen: &str) -> Vec<Move> {
    let board = Board::from_fen(fen).expect("valid FEN");
    board
        .generate_legal_moves()
        .iter()
        .copied()
        .filter(|&mv| {
            let mut after = board.clone();
            after.make_move(mv);
            syzygy::probe_wdl(&after, false) == Some(Wdl::Loss)
        })
        .collect()
}

// One test function so the global tables are loaded once, from one thread.
#[test]
fn fixture_tablebase_root_probes_display_and_extension() {
    kqvk_win_reports_the_tablebase_score_and_a_mate_line();
    kqvk_loss_reports_a_tablebase_loss_and_a_ponder_move();
    kqvk_near_the_rule50_horizon_shows_a_cursed_win();
    a_capture_into_the_tables_is_scored_through_in_search_probes();
    a_best_group_of_one_is_searched_to_the_requested_depth();
    multi_threaded_multipv_reports_distinct_legal_lines();
}

/// KQ v K, white to move: a DTZ-best move, a ponder move, `cp 20000` at a
/// depth too shallow to see the mate, and a PV extended to mate in exactly
/// the tables' distance (mate in 8, 15 plies; Stockfish 19 agrees).
fn kqvk_win_reports_the_tablebase_score_and_a_mate_line() {
    let fen = "4k3/8/8/8/8/8/8/3QK3 w - - 0 1";
    let (raw, bestmove, ponder) = search(fen, |o| o.limits.depth = Some(4));
    assert!(winning_moves(fen).contains(&bestmove), "{raw:#?}");
    assert!(!ponder.is_null(), "a ponder move: {raw:#?}");
    let line = last_pv_line(&raw);
    assert_eq!(score_of(line), "cp 20000", "{line}");
    let (end, plies) = play_pv(fen, line);
    assert!(
        end.is_in_check() && end.generate_legal_moves().is_empty(),
        "ends in mate: {line}"
    );
    assert_eq!(plies, 15, "{line}");
    let pv_second = line
        .split(" pv ")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap();
    assert_eq!(
        ponder.to_string(),
        pv_second,
        "ponder is the PV's second move"
    );
}

/// KQ v K, black to move: a tablebase loss, still a ponder move and a legal
/// PV to mate.
fn kqvk_loss_reports_a_tablebase_loss_and_a_ponder_move() {
    let fen = "4k3/8/8/8/8/8/8/3QK3 b - - 0 1";
    let (raw, _, ponder) = search(fen, |o| o.limits.depth = Some(4));
    assert!(!ponder.is_null(), "{raw:#?}");
    let line = last_pv_line(&raw);
    assert_eq!(score_of(line), "cp -20000", "{line}");
    let (end, _) = play_pv(fen, line);
    assert!(
        end.is_in_check() && end.generate_legal_moves().is_empty(),
        "{line}"
    );
}

/// KQ v K with the rule-50 counter at 92: the win cannot be completed in
/// time, so the root shows the cursed-win band, 1 to 49 centipawns.
fn kqvk_near_the_rule50_horizon_shows_a_cursed_win() {
    let fen = "4k3/8/8/8/8/8/8/3QK3 w - - 92 1";
    let (raw, _, _) = search(fen, |o| o.limits.depth = Some(6));
    let line = last_pv_line(&raw);
    let score = score_of(line);
    let cp: i32 = score
        .strip_prefix("cp ")
        .and_then(|cp| cp.parse().ok())
        .unwrap_or_else(|| panic!("a centipawn score: {line}"));
    assert!((1..=49).contains(&cp), "{line}");
}

/// A four-man root outside the fixture: the rook capture enters KQ v K, which
/// the in-search probe scores as a tablebase win the root then reports.
fn a_capture_into_the_tables_is_scored_through_in_search_probes() {
    let fen = "4k3/8/8/8/8/8/3r4/3QK3 w - - 0 1";
    let (raw, bestmove, _) = search(fen, |o| o.limits.depth = Some(6));
    let line = last_pv_line(&raw);
    assert!(
        ["d1d2", "e1d2"].contains(&bestmove.to_string().as_str()),
        "the rook is taken: {raw:#?}"
    );
    assert!(!line.contains(" tbhits 0 "), "in-search probes ran: {line}");
    let score = score_of(line);
    let decisive = score.starts_with("mate ")
        || score
            .strip_prefix("cp ")
            .and_then(|cp| cp.parse::<i32>().ok())
            .is_some_and(|cp| cp > 19_000);
    assert!(decisive, "a tablebase win or a mate: {line}");
}

/// KQ v K with one mating move: the best-ranked group is that move alone,
/// which is not a forced move, so `go depth 6` reaches depth 6.
fn a_best_group_of_one_is_searched_to_the_requested_depth() {
    let fen = "k7/8/1K6/8/8/8/8/3Q4 w - - 0 1";
    let (raw, bestmove, _) = search(fen, |o| o.limits.depth = Some(6));
    assert_eq!(bestmove.to_string(), "d1d8", "{raw:#?}");
    let deepest = raw
        .iter()
        .filter(|line| line.starts_with("info depth") && line.contains(" pv "))
        .filter_map(|line| line.split_whitespace().nth(2))
        .filter_map(|depth| depth.parse::<u32>().ok())
        .max()
        .expect("a depth");
    assert_eq!(deepest, 6, "{raw:#?}");
}

/// KR v K at Threads 4 and MultiPV 3: distinct, legal lines, each within the
/// best-ranked group.
fn multi_threaded_multipv_reports_distinct_legal_lines() {
    let fen = "8/8/8/4k3/8/8/8/R3K3 w - - 0 1";
    let (raw, bestmove, _) = search(fen, |o| {
        o.limits.depth = Some(6);
        o.engine.threads = 4;
        o.engine.multi_pv = 3;
    });
    let prefix = "info depth 6 ";
    let firsts: Vec<String> = raw
        .iter()
        .filter(|line| line.starts_with(prefix) && line.contains(" pv "))
        .map(|line| {
            play_pv(fen, line);
            line.split(" pv ")
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    assert!(!firsts.is_empty() && firsts.len() <= 3, "{raw:#?}");
    let mut distinct = firsts.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(distinct.len(), firsts.len(), "{firsts:?}");
    let winning = winning_moves(fen);
    assert!(
        firsts
            .iter()
            .all(|mv| winning.iter().any(|w| w.to_string() == *mv))
    );
    assert!(firsts.contains(&bestmove.to_string()));
}

/// `stop` or `ponderhit` written with `go ponder` in one write at a tablebase
/// root: exactly one `bestmove` per `go`, with a legal ponder move.
#[test]
fn a_tablebase_root_answers_each_go_once_under_ponder_races() {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    use std::sync::mpsc;

    let scale = if cfg!(debug_assertions) { 8 } else { 1 };
    let fen = "4k3/8/8/8/8/8/8/3QK3 w - - 0 1";
    for (follow, threads) in [("ponderhit", 1), ("stop", 1), ("ponderhit", 4), ("stop", 4)] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rarog"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Rarog starts");
        let mut stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        let batch = format!(
            "setoption name SyzygyPath value {}\nsetoption name Ponder value true\n\
             setoption name Threads value {threads}\nisready\nposition fen {fen} moves e1e2 e8e7\n\
             go ponder wtime 2000 btime 2000 winc 20 binc 20\n{follow}\n",
            fixture_path()
        );
        stdin.write_all(batch.as_bytes()).expect("write");
        stdin.flush().expect("flush");
        let deadline = Duration::from_secs(10 * scale);
        let mut bestmoves = Vec::new();
        let start = std::time::Instant::now();
        while start.elapsed() < deadline {
            match rx.recv_timeout(Duration::from_millis(200)) {
                Ok(line) if line.starts_with("bestmove") => bestmoves.push(line),
                Ok(_) => {}
                Err(_) if !bestmoves.is_empty() => break,
                Err(_) => {}
            }
        }
        // Nothing more may follow the answer.
        std::thread::sleep(Duration::from_millis(300 * scale));
        while let Ok(line) = rx.try_recv() {
            if line.starts_with("bestmove") {
                bestmoves.push(line);
            }
        }
        let _ = stdin.write_all(b"quit\n");
        let _ = child.wait();
        assert_eq!(
            bestmoves.len(),
            1,
            "{follow} at Threads {threads}: {bestmoves:?}"
        );
        let mut board = Board::from_fen(fen).expect("valid FEN");
        for uci in ["e1e2", "e8e7"] {
            let mv = board.parse_move(uci).expect("legal");
            board.make_move(mv);
        }
        let tokens: Vec<&str> = bestmoves[0].split_whitespace().collect();
        let best = board.parse_move(tokens[1]).expect("legal bestmove");
        if let Some(&ponder) = tokens.get(3) {
            board.make_move(best);
            assert!(board.parse_move(ponder).is_some(), "{bestmoves:?}");
        }
    }
}
