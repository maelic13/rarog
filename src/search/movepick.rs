//! The root's move picker, the scored list every picker fills, and the
//! per-node context of quiet ordering.

use crate::board::attacks::{knight_attacks_of, pawn_attacks_of};
use crate::board::{ATTACKS, Bitboard, Board, CheckInfo, Color, Move, Piece, Threats};

use super::Searcher;
use super::history::{CONT_PLY_BACK, HistoryTables};

/// Squares on which a pawn lever of the side to move stands advanced:
/// ranks 5 and 6 for White, 3 and 4 for Black.
const LEVER_RANKS: [Bitboard; 2] = [
    Bitboard(0x0000_FFFF_0000_0000),
    Bitboard(0x0000_0000_FFFF_0000),
];
const HOME_RANKS: [Bitboard; 2] = [Bitboard(0xFF), Bitboard(0xFF00_0000_0000_0000)];

/// The table move's place in the root's list, above every other tier.
pub(super) const ROOT_TT_SCORE: i32 = 30_000_000;

#[derive(Copy, Clone, Default)]
pub(crate) struct ScoredMove {
    pub mv: Move,
    pub score: i32,
}

/// A node's moves with their ordering scores, at most 256.
pub(super) type ScoredMoveList = crate::board::FixedList<ScoredMove, 256>;

impl ScoredMoveList {
    #[inline(always)]
    pub(super) fn push_scored(&mut self, mv: Move, score: i32) {
        self.push(ScoredMove { mv, score });
    }
}

/// Selection step: move the highest-scored entry of `moves[index..]` into
/// `moves[index]` and return it. Ties resolve to the earliest entry.
pub(super) fn pick_next(moves: &mut [ScoredMove], index: usize) -> ScoredMove {
    let tail = &mut moves[index..];
    let mut best = 0;
    let mut best_score = tail[0].score;
    for (offset, candidate) in tail.iter().enumerate().skip(1) {
        // A new maximum turns up at no predictable point in the scan, so a
        // branch here mispredicts at every one; a conditional move does not.
        let better = candidate.score > best_score;
        best = std::hint::select_unpredictable(better, offset, best);
        best_score = std::hint::select_unpredictable(better, candidate.score, best_score);
    }
    tail.swap(0, best);
    tail[0]
}

pub(super) fn diversify_root_scores(moves: &mut [ScoredMove], offset: usize) {
    moves.sort_unstable_by_key(|m| std::cmp::Reverse(m.score));
    if offset < moves.len() {
        moves[offset].score = moves[0].score.saturating_add(1_000_000);
    }
}

/// The root picker's stages, visited in declaration order and never
/// revisited.
///
/// Contract, covered by the tests below: the TT move first, at most once
/// (the caller validates it), then every other generated legal move once,
/// best first.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum Stage {
    TtMove,
    /// Everything except the TT move, best first.
    AllRemaining,
    Done,
}

/// The root's picker: its whole scored list.
pub(super) struct MovePicker {
    scored: ScoredMoveList,
    index: usize,
    tt_move: Move,
    stage: Stage,
}

impl MovePicker {
    /// An empty picker that hands over `tt_move` first; its list is filled
    /// in place, as the list is too large to move by value once a window.
    #[inline(always)]
    pub(super) fn new(tt_move: Move) -> Self {
        Self {
            scored: ScoredMoveList::new(),
            index: 0,
            tt_move,
            stage: Stage::TtMove,
        }
    }

    /// The list the picker hands over, best first, to fill before the first
    /// move is taken.
    pub(super) fn scored_mut(&mut self) -> &mut ScoredMoveList {
        &mut self.scored
    }

    /// Yield the next move.
    pub(super) fn next(&mut self) -> Option<ScoredMove> {
        loop {
            match self.stage {
                Stage::TtMove => {
                    self.stage = Stage::AllRemaining;
                    if !self.tt_move.is_null() {
                        return Some(tt_scored_move(self.tt_move));
                    }
                }
                Stage::AllRemaining => {
                    while self.index < self.scored.len() {
                        let picked = pick_next(self.scored.as_mut_slice(), self.index);
                        self.index += 1;
                        if picked.mv != self.tt_move {
                            return Some(picked);
                        }
                    }
                    self.stage = Stage::Done;
                }
                Stage::Done => return None,
            }
        }
    }
}

fn tt_scored_move(mv: Move) -> ScoredMove {
    ScoredMove {
        mv,
        score: ROOT_TT_SCORE,
    }
}

/// Attacks of every square in `sources` by a piece moving like `attacks`.
fn attacks_from(sources: Bitboard, attacks: impl Fn(crate::board::Square) -> Bitboard) -> Bitboard {
    sources.fold(Bitboard::EMPTY, |all, sq| all | attacks(sq))
}

