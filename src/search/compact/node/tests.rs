use super::*;
use crate::search::options::SearchOptions;

/// One root search of `fen` from a fresh searcher.
fn root_search(fen: &str, depth: i32, alpha: i32, beta: i32) -> (Searcher, Board, i32) {
    let mut searcher = Searcher::default();
    let mut board = Board::from_fen(fen).expect("a legal FEN");
    let score =
        searcher.search_root_window(&mut board, depth, alpha, beta, &mut || SearchEvent::None);
    (searcher, board, score)
}

/// A root window far above the position's value fails low and one far
/// below it fails high, each returning a bound on its own side; a full
/// window stores the root's exact result under the move it reports.
#[test]
fn a_root_window_fails_on_the_side_of_the_value_and_the_root_stores_its_result() {
    const DEPTH: i32 = 7;
    const FAR: i32 = 400;
    let mut checked = 0;
    for fen in crate::bench::BENCH_FENS.iter().step_by(3) {
        let (searcher, board, exact) = root_search(fen, DEPTH, -INF_SCORE, INF_SCORE);
        assert!(exact.abs() < INF_SCORE, "{fen}");
        let entry = searcher
            .shared
            .tt
            .probe(board.hash())
            .expect("the root is stored");
        assert_eq!(entry.bound(), Some(Bound::Exact), "{fen}");
        assert_eq!(entry.best_move(), Some(searcher.td.pv_table[0][0]), "{fen}");
        assert_eq!(
            i32::from(entry.score),
            exact,
            "{fen}: the stored score is the returned one"
        );
        if is_decisive(exact) {
            continue;
        }
        let (_, _, low) = root_search(fen, DEPTH, exact + FAR, exact + FAR + 1);
        assert!(
            low <= exact + FAR,
            "{fen}: {low} above a window over {exact}"
        );
        let (_, _, high) = root_search(fen, DEPTH, exact - FAR - 1, exact - FAR);
        assert!(
            high >= exact - FAR,
            "{fen}: {high} below a window under {exact}"
        );
        checked += 1;
    }
    assert!(checked >= 10, "{checked} positions");
}

/// The grandchild cutoff reset stays inside the stack at the deepest node
/// that reaches it.
#[test]
fn cutoff_count_reset_is_bounded_at_the_ply_cap() {
    let mut searcher = Searcher::default();
    let mut board = Board::from_fen("4k3/8/8/8/3q4/8/8/4KQ2 w - - 0 1").expect("valid FEN");
    for ply in [MAX_PLY - 3, MAX_PLY - 2] {
        let score = searcher.compact_negamax::<NonPv, _>(
            &mut board,
            -1,
            0,
            2,
            false,
            ply,
            0,
            Move::NULL,
            &mut || SearchEvent::None,
        );
        assert!(score.abs() < INF_SCORE, "ply {ply}");
    }
}

/// An exclusion search whose only legal move is the excluded one fails
/// low, and with no mate, which its empty move list would otherwise
/// score: the forced evasion `Kxg2` is singular.
#[test]
fn an_exclusion_search_with_only_the_excluded_move_returns_alpha() {
    let mut searcher = Searcher::default();
    let mut board = Board::from_fen("7k/8/8/8/8/8/6q1/7K w - - 0 1").expect("valid FEN");
    let forced = board.parse_move("h1g2").expect("legal capture");
    assert_eq!(
        board.generate_legal_moves().len(),
        1,
        "the capture is forced"
    );
    let singular_beta = -100;
    let score = searcher.compact_negamax::<NonPv, _>(
        &mut board,
        singular_beta - 1,
        singular_beta,
        3,
        true,
        1,
        1,
        forced,
        &mut || SearchEvent::None,
    );
    assert!(
        score < singular_beta,
        "{score}: the excluded move is singular"
    );
    assert!(
        score > -MATE_SCORE + infra::to_i32(MAX_PLY),
        "{score}: no mate is scored"
    );
}

#[test]
fn a_line_keeps_eight_times_its_iteration_depth_before_quiescence() {
    let mut searcher = Searcher::default();
    searcher.td.root_depth = 10;
    assert_eq!(
        searcher.compact_line_room(5, 70),
        5,
        "room left: depth kept"
    );
    assert_eq!(searcher.compact_line_room(20, 75), 5, "held to the room");
    assert_eq!(
        searcher.compact_line_room(20, 85),
        -5,
        "past it: quiescence"
    );
    searcher.td.root_depth = 0;
    assert_eq!(
        searcher.compact_line_room(9, 4),
        4,
        "a first iteration counts one ply"
    );
}

fn compact_search(fen: &str, depth: u32) -> (Searcher, super::super::super::SearchResult) {
    let mut options = SearchOptions::default();
    options.limits.depth = Some(depth);
    let mut searcher = Searcher::default();
    let board = Board::from_fen(fen).expect("a legal FEN");
    let result = searcher.search(board, &options, false, || SearchEvent::None);
    (searcher, result)
}

