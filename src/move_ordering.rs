use std::{mem::MaybeUninit, slice};

use crate::board::{Move, Piece};
use crate::infra;

pub const HISTORY_MAX: i32 = 16_384;
pub const CAP_HISTORY_MAX: i32 = 16_384;
pub const CORR_SIZE: usize = 65_536;
pub const CONT_SIZE: usize = 6 * 64 * 6 * 64;
pub const LOW_PLY_HISTORY_SIZE: usize = 8;
pub const PAWN_HISTORY_SIZE: usize = 4_096;
pub const PIECE_TO_SIZE: usize = 6 * 64;

#[derive(Copy, Clone, Default)]
pub(crate) struct ScoredMove {
    pub mv: Move,
    pub score: i32,
    pub see: i16,
    pub quiet_history: i32,
}

// 9.0 KEEP-UNSAFE (measured): see MoveList in board/moves.rs — plain
// initialized arrays cost −10% NPS (2026-07-19). Unsafe confined to the
// slice accessors with a local prefix-initialization invariant.
pub(crate) struct ScoredMoveList {
    moves: [MaybeUninit<ScoredMove>; 256],
    len: usize,
}

impl ScoredMoveList {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            moves: [const { MaybeUninit::uninit() }; 256],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, mv: Move, score: i32, see: i32) {
        self.push_with_history(mv, score, see, 0);
    }

    #[inline(always)]
    pub fn push_with_history(&mut self, mv: Move, score: i32, see: i32, quiet_history: i32) {
        debug_assert!(self.len < self.moves.len());
        self.moves[self.len].write(ScoredMove {
            mv,
            score,
            see: crate::infra::saturating_i16(see),
            quiet_history,
        });
        self.len += 1;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [ScoredMove] {
        // SAFETY: only the initialized prefix below `len` is exposed.
        unsafe { slice::from_raw_parts_mut(self.moves.as_mut_ptr().cast::<ScoredMove>(), self.len) }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BadCapture {
    pub attacker: Piece,
    pub to: u8,
    pub captured: Option<Piece>,
}

pub(crate) struct BadCaptureList {
    items: [MaybeUninit<BadCapture>; 256],
    len: usize,
}

impl BadCaptureList {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            items: [const { MaybeUninit::uninit() }; 256],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, attacker: Piece, to: u8, captured: Option<Piece>) {
        debug_assert!(self.len < self.items.len());
        self.items[self.len].write(BadCapture {
            attacker,
            to,
            captured,
        });
        self.len += 1;
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[BadCapture] {
        // SAFETY: only the initialized prefix below `len` is exposed.
        unsafe { slice::from_raw_parts(self.items.as_ptr().cast::<BadCapture>(), self.len) }
    }
}

/// Selection step: move the highest-scored entry of `moves[index..]` into
/// `moves[index]` and return it.
///
/// 10.3(8d): scans a `split_at_mut` tail by iterator and carries the running
/// best SCORE in a local. The old form indexed `moves[current]` and
/// `moves[best]` per iteration — two loads where one suffices, plus an index
/// LLVM cannot bound-check away (`best` is only provably `< len` by induction
/// through the loop). Ties still resolve to the earliest entry: the comparison
/// stays strictly `>`.
pub(crate) fn pick_next(moves: &mut [ScoredMove], index: usize) -> ScoredMove {
    let tail = &mut moves[index..];
    let mut best = 0;
    let mut best_score = tail[0].score;
    for (offset, candidate) in tail.iter().enumerate().skip(1) {
        if candidate.score > best_score {
            best = offset;
            best_score = candidate.score;
        }
    }
    tail.swap(0, best);
    tail[0]
}

pub(crate) fn diversify_root_scores(moves: &mut [ScoredMove], offset: usize) {
    moves.sort_unstable_by_key(|m| std::cmp::Reverse(m.score));
    if offset < moves.len() {
        moves[offset].score = moves[0].score.saturating_add(1_000_000);
    }
}

pub(crate) fn update_hist_entry(entry: &mut i16, bonus: i32, max_value: i32) {
    let current = *entry as i32;
    let updated = current + bonus - current * bonus.abs() / max_value;
    *entry = crate::infra::saturating_i16(updated);
}

/// Flat index into a continuation-history table.
///
/// 9.0a: the inputs are structurally bounded — `piece`/`prev_piece` come from
/// a 6-variant `Piece` cast and the squares from `Square::index()` (masked to
/// 0..=63), so the result is always < `CONT_SIZE`. The `.min()` remains as the
/// release-mode backstop (it keeps the function total), but it used to be the
/// ONLY thing here: an out-of-range index was silently folded into the last
/// bucket, so a logic bug would quietly corrupt one history cell instead of
/// surfacing. The `debug_assert!`s now state the invariant and fail loudly in
/// debug and under `cargo test`.
///
/// NB `CONT_SIZE` (147,456) and `PIECE_TO_SIZE` (384) are NOT powers of two,
/// so `& (SIZE - 1)` is *not* a valid substitute for `.min()` here — masking
/// would remap valid in-range indices (65,536 would fold to 0). Only
/// `pawn_history_index`'s 4,096-entry slot table may use a mask.
/// Retained as the SPECIFICATION of the continuation index, even though
/// the search now reads a per-ply `cont_key` instead of calling this. The
/// test below proves `cont_index == cont_row_base + piece_to_index`, which
/// is the identity `NodeContext::cont_row_base` depends on — delete these
/// and that identity becomes an unchecked assumption.
#[cfg_attr(not(test), expect(dead_code))]
pub(crate) fn cont_index(prev_piece: usize, prev_to: usize, piece: usize, to: usize) -> usize {
    debug_assert!(prev_piece < 6 && piece < 6, "piece index out of range");
    debug_assert!(prev_to < 64 && to < 64, "square index out of range");
    (((prev_piece * 64 + prev_to) * 6 + piece) * 64 + to).min(CONT_SIZE - 1)
}

/// Node-invariant prefix of [`cont_index`]: the row base for a
/// `(prev_piece, prev_to)` pair, such that
/// `cont_index(pp, pt, piece, to) == (cont_row_base(pp, pt) + piece_to_index(piece, to)).min(CONT_SIZE - 1)`
/// (equivalence pinned by a test below). Move scoring resolves this once per
/// node instead of once per quiet move — the 8.12(g2) hoist from the Basilisk
/// cross-review (its 8.7.6(b+d), +3.03% NPS there).
/// Retained as the SPECIFICATION of the continuation index, even though
/// the search now reads a per-ply `cont_key` instead of calling this. The
/// test below proves `cont_index == cont_row_base + piece_to_index`, which
/// is the identity `NodeContext::cont_row_base` depends on — delete these
/// and that identity becomes an unchecked assumption.
#[cfg_attr(not(test), expect(dead_code))]
pub(crate) fn cont_row_base(prev_piece: usize, prev_to: usize) -> usize {
    debug_assert!(prev_piece < 6, "piece index out of range");
    debug_assert!(prev_to < 64, "square index out of range");
    (prev_piece * 64 + prev_to) * PIECE_TO_SIZE
}

/// Node-invariant prefix of [`pawn_history_index`]: the pawn-key row base,
/// same per-node hoist as [`cont_row_base`].
pub(crate) fn pawn_row_base(pawn_key: u64) -> usize {
    (infra::index(pawn_key) & (PAWN_HISTORY_SIZE - 1)) * PIECE_TO_SIZE
}

/// Flat `(piece, square)` index. Same reasoning as [`cont_index`].
pub(crate) fn piece_to_index(piece: usize, to: usize) -> usize {
    debug_assert!(piece < 6, "piece index out of range");
    debug_assert!(to < 64, "square index out of range");
    (piece * 64 + to).min(PIECE_TO_SIZE - 1)
}

pub(crate) fn pawn_history_index(pawn_key: u64, piece: usize, to: usize) -> usize {
    let slot = infra::index(pawn_key) & (PAWN_HISTORY_SIZE - 1);
    slot * PIECE_TO_SIZE + piece_to_index(piece, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hoisted row-base + per-move offset decomposition must agree with
    /// the original single-shot index everywhere — this is what makes the
    /// 8.12(g2) scoring hoist a pure refactor.
    #[test]
    fn row_base_decomposition_matches_cont_and_pawn_indexes() {
        for prev_piece in 0..6 {
            for prev_to in (0..64).step_by(7) {
                for piece in 0..6 {
                    for to in (0..64).step_by(5) {
                        assert_eq!(
                            (cont_row_base(prev_piece, prev_to) + piece_to_index(piece, to))
                                .min(CONT_SIZE - 1),
                            cont_index(prev_piece, prev_to, piece, to),
                        );
                    }
                }
            }
        }
        for key in [0u64, 1, 0xFFFF, 0xDEAD_BEEF_CAFE_F00D, u64::MAX] {
            for piece in 0..6 {
                for to in (0..64).step_by(9) {
                    assert_eq!(
                        pawn_row_base(key) + piece_to_index(piece, to),
                        pawn_history_index(key, piece, to),
                    );
                }
            }
        }
    }

    #[test]
    fn bad_capture_struct_stays_shrunk() {
        // Phase 2.9.3: `to: usize` (8 bytes) padded BadCapture to 16 bytes;
        // `to: u8` (a 0-63 square fits easily) drops it to ~3-4 bytes. Guard
        // against this creeping back up, since each BadCaptureList is [_; 256]
        // and two are allocated per negamax frame.
        assert!(
            std::mem::size_of::<BadCapture>() <= 4,
            "size_of::<BadCapture>() = {}",
            std::mem::size_of::<BadCapture>()
        );
    }
}
