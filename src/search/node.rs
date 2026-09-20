//! The node kernels: `negamax`, `quiescence`, and the per-move reduction and
//! pruning helpers they share.

use crate::board::{Board, CheckInfo, Move, MoveList, Piece};
use crate::eval::{INF_SCORE, MATE_SCORE, VALUE_NONE, piece_value};
use crate::infra;
use crate::tt::{Bound, TtProbe, TtStore};

use super::movepick::{BadCaptureList, MovePicker, SEE_UNKNOWN, diversify_root_scores, pick_next};
use super::{MAX_PLY, MAX_QPLY, SearchEvent, Searcher};

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

#[inline]
pub(super) fn lmr_reduction(r: i32, new_depth: i32) -> i32 {
    // 4.8.1: the ceiling is what the reduced search is allowed to consume: the
    // reduced search may run at depth 0, i.e. in quiescence. `max(0)` keeps the
    // ceiling non-negative, so a shallow move is left unreduced, not extended.
    (r >> 10).clamp(0, new_depth.max(0))
}
/// 4.5.1 THE REDUCTION CONTRACT'S INPUTS.
///
/// `lmr_reduction_units` took thirteen positional arguments and PLAN 4.5.1
/// named that as the defect: it was a pile of parameters rather than a
/// contract over the per-ply context, and adding an input meant editing three
/// signatures and hoping the call sites stayed in step.
///
/// Naming the inputs also removes the whole class of bug where two `bool`s or
/// two `i32`s are passed in the wrong order and still compile.
#[derive(Copy, Clone)]
pub(super) struct ReductionInputs {
    pub(super) depth: i32,
    pub(super) searched: usize,
    is_quiet: bool,
    pub(super) see: i32,
    tt_pv: bool,
    cut_node: bool,
    pub(super) quiet_hist: i32,
    pub(super) corr_abs: i32,
    ev_is_exact: bool,
    tt_move_is_null: bool,
    pub(super) improving: bool,
    is_root: bool,
}

