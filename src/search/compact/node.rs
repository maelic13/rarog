//! The compact kernel's node: the order at entry, the table, pruning before
//! the moves, the null move, ProbCut, the singular search, the move loop with
//! its two reduction formulas, and the stores and updates at the end.

use crate::board::{Board, CheckInfo, Move, MoveList, Piece};
use crate::eval::{INF_SCORE, MATE_SCORE, VALUE_NONE};
use crate::infra;
use crate::tt::{Bound, TB_WIN_SCORE, TtProbe, TtStore};

use super::super::history::{HistoryTables, SearchedMoves};
use super::super::movepick::{MovePicker, ScoredMoveList, diversify_root_scores};
use super::super::node::{NodeType, NonPv, Pv, TbVerdict, tb_verdict};
#[cfg(feature = "pricing")]
use super::super::pricing::{ReSearch, Rule, Verdict};
use super::super::{MAX_PLY, SearchEvent, Searcher};
use super::picker::{CompactPicker, Stage};
use super::rules::{self, is_decisive};
use super::{KERNEL_SEE_VALUES, bounded_raw, is_quiet_move, kernel_material};

/// A score at or beyond the tablebase loss band.
const fn is_loss(score: i32) -> bool {
    score <= -TB_WIN_SCORE
}

/// The node's moves: the root's whole scored list, or the staged picker.
enum Moves {
    Root(MovePicker),
    Staged(CompactPicker),
}

#[cfg(feature = "pricing")]
impl Moves {
    /// The moves a stage-ending decision skips with the one it prices.
    fn left_behind(&self) -> usize {
        match self {
            Self::Root(_) => 0,
            Self::Staged(picker) => picker.moves_left_behind(),
        }
    }
}