#[test]
fn the_compact_kernel_finds_a_mate_and_counts_each_move_made_once() {
    let (searcher, result) = compact_search("6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1", 4);
    assert_eq!(result.bestmove.to_string(), "d1d8");
    assert!(result.score >= MATE_SCORE - 10, "a mate: {}", result.score);
    assert_eq!(searcher.td.nodes, searcher.td.moves_made);
}

#[test]
fn every_root_move_completes_each_iteration_when_the_window_fails_high() {
    // From the start the kernel's scores climb past a window opened round
    // the previous score of zero, so iterations fail high and search a
    // ply shallower; the root driver still completes each one.
    let (searcher, result) = compact_search(crate::board::STARTING_FEN, 3);
    assert_eq!(result.depth, 3);
    let records = &searcher.td.root_move_records;
    assert_eq!(records.len(), 20);
    assert!(
        records.iter().all(|record| record.samples == 3),
        "{:?}",
        records
            .iter()
            .map(|record| (record.mv.to_string(), record.samples))
            .collect::<Vec<_>>()
    );
}

#[test]
fn many_queens_stay_inside_the_kernels_range_and_score_in_their_band() {
    // (FEN, whether the side to move is the one with the queens).
    for (fen, ahead) in [
        ("k7/p7/8/8/8/8/8/QQQQQQ1K w - - 0 1", true),
        ("qqqqqq1k/8/8/8/8/8/P7/K7 w - - 0 1", false),
        ("3qqqqk/8/8/8/8/8/8/K1QQQQQ1 w - - 0 1", true),
        ("1k6/1p6/8/8/8/8/8/QQQQQQ1K b - - 0 1", false),
    ] {
        let board = Board::from_fen(fen).expect("a legal FEN");
        let mut searcher = Searcher::default();
        let raw = searcher.raw_eval(&board);
        let kernel = bounded_raw(raw);
        assert!(
            kernel.abs() < TB_WIN_SCORE,
            "{fen}: raw {raw}, kernel {kernel}"
        );
        assert_eq!(kernel > 0, ahead, "{fen}: raw {raw}");
        let (_, result) = compact_search(fen, 5);
        let score = result.score;
        assert_eq!(score > 0, ahead, "{fen}: {score}");
        assert!(
            score.abs() < TB_WIN_SCORE || score.abs() >= MATE_SCORE - infra::to_i32(MAX_PLY),
            "{fen}: {score} is neither an evaluation nor a mate"
        );
    }
}

/// Each recursive call types its child as the donor does: the null move,
/// its verification, ProbCut's searches and the PV search all nodes, a
/// reduced late move a cut node, its re-search and a full-depth zero
/// window the opposite of their parent, and the exclusion search its
/// parent's own type. Every call must be reached, so none passes unseen.
#[test]
fn each_recursive_call_types_its_child_by_its_rule() {
    fn rule(call: ChildCall, parent_cut: bool) -> bool {
        match call {
            ChildCall::Null | ChildCall::NullVerify | ChildCall::ProbCut | ChildCall::Pv => false,
            ChildCall::Reduced => true,
            ChildCall::Singular => parent_cut,
            ChildCall::Research | ChildCall::FullDepth => !parent_cut,
        }
    }
    let mut counts = [[[0u64; 2]; 2]; ChildCall::COUNT];
    // A bench position whose null move is verified by depth 12, beside
    // every fourth bench position at depth 10.
    let verified = ("1Q4R1/5k2/4rpp1/3K4/8/7p/8/8 b - - 2 9", 12);
    let searches = crate::bench::BENCH_FENS
        .iter()
        .step_by(4)
        .map(|&fen| (fen, 10))
        .chain([verified]);
    for (fen, depth) in searches {
        let (searcher, _) = compact_search(fen, depth);
        for (total, seen) in counts.iter_mut().zip(searcher.td.child_types) {
            for (total, seen) in total.iter_mut().zip(seen) {
                for (total, seen) in total.iter_mut().zip(seen) {
                    *total += seen;
                }
            }
        }
    }
    for call in ChildCall::ALL {
        let by_parent = counts[call as usize];
        let seen: u64 = by_parent.iter().flatten().sum();
        assert!(seen > 0, "{call:?} was never searched");
        for parent_cut in [false, true] {
            let wrong = by_parent[usize::from(parent_cut)][usize::from(!rule(call, parent_cut))];
            assert_eq!(
                wrong, 0,
                "{call:?} under a parent typed {parent_cut}: {wrong} children against the rule"
            );
        }
    }
}

#[test]
fn the_compact_kernel_searches_the_bench_positions_to_a_legal_move() {
    for fen in crate::bench::BENCH_FENS.iter().step_by(5) {
        let (searcher, result) = compact_search(fen, 6);
        let board = Board::from_fen(fen).expect("a legal FEN");
        assert!(
            board.generate_legal_moves().contains(&result.bestmove),
            "{fen}: {} is not legal",
            result.bestmove
        );
        assert_eq!(result.depth, 6, "{fen}");
        assert_eq!(searcher.td.nodes, searcher.td.moves_made, "{fen}");
    }
}
