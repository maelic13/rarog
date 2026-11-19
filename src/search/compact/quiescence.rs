//! The compact kernel's quiescence search, typed by node like the main
//! search: a PV node's children are PV nodes.

use crate::board::{Board, CheckInfo, Move};
use crate::eval::{INF_SCORE, MATE_SCORE, VALUE_NONE};
use crate::infra;
use crate::tt::{Bound, TtProbe, TtStore};

use super::super::node::NodeType;
#[cfg(feature = "pricing")]
use super::super::pricing::Rule;
use super::super::{MAX_PLY, SearchEvent, Searcher};
use super::picker::CompactPicker;
use super::rules;
use super::{KERNEL_SEE_VALUES, bounded_raw, is_quiet_move, kernel_material};

/// The depth a quiescence result is stored at.
pub(super) const Q_DEPTH: i32 = 0;

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
            let (eval, correction) = self.compact_eval(board, raw, kernel_material(board), ply);
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
                        KERNEL_SEE_VALUES,
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

#[cfg(test)]
mod tests {
    use super::super::super::node::{NonPv, Pv};
    use super::*;

    fn search<NODE: NodeType>(fen: &str, alpha: i32, beta: i32) -> (Searcher, Board, i32) {
        let mut searcher = Searcher::default();
        let mut board = Board::from_fen(fen).expect("a legal FEN");
        let score = searcher
            .compact_quiescence::<NODE, _>(&mut board, alpha, beta, 1, &mut || SearchEvent::None);
        (searcher, board, score)
    }

    #[test]
    fn a_hanging_queen_is_taken_and_a_mate_is_scored() {
        // White to move takes the queen on d5.
        let fen = "4k3/8/8/3q4/8/8/3R4/4K3 w - - 0 1";
        let (mut searcher, board, score) = search::<Pv>(fen, -INF_SCORE, INF_SCORE);
        let raw = bounded_raw(searcher.raw_eval(&board));
        let (stand_pat, _) = searcher.compact_eval(&board, raw, kernel_material(&board), 1);
        // The capture wins the queen: a lost position becomes a won one, a
        // swing well past the deficit, whatever scale the wrapper gives.
        assert!(
            stand_pat < 0 && score > 0 && score - stand_pat > 3 * -stand_pat / 2,
            "{score} after Rxd5 from {stand_pat}"
        );
        assert_eq!(searcher.td.pv_table[1][1].to_string(), "d2d5");

        // Black is mated: in check with no move.
        let (_, _, mated) = search::<NonPv>(
            "4k3/4Q3/4K3/8/8/8/8/8 b - - 0 1",
            -INF_SCORE,
            -INF_SCORE + 1,
        );
        assert_eq!(mated, -MATE_SCORE + 1);
    }

    #[test]
    fn a_stand_pat_at_beta_is_softened_and_stored_as_a_lower_bound() {
        // White, two queens up, stands far above a beta of -100.
        let fen = "k7/8/8/8/8/8/2QQ4/4K3 w - - 0 1";
        let (mut searcher, board, score) = search::<NonPv>(fen, -101, -100);
        let raw = bounded_raw(searcher.raw_eval(&board));
        let (stand_pat, _) = searcher.compact_eval(&board, raw, kernel_material(&board), 1);
        assert_eq!(
            score,
            rules::q_stand_pat_value(rules::SEEDS, stand_pat, -100)
        );
        assert!(
            score > -100 && score < stand_pat,
            "{score} between beta and {stand_pat}"
        );
        let entry = searcher.shared.tt.probe(board.hash()).expect("stored");
        assert_eq!(entry.bound(), Some(Bound::Lower));
        assert_eq!(i32::from(entry.depth), Q_DEPTH);
    }

    #[test]
    fn quiescence_stops_before_ply_stack_overflow() {
        let mut searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/8/8/3q4/8/8/4KQ2 w - - 0 1").expect("valid FEN");
        let before = board.to_fen();
        let score = searcher.compact_quiescence::<Pv, _>(
            &mut board,
            -INF_SCORE,
            INF_SCORE,
            MAX_PLY - 1,
            &mut || SearchEvent::None,
        );
        assert_eq!(board.to_fen(), before);
        assert!(score.abs() < INF_SCORE);
    }
}
