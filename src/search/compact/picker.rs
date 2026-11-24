//! The compact kernel's staged move picker: the TT move, then the noisy
//! moves (captures and queen pushes) whose exchange clears a bar, then the
//! quiets, then the noisy moves that failed the bar, each stage generated
//! only when it is reached. Skipping quiets skips them all.

use crate::board::{Bitboard, Board, Move, MoveList, Piece, Threats};

use super::super::Searcher;
use super::super::history::HistoryTables;
use super::super::movepick::{QuietContext, ROOT_TT_SCORE, ScoredMoveList, pick_next};
use super::is_quiet_move;
use super::rules::{
    NoisyOrderInput, QuietOrderInput, Seeds, good_noisy_threshold, noisy_order, quiet_order,
};

/// The root's tiers below the table move and the pool's searched moves
/// (`apply_shared_root_scores`, from 25,000,000): the good noisy moves,
/// the quiets round zero, the bad noisy moves. Every ordering score lies
/// within `ROOT_ORDER_SPAN` of zero, so no tier reaches the next.
const ROOT_GOOD_NOISY: i32 = 20_000_000;
const ROOT_BAD_NOISY: i32 = -20_000_000;
const ROOT_ORDER_SPAN: i32 = 4_000_000;

/// The picker's stages, visited in this order and never revisited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(in crate::search) enum Stage {
    TtMove,
    GenerateNoisy,
    /// Noisy moves whose exchange clears their bar, best first.
    GoodNoisy,
    GenerateQuiets,
    Quiets,
    /// Noisy moves that failed their bar, in the order they failed it.
    BadNoisy,
    Done,
}

/// One node's picker. Noisy moves sit at the front of one list and quiets
/// behind them; a noisy move that fails its bar is swapped down below the
/// noisy moves not yet looked at, so the failures end up in order at the
/// list's front.
pub(in crate::search) struct CompactPicker {
    moves: ScoredMoveList,
    noisy_len: usize,
    noisy_index: usize,
    bad_len: usize,
    bad_index: usize,
    quiet_index: usize,
    good_emitted: i32,
    pinned: Bitboard,
    tt_move: Move,
    /// A fixed exchange bar for every noisy move, in place of the scored one.
    threshold: Option<i32>,
    stage: Stage,
    ply: usize,
}

impl CompactPicker {
    /// A picker that hands over `tt_move` first, if any, and holds noisy
    /// moves to `threshold` if one is given.
    #[inline(always)]
    pub(in crate::search) fn new(tt_move: Move, threshold: Option<i32>, ply: usize) -> Self {
        Self {
            moves: ScoredMoveList::new(),
            noisy_len: 0,
            noisy_index: 0,
            bad_len: 0,
            bad_index: 0,
            quiet_index: 0,
            good_emitted: 0,
            pinned: Bitboard::EMPTY,
            tt_move,
            threshold,
            stage: Stage::TtMove,
            ply,
        }
    }

    /// The stage of the move last handed over.
    pub(in crate::search) fn stage(&self) -> Stage {
        self.stage
    }

    /// The moves a stage-ending decision skips with the one it prices: the
    /// quiets not yet handed over, or the bad noisy moves.
    #[cfg(feature = "pricing")]
    pub(super) fn moves_left_behind(&self) -> usize {
        match self.stage {
            Stage::Quiets => (self.moves.len() - self.noisy_len).saturating_sub(self.quiet_index),
            Stage::BadNoisy => self.bad_len - self.bad_index,
            _ => 0,
        }
    }

    /// Whether the table's move is quiet by the kernel's partition, in
    /// which an under-promotion push is quiet.
    fn tt_move_quiet(&self) -> bool {
        is_quiet_move(self.tt_move)
    }

