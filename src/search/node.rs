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
/// a PV line without an extra parameter. `cut_node` stays a runtime argument,
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
        let score = self.compact_root(board, depth, alpha, beta, poll);
        // A foreign evaluation's lazy cut reads the root's last score, as
        // Stockfish's reads its thread's best value after each window.
        #[cfg(feature = "donor-eval")]
        self.td.evaluator.set_donor_root(Some(score));
        score
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
mod tests;
