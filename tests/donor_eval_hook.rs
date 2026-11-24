//! The `donor-eval` hook (E.4.0.1): an installed evaluation replaces
//! Rarog's, and the search hands it the root's last score, none before a
//! search's first root window returns.
#![cfg(feature = "donor-eval")]

use std::sync::Mutex;

use rarog::board::Board;
use rarog::eval::{Evaluator, install_donor};
use rarog::search::options::SearchOptions;
use rarog::search::{SearchEvent, Searcher};

/// Every root value the donor was handed, in call order.
static ROOTS: Mutex<Vec<Option<i32>>> = Mutex::new(Vec::new());

const DONOR_VALUE: i32 = 37;

fn recording_donor(_board: &Board, root: Option<i32>) -> i32 {
    ROOTS
        .lock()
        .expect("the recorder is not poisoned")
        .push(root);
    DONOR_VALUE
}

fn search_roots(board: &Board) -> Vec<Option<i32>> {
    ROOTS.lock().expect("the recorder is not poisoned").clear();
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.limits.depth = Some(5);
    searcher.search(board.clone(), &options, false, || SearchEvent::None);
    ROOTS.lock().expect("the recorder is not poisoned").clone()
}

#[test]
fn an_installed_donor_replaces_the_evaluation_and_reads_the_root() {
    let board =
        Board::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4")
            .expect("a legal FEN");
    let native = Evaluator::default().evaluate(&board);
    assert_ne!(native, DONOR_VALUE, "the fixture must tell the two apart");
    assert!(install_donor(recording_donor));
    assert!(!install_donor(recording_donor), "installed once");
    assert_eq!(Evaluator::default().evaluate(&board), DONOR_VALUE);

    for search in 0..2 {
        let roots = search_roots(&board);
        assert!(!roots.is_empty(), "search {search} evaluated nothing");
        assert_eq!(roots[0], None, "search {search} starts with no root score");
        assert!(
            roots.iter().any(Option::is_some),
            "search {search} never handed the donor a root score"
        );
        let first_some = roots.iter().position(Option::is_some).expect("one exists");
        assert!(
            roots[first_some..].iter().all(Option::is_some),
            "search {search} lost its root score"
        );
    }
}