/// Node-invariant inputs of quiet scoring, resolved once per node.
pub(super) struct QuietContext {
    pub(super) stm: Color,
    /// Squares attacked by enemy pieces cheaper than each moving piece type.
    pub(super) threatened: [Bitboard; 6],
    /// Safe squares from which each piece type would attack an enemy piece.
    pub(super) offense: [Bitboard; 6],
    pub(super) check_squares: [Bitboard; 6],
    pub(super) wall_pawns: Bitboard,
    pub(super) pawn_row: usize,
    pub(super) cont: [Option<usize>; 4],
}

impl Searcher {
    pub(super) fn quiet_context(
        &self,
        board: &Board,
        threats: &Threats,
        ply: usize,
        node_ci: &mut Option<CheckInfo>,
    ) -> QuietContext {
        let stm = board.side_to_move();
        let them = !stm;
        let by = &threats.by_piece;
        let all = threats.all;
        let occ = board.occupied();
        let minor =
            by[Piece::Pawn as usize] | by[Piece::Knight as usize] | by[Piece::Bishop as usize];
        let rook = minor | by[Piece::Rook as usize];
        let threatened = [
            Bitboard::EMPTY,
            by[Piece::Pawn as usize],
            by[Piece::Pawn as usize],
            minor,
            rook,
            Bitboard::EMPTY,
        ];
        let non_pawn = by[Piece::Knight as usize]
            | by[Piece::Bishop as usize]
            | by[Piece::Rook as usize]
            | by[Piece::Queen as usize]
            | by[Piece::King as usize];
        let atk = &ATTACKS;
        let their = |piece| board.pieces(them, piece);
        let knight_targets =
            (their(Piece::Bishop) & !all) | their(Piece::Rook) | their(Piece::Queen);
        // A pawn of ours attacks their piece from the squares one of their
        // pawns there would attack; the same holds for knights.
        let pawn_offense = (pawn_attacks_of(them, board.color_occ(them)) & !all)
            | (by[Piece::Pawn as usize] & LEVER_RANKS[stm as usize] & !non_pawn);
        let offense = [
            pawn_offense,
            knight_attacks_of(knight_targets) & !all,
            attacks_from(their(Piece::Rook), |sq| atk.bishop(sq, occ)) & !all,
            (Bitboard::FILE_A << u32::from(board.king_sq(them).0 % 8)) & !all,
            (attacks_from(their(Piece::Bishop) & !all, |sq| atk.rook(sq, occ))
                | attacks_from(their(Piece::Rook) & !all, |sq| atk.bishop(sq, occ)))
                & !all,
            Bitboard::EMPTY,
        ];
        let king = board.king_sq(stm);
        let wall_pawns = if HOME_RANKS[stm as usize].contains(king) {
            atk.king(king) & (board.pieces(stm, Piece::Pawn) | board.pieces(them, Piece::Pawn))
        } else {
            Bitboard::EMPTY
        };
        // The node's check masks, shared with the move loop's check tests:
        // the first of the two to need them computes them.
        let check_info = node_ci.get_or_insert_with(|| board.check_info());
        #[cfg(debug_assertions)]
        {
            let fresh = board.check_info();
            debug_assert!(
                Piece::ALL.iter().all(|&piece| {
                    check_info.direct_check_squares(piece) == fresh.direct_check_squares(piece)
                }),
                "the shared check masks are not this position's"
            );
        }
        let mut check_squares = [Bitboard::EMPTY; 6];
        for piece in Piece::ALL {
            check_squares[piece as usize] = check_info.direct_check_squares(piece);
        }
        let mut cont = [None; 4];
        for (slot, back) in CONT_PLY_BACK.into_iter().enumerate() {
            cont[slot] = self.cont_context_back(ply, back);
        }
        QuietContext {
            stm,
            threatened,
            offense,
            check_squares,
            wall_pawns,
            pawn_row: HistoryTables::pawn_row(board.pawn_key()),
            cont,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `as_mut_slice` exposes `..len` as initialized, so `len` must never pass
    /// the last slot a push wrote: the push after a full list panics
    /// instead (the debug assertion or, in release, the checked index).
    #[test]
    #[should_panic]
    fn a_push_past_the_last_slot_panics() {
        let mut list = ScoredMoveList::new();
        let mv = Move::from_uci("e2e4").expect("valid UCI move shape");
        for _ in 0..256 {
            list.push_scored(mv, 0);
        }
        assert_eq!(list.as_mut_slice().len(), 256);
        list.push_scored(mv, 0);
    }
}
