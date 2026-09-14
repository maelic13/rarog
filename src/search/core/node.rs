//! The node kernels: `negamax`, `quiescence`, and the per-move reduction and
//! pruning helpers they share.

use crate::board::{Board, CheckInfo, Move, MoveList, Piece};
use crate::eval::{INF_SCORE, MATE_SCORE, VALUE_NONE, piece_value};
use crate::infra;
use crate::tt::{Bound, TtProbe, TtStore};

use super::history::{BestMoveUpdate, SearchedMoves, cont_context};
use super::movepick::{MovePicker, Stage, diversify_root_scores, is_noisy, pick_next};
use super::{MAX_PLY, MAX_QPLY, SearchEvent, Searcher, TB_WIN_SCORE};

/// Razoring stays away from decisive windows: alpha must be below this.
const RAZOR_ALPHA_LIMIT: i32 = 936;

// Float→int truncation IS the intended rounding of the LMR table formula
// (kept bit-exact with the pre-9.0b table), hence the scoped cast allow.
#[expect(clippy::cast_possible_truncation)]
pub(super) fn build_lmr_table(base: i32, div: i32) -> Box<[[i32; 64]; 64]> {
    let base_f = base as f64 / 1024.0;
    let div_f = div as f64 / 1024.0;
    let mut table = Box::new([[0i32; 64]; 64]);
    for (depth, row) in table.iter_mut().enumerate().skip(1) {
        for (searched, value) in row.iter_mut().enumerate().skip(1) {
            *value =
                (1024.0 * (base_f + (depth as f64).ln() * (searched as f64).ln() / div_f)) as i32;
        }
    }
    table
}

/// A `negamax` frame's node type, resolved at compile time (B.0 section 6.2).
///
/// Both donors monomorphise the node kernels this way; it is also what lets
/// B.4 give quiescence a PV concept without a new parameter. `cut_node` stays
/// a runtime argument, so an all-node is `!PV && !cut_node`.
pub(super) trait NodeType {
    /// On the principal variation: searched with an open window.
    const PV: bool;
    /// The root, which is ply 0 and nothing else.
    const ROOT: bool;
}

/// The root node: a PV node at ply 0.
pub(super) struct Root;
/// A PV node below the root.
struct Pv;
/// A null-window node.
struct NonPv;

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

/// The inputs of the late-move reduction for one move.
#[derive(Copy, Clone)]
struct LateMoveInputs {
    depth: i32,
    improvement: i32,
    corr_abs: i32,
    /// Alpha has been raised at this node.
    exact: bool,
    tt_score_below_alpha: bool,
    tt_score_above_alpha: bool,
    /// The stored result is shallower / at least as deep as this node.
    tt_shallow: bool,
    tt_deep: bool,
    win_beta: bool,
    is_quiet: bool,
    history: i32,
    /// Alpha minus the estimated score.
    alpha_gap: i32,
    /// Plies since the last node searched without a reduction.
    critical_distance: i32,
    pv: bool,
    window: i32,
    laterality: i32,
    tt_pv: bool,
    cut_node: bool,
    tt_move_null: bool,
    gives_check: bool,
    /// Beta cutoffs among this node's children so far.
    child_cutoffs: i32,
    parent_reduction: i32,
}