impl Searcher {
    /// Search the root position at `depth` inside the window with the
    /// compact kernel.
    pub(in crate::search) fn compact_root<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        depth: i32,
        alpha: i32,
        beta: i32,
        poll: &mut P,
    ) -> i32 {
        self.compact_negamax::<super::super::node::Root, _>(
            board,
            alpha,
            beta,
            depth,
            false,
            0,
            0,
            Move::NULL,
            poll,
        )
    }

    pub(super) fn compact_negamax<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        mut alpha: i32,
        mut beta: i32,
        depth: i32,
        cut_node: bool,
        ply: usize,
        last_critical_ply: usize,
        excluded: Move,
        poll: &mut P,
    ) -> i32 {
        debug_assert_eq!(NODE::ROOT, ply == 0, "the root node type is exactly ply 0");
        debug_assert!(NODE::PV || alpha == beta - 1);
        debug_assert!(
            self.td.stack[ply].mv.is_null(),
            "ply {ply} entered with a move left in its frame"
        );
        let has_excluded = !excluded.is_null();
        #[cfg(feature = "pricing")]
        let pricing_entry = super::pricing::CompactEntry {
            depth,
            alpha,
            beta,
            cut_node,
            last_critical_ply,
            // The last ply returns below before it reads a child's frame.
            child_cutoffs: if ply + 1 < MAX_PLY {
                self.td.stack[ply + 1].cutoff_count
            } else {
                0
            },
            nodes: self.td.nodes,
        };
        let in_check = board.is_in_check();
        if NODE::PV {
            self.td.pv_len[ply] = ply;
        }
        if self.td.stopped {
            return 0;
        }
        let depth = if NODE::ROOT {
            depth
        } else {
            self.compact_line_room(depth, ply)
        };
        if depth <= 0 {
            return if NODE::PV {
                self.compact_quiescence::<Pv, _>(board, alpha, beta, ply, poll)
            } else {
                self.compact_quiescence::<NonPv, _>(board, alpha, beta, ply, poll)
            };
        }
        let draw = rules::draw_score(self.td.nodes);
        if !NODE::ROOT && alpha < draw && board.upcoming_repetition(ply) {
            alpha = draw;
            if alpha >= beta {
                return alpha;
            }
        }
        self.td.seldepth = self.td.seldepth.max(ply + 1);
        if !NODE::ROOT {
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
            alpha = alpha.max(-MATE_SCORE + infra::to_i32(ply));
            beta = beta.min(MATE_SCORE - infra::to_i32(ply) - 1);
            if alpha >= beta {
                return alpha;
            }
        }
        let mut depth = depth.min(infra::to_i32(MAX_PLY) - 1);
        let stm = board.side_to_move();
        let hash = board.hash();

        // A tablebase result ends the node where it decides the window; at a
        // PV node a win otherwise raises the floor and a loss caps the moves.
        let mut tb_floor = None;
        let mut tb_cap = None;
        if let Some((score, bound)) = self.syzygy_wdl_bound(board, depth, ply, excluded) {
            match tb_verdict(score, bound, alpha, beta, NODE::PV) {
                TbVerdict::Return => {
                    self.tt_store(TtStore {
                        key: hash,
                        depth: (depth + 6).min(infra::to_i32(MAX_PLY) - 1),
                        score,
                        bound,
                        mv: Move::NULL,
                        ply,
                        static_eval: VALUE_NONE,
                        is_pv: NODE::PV,
                    });
                    return score;
                }
                TbVerdict::Floor => {
                    tb_floor = Some(score);
                    alpha = alpha.max(score);
                }
                TbVerdict::Cap => tb_cap = Some(score),
                TbVerdict::Search => {}
            }
        }

        let entry = self.tt_probe(hash);
        let ev = TtProbe::from_entry(entry, ply, board.halfmove_clock());
        let tt_score = ev.score;
        let tt_bound = ev.bound;
        let tt_depth = if entry.is_some() { ev.depth } else { 0 };
        let mut tt_pv = ev.pv_line(NODE::PV);
        let tt_was_pv = ev.pv_line(false);
        let mut tt_move = ev
            .mv
            .and_then(|mv| board.legal_move(mv))
            .unwrap_or(Move::NULL);
        if NODE::ROOT && !self.td.root_moves.is_empty() && !self.td.root_moves.contains(&tt_move) {
            // A later MultiPV line excludes the moves already ranked; the
            // line's own candidate stands in.
            tt_move = if self.td.multipv_line > 0 {
                self.td.root_moves[0]
            } else {
                Move::NULL
            };
        }
        let tt_quiet = is_quiet_move(tt_move);

        let tt_cuts = entry.is_some()
            && rules::tt_cuts(
                self.seeds(),
                &rules::TtCutInput {
                    pv: NODE::PV,
                    excluded: has_excluded,
                    tt_depth,
                    depth,
                    tt_score,
                    alpha,
                    beta,
                    bound: tt_bound,
                    cut_node,
                },
            );
        if entry.is_some() {
            probe!(self, TtCut, tt_cuts);
        }
        if tt_cuts {
            let rewards = rules::tt_cut_rewards(
                self.seeds(),
                &rules::TtCutRewardInput {
                    tt_quiet,
                    tt_score,
                    beta,
                    parent_move_count: self.td.stack.back(ply, 1).move_count,
                    depth,
                },
            );
            if let Some((quiet_bonus, cont_bonus)) = rewards {
                let piece = board.moving_piece(tt_move);
                self.td
                    .hist
                    .update_quiet(board.threats().all, stm, tt_move, quiet_bonus);
                self.update_continuations(ply, stm, piece, tt_move.to_sq(), cont_bonus);
            }
            if rules::tt_cut_returns(self.seeds(), i32::from(board.halfmove_clock())) {
                return tt_score;
            }
        }

        let threats = board.threats();
        self.td.stack[ply].threats = threats.all;
        // Read by the evaluation and the null-move gate.
        let material = kernel_material(board);
        let (raw, eval, correction) = if in_check {
            (VALUE_NONE, VALUE_NONE, self.compact_correction(board, ply))
        } else if has_excluded {
            let eval = self.td.stack[ply].static_eval;
            (VALUE_NONE, eval, self.compact_correction(board, ply))
        } else {
            let raw = if ev.raw_static_eval == VALUE_NONE {
                let raw = bounded_raw(self.raw_eval(board));
                if entry.is_none() {
                    self.tt_store_eval(hash, raw, tt_pv);
                }
                raw
            } else {
                ev.raw_static_eval
            };
            let (eval, correction) = self.compact_eval(board, raw, material, ply);
            (raw, eval, correction)
        };
        let estimate = rules::estimated_score(&rules::EstimateInput {
            in_check,
            excluded: has_excluded,
            tt_score,
            bound: tt_bound,
            eval,
        });
        probe!(self, Estimate, estimate != eval);

        self.td.stack[ply].static_eval = eval;
        self.td.stack[ply].tt_move = tt_move;
        self.td.stack[ply].tt_pv = tt_pv;
        self.td.stack[ply].reduction = 0;
        self.td.stack[ply].move_count = 0;
        if ply + 2 < MAX_PLY {
            self.td.stack[ply + 2].cutoff_count = 0;
        }

        let parent = *self.td.stack.back(ply, 1);
        let eval_bonus = rules::eval_bonus_applies(
            self.seeds(),
            &rules::EvalBonusGate {
                root: NODE::ROOT,
                in_check,
                excluded: has_excluded,
                parent_quiet: is_quiet_move(parent.mv),
                parent_eval_valid: parent.static_eval != VALUE_NONE,
                depth,
                has_entry: entry.is_some(),
            },
        );
        if eval_bonus {
            let bonus = rules::eval_difference_bonus(self.seeds(), eval, parent.static_eval);
            probe!(self, EvalBonus, true);
            self.td
                .hist
                .update_quiet(parent.threats, !stm, parent.mv, bonus);
        }
        if !NODE::ROOT && !in_check && !has_excluded && parent.static_eval != VALUE_NONE {
            #[cfg(feature = "probes")]
            let before = depth;
            depth = rules::hindsight_depth(
                self.seeds(),
                depth,
                parent.reduction,
                eval + parent.static_eval,
                tt_pv,
            );
            probe!(self, Hindsight, depth != before);
        }

        let potential_singularity =
            rules::singular_candidate(self.seeds(), depth, tt_pv, tt_depth, tt_bound, tt_score);
        let improvement = if in_check {
            0
        } else {
            let two_back = self.td.stack.back(ply, 2).static_eval;
            let four_back = self.td.stack.back(ply, 4).static_eval;
            if two_back != VALUE_NONE {
                eval - two_back
            } else if four_back != VALUE_NONE {
                eval - four_back
            } else {
                0
            }
        };
        let improving = improvement > 0;
        probe!(self, SingularGate, potential_singularity);

        let razor = rules::razors(
            self.seeds(),
            &rules::RazorInput {
                pv: NODE::PV,
                in_check,
                estimate,
                alpha,
                depth,
                tt_quiet,
                bound: tt_bound,
            },
        );
        let rfp_input = |board: &Board| rules::RfpInput {
            tt_pv,
            in_check,
            excluded: has_excluded,
            estimate,
            beta,
            depth,
            improvement,
            correction,
            unthreatened: (threats.all & board.color_occ(stm)).is_empty(),
        };
        let child_cutoffs = self.td.stack[ply + 1].cutoff_count;
        let nmp_min_ply = self.td.nmp_min_ply;
        let null_input = |board: &Board| rules::NullGateInput {
            cut_node,
            in_check,
            excluded: has_excluded,
            singular: potential_singularity,
            estimate,
            beta,
            depth,
            tt_pv,
            improvement,
            child_cutoffs,
            ply: infra::to_i32(ply),
            nmp_min_ply,
            material,
            tt_capture_refutes: tt_bound == Some(Bound::Lower)
                && tt_move.is_capture()
                && board.captured_piece(tt_move).is_some_and(|piece| {
                    KERNEL_SEE_VALUES.as_array()[piece as usize]
                        >= KERNEL_SEE_VALUES.as_array()[Piece::Knight as usize]
                }),
        };
        // The probes count each of the three gates at every node that reaches
        // them, so there all three are asked before any acts; otherwise a gate
        // is asked only when the one before it did not end the node.
        #[cfg(feature = "probes")]
        {
            probe!(self, Razor, razor);
            probe!(self, Rfp, rules::rfp_cuts(self.seeds(), &rfp_input(board)));
            probe!(
                self,
                NmpGate,
                rules::nmp_gate(self.seeds(), &null_input(board))
            );
        }
        if razor && !pricing_off!(self, ply, Razor) {
            let score = self.compact_quiescence::<NonPv, _>(board, alpha, beta, ply, poll);
            #[cfg(feature = "pricing")]
            self.compact_price_cut::<NODE, _>(
                board,
                Rule::Razor,
                hash,
                depth,
                ply,
                has_excluded,
                &pricing_entry,
                Verdict::Shortcut {
                    returned: score,
                    alpha,
                },
                poll,
            );
            return score;
        }
        let rfp_cuts = rules::rfp_cuts(self.seeds(), &rfp_input(board));
        if rfp_cuts && !pricing_off!(self, ply, ReverseFutility) {
            let value = rules::rfp_value(self.seeds(), estimate, beta);
            #[cfg(feature = "pricing")]
            self.compact_price_cut::<NODE, _>(
                board,
                Rule::ReverseFutility,
                hash,
                depth,
                ply,
                has_excluded,
                &pricing_entry,
                Verdict::Cut { beta },
                poll,
            );
            return value;
        }
        let null_move = rules::nmp_gate(self.seeds(), &null_input(board));
        if null_move && !pricing_off!(self, ply, NullMove) {
            debug_assert!(!parent.mv.is_null(), "a null move after a null move");
            let reduction = rules::nmp_reduction(self.seeds(), improving, depth, estimate, beta);
            probe!(self, NmpR, true);
            let null_beta = rules::null_bound(depth, beta, tt_score, tt_bound, tt_depth);
            self.td.stack[ply].laterality = 0;
            board.make_null_move();
            self.shared.tt.prefetch(board.hash());
            let score = -self.compact_negamax::<NonPv, _>(
                board,
                -null_beta,
                -null_beta + 1,
                depth - reduction,
                false,
                ply + 1,
                ply + 1,
                Move::NULL,
                poll,
            );
            board.unmake_null_move();
            if self.td.stopped || self.td.quit {
                return 0;
            }
            let outcome = rules::null_outcome(
                self.seeds(),
                score,
                null_beta,
                beta,
                depth,
                reduction,
                self.td.nmp_min_ply,
            );
            probe!(self, NmpResult, outcome != rules::NullOutcome::Fail);
            match outcome {
                rules::NullOutcome::Fail => {}
                rules::NullOutcome::Cut => {
                    #[cfg(feature = "pricing")]
                    self.compact_price_cut::<NODE, _>(
                        board,
                        Rule::NullMove,
                        hash,
                        depth,
                        ply,
                        has_excluded,
                        &pricing_entry,
                        Verdict::Cut { beta },
                        poll,
                    );
                    return score;
                }
                rules::NullOutcome::Verify(verify_depth) => {
                    self.td.nmp_min_ply = infra::to_i32(ply) + 3 * verify_depth / 4;
                    let verified = self.compact_negamax::<NonPv, _>(
                        board,
                        beta - 1,
                        beta,
                        verify_depth,
                        false,
                        ply,
                        last_critical_ply,
                        Move::NULL,
                        poll,
                    );
                    self.td.nmp_min_ply = 0;
                    if self.td.stopped || self.td.quit {
                        return 0;
                    }
                    if verified >= beta {
                        #[cfg(feature = "pricing")]
                        self.compact_price_cut::<NODE, _>(
                            board,
                            Rule::NullMove,
                            hash,
                            depth,
                            ply,
                            has_excluded,
                            &pricing_entry,
                            Verdict::Cut { beta },
                            poll,
                        );
                        return score;
                    }
                }
            }
        }

        let mut probcut_beta = rules::probcut_beta(self.seeds(), beta, improving);
        let probcut = rules::probcut_gate(&rules::ProbcutGateInput {
            cut_node,
            beta,
            probcut_beta,
            tt_score,
            eval,
            tt_quiet,
        });
        probe!(self, ProbcutGate, probcut);
        if probcut && !pricing_off!(self, ply, ProbCut) {
            let bar = rules::probcut_see_bar(self.seeds(), probcut_beta, eval);
            let mut picker = CompactPicker::new(Move::NULL, Some(bar), ply);
            while let Some(mv) = picker.next(self, board, &threats, true) {
                if picker.stage() == Stage::BadNoisy {
                    break;
                }
                if mv == excluded {
                    continue;
                }
                let piece = board.moving_piece(mv);
                self.shared.tt.prefetch(board.key_after_hint(mv));
                self.compact_push_move(board, ply, mv, piece, 0);
                board.make_move(mv);
                self.shared.tt.prefetch(board.hash());
                #[cfg(test)]
                {
                    self.td.moves_made += 1;
                }
                self.count_node(poll);
                let mut score = -self.compact_quiescence::<NonPv, _>(
                    board,
                    -probcut_beta,
                    -probcut_beta + 1,
                    ply + 1,
                    poll,
                );
                let verification =
                    rules::probcut_depth(self.seeds(), depth, improving, score, probcut_beta);
                probe!(
                    self,
                    ProbcutDepth,
                    score >= probcut_beta && verification.depth > 0
                );
                let mut probcut_depth = verification.depth;
                if score >= probcut_beta && probcut_depth > 0 {
                    let adjusted = verification.adjusted_beta;
                    score = -self.compact_negamax::<NonPv, _>(
                        board,
                        -adjusted,
                        -adjusted + 1,
                        probcut_depth,
                        false,
                        ply + 1,
                        last_critical_ply,
                        Move::NULL,
                        poll,
                    );
                    if score < adjusted && probcut_beta < adjusted {
                        probcut_depth = verification.base;
                        score = -self.compact_negamax::<NonPv, _>(
                            board,
                            -probcut_beta,
                            -probcut_beta + 1,
                            probcut_depth,
                            false,
                            ply + 1,
                            last_critical_ply,
                            Move::NULL,
                            poll,
                        );
                    } else {
                        probcut_beta = adjusted;
                    }
                }
                board.unmake_move(mv);
                self.clear_move(ply);
                if self.td.stopped || self.td.quit {
                    return 0;
                }
                if score >= probcut_beta {
                    self.tt_store(TtStore {
                        key: hash,
                        depth: probcut_depth + 1,
                        score,
                        bound: Bound::Lower,
                        mv,
                        ply,
                        static_eval: raw,
                        is_pv: tt_pv,
                    });
                    let value = rules::probcut_value(self.seeds(), score, beta);
                    #[cfg(feature = "pricing")]
                    self.compact_price_cut::<NODE, _>(
                        board,
                        Rule::ProbCut,
                        hash,
                        depth,
                        ply,
                        has_excluded,
                        &pricing_entry,
                        Verdict::Cut { beta },
                        poll,
                    );
                    return value;
                }
            }
        }

        let mut extension = 0;
        let mut singular_score = VALUE_NONE;
        let singular_searched =
            !NODE::ROOT && !has_excluded && potential_singularity && !tt_move.is_null();
        let ldse = rules::ldse_extends(
            self.seeds(),
            singular_searched,
            depth,
            in_check,
            cut_node,
            estimate,
            alpha,
        );
        probe!(self, Ldse, ldse);
        if singular_searched {
            let margin = rules::singular_margin(
                self.seeds(),
                depth,
                tt_bound == Some(Bound::Exact),
                tt_pv && !NODE::PV,
            );
            let singular_beta = tt_score - margin;
            let nodes_before = self.td.nodes;
            singular_score = self.compact_negamax::<NonPv, _>(
                board,
                singular_beta - 1,
                singular_beta,
                (depth - 1) / 2,
                cut_node,
                ply,
                ply,
                tt_move,
                poll,
            );
            self.td.stack[ply].tt_pv = tt_pv;
            if self.td.stopped || self.td.quit {
                return 0;
            }
            let outcome = rules::singular_outcome(
                self.seeds(),
                &rules::SingularInput {
                    singular_score,
                    singular_beta,
                    beta,
                    tt_score,
                    pv: NODE::PV,
                    tt_was_pv,
                    tt_quiet,
                    correction,
                    cut_node,
                    searched_a_move: self.td.nodes > nodes_before,
                },
            );
            probe!(
                self,
                SingularResult,
                matches!(
                    outcome,
                    rules::SingularOutcome::Extend(_) | rules::SingularOutcome::MultiCut(_)
                )
            );
            match outcome {
                rules::SingularOutcome::None => {}
                rules::SingularOutcome::Extend(plies) => extension = plies,
                rules::SingularOutcome::MultiCut(score) if !pricing_off!(self, ply, MultiCut) => {
                    #[cfg(feature = "pricing")]
                    self.compact_price_cut::<NODE, _>(
                        board,
                        Rule::MultiCut,
                        hash,
                        depth,
                        ply,
                        has_excluded,
                        &pricing_entry,
                        Verdict::Cut { beta },
                        poll,
                    );
                    return score;
                }
                // Switched off by a pricing probe at this ply.
                rules::SingularOutcome::MultiCut(_) => {}
                rules::SingularOutcome::Demote => tt_move = Move::NULL,
            }
        } else if ldse {
            extension = 1;
        }

        let mut moves = if NODE::ROOT {
            Moves::Root(MovePicker::new(tt_move))
        } else {
            Moves::Staged(CompactPicker::new(tt_move, None, ply))
        };
        if let Moves::Root(picker) = &mut moves {
            self.compact_root_moves(board, &threats, tt_move, picker.scored_mut());
        }
        let mut check_info: Option<CheckInfo> = None;
        let mut best_score = -INF_SCORE;
        let mut best_move = Move::NULL;
        let mut bound = Bound::Upper;
        let mut quiet_moves = SearchedMoves::new();
        let mut noisy_moves = SearchedMoves::new();
        let mut move_count = 0;
        let mut skip_quiets = false;
        let mut search_count = 0u32;
        let mut tt_move_score = VALUE_NONE;
        let original_alpha = alpha;

        loop {
            let (mv, bad_stage) = match &mut moves {
                Moves::Root(picker) => match picker.next() {
                    Some(picked) => (picked.mv, false),
                    None => break,
                },
                Moves::Staged(picker) => match picker.next(self, board, &threats, skip_quiets) {
                    Some(mv) => (mv, picker.stage() == Stage::BadNoisy),
                    None => break,
                },
            };
            if mv == excluded {
                continue;
            }
            // The child's cluster is asked for as soon as the move is known, so
            // it arrives while the pruning rules and the make run; the exact key
            // is asked for again after the make.
            self.shared.tt.prefetch(board.key_after_hint(mv));
            search_count = 0;
            move_count += 1;
            let is_quiet = is_quiet_move(mv);
            let piece = board.moving_piece(mv);
            let direct_check = check_info
                .get_or_insert_with(|| board.check_info())
                .direct_check_squares(piece)
                .contains(mv.to_sq());
            let history = if is_quiet {
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

            if !NODE::ROOT && !is_loss(best_score) {
                let prune = rules::MovePruneInput {
                    in_check,
                    direct_check,
                    quiet: is_quiet,
                    depth,
                    history,
                    eval,
                    alpha,
                    beta,
                    improvement,
                    correction,
                    move_count,
                    bad_stage,
                };
                let lmp = rules::lmp_prunes(self.seeds(), &prune);
                probe!(self, Lmp, lmp);
                if lmp {
                    #[cfg(feature = "pricing")]
                    self.compact_price_prune::<NODE, _>(
                        board,
                        Rule::LateMovePruning,
                        hash,
                        mv,
                        piece,
                        depth,
                        ply,
                        cut_node,
                        has_excluded,
                        move_count,
                        moves.left_behind(),
                        alpha,
                        last_critical_ply,
                        poll,
                    );
                    skip_quiets = true;
                    continue;
                }
                let futility = rules::futility_prunes(self.seeds(), &prune);
                probe!(self, Futility, futility);
                if futility {
                    #[cfg(feature = "pricing")]
                    self.compact_price_prune::<NODE, _>(
                        board,
                        Rule::QuietFutility,
                        hash,
                        mv,
                        piece,
                        depth,
                        ply,
                        cut_node,
                        has_excluded,
                        move_count,
                        moves.left_behind(),
                        alpha,
                        last_critical_ply,
                        poll,
                    );
                    let value = rules::futility_value(self.seeds(), &prune);
                    if !is_decisive(best_score) && best_score < value {
                        best_score = value;
                    }
                    skip_quiets = true;
                    continue;
                }
                let bad_noisy = rules::bad_noisy_prunes(self.seeds(), &prune);
                probe!(self, BadNoisy, bad_noisy);
                if bad_noisy {
                    #[cfg(feature = "pricing")]
                    self.compact_price_prune::<NODE, _>(
                        board,
                        Rule::BadNoisyFutility,
                        hash,
                        mv,
                        piece,
                        depth,
                        ply,
                        cut_node,
                        has_excluded,
                        move_count,
                        moves.left_behind(),
                        alpha,
                        last_critical_ply,
                        poll,
                    );
                    let value = rules::bad_noisy_value(self.seeds(), &prune);
                    if !is_decisive(best_score) && best_score < value {
                        best_score = value;
                    }
                    break;
                }
                let history_prune = rules::history_prunes(self.seeds(), &prune);
                probe!(self, HistoryPrune, history_prune);
                if history_prune {
                    #[cfg(feature = "pricing")]
                    self.compact_price_prune::<NODE, _>(
                        board,
                        Rule::HistoryPruning,
                        hash,
                        mv,
                        piece,
                        depth,
                        ply,
                        cut_node,
                        has_excluded,
                        move_count,
                        0,
                        alpha,
                        last_critical_ply,
                        poll,
                    );
                    continue;
                }
                let see_prune = !in_check
                    && !board.see_ge_exchange_with_values(
                        mv,
                        rules::see_threshold(self.seeds(), is_quiet, depth, history),
                        KERNEL_SEE_VALUES,
                    );
                probe!(self, SeeMargin, see_prune);
                if see_prune {
                    #[cfg(feature = "pricing")]
                    self.compact_price_prune::<NODE, _>(
                        board,
                        if is_quiet {
                            Rule::QuietSee
                        } else {
                            Rule::NoisySee
                        },
                        hash,
                        mv,
                        piece,
                        depth,
                        ply,
                        cut_node,
                        has_excluded,
                        move_count,
                        0,
                        alpha,
                        last_critical_ply,
                        poll,
                    );
                    continue;
                }
            }

            let nodes_before = self.td.nodes;
            self.compact_push_move(board, ply, mv, piece, move_count);
            board.make_move(mv);
            self.shared.tt.prefetch(board.hash());
            #[cfg(test)]
            {
                self.td.moves_made += 1;
            }
            self.count_node(poll);

            let mut new_depth = depth - 1 + if move_count == 1 { extension } else { 0 };
            let mut score = 0;
            if depth >= 2 && move_count >= 2 {
                let reduction = rules::late_move_reduction(
                    self.seeds(),
                    &self.compact_reduction_input(
                        board,
                        ply,
                        last_critical_ply,
                        rules::ReductionInput {
                            depth,
                            improvement,
                            correction,
                            exact: bound == Bound::Exact,
                            tt_score,
                            tt_depth,
                            alpha,
                            beta,
                            quiet: is_quiet,
                            history,
                            estimate,
                            since_critical: 0,
                            pv: NODE::PV,
                            root_delta: self.td.root_delta.max(1),
                            laterality: 0,
                            tt_pv,
                            cut_node,
                            tt_move_null: tt_move.is_null(),
                            gives_check: false,
                            child_cutoffs: self.td.stack[ply + 1].cutoff_count,
                            tt_move_score,
                            singular_score,
                            parent_reduction: parent.reduction,
                            spread: 0,
                            is_tt_move: mv == tt_move,
                        },
                        rules::SpreadTerm::Lmr,
                    ),
                );
                let reduced = rules::lmr_depth(new_depth, reduction, NODE::PV);
                probe!(self, Lmr, true);
                self.td.stack[ply].reduction = reduction;
                score = -self.compact_negamax::<NonPv, _>(
                    board,
                    -alpha - 1,
                    -alpha,
                    reduced,
                    true,
                    ply + 1,
                    ply + 1,
                    Move::NULL,
                    poll,
                );
                self.td.stack[ply].reduction = 0;
                search_count += 1;
                #[cfg(feature = "pricing")]
                if score <= alpha && reduced < new_depth {
                    self.compact_price_reduction::<NODE, _>(
                        board,
                        Rule::LmrFailLow,
                        hash,
                        mv,
                        depth,
                        ply,
                        cut_node,
                        has_excluded,
                        move_count,
                        &ReSearch {
                            new_depth,
                            alpha,
                            cut_node: !cut_node,
                            last_critical_ply,
                        },
                        poll,
                    );
                }
                if score > alpha {
                    new_depth = rules::research_depth(
                        self.seeds(),
                        NODE::ROOT,
                        score,
                        best_score,
                        new_depth,
                    );
                    probe!(self, Research, new_depth > reduced);
                    if new_depth > reduced {
                        score = -self.compact_negamax::<NonPv, _>(
                            board,
                            -alpha - 1,
                            -alpha,
                            new_depth,
                            !cut_node,
                            ply + 1,
                            last_critical_ply,
                            Move::NULL,
                            poll,
                        );
                        search_count += 1;
                    }
                }
            } else if !NODE::PV || move_count >= 2 {
                let reduction = rules::unreduced_reduction(
                    self.seeds(),
                    &self.compact_reduction_input(
                        board,
                        ply,
                        last_critical_ply,
                        rules::ReductionInput {
                            depth,
                            improvement,
                            correction,
                            exact: false,
                            tt_score,
                            tt_depth,
                            alpha,
                            beta,
                            quiet: is_quiet,
                            history,
                            estimate,
                            since_critical: 0,
                            pv: NODE::PV,
                            root_delta: 1,
                            laterality: 0,
                            tt_pv,
                            cut_node,
                            tt_move_null: tt_move.is_null(),
                            gives_check: false,
                            child_cutoffs: self.td.stack[ply + 1].cutoff_count,
                            tt_move_score,
                            singular_score,
                            parent_reduction: parent.reduction,
                            spread: 0,
                            is_tt_move: mv == tt_move,
                        },
                        rules::SpreadTerm::Unreduced,
                    ),
                );
                let reduced = rules::unreduced_depth(self.seeds(), new_depth, reduction);
                probe!(self, Unreduced, reduced < new_depth);
                score = -self.compact_negamax::<NonPv, _>(
                    board,
                    -alpha - 1,
                    -alpha,
                    reduced,
                    !cut_node,
                    ply + 1,
                    if move_count == 1 {
                        last_critical_ply
                    } else {
                        ply + 1
                    },
                    Move::NULL,
                    poll,
                );
                search_count += 1;
                #[cfg(feature = "pricing")]
                if score <= alpha && reduced < new_depth {
                    self.compact_price_reduction::<NODE, _>(
                        board,
                        Rule::FdsFailLow,
                        hash,
                        mv,
                        depth,
                        ply,
                        cut_node,
                        has_excluded,
                        move_count,
                        &ReSearch {
                            new_depth,
                            alpha,
                            cut_node: !cut_node,
                            last_critical_ply: if move_count == 1 {
                                last_critical_ply
                            } else {
                                ply + 1
                            },
                        },
                        poll,
                    );
                }
            }
            if NODE::PV && (move_count == 1 || score > alpha) {
                if mv == tt_move && tt_depth > 1 {
                    new_depth = new_depth.max(1);
                }
                score = -self.compact_negamax::<Pv, _>(
                    board,
                    -beta,
                    -alpha,
                    new_depth,
                    false,
                    ply + 1,
                    last_critical_ply,
                    Move::NULL,
                    poll,
                );
                search_count += 1;
            }
            board.unmake_move(mv);
            self.clear_move(ply);
            if self.td.stopped || self.td.quit {
                return 0;
            }
            if NODE::ROOT {
                let move_nodes = self.td.nodes.saturating_sub(nodes_before);
                let report_depth = self.td.root_report_depth.max(depth);
                self.record_root_move_search(mv, report_depth, score, alpha, beta, move_nodes);
                // The clock's best-move change: a move after the first raising
                // alpha on line one.
                if score > alpha && move_count > 1 && self.td.multipv_line == 0 {
                    self.td.best_move_changes += 1;
                }
            }
            if mv == tt_move {
                tt_move_score = score;
            }
            if score > best_score {
                best_score = score;
                if score > alpha {
                    bound = Bound::Exact;
                    best_move = mv;
                    if NODE::PV {
                        self.compact_update_pv(ply, mv);
                    }
                    if score >= beta {
                        bound = Bound::Lower;
                        self.td.stack[ply].cutoff_count += 1;
                        break;
                    }
                    alpha = score;
                    if !(NODE::ROOT && self.td.multipv_line > 0) && mv != tt_move {
                        self.tt_store(TtStore {
                            key: hash,
                            depth,
                            score,
                            bound: Bound::Lower,
                            mv,
                            ply,
                            static_eval: raw,
                            is_pv: true,
                        });
                    }
                }
            }
            if mv != best_move && move_count < 32 {
                if is_quiet {
                    quiet_moves.push(mv);
                } else {
                    noisy_moves.push(mv);
                }
            }
        }

        if move_count == 0 {
            if has_excluded {
                return -TB_WIN_SCORE + 1;
            }
            // Stalemate: the draw score read now, after the searches before
            // the moves have moved the node count.
            return if in_check {
                -MATE_SCORE + infra::to_i32(ply)
            } else {
                rules::draw_score(self.td.nodes)
            };
        }

        if !best_move.is_null() {
            self.compact_best_move_updates(
                board,
                ply,
                depth,
                cut_node,
                best_move,
                &quiet_moves,
                &noisy_moves,
                search_count > 1 && best_score >= beta,
            );
        }
        if !NODE::ROOT && bound == Bound::Upper && (cut_node || NODE::PV) {
            self.compact_fail_low_reward(board, ply, depth, best_score, eval, in_check);
        }
        tt_pv |= !NODE::ROOT
            && bound == Bound::Upper
            && move_count > 2
            && self.td.stack.back(ply, 1).tt_pv;

        // A tablebase win no move beat stands as a lower bound; a tablebase
        // loss caps what the moves claimed.
        if let Some(floor) = tb_floor
            && best_score <= floor
        {
            best_score = floor;
            bound = Bound::Lower;
        }
        if let Some(cap) = tb_cap {
            best_score = best_score.min(cap);
        }
        debug_assert!(original_alpha <= alpha);

        if !(has_excluded || NODE::ROOT && self.td.multipv_line > 0) {
            let store = TtStore {
                key: hash,
                depth,
                score: best_score,
                bound,
                mv: best_move,
                ply,
                static_eval: raw,
                is_pv: tt_pv,
            };
            if NODE::PV {
                self.tt_store_forced(store);
            } else {
                self.tt_store(store);
            }
        }
        let correction_update = rules::correction_update(
            self.seeds(),
            &rules::CorrectionInput {
                in_check,
                noisy_best: !best_move.is_null() && !is_quiet_move(best_move),
                bound,
                best_score,
                eval,
                depth,
            },
        );
        probe!(self, CorrUpdate, correction_update.is_some());
        if let Some(bonus) = correction_update {
            self.compact_train_correction(board, ply, bonus);
        }
        debug_assert!(best_score > -INF_SCORE && best_score < INF_SCORE);
        #[cfg(feature = "pricing")]
        self.pricing_close(ply, bound == Bound::Upper);
        best_score
    }

    /// The root's moves into `scored`: every legal move in the root list in
    /// the kernel's order, the pool's searched moves lifted above it, rotated
    /// for a helper.
    fn compact_root_moves(
        &self,
        board: &mut Board,
        threats: &crate::board::Threats,
        tt_move: Move,
        scored: &mut ScoredMoveList,
    ) {
        let mut legal = MoveList::new();
        board.generate_legal_movelist_into(&mut legal);
        let root_moves = &self.td.root_moves;
        let narrowed = !root_moves.is_empty() && legal.iter().any(|mv| root_moves.contains(mv));
        self.score_root_moves(
            board,
            threats,
            |mv| !narrowed || root_moves.contains(&mv),
            tt_move,
            scored,
        );
        if scored.len() > 1 {
            self.apply_shared_root_scores(scored);
        }
        if self.td.root_move_offset > 0 && scored.len() > 1 {
            let offset = self.td.root_move_offset % scored.len();
            diversify_root_scores(scored.as_mut_slice(), offset);
        }
    }

    /// The reduction inputs read off the stack and the board after the move
    /// was made: the line's lateness, the distance to the last unreduced
    /// node, whether the move gives check, and the node count's phase.
    fn compact_reduction_input(
        &self,
        board: &Board,
        ply: usize,
        last_critical_ply: usize,
        input: rules::ReductionInput,
        term: rules::SpreadTerm,
    ) -> rules::ReductionInput {
        let thread = u64::try_from(self.td.thread_id).unwrap_or(0);
        rules::ReductionInput {
            since_critical: infra::to_i32(ply - last_critical_ply).min(8),
            laterality: self.td.stack[ply].laterality,
            gives_check: board.is_in_check(),
            spread: rules::spread(self.seeds(), term, self.td.nodes, thread),
            ..input
        }
    }

    /// The histories after a node found a best move: a bonus to it, maluses
    /// to the moves searched before it, a malus to the parent's quiet move
    /// when it came early, and a bonus to a quiet best move found only on a
    /// re-search.
    fn compact_best_move_updates(
        &mut self,
        board: &Board,
        ply: usize,
        depth: i32,
        cut_node: bool,
        best: Move,
        quiets: &SearchedMoves,
        noisies: &SearchedMoves,
        researched: bool,
    ) {
        let quiets = quiets.as_slice();
        let noisies = noisies.as_slice();
        probe!(self, Updates, true);
        let amounts = rules::update_amounts(
            self.seeds(),
            depth,
            cut_node,
            infra::to_i32(noisies.len()),
            infra::to_i32(quiets.len()),
        );
        let stm = board.side_to_move();
        let threats = self.td.stack[ply].threats;
        let best_piece = board.moving_piece(best);
        if is_quiet_move(best) {
            let pawn_row = HistoryTables::pawn_row(board.pawn_key());
            self.td
                .hist
                .update_quiet(threats, stm, best, amounts.quiet_bonus);
            self.td
                .hist
                .update_pawn(pawn_row, stm, best_piece, best.to_sq(), amounts.quiet_bonus);
            self.update_continuations(ply, stm, best_piece, best.to_sq(), amounts.cont_bonus);
            for (index, &mv) in quiets.iter().enumerate() {
                let fade = rules::malus_fade(self.seeds(), infra::to_i32(index));
                let piece = board.moving_piece(mv);
                let malus = -amounts.quiet_malus * fade / 1024;
                self.td.hist.update_quiet(threats, stm, mv, malus);
                self.td
                    .hist
                    .update_pawn(pawn_row, stm, piece, mv.to_sq(), malus);
                self.update_continuations(
                    ply,
                    stm,
                    piece,
                    mv.to_sq(),
                    -amounts.cont_malus * fade / 1024,
                );
            }
        } else {
            self.td.hist.update_noisy(
                threats,
                stm,
                best_piece,
                best.to_sq(),
                board.captured_piece(best),
                amounts.noisy_bonus,
            );
        }
        for &mv in noisies {
            self.td.hist.update_noisy(
                threats,
                stm,
                board.moving_piece(mv),
                mv.to_sq(),
                board.captured_piece(mv),
                -amounts.noisy_malus,
            );
        }
        if ply > 0 {
            let parent = *self.td.stack.back(ply, 1);
            if is_quiet_move(parent.mv) && parent.move_count < 2 {
                self.update_continuations(
                    ply - 1,
                    !stm,
                    parent.piece,
                    parent.mv.to_sq(),
                    -amounts.parent_malus,
                );
            }
        }
        if researched && is_quiet_move(best) {
            self.update_continuations(ply, stm, best_piece, best.to_sq(), amounts.research_bonus);
        }
    }

    /// A node that failed low refuted its parent's move: reward that move.
    fn compact_fail_low_reward(
        &mut self,
        board: &Board,
        ply: usize,
        depth: i32,
        best_score: i32,
        eval: i32,
        in_check: bool,
    ) {
        let parent = *self.td.stack.back(ply, 1);
        let mover = !board.side_to_move();
        if is_quiet_move(parent.mv) {
            probe!(self, FailLowReward, true);
            let reward = rules::fail_low_reward(
                self.seeds(),
                &rules::RewardInput {
                    parent_move_count: parent.move_count,
                    parent_tt: parent.mv == parent.tt_move,
                    in_check,
                    best_score,
                    eval,
                    parent_eval: parent.static_eval,
                    depth,
                },
            );
            self.td
                .hist
                .update_quiet(parent.threats, mover, parent.mv, reward.bonus);
            if let Some(context) = self.cont_context_back(ply, 2) {
                self.td.hist.update_cont(
                    context,
                    mover,
                    parent.piece,
                    parent.mv.to_sq(),
                    reward.grandparent,
                );
            }
        } else if !parent.mv.is_null() {
            probe!(self, FailLowReward, true);
            let piece = if parent.mv.is_promo() {
                parent.mv.promo_piece()
            } else {
                parent.piece
            };
            self.td.hist.update_noisy(
                parent.threats,
                mover,
                piece,
                parent.mv.to_sq(),
                parent.captured,
                rules::noisy_fail_low_reward(self.seeds(), depth),
            );
        }
    }
}

