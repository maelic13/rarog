//! MultiPV through the public API: line count, order, legality, `searchmoves`
//! and threads, read from the `info` lines the search writes to its sink.

use std::sync::{Arc, Mutex};

use rarog::board::{Board, Move};
use rarog::search::{InfoSink, SearchEvent, Searcher};
use rarog::search_options::SearchOptions;

struct Recorder(Arc<Mutex<Vec<String>>>);

impl InfoSink for Recorder {
    fn line(&self, line: &str) {
        self.0.lock().expect("recorder lock").push(line.to_string());
    }
}

/// One reported line: `(depth, multipv index, comparable score, bound, pv)`.
#[derive(Debug, Clone)]
struct Line {
    depth: u32,
    index: usize,
    score: i64,
    bounded: bool,
    pv: Vec<String>,
}

fn token_after<'a>(tokens: &'a [&'a str], name: &str) -> Option<&'a str> {
    tokens
        .iter()
        .position(|token| *token == name)
        .and_then(|at| tokens.get(at + 1).copied())
}

fn parse(line: &str) -> Option<Line> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let index = token_after(&tokens, "multipv")?.parse().ok()?;
    let depth = token_after(&tokens, "depth")?.parse().ok()?;
    let score_at = tokens.iter().position(|token| *token == "score")?;
    let value: i64 = tokens.get(score_at + 2)?.parse().ok()?;
    // Mates rank beyond every centipawn score, a nearer mate higher.
    let score = match *tokens.get(score_at + 1)? {
        "mate" if value > 0 => 1_000_000 - value,
        "mate" => -1_000_000 - value,
        _ => value,
    };
    let bounded = tokens.contains(&"lowerbound") || tokens.contains(&"upperbound");
    let pv_at = tokens.iter().position(|token| *token == "pv")?;
    let pv = tokens[pv_at + 1..]
        .iter()
        .map(|token| (*token).to_string())
        .collect();
    Some(Line {
        depth,
        index,
        score,
        bounded,
        pv,
    })
}

/// Search `fen` to `depth` with `configure` applied and return the lines of
/// the last reported depth and the best move.
fn search_lines(
    fen: &str,
    depth: u32,
    configure: impl FnOnce(&mut SearchOptions),
) -> (Vec<Line>, Move, Vec<String>) {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
    let board = Board::from_fen(fen).expect("valid FEN");
    let mut options = SearchOptions {
        board: board.clone(),
        ..SearchOptions::default()
    };
    options.limits.depth = Some(depth);
    configure(&mut options);
    searcher.configure(&options.engine);
    let result = searcher.search(board, &options, true, || SearchEvent::None);
    let raw = lines.lock().expect("recorder lock").clone();
    let parsed: Vec<Line> = raw.iter().filter_map(|line| parse(line)).collect();
    let last_depth = parsed.iter().map(|line| line.depth).max().unwrap_or(0);
    let last = parsed
        .into_iter()
        .filter(|line| line.depth == last_depth)
        .collect::<Vec<_>>();
    // Keep only the final report of that depth: indices restart at 1.
    let start = last.iter().rposition(|line| line.index == 1).unwrap_or(0);
    (last.into_iter().skip(start).collect(), result.bestmove, raw)
}

fn assert_legal(fen: &str, pv: &[String]) {
    let mut board = Board::from_fen(fen).expect("valid FEN");
    for uci in pv {
        let mv = board
            .parse_move(uci)
            .unwrap_or_else(|| panic!("PV move {uci} is illegal in {}", board.to_fen()));
        board.make_move(mv);
    }
}

fn assert_well_formed(fen: &str, lines: &[Line], expected: usize) {
    assert_eq!(lines.len(), expected, "{lines:#?}");
    for (rank, line) in lines.iter().enumerate() {
        assert_eq!(line.index, rank + 1, "{lines:#?}");
        assert!(
            !line.bounded,
            "a completed depth carries no bounds: {lines:#?}"
        );
        assert!(!line.pv.is_empty());
        assert_legal(fen, &line.pv);
    }
    let firsts: Vec<&String> = lines.iter().map(|line| &line.pv[0]).collect();
    for (at, first) in firsts.iter().enumerate() {
        assert!(
            !firsts[..at].contains(first),
            "distinct first moves: {firsts:?}"
        );
    }
    assert!(
        lines.windows(2).all(|pair| pair[0].score >= pair[1].score),
        "scores non-increasing: {lines:#?}"
    );
}