/// The depth a reduced move is searched at: the reduction in whole plies,
/// never below one ply of main search and never more than two plies above
/// `new_depth`; a PV node searches its reduced moves two plies deeper.
fn reduced_depth(new_depth: i32, reduction: i32, pv: bool) -> i32 {
    (new_depth - reduction / 1024).clamp(1, new_depth + 2) + 2 * i32::from(pv)
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

/// Per-move check test, memoized twice: `cache` holds the answer for THIS
/// move, `node_ci` holds the per-node masks shared by every move at the node
/// (10.3 — see [`Board::check_info`]).
fn move_gives_check(
    board: &Board,
    node_ci: &mut Option<CheckInfo>,
    mv: Move,
    cache: &mut Option<bool>,
) -> bool {
    match *cache {
        Some(gives_check) => gives_check,
        None => {
            let ci = node_ci.get_or_insert_with(|| board.check_info());
            let gives_check = board.gives_check_with(mv, ci);
            *cache = Some(gives_check);
            gives_check
        }
    }
}

impl Searcher {
    /// 4.4c: does the side to move hold enough non-pawn material to trust a
    /// null move?
    ///
    /// At the seeded `NmpMinNonPawnPieces = 1` this is exactly the historical
    /// `has_non_pawn_material` test — one piece suffices — so the default is
    /// inert. Higher values demand more before a pass is believed, because
    /// zugzwang risk concentrates where the mover has almost nothing left to
    /// move: with a single minor and pawns, "pass" and "move" can differ by the
    /// whole game.
    #[inline(always)]
    fn nmp_material_ok(&self, board: &Board) -> bool {
        let color = board.side_to_move();
        let non_pawn = board.pieces(color, Piece::Knight)
            | board.pieces(color, Piece::Bishop)
            | board.pieces(color, Piece::Rook)
            | board.pieces(color, Piece::Queen);
        // `count()` only when the threshold actually needs it; `any()` is the
        // cheap baseline path and keeps the seeded behaviour free.
        if self.cfg.params.nmp_min_non_pawn_pieces <= 1 {
            non_pawn.any()
        } else {
            infra::to_i32(non_pawn.count() as usize) >= self.cfg.params.nmp_min_non_pawn_pieces
        }
    }

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
        self.negamax::<Root, _>(
            board,
            depth,
            alpha,
            beta,
            0,
            true,
            Move::NULL,
            false,
            0,
            poll,
        )
    }

    /// Put the move about to be searched at `ply` on the stack with its
    /// continuation context: whether this node is in check, whether the move
    /// is noisy, and the coloured piece and destination.
    #[inline]
    fn push_move(&mut self, board: &Board, ply: usize, mv: Move, piece: Piece) {
        let captured = board.captured_piece(mv);
        let entry = &mut self.td.stack[ply];
        entry.mv = mv;
        entry.piece = piece;
        entry.captured = captured;
        entry.cont_key = cont_context(
            board.is_in_check(),
            is_noisy(mv),
            board.side_to_move(),
            piece,
            mv.to_sq(),
        );
    }

    /// The late-move reduction in 1024ths of a ply. Positive terms reduce
    /// more: a node that already has an exact score, a TT score at or below
    /// alpha or a shallow entry, a quiet (more with the eval below alpha), a
    /// cut node without a TT move, children that keep failing high. Negative
    /// terms reduce less: improvement, a large correction, good history, a
    /// short distance to the last unreduced node, a PV line (less with a
    /// narrow window), a move that gives check. The node count adds a small
    /// per-thread spread.
    #[inline(always)]
    fn late_move_reduction(&self, i: &LateMoveInputs) -> i32 {
        let p = &self.cfg.core;
        let mut r = p.lmr_log * i.depth.ilog2().cast_signed();
        r -= (p.lmr_improvement * i.improvement / 128).clamp(-241, 1_155);
        r -= p.lmr_correction * i.corr_abs / 1024;
        r += p.lmr_exact * i32::from(i.exact);
        r += p.lmr_tt_score_below_alpha * i32::from(i.tt_score_below_alpha);
        r += p.lmr_tt_shallow * i32::from(i.tt_shallow);
        r += 1024 * i32::from(i.win_beta);
        if i.is_quiet {
            r += p.lmr_quiet - p.lmr_quiet_history * i.history / 1024
                + p.lmr_alpha_gap * i.alpha_gap.clamp(-65, 91) / 128;
        } else {
            r += p.lmr_noisy - p.lmr_noisy_history * i.history / 1024;
        }
        r -= p.lmr_critical_ply * i.critical_distance.min(8);
        if i.pv {
            r -= p.lmr_pv + p.lmr_pv_window * i.window / self.td.root_delta.max(1);
        } else {
            r += p.lmr_laterality * i.laterality - p.lmr_non_pv;
        }
        if i.tt_pv {
            r -= p.lmr_tt_pv
                + p.lmr_tt_pv_score * i32::from(i.tt_score_above_alpha)
                + p.lmr_tt_pv_depth * i32::from(i.tt_deep);
        } else if i.cut_node {
            r += p.lmr_cut_node + p.lmr_cut_node_no_tt_move * i32::from(i.tt_move_null);
        }
        if i.gives_check {
            r -= p.lmr_gives_check;
        }
        if i.child_cutoffs > 2 {
            r += p.lmr_child_cutoffs
                + p.lmr_child_cutoffs_all_node * i32::from(!i.pv && !i.cut_node);
        }
        if !i.pv && i.parent_reduction > r + 414 {
            r += p.lmr_parent;
        }
        let thread = u64::try_from(self.td.thread_id).unwrap_or(0);
        let spread = i32::try_from((self.td.nodes + thread * 27) % 128).unwrap_or(0);
        r + spread - 59
    }

    /// Record the order index of the move about to be searched at `ply` and
    /// the line's accumulated lateness: the parent's laterality plus
    /// [`laterality_step`] of this move's index.
    #[inline]
    fn record_move_order(&mut self, ply: usize, move_count: i32) {
        let parent = self.td.stack.back(ply, 1).laterality;
        let entry = &mut self.td.stack[ply];
        entry.move_count = move_count;
        entry.laterality = parent + laterality_step(move_count);
    }

    pub(super) fn negamax<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        mut depth: i32,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        allow_null: bool,
        excluded: Move,
        cut_node: bool,
        last_critical_ply: usize,
        poll: &mut P,
    ) -> i32 {
        debug_assert_eq!(NODE::ROOT, ply == 0, "the root node type is exactly ply 0");
        if self.check_stop(poll) {
            return 0;
        }
        if ply >= MAX_PLY - 1 {
            return self.corrected_eval(board, ply);
        }
        self.td.pv_len[ply] = ply;
        self.td.seldepth = self.td.seldepth.max(ply);

        if !NODE::ROOT && board.can_declare_draw_in_search() {
            return 0;
        }

        let in_check = board.is_in_check();

        let mate_alpha = -MATE_SCORE + infra::to_i32(ply);
        let mate_beta = MATE_SCORE - infra::to_i32(ply) - 1;
        alpha = alpha.max(mate_alpha);
        let beta = beta.min(mate_beta);
        if alpha >= beta {
            return alpha;
        }

        if depth <= 0 {
            return if NODE::PV {
                self.quiescence::<Pv, _>(board, alpha, beta, ply, 0, poll)
            } else {
                self.quiescence::<NonPv, _>(board, alpha, beta, ply, 0, poll)
            };
        }

        // 4.2: counted HERE, after the depth<=0 hand-off, so `nodes` means
        // interior nodes actually searched — which is the oracle's population.
        // Counting at function entry inflated it by every node that
        // immediately became a qnode, and that same node was then counted a
        // second time as `qnodes`. That double count silently deflated every
        // rate taken against `nodes`.
        crate::diag_count!(nodes);
        #[cfg(feature = "diag")]
        if in_check {
            crate::diag_count!(nodes_in_check);
        }

        let original_alpha = alpha;
        let hash = board.hash();
        #[cfg(feature = "diag")]
        let diag_sample = crate::diag::sampled(hash, ply, crate::diag::SAMPLE_MAIN);
        #[cfg(feature = "diag")]
        if diag_sample {
            crate::diag_count!(sampled_main_nodes);
        }
        if let Some(score) = self.syzygy_wdl_score(board, depth, ply, excluded) {
            self.shared.tt.store(TtStore {
                key: hash,
                depth,
                score,
                bound: Bound::Exact,
                mv: Move::NULL,
                ply,
                static_eval: VALUE_NONE,
                is_pv: NODE::PV,
            });
            return score;
        }
        let tt_entry = self.shared.tt.probe(hash);
        // 9.7.5(b): main thread only. If helper work is reaching the thread
        // that owns the answer, this hit rate must RISE with thread count; a
        // flat rate means the helpers are filling a table nobody reads.
        if self.td.thread_id == 0 {
            crate::diag_count!(main_tt_probes);
            if tt_entry.is_some() {
                crate::diag_count!(main_tt_hits);
            }
        }
        // 4.2: one decode of the probe for the whole node. Mate distance and
        // rule-50 are resolved exactly once here — the pre-4.2 code decoded the
        // same entry twice, at `tt_score` and again inside the cutoff block.
        let ev = TtProbe::from_entry(tt_entry, ply, board.halfmove_clock());
        let mut tt_pv = ev.pv_line(NODE::PV);
        #[cfg(feature = "diag")]
        if diag_sample {
            if ev.hit {
                crate::diag_count!(tt_sample_hit);
                if !NODE::PV && excluded.is_null() && ev.depth >= depth {
                    match ev.bound {
                        Some(Bound::Exact) => {
                            crate::diag_count!(tt_cut_exact);
                        }
                        Some(Bound::Lower) if ev.score >= beta => {
                            crate::diag_count!(tt_cut_lower);
                        }
                        Some(Bound::Upper) if ev.score <= alpha => {
                            crate::diag_count!(tt_cut_upper);
                        }
                        Some(_) => {
                            crate::diag_count!(tt_bound_not_usable);
                        }
                        None => {}
                    }
                    if ev.contradicts_window(alpha, beta) {
                        crate::diag_count!(tt_bound_contradicts_window);
                    }
                } else if ev.bound.is_some() {
                    // 4.9b: this `else` used to add to `tt_bound_not_usable`
                    // as well, which made one counter answer three unrelated
                    // questions — and the one 4.9 needs is the third. A PV node
                    // and an excluded-move search can never cut whatever the
                    // entry says, and neither population moves with thread
                    // count; a SHALLOW entry is the only cause the "helpers add
                    // entries that cannot cut" hypothesis predicts should grow.
                    // Lumped together they cannot be told apart.
                    //
                    // PV is attributed before depth on purpose: at a PV node the
                    // entry is refused regardless of how deep it is, so PV is
                    // the binding reason even when the entry is also shallow.
                    if NODE::PV {
                        crate::diag_count!(tt_reject_pv);
                    } else if !excluded.is_null() {
                        crate::diag_count!(tt_reject_excluded);
                    } else {
                        crate::diag_count!(tt_reject_shallow);
                        // How far short, so "marginally too shallow" and
                        // "hopelessly too shallow" are distinguishable: the
                        // first is a replacement-policy question, the second is
                        // not worth chasing at all.
                        crate::diag_add!(
                            tt_reject_shallow_deficit,
                            u64::try_from(depth - ev.depth).unwrap_or(0)
                        );
                    }
                }
            } else {
                crate::diag_count!(tt_sample_miss);
            }
        }
        // Near the fifty-move horizon a stored result may no longer hold, so
        // the node searches instead of trusting it.
        if !NODE::PV
            && excluded.is_null()
            && let Some(score) = ev.node_cutoff_score(depth, alpha, beta, cut_node)
        {
            // A quiet TT move that cuts again, reached through one of the
            // parent's first few moves, is confirmed as a refutation.
            if score >= beta
                && self.td.stack.back(ply, 1).move_count < 4
                && let Some(mv) = ev.mv.and_then(|mv| board.legal_move(mv))
                && !is_noisy(mv)
            {
                crate::diag_count!(tt_cutoff_quiet_bonus);
                let stm = board.side_to_move();
                let piece = board.moving_piece(mv);
                let quiet_bonus = (190 * depth - 81).min(self.cfg.core.hist_tt_cutoff_bonus_cap);
                let cont_bonus = (96 * depth - 73).min(1_206);
                self.td
                    .hist
                    .update_quiet(board.threats().all, stm, mv, quiet_bonus);
                self.update_continuations(ply, stm, piece, mv.to_sq(), cont_bonus);
            }
            if board.halfmove_clock() < 90 {
                trace_decision!(
                    self,
                    ply,
                    "tt_cut depth {depth} tt_depth {} bound {:?} score {score} window {alpha} {beta}",
                    ev.depth,
                    ev.bound
                );
                return score;
            }
        }
        let mut tt_move = ev
            .mv
            .and_then(|mv| board.legal_move(mv))
            .unwrap_or(Move::NULL);
        if NODE::ROOT && !self.td.root_moves.is_empty() && !self.td.root_moves.contains(&tt_move) {
            tt_move = Move::NULL;
        }

        // IIR: reduce depth when we lack a good TT entry to guide move ordering
        if !self.ablated(4)
            && excluded.is_null()
            && depth >= self.cfg.core.iir_min_depth
            && (tt_move.is_null() || (!NODE::PV && ev.too_shallow_to_order(depth)))
        {
            #[cfg(feature = "diag")]
            if diag_sample {
                crate::diag_count!(iir_applied);
                if NODE::PV {
                    crate::diag_count!(iir_pv);
                }
                if tt_move.is_null() {
                    crate::diag_count!(iir_no_tt_move);
                } else {
                    crate::diag_count!(iir_shallow_tt);
                }
            }
            trace_decision!(
                self,
                ply,
                "iir depth {depth} tt_move {tt_move} tt_depth {}",
                ev.depth
            );
            depth -= 1;
        }

        // 4.2: the pre-4.2 form spelled out three branches whose two `else`
        // arms were identical, because a probe MISS and a hit carrying no
        // stored eval both fall back to a fresh raw eval. `TtProbe::MISS`
        // already reports `VALUE_NONE`, so one test covers both.
        let (static_eval, raw_static_eval, correction) = if in_check {
            (VALUE_NONE, VALUE_NONE, 0)
        } else {
            let raw = if ev.raw_static_eval == VALUE_NONE {
                let raw = self.raw_eval(board);
                // A miss stores the eval at once, so the next visit to this
                // position skips the evaluation even if this search is cut.
                if tt_entry.is_none() && excluded.is_null() {
                    self.shared.tt.store_eval(hash, raw, tt_pv);
                }
                raw
            } else {
                ev.raw_static_eval
            };
            let (eval, correction) = self.corrected_eval_parts(board, raw, ply);
            (eval, raw, correction)
        };
        self.td.stack[ply].static_eval = static_eval;
        self.td.stack[ply].tt_pv = tt_pv;
        self.td.stack[ply].tt_move = tt_move;
        let threats = board.threats();
        self.td.stack[ply].threats = threats.all;
        // The eval swing across the parent's quiet move trains that move's
        // history: a move after which the side to move stands worse than the
        // parent stood was a good one for the parent.
        if !NODE::ROOT && !in_check && excluded.is_null() && (depth < 6 || tt_entry.is_none()) {
            let parent = *self.td.stack.back(ply, 1);
            if !parent.mv.is_null() && !is_noisy(parent.mv) && parent.static_eval != VALUE_NONE {
                let bonus = (812 * -(static_eval + parent.static_eval) / 128).clamp(-144, 324);
                self.td
                    .hist
                    .update_quiet(parent.threats, !board.side_to_move(), parent.mv, bonus);
            }
        }
        self.td.stack[ply].reduction = 0;
        self.td.stack[ply].move_count = 0;
        if ply + 2 < MAX_PLY {
            self.td.stack[ply + 2].cutoff_count = 0;
        }
        // How much the tables moved this node's eval: a large correction marks
        // an eval the search has repeatedly found wrong, so margins widen and
        // reductions shrink with it. Zero in check.
        let corr_abs = correction.abs();
        // Improvement: this eval against the side to move's eval two plies
        // back, or four plies back when that node was in check; zero when
        // neither exists. Its sign is `improving`; its size moves margins.
        let improvement = if in_check {
            0
        } else {
            let two_back = self.td.stack.back(ply, 2).static_eval;
            let four_back = self.td.stack.back(ply, 4).static_eval;
            if two_back != VALUE_NONE {
                static_eval - two_back
            } else if four_back != VALUE_NONE {
                static_eval - four_back
            } else {
                0
            }
        };
        let improving = improvement > 0;
        let improving_i = i32::from(improving);
        // The estimated score: the corrected eval, replaced by a stored bound
        // that tightens it.
        let eval_for_pruning = if in_check {
            static_eval
        } else {
            ev.refine_eval(static_eval, 0)
        };
        #[cfg(feature = "diag")]
        if diag_sample
            && eval_for_pruning != VALUE_NONE
            && static_eval != VALUE_NONE
            && eval_for_pruning != static_eval
        {
            crate::diag_count!(tt_eval_refined);
            let delta = u64::from(eval_for_pruning.saturating_sub(static_eval).unsigned_abs());
            crate::diag_add!(tt_eval_delta_sum, delta);
        }
        // A non-PV, non-check node where the stored PV bit alone vetoes the
        // null-move and ProbCut searches below.
        if tt_pv && !NODE::PV && !in_check && excluded.is_null() {
            crate::diag_count!(tt_pv_veto);
        }

        // Hindsight: the parent reduced the move that led here. If both evals
        // say the position got worse for the parent's opponent after a large
        // reduction, search one ply deeper; if both say it got better for the
        // side that was reduced against, one ply shallower.
        if !self.ablated(4) && !NODE::ROOT && !in_check && excluded.is_null() {
            let parent = *self.td.stack.back(ply, 1);
            if parent.static_eval != VALUE_NONE {
                let eval_delta = static_eval + parent.static_eval;
                if parent.reduction >= self.cfg.core.hindsight_deepen_reduction && eval_delta < 0 {
                    crate::diag_count!(hindsight_up);
                    trace_decision!(
                        self,
                        ply,
                        "hindsight_up depth {depth} parent_reduction {} eval_delta {eval_delta}",
                        parent.reduction
                    );
                    depth += 1;
                }
                if !tt_pv
                    && depth >= 2
                    && parent.reduction > 0
                    && eval_delta > self.cfg.core.hindsight_reduce_margin
                {
                    crate::diag_count!(hindsight_down);
                    trace_decision!(
                        self,
                        ply,
                        "hindsight_down depth {depth} parent_reduction {} eval_delta {eval_delta}",
                        parent.reduction
                    );
                    depth -= 1;
                }
            }
        }

        // Razoring: far enough below alpha that only a tactic can help, so
        // the node asks quiescence. Not on a PV line, not when alpha is
        // already a decisive-looking score, not when a quiet TT move or a
        // fail-high entry says there is more here.
        if !self.ablated(0)
            && !NODE::PV
            && !in_check
            && eval_for_pruning
                < alpha - self.cfg.core.razor_base - self.cfg.core.razor_square * depth * depth
            && alpha < RAZOR_ALPHA_LIMIT
            && (tt_move.is_null() || is_noisy(tt_move))
            && ev.bound != Some(Bound::Lower)
        {
            crate::diag_count!(razor_drop);
            trace_decision!(
                self,
                ply,
                "razor depth {depth} estimated {eval_for_pruning} margin {} alpha {alpha}",
                self.cfg.core.razor_base + self.cfg.core.razor_square * depth * depth
            );
            return self.quiescence::<NonPv, _>(board, alpha, beta, ply, 0, poll);
        }

        // Reverse futility: far enough above beta that the node is expected
        // to hold. The margin grows with the square of the depth, shrinks as
        // the position improves, widens with the correction's size (an eval
        // the tables keep moving is less trusted) and relaxes when no piece
        // of ours stands attacked. Returns a score pulled toward beta.
        if !self.ablated(1) && !tt_pv && !in_check && excluded.is_null() {
            let core = &self.cfg.core;
            let unthreatened = (threats.all & board.color_occ(board.side_to_move())).is_empty();
            let margin = (core.rfp_square * depth * depth / 16 + core.rfp_linear * depth
                - core.rfp_improvement * improvement / 1024
                + core.rfp_correction * corr_abs / 1024
                - core.rfp_threat * i32::from(unthreatened)
                + core.rfp_constant)
                .max(2);
            if eval_for_pruning >= beta + margin
                && beta > -TB_WIN_SCORE
                && eval_for_pruning < TB_WIN_SCORE
            {
                crate::diag_count!(rfp_cut);
                #[cfg(feature = "diag")]
                match depth {
                    ..=3 => crate::diag_count!(rfp_cut_d1_3),
                    4..=7 => crate::diag_count!(rfp_cut_d4_7),
                    _ => crate::diag_count!(rfp_cut_d8_plus),
                }
                let score = eval_for_pruning + (beta - eval_for_pruning) * core.rfp_lerp / 1024;
                trace_decision!(
                    self,
                    ply,
                    "rfp depth {depth} estimated {eval_for_pruning} static {static_eval} \
                     improvement {improvement} corr {corr_abs} unthreatened {unthreatened} \
                     margin {margin} beta {beta} returns {score}"
                );
                return score;
            }
        }

        if !tt_pv && !in_check && excluded.is_null() {
            if !self.ablated(2)
                && allow_null
                && depth >= 3
                && eval_for_pruning
                    >= beta
                        - self.cfg.params.nm_depth_coeff * depth
                        - self.cfg.params.nm_improving_bonus * improving_i
                && self.nmp_material_ok(board)
            {
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(nmp_attempt);
                }
                let reduction = 4 + depth / 4 + ((eval_for_pruning - beta) / 200).clamp(0, 3);
                self.td.stack[ply].laterality = 0;
                board.make_null_move();
                self.shared.tt.prefetch(board.hash());
                let score = -self.negamax::<NonPv, _>(
                    board,
                    depth - reduction,
                    -beta,
                    -beta + 1,
                    ply + 1,
                    false,
                    Move::NULL,
                    true,
                    ply + 1,
                    poll,
                );
                board.unmake_null_move();
                if self.td.stopped || self.td.quit {
                    return 0;
                }
                if score >= beta {
                    crate::diag_count!(nmp_cut);
                    // 4.10a CORRECTNESS REPAIR. A null-move cutoff
                    // establishes only "at least beta"; when the reduced null
                    // search comes back in mate range, Rarog returns that mate
                    // score, claiming a forced mate no real line demonstrated.
                    // It then travels through the TT as a Lower bound.
                    // Stockfish clamps this to beta
                    // (`if (nullValue >= VALUE_MATE_IN_MAX_PLY) nullValue = beta;`).
                    //
                    // Measured: 11 such returns at `bench 13`, 195 at `bench 18`.
                    // The removed `NmpDecisiveGuard` did NOT cover it — its
                    // predicate was on the WINDOW, and its population at depth 13
                    // is ZERO while these 11 still occurred.
                    // CLAMPED: a fail-high asserts only "at least beta", so
                    // that is what is returned. The cutoff is preserved exactly
                    // — the value is still >= beta — while the unproven mate is
                    // refused. Measured cost: bench 13 6,502,902 -> 6,519,711
                    // (+0.26%), bench 18 +22.9%, because a weaker fail-high
                    // cuts less higher up in mate-heavy subtrees. That cost is
                    // what the registered non-inferiority gate is deciding.
                    let score = if score >= MATE_SCORE - infra::to_i32(MAX_PLY) {
                        crate::diag_count!(nmp_cut_unproven_mate);
                        beta
                    } else {
                        score
                    };
                    trace_decision!(
                        self,
                        ply,
                        "nmp_cut depth {depth} reduction {reduction} eval {eval_for_pruning} \
                         score {score} beta {beta}"
                    );
                    #[cfg(feature = "diag")]
                    if diag_sample {
                        crate::diag_count!(nmp_sample_cut);
                    }
                    if depth >= 10 {
                        crate::diag_count!(nmp_verify_attempt);
                        let verify_depth = (depth - reduction).max(1);
                        let verified = self.negamax::<NonPv, _>(
                            board,
                            verify_depth,
                            beta - 1,
                            beta,
                            ply,
                            false,
                            Move::NULL,
                            false,
                            last_critical_ply,
                            poll,
                        );
                        if self.td.stopped || self.td.quit {
                            return 0;
                        }
                        trace_decision!(
                            self,
                            ply,
                            "nmp_verify depth {verify_depth} score {verified} beta {beta}"
                        );
                        if verified < beta {
                            crate::diag_count!(nmp_verify_fail);
                            // Continue normally when the null cutoff is not stable
                            // under a verification search with null move disabled.
                        } else {
                            crate::diag_count!(nmp_verify_pass);
                            return score;
                        }
                    } else {
                        return score;
                    }
                }
            }

            if !self.ablated(3) && depth >= 4 {
                // Per NODE entering the block, before capture generation, so
                // nodes with no eligible capture are counted here too. This
                // carried the `probcut_attempt` name until 4.7c prep, and was
                // differenced against the oracle's per-MOVE counter of the same
                // name -- the RAR-S25 denominator shape. See the RAR-S55
                // correction.
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(probcut_nodes);
                }
                let probcut_beta = beta + self.cfg.params.probcut_margin;
                // 4.7c PROBCUT MOVE FILTER. The entry contract for the
                // speculative capture search moves from "this capture does not
                // lose material" to "this capture can plausibly bridge the gap
                // to probcut_beta".
                //
                // RAR-S55 v3 measured what the old contract costs. Per node the
                // two engines convert alike -- 22.7% against the reference's
                // 25.2% -- so the yield was never the divergence. The PRICE was:
                // Rarog searched 5.17x the normalised ProbCut moves and
                // converted 32.6% of them against 71.9%. Two in three of its
                // ProbCut move-searches produced nothing.
                //
                // `see_ge(mv, 0)` admits any capture that is not outright
                // losing, which is unrelated to the question this search asks.
                // The gap `probcut_beta - static_eval` IS that question in
                // material terms, and it is floored at 0 so the filter can only
                // tighten the old contract, never loosen it -- a negative
                // threshold would admit losing captures at nodes already above
                // probcut_beta, which is de-selectivity nothing here motivates.
                //
                // `static_eval` is real: the whole block is under `!in_check`.
                // i32 throughout: the gap is bounded by the mate range, so
                // gap * 100 cannot approach i32's limit.
                let see_threshold =
                    ((probcut_beta - static_eval) * self.cfg.params.probcut_see_gap_scale / 100)
                        .max(0);
                // The flat cap of 8 had no stated derivation. Scale it by the
                // node's own prediction instead: a cut node is where a fail-high
                // is expected and the speculative search is likeliest to pay.
                let move_cap = self.cfg.params.probcut_move_cap_base
                    + if cut_node {
                        self.cfg.params.probcut_move_cap_cut_bonus
                    } else {
                        0
                    };
                let mut captures = MoveList::new();
                board.generate_legal_captures_into(&mut captures);
                let mut scored =
                    self.score_tactical_moves(board, &threats, captures.as_slice(), tt_move);
                let mut searched_here = 0i32;
                for index in 0..scored.len() {
                    if searched_here >= move_cap {
                        break;
                    }
                    let picked = pick_next(scored.as_mut_slice(), index);
                    let mv = picked.mv;
                    if !board.see_ge(mv, see_threshold) {
                        continue;
                    }
                    searched_here += 1;
                    // Per MOVE: a ProbCut search is about to start. This is the
                    // counter the oracle's `probcut_attempt` can be differenced
                    // against -- placed after the eligibility filter and before
                    // the qsearch, exactly where the oracle places its own.
                    // Up to 8 of these can fire at a single node.
                    #[cfg(feature = "diag")]
                    if diag_sample {
                        crate::diag_count!(probcut_attempt);
                    }
                    let probcut_piece = board.moving_piece(mv);
                    // ProbCut's child is a verification search, not a
                    // reduced sibling, so it consumes neither selectivity
                    // input. Written explicitly rather than left stale.
                    self.push_move(board, ply, mv, probcut_piece);
                    self.record_move_order(ply, 0);
                    board.make_move(mv);
                    self.shared.tt.prefetch(board.hash());
                    let score = -self.quiescence::<NonPv, _>(
                        board,
                        -probcut_beta,
                        -probcut_beta + 1,
                        ply + 1,
                        0,
                        poll,
                    );
                    let score = if score >= probcut_beta {
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            crate::diag_count!(probcut_qpass);
                        }
                        -self.negamax::<NonPv, _>(
                            board,
                            depth - 4,
                            -probcut_beta,
                            -probcut_beta + 1,
                            ply + 1,
                            false,
                            Move::NULL,
                            true,
                            last_critical_ply,
                            poll,
                        )
                    } else {
                        score
                    };
                    board.unmake_move(mv);
                    self.clear_move(ply);
                    if self.td.stopped || self.td.quit {
                        return 0;
                    }
                    if score >= probcut_beta {
                        // Deliberately EXACT, like every other `*_cut` counter
                        // (`rfp_cut`, `nmp_cut`, `see_prune`). The core set is
                        // guarded inconsistently on purpose: the spec's chosen
                        // resolution is `RAROG_DIAG_SAMPLE_STRIDE=1`, which
                        // makes the sampled half exact in one place rather than
                        // lifting counters out of guards in the hottest file.
                        // Never read this against `probcut_attempt` at the
                        // default stride.
                        crate::diag_count!(probcut_cut);
                        let cutoff_score = score - (probcut_beta - beta);
                        trace_decision!(
                            self,
                            ply,
                            "probcut depth {depth} move {mv} score {score} probcut_beta {probcut_beta} \
                             static {static_eval} returns {cutoff_score}"
                        );
                        self.shared.tt.store(TtStore {
                            key: hash,
                            depth: depth - 3,
                            // The margin-shifted value, not the actual
                            // fail-high: storing that costs +5.55% time-to-depth
                            // on its own (RAR-S34).
                            score: cutoff_score,
                            bound: Bound::Lower,
                            mv,
                            ply,
                            static_eval: raw_static_eval,
                            is_pv: false,
                        });
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            crate::diag_count!(probcut_tt_store);
                        }
                        return cutoff_score;
                    }
                }
            }
        }

        let mut move_picker = if NODE::ROOT {
            let mut legal_moves = MoveList::new();
            board.generate_legal_movelist_into(&mut legal_moves);
            if legal_moves.is_empty() {
                return if in_check {
                    -MATE_SCORE + infra::to_i32(ply)
                } else {
                    0
                };
            }

            let root_moves;
            let legal_moves = if NODE::ROOT && !self.td.root_moves.is_empty() {
                root_moves = legal_moves
                    .iter()
                    .copied()
                    .filter(|mv| self.td.root_moves.contains(mv))
                    .collect::<Vec<_>>();
                if root_moves.is_empty() {
                    legal_moves.as_slice()
                } else {
                    root_moves.as_slice()
                }
            } else {
                legal_moves.as_slice()
            };

            let mut scored = self.score_moves(board, &threats, legal_moves, tt_move, ply);
            // 8.13: order the root list from the POOL's view. A move another
            // thread has already proven good at a deeper depth is tried first
            // here too, so threads stop re-deriving each other's refutations.
            // Applied BEFORE the rotation below, which diversifies on top.
            if NODE::ROOT && scored.len() > 1 {
                // No-op serially: with no shared state there are no pool
                // scores to fold in.
                self.apply_shared_root_scores(legal_moves, &mut scored);
            }
            // Helpers rotate their root list on top of the pool ordering, so
            // the pool's shared view refines the ordering without collapsing
            // every thread onto the same tree.
            let rotate = self.td.root_move_offset > 0;
            if NODE::ROOT && rotate && scored.len() > 1 {
                let offset = self.td.root_move_offset % scored.len();
                diversify_root_scores(scored.as_mut_slice(), offset);
            }
            MovePicker::full(scored, tt_move)
        } else {
            MovePicker::staged(self, board, &threats, tt_move, ply)
        };
        let mut best_move = Move::NULL;
        let mut best_score = -INF_SCORE;
        let mut searched = 0usize;
        // Every move the picker hands over, pruned ones included.
        let mut move_count = 0i32;
        #[cfg(feature = "diag")]
        let diag_order_sample = diag_sample && excluded.is_null();
        let mut legal_move_seen = false;
        // 10.3: per-node check masks, built at most once and reused by every
        // move at this node — for the pruning-side `move_gives_check` calls
        // and for the `make_move` check hint below. `board` is restored by
        // `unmake_move` each iteration, so these stay valid for the whole loop.
        let mut node_ci: Option<CheckInfo> = None;
        let mut quiet_moves = SearchedMoves::new();
        let mut noisy_moves = SearchedMoves::new();
        // Set by late-move or quiet-futility pruning: the picker hands over no
        // more quiets.
        let mut skip_quiets = false;
        while let Some(picked) = move_picker.next(self, board, &threats, skip_quiets) {
            let mv = picked.mv;
            if mv == excluded {
                continue;
            }
            legal_move_seen = true;
            move_count += 1;
            let is_capture = mv.is_capture();
            let is_quiet = board.is_quiet_move(mv);
            let mut search_count = 0u32;
            let moving_piece = board.moving_piece(mv);
            let captured_piece = board.captured_piece(mv);
            // The TT move is emitted before quiets are scored, so its history
            // is read here.
            let quiet_hist = if !is_quiet {
                0
            } else if mv == tt_move {
                self.quiet_pruning_history(board, threats.all, ply, mv)
            } else {
                picked.quiet_history
            };
            let mut gives_check = None;
            // The picker stage the move came from, classified at pick time: `see`
            // is refined later for some moves, so it must be read here.
            #[cfg(feature = "diag")]
            if diag_order_sample {
                if mv == tt_move {
                    crate::diag_count!(move_seen_tt);
                } else if is_capture && picked.see >= 0 {
                    crate::diag_count!(move_seen_good_capture);
                } else if is_quiet {
                    crate::diag_count!(move_seen_quiet);
                } else {
                    crate::diag_count!(move_seen_bad_capture);
                }
            }

            // Move-loop pruning. Never at the root, never in check, and only
            // once a move has produced a non-losing score, so the first move
            // and every move while all scores are mated are searched.
            if !NODE::ROOT && !in_check && best_score > -TB_WIN_SCORE && !self.ablated(5) {
                let core = &self.cfg.core;
                let is_direct_check = node_ci
                    .get_or_insert_with(|| board.check_info())
                    .direct_check_squares(moving_piece)
                    .contains(mv.to_sq());
                let history = if is_quiet {
                    quiet_hist
                } else {
                    self.td.hist.noisy(
                        threats.all,
                        board.side_to_move(),
                        moving_piece,
                        mv.to_sq(),
                        captured_piece,
                    )
                };
                crate::diag_count!(prune_shadow_moves);

                // Late-move pruning: past a move count that grows with the
                // square of the depth, the remaining quiets are skipped. A
                // quiet that gives direct check survives the skip.
                if is_quiet
                    && !is_direct_check
                    && beta < TB_WIN_SCORE
                    && move_count
                        >= (core.lmp_base
                            + core.lmp_improvement * improvement / 16
                            + core.lmp_square * depth * depth
                            + core.lmp_history * history / 1024)
                            / 1024
                {
                    crate::diag_count!(lmp_prune);
                    #[cfg(feature = "diag")]
                    if !skip_quiets {
                        crate::diag_count!(lmp_nodes);
                        crate::diag_count!(skip_quiets_nodes);
                    }
                    trace_decision!(
                        self,
                        ply,
                        "lmp depth {depth} move {mv} move_count {move_count} history {history} \
                         improvement {improvement} alpha {alpha}"
                    );
                    skip_quiets = true;
                    continue;
                }

                // Quiet futility: a quiet that cannot lift the eval to alpha
                // even with a depth- and history-scaled margin. The node's
                // best score rises to the margin (fail-soft) and the rest of
                // the quiets are skipped.
                let futility_value = static_eval
                    + core.fp_base
                    + core.fp_linear * depth
                    + core.fp_history * history / 1024
                    + core.fp_eval_above_beta * i32::from(static_eval >= beta)
                    + core.fp_correction * corr_abs / 1024;
                if is_quiet && !is_direct_check && depth < 14 && futility_value <= alpha {
                    crate::diag_count!(quiet_futility_prune);
                    #[cfg(feature = "diag")]
                    if !skip_quiets {
                        crate::diag_count!(skip_quiets_nodes);
                    }
                    trace_decision!(
                        self,
                        ply,
                        "futility depth {depth} move {mv} move_count {move_count} eval {static_eval} \
                         history {history} corr {corr_abs} value {futility_value} alpha {alpha}"
                    );
                    if best_score < futility_value && best_score.abs() < TB_WIN_SCORE {
                        best_score = futility_value;
                    }
                    skip_quiets = true;
                    continue;
                }

                // Bad-noisy futility: once the picker is down to the losing
                // noisy moves, one that cannot reach alpha with its victim
                // and a margin ends the move loop.
                let noisy_futility_value = static_eval
                    + core.bnfp_base
                    + core.bnfp_linear * depth
                    + core.bnfp_history * history / 1024
                    + captured_piece.map_or(0, piece_value);
                if !is_direct_check
                    && depth < 11
                    && move_picker.stage() == Stage::BadNoisy
                    && noisy_futility_value <= alpha
                {
                    crate::diag_count!(bad_noisy_futility);
                    trace_decision!(
                        self,
                        ply,
                        "bad_noisy_futility depth {depth} move {mv} move_count {move_count} \
                         eval {static_eval} history {history} value {noisy_futility_value} alpha {alpha}"
                    );
                    if best_score < noisy_futility_value && best_score.abs() < TB_WIN_SCORE {
                        best_score = noisy_futility_value;
                    }
                    break;
                }

                // History pruning: a quiet whose history is far below zero at
                // shallow depth.
                if is_quiet && depth < 5 && history < -core.hp_slope * depth {
                    crate::diag_count!(history_pruned);
                    trace_decision!(
                        self,
                        ply,
                        "history_prune depth {depth} move {mv} move_count {move_count} history {history}"
                    );
                    continue;
                }

                // SEE pruning, both move classes: the exchange must not lose
                // more than a depth- and history-scaled allowance.
                let threshold = if is_quiet {
                    (-core.see_quiet_square * depth * depth + core.see_quiet_linear * depth
                        - core.see_quiet_history * history / 1024
                        + core.see_quiet_constant)
                        .min(0)
                } else {
                    (-core.see_noisy_square * depth * depth
                        - core.see_noisy_linear * depth
                        - core.see_noisy_history * history / 1024
                        + core.see_noisy_constant)
                        .min(0)
                };
                if !board.see_ge_quiet_aware(mv, threshold) {
                    if is_quiet {
                        crate::diag_count!(quiet_see_prune);
                    } else {
                        crate::diag_count!(see_prune);
                    }
                    trace_decision!(
                        self,
                        ply,
                        "see_prune depth {depth} move {mv} move_count {move_count} quiet {is_quiet} \
                         history {history} threshold {threshold}"
                    );
                    continue;
                }
            }

            let child_is_pv = NODE::PV && searched == 0;
            let mut extension = 0;
            let singular_move_candidate = !self.ablated(6)
                && !NODE::ROOT
                && mv == tt_move
                && excluded.is_null()
                && depth >= 4;
            if singular_move_candidate
                && ev.allows_singular(depth, self.cfg.params.singular_tt_depth_margin)
            {
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(singular_attempt);
                }
                let singular_beta = ev.score - self.cfg.params.singular_beta_mult * depth;
                let singular_depth = (depth - 1) / 2;
                let singular_score = self.negamax::<NonPv, _>(
                    board,
                    singular_depth,
                    singular_beta - 1,
                    singular_beta,
                    ply,
                    false,
                    mv,
                    false,
                    ply,
                    poll,
                );
                // The verification search ran at this ply and overwrote it.
                self.td.stack[ply].tt_pv = tt_pv;
                if self.td.stopped || self.td.quit {
                    return 0;
                }
                trace_decision!(
                    self,
                    ply,
                    "singular depth {depth} move {mv} tt_score {} singular_beta {singular_beta} \
                     score {singular_score} beta {beta}",
                    ev.score
                );
                if singular_score < singular_beta {
                    extension = if !NODE::PV
                        && singular_score < singular_beta - self.cfg.params.singular_double_margin
                    {
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            crate::diag_count!(singular_extend_two);
                        }
                        2
                    } else {
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            crate::diag_count!(singular_extend_one);
                        }
                        1
                    };
                } else if singular_beta >= beta {
                    #[cfg(feature = "diag")]
                    if diag_sample {
                        crate::diag_count!(singular_multicut);
                    }
                    return singular_beta;
                } else if ev.score >= beta {
                    #[cfg(feature = "diag")]
                    if diag_sample {
                        crate::diag_count!(singular_negative_extension);
                    }
                    extension = -1;
                }
            }

            self.push_move(board, ply, mv, moving_piece);
            self.record_move_order(ply, move_count);
            let nodes_before_move = if NODE::ROOT { self.td.nodes } else { 0 };
            // The check predicate is cheap here (node masks and two bitboard
            // tests) and lets `make_move` skip `calculate_checkers` for the
            // common non-checking move.
            let mv_gives_check = move_gives_check(board, &mut node_ci, mv, &mut gives_check);
            board.make_move_with_check(mv, mv_gives_check);
            self.shared.tt.prefetch(board.hash());
            let mut new_depth = depth - 1 + extension;
            #[cfg(feature = "diag")]
            if diag_sample {
                crate::diag_add!(
                    prospective_depth_sum,
                    u64::try_from(new_depth.max(0)).unwrap_or(0)
                );
            }
            let mut score;

            if searched == 0 {
                search_count += 1;
                score = if child_is_pv {
                    -self.negamax::<Pv, _>(
                        board,
                        new_depth,
                        -beta,
                        -alpha,
                        ply + 1,
                        true,
                        Move::NULL,
                        !child_is_pv && !cut_node,
                        last_critical_ply,
                        poll,
                    )
                } else {
                    -self.negamax::<NonPv, _>(
                        board,
                        new_depth,
                        -beta,
                        -alpha,
                        ply + 1,
                        true,
                        Move::NULL,
                        !child_is_pv && !cut_node,
                        last_critical_ply,
                        poll,
                    )
                };
            } else {
                // Late-move reductions: every move after the first, from depth
                // 2, never at the root or in check.
                if !self.ablated(7) && !NODE::ROOT && !in_check && depth >= 2 {
                    let tt_valid = ev.bound.is_some();
                    let reduction = self.late_move_reduction(&LateMoveInputs {
                        depth,
                        improvement,
                        corr_abs,
                        exact: alpha > original_alpha,
                        tt_score_below_alpha: tt_valid && ev.score <= alpha,
                        tt_score_above_alpha: tt_valid && ev.score > alpha,
                        tt_shallow: tt_valid && ev.depth < depth,
                        tt_deep: tt_valid && ev.depth >= depth,
                        win_beta: beta >= TB_WIN_SCORE,
                        is_quiet,
                        history: if is_quiet {
                            quiet_hist
                        } else {
                            self.td.hist.noisy(
                                threats.all,
                                !board.side_to_move(),
                                moving_piece,
                                mv.to_sq(),
                                captured_piece,
                            )
                        },
                        alpha_gap: alpha - eval_for_pruning,
                        critical_distance: infra::to_i32(ply.saturating_sub(last_critical_ply)),
                        pv: NODE::PV,
                        window: beta - alpha,
                        laterality: self.td.stack[ply].laterality,
                        tt_pv,
                        cut_node,
                        tt_move_null: tt_move.is_null(),
                        gives_check: mv_gives_check,
                        child_cutoffs: self.td.stack[ply + 1].cutoff_count,
                        parent_reduction: self.td.stack.back(ply, 1).reduction,
                    });
                    let reduced_depth = reduced_depth(new_depth, reduction, NODE::PV);
                    crate::diag_count!(lmr_applied);
                    #[cfg(feature = "diag")]
                    {
                        if new_depth - reduction / 1024 < 1 {
                            crate::diag_count!(lmr_floor_hits);
                        }
                        if reduced_depth > new_depth {
                            crate::diag_count!(lmr_extended);
                        } else if reduced_depth == new_depth {
                            crate::diag_count!(node_lmr_zero_reduction);
                        }
                        crate::diag_add!(
                            reduction_depth_sum,
                            u64::try_from(new_depth - reduced_depth).unwrap_or(0)
                        );
                    }
                    trace_decision!(
                        self,
                        ply,
                        "lmr depth {depth} move {mv} move_count {move_count} quiet {is_quiet} \
                         history {quiet_hist} improvement {improvement} corr {corr_abs} \
                         cutoffs {} laterality {} units {reduction} new_depth {new_depth} \
                         reduced_depth {reduced_depth} window {alpha} {beta}",
                        self.td.stack[ply + 1].cutoff_count,
                        self.td.stack[ply].laterality
                    );
                    self.td.stack[ply].reduction = reduction;
                    search_count += 1;
                    score = -self.negamax::<NonPv, _>(
                        board,
                        reduced_depth,
                        -alpha - 1,
                        -alpha,
                        ply + 1,
                        true,
                        Move::NULL,
                        true,
                        ply + 1,
                        poll,
                    );
                    self.td.stack[ply].reduction = 0;
                    if score > alpha {
                        // A reduced move that beat alpha by a wide margin earns
                        // a deeper verification; one that barely did, a
                        // shallower one.
                        let deeper = score > best_score + self.cfg.core.lmr_research_deeper;
                        let shallower = score < best_score + self.cfg.core.lmr_research_shallower;
                        if deeper {
                            crate::diag_count!(lmr_research_deeper);
                        }
                        if shallower {
                            crate::diag_count!(lmr_research_shallower);
                        }
                        new_depth += i32::from(deeper) - i32::from(shallower);
                        if new_depth > reduced_depth {
                            crate::diag_count!(lmr_research);
                            trace_decision!(
                                self,
                                ply,
                                "lmr_research move {mv} reduced_score {score} alpha {alpha} \
                                 new_depth {new_depth}"
                            );
                            search_count += 1;
                            score = -self.negamax::<NonPv, _>(
                                board,
                                new_depth,
                                -alpha - 1,
                                -alpha,
                                ply + 1,
                                true,
                                Move::NULL,
                                !cut_node,
                                last_critical_ply,
                                poll,
                            );
                        }
                    }
                } else {
                    search_count += 1;
                    score = -self.negamax::<NonPv, _>(
                        board,
                        new_depth,
                        -alpha - 1,
                        -alpha,
                        ply + 1,
                        true,
                        Move::NULL,
                        !cut_node,
                        ply + 1,
                        poll,
                    );
                }
                // Principal variation search: at a PV node a later move that
                // beats alpha is searched again with the full window.
                if NODE::PV && score > alpha {
                    if mv == tt_move && ev.depth > 1 {
                        new_depth = new_depth.max(1);
                    }
                    search_count += 1;
                    score = -self.negamax::<Pv, _>(
                        board,
                        new_depth,
                        -beta,
                        -alpha,
                        ply + 1,
                        true,
                        Move::NULL,
                        false,
                        last_critical_ply,
                        poll,
                    );
                }
            }
            board.unmake_move(mv);
            self.clear_move(ply);

            if self.td.stopped || self.td.quit {
                return 0;
            }

            let move_nodes = if NODE::ROOT {
                self.td.nodes.saturating_sub(nodes_before_move)
            } else {
                0
            };
            searched += 1;
            // 8.13: publish EVERY searched root move to the pool with its
            // real bound — a fail-low ("true <= score", Upper) is exactly the
            // "stop re-deriving each other's refutations" knowledge a
            // best-move-only summary would lose. `alpha` is still pre-update
            // here, so the classification reads: cutoff = Lower, raised
            // alpha = Exact, else Upper. Serial searches have no shared state.
            if NODE::ROOT {
                self.record_root_move_search(mv, depth, score, alpha, beta, move_nodes);
            }
            if score > best_score {
                best_score = score;
                best_move = mv;
                if NODE::ROOT {
                    self.td.root_best_nodes = move_nodes;
                }
            }
            let raised_alpha = score > alpha;
            if raised_alpha {
                alpha = score;
                self.td.pv_table[ply][ply] = mv;
                let child_len = self.td.pv_len[ply + 1].max(ply + 1);
                for next_ply in ply + 1..child_len {
                    self.td.pv_table[ply][next_ply] = self.td.pv_table[ply + 1][next_ply];
                }
                self.td.pv_len[ply] = child_len;

                if score >= beta {
                    self.td.stack[ply].cutoff_count += 1;
                    if excluded.is_null() {
                        // 10.0(a): `searched` was incremented for this move
                        // above, so `== 1` means the node's FIRST move failed
                        // high. Denominator is cutoff_quiet + cutoff_capture,
                        // both counted in this same block. `cfg`-gated rather
                        // than relying on `diag_count!` expanding to nothing,
                        // because the condition would leave an empty `if` in
                        // the default build.
                        #[cfg(feature = "diag")]
                        if searched == 1 {
                            crate::diag_count!(cutoff_first_move);
                        }
                        // 4.2 core: cutoff rank, exact, same block and same
                        // denominator as cutoff_quiet + cutoff_capture. The
                        // buckets must sum to that total and best_rank_1 must
                        // equal cutoff_first_move; both are cross-checks the
                        // oracle satisfies too.
                        #[cfg(feature = "diag")]
                        match searched {
                            1 => crate::diag_count!(best_rank_1),
                            2 | 3 => crate::diag_count!(best_rank_2_3),
                            4..=7 => crate::diag_count!(best_rank_4_7),
                            _ => crate::diag_count!(best_rank_8_plus),
                        }
                        if is_capture {
                            crate::diag_count!(cutoff_capture);
                        } else {
                            crate::diag_count!(cutoff_quiet);
                        }
                        self.shared.tt.store(TtStore {
                            key: hash,
                            depth,
                            score,
                            bound: Bound::Lower,
                            mv,
                            ply,
                            static_eval: raw_static_eval,
                            is_pv: tt_pv,
                        });
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            crate::diag_count!(main_store_lower);
                        }
                    }
                    self.update_best_move_histories(
                        board,
                        &BestMoveUpdate {
                            threats: threats.all,
                            ply,
                            depth,
                            cut_node,
                            best: mv,
                            quiets: quiet_moves.as_slice(),
                            noisies: noisy_moves.as_slice(),
                            search_count,
                            fail_high: true,
                        },
                    );
                    // A fail-high above the static eval with a quiet move
                    // trains the correction; see the node's end.
                    if !in_check && !is_noisy(mv) && score > static_eval {
                        self.train_correction(board, depth, score - static_eval, ply);
                    }
                    return score;
                }
            }

            if !raised_alpha && move_count < 32 {
                if is_quiet {
                    quiet_moves.push(mv);
                } else {
                    noisy_moves.push(mv);
                }
            }
        }

        if !legal_move_seen {
            return if in_check {
                -MATE_SCORE + infra::to_i32(ply)
            } else {
                0
            };
        }

        let bound = if best_score > original_alpha {
            Bound::Exact
        } else {
            Bound::Upper
        };
        // A fail-low reached through a PV-line parent after several moves is
        // still on that line: the parent's next iteration will search it with
        // an open window.
        tt_pv |= !NODE::ROOT
            && bound == Bound::Upper
            && move_count > 2
            && self.td.stack.back(ply, 1).tt_pv;
        // The correction learns only what the static eval can be blamed for:
        // never in check, never from a noisy best move (the residual is then
        // tactics, not a positional error), and only where the bound agrees
        // with the residual's sign (a fail-low above the eval says nothing
        // about how far above).
        if !in_check
            && !(bound == Bound::Exact && is_noisy(best_move))
            && !(bound == Bound::Upper && best_score >= static_eval)
        {
            self.train_correction(board, depth, best_score - static_eval, ply);
        }
        if bound == Bound::Exact {
            self.update_best_move_histories(
                board,
                &BestMoveUpdate {
                    threats: threats.all,
                    ply,
                    depth,
                    cut_node,
                    best: best_move,
                    quiets: quiet_moves.as_slice(),
                    noisies: noisy_moves.as_slice(),
                    search_count: 0,
                    fail_high: false,
                },
            );
        } else if !NODE::ROOT && (cut_node || NODE::PV) {
            self.reward_parent_after_fail_low(board, ply, depth, best_score, static_eval, in_check);
        }
        if excluded.is_null() {
            self.shared.tt.store(TtStore {
                key: hash,
                depth,
                score: best_score,
                bound,
                mv: best_move,
                ply,
                static_eval: raw_static_eval,
                is_pv: tt_pv,
            });
            #[cfg(feature = "diag")]
            if diag_sample {
                match bound {
                    Bound::Exact => {
                        crate::diag_count!(main_store_exact);
                    }
                    Bound::Upper => {
                        crate::diag_count!(main_store_upper);
                    }
                    Bound::Lower => {}
                }
            }
        }
        best_score
    }

    pub(super) fn quiescence<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        qply: usize,
        poll: &mut P,
    ) -> i32 {
        if self.check_stop(poll) {
            return 0;
        }
        if ply >= MAX_PLY - 1 {
            return self.corrected_eval(board, MAX_PLY - 1);
        }
        self.td.pv_len[ply] = ply;
        self.td.seldepth = self.td.seldepth.max(ply);

        if board.can_declare_draw_in_search() {
            return 0;
        }

        let in_check = board.is_in_check();
        crate::diag_count!(qnodes);
        let hash = board.hash();
        #[cfg(feature = "diag")]
        let diag_q_sample = crate::diag::sampled(hash, ply + qply, crate::diag::SAMPLE_QSEARCH);
        #[cfg(feature = "diag")]
        if diag_q_sample {
            crate::diag_count!(sampled_qnodes);
            if in_check {
                crate::diag_count!(q_in_check);
            }
        }
        let original_alpha = alpha;
        let tt_entry = self.shared.tt.probe(hash);
        let ev = TtProbe::from_entry(tt_entry, ply, board.halfmove_clock());
        #[cfg(feature = "diag")]
        if diag_q_sample && ev.hit {
            crate::diag_count!(q_tt_hit);
            if ev.cutoff_score(0, alpha, beta).is_some() {
                crate::diag_count!(q_tt_cut);
            }
        }
        // Depth 0 is the whole admission bar here: any stored entry outranks a
        // qsearch node. That includes a stand pat stored by an earlier visit.
        if let Some(score) = ev.cutoff_score(0, alpha, beta) {
            return score;
        }
        let tt_move = ev
            .mv
            .and_then(|mv| board.legal_move(mv))
            .unwrap_or(Move::NULL);

        let mut q_raw_static_eval = VALUE_NONE;
        let mut stand_pat_for_pruning = VALUE_NONE;
        // 8.11 RE-APPLIED for 10.4.6(a) (was rejected standalone at −5.96 ±
        // 7.33, LOS 5.56%, 3,558 games). The two prune exits below are
        // fail-soft: they report the stand pat, which is genuinely BELOW the
        // window, instead of a bare `alpha` that overstates what this node
        // proved. `negamax` has always been fail-soft; this makes qsearch agree.
        //
        // Why it is back, and why only bundled: its −5.96 was mechanically
        // traced to the pruning group having been SPSA-fitted against
        // fail-hard's inflated bounds, so a standalone gate against the tuned
        // head is rigged to fail (lesson 15, same shape as 7.2's SEE bundle).
        // 10.4.6(a) re-tunes that exact group, so this rides its gate and 8.11
        // closes either way: if the bundle loses, the registered fallback is one
        // re-gate at the fitted values WITHOUT this change, no new tune.
        //
        // ⚠ This is 8.11 as GATED (the commit's "variant B", prune exits only,
        // +2.8% nodes). The full form that also made the tail store/return
        // fail-soft measured +17.2% nodes and was explicitly ruled out — do not
        // widen to it here. The tail deliberately still stores `alpha`, so the
        // depth-0 Upper bound that `eval_for_pruning` consumes is UNCHANGED by
        // this edit. Restricting that coupling was tested separately and did not
        // earn its complexity, so the accepted zero-depth floor is hardwired.
        //
        // Written without the original's `best_score` accumulator: both exits
        // return `stand_pat`, so the variable and its in-loop update were dead
        // weight (nothing after the move loop reads it in this variant). The
        // node behaviour is identical — bench must land on 5,320,596, the figure
        // the gated candidate measured.
        if !in_check {
            // Same three-branch collapse as the main search — see there.
            let (stand_pat, raw_stand_pat) = {
                let raw = if ev.raw_static_eval == VALUE_NONE {
                    self.raw_eval(board)
                } else {
                    ev.raw_static_eval
                };
                (self.corrected_eval_from_raw(board, raw, ply), raw)
            };
            q_raw_static_eval = raw_stand_pat;
            // Mirror the main search's eval_for_pruning TT-bound refinement.
            // If the TT score is bounded (Exact, or a one-sided bound that
            // agrees with the bound direction), use it as the stand_pat instead
            // of the raw static eval — cheap cutoffs we would otherwise miss.
            // Preserve the accepted depth-0 refinement behavior. Restricting it
            // would remove much of RAR-S02's accepted mechanism.
            let stand_pat = ev.refine_eval(stand_pat, 0);
            stand_pat_for_pruning = stand_pat;
            if stand_pat >= beta {
                #[cfg(feature = "diag")]
                if diag_q_sample {
                    crate::diag_count!(q_stand_pat_cut);
                    crate::diag_count!(q_stand_pat_store);
                }
                // 4.6.1 MEASURED AND KEPT. A bare stand-pat is a Lower bound
                // at depth 0 that searched no move and carries none, and it is
                // 35.87% of all stores (RAR-S23), so the audit suspected it of
                // causing `tt_bound_not_usable` 2.13x. Suppressing it makes
                // that metric WORSE, not better: not-usable per hit rises
                // 9.5% -> 14.9%, hit rate falls 20.6pp, and total TT cutoffs
                // fall 10.3% against a 7.5% smaller tree — cutoffs dropping
                // faster than nodes, which is the RAR-S59 signature of a bad
                // change. These entries earn their slot. Do not re-derive.
                self.shared.tt.store(TtStore {
                    key: hash,
                    depth: 0,
                    score: stand_pat,
                    bound: Bound::Lower,
                    mv: Move::NULL,
                    ply,
                    static_eval: q_raw_static_eval,
                    is_pv: false,
                });
                return stand_pat;
            }
            if qply >= MAX_QPLY {
                return stand_pat;
            }
            if stand_pat > alpha {
                alpha = stand_pat;
            }
            if board.occupied_count() > 8 && stand_pat + piece_value(Piece::Queen) + 200 < alpha {
                // Reached only when `stand_pat < alpha`, so `alpha` here is still
                // the caller's bound and `stand_pat` is the honest lower figure.
                return stand_pat;
            }
        }

        let mut moves = MoveList::new();
        if in_check {
            board.generate_legal_movelist_into(&mut moves);
        } else {
            board.generate_legal_captures_into(&mut moves);
        }

        if in_check && moves.is_empty() {
            return -MATE_SCORE + infra::to_i32(ply);
        }

        let mut best_move = Move::NULL;
        let threats = board.threats();
        let mut scored = if in_check {
            self.score_moves(board, &threats, moves.as_slice(), tt_move, ply)
        } else {
            self.score_tactical_moves(board, &threats, moves.as_slice(), tt_move)
        };
        // 4.9d sizing: what did scoring every evasion buy at this node?
        #[cfg(feature = "diag")]
        if in_check {
            crate::diag_count!(q_check_nodes);
            crate::diag_add!(
                q_check_moves_scored,
                u64::try_from(scored.len()).unwrap_or(u64::MAX)
            );
        }
        // 10.3: per-node check masks, built lazily and shared by every move at
        // this qnode (see negamax for the same pattern). Capture-only qnodes
        // that never test for check never build it.
        let mut node_ci: Option<CheckInfo> = None;
        let mut tactical_count = 0usize;
        for index in 0..scored.len() {
            let picked = pick_next(scored.as_mut_slice(), index);
            #[cfg(feature = "diag")]
            if in_check {
                crate::diag_count!(q_check_moves_tried);
            }
            let mv = picked.mv;
            if !in_check {
                let mut gives_check = None;
                tactical_count += 1;
                if !mv.is_promo()
                    && stand_pat_for_pruning != VALUE_NONE
                    && stand_pat_for_pruning + board.captured_piece(mv).map_or(0, piece_value) + 150
                        <= alpha
                    && !move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                {
                    continue;
                }
                if !mv.is_promo()
                    && tactical_count > 6
                    && picked.see < 0
                    && !move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                {
                    continue;
                }
                if !mv.is_promo() {
                    let see_threshold =
                        (alpha - stand_pat_for_pruning - self.cfg.params.qs_see_margin).clamp(
                            self.cfg.params.qs_see_clamp_lo,
                            self.cfg.params.qs_see_clamp_hi,
                        );
                    if !board.see_ge(mv, see_threshold) {
                        continue;
                    }
                }
                if picked.see < 0 && !board.see_ge(mv, self.cfg.params.qs_see_bad_floor) {
                    continue;
                }
            }
            let moving_piece = board.moving_piece(mv);
            self.push_move(board, ply, mv, moving_piece);
            board.make_move(mv);
            self.shared.tt.prefetch(board.hash());
            let score = -self.quiescence::<NODE, _>(board, -beta, -alpha, ply + 1, qply + 1, poll);
            board.unmake_move(mv);
            self.clear_move(ply);
            if self.td.stopped || self.td.quit {
                return 0;
            }
            if score >= beta {
                #[cfg(feature = "diag")]
                if diag_q_sample {
                    crate::diag_count!(q_move_cut);
                    crate::diag_count!(q_move_store);
                }
                self.shared.tt.store(TtStore {
                    key: hash,
                    depth: 0,
                    score,
                    bound: Bound::Lower,
                    mv,
                    ply,
                    static_eval: q_raw_static_eval,
                    is_pv: false,
                });
                return score;
            }
            if score > alpha {
                alpha = score;
                best_move = mv;
                self.td.pv_table[ply][ply] = mv;
                let child_len = self.td.pv_len[ply + 1].max(ply + 1);
                for next_ply in ply + 1..child_len {
                    self.td.pv_table[ply][next_ply] = self.td.pv_table[ply + 1][next_ply];
                }
                self.td.pv_len[ply] = child_len;
            }
        }
        let bound = if alpha > original_alpha {
            Bound::Exact
        } else {
            Bound::Upper
        };
        #[cfg(feature = "diag")]
        if diag_q_sample {
            match bound {
                Bound::Exact => {
                    crate::diag_count!(q_tail_exact_store);
                }
                Bound::Upper => {
                    crate::diag_count!(q_tail_upper_store);
                }
                Bound::Lower => {}
            }
        }
        self.shared.tt.store(TtStore {
            key: hash,
            depth: 0,
            score: alpha,
            bound,
            mv: best_move,
            ply,
            static_eval: q_raw_static_eval,
            is_pv: false,
        });
        alpha
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reduced move always keeps at least one ply of main search, a
    /// negative reduction extends by at most two plies, and PV nodes add two.
    #[test]
    fn reduced_depth_is_floored_at_one_ply_and_capped_above() {
        assert_eq!(reduced_depth(8, 3 * 1024, false), 5);
        assert_eq!(reduced_depth(8, 3 * 1024 + 1023, false), 5, "whole plies");
        assert_eq!(reduced_depth(3, 10 * 1024, false), 1, "floor");
        assert_eq!(reduced_depth(1, 1024, false), 1, "floor at depth 1");
        assert_eq!(reduced_depth(6, -5 * 1024, false), 8, "cap");
        assert_eq!(reduced_depth(6, -1023, false), 6, "truncates toward zero");
        assert_eq!(reduced_depth(6, 2 * 1024, true), 6, "PV adds two");
        assert_eq!(reduced_depth(3, 10 * 1024, true), 3, "PV floor plus two");
        for new_depth in 1..40 {
            for reduction in (-20_000..20_000).step_by(977) {
                assert!(reduced_depth(new_depth, reduction, false) >= 1);
                assert!(reduced_depth(new_depth, reduction, false) <= new_depth + 2);
            }
        }
    }

    #[test]
    fn laterality_grows_by_the_log_of_the_move_index() {
        let steps = [0, 0, 0, 0, 1, 1, 1, 1, 2, 2];
        for (move_count, &expected) in steps.iter().enumerate() {
            assert_eq!(
                laterality_step(infra::to_i32(move_count)),
                expected,
                "move {move_count}"
            );
        }
        assert_eq!(laterality_step(16), 3);
        assert_eq!(laterality_step(255), 6);
    }

    /// After a search unwinds, no ply still carries an LMR reduction: the
    /// field is non-zero only while the reduced child is being searched, and
    /// a child reads its parent's entry.
    #[test]
    fn stack_reductions_unwind_to_zero() {
        let mut searcher = Searcher::default();
        let mut board =
            Board::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4")
                .expect("valid FEN");
        let score = searcher.search_root_window(&mut board, 7, -INF_SCORE, INF_SCORE, &mut || {
            SearchEvent::None
        });
        assert!(score.abs() < INF_SCORE);
        assert!(
            searcher.td.nodes > 1_000,
            "the search must reach reductions"
        );
        for ply in 0..MAX_PLY {
            assert_eq!(searcher.td.stack[ply].reduction, 0, "ply {ply}");
        }
    }

    /// A quiet move that gives direct check survives the move-count skip and
    /// quiet futility even when it is ordered last: every other quiet gets a
    /// saturated history, so `Ra8#` comes out after them, past the count at
    /// which the rest of the quiets are skipped, at a node far below its
    /// window.
    #[test]
    fn a_late_direct_check_survives_the_quiet_skip() {
        let mut searcher = Searcher::default();
        let mut board = Board::from_fen("6k1/5ppp/8/8/8/8/1P6/R3K3 w - - 0 1").expect("valid FEN");
        let mate = board.parse_move("a1a8").expect("legal quiet mate");
        let threats = board.threats();
        for mv in board.generate_legal_moves() {
            if mv != mate && board.is_quiet_move(mv) {
                for _ in 0..64 {
                    searcher.td.hist.update_quiet(
                        threats.all,
                        crate::board::Color::White,
                        mv,
                        8_192,
                    );
                }
            }
        }
        let quiets = board
            .generate_legal_moves()
            .into_iter()
            .filter(|&mv| board.is_quiet_move(mv))
            .count();
        assert!(quiets > 8, "the skip must be reachable: {quiets} quiets");
        let alpha = MATE_SCORE - 100;
        let score = searcher.negamax::<NonPv, _>(
            &mut board,
            1,
            alpha,
            alpha + 1,
            1,
            false,
            Move::NULL,
            false,
            1,
            &mut || SearchEvent::None,
        );
        assert_eq!(score, MATE_SCORE - 2, "the late quiet mate was pruned");
    }

    /// The grandchild cutoff reset stays inside the stack at the deepest node
    /// that reaches it.
    #[test]
    fn cutoff_count_reset_is_bounded_at_the_ply_cap() {
        let mut searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/8/8/3q4/8/8/4KQ2 w - - 0 1").expect("valid FEN");
        for ply in [MAX_PLY - 3, MAX_PLY - 2] {
            let score = searcher.negamax::<NonPv, _>(
                &mut board,
                2,
                -1,
                0,
                ply,
                false,
                Move::NULL,
                false,
                0,
                &mut || SearchEvent::None,
            );
            assert!(score.abs() < INF_SCORE, "ply {ply}");
        }
    }

    #[test]
    fn quiescence_detects_mate_after_first_qply_check() {
        let mut searcher = Searcher::default();
        let mut board =
            Board::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
                .expect("valid fool's mate FEN");

        let score =
            searcher.quiescence::<NonPv, _>(&mut board, -INF_SCORE, INF_SCORE, 0, 1, &mut || {
                SearchEvent::None
            });

        assert_eq!(score, -MATE_SCORE);
    }

    #[test]
    fn quiescence_stops_before_ply_stack_overflow() {
        let mut searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/8/8/3q4/8/8/4KQ2 w - - 0 1").expect("valid FEN");
        let before = board.to_fen();

        let score = searcher.quiescence::<NonPv, _>(
            &mut board,
            -INF_SCORE,
            INF_SCORE,
            MAX_PLY - 1,
            0,
            &mut || SearchEvent::None,
        );

        assert_eq!(board.to_fen(), before);
        assert!(score.abs() < INF_SCORE);
    }
}
