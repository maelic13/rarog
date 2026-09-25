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

/// A score in the tablebase-win band or beyond: a win the search may return
/// or store only where it proved it.
#[cfg(feature = "b3proof")]
const fn is_win(score: i32) -> bool {
    score >= TB_WIN_SCORE
}

/// A score in the tablebase-loss band or beyond.
#[cfg(feature = "b3proof")]
const fn is_loss(score: i32) -> bool {
    score <= -TB_WIN_SCORE
}

/// A win or a loss, see [`is_win`].
#[cfg(feature = "b3proof")]
const fn is_decisive(score: i32) -> bool {
    is_win(score) || is_loss(score)
}

/// What a null-move cutoff returns: the null search's score, but never a win
/// the reduced search did not prove and never less than beta, which the
/// verification established when the null search ran against a lower bound
/// below it.
#[cfg(feature = "b3proof")]
fn null_cutoff_score(score: i32, beta: i32) -> i32 {
    if is_win(score) { beta } else { score.max(beta) }
}

/// What a multi-cut returns: `None` unless the exclusion search failed high
/// with a score short of the decisive band; otherwise the score pulled
/// `lerp` 1024ths of the way to beta, and never into the decisive band, which
/// only a proof may reach.
#[cfg(feature = "b3proof")]
fn multicut_score(score: i32, beta: i32, lerp: i32) -> Option<i32> {
    (score >= beta && !is_decisive(score))
        .then(|| (score + (beta - score) * lerp / 1024).max(-TB_WIN_SCORE + 1))
}

// Float-to-int truncation is the table formula's rounding, hence the scoped
// cast allow. The root loop builds the table for either arm; this arm's
// reductions do not read it.
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

/// A `negamax` frame's node type, resolved at compile time, so quiescence can
/// know it is on a PV line without a new parameter. `cut_node` stays a runtime
/// argument, so an all-node is `!PV && !cut_node`.
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
    /// The TT move's score at this node minus its exclusion search's, when
    /// both were searched.
    #[cfg(feature = "b3proof")]
    singular_gap: Option<i32>,
}

/// The inputs of the full-depth branch's reduction for one move.
#[derive(Copy, Clone)]
struct FullDepthInputs {
    depth: i32,
    improvement: i32,
    corr_abs: i32,
    is_quiet: bool,
    history: i32,
    tt_pv: bool,
    /// A stored result at least as deep as this node.
    tt_deep: bool,
    cut_node: bool,
    tt_move_null: bool,
    is_tt_move: bool,
    /// Beta cutoffs among this node's children so far.
    child_cutoffs: i32,
    parent_reduction: i32,
}