    /// The next move, or `None` when every stage is spent. Once
    /// `skip_quiets` is set, no quiet is handed over again.
    pub(in crate::search) fn next(
        &mut self,
        searcher: &Searcher,
        board: &mut Board,
        threats: &Threats,
        skip_quiets: bool,
    ) -> Option<Move> {
        loop {
            match self.stage {
                Stage::TtMove => {
                    self.stage = Stage::GenerateNoisy;
                    if !self.tt_move.is_null() {
                        return Some(self.tt_move);
                    }
                }
                Stage::GenerateNoisy => {
                    let mut noisy = MoveList::new();
                    self.pinned = board.generate_legal_noisy_staged_into(threats, &mut noisy);
                    for &mv in noisy.as_slice() {
                        if mv != self.tt_move {
                            let score = searcher.compact_noisy_score(board, threats, mv);
                            self.moves.push_scored(mv, score);
                        }
                    }
                    self.noisy_len = self.moves.len();
                    self.stage = Stage::GoodNoisy;
                }
                Stage::GoodNoisy => {
                    let tt_quiet = self.tt_move_quiet();
                    while self.noisy_index < self.noisy_len {
                        let picked = pick_next(
                            &mut self.moves.as_mut_slice()[..self.noisy_len],
                            self.noisy_index,
                        );
                        let bar = self.threshold.unwrap_or_else(|| {
                            good_noisy_threshold(
                                searcher.seeds(),
                                picked.score,
                                tt_quiet,
                                self.good_emitted,
                            )
                        });
                        if board.see_ge_exchange_with_values(
                            picked.mv,
                            bar,
                            searcher.exchange_values(),
                        ) {
                            self.noisy_index += 1;
                            self.good_emitted += 1;
                            return Some(picked.mv);
                        }
                        self.moves
                            .as_mut_slice()
                            .swap(self.noisy_index, self.bad_len);
                        self.bad_len += 1;
                        self.noisy_index += 1;
                    }
                    self.stage = if skip_quiets {
                        Stage::BadNoisy
                    } else {
                        Stage::GenerateQuiets
                    };
                }
                Stage::GenerateQuiets => {
                    let mut quiets = MoveList::new();
                    board.generate_legal_quiets_staged_into(self.pinned, threats, &mut quiets);
                    searcher.append_compact_quiets(
                        board,
                        threats,
                        quiets.as_slice(),
                        self.tt_move,
                        self.ply,
                        &mut self.moves,
                    );
                    self.stage = Stage::Quiets;
                }
                Stage::Quiets => {
                    let quiets = &mut self.moves.as_mut_slice()[self.noisy_len..];
                    if !skip_quiets && self.quiet_index < quiets.len() {
                        let picked = pick_next(quiets, self.quiet_index);
                        self.quiet_index += 1;
                        return Some(picked.mv);
                    }
                    self.stage = Stage::BadNoisy;
                }
                Stage::BadNoisy => {
                    if self.bad_index < self.bad_len {
                        let mv = self.moves.as_mut_slice()[self.bad_index].mv;
                        self.bad_index += 1;
                        return Some(mv);
                    }
                    self.stage = Stage::Done;
                }
                Stage::Done => return None,
            }
        }
    }
}

impl Searcher {
    /// The root's moves, written into `out` in the order the staged picker
    /// hands them over at any other node, ties included: the table move, the
    /// noisy moves that clear the good-noisy bar, the quiets, the noisy moves
    /// that failed it. The moves come from the staged picker's generators and
    /// are put in order by its selection step, so a tie falls as it does
    /// there, and the bar is decided in that order, so its count of good
    /// moves so far is the same. The whole list is ordered and then narrowed
    /// to the moves `keep` admits, as the donor's root skips a move outside
    /// its lines once its picker hands it over. The scores never rise along
    /// the list, so the root's picker takes it in place until the pool's
    /// scores reorder it.
    pub(in crate::search) fn score_root_moves(
        &self,
        board: &mut Board,
        threats: &Threats,
        keep: impl Fn(Move) -> bool,
        tt_move: Move,
        out: &mut ScoredMoveList,
    ) {
        debug_assert_eq!(out.len(), 0, "the root's list starts empty");
        if !tt_move.is_null() && keep(tt_move) {
            out.push_scored(tt_move, ROOT_TT_SCORE);
        }
        let mut generated = MoveList::new();
        let pinned = board.generate_legal_noisy_staged_into(threats, &mut generated);
        let mut noisy_list = ScoredMoveList::new();
        for &mv in generated.as_slice() {
            if mv != tt_move {
                noisy_list.push_scored(mv, self.compact_noisy_score(board, threats, mv));
            }
        }
        let tt_quiet = is_quiet_move(tt_move);
        let mut good = 0;
        let noisy = noisy_list.as_mut_slice();
        for index in 0..noisy.len() {
            let picked = pick_next(noisy, index);
            debug_assert!(picked.score.abs() < ROOT_ORDER_SPAN, "{}", picked.score);
            let bar = good_noisy_threshold(self.seeds(), picked.score, tt_quiet, good);
            let tier = if board.see_ge_exchange_with_values(picked.mv, bar, self.exchange_values())
            {
                good += 1;
                ROOT_GOOD_NOISY
            } else {
                ROOT_BAD_NOISY
            };
            noisy[index].score = tier + picked.score;
        }
        let good_tier = |score: i32| score > ROOT_GOOD_NOISY - ROOT_ORDER_SPAN;
        for entry in noisy
            .iter()
            .filter(|entry| good_tier(entry.score) && keep(entry.mv))
        {
            out.push_scored(entry.mv, entry.score);
        }
        generated = MoveList::new();
        board.generate_legal_quiets_staged_into(pinned, threats, &mut generated);
        // The quiet generator's queen pushes are the noisy generator's too.
        let mut quiets = generated
            .iter()
            .copied()
            .filter(|&mv| mv != tt_move && is_quiet_move(mv))
            .peekable();
        if quiets.peek().is_some() {
            let ctx = self.quiet_context(board, threats, 0, &mut None);
            let mut quiet_list = ScoredMoveList::new();
            for mv in quiets {
                probe!(self, PickQuiet, true);
                let score =
                    compact_quiet_score(self.seeds(), board, threats, &ctx, &self.td.hist, mv);
                debug_assert!(score.abs() < ROOT_ORDER_SPAN, "{score}");
                quiet_list.push_scored(mv, score);
            }
            let quiet = quiet_list.as_mut_slice();
            for index in 0..quiet.len() {
                let picked = pick_next(quiet, index);
                if keep(picked.mv) {
                    out.push_scored(picked.mv, picked.score);
                }
            }
        }
        for entry in noisy
            .iter()
            .filter(|entry| !good_tier(entry.score) && keep(entry.mv))
        {
            out.push_scored(entry.mv, entry.score);
        }
    }

