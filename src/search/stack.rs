//! The per-ply search stack.

use crate::board::{Move, Piece};
use crate::eval::VALUE_NONE;

use super::Searcher;
use super::history::{PIECE_TO_SIZE, piece_to_index};

#[derive(Copy, Clone)]
pub(super) struct NodeContext {
    /// The move made AT this ply. `Move::NULL` when the ply holds no move.
    pub(super) mv: Move,
    /// The piece that made `mv`. Only meaningful when `mv` is not null.
    pub(super) piece: Piece,
    /// Static eval of the position at this ply, or `VALUE_NONE` in check.
    pub(super) static_eval: i32,
    /// 4.5.3 CONTINUATION KEY: `piece_to_index(piece, mv.to)`, derived once
    /// when the move is pushed. Meaningless when `mv` is null; every consumer
    /// checks that first.
    ///
    /// This exists to make a class of bug unrepresentable, not to save the
    /// multiply. `mv` and `piece` used to be written by hand at four sites and
    /// ProbCut wrote only `mv`, so continuation history inside a ProbCut child
    /// search was indexed by the ProbCut move's destination paired with a piece
    /// left over from a sibling subtree. Deriving the key at push time means
    /// the three can no longer disagree.
    pub(super) cont_key: usize,
}

impl NodeContext {
    /// Row base for continuation tables at this ply.
    ///
    /// `cont_row_base(piece, to) == piece_to_index(piece, to) * PIECE_TO_SIZE`,
    /// so the stored key serves every continuation site and none of them needs
    /// to re-derive the pair from `mv`/`piece`.
    #[inline]
    pub(super) fn cont_row_base(&self) -> usize {
        self.cont_key * PIECE_TO_SIZE
    }
}

impl Default for NodeContext {
    fn default() -> Self {
        Self {
            mv: Move::NULL,
            piece: Piece::Pawn,
            static_eval: VALUE_NONE,
            cont_key: 0,
        }
    }
}

impl Searcher {
    /// Record the move being searched at `ply`, deriving its continuation key.
    ///
    /// The ONLY way to put a move on the stack. Writing `mv` and `piece`
    /// separately is what let ProbCut desynchronise them (see `NodeContext`).
    #[inline]
    pub(super) fn push_move(&mut self, ply: usize, mv: Move, piece: Piece) {
        self.stack[ply].mv = mv;
        self.stack[ply].piece = piece;
        self.stack[ply].cont_key = piece_to_index(piece as usize, mv.to_sq().index());
    }

    /// Clear the move at `ply`. The static eval is deliberately preserved: it
    /// belongs to the node, not to the move being tried at it.
    #[inline]
    pub(super) fn clear_move(&mut self, ply: usize) {
        self.stack[ply].mv = Move::NULL;
        self.stack[ply].cont_key = 0;
    }
}
