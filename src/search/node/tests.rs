use super::*;
use crate::infra;
use crate::score::{INF_SCORE, MAX_PLY, TB_WIN_SCORE};

#[test]
fn laterality_grows_by_the_log_of_the_move_index() {
    let steps = [0, 0, 0, 0, 1, 1, 1, 1, 2, 2];
    for (move_count, &expected) in steps.iter().enumerate() {
        assert_eq!(
            laterality_step(infra::to_i32(move_count)),
            expected,
            "move {move_count}"
        );
    }
    assert_eq!(laterality_step(16), 3);
    assert_eq!(laterality_step(255), 6);
}

/// The search leaves no reduction on the stack after it unwinds and a
/// root PV made of legal moves, from a quiet middlegame root and from a
/// root in check.
#[test]
fn search_keeps_reductions_unwinding_and_the_pv_legal() {
    let roots = [
        "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        "rnbqk1nr/pppp1ppp/8/4p3/1b1PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3",
    ];
    for fen in roots {
        let mut searcher = Searcher::default();
        let root = Board::from_fen(fen).expect("valid FEN");
        let mut board = root.clone();
        let score = searcher.search_root_window(&mut board, 7, -INF_SCORE, INF_SCORE, &mut || {
            SearchEvent::None
        });
        let context = format!("at {fen}");
        assert!(score.abs() < INF_SCORE, "{context}");
        assert_eq!(board.to_fen(), root.to_fen(), "{context}");
        for ply in 0..MAX_PLY {
            assert_eq!(searcher.td.stack[ply].reduction, 0, "{context}, ply {ply}");
        }
        let mut line = root;
        let pv_len = searcher.td.pv_len[0].min(MAX_PLY);
        assert!(pv_len > 0, "{context}: empty root PV");
        for &mv in &searcher.td.pv_table[0][..pv_len] {
            let legal = line
                .parse_move(&mv.to_string())
                .unwrap_or_else(|| panic!("{context}: illegal PV move {mv}"));
            line.make_move(legal);
        }
    }
}

/// After a search unwinds, no ply still carries an LMR reduction: the
/// field is non-zero only while the reduced child is being searched, and
/// a child reads its parent's entry.
///
/// The search is the one the engine plays, iterative deepening through
/// `Searcher::search`: a single full-window root search from a fresh
/// searcher is a path the engine never takes, and its size moved fourfold
/// between two fitted evaluations, enough to drop below the floor.
#[test]
fn stack_reductions_unwind_to_zero() {
    let mut searcher = Searcher::default();
    let board =
        Board::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4")
            .expect("valid FEN");
    let mut options = crate::search::options::SearchOptions {
        board: board.clone(),
        ..crate::search::options::SearchOptions::default()
    };
    options.limits.depth = Some(7);
    let result = searcher.search(board, &options, false, || SearchEvent::None);
    assert_eq!(result.depth, 7);
    assert!(result.score.abs() < INF_SCORE);
    assert!(
        searcher.td.nodes > 1_000,
        "the search must reach reductions"
    );
    for ply in 0..MAX_PLY {
        assert_eq!(searcher.td.stack[ply].reduction, 0, "ply {ply}");
    }
}

#[test]
fn a_tablebase_result_ends_the_node_only_where_it_decides_the_window() {
    let (alpha, beta) = (-50, 50);
    let win = TB_WIN_SCORE + 40;
    // A draw is exact: it always decides.
    for pv in [false, true] {
        assert_eq!(
            tb_verdict(0, Bound::Exact, alpha, beta, pv),
            TbVerdict::Return
        );
    }
    // A win at or above beta cuts; below beta it is a floor at a PV node.
    assert_eq!(
        tb_verdict(win, Bound::Lower, alpha, beta, false),
        TbVerdict::Return
    );
    assert_eq!(
        tb_verdict(win, Bound::Lower, -win - 1, win + 1, false),
        TbVerdict::Search
    );
    assert_eq!(
        tb_verdict(win, Bound::Lower, -win - 1, win + 1, true),
        TbVerdict::Floor
    );
    // A loss at or below alpha cuts; above alpha it is a cap at a PV node.
    assert_eq!(
        tb_verdict(-win, Bound::Upper, alpha, beta, true),
        TbVerdict::Return
    );
    assert_eq!(
        tb_verdict(-win, Bound::Upper, -win - 1, win, false),
        TbVerdict::Search
    );
    assert_eq!(
        tb_verdict(-win, Bound::Upper, -win - 1, win, true),
        TbVerdict::Cap
    );
}
