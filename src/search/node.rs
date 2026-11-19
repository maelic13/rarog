//! What every node shares: its type, the tablebase verdict, the root's entry
//! to the kernel, the move order record and the table's access.

use crate::board::Board;
use crate::tt::{Bound, TtEntry, TtStore};

use super::{SearchEvent, Searcher};

/// What an in-search tablebase result does at a node.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum TbVerdict {
    /// It decides the window: store it with its bound and return it.
    Return,
    /// A win below beta at a PV node: the moves must beat it.
    Floor,
    /// A loss above alpha at a PV node: it caps what the moves may claim.
    Cap,
    /// A bound that decides nothing at a node off the PV.
    Search,
}

/// A tablebase result ends the node only where it decides the window: a draw
/// always, a win (a lower bound) at or above beta, a loss (an upper bound) at
/// or below alpha. Otherwise only a PV node keeps it, as a floor or a cap.
pub(super) fn tb_verdict(score: i32, bound: Bound, alpha: i32, beta: i32, pv: bool) -> TbVerdict {
    let decides = match bound {
        Bound::Exact => true,
        Bound::Lower => score >= beta,
        Bound::Upper => score <= alpha,
    };
    if decides {
        TbVerdict::Return
    } else if !pv {
        TbVerdict::Search
    } else if bound == Bound::Lower {
        TbVerdict::Floor
    } else {
        TbVerdict::Cap
    }
}

/// A node's type, resolved at compile time, so quiescence can know it is on
/// a PV line without a new parameter. `cut_node` stays a runtime argument,
/// so an all-node is `!PV && !cut_node`.
pub(super) trait NodeType {
    /// On the principal variation: searched with an open window.
    const PV: bool;
    /// The root, which is ply 0 and nothing else.
    const ROOT: bool;
}

/// The root node: a PV node at ply 0.
pub(super) struct Root;
/// A PV node below the root.
pub(super) struct Pv;
/// A null-window node.
pub(super) struct NonPv;

impl NodeType for Root {
    const PV: bool = true;
    const ROOT: bool = true;
}

impl NodeType for Pv {
    const PV: bool = true;
    const ROOT: bool = false;
}

impl NodeType for NonPv {
    const PV: bool = false;
    const ROOT: bool = false;
}

/// How much later than the first few moves the move at `move_count` is:
/// `max(ilog2(move_count) - 1, 0)`, zero for the first three moves.
fn laterality_step(move_count: i32) -> i32 {
    if move_count > 0 {
        (move_count.ilog2().cast_signed() - 1).max(0)
    } else {
        0
    }
}

impl Searcher {
    /// Search the root position at `depth` inside the window: the entry the
    /// iterative-deepening loop calls, so it never depends on the kernel's
    /// argument list.
    pub(super) fn search_root_window<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        depth: i32,
        alpha: i32,
        beta: i32,
        poll: &mut P,
    ) -> i32 {
        self.td.root_delta = beta - alpha;
        {
            self.td.root_depth = depth;
        }
        self.compact_root(board, depth, alpha, beta, poll)
    }

    /// Record the order index of the move about to be searched at `ply` and
    /// the line's accumulated lateness: the parent's laterality plus
    /// [`laterality_step`] of this move's index.
    #[inline]
    pub(super) fn record_move_order(&mut self, ply: usize, move_count: i32) {
        let parent = self.td.stack.back(ply, 1).laterality;
        let entry = &mut self.td.stack[ply];
        entry.move_count = move_count;
        entry.laterality = parent + laterality_step(move_count);
    }

    /// The table entry for `key`. A pricing probe also reads its scratch
    /// table.
    #[inline(always)]
    pub(super) fn tt_probe(&self, key: u64) -> Option<TtEntry> {
        #[cfg(feature = "pricing")]
        if self.td.pricing.probing {
            return self.pricing_tt_probe(key);
        }
        self.shared.tt.probe(key)
    }

    /// Store a search result. A pricing probe stores into its scratch table,
    /// so the main table stays as the measured search left it; an identity
    /// probe stores nothing.
    #[inline(always)]
    pub(super) fn tt_store(&mut self, store: TtStore) {
        #[cfg(feature = "pricing")]
        if self.td.pricing.probing {
            if let Some(table) = self.pricing_scratch() {
                table.store(store);
            }
            return;
        }
        self.shared.tt.store(store);
    }

    /// Store a search result over any entry the replacement rule would keep,
    /// with the same redirection as `tt_store`.
    #[inline(always)]
    pub(super) fn tt_store_forced(&mut self, store: TtStore) {
        #[cfg(feature = "pricing")]
        if self.td.pricing.probing {
            if let Some(table) = self.pricing_scratch() {
                table.store_forced(store);
            }
            return;
        }
        self.shared.tt.store_forced(store);
    }

    /// Store a static evaluation alone; a pricing probe's goes to its
    /// scratch table.
    #[inline(always)]
    pub(super) fn tt_store_eval(&mut self, key: u64, raw_eval: i32, is_pv: bool) {
        #[cfg(feature = "pricing")]
        if self.td.pricing.probing {
            if let Some(table) = self.pricing_scratch() {
                table.store_eval(key, raw_eval, is_pv);
            }
            return;
        }
        self.shared.tt.store_eval(key, raw_eval, is_pv);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::INF_SCORE;
    use crate::infra;
    use crate::search::MAX_PLY;
    use crate::tt::TB_WIN_SCORE;

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
            let score =
                searcher.search_root_window(&mut board, 7, -INF_SCORE, INF_SCORE, &mut || {
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
}
