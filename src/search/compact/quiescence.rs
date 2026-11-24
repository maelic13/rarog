//! The compact kernel's quiescence search, typed by node like the main
//! search: a PV node's children are PV nodes.

use crate::board::{Board, CheckInfo, Color, Move, MoveList, Piece};
use crate::eval::{INF_SCORE, MATE_SCORE, VALUE_NONE};
use crate::infra;
use crate::tt::{Bound, TtProbe, TtStore};

use super::super::node::NodeType;
#[cfg(feature = "pricing")]
use super::super::pricing::Rule;
use super::super::{MAX_PLY, SearchEvent, Searcher};
use super::picker::CompactPicker;
use super::rules;
use super::{bounded_raw, is_quiet_move, kernel_material};

/// The depth a quiescence result is stored at: the table's.
pub(super) const Q_DEPTH: i32 = crate::tt::QUIESCENCE_DEPTH;

impl Searcher {
    pub(super) fn compact_quiescence<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        poll: &mut P,
    ) -> i32 {
        debug_assert!(!NODE::ROOT);
        debug_assert!(NODE::PV || alpha == beta - 1);
        let draw = rules::draw_score(self.td.nodes);
        // A move that repeats a position is available: the side to move can
        // have at least a draw.
        if alpha < draw && board.upcoming_repetition(ply) {
            alpha = draw;
            if alpha >= beta {
                return alpha;
            }
        }
        let in_check = board.is_in_check();
        if NODE::PV {
            self.td.pv_len[ply] = ply;
        }
        self.td.seldepth = self.td.seldepth.max(ply + 1);
        if self.td.stopped {
            return 0;
        }
        if board.is_draw_at(ply) {
            return draw;
        }
        if ply >= MAX_PLY - 1 {
            return if in_check {
                draw
            } else {
                bounded_raw(self.raw_eval(board))
            };
        }

        let hash = board.hash();
        let entry = self.tt_probe(hash);
        let ev = TtProbe::from_entry(entry, ply, board.halfmove_clock());
        let tt_pv = ev.pv_line(NODE::PV);
        if entry.is_some() {
            let cuts = rules::q_tt_cuts(NODE::PV, ev.score, ev.bound, alpha, beta);
            probe!(self, QTtCut, cuts);
            if cuts {
                return ev.score;
            }
        }

        let (raw, eval, correction) = if in_check {
            (VALUE_NONE, VALUE_NONE, self.compact_correction(board, ply))
        } else {
            let raw = if ev.raw_static_eval == VALUE_NONE {
                bounded_raw(self.raw_eval(board))
            } else {
                ev.raw_static_eval
            };
            let material = kernel_material(board, self.exchange_values());
            let (eval, correction) = self.compact_eval(board, raw, material, ply);
            (raw, eval, correction)
        };
        let mut best_score = rules::stand_pat(in_check, NODE::PV, eval, ev.score, ev.bound);
        probe!(self, QStandPat, best_score >= beta);
        if best_score >= beta {
            let value = rules::q_stand_pat_value(self.seeds(), best_score, beta);
            if entry.is_none() {
                self.tt_store(TtStore {
                    key: hash,
                    depth: Q_DEPTH,
                    score: value,
                    bound: Bound::Lower,
                    mv: Move::NULL,
                    ply,
                    static_eval: raw,
                    is_pv: tt_pv,
                });
            }
            return value;
        }
        alpha = alpha.max(best_score);

        let threats = board.threats();
        let stm = board.side_to_move();
        let mut check_info: Option<CheckInfo> = None;
        let mut picker = CompactPicker::new(Move::NULL, None, ply);
        let mut best_move = Move::NULL;
        let mut move_count = 0;
        // Out of check only noisy moves; in check the quiet evasions too,
        // while every evasion so far loses.
        while let Some(mv) = picker.next(
            self,
            board,
            &threats,
            !in_check || best_score > -crate::tt::TB_WIN_SCORE,
        ) {
            self.shared.tt.prefetch(board.key_after_hint(mv));
            move_count += 1;
            let piece = board.moving_piece(mv);
            if best_score > -crate::tt::TB_WIN_SCORE {
                let direct_check = check_info
                    .get_or_insert_with(|| board.check_info())
                    .direct_check_squares(piece)
                    .contains(mv.to_sq());
                let stops = rules::q_stops(self.seeds(), move_count, direct_check);
                probe!(self, QLmp, stops);
                if stops {
                    #[cfg(feature = "pricing")]
                    self.compact_price_q::<NODE, _>(
                        board,
                        Rule::QCount,
                        hash,
                        mv,
                        ply,
                        move_count,
                        alpha,
                        beta,
                        poll,
                    );
                    break;
                }
                let history = if is_quiet_move(mv) {
                    self.quiet_pruning_history(board, threats.all, ply, mv)
                } else {
                    self.td.hist.noisy(
                        threats.all,
                        stm,
                        piece,
                        mv.to_sq(),
                        board.captured_piece(mv),
                    )
                };
                let see_prune = eval != VALUE_NONE
                    && !board.see_ge_exchange_with_values(
                        mv,
                        rules::q_see_threshold(self.seeds(), alpha, eval, correction, history),
                        self.exchange_values(),
                    );
                probe!(self, QSee, see_prune);
                if see_prune {
                    #[cfg(feature = "pricing")]
                    self.compact_price_q::<NODE, _>(
                        board,
                        Rule::QSee,
                        hash,
                        mv,
                        ply,
                        move_count,
                        alpha,
                        beta,
                        poll,
                    );
                    continue;
                }
            }

            self.compact_push_move(board, ply, mv, piece, move_count);
            board.make_move(mv);
            self.shared.tt.prefetch(board.hash());
            #[cfg(test)]
            {
                self.td.moves_made += 1;
            }
            self.count_node(poll);
            let score = -self.compact_quiescence::<NODE, _>(board, -beta, -alpha, ply + 1, poll);
            board.unmake_move(mv);
            self.clear_move(ply);
            if self.td.stopped || self.td.quit {
                return 0;
            }

            if score > best_score {
                best_score = score;
                if score > alpha {
                    best_move = mv;
                    if NODE::PV {
                        self.compact_update_pv(ply, mv);
                    }
                    if score >= beta {
                        break;
                    }
                    alpha = score;
                }
            }
        }

        if in_check && move_count == 0 {
            return -MATE_SCORE + infra::to_i32(ply);
        }
        if move_count == 0 && self.compact_q_stalemated(board, ply) {
            best_score = rules::draw_score(self.td.nodes);
        }
        let noisy_best = !best_move.is_null() && !is_quiet_move(best_move);
        if let Some(bonus) = rules::q_cutoff_reward(self.seeds(), best_score, beta, noisy_best) {
            let piece = board.moving_piece(best_move);
            let captured = board.captured_piece(best_move);
            self.td
                .hist
                .update_noisy(threats.all, stm, piece, best_move.to_sq(), captured, bonus);
        }
        probe!(self, QTail, best_score >= beta);
        let best_score = rules::q_tail_value(self.seeds(), best_score, beta);
        debug_assert!(best_score > -INF_SCORE && best_score < INF_SCORE);
        self.tt_store(TtStore {
            key: hash,
            depth: Q_DEPTH,
            score: best_score,
            bound: if best_score >= beta {
                Bound::Lower
            } else {
                Bound::Upper
            },
            mv: best_move,
            ply,
            static_eval: raw,
            is_pv: tt_pv,
        });
        best_score
    }
}

impl Searcher {
    /// Whether the side to move, out of check with no noisy move, is
    /// stalemated where the stand pat would read material: it has only king
    /// and pawns, the move into the node took a knight or more, and it has no
    /// legal move. Stockfish's narrow test; the pawn pushes go first, since
    /// they settle most cases, and the move list only after every filter.
    fn compact_q_stalemated(&self, board: &Board, ply: usize) -> bool {
        let us = board.side_to_move();
        let pawns = board.pieces(us, Piece::Pawn);
        let pushed = match us {
            Color::White => pawns.north(),
            Color::Black => pawns.south(),
        };
        if (pushed & !board.occupied()).any() {
            return false;
        }
        let pieces = board.pieces(us, Piece::Knight)
            | board.pieces(us, Piece::Bishop)
            | board.pieces(us, Piece::Rook)
            | board.pieces(us, Piece::Queen);
        if pieces.any() {
            return false;
        }
        if !matches!(self.td.stack.back(ply, 1).captured, Some(piece) if piece != Piece::Pawn) {
            return false;
        }
        let mut moves = MoveList::new();
        board.generate_legal_movelist_into(&mut moves);
        moves.is_empty()
    }
}

#[cfg(test)]
mod tests;
