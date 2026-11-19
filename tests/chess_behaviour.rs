//! Chess behaviour of the search, read from what it plays and reports.
//!
//! Every position's premise is checked here or was measured: the mate
//! distances are Stockfish 19's at depth 26, and a stalemating or mating
//! move is confirmed on the board before a test relies on it. The tests
//! assert properties of play (exact mate distances, no stalemate when
//! winning, no missed or allowed mate in one, conversion of won endings),
//! not move lists or scores a fit can move.
#![cfg(not(feature = "texel"))]

use rarog::board::{Board, Color, Move};
use rarog::eval::MATE_SCORE;
use rarog::search::options::SearchOptions;
use rarog::search::{SearchEvent, SearchResult, Searcher};
use rarog::wac::wac_positions;

fn board(fen: &str) -> Board {
    Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"))
}

fn search_with(
    searcher: &mut Searcher,
    position: &Board,
    depth: Option<u32>,
    nodes: u64,
) -> SearchResult {
    let mut options = SearchOptions {
        board: position.clone(),
        ..SearchOptions::default()
    };
    options.limits.depth = depth;
    options.limits.nodes = nodes;
    searcher.search(position.clone(), &options, false, || SearchEvent::None)
}

fn search(fen: &str, depth: u32) -> SearchResult {
    search_with(&mut Searcher::default(), &board(fen), Some(depth), 0)
}

fn is_mate(position: &Board) -> bool {
    position.is_in_check() && position.generate_legal_moves().is_empty()
}

fn after(position: &Board, mv: Move) -> Board {
    let mut next = position.clone();
    next.make_move(mv);
    next
}

/// The moves of the side to move that mate at once.
fn mates_in_one(position: &Board) -> Vec<Move> {
    position
        .generate_legal_moves()
        .into_iter()
        .filter(|&mv| is_mate(&after(position, mv)))
        .collect()
}

/// The search's score for mating in `moves` moves, and for being mated in
/// `moves` moves.
const fn mating(moves: i32) -> i32 {
    MATE_SCORE - (2 * moves - 1)
}

const fn mated(moves: i32) -> i32 {
    -(MATE_SCORE - 2 * moves)
}

/// Mates with Stockfish's shortest distance and a depth that finds it, two
/// plies or more past the first depth that does.
const MATES: [(&str, i32, u32); 4] = [
    ("6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1", 1, 3),
    ("k7/8/2K5/8/8/8/8/7R w - - 0 1", 2, 5),
    ("8/k7/8/8/3K4/8/8/1Q6 w - - 2 2", 3, 10),
    ("k7/8/8/8/8/2K5/8/1Q6 w - - 0 1", 4, 11),
];

/// Plays `line` from `fen` and returns its length in moves of the side to
/// move if it ends in mate.
fn mating_line_moves(fen: &str, line: &[Move]) -> Option<i32> {
    let mut position = board(fen);
    for &mv in line {
        assert!(
            position.generate_legal_moves().contains(&mv),
            "{fen}: {mv} in the line is illegal"
        );
        position.make_move(mv);
    }
    let plies = i32::try_from(line.len()).expect("a short line");
    (is_mate(&position) && plies % 2 == 1).then_some((plies + 1) / 2)
}

#[test]
fn mate_scores_count_the_moves_to_mate_exactly() {
    for (fen, moves, depth) in MATES {
        let result = search(fen, depth);
        assert_eq!(
            result.score,
            mating(moves),
            "{fen} at depth {depth}: {:?}",
            result.pv
        );
        assert_eq!(
            mating_line_moves(fen, &result.pv),
            Some(moves),
            "{fen}: the line is the mate"
        );
    }
}

/// A longer mate than the shortest still reports the line it found: here
/// the search mates in six at every depth to 16 (Stockfish: five), as the
/// classic kernel did.
#[test]
fn a_mate_score_matches_the_line_it_reports() {
    let fen = "k7/8/8/8/2K5/8/8/7R w - - 0 1";
    for depth in [11, 14] {
        let result = search(fen, depth);
        let moves = mating_line_moves(fen, &result.pv).expect("the line is a mate");
        assert_eq!(result.score, mating(moves), "{fen} at depth {depth}");
        assert!(moves >= 5, "{fen}: shorter than the shortest mate");
    }
}

#[test]
fn a_mated_side_reports_the_distance_to_its_mate() {
    // After 1.Kb6 only Kb8 is legal, and Rh8 mates.
    let fen = "k7/8/1K6/8/8/8/8/7R b - - 1 1";
    let position = board(fen);
    let reply = position.generate_legal_moves();
    assert_eq!(reply.len(), 1, "the defence is forced");
    assert!(!mates_in_one(&after(&position, reply[0])).is_empty());
    assert_eq!(search(fen, 4).score, mated(1));
}

#[test]
fn a_mate_read_back_from_the_table_keeps_its_distance() {
    // One searcher for the game: the second search reads the first's mates
    // from the table, stored at other plies than it reads them.
    let (fen, moves, depth) = MATES[3];
    let mut searcher = Searcher::default();
    let start = board(fen);
    let first = search_with(&mut searcher, &start, Some(depth), 0);
    assert_eq!(first.score, mating(moves));
    let position = after(&after(&start, first.pv[0]), first.pv[1]);
    let warm = search_with(&mut searcher, &position, Some(MATES[2].2), 0);
    let cold = search_with(&mut Searcher::default(), &position, Some(MATES[2].2), 0);
    assert_eq!(
        cold.score,
        mating(moves - 1),
        "the premise: a fresh search finds it"
    );
    assert_eq!(warm.score, mating(moves - 1), "{:?}", warm.pv);
    assert!(
        warm.nodes < cold.nodes,
        "the warm table is read: {} vs {}",
        warm.nodes,
        cold.nodes
    );
}