/// The depth the full-depth branch searches a move at: one ply off from a
/// reduction of 2621 units and two from 5579, never below one ply and never
/// deeper than `new_depth`.
fn full_depth_searched_depth(new_depth: i32, reduction: i32) -> i32 {
    let plies = i32::from(reduction >= 2_621) + i32::from(reduction >= 5_579);
    (new_depth - plies).max(new_depth.min(1))
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

/// Per-move check test, memoized twice: `cache` holds the answer for this
/// move, `node_ci` the per-node masks every move at the node shares (see
/// [`Board::check_info`]).
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
    /// Does the side to move hold enough non-pawn material to trust a null
    /// move? At `NmpMinNonPawnPieces = 1` one piece suffices; more demands
    /// more, because zugzwang risk concentrates where the mover has almost
    /// nothing left to move.
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

    /// Whether the null move's population takes this node: every node off
    /// the PV line at `CoreNmpNodes = 0`, expected cut nodes only at 1.
    #[cfg(feature = "b3proof")]
    #[inline(always)]
    fn nmp_population(&self, pv: bool, cut_node: bool, tt_pv: bool) -> bool {
        if self.cfg.proof.nmp_nodes == 0 {
            !tt_pv
        } else {
            cut_node && !pv
        }
    }

    /// Whether ProbCut's population takes this node: every node off the PV
    /// line at `CoreProbcutNodes = 0`, expected cut nodes only at 1.
    #[cfg(feature = "b3proof")]
    #[inline(always)]
    fn probcut_population(&self, pv: bool, cut_node: bool, tt_pv: bool) -> bool {
        if self.cfg.proof.probcut_nodes == 0 {
            !tt_pv
        } else {
            cut_node && !pv
        }
    }

    /// How far a singular TT move extends, one to three plies, given how far
    /// its exclusion search fell below the singular beta (`below > 0`). The
    /// bars for two and three plies are higher on a PV node, higher still
    /// when the table did not already place it on a PV line, lower for a
    /// quiet TT move and with the correction's size.
    #[cfg(feature = "b3proof")]
    fn singular_extension(
        &self,
        below: i32,
        pv: bool,
        tt_was_pv: bool,
        quiet_tt: bool,
        corr_abs: i32,
    ) -> i32 {
        let p = &self.cfg.proof;
        let new_pv = i32::from(pv && !tt_was_pv);
        let double = p.sing_double_pv * i32::from(pv) + p.sing_double_not_tt_pv * new_pv
            - p.sing_double_quiet * i32::from(quiet_tt)
            - p.sing_double_corr * corr_abs / 128
            + p.sing_double_base;
        let triple = p.sing_triple_pv * i32::from(pv) + p.sing_triple_not_tt_pv * new_pv
            - p.sing_triple_quiet * i32::from(quiet_tt)
            - p.sing_triple_corr * corr_abs / 128
            + p.sing_triple_base;
        1 + i32::from(below > double) + i32::from(below > triple)
    }

    /// The part of a positive extension the line's budget still allows: the
    /// extensions taken from the root may sum to the iteration's depth and no
    /// more, so an extension at the edge is truncated, never refused whole.
    #[cfg(feature = "b3proof")]
    #[inline(always)]
    fn within_extension_budget(&mut self, extension: i32, spent: i32) -> i32 {
        let granted = extension.min(self.td.root_depth - spent).max(0);
        if granted < extension {
            crate::diag_count!(singular_extension_truncated);
            #[cfg(test)]
            {
                self.td.budget_truncations += 1;
            }
        }
        granted
    }

    /// Whether a node without a singular candidate extends its first move:
    /// an expected cut node at depth 7 or less, out of check, whose estimate
    /// sits `CoreLdseMargin` or more below alpha.
    #[cfg(feature = "b3proof")]
    #[inline(always)]
    fn ldse_applies(
        &self,
        depth: i32,
        in_check: bool,
        cut_node: bool,
        estimate: i32,
        alpha: i32,
    ) -> bool {
        depth <= 7 && !in_check && cut_node && estimate <= alpha - self.cfg.proof.ldse_margin
    }

    /// The late-move reduction's singular term, in 1024ths of a ply: how far
    /// the TT move's score at this node stood above its exclusion search,
    /// past an offset, scaled and capped. Zero unless both scores exist.
    #[cfg(feature = "b3proof")]
    #[inline(always)]
    fn lmr_singular_term(&self, singular_gap: Option<i32>) -> i32 {
        let p = &self.cfg.proof;
        singular_gap.map_or(0, |gap| {
            (p.lmr_singular_slope * (gap - p.lmr_singular_offset) / 128)
                .clamp(0, p.lmr_singular_cap)
        })
    }

    /// How far above beta the estimated score must stand for a null move, in
    /// evaluation units, never below 2: less at depth and with improvement,
    /// more on a PV line, less when the children have failed high fewer than
    /// twice (a null move there is likelier to hold).
    #[cfg(feature = "b3proof")]
    #[inline(always)]
    fn nmp_margin(&self, depth: i32, tt_pv: bool, improvement: i32, child_cutoffs: i32) -> i32 {
        let p = &self.cfg.proof;
        (p.nmp_base - p.nmp_depth * depth + p.nmp_tt_pv * i32::from(tt_pv)
            - p.nmp_improvement * improvement / 1024
            - p.nmp_cutoff * i32::from(child_cutoffs < 2))
        .max(2)
    }

    /// The null move's reduction in whole plies: a base, more when improving,
    /// with depth and with the estimate's surplus over beta up to a clamp.
    #[cfg(feature = "b3proof")]
    #[inline(always)]
    fn nmp_reduction(&self, depth: i32, improving: bool, surplus: i32) -> i32 {
        let p = &self.cfg.proof;
        (p.nmp_r_base
            + p.nmp_r_improving * i32::from(improving)
            + p.nmp_r_depth * depth
            + p.nmp_r_eval * surplus.clamp(0, p.nmp_r_clamp) / 128)
            / 1024
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
        #[cfg(feature = "b3proof")]
        {
            self.td.root_depth = depth;
        }
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
        r -= (p.lmr_improvement * i.improvement / 128)
            .clamp(p.lmr_improvement_clamp_lo, p.lmr_improvement_clamp_hi);
        r -= p.lmr_correction * i.corr_abs / 1024;
        r += p.lmr_exact * i32::from(i.exact);
        r += p.lmr_tt_score_below_alpha * i32::from(i.tt_score_below_alpha);
        r += p.lmr_tt_shallow * i32::from(i.tt_shallow);
        r += 1024 * i32::from(i.win_beta);
        if i.is_quiet {
            r += p.lmr_quiet - p.lmr_quiet_history * i.history / 1024
                + p.lmr_alpha_gap * i.alpha_gap.clamp(p.lmr_alpha_gap_lo, p.lmr_alpha_gap_hi) / 128;
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
        #[cfg(feature = "b3proof")]
        {
            let singular = self.lmr_singular_term(i.singular_gap);
            if singular > 0 {
                crate::diag_count!(lmr_singular_term);
            }
            r += singular;
        }
        if !i.pv && i.parent_reduction > r + 414 {
            r += p.lmr_parent;
        }
        let thread = u64::try_from(self.td.thread_id).unwrap_or(0);
        let spread = i32::try_from((self.td.nodes + thread * 27) % 128).unwrap_or(0);
        r + spread - 59
    }

    /// The donor's second reduction, for the moves late-move reductions do not
    /// take (`CoreLmrFullDepth`), in 1024ths of a ply. Its constants are the
    /// donor's seeds, converted where they bound evaluation units, and are
    /// not coordinates. Reduces more: a quiet, a cut node (more without a TT
    /// move), children that keep failing high, a parent that reduced far
    /// more. Reduces less: improvement, a large correction, good history, a
    /// PV line in the table (more when its entry is deep), the TT move.
    fn full_depth_reduction(&self, i: &FullDepthInputs) -> i32 {
        let mut r = 207 * i.depth.ilog2().cast_signed();
        r -= (366 * i.improvement / 128).clamp(-94, 626);
        r -= 2_255 * i.corr_abs / 1024;
        if i.is_quiet {
            r += 1_468 - 118 * i.history / 1024;
        } else {
            r += 940 - 63 * i.history / 1024;
        }
        if i.tt_pv {
            r -= 844 + 1_129 * i32::from(i.tt_deep);
        } else if i.cut_node {
            r += 1_260 + 2_168 * i32::from(i.tt_move_null);
        }
        if i.child_cutoffs > 2 {
            r += 1_394 + 258 * i32::from(!i.cut_node);
        }
        if i.is_tt_move {
            r -= 3_002;
        }
        if i.parent_reduction > r + 590 {
            r += 130;
        }
        let thread = u64::try_from(self.td.thread_id).unwrap_or(0);
        let spread = i32::try_from((self.td.nodes + thread * 26) % 128).unwrap_or(0);
        r + spread - 56
    }

    /// Whether a node's result may train the correction, given the
    /// categorical admission switches: a decisive score is admitted only when
    /// `CoreCorrTrainDecisive` is set, a singular-exclusion node only when
    /// `CoreCorrTrainExcluded` is.
    #[inline(always)]
    fn admits_correction_training(&self, score: i32, excluded: Move) -> bool {
        (self.cfg.core.corr_train_decisive != 0 || score.abs() < TB_WIN_SCORE)
            && (self.cfg.core.corr_train_excluded != 0 || excluded.is_null())
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
        self.td.seldepth = self.td.seldepth.max(ply + 1);

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

        // Counted after the hand-off to quiescence, so `nodes` means interior
        // nodes and no node is also counted as a qnode.
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
        // Main thread only: if helpers' work reaches the thread that owns the
        // answer, this hit rate rises with the thread count.
        if self.td.thread_id == 0 {
            crate::diag_count!(main_tt_probes);
            if tt_entry.is_some() {
                crate::diag_count!(main_tt_hits);
            }
        }
        // One decode of the probe for the whole node: mate distance and rule-50
        // are resolved exactly once.
        let ev = TtProbe::from_entry(tt_entry, ply, board.halfmove_clock());
        let mut tt_pv = ev.pv_line(NODE::PV);
        // Sampled census of what the probe allows, by the admission rule this
        // node applies. PV and excluded-move nodes never cut, so they are
        // attributed first; an entry that fails the depth test is "shallow",
        // one that passes it but does not resolve the window or the node-type
        // test is "not usable".
        #[cfg(feature = "diag")]
        if diag_sample {
            if ev.hit {
                crate::diag_count!(tt_sample_hit);
                if let Some(bound) = ev.bound {
                    if NODE::PV {
                        crate::diag_count!(tt_reject_pv);
                    } else if !excluded.is_null() {
                        crate::diag_count!(tt_reject_excluded);
                    } else if ev.node_cutoff_score(depth, alpha, beta, cut_node).is_some() {
                        match bound {
                            Bound::Exact => crate::diag_count!(tt_cut_exact),
                            Bound::Lower => crate::diag_count!(tt_cut_lower),
                            Bound::Upper => crate::diag_count!(tt_cut_upper),
                        }
                    } else if ev.depth <= depth - i32::from(ev.score < beta) {
                        crate::diag_count!(tt_reject_shallow);
                        crate::diag_add!(
                            tt_reject_shallow_deficit,
                            u64::try_from(depth - ev.depth).unwrap_or(0)
                        );
                    } else {
                        crate::diag_count!(tt_bound_not_usable);
                    }
                    if !NODE::PV && ev.contradicts_window(alpha, beta) {
                        crate::diag_count!(tt_bound_contradicts_window);
                    }
                }
            } else {
                crate::diag_count!(tt_sample_miss);
            }
        }
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
            // Near the fifty-move horizon a stored result may no longer hold,
            // so the node searches instead of trusting it.
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
            // A later MultiPV line excludes the moves already ranked, which
            // usually include the stored one. The line's own candidate stands
            // in, so the root is not taken for a TT-less node and reduced.
            tt_move = if self.td.multipv_line > 0 {
                self.td.root_moves[0]
            } else {
                Move::NULL
            };
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

        // A probe miss and a hit without a stored eval both evaluate afresh;
        // `TtProbe::MISS` reports `VALUE_NONE`, so one test covers both.
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
        #[cfg(not(feature = "b3proof"))]
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
        // fail-high entry says there is more here. With the guards on, also
        // not at a node the table places on a PV line and not above depth 3.
        if !self.ablated(0)
            && !NODE::PV
            && !in_check
            && (self.cfg.core.razor_guards == 0 || (!tt_pv && depth <= 3))
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

        // Whether the table suggests the TT move may be singular: a deep
        // enough lower or exact bound with a score short of the decisive
        // band. The null move stays off such a node; the singular search
        // reads it.
        // The positive extensions the line to this node has taken: its moves
        // may take more only up to the iteration's depth, so no line grows
        // deeper than twice the iteration plus quiescence.
        #[cfg(feature = "b3proof")]
        let extension_spent = self.td.stack.back(ply, 1).extension_spent;
        #[cfg(all(test, feature = "b3proof"))]
        {
            self.td.budget_overrun = self
                .td
                .budget_overrun
                .max(extension_spent - self.td.root_depth.max(0));
        }
        #[cfg(feature = "b3proof")]
        let potential_singularity = !self.ablated(6)
            && depth
                >= if self.cfg.proof.singular_floor == 0 {
                    4
                } else {
                    5 + i32::from(tt_pv)
                }
            && ev.depth >= depth - self.cfg.proof.singular_tt_depth_margin
            && matches!(ev.bound, Some(Bound::Lower | Bound::Exact))
            && !is_decisive(ev.score);

        // Null move: far enough above beta that passing should still hold.
        // Not where the stored refutation is a lower bound won by capturing
        // a knight or more, not at a potentially singular node, and not
        // inside a verification region. A stored lower bound below beta and
        // deep enough is what the null search must hold instead of beta. A
        // fail-high from `CoreNmpVerifyDepth`, or any fail-high against that
        // lower bound, is verified by a search of this node with the null
        // move disabled from here to `nmp_min_ply`; inside a region a
        // fail-high at beta returns unverified and one below it is dropped,
        // so verifications never nest.
        #[cfg(feature = "b3proof")]
        if !NODE::ROOT
            && !self.ablated(2)
            && allow_null
            && depth >= 3
            && !in_check
            && excluded.is_null()
            && self.nmp_population(NODE::PV, cut_node, tt_pv)
            && !potential_singularity
            && !is_loss(beta)
            && !is_win(eval_for_pruning)
            && eval_for_pruning
                >= beta
                    + self.nmp_margin(
                        depth,
                        tt_pv,
                        improvement,
                        self.td.stack[ply + 1].cutoff_count,
                    )
            && !(ev.bound == Some(Bound::Lower)
                && !tt_move.is_null()
                && board
                    .captured_piece(tt_move)
                    .is_some_and(|piece| piece_value(piece) >= piece_value(Piece::Knight)))
            && self.nmp_material_ok(board)
        {
            if infra::to_i32(ply) < self.td.nmp_min_ply {
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(nmp_skip_region);
                }
                trace_decision!(
                    self,
                    ply,
                    "nmp_skip_region depth {depth} min_ply {}",
                    self.td.nmp_min_ply
                );
            } else {
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(nmp_attempt);
                }
                let reduction = self.nmp_reduction(depth, improving, eval_for_pruning - beta);
                let null_bound =
                    if ev.bound == Some(Bound::Lower) && ev.score < beta && depth - 2 <= ev.depth {
                        ev.score
                    } else {
                        beta
                    };
                #[cfg(feature = "diag")]
                if diag_sample && null_bound < beta {
                    crate::diag_count!(nmp_bound_shortcut);
                }
                self.td.stack[ply].laterality = 0;
                self.td.stack[ply].extension_spent = extension_spent;
                #[cfg(test)]
                self.td.null_move_plies.push(ply);
                board.make_null_move();
                self.shared.tt.prefetch(board.hash());
                let score = -self.negamax::<NonPv, _>(
                    board,
                    depth - reduction,
                    -null_bound,
                    -null_bound + 1,
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
                    #[cfg(feature = "diag")]
                    match depth {
                        ..=6 => crate::diag_count!(nmp_cut_d3_6),
                        7..=12 => crate::diag_count!(nmp_cut_d7_12),
                        _ => crate::diag_count!(nmp_cut_d13_plus),
                    }
                    #[cfg(feature = "diag")]
                    if diag_sample {
                        crate::diag_count!(nmp_sample_cut);
                    }
                }
                trace_decision!(
                    self,
                    ply,
                    "nmp depth {depth} reduction {reduction} estimated {eval_for_pruning} \
                     margin {} bound {null_bound} score {score} beta {beta} min_ply {}",
                    self.nmp_margin(
                        depth,
                        tt_pv,
                        improvement,
                        self.td.stack[ply + 1].cutoff_count
                    ),
                    self.td.nmp_min_ply
                );
                if score >= null_bound && !is_loss(score) {
                    let in_region = self.td.nmp_min_ply > 0;
                    if score >= beta && (depth < self.cfg.proof.nmp_verify_depth || in_region) {
                        if is_win(score) {
                            crate::diag_count!(nmp_cut_unproven_mate);
                        }
                        return null_cutoff_score(score, beta);
                    }
                    if !in_region {
                        let reduced = if score < beta {
                            depth / 2
                        } else {
                            depth - reduction
                        };
                        let min_ply = infra::to_i32(ply) + 3 * reduced / 4;
                        crate::diag_count!(nmp_verify_attempt);
                        #[cfg(test)]
                        {
                            self.td.nmp_verifications += 1;
                        }
                        // The verification searches this ply again and
                        // overwrites its stack entry.
                        let tt_pv_here = self.td.stack[ply].tt_pv;
                        let laterality_here = self.td.stack[ply].laterality;
                        self.td.nmp_min_ply = min_ply;
                        let verified = self.negamax::<NonPv, _>(
                            board,
                            reduced,
                            beta - 1,
                            beta,
                            ply,
                            false,
                            Move::NULL,
                            false,
                            last_critical_ply,
                            poll,
                        );
                        self.td.nmp_min_ply = 0;
                        self.td.stack[ply].tt_pv = tt_pv_here;
                        self.td.stack[ply].laterality = laterality_here;
                        if self.td.stopped || self.td.quit {
                            return 0;
                        }
                        trace_decision!(
                            self,
                            ply,
                            "nmp_verify depth {reduced} min_ply {min_ply} score {verified} \
                             null_score {score} beta {beta}"
                        );
                        if verified >= beta {
                            crate::diag_count!(nmp_verify_pass);
                            if is_win(score) {
                                crate::diag_count!(nmp_cut_unproven_mate);
                            }
                            return null_cutoff_score(score, beta);
                        }
                        crate::diag_count!(nmp_verify_fail);
                        if min_ply > infra::to_i32(ply) {
                            crate::diag_count!(nmp_verify_region_fail);
                        }
                    }
                }
            }
        }

        #[cfg(not(feature = "b3proof"))]
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
                    #[cfg(feature = "diag")]
                    match depth {
                        ..=6 => crate::diag_count!(nmp_cut_d3_6),
                        7..=12 => crate::diag_count!(nmp_cut_d7_12),
                        _ => crate::diag_count!(nmp_cut_d13_plus),
                    }
                    // A null-move fail-high proves only "at least beta". A mate
                    // score from the reduced null search is no demonstrated
                    // mate, so the cutoff returns beta instead.
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
                // Per node entering the block, before capture generation, so
                // nodes with no eligible capture count too.
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(probcut_nodes);
                }
                let probcut_beta = beta + self.cfg.params.probcut_margin;
                // A ProbCut capture must plausibly bridge the gap to
                // probcut_beta by SEE. The gap is floored at zero, so at a
                // node already above probcut_beta a capture still may not
                // lose material. The static eval is real: the block runs only
                // out of check.
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
                    #[cfg(feature = "diag")]
                    let qpassed = score >= probcut_beta;
                    let score = if score >= probcut_beta {
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            crate::diag_count!(probcut_qpass);
                            if depth == 4 {
                                crate::diag_count!(probcut_qpass_base0);
                            } else {
                                crate::diag_count!(probcut_verify);
                            }
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
                    #[cfg(feature = "diag")]
                    if diag_sample && qpassed && depth > 4 && score < probcut_beta {
                        crate::diag_count!(probcut_verify_fail);
                    }
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
                            // The margin-shifted value: storing the raw
                            // fail-high measured 5.55% slower to depth.
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

        // ProbCut: a good capture whose reduced search clears beta by a
        // margin refutes the parent's move. With a stored score, it must
        // already clear that margin and not be decisive; without one, the
        // estimate must be at beta. The captures are Rarog's: the SEE must
        // bridge the gap to the margin and at most `CoreProbcutMoveCap` are
        // searched. After a quiescence pass, the verification is shallower
        // the more the pass cleared the margin by, and must then clear a
        // bound raised by `CoreProbcutAdjust` per ply saved; failing that,
        // it is repeated at the full depth. A cut stores a lower bound with
        // its move and returns a score pulled toward beta; a decisive score
        // is returned and stored as proved.
        #[cfg(feature = "b3proof")]
        if !NODE::ROOT
            && !self.ablated(3)
            && !in_check
            && excluded.is_null()
            && !is_win(beta)
            && self.probcut_population(NODE::PV, cut_node, tt_pv)
        {
            let improving_i = i32::from(improving);
            let tt_margin = self.cfg.proof.probcut_tt_margin;
            if self.cfg.proof.probcut_tt_served != 0
                && ev.bound == Some(Bound::Lower)
                && ev.depth >= depth - 4
                && !is_decisive(beta)
                && !is_decisive(ev.score)
                && ev.score >= beta + tt_margin
            {
                crate::diag_count!(probcut_tt_served);
                trace_decision!(
                    self,
                    ply,
                    "probcut_tt_served depth {depth} tt_depth {} tt_score {} beta {beta} \
                     returns {}",
                    ev.depth,
                    ev.score,
                    beta + tt_margin
                );
                return beta + tt_margin;
            }
            let probcut_beta =
                beta + self.cfg.proof.probcut_base - self.cfg.proof.probcut_improving * improving_i;
            let quiet_tt_move =
                self.cfg.proof.probcut_nodes != 0 && !tt_move.is_null() && !is_noisy(tt_move);
            let tt_gate = if ev.bound.is_some() {
                ev.score >= probcut_beta && !is_decisive(ev.score)
            } else {
                eval_for_pruning >= beta
            };
            #[cfg(feature = "diag")]
            if diag_sample && depth >= 4 {
                if quiet_tt_move {
                    crate::diag_count!(probcut_quiet_tt_reject);
                } else if !tt_gate {
                    crate::diag_count!(probcut_tt_gate_reject);
                }
            }
            if depth >= 4 && !quiet_tt_move && tt_gate {
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(probcut_nodes);
                }
                // The gap is floored at zero, so at a node already above
                // `probcut_beta` a capture still may not lose material.
                let see_threshold =
                    ((probcut_beta - static_eval) * self.cfg.params.probcut_see_gap_scale / 100)
                        .max(0);
                let move_cap = self.cfg.proof.probcut_move_cap;
                let base_depth = (depth - 4 - improving_i).max(0);
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
                    #[cfg(feature = "diag")]
                    if diag_sample {
                        crate::diag_count!(probcut_attempt);
                    }
                    #[cfg(test)]
                    {
                        self.td.probcut_searches += 1;
                    }
                    let probcut_piece = board.moving_piece(mv);
                    // ProbCut's child is a verification search, not a
                    // reduced sibling, so it consumes neither selectivity
                    // input. Written explicitly rather than left stale.
                    self.push_move(board, ply, mv, probcut_piece);
                    self.record_move_order(ply, 0);
                    self.td.stack[ply].extension_spent = extension_spent;
                    board.make_move(mv);
                    self.shared.tt.prefetch(board.hash());
                    let mut score = -self.quiescence::<NonPv, _>(
                        board,
                        -probcut_beta,
                        -probcut_beta + 1,
                        ply + 1,
                        0,
                        poll,
                    );
                    #[cfg(feature = "diag")]
                    let qsearch_score = score;
                    let mut probcut_depth = (base_depth
                        - (score - probcut_beta) / self.cfg.proof.probcut_depth_div)
                        .clamp(0, base_depth);
                    // The bound this move's search must clear to cut.
                    let mut move_beta = probcut_beta;
                    if score >= probcut_beta {
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            crate::diag_count!(probcut_qpass);
                        }
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            if base_depth == 0 {
                                crate::diag_count!(probcut_qpass_base0);
                            }
                            if probcut_depth > 0 {
                                crate::diag_count!(probcut_verify);
                            }
                            if probcut_depth > 0 && probcut_depth < base_depth {
                                crate::diag_count!(probcut_verify_raised);
                            }
                        }
                        if probcut_depth > 0 {
                            let adjusted = (probcut_beta
                                + self.cfg.proof.probcut_adjust * (base_depth - probcut_depth))
                                .min(INF_SCORE);
                            score = -self.negamax::<NonPv, _>(
                                board,
                                probcut_depth,
                                -adjusted,
                                -adjusted + 1,
                                ply + 1,
                                false,
                                Move::NULL,
                                true,
                                last_critical_ply,
                                poll,
                            );
                            if score < adjusted && probcut_beta < adjusted {
                                #[cfg(feature = "diag")]
                                if diag_sample {
                                    crate::diag_count!(probcut_deeper_research);
                                }
                                probcut_depth = base_depth;
                                score = -self.negamax::<NonPv, _>(
                                    board,
                                    base_depth,
                                    -probcut_beta,
                                    -probcut_beta + 1,
                                    ply + 1,
                                    false,
                                    Move::NULL,
                                    true,
                                    last_critical_ply,
                                    poll,
                                );
                            } else {
                                move_beta = adjusted;
                            }
                        }
                    }
                    board.unmake_move(mv);
                    self.clear_move(ply);
                    if self.td.stopped || self.td.quit {
                        return 0;
                    }
                    #[cfg(feature = "diag")]
                    if diag_sample
                        && qsearch_score >= probcut_beta
                        && probcut_depth > 0
                        && score < move_beta
                    {
                        crate::diag_count!(probcut_verify_fail);
                    }
                    if score >= move_beta {
                        crate::diag_count!(probcut_cut);
                        let returned = if is_decisive(score) {
                            score
                        } else {
                            score + (beta - score) * self.cfg.proof.probcut_lerp / 1024
                        };
                        trace_decision!(
                            self,
                            ply,
                            "probcut depth {depth} move {mv} qsearch {qsearch_score} \
                             probcut_depth {probcut_depth} bound {move_beta} score {score} \
                             static {static_eval} returns {returned}"
                        );
                        self.shared.tt.store(TtStore {
                            key: hash,
                            depth: probcut_depth + 1,
                            // The fail-high shifted down by the margin it
                            // cleared: storing the raw fail-high measured
                            // 5.55% slower to depth.
                            score: if is_decisive(score) {
                                score
                            } else {
                                score - (move_beta - beta)
                            },
                            bound: Bound::Lower,
                            mv,
                            ply,
                            static_eval: raw_static_eval,
                            is_pv: tt_pv,
                        });
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            crate::diag_count!(probcut_tt_store);
                        }
                        #[cfg(test)]
                        {
                            self.td.probcut_cuts += 1;
                        }
                        return returned;
                    }
                }
            }
        }

        // Singular extension, decided before the move loop so a TT move the
        // exclusion search beat can lose its first slot. The TT move is
        // searched against a margin below its stored score with itself
        // excluded. Everything failing low marks it singular: it extends one
        // to three plies. With the singular beta at or above beta, the
        // exclusion fail-high is a multi-cut and returns a softened score.
        // One that beat the stored score demotes the TT move to the ordinary
        // picker order. Otherwise a stored score at beta, or a cut node,
        // reduces it three plies, keeping a one-ply child. With no singular
        // candidate, a shallow cut node well below alpha extends its first
        // move by one instead.
        #[cfg(feature = "b3proof")]
        let mut node_extension = 0;
        #[cfg(feature = "b3proof")]
        let mut singular_score: Option<i32> = None;
        #[cfg(feature = "b3proof")]
        if !NODE::ROOT && excluded.is_null() && potential_singularity && !tt_move.is_null() {
            #[cfg(feature = "diag")]
            if diag_sample {
                crate::diag_count!(singular_attempt);
            }
            #[cfg(feature = "diag")]
            if diag_sample && cut_node {
                crate::diag_count!(singular_attempt_cut_node);
            }
            let k = self.cfg.proof.singular_margin;
            let span = if ev.bound == Some(Bound::Exact) {
                (depth + 3) / 4
            } else {
                depth
            };
            let margin = span * k + depth * k * i32::from(tt_pv && !NODE::PV);
            let singular_beta = ev.score - margin;
            let singular_depth = (depth - 1) / 2;
            #[cfg(test)]
            {
                self.td.singular_searches += 1;
            }
            let score = self.negamax::<NonPv, _>(
                board,
                singular_depth,
                singular_beta - 1,
                singular_beta,
                ply,
                false,
                tt_move,
                cut_node,
                ply,
                poll,
            );
            // The exclusion search ran at this ply and overwrote it.
            self.td.stack[ply].tt_pv = tt_pv;
            if self.td.stopped || self.td.quit {
                return 0;
            }
            singular_score = Some(score);
            trace_decision!(
                self,
                ply,
                "singular depth {depth} move {tt_move} tt_score {} margin {margin} \
                 singular_beta {singular_beta} score {score} beta {beta} cut_node {cut_node}",
                ev.score
            );
            if score < singular_beta {
                node_extension = self.singular_extension(
                    singular_beta - score,
                    NODE::PV,
                    ev.pv_line(false),
                    !is_noisy(tt_move),
                    corr_abs,
                );
                node_extension = self.within_extension_budget(node_extension, extension_spent);
                #[cfg(feature = "diag")]
                if diag_sample && node_extension > 0 {
                    match node_extension {
                        1 => crate::diag_count!(singular_extend_one),
                        2 => crate::diag_count!(singular_extend_two),
                        _ => crate::diag_count!(singular_extend_three),
                    }
                    if !NODE::PV {
                        crate::diag_count!(singular_extend_non_pv);
                    }
                }
            } else if singular_beta >= beta
                && let Some(cut) = multicut_score(score, beta, self.cfg.proof.sing_multicut_lerp)
            {
                // Only a window at or above beta proves the cut: an exclusion
                // fail-high below beta is a fail-soft guess, and taking it
                // measured worse at driving a won endgame.
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(singular_multicut);
                }
                trace_decision!(
                    self,
                    ply,
                    "multicut score {score} beta {beta} returns {cut}"
                );
                return cut;
            } else if score > ev.score && !is_decisive(score) {
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(singular_ttmove_demoted);
                }
                tt_move = Move::NULL;
            } else if ev.score >= beta || cut_node {
                // Three plies, but never below one ply of main search: from
                // depth 4 the TT move keeps a one-ply child.
                node_extension = (-3).max(2 - depth);
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(singular_negative_extension);
                    if node_extension > -3 {
                        crate::diag_count!(singular_negative_clamped);
                    }
                }
            }
        } else if !NODE::ROOT
            && !self.ablated(6)
            && self.ldse_applies(depth, in_check, cut_node, eval_for_pruning, alpha)
        {
            node_extension = self.within_extension_budget(1, extension_spent);
            #[cfg(feature = "diag")]
            if diag_sample && node_extension > 0 {
                crate::diag_count!(ldse_applied);
            }
            trace_decision!(
                self,
                ply,
                "ldse depth {depth} estimated {eval_for_pruning} alpha {alpha} margin {} \
                 spent {extension_spent} granted {node_extension}",
                self.cfg.proof.ldse_margin
            );
        }
        #[cfg(feature = "b3proof")]
        let mut tt_move_score: Option<i32> = None;
        #[cfg(all(test, feature = "b3proof"))]
        if node_extension != 0 {
            self.td.extended_nodes += 1;
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
            // Order the root list from the pool's view: a move another thread
            // already proved at a deeper depth goes first. The rotation below
            // diversifies on top of it.
            if NODE::ROOT && scored.len() > 1 {
                // No-op serially: with no shared state there are no pool
                // scores to fold in.
                self.apply_shared_root_scores(&mut scored);
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
        // Per-node check masks, built at most once and shared by every move's
        // check test and the make-move hint; unmaking restores the board each
        // iteration, so they stay valid for the whole loop.
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
            // The node's extension belongs to its first move: the TT move
            // when it has one and kept its slot.
            #[cfg(feature = "b3proof")]
            let extension = if move_count == 1 { node_extension } else { 0 };
            #[cfg(not(feature = "b3proof"))]
            let mut extension = 0;
            #[cfg(not(feature = "b3proof"))]
            {
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
                            && singular_score
                                < singular_beta - self.cfg.params.singular_double_margin
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
            }

            self.push_move(board, ply, mv, moving_piece);
            self.record_move_order(ply, move_count);
            #[cfg(feature = "b3proof")]
            {
                self.td.stack[ply].extension_spent = extension_spent + extension.max(0);
            }
            let nodes_before_move = if NODE::ROOT { self.td.nodes } else { 0 };
            // The check predicate is cheap here (node masks and two bitboard
            // tests) and lets `make_move` skip `calculate_checkers` for the
            // common non-checking move.
            let mv_gives_check = move_gives_check(board, &mut node_ci, mv, &mut gives_check);
            board.make_move_with_check(mv, mv_gives_check);
            self.shared.tt.prefetch(board.hash());
            let mut new_depth = depth - 1 + extension;
            #[cfg(feature = "b3proof")]
            debug_assert!(
                (-3..=3).contains(&extension) && (extension == 0 || new_depth >= 1),
                "extension {extension} at depth {depth}"
            );
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
                } else if self.cfg.core.lmr_full_depth != 0
                    && !NODE::ROOT
                    && !in_check
                    && !self.ablated(7)
                {
                    // The donor's full-depth branch. Late-move reductions take
                    // every later move from depth 2, and at depth 1 the one-ply
                    // floor leaves nothing to take, so the moves it reaches are
                    // the first moves of non-PV nodes. A fail-high on the
                    // shortened search is verified at `new_depth`.
                    let tt_valid = ev.bound.is_some();
                    let reduction = self.full_depth_reduction(&FullDepthInputs {
                        depth,
                        improvement,
                        corr_abs,
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
                        tt_pv,
                        tt_deep: tt_valid && ev.depth >= depth,
                        cut_node,
                        tt_move_null: tt_move.is_null(),
                        is_tt_move: mv == tt_move,
                        child_cutoffs: self.td.stack[ply + 1].cutoff_count,
                        parent_reduction: self.td.stack.back(ply, 1).reduction,
                    });
                    let searched_depth = full_depth_searched_depth(new_depth, reduction);
                    trace_decision!(
                        self,
                        ply,
                        "fds depth {depth} move {mv} units {reduction} new_depth {new_depth} \
                         searched_depth {searched_depth} window {alpha} {beta}"
                    );
                    let mut first = -self.negamax::<NonPv, _>(
                        board,
                        searched_depth,
                        -beta,
                        -alpha,
                        ply + 1,
                        true,
                        Move::NULL,
                        !cut_node,
                        last_critical_ply,
                        poll,
                    );
                    if first > alpha && searched_depth < new_depth {
                        search_count += 1;
                        first = -self.negamax::<NonPv, _>(
                            board,
                            new_depth,
                            -beta,
                            -alpha,
                            ply + 1,
                            true,
                            Move::NULL,
                            !cut_node,
                            last_critical_ply,
                            poll,
                        );
                    }
                    first
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
                // 2; at the root and in check only under `CoreLmrCheckRoot`.
                if !self.ablated(7)
                    && (self.cfg.core.lmr_check_root != 0 || (!NODE::ROOT && !in_check))
                    && depth >= 2
                {
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
                        #[cfg(feature = "b3proof")]
                        singular_gap: tt_move_score
                            .zip(singular_score)
                            .map(|(tt_score, excluded_score)| tt_score - excluded_score),
                    });
                    #[cfg(all(feature = "b3proof", feature = "diag"))]
                    if let Some((tt_score, excluded_score)) = tt_move_score.zip(singular_score) {
                        trace_decision!(
                            self,
                            ply,
                            "lmr_singular move {mv} tt_move_score {tt_score} \
                             singular_score {excluded_score} term {}",
                            self.lmr_singular_term(Some(tt_score - excluded_score))
                        );
                    }
                    let reduced_depth = reduced_depth(new_depth, reduction, NODE::PV);
                    crate::diag_count!(lmr_applied);
                    #[cfg(test)]
                    {
                        self.td.lmr_at_root += u64::from(NODE::ROOT);
                        self.td.lmr_in_check += u64::from(in_check);
                    }
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
                        // shallower one. Not at the root, as the donor does.
                        let deeper =
                            !NODE::ROOT && score > best_score + self.cfg.core.lmr_research_deeper;
                        let shallower = !NODE::ROOT
                            && score < best_score + self.cfg.core.lmr_research_shallower;
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
            #[cfg(feature = "b3proof")]
            if mv == tt_move {
                tt_move_score = Some(score);
            }

            let move_nodes = if NODE::ROOT {
                self.td.nodes.saturating_sub(nodes_before_move)
            } else {
                0
            };
            searched += 1;
            // Publish every searched root move to the pool with its bound: a
            // cutoff is Lower, a raised alpha Exact, a fail-low Upper (alpha is
            // not yet updated here).
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
                        // `searched` already counts this move, so 1 means the
                        // first move failed high; the denominator is
                        // cutoff_quiet + cutoff_capture from this block.
                        #[cfg(feature = "diag")]
                        if searched == 1 {
                            crate::diag_count!(cutoff_first_move);
                        }
                        // Cutoff rank over the same denominator: the buckets
                        // sum to it and best_rank_1 equals cutoff_first_move.
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
                        // A later MultiPV line's root result is not the
                        // position's: it excludes the better moves.
                        if !(NODE::ROOT && self.td.multipv_line > 0) {
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
                        }
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
                    if !in_check
                        && !is_noisy(mv)
                        && score > static_eval
                        && self.admits_correction_training(score, excluded)
                    {
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

        // An exclusion search whose only legal move is the excluded one
        // proves nothing about the other moves: it fails low by exactly one,
        // so a forced move is singular by one ply, never by three. Scoring it
        // as mate or stalemate let forced lines extend without bound.
        #[cfg(feature = "b3proof")]
        if !legal_move_seen && !excluded.is_null() {
            crate::diag_count!(singular_lone_move);
            return alpha;
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
            && self.admits_correction_training(best_score, excluded)
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
            if !(NODE::ROOT && self.td.multipv_line > 0) {
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
            }
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
        self.td.seldepth = self.td.seldepth.max(ply + 1);

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
        // The two prune exits below are fail-soft: they report the stand pat,
        // which is below the window, rather than alpha. The tail still stores
        // alpha, so the depth-0 upper bound the main search refines its
        // estimate with is unchanged; a fully fail-soft tail measured 17.2%
        // more nodes.
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
            // Refine the stand pat with a bound-consistent TT score, as the
            // main search refines its estimate.
            let stand_pat = ev.refine_eval(stand_pat, 0);
            stand_pat_for_pruning = stand_pat;
            if stand_pat >= beta {
                #[cfg(feature = "diag")]
                if diag_q_sample {
                    crate::diag_count!(q_stand_pat_cut);
                    crate::diag_count!(q_stand_pat_store);
                }
                // A stand-pat fail-high is stored as a depth-0 lower bound.
                // Suppressing these stores measured worse: TT cutoffs fell
                // faster than the tree shrank.
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
        // How many evasions each in-check qnode scores.
        #[cfg(feature = "diag")]
        if in_check {
            crate::diag_count!(q_check_nodes);
            crate::diag_add!(
                q_check_moves_scored,
                u64::try_from(scored.len()).unwrap_or(u64::MAX)
            );
        }
        // Per-node check masks, built lazily as in the main search; a
        // capture-only qnode that never tests for check never builds them.
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

    /// A quiet, non-PV late move with every optional term neutral.
    fn quiet_late_move() -> LateMoveInputs {
        LateMoveInputs {
            depth: 8,
            improvement: 0,
            corr_abs: 0,
            exact: false,
            tt_score_below_alpha: false,
            tt_score_above_alpha: false,
            tt_shallow: false,
            tt_deep: false,
            win_beta: false,
            is_quiet: true,
            history: 0,
            alpha_gap: 0,
            critical_distance: 0,
            pv: false,
            window: 1,
            laterality: 0,
            tt_pv: false,
            cut_node: false,
            tt_move_null: false,
            gives_check: false,
            child_cutoffs: 0,
            parent_reduction: 0,
            #[cfg(feature = "b3proof")]
            singular_gap: None,
        }
    }

    /// The improvement term saturates at its coordinates' bounds and the alpha
    /// gap at its own; moving a coordinate moves the saturation point. The
    /// bounds are read from the parameters, not written here: they are SPSA
    /// coordinates, so a test that pinned their values would fail on every
    /// fit while proving nothing about the saturation it exists to check.
    #[test]
    fn late_move_reduction_clamps_follow_their_coordinates() {
        let mut searcher = Searcher::default();
        let p = &searcher.cfg.core;
        let (clamp_lo, clamp_hi) = (p.lmr_improvement_clamp_lo, p.lmr_improvement_clamp_hi);
        let (gap_lo, gap_hi) = (p.lmr_alpha_gap_lo, p.lmr_alpha_gap_hi);
        assert!(clamp_lo < 0 && clamp_hi > 0 && gap_lo < 0 && gap_hi > 0);

        let reductions = |searcher: &Searcher| {
            let with = |edit: &dyn Fn(&mut LateMoveInputs)| {
                let mut inputs = quiet_late_move();
                edit(&mut inputs);
                searcher.late_move_reduction(&inputs)
            };
            let neutral = with(&|_| {});
            (
                neutral - with(&|i| i.improvement = 10_000),
                neutral - with(&|i| i.improvement = -10_000),
                with(&|i| i.alpha_gap = 1_000) - neutral,
                with(&|i| i.alpha_gap = -1_000) - neutral,
            )
        };
        let gap = searcher.cfg.core.lmr_alpha_gap;
        assert_eq!(
            reductions(&searcher),
            (clamp_hi, clamp_lo, gap * gap_hi / 128, gap * gap_lo / 128)
        );
        // Below saturation the term is linear in the improvement.
        let slope = searcher.cfg.core.lmr_improvement;
        let with_improvement = |searcher: &Searcher, improvement: i32| {
            let mut inputs = quiet_late_move();
            let neutral = searcher.late_move_reduction(&inputs);
            inputs.improvement = improvement;
            neutral - searcher.late_move_reduction(&inputs)
        };
        assert_eq!(with_improvement(&searcher, 128), slope);

        searcher.cfg.core.lmr_improvement_clamp_lo = -110;
        searcher.cfg.core.lmr_improvement_clamp_hi = 528;
        searcher.cfg.core.lmr_alpha_gap_lo = -30;
        searcher.cfg.core.lmr_alpha_gap_hi = 42;
        assert_eq!(
            reductions(&searcher),
            (528, -110, gap * 42 / 128, gap * -30 / 128)
        );
    }

    /// A non-PV node whose table entry carries the PV bit, far below alpha
    /// with no TT move and a quiet mate in one. Without the guards it razors
    /// and quiescence, which does not play the quiet mate, returns a score
    /// below alpha; with them the node searches and finds the mate.
    #[test]
    fn razor_guards_keep_a_tt_pv_node_out_of_razoring() {
        // White is a queen and a rook down; Ra8 is mate on the back rank.
        let fen = "6k1/5ppp/7q/7r/8/8/1P6/R3K3 w - - 0 1";
        let search = |guards: i32| {
            let mut searcher = Searcher::default();
            searcher.cfg.core.razor_guards = guards;
            // The position evaluates near -927; the margin is pinned so the
            // fixture stays below it whatever the fitted defaults become.
            searcher.cfg.core.razor_base = 288;
            searcher.cfg.core.razor_square = 143;
            let mut board = Board::from_fen(fen).expect("valid FEN");
            searcher.shared.tt.store(TtStore {
                key: board.hash(),
                depth: 1,
                score: -2_000,
                bound: Bound::Upper,
                mv: Move::NULL,
                ply: 1,
                static_eval: VALUE_NONE,
                is_pv: true,
            });
            let margin = searcher.cfg.core.razor_base + searcher.cfg.core.razor_square * 4;
            assert!(
                searcher.corrected_eval(&board, 1) < -margin,
                "the node must sit below the razoring margin"
            );
            searcher.negamax::<NonPv, _>(
                &mut board,
                2,
                0,
                1,
                1,
                false,
                Move::NULL,
                false,
                1,
                &mut || SearchEvent::None,
            )
        };
        assert!(
            search(0) <= 0,
            "guards 0 razors the tt_pv node into quiescence"
        );
        assert!(
            search(1) >= MATE_SCORE - 3,
            "guards 1 searches the tt_pv node and finds the mate"
        );
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

    /// Sets one categorical switch on a searcher.
    type SetSwitch = fn(&mut Searcher, i32);

    /// A quiet mate in one searched at depth 1 with a window above the static
    /// eval and above the razoring limit: the mate fails high and is the
    /// node's only training event. At
    /// `CoreCorrTrainDecisive = 0` every correction table stays zero; at 1
    /// the mate residual is trained.
    #[test]
    fn corr_train_decisive_zero_leaves_every_table_untouched_after_a_mate() {
        let search = |decisive: i32| {
            let mut searcher = Searcher::default();
            searcher.cfg.core.corr_train_decisive = decisive;
            let mut board =
                Board::from_fen("6k1/5ppp/8/8/8/8/1P6/R3K3 w - - 0 1").expect("valid FEN");
            assert!(searcher.td.corr.untouched());
            let score = searcher.negamax::<NonPv, _>(
                &mut board,
                1,
                1_000,
                1_001,
                1,
                false,
                Move::NULL,
                false,
                1,
                &mut || SearchEvent::None,
            );
            assert!(score >= TB_WIN_SCORE, "the mate is found: {score}");
            searcher.td.corr.untouched()
        };
        assert!(search(0), "a decisive residual trained a table at 0");
        assert!(!search(1), "the mate residual is trained at 1");
    }

    /// The same node searched as a singular-exclusion search (a non-mating
    /// pawn move excluded): at `CoreCorrTrainExcluded = 0` it trains nothing;
    /// at 1 the fail-high trains the correction.
    #[test]
    fn corr_train_excluded_zero_trains_nothing_at_a_singular_search() {
        let search = |admit: i32| {
            let mut searcher = Searcher::default();
            searcher.cfg.core.corr_train_excluded = admit;
            let mut board =
                Board::from_fen("6k1/5ppp/8/8/8/8/1P6/R3K3 w - - 0 1").expect("valid FEN");
            let excluded = board.parse_move("b2b4").expect("legal pawn move");
            let score = searcher.negamax::<NonPv, _>(
                &mut board,
                1,
                1_000,
                1_001,
                1,
                false,
                excluded,
                false,
                1,
                &mut || SearchEvent::None,
            );
            assert!(score > 1_000, "the node fails high: {score}");
            searcher.td.corr.untouched()
        };
        assert!(search(0), "a singular-exclusion node trained a table at 0");
        assert!(!search(1), "the singular-exclusion node trains at 1");
    }

    /// A cut-node first move without a TT move, quiet, at depth 8.
    fn cut_node_first_move() -> FullDepthInputs {
        FullDepthInputs {
            depth: 8,
            improvement: 0,
            corr_abs: 0,
            is_quiet: true,
            history: 0,
            tt_pv: false,
            tt_deep: false,
            cut_node: true,
            tt_move_null: true,
            is_tt_move: false,
            child_cutoffs: 0,
            parent_reduction: 0,
        }
    }

    /// Under `CoreLmrFullDepth` a non-PV first move is searched below
    /// `new_depth` when its reduction reaches 2621 units (two plies below
    /// from 5579) and at `new_depth` otherwise; the one-ply floor never
    /// deepens a move.
    #[test]
    fn full_depth_branch_searches_a_first_move_below_new_depth_from_2621_units() {
        assert_eq!(full_depth_searched_depth(8, 2_620), 8);
        assert_eq!(full_depth_searched_depth(8, 2_621), 7);
        assert_eq!(full_depth_searched_depth(8, 5_578), 7);
        assert_eq!(full_depth_searched_depth(8, 5_579), 6);
        assert_eq!(full_depth_searched_depth(2, 9_000), 1, "one-ply floor");
        assert_eq!(full_depth_searched_depth(1, 9_000), 1, "one-ply floor");
        assert_eq!(full_depth_searched_depth(0, 9_000), 0, "never deeper");
        assert_eq!(full_depth_searched_depth(7, -9_000), 7, "never deeper");

        let searcher = Searcher::default();
        let reduction = |edit: &dyn Fn(&mut FullDepthInputs)| {
            let mut inputs = cut_node_first_move();
            edit(&mut inputs);
            searcher.full_depth_reduction(&inputs)
        };
        // 207*3 + 1468 + 1260 + 2168 - 56 = 5461: one ply off.
        assert_eq!(reduction(&|_| {}), 5_461);
        assert_eq!(full_depth_searched_depth(9, reduction(&|_| {})), 8);
        // Children that keep failing high push it past 5579: two plies.
        let cutoffs = reduction(&|i| i.child_cutoffs = 3);
        assert_eq!(cutoffs, 5_461 + 1_394);
        assert_eq!(full_depth_searched_depth(9, cutoffs), 7);
        // The TT move at a cut node that has one stays at full depth.
        let tt_move = reduction(&|i| {
            i.tt_move_null = false;
            i.is_tt_move = true;
        });
        assert_eq!(tt_move, 5_461 - 2_168 - 3_002);
        assert_eq!(full_depth_searched_depth(9, tt_move), 9);
        // The improvement clamp is in Rarog's evaluation units.
        assert_eq!(
            reduction(&|_| {}) - reduction(&|i| i.improvement = 10_000),
            626
        );
        assert_eq!(
            reduction(&|_| {}) - reduction(&|i| i.improvement = -10_000),
            -94
        );
    }

    /// `CoreLmrCheckRoot` widens late-move reductions to the root and to
    /// nodes in check: from a quiet root and from a root in check, a late
    /// move is reduced at both kinds of node at 1 and at neither at 0.
    #[test]
    fn lmr_check_root_reduces_at_the_root_and_in_check_only_when_set() {
        let run = |scope: i32, fen: &str| {
            let mut searcher = Searcher::default();
            searcher.cfg.core.lmr_check_root = scope;
            let mut board = Board::from_fen(fen).expect("valid FEN");
            let score =
                searcher.search_root_window(&mut board, 7, -INF_SCORE, INF_SCORE, &mut || {
                    SearchEvent::None
                });
            assert!(score.abs() < INF_SCORE);
            (searcher.td.lmr_at_root, searcher.td.lmr_in_check)
        };
        let quiet = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let checked = "rnbqk1nr/pppp1ppp/8/4p3/1b1PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3";
        for fen in [quiet, checked] {
            assert_eq!(
                run(0, fen),
                (0, 0),
                "no reduction at the root or in check at 0: {fen}"
            );
        }
        let (root, _) = run(1, quiet);
        assert!(root > 0, "the root reduces a late move at 1");
        let (root_in_check, in_check) = run(1, checked);
        assert!(
            root_in_check > 0,
            "a root in check reduces a late move at 1"
        );
        assert!(in_check > 0, "a node in check reduces a late move at 1");
    }

    /// The categorical switches, each settable to 0 or 1 on a searcher.
    const SWITCHES: &[(&str, SetSwitch)] = &[
        ("CoreRazorGuards", |s, v| {
            s.cfg.core.razor_guards = v;
        }),
        ("CoreCorrTrainDecisive", |s, v| {
            s.cfg.core.corr_train_decisive = v;
        }),
        ("CoreCorrTrainExcluded", |s, v| {
            s.cfg.core.corr_train_excluded = v;
        }),
        ("CoreLmrFullDepth", |s, v| {
            s.cfg.core.lmr_full_depth = v;
        }),
        ("CoreLmrCheckRoot", |s, v| {
            s.cfg.core.lmr_check_root = v;
        }),
    ];

    /// The proof-search arm's categorical switches.
    #[cfg(feature = "b3proof")]
    const PROOF_SWITCHES: &[(&str, SetSwitch)] = &[
        ("CoreNmpNodes", |s, v| {
            s.cfg.proof.nmp_nodes = v;
        }),
        ("CoreProbcutNodes", |s, v| {
            s.cfg.proof.probcut_nodes = v;
        }),
        ("CoreProbcutTtServed", |s, v| {
            s.cfg.proof.probcut_tt_served = v;
        }),
        ("CoreSingularFloor", |s, v| {
            s.cfg.proof.singular_floor = v;
        }),
    ];
    #[cfg(not(feature = "b3proof"))]
    const PROOF_SWITCHES: &[(&str, SetSwitch)] = &[];

    /// Every switch at both values leaves no reduction on the stack after
    /// the search unwinds and a root PV made of legal moves, from a quiet
    /// middlegame root and from a root in check.
    #[test]
    fn switches_keep_reductions_unwinding_and_the_pv_legal() {
        let roots = [
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            "rnbqk1nr/pppp1ppp/8/4p3/1b1PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3",
        ];
        for &(name, set) in SWITCHES.iter().chain(PROOF_SWITCHES) {
            for value in 0..=1 {
                for fen in roots {
                    let mut searcher = Searcher::default();
                    set(&mut searcher, value);
                    let root = Board::from_fen(fen).expect("valid FEN");
                    let mut board = root.clone();
                    let score = searcher.search_root_window(
                        &mut board,
                        7,
                        -INF_SCORE,
                        INF_SCORE,
                        &mut || SearchEvent::None,
                    );
                    let context = format!("{name}={value} at {fen}");
                    assert!(score.abs() < INF_SCORE, "{context}");
                    assert_eq!(board.to_fen(), root.to_fen(), "{context}");
                    for ply in 0..MAX_PLY {
                        assert_eq!(searcher.td.stack[ply].reduction, 0, "{context}, ply {ply}");
                    }
                    let mut line = root.clone();
                    let pv_len = searcher.td.pv_len[0].min(MAX_PLY);
                    assert!(pv_len > 0, "{context}: empty root PV");
                    for &mv in &searcher.td.pv_table[0][..pv_len] {
                        let legal = line
                            .parse_move(&mv.to_string())
                            .unwrap_or_else(|| panic!("{context}: illegal PV move {mv}"));
                        line.make_move(legal);
                    }
                }
            }
        }
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

    /// White a queen, two rooks and more ahead, nothing hanging either way.
    #[cfg(feature = "b3proof")]
    const WHITE_FAR_AHEAD: &str = "4k3/pppp4/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1";

    /// How a probe node is searched: its type and role, and the searcher
    /// adjustment made before it.
    #[cfg(feature = "b3proof")]
    struct NullProbe {
        fen: &'static str,
        pv: bool,
        cut_node: bool,
        allow_null: bool,
        excluded: Option<&'static str>,
        setup: fn(&mut Searcher),
    }

    #[cfg(feature = "b3proof")]
    impl NullProbe {
        fn new() -> Self {
            Self {
                fen: WHITE_FAR_AHEAD,
                pv: false,
                cut_node: true,
                allow_null: true,
                excluded: None,
                setup: |_| {},
            }
        }

        /// Search the node at ply 1, depth 8, with beta 300 below its
        /// estimate: inside the null move's margin and short of reverse
        /// futility's. Whether a null move was made at ply 1.
        fn null_move_at_node(&self) -> bool {
            let mut searcher = Searcher::default();
            (self.setup)(&mut searcher);
            let mut board = Board::from_fen(self.fen).expect("valid FEN");
            let beta = searcher.corrected_eval(&board, 1) - 300;
            let excluded = self
                .excluded
                .map_or(Move::NULL, |mv| board.parse_move(mv).expect("legal move"));
            let (depth, ply) = (8, 1);
            let score = if self.pv {
                searcher.negamax::<Pv, _>(
                    &mut board,
                    depth,
                    beta - 1,
                    beta,
                    ply,
                    self.allow_null,
                    excluded,
                    self.cut_node,
                    ply,
                    &mut || SearchEvent::None,
                )
            } else {
                searcher.negamax::<NonPv, _>(
                    &mut board,
                    depth,
                    beta - 1,
                    beta,
                    ply,
                    self.allow_null,
                    excluded,
                    self.cut_node,
                    ply,
                    &mut || SearchEvent::None,
                )
            };
            assert!(score.abs() < INF_SCORE);
            searcher.td.null_move_plies.contains(&1)
        }
    }

    /// The null move's gates, each from a node that takes the null move
    /// with the gate open: the population switch, the PV line, an excluded
    /// move, a verification region above the node, a node in check and the
    /// consecutive-null guard.
    #[cfg(feature = "b3proof")]
    #[test]
    fn null_move_keeps_to_its_population_and_out_of_regions() {
        let base = NullProbe::new;
        assert!(
            base().null_move_at_node(),
            "the probe node must take the null move"
        );
        assert!(
            NullProbe {
                cut_node: false,
                ..base()
            }
            .null_move_at_node(),
            "an all node off the PV line takes it at CoreNmpNodes = 0"
        );
        let cut_nodes_only: fn(&mut Searcher) = |s| s.cfg.proof.nmp_nodes = 1;
        assert!(
            NullProbe {
                setup: cut_nodes_only,
                ..base()
            }
            .null_move_at_node(),
            "a cut node takes it at CoreNmpNodes = 1"
        );
        assert!(
            !NullProbe {
                cut_node: false,
                setup: cut_nodes_only,
                ..base()
            }
            .null_move_at_node(),
            "an all node does not at CoreNmpNodes = 1"
        );
        for setup in [|_: &mut Searcher| {}, cut_nodes_only] {
            assert!(
                !NullProbe {
                    pv: true,
                    setup,
                    ..base()
                }
                .null_move_at_node(),
                "a PV node never takes it"
            );
        }
        assert!(
            !NullProbe {
                excluded: Some("a2a3"),
                ..base()
            }
            .null_move_at_node(),
            "a singular-exclusion node never takes it"
        );
        assert!(
            !NullProbe {
                setup: |s| s.td.nmp_min_ply = 5,
                ..base()
            }
            .null_move_at_node(),
            "a node above a verification region's end never takes it"
        );
        assert!(
            !NullProbe {
                allow_null: false,
                ..base()
            }
            .null_move_at_node(),
            "the reply to a null move never takes one"
        );
        assert!(
            !NullProbe {
                fen: "4k3/pppp4/8/8/8/8/PPPPrPPP/RNBQKBNR w KQ - 0 1",
                ..base()
            }
            .null_move_at_node(),
            "a node in check never takes it"
        );
    }

    /// The root never takes a null move, at either population.
    #[cfg(feature = "b3proof")]
    #[test]
    fn the_root_never_takes_a_null_move() {
        for nodes in 0..=1 {
            let mut searcher = Searcher::default();
            searcher.cfg.proof.nmp_nodes = nodes;
            let mut board = Board::from_fen(
                "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            )
            .expect("valid FEN");
            for depth in 1..=10 {
                let score = searcher.search_root_window(
                    &mut board,
                    depth,
                    -INF_SCORE,
                    INF_SCORE,
                    &mut || SearchEvent::None,
                );
                assert!(score.abs() < INF_SCORE);
            }
            assert!(
                !searcher.td.null_move_plies.is_empty(),
                "the tree must try null moves"
            );
            assert!(
                !searcher.td.null_move_plies.contains(&0),
                "CoreNmpNodes = {nodes}"
            );
        }
    }

    /// With verification from depth 4, a middlegame search verifies null
    /// moves; the region is cleared after every one, and a new search starts
    /// with none.
    #[cfg(feature = "b3proof")]
    #[test]
    fn the_verification_region_is_cleared_after_verifying_and_at_search_start() {
        let mut searcher = Searcher::default();
        searcher.cfg.proof.nmp_verify_depth = 4;
        let mut board =
            Board::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4")
                .expect("valid FEN");
        let score = searcher.search_root_window(&mut board, 9, -INF_SCORE, INF_SCORE, &mut || {
            SearchEvent::None
        });
        assert!(score.abs() < INF_SCORE);
        assert!(searcher.td.nmp_verifications > 0, "the search must verify");
        assert_eq!(searcher.td.nmp_min_ply, 0);

        searcher.td.nmp_min_ply = 7;
        let mut options = crate::search_options::EngineOptions::default();
        options.proof_params.nmp_verify_depth = 4;
        searcher.reset_search_state(
            &crate::search_options::SearchLimits::default(),
            &options,
            board.side_to_move(),
            0,
            true,
            false,
        );
        assert_eq!(
            searcher.td.nmp_min_ply, 0,
            "a search starts outside any region"
        );
    }

    /// A null-move cutoff returns at least beta and never a win the reduced
    /// search did not prove.
    #[cfg(feature = "b3proof")]
    #[test]
    fn a_null_cutoff_returns_neither_a_win_nor_less_than_beta() {
        assert_eq!(null_cutoff_score(150, 100), 150);
        assert_eq!(
            null_cutoff_score(80, 100),
            100,
            "a verified lower-bound fail-high"
        );
        assert_eq!(null_cutoff_score(TB_WIN_SCORE, 100), 100);
        assert_eq!(null_cutoff_score(MATE_SCORE - 3, 100), 100);
        assert_eq!(null_cutoff_score(-50, -60), -50);
    }

    /// The margin sits above beta, never below 2, and each term moves it in
    /// its stated direction; the reduction grows with depth and surplus up to
    /// the clamp.
    #[cfg(feature = "b3proof")]
    #[test]
    fn null_move_margin_and_reduction_follow_their_coordinates() {
        let searcher = Searcher::default();
        let p = &searcher.cfg.proof;
        let margin = |depth, tt_pv, improvement, cutoffs| {
            searcher.nmp_margin(depth, tt_pv, improvement, cutoffs)
        };
        assert_eq!(margin(8, false, 0, 2), p.nmp_base - 8 * p.nmp_depth);
        assert_eq!(margin(8, true, 0, 2) - margin(8, false, 0, 2), p.nmp_tt_pv);
        assert_eq!(
            margin(8, false, 0, 2) - margin(8, false, 0, 1),
            p.nmp_cutoff
        );
        assert!(margin(8, false, 1024, 2) < margin(8, false, 0, 2));
        assert_eq!(margin(8, false, 1_000_000, 0), 2, "floored at 2");

        let reduction =
            |depth, improving, surplus| searcher.nmp_reduction(depth, improving, surplus);
        assert!(reduction(12, false, 0) >= reduction(6, false, 0));
        assert!(reduction(8, true, 0) >= reduction(8, false, 0));
        assert_eq!(
            reduction(8, false, -500),
            reduction(8, false, 0),
            "no negative surplus"
        );
        assert_eq!(
            reduction(8, false, p.nmp_r_clamp),
            reduction(8, false, 10 * p.nmp_r_clamp),
            "the surplus saturates"
        );
    }

    /// White to move can take a hanging queen with a pawn.
    #[cfg(feature = "b3proof")]
    const HANGING_QUEEN: &str = "4k3/8/8/3q4/4P3/8/8/4K3 w - - 0 1";

    /// Search `HANGING_QUEEN` at ply 1, depth 8, at a cut node with the null
    /// move off, beta 100 below the estimate plus `beta_offset`, after an
    /// optional stored entry. Returns the score, beta and the searcher.
    #[cfg(feature = "b3proof")]
    fn probcut_probe(
        setup: fn(&mut Searcher),
        stored: Option<(i32, i32, &str)>,
        beta_offset: i32,
    ) -> (i32, i32, Searcher) {
        let mut searcher = Searcher::default();
        setup(&mut searcher);
        let mut board = Board::from_fen(HANGING_QUEEN).expect("valid FEN");
        let estimate = searcher.corrected_eval(&board, 1);
        let beta = estimate - 100 + beta_offset;
        if let Some((depth, above_beta, mv)) = stored {
            searcher.shared.tt.store(TtStore {
                key: board.hash(),
                depth,
                score: beta + above_beta,
                bound: Bound::Lower,
                mv: board.parse_move(mv).expect("legal move"),
                ply: 1,
                static_eval: VALUE_NONE,
                is_pv: false,
            });
        }
        let score = searcher.negamax::<NonPv, _>(
            &mut board,
            8,
            beta - 1,
            beta,
            1,
            false,
            Move::NULL,
            true,
            1,
            &mut || SearchEvent::None,
        );
        (score, beta, searcher)
    }

    /// Taking the queen cuts the node through ProbCut: the node returns a
    /// score at or above beta and the table holds a lower bound with the
    /// capture, at the verification depth plus one.
    #[cfg(feature = "b3proof")]
    #[test]
    fn a_probcut_cut_stores_a_lower_bound_with_its_move() {
        let (score, beta, searcher) = probcut_probe(|_| {}, None, 0);
        assert_eq!(searcher.td.probcut_cuts, 1, "ProbCut must cut this node");
        let board = Board::from_fen(HANGING_QUEEN).expect("valid FEN");
        assert!(
            score >= beta,
            "a cut returns at least beta: {score} < {beta}"
        );
        let entry = searcher.shared.tt.probe(board.hash());
        let ev = TtProbe::from_entry(entry, 1, board.halfmove_clock());
        assert_eq!(ev.bound, Some(Bound::Lower));
        assert_eq!(ev.mv, board.parse_move("e4d5"));
        assert!(ev.depth >= 1, "stored at the verification depth plus one");
        assert!(ev.score >= beta);
    }

    /// No ProbCut search starts against a decisive beta, and at the cut-node
    /// population none starts when the TT move is quiet; a capture TT move,
    /// or the default population, lets it run.
    #[cfg(feature = "b3proof")]
    #[test]
    fn probcut_refuses_a_decisive_beta_and_a_quiet_tt_move_at_cut_nodes() {
        let (_, _, decisive) = probcut_probe(|_| {}, None, TB_WIN_SCORE + 1_000);
        assert_eq!(decisive.td.probcut_searches, 0, "decisive beta");

        let cut_nodes_only: fn(&mut Searcher) = |s| s.cfg.proof.probcut_nodes = 1;
        let quiet = Some((1, 200, "e1f1"));
        let (_, _, vetoed) = probcut_probe(cut_nodes_only, quiet, 0);
        assert_eq!(vetoed.td.probcut_searches, 0, "quiet TT move at a cut node");
        let (_, _, capture) = probcut_probe(cut_nodes_only, Some((1, 200, "e4d5")), 0);
        assert!(
            capture.td.probcut_searches > 0,
            "capture TT move at a cut node"
        );
        let (_, _, default) = probcut_probe(|_| {}, quiet, 0);
        assert!(
            default.td.probcut_searches > 0,
            "quiet TT move, default population"
        );
    }

    /// Under `CoreProbcutTtServed` a stored lower bound at most four plies
    /// shallower that clears beta by the margin returns `beta + margin`
    /// before any capture is searched; with the switch off the capture
    /// search runs.
    #[cfg(feature = "b3proof")]
    #[test]
    fn a_tt_served_probcut_returns_the_margin_only_when_switched_on() {
        let stored = Some((6, 300, "e1f1"));
        let (score, beta, served) = probcut_probe(|s| s.cfg.proof.probcut_tt_served = 1, stored, 0);
        assert_eq!(score, beta + served.cfg.proof.probcut_tt_margin);
        assert_eq!(served.td.probcut_searches, 0);
        let (_, _, searched) = probcut_probe(|_| {}, stored, 0);
        assert!(searched.td.probcut_searches > 0);
    }

    /// A singular extension is one to three plies for every input and never
    /// shrinks as the exclusion score falls further below the singular beta.
    #[cfg(feature = "b3proof")]
    #[test]
    fn a_singular_extension_is_one_to_three_plies_and_grows_with_the_gap() {
        let searcher = Searcher::default();
        for pv in [false, true] {
            for tt_was_pv in [false, true] {
                for quiet_tt in [false, true] {
                    for corr_abs in [0, 50, 400, 3_000] {
                        let mut last = 1;
                        for below in 1..600 {
                            let extension = searcher
                                .singular_extension(below, pv, tt_was_pv, quiet_tt, corr_abs);
                            assert!((1..=3).contains(&extension));
                            assert!(extension >= last, "shrank at {below}");
                            last = extension;
                        }
                        assert_eq!(last, 3, "a deep enough fail-low extends three plies");
                    }
                }
            }
        }
        // On a PV node the two-ply bar is the PV term plus the new-PV term.
        let p = &searcher.cfg.proof;
        let bar = p.sing_double_pv + p.sing_double_not_tt_pv + p.sing_double_base;
        assert_eq!(searcher.singular_extension(bar, true, false, false, 0), 1);
        assert_eq!(
            searcher.singular_extension(bar + 1, true, false, false, 0),
            2
        );
    }

    /// A multi-cut needs an exclusion fail-high short of the decisive band and
    /// returns a score between beta and it, never decisive, even against a
    /// losing beta.
    #[cfg(feature = "b3proof")]
    #[test]
    fn a_multicut_never_returns_a_decisive_score() {
        assert_eq!(multicut_score(99, 100, 412), None, "below beta");
        assert_eq!(multicut_score(TB_WIN_SCORE, 100, 412), None, "decisive");
        assert_eq!(multicut_score(MATE_SCORE - 5, 100, 412), None, "a mate");
        assert_eq!(multicut_score(1_124, 100, 412), Some(1_124 - 412));
        for beta in [-MATE_SCORE + 10, -TB_WIN_SCORE, -500, 0, 700] {
            for score in [beta, beta + 1, beta + 300, TB_WIN_SCORE - 1] {
                if score < beta || is_decisive(score) {
                    continue;
                }
                let cut = multicut_score(score, beta, 412).expect("a multi-cut");
                assert!(!is_decisive(cut), "{score} against {beta} returned {cut}");
                assert!(cut <= score && cut >= beta.max(-TB_WIN_SCORE + 1));
            }
        }
    }

    /// The low-depth singular extension applies only at an expected cut node
    /// at depth 7 or less, out of check, with the estimate the margin below
    /// alpha.
    #[cfg(feature = "b3proof")]
    #[test]
    fn ldse_applies_only_at_shallow_cut_nodes_well_below_alpha() {
        let searcher = Searcher::default();
        let margin = searcher.cfg.proof.ldse_margin;
        let below = -margin;
        assert!(searcher.ldse_applies(7, false, true, below, 0));
        assert!(searcher.ldse_applies(1, false, true, below - 500, 0));
        assert!(!searcher.ldse_applies(8, false, true, below, 0), "depth 8");
        assert!(!searcher.ldse_applies(7, true, true, below, 0), "in check");
        assert!(
            !searcher.ldse_applies(7, false, false, below, 0),
            "not a cut node"
        );
        assert!(
            !searcher.ldse_applies(7, false, true, below + 1, 0),
            "too close to alpha"
        );
    }

    /// The LMR singular term is zero unless both scores exist and the gap
    /// passes the offset, grows with the gap and stops at the cap; it moves
    /// the late-move reduction by exactly its value.
    #[cfg(feature = "b3proof")]
    #[test]
    fn the_lmr_singular_term_needs_both_scores_and_is_capped() {
        let searcher = Searcher::default();
        let p = &searcher.cfg.proof;
        assert_eq!(searcher.lmr_singular_term(None), 0);
        assert_eq!(searcher.lmr_singular_term(Some(p.lmr_singular_offset)), 0);
        assert_eq!(searcher.lmr_singular_term(Some(-5_000)), 0);
        assert_eq!(
            searcher.lmr_singular_term(Some(p.lmr_singular_offset + 128)),
            p.lmr_singular_slope.min(p.lmr_singular_cap)
        );
        assert_eq!(searcher.lmr_singular_term(Some(60_000)), p.lmr_singular_cap);

        let gap = p.lmr_singular_offset + 100;
        let without = searcher.late_move_reduction(&quiet_late_move());
        let with = searcher.late_move_reduction(&LateMoveInputs {
            singular_gap: Some(gap),
            ..quiet_late_move()
        });
        assert_eq!(with - without, searcher.lmr_singular_term(Some(gap)));
    }

    /// `CoreSingularFloor`: a depth-4 node with a deep enough stored lower
    /// bound and a TT move runs the singular search at 0 (from depth 4) and
    /// not at 1 (from depth 5, 6 on a PV line). At 1 nothing in the probe's
    /// tree can be a candidate: without a singular extension at the probe no
    /// child reaches depth 5.
    #[cfg(feature = "b3proof")]
    #[test]
    fn the_singular_floor_switch_sets_the_least_candidate_depth() {
        let singular_searches_at_depth_4 = |floor: i32| {
            let mut searcher = Searcher::default();
            searcher.cfg.proof.singular_floor = floor;
            let mut board = Board::from_fen(
                "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            )
            .expect("valid FEN");
            let estimate = searcher.corrected_eval(&board, 1);
            // Beta above the stored score keeps reverse futility, the null
            // move, ProbCut and the table cutoff out of the way.
            let beta = estimate + 50;
            searcher.shared.tt.store(TtStore {
                key: board.hash(),
                depth: 4,
                score: estimate,
                bound: Bound::Lower,
                mv: board.parse_move("d2d3").expect("legal move"),
                ply: 1,
                static_eval: VALUE_NONE,
                is_pv: false,
            });
            let score = searcher.negamax::<NonPv, _>(
                &mut board,
                4,
                beta - 1,
                beta,
                1,
                false,
                Move::NULL,
                true,
                1,
                &mut || SearchEvent::None,
            );
            assert!(score.abs() < INF_SCORE);
            searcher.td.singular_searches
        };
        assert!(
            singular_searches_at_depth_4(0) > 0,
            "depth 4 is a candidate at 0"
        );
        assert_eq!(
            singular_searches_at_depth_4(1),
            0,
            "no node of a depth-4 tree is a candidate at 1"
        );
    }

    /// An exclusion search whose only legal move is the excluded one returns
    /// alpha, one below the singular beta, not the mate its empty move list
    /// would otherwise score: the forced evasion `Kxg2` is singular by one.
    #[cfg(feature = "b3proof")]
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
        let score = searcher.negamax::<NonPv, _>(
            &mut board,
            3,
            singular_beta - 1,
            singular_beta,
            1,
            false,
            forced,
            true,
            1,
            &mut || SearchEvent::None,
        );
        assert_eq!(score, singular_beta - 1);
    }

    /// The budget grants a positive extension in full while the line has
    /// room, truncates it at the edge, and grants nothing once the line has
    /// spent the iteration's depth.
    #[cfg(feature = "b3proof")]
    #[test]
    fn the_extension_budget_truncates_at_the_edge_and_then_grants_nothing() {
        let mut searcher = Searcher::default();
        searcher.td.root_depth = 10;
        assert_eq!(searcher.within_extension_budget(3, 0), 3);
        assert_eq!(searcher.within_extension_budget(3, 7), 3);
        assert_eq!(searcher.within_extension_budget(3, 8), 2, "truncated");
        assert_eq!(searcher.within_extension_budget(1, 9), 1);
        assert_eq!(searcher.within_extension_budget(1, 10), 0, "spent");
        assert_eq!(searcher.within_extension_budget(3, 10), 0, "spent");
        assert_eq!(searcher.td.budget_truncations, 3);
    }

    /// Along every line the positive extensions taken from the root never
    /// sum past the iteration's depth: from the two bench positions whose
    /// singular lines once ran to the ply cap, iterative searches extend,
    /// hit the budget's edge, and no node is ever reached with more spent
    /// than its iteration allows.
    #[cfg(feature = "b3proof")]
    #[test]
    fn no_line_spends_more_than_the_iteration_depth_on_extensions() {
        let forced_lines = [
            "2r3k1/1q2Rp1p/p2p2p1/1p1P4/1Pp1P3/2Q5/1P4PP/6K1 w - - 0 1",
            "8/8/p1p5/1p5p/1P5P/P1P5/8/K1k5 w - - 0 1",
        ];
        let mut truncations = 0;
        for fen in forced_lines {
            let mut searcher = Searcher::default();
            let mut board = Board::from_fen(fen).expect("valid FEN");
            for iteration in 1..=12 {
                let score = searcher.search_root_window(
                    &mut board,
                    iteration,
                    -INF_SCORE,
                    INF_SCORE,
                    &mut || SearchEvent::None,
                );
                assert!(score.abs() < INF_SCORE);
                assert!(
                    searcher.td.budget_overrun <= 0,
                    "a line overspent by {} at depth {iteration} from {fen}",
                    searcher.td.budget_overrun
                );
            }
            assert!(searcher.td.extended_nodes > 0, "no extension from {fen}");
            truncations += searcher.td.budget_truncations;
        }
        assert!(truncations > 0, "the budget never bound");
    }

    /// A middlegame search runs singular searches and extends or reduces
    /// first moves, so the debug build's extension-range assertion is
    /// exercised; the search unwinds with no reduction left on the stack.
    #[cfg(feature = "b3proof")]
    #[test]
    fn singular_decisions_run_in_a_middlegame_search() {
        let mut searcher = Searcher::default();
        let mut board =
            Board::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4")
                .expect("valid FEN");
        for depth in 1..=10 {
            let score =
                searcher.search_root_window(&mut board, depth, -INF_SCORE, INF_SCORE, &mut || {
                    SearchEvent::None
                });
            assert!(score.abs() < INF_SCORE);
        }
        assert!(searcher.td.singular_searches > 0, "no singular search ran");
        assert!(searcher.td.extended_nodes > 0, "no first move was extended");
        for ply in 0..MAX_PLY {
            assert_eq!(searcher.td.stack[ply].reduction, 0, "ply {ply}");
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
