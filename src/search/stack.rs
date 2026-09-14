//! The per-ply search stack.

use std::ops::{Index, IndexMut};

use crate::board::{Move, Piece};
use crate::eval::VALUE_NONE;

use super::history::{PIECE_TO_SIZE, piece_to_index};
use super::{MAX_PLY, Searcher};

/// Sentinel plies below the root. Look-backs of up to this many plies read a
/// sentinel entry instead of needing a `ply >= n` guard.
const STACK_SENTINELS: usize = 8;

/// A per-ply array indexed from ply `-STACK_SENTINELS` (Reckless `PlyArray`).
///
/// `array[ply]` is the entry of a real ply (`ply >= 0`); [`Self::back`] reads
/// `back` plies before one, which below the root lands on a sentinel entry.
/// Sentinels are written only by construction, so they hold their default
/// value forever: a null move and `VALUE_NONE`, which is exactly what each
/// removed guard substituted.
#[derive(Clone)]
pub(super) struct PlyArray<T> {
    data: [T; MAX_PLY + STACK_SENTINELS],
}

impl<T: Copy> PlyArray<T> {
    pub(super) fn new(value: T) -> Self {
        Self {
            data: [value; MAX_PLY + STACK_SENTINELS],
        }
    }

    /// The entry `back` plies before `ply`, a sentinel when that is below the
    /// root.
    #[inline(always)]
    pub(super) fn back(&self, ply: usize, back: usize) -> &T {
        debug_assert!(
            back <= STACK_SENTINELS,
            "look-back {back} exceeds the sentinels"
        );
        &self.data[ply + STACK_SENTINELS - back]
    }
}

impl<T> Index<usize> for PlyArray<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, ply: usize) -> &T {
        &self.data[ply + STACK_SENTINELS]
    }
}

impl<T> IndexMut<usize> for PlyArray<T> {
    #[inline(always)]
    fn index_mut(&mut self, ply: usize) -> &mut T {
        &mut self.data[ply + STACK_SENTINELS]
    }
}

/// Per-ply search context: one record per ply.
///
/// The move and the piece that made it are read together by every
/// continuation-history lookup, and the static eval is written at the same
/// node and read at `ply - 2` by the improving test. The record is a
/// representation, not a speed choice: against parallel arrays it measured
/// **+0.11%, CI −0.14%..+0.48%**, a null inside the ±0.5% floor. A field joins
/// it only with a consumer.
#[derive(Copy, Clone)]
pub(super) struct StackEntry {
    /// The move made AT this ply. `Move::NULL` when the ply holds no move.
    pub(super) mv: Move,
    /// The piece that made `mv`. Only meaningful when `mv` is not null.
    pub(super) piece: Piece,
    /// Static eval of the position at this ply, or `VALUE_NONE` in check.
    pub(super) static_eval: i32,
    /// Continuation key: `piece_to_index(piece, mv.to)`, derived once when
    /// the move is pushed. Meaningless when `mv` is null; every consumer
    /// checks that first.
    ///
    /// Deriving the key at push time keeps `mv`, `piece` and the key from
    /// disagreeing: a push that wrote only `mv` would index continuation
    /// history with a piece left over from a sibling subtree.
    pub(super) cont_key: usize,
    /// Order index of `mv` among the moves its node's picker handed over,
    /// pruned moves included; zero for a ProbCut or null move.
    #[cfg(feature = "b2core")]
    pub(super) move_count: i32,
    /// Accumulated lateness of the line to this ply: the parent's laterality
    /// plus `max(ilog2(move_count) - 1, 0)`; zero after a null move.
    #[cfg(feature = "b2core")]
    pub(super) laterality: i32,
    /// The LMR reduction, in 1024ths of a ply, of the move being searched at
    /// this ply; zero outside that reduced search. The child reads it.
    #[cfg(feature = "b2core")]
    pub(super) reduction: i32,
    /// Beta cutoffs at this ply since the grandparent reset it on entry.
    #[cfg(feature = "b2core")]
    pub(super) cutoff_count: i32,
}

impl StackEntry {
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

impl Default for StackEntry {
    fn default() -> Self {
        Self {
            mv: Move::NULL,
            piece: Piece::Pawn,
            static_eval: VALUE_NONE,
            cont_key: 0,
            #[cfg(feature = "b2core")]
            move_count: 0,
            #[cfg(feature = "b2core")]
            laterality: 0,
            #[cfg(feature = "b2core")]
            reduction: 0,
            #[cfg(feature = "b2core")]
            cutoff_count: 0,
        }
    }
}

impl Searcher {
    /// Record the move being searched at `ply`, deriving its continuation key.
    ///
    /// The ONLY way to put a move on the stack. Writing `mv` and `piece`
    /// separately is what let ProbCut desynchronise them (see `StackEntry`).
    #[inline]
    pub(super) fn push_move(&mut self, ply: usize, mv: Move, piece: Piece) {
        self.td.stack[ply].mv = mv;
        self.td.stack[ply].piece = piece;
        self.td.stack[ply].cont_key = piece_to_index(piece as usize, mv.to_sq().index());
    }

    /// Clear the move at `ply`. The static eval is deliberately preserved: it
    /// belongs to the node, not to the move being tried at it.
    #[inline]
    pub(super) fn clear_move(&mut self, ply: usize) {
        self.td.stack[ply].mv = Move::NULL;
        self.td.stack[ply].cont_key = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A look-back below the root reads a sentinel holding the default entry,
    /// which is what every removed `ply >= n` guard substituted; above the
    /// root it reads the same slot `Index` does.
    #[test]
    fn look_back_below_the_root_reads_a_default_sentinel() {
        let mut stack = PlyArray::new(StackEntry::default());
        stack[0].mv = Move::from_uci("e2e4").expect("valid move");
        stack[0].static_eval = 17;
        stack[1].static_eval = 23;
        for ply in 0..2 {
            for back in ply + 1..=STACK_SENTINELS {
                let entry = stack.back(ply, back);
                assert!(entry.mv.is_null(), "ply {ply} back {back}");
                assert_eq!(entry.static_eval, VALUE_NONE, "ply {ply} back {back}");
            }
        }
        assert_eq!(stack.back(2, 2).static_eval, 17);
        assert_eq!(stack.back(2, 1).static_eval, 23);
        assert_eq!(stack.back(1, 1).mv, stack[0].mv);
    }
}