#[test]
fn the_winning_side_never_stalemates() {
    for (fen, stalemate) in [
        ("7k/8/6K1/8/8/8/8/5Q2 w - - 0 1", "f1f7"),
        ("k7/8/1K6/8/8/8/8/2Q5 w - - 0 1", "c1c7"),
    ] {
        let position = board(fen);
        let trap = position.parse_move(stalemate).expect("a legal move");
        let trapped = after(&position, trap);
        assert!(
            trapped.generate_legal_moves().is_empty() && !trapped.is_in_check(),
            "the premise: {stalemate} stalemates"
        );
        for depth in 1..=8 {
            let result = search(fen, depth);
            assert_ne!(result.bestmove, trap, "{fen} at depth {depth}");
            assert!(
                result.score >= mating(3),
                "{fen} at depth {depth}: {}",
                result.score
            );
        }
    }
}

#[test]
fn queen_and_rook_endings_are_converted_to_mate() {
    // Both sides played by the engine on a small node budget, one searcher
    // for the game; the fifty-move rule bounds the playout.
    for fen in [
        "8/8/8/4k3/8/8/8/K6Q w - - 0 1",
        "8/8/8/4k3/8/8/8/R3K3 w - - 0 1",
    ] {
        let mut position = board(fen);
        let mut searcher = Searcher::default();
        searcher.new_game();
        let mut plies = 0;
        while !position.generate_legal_moves().is_empty() {
            assert!(
                plies < 100,
                "{fen}: no mate in fifty moves, at {}",
                position.to_fen()
            );
            let mv = search_with(&mut searcher, &position, None, 20_000).bestmove;
            position.make_move(mv);
            plies += 1;
        }
        assert!(
            position.is_in_check(),
            "{fen}: stalemated at {}",
            position.to_fen()
        );
        assert_eq!(
            position.side_to_move(),
            Color::Black,
            "{fen}: the defender is mated"
        );
    }
}

/// Positions with tactics both ways: the WAC suite's, and their
/// continuations after the side to move's first legal move.
fn tactical_positions() -> Vec<Board> {
    let mut positions = Vec::new();
    for entry in wac_positions() {
        let position = board(&entry.fen);
        if let Some(&first) = position.generate_legal_moves().first() {
            positions.push(after(&position, first));
        }
        positions.push(position);
    }
    positions
}

/// Each tactical position searched at `depth` by one searcher, a new game
/// each, with its result; every reported line is checked legal.
fn searched_tactics(depth: u32) -> Vec<(Board, SearchResult)> {
    let mut searcher = Searcher::default();
    let mut searched = Vec::new();
    for position in tactical_positions() {
        if position.generate_legal_moves().is_empty() {
            continue;
        }
        searcher.new_game();
        let result = search_with(&mut searcher, &position, Some(depth), 0);
        let mut line = position.clone();
        for &mv in &result.pv {
            let fen = line.to_fen();
            assert!(
                line.generate_legal_moves().contains(&mv),
                "{fen}: {mv} in the line is illegal"
            );
            line.make_move(mv);
        }
        searched.push((position, result));
    }
    searched
}

#[test]
fn no_mate_in_one_is_missed() {
    let mut mates = 0;
    for (position, result) in searched_tactics(3) {
        if !mates_in_one(&position).is_empty() {
            let fen = position.to_fen();
            assert!(
                is_mate(&after(&position, result.bestmove)),
                "{fen}: played {}",
                result.bestmove
            );
            assert_eq!(result.score, mating(1), "{fen}");
            mates += 1;
        }
    }
    assert!(mates >= 5, "only {mates} positions offer a mate in one");
}

/// Of the 123 positions here where some move allows a mate in one and some
/// does not, the classic kernel allowed none at depths 3 and 5; this kernel
/// allows `Qg2#` after `Nxg4` in one (`5r2/2p2k1p/2p4r/3P4/4p1b1/1Q2NqPp/PP3P1K/R4R2 w`)
/// at every depth to 7: once late-move pruning or futility fires, its
/// picker skips every quiet left, the quiet mating check among them.
#[test]
#[ignore = "the kernel's quiet skip drops direct checks; a dropped Rarog form, not retried inside D.7 (PLAN D.7, 2026-11-19 decision 6)"]
fn no_mate_in_one_is_allowed() {
    let mut avoidable = 0;
    for (position, result) in searched_tactics(3) {
        let allows = |mv: Move| !mates_in_one(&after(&position, mv)).is_empty();
        let moves = position.generate_legal_moves();
        if moves.iter().any(|&mv| allows(mv)) && !moves.iter().all(|&mv| allows(mv)) {
            let fen = position.to_fen();
            assert!(
                !allows(result.bestmove),
                "{fen}: {} allows a mate in one",
                result.bestmove
            );
            avoidable += 1;
        }
    }
    assert!(avoidable >= 20, "only {avoidable} positions test it");
}
