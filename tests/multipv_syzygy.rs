//! MultiPV against a tablebase-filtered root. Tablebase state is global to
//! the process, so these tests live in their own binary. They need tables:
//! set `RAROG_SYZYGY_PATH` to a directory holding at least the 4-man set, or
//! they skip.

use std::sync::{Arc, Mutex};

use rarog::board::Board;
use rarog::search::{InfoSink, SearchEvent, Searcher};
use rarog::search_options::SearchOptions;
use rarog::syzygy::{self, Wdl};

struct Recorder(Arc<Mutex<Vec<String>>>);

impl InfoSink for Recorder {
    fn line(&self, line: &str) {
        self.0.lock().expect("recorder lock").push(line.to_string());
    }
}

fn tables() -> Option<String> {
    let path = std::env::var("RAROG_SYZYGY_PATH").ok()?;
    if syzygy::initialize(&path) < 4 {
        eprintln!("skipped: RAROG_SYZYGY_PATH={path} holds no 4-man tables");
        return None;
    }
    Some(path)
}

/// Search `fen` at `lines` lines and return the first moves of the deepest
/// report, the raw output and the best move.
fn search(path: &str, fen: &str, lines: usize) -> (Vec<String>, Vec<String>, String) {
    let output = Arc::new(Mutex::new(Vec::new()));
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&output))));
    let board = Board::from_fen(fen).expect("valid FEN");
    let mut options = SearchOptions {
        board: board.clone(),
        ..SearchOptions::default()
    };
    options.engine.syzygy.path = path.to_string();
    options.engine.multi_pv = lines;
    options.limits.depth = Some(5);
    searcher.configure(&options.engine);
    let result = searcher.search(board, &options, true, || SearchEvent::None);
    let raw = output.lock().expect("recorder lock").clone();
    // A single root move stops early, so read the deepest depth reported.
    let deepest = raw
        .iter()
        .filter(|line| line.contains(" pv "))
        .filter_map(|line| line.split_whitespace().nth(2))
        .filter_map(|depth| depth.parse::<u32>().ok())
        .max()
        .expect("a depth was reported");
    // Every line of the deepest report, in `multipv` order.
    let prefix = format!("info depth {deepest} ");
    let firsts = raw
        .iter()
        .filter(|line| line.starts_with(&prefix))
        .filter_map(|line| line.split(" pv ").nth(1))
        .filter_map(|pv| pv.split_whitespace().next())
        .map(str::to_string)
        .collect();
    (firsts, raw, result.bestmove.to_string())
}

#[test]
fn a_won_tablebase_root_reports_its_best_ranked_group() {
    let Some(path) = tables() else { return };
    // KQ v K: distance to zeroing is distance to mate, so the root keeps the
    // fastest wins only, a strict subset of the winning moves, and reports
    // one line per kept move however many lines are asked for.
    let fen = "8/8/8/4k3/8/8/8/3QK3 w - - 0 1";
    let board = Board::from_fen(fen).expect("valid FEN");
    let winning: Vec<String> = board
        .generate_legal_moves()
        .iter()
        .filter(|&&mv| {
            let mut after = board.clone();
            after.make_move(mv);
            syzygy::probe_wdl(&after, false) == Some(Wdl::Loss)
        })
        .map(ToString::to_string)
        .collect();
    let (firsts, raw, bestmove) = search(&path, fen, 40);
    assert!(!firsts.is_empty(), "{raw:#?}");
    assert!(
        firsts.len() < winning.len(),
        "DTZ ranking must keep fewer than the {} winning moves: {firsts:?}",
        winning.len()
    );
    assert!(firsts.iter().all(|mv| winning.contains(mv)), "{firsts:?}");
    assert!(firsts.contains(&bestmove), "{raw:#?}");
    assert!(
        raw.iter().any(|line| !line.contains(" tbhits 0 ")),
        "{raw:#?}"
    );
    assert!(
        Board::from_fen(fen)
            .unwrap()
            .parse_move(&bestmove)
            .is_some()
    );
}

#[test]
fn a_drawn_tablebase_root_reports_only_the_drawing_moves() {
    let Some(path) = tables() else { return };
    // KR v KR: rook moves that hang the rook lose, the rest hold the draw.
    let fen = "4k3/8/8/8/8/8/r7/4K2R w - - 0 1";
    let board = Board::from_fen(fen).expect("valid FEN");
    let legal = board.generate_legal_moves();
    let drawing: Vec<String> = legal
        .iter()
        .filter(|&&mv| {
            let mut after = board.clone();
            after.make_move(mv);
            syzygy::probe_wdl(&after, false) == Some(Wdl::Draw)
        })
        .map(ToString::to_string)
        .collect();
    assert!(
        drawing.len() > 1 && drawing.len() < legal.len(),
        "the root filter must bite: {drawing:?} of {}",
        legal.len()
    );

    let (firsts, raw, bestmove) = search(&path, fen, legal.len() + 5);
    // The deepest report lists each drawing move once and nothing else.
    let report = &firsts;
    let mut sorted_report = report.clone();
    sorted_report.sort();
    let mut sorted_drawing = drawing.clone();
    sorted_drawing.sort();
    assert_eq!(sorted_report, sorted_drawing, "{raw:#?}");
    assert!(drawing.contains(&bestmove));
    assert_eq!(report[0], bestmove);
}

/// The record's six-man positions (B.5.2): each reports a tablebase win, a
/// ponder move and a PV legal to its end. Local only: needs the 6-man set.
#[test]
fn the_six_man_record_positions_report_a_win_a_ponder_move_and_a_legal_line() {
    let Some(path) = tables() else { return };
    if syzygy::largest() < 6 {
        eprintln!("skipped: RAROG_SYZYGY_PATH={path} holds no 6-man tables");
        return;
    }
    for fen in [
        "7r/5R2/8/2k1PB2/8/4K3/8/8 w - - 0 86",
        "8/P4k2/8/1N6/1P2B1K1/8/8/8 w - - 7 81",
        "1r6/R7/6k1/8/8/5PP1/6K1/8 w - - 6 72",
    ] {
        let output = Arc::new(Mutex::new(Vec::new()));
        let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&output))));
        let board = Board::from_fen(fen).expect("valid FEN");
        let mut options = SearchOptions {
            board: board.clone(),
            ..SearchOptions::default()
        };
        options.engine.syzygy.path = path.clone();
        options.limits.depth = Some(10);
        searcher.configure(&options.engine);
        let result = searcher.search(board.clone(), &options, true, || SearchEvent::None);
        let raw = output.lock().expect("recorder lock").clone();
        let line = raw
            .iter()
            .rev()
            .find(|line| line.starts_with("info depth") && line.contains(" pv "))
            .expect("an info line");
        assert!(
            line.contains(" score cp 20000 ") || line.contains(" score mate "),
            "{fen}: {line}"
        );
        assert!(!result.pondermove.is_null(), "{fen}: {raw:#?}");
        let mut played = board.clone();
        let pv = line.split(" pv ").nth(1).expect("pv");
        assert!(pv.split_whitespace().count() > 1, "{fen}: {line}");
        for uci in pv.split_whitespace() {
            let mv = played
                .parse_move(uci)
                .unwrap_or_else(|| panic!("{fen}: illegal {uci} in {line}"));
            played.make_move(mv);
        }
    }
}
