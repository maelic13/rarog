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

/// 4.5.1 PER-PLY SEARCH CONTEXT.
///
/// Replaces three parallel `[_; MAX_PLY]` arrays with one record per ply.
///
/// The move and the piece that made it were never independent: every
/// continuation-history lookup read both at the same ply, so the split cost
/// two cache lines to answer one question. The static eval joins them because
/// it is written at the same node and read at `ply - 2` by the improving test.
///
/// That locality argument did NOT pay, and the record should say so: a pooled
/// three-build-per-arm PGO A/B measured **+0.11%, CI −0.14%..+0.48%** — a null
/// inside this machine's ±0.5% floor (RAR-P17). The justification for this
/// change is that it is the substrate 4.5.2–4.5.4 consume, not that it is
/// faster. It is not faster.
///
/// This is a REPRESENTATION change and nothing else. PLAN 4.5.1 also lists
/// TT/PV evidence, previous reduction, statistical score, cutoff count,
/// previous-PV following and continuation keys — none are added here, because
/// nothing consumes them yet and rule 2 forbids landing speculative state.
/// They arrive with 4.5.2–4.5.4, which is where their consumers are.
#[derive(Copy, Clone)]
pub(super) struct StackEntry {
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