const MIDDLEGAME: &str = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";

#[test]
fn three_lines_are_distinct_ordered_and_legal_and_line_one_is_the_move() {
    let (lines, bestmove, _) = search_lines(MIDDLEGAME, 7, |options| {
        options.engine.multi_pv = 3;
    });
    assert_well_formed(MIDDLEGAME, &lines, 3);
    assert_eq!(bestmove.to_string(), lines[0].pv[0]);
}

#[test]
fn more_lines_than_legal_moves_reports_every_legal_move_once() {
    let fen = "k7/8/8/8/8/8/8/K7 w - - 0 1";
    let legal = Board::from_fen(fen)
        .expect("valid FEN")
        .generate_legal_moves()
        .len();
    assert_eq!(legal, 3);
    let (lines, bestmove, _) = search_lines(fen, 6, |options| {
        options.engine.multi_pv = 10;
    });
    assert_well_formed(fen, &lines, legal);
    assert_eq!(bestmove.to_string(), lines[0].pv[0]);
}

#[test]
fn lines_are_drawn_from_the_searchmoves_set() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let requested = ["e2e4", "d2d4"];
    let (lines, bestmove, _) = search_lines(fen, 6, |options| {
        options.engine.multi_pv = 3;
        options.limits.search_moves = requested
            .iter()
            .map(|uci| Move::from_uci(uci).expect("valid move"))
            .collect();
    });
    assert_well_formed(fen, &lines, 2);
    for line in &lines {
        assert!(requested.contains(&line.pv[0].as_str()), "{lines:#?}");
    }
    assert_eq!(bestmove.to_string(), lines[0].pv[0]);
}

#[test]
fn one_line_prints_no_multipv_token() {
    let (lines, _, raw) = search_lines(MIDDLEGAME, 5, |options| {
        options.engine.multi_pv = 1;
    });
    assert!(lines.is_empty());
    assert!(raw.iter().any(|line| line.starts_with("info depth 5 ")));
    assert!(raw.iter().all(|line| !line.contains("multipv")), "{raw:#?}");
}

#[test]
fn a_stopped_depth_carries_over_only_lines_the_previous_depth_reported() {
    let mut carried = 0;
    for step in 1..=12 {
        let (_, bestmove, raw) = search_lines(MIDDLEGAME, 64, |options| {
            options.engine.multi_pv = 4;
            options.limits.nodes = step * 7_919;
        });
        let parsed: Vec<Line> = raw.iter().filter_map(|line| parse(line)).collect();
        let starts: Vec<usize> = parsed
            .iter()
            .enumerate()
            .filter(|(_, line)| line.index == 1)
            .map(|(at, _)| at)
            .collect();
        let last = *starts.last().expect("a report");
        let final_report = &parsed[last..];
        let previous_report = starts
            .len()
            .checked_sub(2)
            .map(|at| &parsed[starts[at]..last]);
        assert_eq!(bestmove.to_string(), final_report[0].pv[0]);
        let current = final_report[0].depth;
        let mut firsts = Vec::new();
        for (rank, line) in final_report.iter().enumerate() {
            assert_eq!(line.index, rank + 1, "{final_report:#?}");
            assert!(!firsts.contains(&line.pv[0]), "{final_report:#?}");
            firsts.push(line.pv[0].clone());
            assert_legal(MIDDLEGAME, &line.pv);
            if line.depth < current {
                carried += 1;
                let previous = previous_report.expect("a carried line has a previous report");
                assert!(
                    !line.bounded
                        && previous.iter().any(|earlier| !earlier.bounded
                            && earlier.depth == line.depth
                            && earlier.score == line.score
                            && earlier.pv == line.pv),
                    "carried line not in the previous report: {line:#?}\n{previous:#?}"
                );
            }
        }
    }
    assert!(
        carried > 0,
        "no budget stopped a depth after its first line"
    );
}

#[test]
fn four_threads_with_three_lines_report_distinct_legal_lines() {
    let (lines, bestmove, _) = search_lines(MIDDLEGAME, 8, |options| {
        options.engine.multi_pv = 3;
        options.engine.threads = 4;
    });
    assert_well_formed(MIDDLEGAME, &lines, 3);
    assert_eq!(bestmove.to_string(), lines[0].pv[0]);
}