    /// A noisy move's ordering score in the compact kernel.
    fn compact_noisy_score(&self, board: &Board, threats: &Threats, mv: Move) -> i32 {
        probe!(self, PickNoisy, true);
        let piece = board.moving_piece(mv);
        let captured = board.captured_piece(mv);
        noisy_order(
            self.seeds(),
            &NoisyOrderInput {
                victim: captured.map_or(0, |p| self.exchange_values().as_array()[p as usize]),
                history: self.td.hist.noisy(
                    threats.all,
                    board.side_to_move(),
                    piece,
                    mv.to_sq(),
                    captured,
                ),
                queen_promotion: mv.is_promo() && mv.promo_piece() == Piece::Queen,
                in_check: board.is_in_check(),
                piece: piece as i32,
            },
        )
    }

    /// Score and append the quiets: every legal move that is not noisy, the
    /// rook, bishop and knight pushes among them.
    fn append_compact_quiets(
        &self,
        board: &Board,
        threats: &Threats,
        quiets: &[Move],
        tt_move: Move,
        ply: usize,
        out: &mut ScoredMoveList,
    ) {
        if quiets.is_empty() {
            return;
        }
        let ctx = self.quiet_context(board, threats, ply, &mut None);
        let hist = &self.td.hist;
        for &mv in quiets {
            if mv == tt_move || (mv.is_promo() && mv.promo_piece() == Piece::Queen) {
                continue;
            }
            probe!(self, PickQuiet, true);
            out.push_scored(
                mv,
                compact_quiet_score(self.seeds(), board, threats, &ctx, hist, mv),
            );
        }
    }
}

/// A quiet move's ordering score in the compact kernel.
fn compact_quiet_score(
    seeds: &Seeds,
    board: &Board,
    threats: &Threats,
    ctx: &QuietContext,
    hist: &HistoryTables,
    mv: Move,
) -> i32 {
    let piece = board.moving_piece(mv);
    let index = piece as usize;
    let (from, to) = (mv.from_sq(), mv.to_sq());
    let mut cont = [0; 4];
    for (value, context) in cont.iter_mut().zip(ctx.cont) {
        if let Some(context) = context {
            *value = hist.cont(context, ctx.stm, piece, to);
        }
    }
    quiet_order(
        seeds,
        &QuietOrderInput {
            quiet: hist.quiet(threats.all, ctx.stm, mv),
            pawn: hist.pawn(ctx.pawn_row, ctx.stm, piece, to),
            cont,
            piece: index,
            escape: ctx.threatened[index].contains(from),
            check_square: ctx.check_squares[index].contains(to),
            threatened_to: ctx.threatened[index].contains(to),
            offense: ctx.offense[index].contains(to),
            wall: ctx.wall_pawns.contains(from),
        },
    )
}

#[cfg(test)]
mod tests;