/// A line searches at most this many times its iteration's depth before
/// quiescence. In a won ending with a mate in view the donor's extensions and
/// re-searches can carry lines to the ply cap: a fifth of a fit's
/// perturbations turned one bench position from 43 thousand nodes into a
/// billion. Eight times the depth makes every such case finish and binds no
/// ordinary tree (the bench is unchanged); four times moved the bench by 2%
/// and lost a minor in the KBNK anchor.
const LINE_BOUND: i32 = 8;

impl Searcher {
    /// A node's depth held to its line's room under the line bound.
    #[inline(always)]
    fn compact_line_room(&self, depth: i32, ply: usize) -> i32 {
        depth.min(LINE_BOUND * self.td.root_depth.max(1) - infra::to_i32(ply))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::options::SearchOptions;

    /// One root search of `fen` from a fresh searcher.
    fn root_search(fen: &str, depth: i32, alpha: i32, beta: i32) -> (Searcher, Board, i32) {
        let mut searcher = Searcher::default();
        let mut board = Board::from_fen(fen).expect("a legal FEN");
        let score =
            searcher.search_root_window(&mut board, depth, alpha, beta, &mut || SearchEvent::None);
        (searcher, board, score)
    }

    /// A root window far above the position's value fails low and one far
    /// below it fails high, each returning a bound on its own side; a full
    /// window stores the root's exact result under the move it reports.
    #[test]
    fn a_root_window_fails_on_the_side_of_the_value_and_the_root_stores_its_result() {
        const DEPTH: i32 = 7;
        const FAR: i32 = 400;
        let mut checked = 0;
        for fen in crate::bench::BENCH_FENS.iter().step_by(3) {
            let (searcher, board, exact) = root_search(fen, DEPTH, -INF_SCORE, INF_SCORE);
            assert!(exact.abs() < INF_SCORE, "{fen}");
            let entry = searcher
                .shared
                .tt
                .probe(board.hash())
                .expect("the root is stored");
            assert_eq!(entry.bound(), Some(Bound::Exact), "{fen}");
            assert_eq!(entry.best_move(), Some(searcher.td.pv_table[0][0]), "{fen}");
            assert_eq!(
                i32::from(entry.score),
                exact,
                "{fen}: the stored score is the returned one"
            );
            if is_decisive(exact) {
                continue;
            }
            let (_, _, low) = root_search(fen, DEPTH, exact + FAR, exact + FAR + 1);
            assert!(
                low <= exact + FAR,
                "{fen}: {low} above a window over {exact}"
            );
            let (_, _, high) = root_search(fen, DEPTH, exact - FAR - 1, exact - FAR);
            assert!(
                high >= exact - FAR,
                "{fen}: {high} below a window under {exact}"
            );
            checked += 1;
        }
        assert!(checked >= 10, "{checked} positions");
    }

    /// The grandchild cutoff reset stays inside the stack at the deepest node
    /// that reaches it.
    #[test]
    fn cutoff_count_reset_is_bounded_at_the_ply_cap() {
        let mut searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/8/8/3q4/8/8/4KQ2 w - - 0 1").expect("valid FEN");
        for ply in [MAX_PLY - 3, MAX_PLY - 2] {
            let score = searcher.compact_negamax::<NonPv, _>(
                &mut board,
                -1,
                0,
                2,
                false,
                ply,
                0,
                Move::NULL,
                &mut || SearchEvent::None,
            );
            assert!(score.abs() < INF_SCORE, "ply {ply}");
        }
    }

    /// An exclusion search whose only legal move is the excluded one fails
    /// low, and with no mate, which its empty move list would otherwise
    /// score: the forced evasion `Kxg2` is singular.
    #[test]
    fn an_exclusion_search_with_only_the_excluded_move_returns_alpha() {
        let mut searcher = Searcher::default();
        let mut board = Board::from_fen("7k/8/8/8/8/8/6q1/7K w - - 0 1").expect("valid FEN");
        let forced = board.parse_move("h1g2").expect("legal capture");
        assert_eq!(
            board.generate_legal_moves().len(),
            1,
            "the capture is forced"
        );
        let singular_beta = -100;
        let score = searcher.compact_negamax::<NonPv, _>(
            &mut board,
            singular_beta - 1,
            singular_beta,
            3,
            true,
            1,
            1,
            forced,
            &mut || SearchEvent::None,
        );
        assert!(
            score < singular_beta,
            "{score}: the excluded move is singular"
        );
        assert!(
            score > -MATE_SCORE + infra::to_i32(MAX_PLY),
            "{score}: no mate is scored"
        );
    }

    #[test]
    fn a_line_keeps_eight_times_its_iteration_depth_before_quiescence() {
        let mut searcher = Searcher::default();
        searcher.td.root_depth = 10;
        assert_eq!(
            searcher.compact_line_room(5, 70),
            5,
            "room left: depth kept"
        );
        assert_eq!(searcher.compact_line_room(20, 75), 5, "held to the room");
        assert_eq!(
            searcher.compact_line_room(20, 85),
            -5,
            "past it: quiescence"
        );
        searcher.td.root_depth = 0;
        assert_eq!(
            searcher.compact_line_room(9, 4),
            4,
            "a first iteration counts one ply"
        );
    }

    fn compact_search(fen: &str, depth: u32) -> (Searcher, super::super::super::SearchResult) {
        let mut options = SearchOptions::default();
        options.limits.depth = Some(depth);
        let mut searcher = Searcher::default();
        let board = Board::from_fen(fen).expect("a legal FEN");
        let result = searcher.search(board, &options, false, || SearchEvent::None);
        (searcher, result)
    }

    #[test]
    fn the_compact_kernel_finds_a_mate_and_counts_each_move_made_once() {
        let (searcher, result) = compact_search("6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1", 4);
        assert_eq!(result.bestmove.to_string(), "d1d8");
        assert!(result.score >= MATE_SCORE - 10, "a mate: {}", result.score);
        assert_eq!(searcher.td.nodes, searcher.td.moves_made);
    }

    #[test]
    fn every_root_move_completes_each_iteration_when_the_window_fails_high() {
        // From the start the kernel's scores climb past a window opened round
        // the previous score of zero, so iterations fail high and search a
        // ply shallower; the root driver still completes each one.
        let (searcher, result) = compact_search(crate::board::STARTING_FEN, 3);
        assert_eq!(result.depth, 3);
        let records = &searcher.td.root_move_records;
        assert_eq!(records.len(), 20);
        assert!(
            records.iter().all(|record| record.samples == 3),
            "{:?}",
            records
                .iter()
                .map(|record| (record.mv.to_string(), record.samples))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn many_queens_stay_inside_the_kernels_range_and_score_in_their_band() {
        // (FEN, whether the side to move is the one with the queens).
        for (fen, ahead) in [
            ("k7/p7/8/8/8/8/8/QQQQQQ1K w - - 0 1", true),
            ("qqqqqq1k/8/8/8/8/8/P7/K7 w - - 0 1", false),
            ("3qqqqk/8/8/8/8/8/8/K1QQQQQ1 w - - 0 1", true),
            ("1k6/1p6/8/8/8/8/8/QQQQQQ1K b - - 0 1", false),
        ] {
            let board = Board::from_fen(fen).expect("a legal FEN");
            let mut searcher = Searcher::default();
            let raw = searcher.raw_eval(&board);
            let kernel = bounded_raw(raw);
            assert!(
                kernel.abs() < TB_WIN_SCORE,
                "{fen}: raw {raw}, kernel {kernel}"
            );
            assert_eq!(kernel > 0, ahead, "{fen}: raw {raw}");
            let (_, result) = compact_search(fen, 5);
            let score = result.score;
            assert_eq!(score > 0, ahead, "{fen}: {score}");
            assert!(
                score.abs() < TB_WIN_SCORE || score.abs() >= MATE_SCORE - infra::to_i32(MAX_PLY),
                "{fen}: {score} is neither an evaluation nor a mate"
            );
        }
    }

    #[test]
    fn the_compact_kernel_searches_the_bench_positions_to_a_legal_move() {
        for fen in crate::bench::BENCH_FENS.iter().step_by(5) {
            let (searcher, result) = compact_search(fen, 6);
            let board = Board::from_fen(fen).expect("a legal FEN");
            assert!(
                board.generate_legal_moves().contains(&result.bestmove),
                "{fen}: {} is not legal",
                result.bestmove
            );
            assert_eq!(result.depth, 6, "{fen}");
            assert_eq!(searcher.td.nodes, searcher.td.moves_made, "{fen}");
        }
    }
}
