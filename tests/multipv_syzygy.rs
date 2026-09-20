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
    // Every line carries `multipv` now, so a second reported line shows up as
    // an index above 1 rather than as the token's presence.
    let prefix = format!("info depth {deepest} ");
    let firsts = raw
        .iter()
        .filter(|line| line.starts_with(&prefix))
        .filter(|line| !line.contains(" multipv 1 "))
        .filter_map(|line| line.split(" pv ").nth(1))
        .filter_map(|pv| pv.split_whitespace().next())
        .map(str::to_string)
        .collect();
    (firsts, raw, result.bestmove.to_string())
}

#[test]
fn a_won_tablebase_root_reports_one_line() {
    let Some(path) = tables() else { return };
    // KQ v K: the root keeps only the tablebase's preferred winning move.
    let fen = "8/8/8/4k3/8/8/8/3QK3 w - - 0 1";
    let (firsts, raw, bestmove) = search(&path, fen, 5);
    assert!(
        firsts.is_empty(),
        "one reported line means no index above 1: {raw:#?}"
    );
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