fn late_move_prune_count(depth: i32, improving: bool, count_base: i32) -> usize {
    let base = count_base + 2 * depth * depth / 3;
    if improving {
        infra::to_usize(base + depth)
    } else {
        infra::to_usize(base)
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

    /// LMR reduction in 1024ths of a ply. Excludes the per-thread jitter, which
    /// mutates PRNG state and is drawn once at the reduction site (and not at
    /// all at `Threads = 1`).
    ///
    /// 4.5.1 removed this function's `#[expect(clippy::too_many_arguments)]`:
    /// the thirteen positional arguments are now one named struct, and the
    /// expectation went unfulfilled the moment they did. That is the
    /// self-cleaning property `#[expect]` is used for.
    #[inline(always)]
    pub(super) fn lmr_reduction_units(&self, i: ReductionInputs) -> i32 {
        let ReductionInputs {
            depth,
            searched,
            is_quiet,
            see,
            tt_pv,
            cut_node,
            quiet_hist,
            corr_abs,
            ev_is_exact,
            tt_move_is_null,
            improving,
            is_root,
        } = i;
        let mut r = self.cfg.lmr_table[infra::to_usize(depth.min(63))][searched.min(63)];
        if tt_pv {
            r -= self.cfg.params.lmr_tt_pv_adj;
        } else if is_quiet {
            r += 1024;
        }
        if improving {
            r -= 1024;
        }
        if ev_is_exact {
            r += self.cfg.params.lmr_exact_bound;
        }
        // `lmr_shallow_tt` is a misnomer: it fires on TT-move PRESENCE and was
        // SPSA'd as such. See the `search/params.rs` note.
        if !tt_move_is_null && searched >= 4 {
            r += self.cfg.params.lmr_shallow_tt;
        }
        if cut_node {
            r += self.cfg.params.lmr_cut_node;
        }
        if !is_quiet && see < 0 {
            r += 1024;
        }
        if !tt_pv && !cut_node && quiet_hist > 4_000 {
            r -= 1024;
        }
        r -= quiet_hist * 1024 / self.cfg.params.lmr_hist_div;
        // 8.5(b): reduce less when the static eval is heavily corrected.
        r -= corr_abs * self.cfg.params.corr_lmr_scale / 128;
        // 4.6.7: the root is where the answer is chosen, and it was the
        // one node type the reduction could not see.
        if is_root {
            r -= self.cfg.params.lmr_root_relief;
        }
        r
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
        self.negamax::<Root, _>(board, depth, alpha, beta, 0, true, Move::NULL, false, poll)
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
        let tt_pv = ev.pv_line(NODE::PV);
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
        if !NODE::PV
            && excluded.is_null()
            && let Some(score) = ev.cutoff_score(depth, alpha, beta)
        {
            trace_decision!(
                self,
                ply,
                "tt_cut depth {depth} tt_depth {} bound {:?} score {score} window {alpha} {beta}",
                ev.depth,
                ev.bound
            );
            return score;
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
            && depth >= 4
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
        let (static_eval, raw_static_eval) = if in_check {
            (VALUE_NONE, VALUE_NONE)
        } else {
            let raw = if ev.raw_static_eval == VALUE_NONE {
                self.raw_eval(board)
            } else {
                ev.raw_static_eval
            };
            (self.corrected_eval_from_raw(board, raw, ply), raw)
        };
        self.td.stack[ply].static_eval = static_eval;
        // 8.5(b): magnitude of the correction applied to this node's static
        // eval. A large |corr| means the raw eval is being heavily adjusted and
        // is less trustworthy, so the margin/reduction knobs below prune and
        // reduce less. Zero in check (no static eval).
        //
        // ⚠ The comment here used to say "seeds leave every scale at 0, so this
        // term vanishes". That is no longer true and had gone stale: the fitted
        // seeds are `CorrRfpScale = 3`, `CorrFutScale = 3` and
        // `CorrLmrScale = 27`, so this term is LIVE in the accepted baseline.
        //
        // It is also applied when `eval_for_pruning` below was REPLACED by a TT
        // bound (28.5% of sampled hits refine it, RAR-S30) and the corrected
        // eval discarded; B.2 rebuilds the corrected-eval formula.
        let corr_abs = if static_eval == VALUE_NONE {
            0
        } else {
            (static_eval - raw_static_eval).abs()
        };
        // A `ply - 4` fallback for an unusable `ply - 2` was measured and
        // rejected: RAR-S66 stopped at 13,882 games with the LLR receding from
        // a +2.44 peak. `improving = false` after a check is a conservative
        // default, not a defect — there is genuinely no comparable static eval
        // two plies back when that node was in check.
        let two_back = self.td.stack.back(ply, 2).static_eval;
        let improving = !in_check && two_back != VALUE_NONE && static_eval > two_back;
        let improving_i = if improving { 1 } else { 0 };
        let not_improving_i = 1 - improving_i;
        // 9.7.5 lead: the TT may only stand in for the static eval here if its
        // entry is deep enough to be worth trusting — see the param doc. At the
        // seeded 0 this admits everything, exactly as before.
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
        // 8.3 diagnostic: a non-PV, non-check node where the *stored* PV bit
        // (tt_pv true on a non-PV node type) is what keeps the whole forward-pruning
        // block below from running.
        if tt_pv && !NODE::PV && !in_check && excluded.is_null() {
            crate::diag_count!(tt_pv_veto);
        }
        if !tt_pv && !in_check && excluded.is_null() {
            let futility_margin = (self.cfg.params.futility_base
                + self.cfg.params.futility_not_improving * not_improving_i)
                * depth
                + corr_abs * self.cfg.params.corr_rfp_scale / 128; // 8.5(b)
            if !self.ablated(1) && depth <= 8 && eval_for_pruning - futility_margin >= beta {
                crate::diag_count!(rfp_cut);
                trace_decision!(
                    self,
                    ply,
                    "rfp depth {depth} eval {eval_for_pruning} static {static_eval} corr {corr_abs} \
                     improving {improving} margin {futility_margin} beta {beta}"
                );
                return eval_for_pruning;
            }
            if !self.ablated(0)
                && depth <= 3
                && eval_for_pruning + self.cfg.params.razoring_coeff * depth < alpha
            {
                crate::diag_count!(razor_drop);
                trace_decision!(
                    self,
                    ply,
                    "razor depth {depth} eval {eval_for_pruning} margin {} alpha {alpha}",
                    self.cfg.params.razoring_coeff * depth
                );
                return if NODE::PV {
                    self.quiescence::<Pv, _>(board, alpha, beta, ply, 0, poll)
                } else {
                    self.quiescence::<NonPv, _>(board, alpha, beta, ply, 0, poll)
                };
            }
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
                let mut scored = self.score_tactical_moves(board, captures.as_slice(), tt_move);
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
                    self.push_move(ply, mv, probcut_piece);
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

        let mut move_picker = if in_check || NODE::ROOT || !excluded.is_null() {
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

            let mut scored = self.score_moves(board, legal_moves, tt_move, ply);
            // 8.13: order the root list from the POOL's view. A move another
            // thread has already proven good at a deeper depth is tried first
            // here too, so threads stop re-deriving each other's refutations.
            // Applied BEFORE the rotation below, which diversifies on top.
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
            MovePicker::staged(self, board, tt_move, ply)
        };
        let mut best_move = Move::NULL;
        let mut best_score = -INF_SCORE;
        let mut searched = 0usize;
        #[cfg(feature = "diag")]
        let diag_order_sample = diag_sample && excluded.is_null();
        let mut legal_move_seen = false;
        // 10.3: per-node check masks, built at most once and reused by every
        // move at this node — for the pruning-side `move_gives_check` calls
        // and for the `make_move` check hint below. `board` is restored by
        // `unmake_move` each iteration, so these stay valid for the whole loop.
        let mut node_ci: Option<CheckInfo> = None;
        // 4.7b: latch so `lmp_nodes` counts NODES, not moves -- the oracle can
        // only observe the per-node event, so that is the comparable unit.
        #[cfg(feature = "diag")]
        let mut diag_node_lmp_seen = false;
        let mut quiets = MoveList::new();
        let mut good_caps = BadCaptureList::new();
        let mut bad_caps = BadCaptureList::new();
        let previous_move = self.td.stack.back(ply, 1).mv;
        while let Some(picked) = move_picker.next(self, board) {
            let mv = picked.mv;
            if mv == excluded {
                continue;
            }
            legal_move_seen = true;
            let is_capture = mv.is_capture();
            let is_quiet = board.is_quiet_move(mv);
            let mut see = if is_capture { picked.see as i32 } else { 0 };
            let moving_piece = board.moving_piece(mv);
            let captured_piece = board.captured_piece(mv);
            let quiet_hist = if is_quiet { picked.quiet_history } else { 0 };
            let mut gives_check = None;
            // The picker stage the move came from, classified at pick time: `see`
            // is refined later for some moves, so it must be read here.
            #[cfg(feature = "diag")]
            if diag_order_sample {
                if mv == tt_move {
                    crate::diag_count!(move_seen_tt);
                } else if is_capture && see >= 0 {
                    crate::diag_count!(move_seen_good_capture);
                } else if is_quiet {
                    crate::diag_count!(move_seen_quiet);
                } else {
                    crate::diag_count!(move_seen_bad_capture);
                }
            }

            // 4.5.1: the reduction contract's inputs, built once per move.
            let reduction_inputs = ReductionInputs {
                depth,
                searched,
                is_quiet,
                see,
                tt_pv,
                cut_node,
                quiet_hist,
                corr_abs,
                ev_is_exact: ev.is_exact(),
                tt_move_is_null: tt_move.is_null(),
                improving,
                is_root: NODE::ROOT,
            };
            if !tt_pv && !in_check && searched > 0 {
                #[cfg(feature = "diag")]
                if diag_order_sample {
                    crate::diag_count!(prune_shadow_moves);
                    if is_quiet {
                        let lmp_margin = (self.cfg.params.lmp_base
                            + self.cfg.params.lmp_not_improving * not_improving_i)
                            * depth;
                        let lmp = (depth <= 3 && eval_for_pruning + lmp_margin <= alpha)
                            || (depth <= 8
                                && searched
                                    > late_move_prune_count(
                                        depth,
                                        improving,
                                        self.cfg.params.lmp_count_base,
                                    ))
                            || (depth <= 4 && quiet_hist < -10_000)
                            || (depth <= 7
                                && quiet_hist < -(self.cfg.params.quiet_hist_prune_coeff * depth));
                        let futility = depth <= 8
                            && eval_for_pruning
                                + self.cfg.params.fp_base
                                + self.cfg.params.fp_coeff * depth
                                + corr_abs * self.cfg.params.corr_fut_scale / 128
                                <= alpha;
                        let checking = (lmp || futility)
                            && move_gives_check(board, &mut node_ci, mv, &mut gives_check);
                        if lmp {
                            crate::diag_count!(prune_shadow_lmp);
                        }
                        if futility {
                            crate::diag_count!(prune_shadow_futility);
                        }
                        if lmp && futility {
                            crate::diag_count!(prune_shadow_overlap_two_plus);
                        }
                        if checking {
                            crate::diag_count!(prune_shadow_check_exempt);
                        }
                    } else if is_capture && see < 0 {
                        let cap_hist = captured_piece.map_or(0, |cap| {
                            self.td.hist.cap_history[moving_piece as usize][mv.to_sq().index()]
                                [cap as usize] as i32
                        });
                        let threshold = (-self.cfg.params.see_pruning_coeff * depth - cap_hist / 8)
                            .max(-self.cfg.params.see_pruning_max);
                        let see_shadow = depth <= 8 && !board.see_ge(mv, threshold);
                        if see_shadow {
                            crate::diag_count!(prune_shadow_see);
                            if move_gives_check(board, &mut node_ci, mv, &mut gives_check) {
                                crate::diag_count!(prune_shadow_check_exempt);
                            }
                        }
                    }
                }
                if is_quiet {
                    let prune_margin = (self.cfg.params.lmp_base
                        + self.cfg.params.lmp_not_improving * not_improving_i)
                        * depth;
                    // 4.6.5: the move-count component ALONE, so it can be fed
                    // back to the picker the way the reference feeds its
                    // `moveCountPruning` flag into `next_move`.
                    let move_count_pruning = depth <= 8
                        && searched
                            > late_move_prune_count(
                                depth,
                                improving,
                                self.cfg.params.lmp_count_base,
                            );
                    let prune_candidate = !self.ablated(5)
                        && ((depth <= 3 && eval_for_pruning + prune_margin <= alpha)
                            || move_count_pruning
                            || (depth <= 4 && quiet_hist < -10_000)
                            || (depth <= 7
                                && quiet_hist < -(self.cfg.params.quiet_hist_prune_coeff * depth)));
                    if prune_candidate
                        && !move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                    {
                        crate::diag_count!(lmp_prune);
                        trace_decision!(
                            self,
                            ply,
                            "lmp depth {depth} move {mv} searched {searched} history {quiet_hist} \
                             eval {eval_for_pruning} margin {prune_margin} count_prune {move_count_pruning} \
                             alpha {alpha}"
                        );
                        #[cfg(feature = "diag")]
                        if !diag_node_lmp_seen {
                            diag_node_lmp_seen = true;
                            crate::diag_count!(lmp_nodes);
                        }
                        continue;
                    }
                    // Per-move quiet futility pruning (Phase 2.7): a quiet move
                    // whose TT-refined static eval plus a margin can't reach alpha
                    // is skipped. Plain skip (no fail-soft best_score update), to
                    // match the existing LMP/SEE prunes in this loop.
                    if depth <= 8
                        && eval_for_pruning
                            + self.cfg.params.fp_base
                            + self.cfg.params.fp_coeff * depth
                            + corr_abs * self.cfg.params.corr_fut_scale / 128 // 8.5(b)
                            <= alpha
                        && !move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                    {
                        crate::diag_count!(quiet_futility_prune);
                        trace_decision!(
                            self,
                            ply,
                            "futility depth {depth} move {mv} searched {searched} eval {eval_for_pruning} \
                             corr {corr_abs} alpha {alpha}"
                        );
                        continue;
                    }
                } else if is_capture && see < 0 {
                    let cap_hist = captured_piece.map_or(0, |cap| {
                        self.td.hist.cap_history[moving_piece as usize][mv.to_sq().index()]
                            [cap as usize] as i32
                    });
                    let see_threshold = (-self.cfg.params.see_pruning_coeff * depth - cap_hist / 8)
                        .max(-self.cfg.params.see_pruning_max);
                    if depth <= 8
                        && !board.see_ge(mv, see_threshold)
                        && !move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                    {
                        crate::diag_count!(see_prune);
                        trace_decision!(
                            self,
                            ply,
                            "see_prune depth {depth} move {mv} searched {searched} threshold {see_threshold}"
                        );
                        continue;
                    }
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
                    poll,
                );
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

            let checking_move =
                if depth >= 3 && searched >= 2 && (is_quiet || see < 0) && !mv.is_promo() {
                    move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                } else {
                    gives_check.unwrap_or(false)
                };

            self.push_move(ply, mv, moving_piece);
            let nodes_before_move = if NODE::ROOT { self.td.nodes } else { 0 };
            // 10.3: the check predicate is cheap here (node masks + two
            // bitboard tests) and lets `make_move` skip `calculate_checkers`
            // for the overwhelmingly common non-checking move.
            let mv_gives_check = move_gives_check(board, &mut node_ci, mv, &mut gives_check);
            board.make_move_with_check(mv, mv_gives_check);
            self.shared.tt.prefetch(board.hash());
            let new_depth = depth - 1 + extension;
            #[cfg(feature = "diag")]
            if diag_sample {
                crate::diag_add!(
                    prospective_depth_sum,
                    u64::try_from(new_depth.max(0)).unwrap_or(0)
                );
            }
            let mut score;

            if searched == 0 {
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
                        poll,
                    )
                };
            } else {
                // Late evasions are intentionally not reduced. The alternative
                // increased the deterministic tree by 14.83% and had no owner.
                let reducible = !self.ablated(7)
                    && depth >= 3
                    && searched >= 2
                    && (is_quiet || see < 0)
                    && !mv.is_promo()
                    && !in_check
                    && !checking_move;
                if reducible {
                    // Accumulate in 1024ths; `>> 10` gives integer ply reduction.
                    // Defaults for lmr_* params = 1024, reproducing the original ±1 ply
                    // behavior exactly. SPSA tunes from this baseline.
                    // `reducible` already guarantees depth >= 3 && searched >= 2, so the
                    // table lookup is always in the populated region.
                    let mut r = self.lmr_reduction_units(reduction_inputs);
                    // 8.13: per-thread reduction jitter, the Reckless
                    // diversification shape. `r` is in 1024ths of a ply, so
                    // ±64 is ±6% of one ply: enough to send threads down
                    // different trees, small enough not to distort the mean
                    // reduction. It composes WITH rotation and pool ordering —
                    // pool knowledge correlates the threads' root ordering, so
                    // in-tree decorrelation matters more here than it did
                    // standalone (jitter-for-rotation alone measured ±0).
                    // 9.7.5(k): a real per-thread PRNG (see `next_jitter`),
                    // replacing a node-counter modulo that was both correlated
                    // with the counter and biased +4.5/1024. Only in a parallel
                    // search: the thread count gates it, which is what keeps
                    // bench identical.
                    // SMP diversification, unchanged: magnitude 64 reproduces
                    // the original expression exactly.
                    if self.shared.threads > 1 {
                        r += self.next_jitter(64);
                    }
                    // 10.2.5 candidate: strong late moves may escape the old
                    // mandatory one-ply reduction. A zero reduction is a normal
                    // full-depth PVS search and must not trigger a redundant
                    // verification search at the same depth.
                    let reduction = lmr_reduction(r, new_depth);
                    trace_decision!(
                        self,
                        ply,
                        "lmr depth {depth} move {mv} searched {searched} history {quiet_hist} \
                         units {r} reduction {reduction} new_depth {new_depth} window {alpha} {beta}"
                    );
                    #[cfg(feature = "diag")]
                    {
                        if new_depth > 0 && reduction == new_depth {
                            crate::diag_count!(node_lmr_qs_clamped);
                        }
                        // 4.2: EXACT, because its denominator `lmr_applied` is
                        // exact. Sampling only the numerator made the mean
                        // reduction read 1024x low at the default stride.
                        crate::diag_add!(
                            reduction_depth_sum,
                            u64::try_from(reduction).unwrap_or(0)
                        );
                    }
                    if reduction == 0 {
                        crate::diag_count!(node_lmr_zero_reduction);
                    } else {
                        crate::diag_count!(lmr_applied);
                    }
                    score = -self.negamax::<NonPv, _>(
                        board,
                        new_depth - reduction,
                        -alpha - 1,
                        -alpha,
                        ply + 1,
                        true,
                        Move::NULL,
                        true,
                        poll,
                    );
                    if reduction > 0 && score > alpha {
                        crate::diag_count!(lmr_research);
                        trace_decision!(
                            self,
                            ply,
                            "lmr_research move {mv} reduced_score {score} alpha {alpha}"
                        );
                        // Full-depth verification re-search. (A do-deeper / do-shallower
                        // LMR re-search adjustment was tried as Phase 2.8 and dropped:
                        // do_shallower was proven dead, and SPSA-tuned do_deeper failed
                        // its SPRT gate at st=0.1 — -1.38 Elo for ~4% more nodes
                        // (bench 5,612,008 vs 5,401,662), a TC-transfer failure like the
                        // 2.4 LMR tune. See PLAN §5 2.8.)
                        score = -self.negamax::<NonPv, _>(
                            board,
                            new_depth,
                            -alpha - 1,
                            -alpha,
                            ply + 1,
                            true,
                            Move::NULL,
                            !cut_node,
                            poll,
                        );
                    }
                } else {
                    score = -self.negamax::<NonPv, _>(
                        board,
                        new_depth,
                        -alpha - 1,
                        -alpha,
                        ply + 1,
                        true,
                        Move::NULL,
                        true,
                        poll,
                    );
                }
                if score > alpha && score < beta {
                    score = -self.negamax::<Pv, _>(
                        board,
                        new_depth,
                        -beta,
                        -alpha,
                        ply + 1,
                        true,
                        Move::NULL,
                        false,
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
            if score > alpha {
                alpha = score;
                self.td.pv_table[ply][ply] = mv;
                let child_len = self.td.pv_len[ply + 1].max(ply + 1);
                for next_ply in ply + 1..child_len {
                    self.td.pv_table[ply][next_ply] = self.td.pv_table[ply + 1][next_ply];
                }
                self.td.pv_len[ply] = child_len;

                if score >= beta {
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
                        // 8.4(e): the cutoff REWARD is scaled when the node
                        // static eval sat below beta - the search found a good
                        // move the eval did not credit. 100 = neutral; maluses
                        // stay unscaled.
                        let bonus_pct = if static_eval != VALUE_NONE && static_eval < beta {
                            self.cfg.params.surprise_bonus_pct
                        } else {
                            100
                        };
                        if !is_capture {
                            crate::diag_count!(cutoff_quiet);
                            self.update_cutoff_tables(
                                board,
                                mv,
                                moving_piece,
                                previous_move,
                                ply,
                                depth,
                                bonus_pct,
                                quiets.as_slice(),
                                &good_caps,
                                &bad_caps,
                            );
                        } else {
                            crate::diag_count!(cutoff_capture);
                            self.update_capture_history(
                                moving_piece,
                                mv.to_sq().index(),
                                captured_piece,
                                self.history_bonus(depth) * bonus_pct / 100,
                            );
                            let malus = self.history_malus(depth);
                            for gc in good_caps.as_slice() {
                                self.update_capture_history(
                                    gc.attacker,
                                    gc.to as usize,
                                    gc.captured,
                                    -malus,
                                );
                            }
                            // 8.4(c): a capture cutoff today penalizes only the
                            // earlier good captures - the searched quiets and
                            // bad captures that failed to cut escape unscathed.
                            // Cross-category malus at a tunable fraction; seed 0
                            // = skip. Good-SEE captures keep the existing malus
                            // only (the all-capture form was bench-vetoed in the
                            // Basilisk cross-review).
                            if self.cfg.params.capture_malus_pct != 0 {
                                let xmalus = malus * self.cfg.params.capture_malus_pct / 100;
                                let color = board.side_to_move();
                                let pawn_key = board.pawn_key();
                                for &quiet in quiets.as_slice() {
                                    self.update_quiet_history(
                                        color,
                                        quiet,
                                        board.moving_piece(quiet),
                                        pawn_key,
                                        ply,
                                        -xmalus,
                                    );
                                }
                                for bc in bad_caps.as_slice() {
                                    self.update_capture_history(
                                        bc.attacker,
                                        bc.to as usize,
                                        bc.captured,
                                        -xmalus,
                                    );
                                }
                            }
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
                        if static_eval != VALUE_NONE
                            && score.abs() < MATE_SCORE - infra::to_i32(MAX_PLY)
                            && score > static_eval
                        {
                            crate::diag_count!(correction_updates);
                            // 8.5a diagnostic: correction trained by a *capture*
                            // beta cutoff — the eval learning to absorb search
                            // tactics that then feed back into pruning.
                            if is_capture {
                                crate::diag_count!(corr_on_capture);
                            }
                            let residual = self.attributed_residual(
                                score - static_eval,
                                is_capture,
                                board.halfmove_clock(),
                            );
                            self.update_correction(board, residual, depth, ply);
                        }
                    }
                    return score;
                }
            }

            if is_quiet {
                quiets.push(mv);
            } else if is_capture {
                if see == SEE_UNKNOWN as i32 {
                    see = if board.see_ge(mv, 0) { 0 } else { -1 };
                }
                if see >= 0 {
                    good_caps.push(moving_piece, mv.to_sq().0, captured_piece);
                } else {
                    bad_caps.push(moving_piece, mv.to_sq().0, captured_piece);
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
        if excluded.is_null()
            && static_eval != VALUE_NONE
            && best_score.abs() < MATE_SCORE - infra::to_i32(MAX_PLY)
        {
            let diff = best_score - static_eval;
            // Update correction for PV nodes (Exact) and fail-lows where score < static_eval
            if bound == Bound::Exact || (bound == Bound::Upper && diff < 0) {
                crate::diag_count!(correction_updates);
                if best_move.is_capture() {
                    crate::diag_count!(corr_on_capture);
                }
                let residual =
                    self.attributed_residual(diff, best_move.is_capture(), board.halfmove_clock());
                self.update_correction(board, residual, depth, ply);
            }
        }
        if excluded.is_null() {
            // 8.4(b): an Exact (PV) node best move improved alpha without
            // cutting - today it gets zero feedback. Reward the QUIET best
            // move at a tunable fraction of the cutoff bonus. REWARD-ONLY by
            // design: no sibling malus, no killer/countermove write, no
            // capture reward (Basilisk cross-review: reward-only +4.90, the
            // sibling-malus form -84.21). Seed 0 = skip.
            if bound == Bound::Exact
                && self.cfg.params.exact_bonus_pct != 0
                && !best_move.is_null()
                && !best_move.is_capture()
                && !best_move.is_promo()
            {
                let bonus = self.history_bonus(depth) * self.cfg.params.exact_bonus_pct / 100;
                self.update_quiet_history(
                    board.side_to_move(),
                    best_move,
                    board.moving_piece(best_move),
                    board.pawn_key(),
                    ply,
                    bonus,
                );
            }
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
        let mut scored = if in_check {
            self.score_moves(board, moves.as_slice(), tt_move, ply)
        } else {
            self.score_tactical_moves(board, moves.as_slice(), tt_move)
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
            self.push_move(ply, mv, moving_piece);
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

    #[test]
    fn lmr_reduction_allows_strong_late_moves_to_reach_zero() {
        assert_eq!(lmr_reduction(1023, 8), 0);
        assert_eq!(lmr_reduction(1024, 8), 1);
        // 4.8.1: the reduction may consume the whole of new_depth, so the
        // reduced search runs in quiescence (B.2 floors it at one ply).
        assert_eq!(lmr_reduction(4096, 3), 3);
        assert_eq!(lmr_reduction(-1, 8), 0);
        // Never extend, and never reduce below depth 0.
        assert_eq!(lmr_reduction(1024, 0), 0);
        assert_eq!(lmr_reduction(4096, -1), 0);
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
