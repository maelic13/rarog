// `clippy::too_many_arguments` is accepted crate-wide for search kernels —
// see the rationale in Cargo.toml's [lints.clippy] section.
use std::sync::{Arc, atomic::Ordering, mpsc};
use std::time::Instant;

use crate::board::{Bitboard, Board, CheckInfo, Color, GameResult, Move, MoveList, Piece};
use crate::eval::{Evaluator, INF_SCORE, MATE_SCORE, VALUE_NONE, piece_value};
use crate::infra;
use crate::move_ordering::{
    BadCaptureList, CAP_HISTORY_MAX, CONT_SIZE, CORR_SIZE, HISTORY_MAX, LOW_PLY_HISTORY_SIZE,
    PAWN_HISTORY_SIZE, PIECE_TO_SIZE, ScoredMove, ScoredMoveList, diversify_root_scores,
    pawn_history_index, pawn_row_base, pick_next, piece_to_index, update_hist_entry,
};
use crate::params::SearchParams;
use crate::search_options::{EngineOptions, MAX_THREADS, SearchLimits, SearchOptions};
use crate::search_threads::{
    RootBound, STOP_NONE, STOP_QUIT, STOP_SEARCH, SharedSearchState, WorkerJob, WorkerPool,
};
use crate::syzygy::{self, Wdl};
use crate::time_manager::{RuntimeLimits, compute_runtime_limits};
// `MoveClass` is read only by the diagnostic best-move census; the class itself
// is always computed (it lives on `MoveEvidence`), but nothing in a production
// build names the type.
#[cfg(feature = "diag")]
use crate::evidence::MoveClass;
use crate::evidence::{MoveEvidence, NodeEvidence, OutcomeKind};
use crate::tt::{Bound, TranspositionTable, TtStore};

const MAX_DEPTH: usize = 100;

/// Continuation-history look-back distances and their bonus divisors.
///
/// 9.0a: replaces four parallel `cont_history_N` fields and four copy-pasted
/// blocks in each of the read / update / age paths (twelve near-identical
/// stanzas). `(plies_back, bonus_divisor)` — slot order is the array order in
/// [`Searcher::cont_history`], so adding a look-back distance is one entry
/// here rather than a field plus three new blocks.
const CONT_PLY_BACK: [(usize, i32); 4] = [(1, 1), (2, 1), (4, 2), (6, 3)];
const CONT_TABLES: usize = CONT_PLY_BACK.len();

/// Node-invariant half of quiet-history scoring, resolved once per node by
/// [`Searcher::quiet_history_ctx`] (8.12(g2)): the continuation rows that
/// apply at this ply (`None` = guard failed — too shallow or null previous
/// move) and the pawn-history row for this pawn structure. Per move, scoring
/// adds only `piece_to_index(piece, to)` to each base.
struct QuietHistoryCtx {
    cont_bases: [Option<usize>; CONT_TABLES],
    pawn_base: usize,
}
const MAX_PLY: usize = 128;
const MAX_QPLY: usize = 16;
const MIN_PARALLEL_DEPTH: usize = 4;
/// 9.7.5(k) jitter-PRNG seeding. Two odd 64-bit constants (SplitMix64's
/// increment and Xorshift*'s multiplier); the `| 1` at the use site guarantees
/// the state is never zero, xorshift's fixed point.
const JITTER_SEED: u64 = 0x9E37_79B9_7F4A_7C15;
const JITTER_STRIDE: u64 = 0x2545_F491_4F6C_DD1D;
const SHARED_NODE_BATCH: u64 = 128;
const SHARED_NODE_BATCH_MASK: u64 = SHARED_NODE_BATCH - 1;
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // const-evaluated; MAX_PLY = 128
const TB_WIN_SCORE: i32 = MATE_SCORE - (MAX_PLY as i32) * 2;
const SEE_UNKNOWN: i16 = i16::MIN;
/// Heap-allocate the continuation tables without a ~1.1 MB stack temporary
/// (`Box::new([[0; CONT_SIZE]; N])` would materialize the array on the stack
/// first). Startup-only.
fn boxed_cont_tables() -> Box<[[i16; CONT_SIZE]; CONT_TABLES]> {
    let tables: Box<[[i16; CONT_SIZE]]> = vec![[0; CONT_SIZE]; CONT_TABLES].into_boxed_slice();
    tables
        .try_into()
        .unwrap_or_else(|_| unreachable!("length is CONT_TABLES by construction"))
}

// Float→int truncation IS the intended rounding of the LMR table formula
// (kept bit-exact with the pre-9.0b table), hence the scoped cast allow.
#[expect(clippy::cast_possible_truncation)]
fn build_lmr_table(base: i32, div: i32) -> Box<[[i32; 64]; 64]> {
    let base_f = base as f64 / 1024.0;
    let div_f = div as f64 / 1024.0;
    let mut table = Box::new([[0i32; 64]; 64]);
    for (depth, row) in table.iter_mut().enumerate().skip(1) {
        for (move_index, value) in row.iter_mut().enumerate().skip(1) {
            *value =
                (1024.0 * (base_f + (depth as f64).ln() * (move_index as f64).ln() / div_f)) as i32;
        }
    }
    table
}

#[inline]
fn lmr_reduction(r: i32, new_depth: i32, min_reduced_depth: i32) -> i32 {
    // 4.8.1: the ceiling is what the reduced search is allowed to consume.
    // At `min_reduced_depth == 0` this is the accepted behaviour and the
    // reduced search may run at depth 0, i.e. in quiescence. `max(0)` keeps
    // the ceiling non-negative when new_depth is already below the floor,
    // so a shallow move is simply left unreduced rather than extended.
    let ceiling = (new_depth - min_reduced_depth).max(0);
    (r >> 10).clamp(0, ceiling)
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SearchEvent {
    None,
    Stop,
    Quit,
    PonderHit,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SearchExit {
    Stop,
    Quit,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub bestmove: Move,
    pub pondermove: Move,
    pub score: i32,
    pub depth: usize,
    pub nodes: u64,
    pub tb_hits: u64,
    pub elapsed_ms: u128,
    pub exit: SearchExit,
    pub ponderhit: bool,
}

/// Persistent state for one legal root move across iterative-deepening passes.
///
/// Phase 10.1 deliberately only PRODUCES this information. Aspiration, time
/// management, interrupted-iteration fallback, MultiPV, and SMP consumers land
/// later, after the bookkeeping substrate is proven bench-identical.
#[derive(Debug, Clone)]
struct RootMove {
    mv: Move,
    score: i32,
    previous_score: i32,
    average_score: f64,
    mean_squared_score: f64,
    samples: u32,
    last_search_depth: usize,
    pv: [Move; MAX_PLY],
    pv_len: usize,
    nodes: u64,
    seldepth: usize,
    fail_highs: u32,
    fail_lows: u32,
    last_best_depth: usize,
}

impl RootMove {
    fn new(mv: Move) -> Self {
        let mut pv = [Move::NULL; MAX_PLY];
        pv[0] = mv;
        Self {
            mv,
            score: -INF_SCORE,
            previous_score: -INF_SCORE,
            average_score: 0.0,
            mean_squared_score: 0.0,
            samples: 0,
            last_search_depth: 0,
            pv,
            pv_len: 1,
            nodes: 0,
            seldepth: 0,
            fail_highs: 0,
            fail_lows: 0,
            last_best_depth: 0,
        }
    }

    /// Freeze the last iteration's score before this iteration starts.
    ///
    /// If the new iteration is interrupted, `previous_score` remains the last
    /// completed-iteration fallback while `score` may contain newer partial
    /// information, matching the distinction later consumers need.
    fn begin_iteration(&mut self) {
        self.previous_score = self.score;
    }

    /// Root-only bookkeeping must not be inlined into the node kernel. Besides
    /// executing only a few hundred times per search, keeping the floating
    /// point/statistics block cold prevents it from perturbing `negamax`'s hot
    /// code layout (the first 10.1 implementation measured a real NPS loss).
    #[cold]
    #[inline(never)]
    fn record_search(
        &mut self,
        depth: usize,
        score: i32,
        nodes: u64,
        seldepth: usize,
        bound: RootBound,
    ) {
        self.score = score;
        self.last_search_depth = depth;
        self.nodes = self.nodes.saturating_add(nodes);
        self.seldepth = self.seldepth.max(seldepth);
        match bound {
            RootBound::Lower => self.fail_highs = self.fail_highs.saturating_add(1),
            RootBound::Upper => self.fail_lows = self.fail_lows.saturating_add(1),
            RootBound::Exact => {}
        }
    }

    /// Add exactly one distribution sample for a COMPLETED iteration. Failed
    /// aspiration visits still update score/bounds/effort above, but must not
    /// overweight volatile iterations in the statistics consumed by 10.2.
    fn complete_iteration(&mut self) {
        self.samples = self.samples.saturating_add(1);
        let weight = 1.0 / f64::from(self.samples);
        let score = f64::from(self.score);
        self.average_score += (score - self.average_score) * weight;
        let squared_score = score * score;
        self.mean_squared_score += (squared_score - self.mean_squared_score) * weight;
    }
}

/// How well-determined the root is after one COMPLETED iteration.
///
/// Phase 4.7b's whole point is that there is exactly ONE of these. Before it,
/// the clock derived its own view of root stability from best-move changes and
/// effort while aspiration derived a separate view from nothing at all, and
/// neither could say what the other was already charging for. One snapshot, two
/// consumers, each behind its own switch.
///
/// Built only from a completed iteration, never a partial one — which is what
/// keeps an aborted search on last-completed evidence (4.7a, `tests/root_abort`).
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct RootConfidence {
    /// Score gap from the best move to the best of the rest, in centipawns.
    #[cfg_attr(not(feature = "diag"), allow(dead_code))]
    gap: i32,
    /// Standard deviation of the best move's completed-iteration scores, in
    /// centipawns, from its running mean/mean-square pair.
    deviation: i32,
    /// Share of the iteration's nodes spent under the best move, 0.0..=1.0.
    effort: f64,
    /// Aspiration re-searches this iteration, by direction.
    fail_lows: i32,
    fail_highs: i32,
    /// This thread's decaying best-move-change count.
    instability: f64,
    /// The pool's mean of the same count, when `RootConfPoolInstability` is on
    /// and a pool exists. ⚠ Read by [`Self::time_factor`] and nothing else.
    pool_instability: Option<f64>,
}

/// Confidence returned when the model has no enabled inputs at all, i.e. every
/// weight was fitted to zero. "Unknown" is the midpoint, not zero.
const NEUTRAL_CONFIDENCE: i32 = 500;

/// Widening bumps a single side of the aspiration window may accumulate. Makes
/// `RootConfAspiration` bounded by construction rather than by the scores
/// happening to behave.
const ASP_CONF_FAIL_BUMP_CAP: i32 = 3;

impl RootConfidence {
    /// Confidence in the completed root result, per mille.
    ///
    /// Four terms, each normalised to `0..=1000` and weighted by its own
    /// coordinate: SEPARATION (the gap to the best of the rest), STEADINESS
    /// (the best move's score deviation), EFFORT (share of the iteration spent
    /// on it) and WINDOW (aspiration re-searches this iteration).
    ///
    /// ⚠ Two of the inputs Plan 4.7 lists are deliberately absent, and both
    /// absences are the no-double-counting rule rather than oversights.
    ///
    /// INSTABILITY is time-only. Keeping it out of the scalar is what makes
    /// "pool worker instability for time, not result ownership" structural:
    /// [`Self::aspiration_deltas`] reads only this function, so it cannot see
    /// pooled helper state however the switches are set.
    ///
    /// BEST-MOVE AGE is the same signal as instability in another encoding —
    /// `tot_best_move_changes` halves every iteration and gains 1 on a change,
    /// so an age of `n` implies an instability below `2^-n` plus older history.
    /// Charging both would price one fact twice, so age is not a field here at
    /// all; it is measured instead (`rootconf_best_age_sum` against
    /// `rootconf_instab_milli_sum`), which keeps the claim falsifiable without
    /// leaving an uncharged input lying inside the model.
    ///
    /// PV LENGTH is likewise measured only (`rootconf_pv_truncated`), and stays
    /// out until that counter shows a population — 4.5d's lesson, where a
    /// context with a real mean and no population would have bought a table
    /// that never filled.
    pub(crate) fn scalar(&self, params: &SearchParams) -> i32 {
        let deviation = i64::from(self.deviation.max(0));
        let dev_scale = i64::from(params.root_conf_dev_scale.max(1));
        let steadiness = 1000 * dev_scale / (dev_scale + deviation);

        // KEEP-ALLOW: `effort_term` clamps to `0.0..=1.0` BEFORE this, so the
        // product is provably inside `0..=1000` rather than merely saturating,
        // and `round` is the rounding intended. Same contract as the window
        // centre's cast in `search_root`.
        #[expect(clippy::cast_possible_truncation)]
        let effort = (effort_term(self.effort) * 1000.0).round() as i64;

        let fails = i64::from(self.fail_lows.max(0) + self.fail_highs.max(0));
        let window = 1000 / (1 + fails);

        let weights = [
            (i64::from(params.root_conf_w_deviation.max(0)), steadiness),
            (i64::from(params.root_conf_w_effort.max(0)), effort),
            (i64::from(params.root_conf_w_window.max(0)), window),
        ];
        let total: i64 = weights.iter().map(|(weight, _)| *weight).sum();
        if total == 0 {
            return NEUTRAL_CONFIDENCE;
        }
        let weighted: i64 = weights.iter().map(|(weight, term)| weight * term).sum();
        i32::try_from((weighted / total).clamp(0, 1000)).unwrap_or(NEUTRAL_CONFIDENCE)
    }

    /// The clock multiplier that REPLACES the baseline's
    /// `instability × effort` product when `RootConfTime` is on.
    ///
    /// No double counting, by construction. The baseline product has exactly
    /// two inputs: effort, which lives inside [`Self::scalar`] and is therefore
    /// charged there and only there, and instability, which is applied here and
    /// only here. The interpolation has the same SHAPE as the baseline effort
    /// factor — one linear ramp between two endpoints — but its own pair of
    /// them, `TmConfHigh`/`TmConfLow`, seeded so the arm redistributes the
    /// clock without moving its total. See those coordinates for the derivation.
    ///
    /// ⚠ The one place `pool_instability` is read. See its field note.
    pub(crate) fn time_factor(&self, params: &SearchParams) -> f64 {
        let confidence = f64::from(self.scalar(params)) / 1000.0;
        let instability = self.pool_instability.unwrap_or(self.instability);
        tm_confidence_factor(params, confidence) * tm_instability_factor(params, instability)
    }

    /// Initial `(alpha_delta, beta_delta)` for the next iteration's aspiration
    /// window when `RootConfAspiration` is on.
    ///
    /// Bounded and asymmetric. Bounded: the width is a percentage of the
    /// baseline delta between two declared coordinates, each side takes at most
    /// [`ASP_CONF_FAIL_BUMP_CAP`] widening bumps, and the result is clamped into
    /// the score domain. Asymmetric: a side that failed last iteration is the
    /// side whose bound was wrong, so it — and not its partner — opens.
    pub(crate) fn aspiration_deltas(&self, base_delta: i32, params: &SearchParams) -> (i32, i32) {
        let confidence = i64::from(self.scalar(params));
        let wide = i64::from(params.asp_conf_wide_pct);
        let narrow = i64::from(params.asp_conf_narrow_pct);
        let percent = wide + (narrow - wide) * confidence / 1000;
        let width = i64::from(base_delta) * percent / 100;
        let widen = |fails: i32| -> i32 {
            let bumps = i64::from(fails.clamp(0, ASP_CONF_FAIL_BUMP_CAP));
            let scaled = width * (100 + i64::from(params.asp_conf_fail_pct) * bumps) / 100;
            i32::try_from(scaled.clamp(1, i64::from(INF_SCORE))).unwrap_or(INF_SCORE)
        };
        (widen(self.fail_lows), widen(self.fail_highs))
    }
}

/// Share of an iteration below which the effort term reads its floor. Named
/// because 4.7b's diagnostics have to count how often the term is above it —
/// see `rootconf_effort_term_live`.
pub(crate) const EFFORT_TERM_FLOOR: f64 = 0.79;

/// Iteration-skip pattern for helper threads (4.9c), the classic Lazy-SMP
/// tables: a thread skips depth `d` when `((d + phase) / size)` is odd.
///
/// The shape is what matters. `size` grows with thread index so low-numbered
/// helpers alternate quickly and high-numbered ones skip in longer runs, and
/// `phase` offsets them against each other, so at any instant the pool is
/// spread across several depths instead of stacked on one. Twenty entries then
/// repeat, which is ample: the pattern only has to decorrelate threads from one
/// another, not be unique per thread forever.
///
/// Deliberately NOT including a position term. Stockfish's variant adds
/// `game_ply` so the pattern differs move to move; that is omitted here so a
/// given position always produces the same skip structure, which keeps a 4T/8T
/// A/B reproducible. Adding it is a later question, not a free improvement.
const SMP_SKIP_SIZE: [u32; 20] = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3];
const SMP_SKIP_PHASE: [u32; 20] = [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];

/// Does this helper skip iteration `depth`?
///
/// Thread 0 NEVER skips, whatever the switch says. It owns the reported result,
/// the emitted info lines and the soft-stop estimate, so an iteration it did not
/// run is one the caller never sees — the whole mechanism has to be paid for by
/// the helpers.
fn helper_skips_iteration(thread_id: usize, depth: usize) -> bool {
    if thread_id == 0 {
        return false;
    }
    let slot = (thread_id - 1) % SMP_SKIP_SIZE.len();
    let depth = u32::try_from(depth).unwrap_or(u32::MAX);
    ((depth + SMP_SKIP_PHASE[slot]) / SMP_SKIP_SIZE[slot]) % 2 == 1
}

/// Effort normalised to `0.0..=1.0`: 0 at or below [`EFFORT_TERM_FLOOR`] of the
/// iteration spent on the best move, 1 at the whole iteration.
///
/// Single-sourced on purpose. The baseline clock and the 4.7b confidence scalar
/// both consume effort, and a second copy of this interpolation is exactly how
/// two consumers drift into disagreeing about the same number.
fn effort_term(effort: f64) -> f64 {
    ((effort - EFFORT_TERM_FLOOR) / (1.0 - EFFORT_TERM_FLOOR)).clamp(0.0, 1.0)
}

/// Interpolate a clock multiplier from `high` at `t = 0` to `low` at `t = 1`.
///
/// Clamped to the ordered pair so an SPSA-crossed (`low > high`) setting cannot
/// panic `f64::clamp`; for the effort endpoints this is `clamp(0.71, 0.924)`.
fn tm_interpolate(high: f64, low: f64, t: f64) -> f64 {
    (high + t * (low - high)).clamp(low.min(high), low.max(high))
}

/// Baseline effort factor: the shipped clock's own confidence proxy.
///
/// ⚠ Measured near-CONSTANT (RAR-S47). The band starts at
/// [`EFFORT_TERM_FLOOR`], and effort averages 37.9% of the iteration, so this
/// reads its `TmEffortHigh` endpoint on 473 of 520 bench iterations (91.0%).
/// That is the concrete thing `RootConfTime` replaces: not a rival estimator,
/// but a term that is a function on 9% of iterations and a constant on the rest.
fn tm_effort_factor(params: &SearchParams, effort: f64) -> f64 {
    tm_interpolate(
        f64::from(params.tm_effort_high) / 10_000.0,
        f64::from(params.tm_effort_low) / 10_000.0,
        effort_term(effort),
    )
}

/// Confidence factor: the same ramp shape on its own endpoints.
fn tm_confidence_factor(params: &SearchParams, confidence: f64) -> f64 {
    tm_interpolate(
        f64::from(params.tm_conf_high) / 10_000.0,
        f64::from(params.tm_conf_low) / 10_000.0,
        confidence,
    )
}

/// Best-move-instability factor: rises when the best move changed recently.
/// Shared by the baseline clock and by [`RootConfidence::time_factor`], so the
/// arm cannot silently re-shape the term it claims to keep.
fn tm_instability_factor(params: &SearchParams, instability: f64) -> f64 {
    f64::from(params.tm_instab_base) / 10_000.0
        + f64::from(params.tm_instab_slope) / 10_000.0 * instability
}

// 9.0: `clippy::large_enum_variant` is deliberately allowed here. The `Full`
// variant embeds a ScoredMoveList (~3 KB) inline, which is the point: a
// MovePicker is constructed at EVERY interior node, and boxing the large
// variant would trade a stack-resident list for a heap allocation per node.
// Measured elsewhere in 9.0: making the move lists heap/initialized cost
// -10% NPS. The enum size is a deliberate space-for-speed trade, and the
// note is kept even though clippy no longer objects: if the variants diverge
// again the lint fires, and this is the reason not to "fix" it by boxing.
/// 4.5.2 MOVE-PICKER STAGE CONTRACT.
///
/// The staged picker's transitions used to live implicitly in three cursor
/// comparisons (`good_index < good_len`, `cap_len + quiet_index < len`,
/// `good_len + bad_index < cap_len`). Reading the order off that required
/// reconstructing the buffer partition in your head, and nothing named the
/// order or made it assertable. This enum is that order.
///
/// Contract, and all three parts are covered by tests below:
///   ORDER      staged: TtMove, GoodCaptures, Quiets, BadCaptures.
///              full (root and in-check): TtMove, AllRemaining.
///   DUPLICATES the TT move is emitted at most once, and every later stage
///              filters it out, so no move is ever emitted twice.
///   LEGALITY   both paths only ever emit moves from a legal generator; the
///              TT move is validated by the caller before construction.
///
/// Stages are visited in declaration order and never revisited. `Done` is
/// terminal: once reached the picker yields `None` forever.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Stage {
    /// Emit the TT move alone, if there is one.
    TtMove,
    /// Staged path: captures with SEE >= 0, best-first.
    GoodCaptures,
    /// Staged path: generate quiets on demand, once.
    GenerateQuiets,
    /// Staged path: quiets, best-first.
    Quiets,
    /// Staged path: SEE-losing captures, best-first, deliberately last.
    BadCaptures,
    /// Full path: everything except the TT move, best-first from one list.
    AllRemaining,
    /// Terminal.
    Done,
}

enum MovePicker {
    Full {
        scored: ScoredMoveList,
        index: usize,
        tt_move: Move,
        stage: Stage,
    },
    /// 10.3(4): ONE buffer, partitioned in place, instead of three separate
    /// 3,080-byte lists. Layout — `[0, good_len)` good captures,
    /// `[good_len, cap_len)` bad captures, `[cap_len, len)` quiets. Every
    /// push is sequential, so `ScoredMoveList`'s prefix-initialization
    /// invariant is untouched. Total legal moves in any position (~218) fit
    /// the 256 capacity, so captures and quiets provably coexist.
    Staged {
        moves: ScoredMoveList,
        good_len: usize,
        cap_len: usize,
        /// Cursor within `[0, good_len)`.
        good_index: usize,
        /// Cursor within `[cap_len, len)`, relative to `cap_len`.
        quiet_index: usize,
        /// Cursor within `[good_len, cap_len)`, relative to `good_len`.
        bad_index: usize,
        /// 10.3(5): the pinned set computed by capture generation, reused when
        /// quiets are generated later at this same node. `board` is restored
        /// by `unmake_move` between the two stages, so the position — and
        /// therefore the pin structure — is unchanged. 10.3(7) made this
        /// unconditional: the staged path no longer pre-scans for captures, so
        /// a pinned set always exists to share.
        pinned: Bitboard,
        tt_move: Move,
        stage: Stage,
        ply: usize,
    },
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
/// 4.5.1 THE REDUCTION CONTRACT'S INPUTS.
///
/// `lmr_reduction_units` took thirteen positional arguments and PLAN 4.5.1
/// named that as the defect: it was a pile of parameters rather than a
/// contract over the per-ply context, and adding an input meant editing three
/// signatures and hoping the two call sites stayed in step. They must, because
/// one computes the PROSPECTIVE depth the pruning consumers read and the other
/// computes the reduction actually applied; a `debug_assert_eq!` between them
/// is what keeps 4.6b's "ONE formula" rule honest.
///
/// Naming the inputs also removes the whole class of bug where two `bool`s or
/// two `i32`s are passed in the wrong order and still compile.
#[derive(Copy, Clone)]
struct ReductionInputs {
    depth: i32,
    searched: usize,
    is_quiet: bool,
    see: i32,
    tt_pv: bool,
    cut_node: bool,
    quiet_hist: i32,
    corr_abs: i32,
    ev_is_exact: bool,
    tt_move_is_null: bool,
    improving: bool,
    is_root: bool,
    /// Move count at the PARENT when it played into this node.
    parent_move_count: i32,
    /// Quiet history of the move that led to this node.
    parent_stat_score: i32,
    /// The node's TT move is a capture.
    tt_move_is_capture: bool,
    /// The node's TT move was proved singular by the extension search.
    tt_move_singular: bool,
}

#[derive(Copy, Clone)]
struct NodeContext {
    /// The move made AT this ply. `Move::NULL` when the ply holds no move.
    mv: Move,
    /// The piece that made `mv`. Only meaningful when `mv` is not null.
    piece: Piece,
    /// Static eval of the position at this ply, or `VALUE_NONE` in check.
    static_eval: i32,
    /// 4.5.1 MOVE COUNT at this ply when `mv` was made, i.e. how many moves
    /// this node had already searched. Read by the CHILD, which reduces less
    /// when its parent had to look at many moves before this one — a node
    /// whose parent was still searching late is less likely to be a clean cut.
    move_count: i32,
    /// 4.5.1 STAT SCORE of `mv`: the quiet history the reduction saw when this
    /// move was chosen. Read by the CHILD, which compares its own move's
    /// history against it. An absolute history says how good a move looks; the
    /// comparison says whether the position is getting better or worse for the
    /// side to move, which is what the reduction wants to know.
    stat_score: i32,
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
    cont_key: usize,
}

impl NodeContext {
    /// Row base for continuation tables at this ply.
    ///
    /// `cont_row_base(piece, to) == piece_to_index(piece, to) * PIECE_TO_SIZE`,
    /// so the stored key serves every continuation site and none of them needs
    /// to re-derive the pair from `mv`/`piece`.
    #[inline]
    fn cont_row_base(&self) -> usize {
        self.cont_key * PIECE_TO_SIZE
    }
}

impl Default for NodeContext {
    fn default() -> Self {
        Self {
            mv: Move::NULL,
            piece: Piece::Pawn,
            static_eval: VALUE_NONE,
            cont_key: 0,
            move_count: 0,
            stat_score: 0,
        }
    }
}

pub struct Searcher {
    tt: TranspositionTable,
    hash_mb: usize,
    worker_pool: WorkerPool,
    evaluator: Evaluator,
    shared_state: Option<Arc<SharedSearchState>>,
    nodes: u64,
    tb_hits: u64,
    seldepth: usize,
    stopped: bool,
    quit: bool,
    pondering: bool,
    ponderhit: bool,
    stop_on_ponderhit: bool,
    start: Instant,
    limits: RuntimeLimits,
    pv_table: [[Move; MAX_PLY]; MAX_PLY],
    pv_len: [usize; MAX_PLY],
    /// 4.5.1 per-ply search context. See `NodeContext`.
    stack: [NodeContext; MAX_PLY],
    killers: [[Move; 2]; MAX_PLY],
    /// Compact root-order/index backbone. Keep this separate from the larger
    /// records below so existing move-membership and SMP hot reads retain
    /// their pre-10.1 cache layout.
    root_moves: Vec<Move>,
    root_move_records: Vec<RootMove>,
    lmr_table: Box<[[i32; 64]; 64]>,
    lmr_table_key: (i32, i32),
    main_history: Box<[[[i16; 64]; 64]; 2]>,
    cap_history: Box<[[[i16; 6]; 64]; 6]>,
    low_ply_history: Box<[[[i16; 64]; 64]; LOW_PLY_HISTORY_SIZE]>,
    /// 10.3(8a): boxed const-size, NOT `Vec<i16>` — see [`Searcher::cont_history`].
    pawn_history: Box<[i16; PAWN_HISTORY_SIZE * PIECE_TO_SIZE]>,
    /// Continuation history, one table per look-back distance. Indexed by
    /// [`CONT_PLY_BACK`] position, NOT by ply distance — see that table.
    ///
    /// KEEP-PERF (10.3, 2026-07-22): `Box<[[i16; CONT_SIZE]; N]>`, NOT
    /// `[Vec<i16>; N]`. The Vec form was bisected to a −2.1% NPS regression
    /// (commit 886916b, isolated by a 7-waypoint compiler-fixed bisect): four
    /// separate Vec headers with *runtime* lengths defeat bounds-check
    /// elision in the hottest loops in the engine. With a boxed array the
    /// inner length is a compile-time constant, so `cont_index`'s
    /// `.min(CONT_SIZE − 1)` lets LLVM prove both index bounds and drop the
    /// checks, and there is one base pointer instead of four.
    cont_history: Box<[[i16; CONT_SIZE]; CONT_TABLES]>,
    correction_history: Box<[[i16; CORR_SIZE]; 2]>,
    minor_correction_history: Box<[[i16; CORR_SIZE]; 2]>,
    non_pawn_correction_history: Box<[[[i16; CORR_SIZE]; 2]; 2]>,
    /// 10.3(8a): boxed const-size, see [`Searcher::pawn_history`].
    continuation_correction_history: Box<[i16; PIECE_TO_SIZE]>,
    /// 4.5b: continuation correction at 2- and 4-ply distance.
    ///
    /// The pre-4.5b model had a SINGLE slot keyed on the 1-ply-previous
    /// `(piece, to)`, so a correction learned from one reply was the only
    /// continuation context available. These add the same compact keying at
    /// distance 2 and 4, which is what PLAN 4.5's "true compact 2/4-ply
    /// continuation-correction pairs" asks for; the three together form the pair
    /// structure rather than a single slot standing in for it.
    ///
    /// Both are inert at the seeded weights of 0: `corrected_eval_from_raw`
    /// skips the read and `update_correction` skips the write, so neither table
    /// is even touched and `bench` is unchanged.
    continuation_correction_2ply: Box<[i16; PIECE_TO_SIZE]>,
    continuation_correction_4ply: Box<[i16; PIECE_TO_SIZE]>,
    countermove: Box<[[Move; 64]; 64]>,
    root_move_offset: usize,
    /// 8.13: 0 = main thread, 1.. = helper index. Seeds the reduction jitter.
    thread_id: usize,
    /// 9.7.5(k) xorshift64 state for the per-thread LMR jitter. Re-seeded from
    /// `thread_id` on every `reset_search_state`; never zero.
    jitter_state: u64,
    syzygy_probe_depth: i32,
    syzygy_probe_limit: usize,
    syzygy_50_move_rule: bool,
    syzygy_largest: usize,
    params: SearchParams,
    root_iteration_nodes: u64,
    root_best_nodes: u64,
    root_best_effort: f64,
    /// Non-zero while a verified null cutoff is being re-searched.
    ///
    /// 4.4a promoted this from a diagnostic-only counter to a production one:
    /// NMP verification passes `allow_null = false` at its own root only, so
    /// descendants re-enable null and the subtree can null-prune inside the very
    /// search meant to check a null cutoff. With
    /// `NmpSuppressNullInVerification` on, this field suppresses NMP for the
    /// whole subtree instead of just its root.
    nmp_verify_nesting: usize,
}

impl Default for Searcher {
    fn default() -> Self {
        Self {
            tt: TranspositionTable::default(),
            hash_mb: 64,
            worker_pool: WorkerPool::default(),
            evaluator: Evaluator::default(),
            shared_state: None,
            nodes: 0,
            tb_hits: 0,
            seldepth: 0,
            stopped: false,
            quit: false,
            pondering: false,
            ponderhit: false,
            stop_on_ponderhit: false,
            start: Instant::now(),
            limits: RuntimeLimits {
                depth: MAX_DEPTH,
                nodes: 0,
                optimum_ms: f64::INFINITY,
                maximum_ms: f64::INFINITY,
                movetime_mode: false,
                analysis_mode: false,
            },
            pv_table: [[Move::NULL; MAX_PLY]; MAX_PLY],
            pv_len: [0; MAX_PLY],
            stack: [NodeContext::default(); MAX_PLY],
            killers: [[Move::NULL; 2]; MAX_PLY],
            root_moves: Vec::new(),
            root_move_records: Vec::new(),
            lmr_table: build_lmr_table(768, 2304),
            lmr_table_key: (768, 2304),
            main_history: Box::new([[[0; 64]; 64]; 2]),
            cap_history: Box::new([[[0; 6]; 64]; 6]),
            low_ply_history: Box::new([[[0; 64]; 64]; LOW_PLY_HISTORY_SIZE]),
            pawn_history: Box::new([0; PAWN_HISTORY_SIZE * PIECE_TO_SIZE]),
            cont_history: boxed_cont_tables(),
            correction_history: Box::new([[0; CORR_SIZE]; 2]),
            minor_correction_history: Box::new([[0; CORR_SIZE]; 2]),
            non_pawn_correction_history: Box::new([[[0; CORR_SIZE]; 2]; 2]),
            continuation_correction_history: Box::new([0; PIECE_TO_SIZE]),
            continuation_correction_2ply: Box::new([0; PIECE_TO_SIZE]),
            continuation_correction_4ply: Box::new([0; PIECE_TO_SIZE]),
            countermove: Box::new([[Move::NULL; 64]; 64]),
            root_move_offset: 0,
            thread_id: 0,
            jitter_state: JITTER_SEED,
            syzygy_probe_depth: 1,
            syzygy_probe_limit: 7,
            syzygy_50_move_rule: true,
            syzygy_largest: 0,
            params: SearchParams::default(),
            root_iteration_nodes: 0,
            root_best_nodes: 0,
            root_best_effort: 0.0,
            nmp_verify_nesting: 0,
        }
    }
}

impl MovePicker {
    fn full(scored: ScoredMoveList, tt_move: Move) -> Self {
        Self::Full {
            scored,
            index: 0,
            tt_move,
            stage: Stage::TtMove,
        }
    }

    fn staged(searcher: &Searcher, board: &mut Board, tt_move: Move, ply: usize) -> Self {
        let mut captures = MoveList::new();
        let pinned = board.generate_legal_captures_pinned_into(&mut captures);
        let (moves, good_len, cap_len) =
            searcher.score_staged_captures(board, captures.as_slice(), tt_move);
        Self::Staged {
            moves,
            good_len,
            cap_len,
            good_index: 0,
            quiet_index: 0,
            bad_index: 0,
            pinned,
            tt_move,
            stage: Stage::TtMove,
            ply,
        }
    }

    /// Abandon the remaining quiet moves.
    ///
    /// 4.6.5: the reference feeds its move-count-pruning flag back INTO the
    /// picker, so once the flag is set it stops emitting quiets altogether.
    /// Rarog had no such path: it generated every quiet, scored it, and then
    /// rejected them one at a time in the move loop. Skipping from
    /// `GenerateQuiets` also avoids generating and scoring them at all, which
    /// is where most of the saving is.
    ///
    /// Safe from either stage: `BadCaptures` is next in declaration order, and
    /// the bad-capture partition is independent of the quiet buffer.
    fn skip_quiets(&mut self) {
        if let Self::Staged { stage, .. } = self
            && matches!(*stage, Stage::GenerateQuiets | Stage::Quiets)
        {
            *stage = Stage::BadCaptures;
        }
    }

    /// Yield the next move, advancing through `Stage` in declaration order.
    ///
    /// Every stage is a `loop`+`match` step rather than a fallthrough chain, so
    /// a stage that runs dry advances exactly once and the order is readable
    /// without reconstructing the buffer partition.
    fn next(&mut self, searcher: &Searcher, board: &mut Board) -> Option<ScoredMove> {
        match self {
            Self::Full {
                scored,
                index,
                tt_move,
                stage,
            } => loop {
                match *stage {
                    Stage::TtMove => {
                        *stage = Stage::AllRemaining;
                        if !tt_move.is_null() {
                            return Some(tt_scored_move(*tt_move));
                        }
                    }
                    Stage::AllRemaining => {
                        while *index < scored.len() {
                            let picked = pick_next(scored.as_mut_slice(), *index);
                            *index += 1;
                            // Duplicate guarantee: the TT move already went out
                            // in Stage::TtMove.
                            if picked.mv != *tt_move {
                                return Some(picked);
                            }
                        }
                        *stage = Stage::Done;
                    }
                    _ => return None,
                }
            },
            Self::Staged {
                moves,
                good_len,
                cap_len,
                good_index,
                quiet_index,
                bad_index,
                pinned,
                tt_move,
                stage,
                ply,
            } => loop {
                match *stage {
                    Stage::TtMove => {
                        *stage = Stage::GoodCaptures;
                        if !tt_move.is_null() {
                            return Some(tt_scored_move(*tt_move));
                        }
                    }
                    Stage::GoodCaptures => {
                        // The selection scan is bounded to the good partition,
                        // so it can never pull a bad capture forward.
                        while *good_index < *good_len {
                            let picked =
                                pick_next(&mut moves.as_mut_slice()[..*good_len], *good_index);
                            *good_index += 1;
                            if picked.mv != *tt_move {
                                return Some(picked);
                            }
                        }
                        *stage = Stage::GenerateQuiets;
                    }
                    Stage::GenerateQuiets => {
                        // Once, on demand, appended after the captures. The
                        // stage exists so "have the quiets been generated" is a
                        // position in the order rather than a bool.
                        let mut quiet_moves = MoveList::new();
                        board.generate_legal_quiets_pinned_into(*pinned, &mut quiet_moves);
                        searcher.append_scored_moves(
                            board,
                            quiet_moves.as_slice(),
                            *tt_move,
                            *ply,
                            moves,
                        );
                        *stage = Stage::Quiets;
                    }
                    Stage::Quiets => {
                        while *cap_len + *quiet_index < moves.len() {
                            let picked =
                                pick_next(&mut moves.as_mut_slice()[*cap_len..], *quiet_index);
                            *quiet_index += 1;
                            if picked.mv != *tt_move {
                                return Some(picked);
                            }
                        }
                        *stage = Stage::BadCaptures;
                    }
                    Stage::BadCaptures => {
                        while *good_len + *bad_index < *cap_len {
                            let picked = pick_next(
                                &mut moves.as_mut_slice()[*good_len..*cap_len],
                                *bad_index,
                            );
                            *bad_index += 1;
                            if picked.mv != *tt_move {
                                return Some(picked);
                            }
                        }
                        *stage = Stage::Done;
                    }
                    _ => return None,
                }
            },
        }
    }
}

fn tt_scored_move(mv: Move) -> ScoredMove {
    let see = if mv.is_capture() { SEE_UNKNOWN } else { 0 };
    ScoredMove {
        mv,
        score: 30_000_000,
        see,
        quiet_history: 0,
    }
}

impl Searcher {
    pub(crate) fn worker_default() -> Self {
        Self {
            worker_pool: WorkerPool::default(),
            ..Self::default()
        }
    }

    pub(crate) fn reset_worker_state_for_new_game(&mut self) {
        self.clear_history();
        self.evaluator.clear_pawn_table();
    }

    pub(crate) fn run_worker_job<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        job: WorkerJob,
        poll: &mut P,
    ) -> SearchResult {
        self.tt = job.tt;
        self.hash_mb = job.hash_mb;
        self.shared_state = Some(Arc::clone(&job.shared_state));
        self.root_move_offset = job.root_move_offset;
        self.thread_id = job.thread_id;
        let result = self.search_worker(
            job.root,
            &job.limits,
            &job.engine_options,
            job.root_moves.as_ref(),
            poll,
        );
        self.shared_state = None;
        result
    }

    pub fn configure(&mut self, options: &SearchOptions) {
        self.configure_engine(&options.engine);
    }

    fn configure_engine(&mut self, options: &EngineOptions) {
        if options.hash_mb != self.hash_mb {
            if self.tt.resize(options.hash_mb) {
                self.hash_mb = options.hash_mb;
            } else {
                crate::info_string!(
                    "Unable to allocate Hash value {}; keeping {} MiB.",
                    options.hash_mb,
                    self.hash_mb
                );
            }
        }
        if options.clear_hash {
            self.tt.clear();
        }
        let old_path = syzygy::current_path();
        let largest = syzygy::initialize(&options.syzygy.path);
        if old_path != options.syzygy.path && !options.syzygy.path.is_empty() {
            if largest == 0 {
                crate::info_string!("SyzygyPath loaded no usable tablebases.");
            } else {
                let (wdl, dtz) = syzygy::tablebase_file_counts(&options.syzygy.path);
                crate::info_string!(
                    "Found {wdl} WDL and {dtz} DTZ tablebase files (up to {largest}-man)."
                );
            }
        }
        self.worker_pool
            .set_helper_count(options.threads.saturating_sub(1));
    }

    pub fn new_game(&mut self) {
        self.tt.clear();
        self.clear_history();
        self.evaluator.clear_pawn_table();
        self.worker_pool.new_game();
    }

    pub fn clear_history(&mut self) {
        *self.main_history = [[[0; 64]; 64]; 2];
        *self.cap_history = [[[0; 6]; 64]; 6];
        *self.low_ply_history = [[[0; 64]; 64]; LOW_PLY_HISTORY_SIZE];
        *self.pawn_history = [0; PAWN_HISTORY_SIZE * PIECE_TO_SIZE];
        for table in self.cont_history.iter_mut() {
            table.fill(0);
        }
        *self.correction_history = [[0; CORR_SIZE]; 2];
        *self.minor_correction_history = [[0; CORR_SIZE]; 2];
        *self.non_pawn_correction_history = [[[0; CORR_SIZE]; 2]; 2];
        *self.continuation_correction_history = [0; PIECE_TO_SIZE];
        *self.continuation_correction_2ply = [0; PIECE_TO_SIZE];
        *self.continuation_correction_4ply = [0; PIECE_TO_SIZE];
        *self.countermove = [[Move::NULL; 64]; 64];
        self.killers = [[Move::NULL; 2]; MAX_PLY];
    }

    pub fn hashfull(&self) -> usize {
        self.tt.hashfull()
    }

    pub fn search(
        &mut self,
        root: Board,
        options: &SearchOptions,
        emit_info: bool,
        mut poll: impl FnMut() -> SearchEvent,
    ) -> SearchResult {
        self.search_impl::<true, _>(
            root,
            // 9.0a: both by reference — this used to clone a SearchLimits AND
            // a whole EngineOptions (SearchParams included) per search entry.
            &options.limits,
            &options.engine,
            emit_info,
            &mut poll,
        )
    }

    fn search_impl<const ALLOW_PARALLEL: bool, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        mut root: Board,
        limits: &SearchLimits,
        engine_options: &EngineOptions,
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        if ALLOW_PARALLEL && engine_options.threads <= 1 && !self.tt.ensure_local(self.hash_mb) {
            crate::info_string!(
                "Unable to restore local transposition table at {} MiB.",
                self.hash_mb
            );
        }
        self.shared_state = None;
        self.root_move_offset = 0;
        self.thread_id = 0;

        // Reserve the whole search's history headroom once, before any hot
        // path or helper exists. Search pushes one UnmakeInfo per ply and pops
        // it again, so MAX_PLY bounds the peak above the game history already
        // present. Board::clone preserves capacity, so every worker's
        // root.clone() inherits this and no thread reallocates while searching.
        root.reserve_history(MAX_PLY);

        let game_ply = 2 * root.fullmove.saturating_sub(1) as u32
            + (root.side_to_move() == Color::Black) as u32;
        self.reset_search_state(
            limits,
            engine_options,
            root.side_to_move(),
            game_ply,
            true,
            true,
        );

        let board = root;
        let mut legal_moves = MoveList::new();
        board.generate_legal_movelist_into(&mut legal_moves);
        if legal_moves.is_empty() {
            return self.no_legal_moves_result(&board);
        }

        let filtered_root_moves;
        let root_candidates = if limits.search_moves.is_empty() {
            legal_moves.as_slice()
        } else {
            filtered_root_moves = legal_moves
                .iter()
                .copied()
                .filter(|mv| {
                    limits
                        .search_moves
                        .iter()
                        .any(|requested| mv.same_uci_move(*requested))
                })
                .collect::<Vec<_>>();
            if filtered_root_moves.is_empty() {
                legal_moves.as_slice()
            } else {
                filtered_root_moves.as_slice()
            }
        };

        let syzygy_root_moves = self.syzygy_root_moves(&board, root_candidates);
        let root_moves = syzygy_root_moves.as_deref().unwrap_or(root_candidates);

        if ALLOW_PARALLEL {
            let threads = engine_options.threads.clamp(1, MAX_THREADS);
            if threads > 1 && self.limits.depth.min(MAX_DEPTH - 1) >= MIN_PARALLEL_DEPTH {
                return self.search_parallel(
                    board,
                    root_moves,
                    limits,
                    engine_options.clone(),
                    threads,
                    emit_info,
                    poll,
                );
            }
        }

        self.search_root(board, root_moves, emit_info, poll)
    }

    fn reset_search_state(
        &mut self,
        limits: &SearchLimits,
        engine_options: &EngineOptions,
        side_to_move: Color,
        game_ply: u32,
        age_tt: bool,
        age_history: bool,
    ) {
        // LazyMargin changes the raw evaluation function. The evaluator owns
        // its whole-eval cache, while the main searcher owns TT lifecycle and
        // must also discard stored raw evals and bounds from the old function.
        // Helpers receive the already-cleared shared TT, so they clear only
        // their private evaluator cache; clearing shared storage here would
        // race with helpers or main already searching.
        let lazy_margin_changed = self
            .evaluator
            .set_lazy_margin(engine_options.search_params.lazy_margin);
        if lazy_margin_changed && age_tt {
            self.tt.clear();
        }

        // The clock starts when `go` was parsed, as the harness measures it;
        // configuration invalidation and thread hand-off are on the clock
        // because they are on the harness's clock (A.3.3, RAR-R11).
        self.start = limits.issued.unwrap_or_else(Instant::now);
        self.nodes = 0;
        self.tb_hits = 0;
        self.seldepth = 0;
        self.stopped = false;
        self.quit = false;
        self.pondering = limits.ponder;
        self.ponderhit = false;
        self.stop_on_ponderhit = false;
        self.limits =
            compute_runtime_limits(limits, engine_options, side_to_move, game_ply, MAX_DEPTH);
        self.syzygy_probe_depth = engine_options.syzygy.probe_depth;
        self.syzygy_probe_limit = engine_options.syzygy.probe_limit;
        self.syzygy_50_move_rule = engine_options.syzygy.fifty_move_rule;
        self.params = engine_options.search_params.clone();
        let table_key = (self.params.lmr_table_base, self.params.lmr_table_div);
        if table_key != self.lmr_table_key {
            self.lmr_table = build_lmr_table(table_key.0, table_key.1);
            self.lmr_table_key = table_key;
        }
        self.syzygy_largest = syzygy::largest().min(self.syzygy_probe_limit);
        self.root_iteration_nodes = 0;
        self.root_best_nodes = 0;
        self.root_best_effort = 0.0;
        self.nmp_verify_nesting = 0;
        if age_tt {
            self.tt.new_search();
        }
        if age_history {
            self.age_history();
        }
        self.pv_table = [[Move::NULL; MAX_PLY]; MAX_PLY];
        self.pv_len = [0; MAX_PLY];
        self.stack = [NodeContext::default(); MAX_PLY];
        // 9.7.5(k): re-seed the LMR-jitter PRNG per search, per thread, so each
        // thread walks a different sequence and a given thread's sequence does
        // not depend on how the previous search happened to end. `thread_id` is
        // bounded by MAX_THREADS so the conversion always succeeds; a fallback
        // seed would only pick a different sequence, never break anything.
        let thread_seed = u64::try_from(self.thread_id).unwrap_or(0);
        self.jitter_state = JITTER_SEED ^ thread_seed.wrapping_mul(JITTER_STRIDE) | 1;
    }

    /// One xorshift64 step, mapped to LMR-reduction jitter in 1024ths of a ply.
    ///
    /// 9.7.5(k) replaces `(nodes + id·27) % 128 − 59`, which had two defects a
    /// PRNG does not: it was **correlated with the node counter** (consecutive
    /// nodes got consecutive jitter, so "random" perturbation moved in ramps),
    /// and it was **biased +4.5/1024**, quietly raising every thread's mean
    /// reduction rather than only spreading it. The range here is [−64, 63], the
    /// same amplitude as before, with mean −0.5/1024 — nine times closer to
    /// zero, so the jitter now diversifies without also pruning harder.
    #[inline(always)]
    fn next_jitter(&mut self, magnitude: i32) -> i32 {
        let mut x = self.jitter_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.jitter_state = x;
        // Top 7 bits, not the bottom ones: xorshift64's low bits are its
        // weakest (they carry the least mixing), and taking them measurably
        // skewed the mean. `>> 57` yields 0..=127, so the result is [−64, 63].
        // `magnitude` in 1024ths of a ply; the result is [−magnitude,
        // +magnitude]. At magnitude 64 this is EXACTLY the pre-4.5 expression
        // `(x >> 57) - 64`, since `bits * 64 / 64 - 64 == bits - 64`, so the
        // SMP path is unchanged by construction rather than by measurement.
        let bits = i32::try_from(x >> 57).expect("7-bit shift fits i32");
        bits * magnitude / 64 - magnitude
    }

    fn no_legal_moves_result(&mut self, board: &Board) -> SearchResult {
        let result = self.result_for_no_legal_moves(board);
        SearchResult {
            bestmove: Move::NULL,
            pondermove: Move::NULL,
            score: self
                .evaluator
                .evaluate_result(result, board.side_to_move(), 0),
            depth: 0,
            nodes: 0,
            tb_hits: self.tb_hits,
            elapsed_ms: self.start.elapsed().as_millis(),
            exit: SearchExit::Stop,
            ponderhit: self.ponderhit,
        }
    }

    fn syzygy_root_moves(&mut self, board: &Board, legal_moves: &[Move]) -> Option<Vec<Move>> {
        if !self.can_probe_syzygy_root(board) || board.can_declare_draw() || self.limits.nodes > 0 {
            return None;
        }

        let probe = syzygy::probe_root_moves(
            board,
            self.syzygy_50_move_rule,
            board.has_repeated_position(),
        )?;
        self.record_tb_hit();

        let mut tb_moves = Vec::new();
        for probe_move in &probe.moves {
            let Some(mv) = syzygy::legal_move_from_root_probe(board, probe_move.root_move) else {
                continue;
            };
            if legal_moves.contains(&mv) {
                tb_moves.push((mv, probe_move.rank, probe_move.score));
            }
        }

        let best_rank = tb_moves.iter().map(|(_, rank, _)| *rank).max()?;
        let preferred_move = if probe.used_dtz && best_rank != 0 {
            syzygy::probe_root(board, self.syzygy_50_move_rule)
                .and_then(|probe| probe.best_move)
                .and_then(|root_move| syzygy::legal_move_from_root_probe(board, root_move))
        } else {
            None
        };

        if best_rank != 0
            && let Some(preferred_move) = preferred_move
            && tb_moves
                .iter()
                .any(|(tb_move, rank, _)| *tb_move == preferred_move && *rank == best_rank)
        {
            self.record_tb_hit();
            return Some(vec![preferred_move]);
        }

        let mut root_moves = Vec::with_capacity(legal_moves.len());
        for &legal_move in legal_moves {
            if tb_moves
                .iter()
                .any(|(tb_move, rank, _)| *tb_move == legal_move && *rank == best_rank)
            {
                root_moves.push(legal_move);
            }
        }

        if root_moves.is_empty() {
            None
        } else {
            Some(root_moves)
        }
    }

    /// Summarise the iteration that has just completed (4.7b).
    ///
    /// Cold and never inlined for the same reason `RootMove::record_search` is:
    /// it runs a few dozen times per search, and letting a floating-point
    /// statistics block into `negamax`'s code layout measurably cost NPS when
    /// 10.1 first tried it.
    #[cold]
    #[inline(never)]
    fn root_confidence(
        &self,
        depth: usize,
        bestmove: Move,
        best_score: i32,
        instability: f64,
        fail_lows: i32,
        fail_highs: i32,
    ) -> RootConfidence {
        let best = self.root_move_records.iter().find(|rm| rm.mv == bestmove);
        // The best of the REST, restricted to moves this iteration actually
        // searched — a stale score from an earlier depth is not evidence about
        // this one. With no rival searched the gap is 0, i.e. no separation is
        // claimed rather than an infinite one.
        let second_score = self
            .root_move_records
            .iter()
            .filter(|rm| rm.last_search_depth == depth && rm.mv != bestmove)
            .map(|rm| rm.score)
            .max()
            .unwrap_or(best_score);
        let variance = best.map_or(0.0, |rm| {
            (rm.mean_squared_score - rm.average_score * rm.average_score).max(0.0)
        });
        // KEEP-ALLOW: the square root of a non-negative variance is clamped
        // into the score domain BEFORE the cast, so the conversion is provably
        // exact rather than saturating, and NaN is handled by the clamp instead
        // of the language's float-cast rule.
        #[expect(clippy::cast_possible_truncation)]
        let deviation = variance.sqrt().clamp(0.0, f64::from(INF_SCORE)).round() as i32;
        // Pooling is a CLOCK input, so it is admitted only when its own switch
        // is on and only when a pool exists; a serial search has no shared
        // state and this stays `None`.
        let pool_instability = self
            .shared_state
            .as_ref()
            .filter(|_| self.params.root_conf_pool_instability == 1)
            .and_then(|shared| shared.pooled_instability())
            .map(|milli| {
                // KEEP-ALLOW: `u64 -> f64` on a value bounded by the published
                // thousandths of a decaying count that converges to 2.0.
                #[expect(clippy::cast_precision_loss)]
                let milli = milli as f64;
                milli / 1000.0
            });
        RootConfidence {
            gap: best_score.saturating_sub(second_score),
            deviation,
            effort: self.root_best_effort,
            fail_lows,
            fail_highs,
            instability,
            pool_instability,
        }
    }

    fn search_root<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        mut board: Board,
        legal_moves: &[Move],
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        // 9.7.5(b): the SERIAL path owns the diag lifecycle here. In a parallel
        // search `search_parallel` resets before spawning and dumps after
        // joining — helpers reach this function too, so a reset/dump left
        // unconditional ran once PER THREAD, wiping earlier threads' counts on
        // the way in and emitting N competing dumps on the way out. Every
        // multi-thread diag figure produced before this fix was junk.
        if self.shared_state.is_none() {
            crate::diag::reset();
        }
        self.root_moves.clear();
        self.root_moves.extend_from_slice(legal_moves);
        self.root_move_records.clear();
        self.root_move_records
            .extend(legal_moves.iter().copied().map(RootMove::new));
        let mut bestmove = legal_moves[0];
        let mut pondermove = Move::NULL;
        let mut best_score = -INF_SCORE;
        let mut completed_depth = 0;
        let max_depth = self.limits.depth.min(MAX_DEPTH - 1);
        let mut prev_avg_score = 0.0_f64; // EWMA of completed root scores (SF bestPreviousAverageScore)
        let mut tot_best_move_changes = 0.0_f64; // decaying count of best-move changes
        // 8.13: this thread's soft-stop vote is cast at most ONCE per search.
        // Without the latch a thread that keeps iterating past its own soft
        // target votes again every iteration and can reach the majority
        // single-handedly — which is the opposite of pooling the decision.
        let mut cast_stop_vote = false;
        // 4.7b: the last COMPLETED iteration's snapshot. `None` until one
        // completes, which is why aspiration cannot act on a partial root.
        let mut confidence: Option<RootConfidence> = None;
        // Depth at which the current best move took over. Diagnostic-only, and
        // compiled out otherwise: best-move age is MEASURED against instability
        // (see `RootConfidence::scalar`) precisely because it must not become a
        // second charge for the same fact, so production carries no state for
        // it.
        #[cfg(feature = "diag")]
        let mut best_since_depth = 0usize;

        for depth in 1..=max_depth {
            // 4.9c: helpers may skip this iteration so the pool spreads across
            // depths instead of stacking on one. Gated on `shared_state` as
            // well as `thread_id` so a serial search cannot reach it even if
            // the switch is on and a stale id survived.
            if self.params.smp_iteration_skip == 1
                && self.shared_state.is_some()
                && helper_skips_iteration(self.thread_id, depth)
            {
                continue;
            }
            for root_move in &mut self.root_move_records {
                root_move.begin_iteration();
            }
            let previous_bestmove = bestmove;
            self.root_iteration_nodes = self.nodes;
            self.root_best_nodes = 0;
            self.root_best_effort = 0.0;
            // 8.13: the aspiration window centers on this thread's own last
            // completed score — unless the pool has already proven an Exact
            // root score DEEPER than this thread's progress, in which case it
            // centers on the pool's estimate (fewer fail-high/low re-searches
            // when joining the pool's view). Serial searches have no shared
            // state and keep `best_score` bit-for-bit.
            let mut window_center = best_score;
            if let Some(shared) = &self.shared_state
                && let Some((pool_depth, pool_score)) = shared.pool_best_exact()
                && pool_depth > infra::to_i32(completed_depth)
            {
                window_center = pool_score;
            }
            // 10.2(a): blend the last completed score with the running average
            // of completed scores. `asp_center_avg_pct == 0` skips this
            // entirely and keeps the pure last-score centre bit-for-bit.
            if self.params.asp_center_avg_pct > 0 && completed_depth > 1 {
                let pct = self.params.asp_center_avg_pct;
                // KEEP-ALLOW: the value is clamped into the score domain
                // BEFORE the cast, so this conversion is provably exact rather
                // than merely saturating — `[-INF_SCORE, INF_SCORE]` is far
                // inside `i32`, and `.round()` is the rounding intended. The
                // clamp also makes NaN handling explicit instead of relying on
                // the language's float-cast NaN rule.
                #[expect(clippy::cast_possible_truncation)]
                let avg = prev_avg_score
                    .round()
                    .clamp(-f64::from(INF_SCORE), f64::from(INF_SCORE))
                    as i32;
                window_center = (window_center * (100 - pct) + avg * pct) / 100;
            }
            let use_aspiration =
                depth >= 4 && window_center.abs() < MATE_SCORE - infra::to_i32(MAX_PLY);
            // 10.2(a): magnitude-scaled initial half-width; div 0 = flat.
            let base_delta = if self.params.asp_magnitude_div > 0 {
                self.params.aspiration_delta + window_center.abs() / self.params.asp_magnitude_div
            } else {
                self.params.aspiration_delta
            };
            // 4.7b: confidence sizes the window, asymmetrically, from the last
            // COMPLETED iteration. Off by default, and inert before the first
            // completed iteration whatever the switch says.
            let (mut alpha_delta, mut beta_delta) = match confidence {
                Some(snapshot) if self.params.root_conf_aspiration == 1 => {
                    snapshot.aspiration_deltas(base_delta, &self.params)
                }
                _ => (base_delta, base_delta),
            };
            let mut alpha = if use_aspiration {
                (window_center - alpha_delta).max(-INF_SCORE)
            } else {
                -INF_SCORE
            };
            let mut beta = if use_aspiration {
                (window_center + beta_delta).min(INF_SCORE)
            } else {
                INF_SCORE
            };
            // 10.2(a) TERMINATION BY CONSTRUCTION: once a side has failed
            // `asp_max_fails` times it is opened to ±INF and cannot fail again,
            // so this loop runs at most `2 * asp_max_fails` times whatever the
            // scores do. That is the property the old 7.0b guard bought with
            // mate-magnitude and saturation special cases; the counter makes it
            // structural instead of case-based.
            let mut fail_low_count = 0i32;
            let mut fail_high_count = 0i32;

            loop {
                // 10.2(a): confirm a fail-high at slightly reduced depth. The
                // reduction is 0 by default, which searches at full depth.
                let search_depth = (infra::to_i32(depth)
                    - fail_high_count * self.params.asp_fail_high_reduction)
                    .max(1);
                let score = self.negamax(
                    &mut board,
                    search_depth,
                    alpha,
                    beta,
                    0,
                    true,
                    true,
                    Move::NULL,
                    false,
                    poll,
                );
                if self.stopped || self.quit {
                    break;
                }
                // Termination guard (Phase 7.0b). The widened window re-centers
                // on the previous iteration's best_score; with the delta
                // clamped to INF_SCORE that caps the reachable bound at
                // best_score ± INF_SCORE, which can never contain a mate score
                // found *this* iteration when best_score is negative-ish
                // (prev + 32001 < mate) — the fail-high loop then never
                // terminates (WAC.005 hung every fixed-depth search ≥ 4; games
                // masked it because the clock aborts the iteration). Force the
                // failing side fully open once a mate-magnitude score appears
                // or the delta saturates; every other re-search keeps the old
                // best_score-centered dynamics exactly — the SF-style
                // "re-center on the failing score" variant was SPRT-rejected
                // (H0, −4.52 ± 4.80): AspirationDelta and the pruning group
                // were tuned around the old window dynamics (lesson 13).
                if score <= alpha {
                    crate::diag_count!(asp_fail_low);
                    fail_low_count += 1;
                    alpha_delta = (alpha_delta * self.params.asp_growth_pct / 100
                        + self.params.asp_growth_add)
                        .min(INF_SCORE);
                    alpha = if fail_low_count >= self.params.asp_max_fails
                        || alpha_delta >= INF_SCORE
                        || score <= -(MATE_SCORE - infra::to_i32(MAX_PLY))
                    {
                        -INF_SCORE
                    } else {
                        (window_center - alpha_delta).max(-INF_SCORE)
                    };
                    beta = (alpha + beta) / 2;
                    continue;
                }
                if score >= beta {
                    crate::diag_count!(asp_fail_high);
                    fail_high_count += 1;
                    beta_delta = (beta_delta * self.params.asp_growth_high_pct / 100
                        + self.params.asp_growth_add)
                        .min(INF_SCORE);
                    beta = if fail_high_count >= self.params.asp_max_fails
                        || beta_delta >= INF_SCORE
                        || score >= MATE_SCORE - infra::to_i32(MAX_PLY)
                    {
                        INF_SCORE
                    } else {
                        (window_center + beta_delta).min(INF_SCORE)
                    };
                    continue;
                }
                best_score = score;
                completed_depth = depth;
                let iteration_nodes = self.nodes.saturating_sub(self.root_iteration_nodes).max(1);
                self.root_best_effort = self.root_best_nodes as f64 / iteration_nodes as f64;
                if self.pv_len[0] > 0 {
                    bestmove = self.pv_table[0][0];
                    pondermove = if self.pv_len[0] > 1 {
                        self.pv_table[0][1]
                    } else {
                        Move::NULL
                    };
                }
                if let Some(root_move) = self
                    .root_move_records
                    .iter_mut()
                    .find(|rm| rm.mv == bestmove)
                {
                    root_move.last_best_depth = depth;
                }
                for root_move in &mut self.root_move_records {
                    if root_move.last_search_depth == depth {
                        root_move.complete_iteration();
                    }
                }
                break;
            }

            if self.stopped || self.quit {
                break;
            }

            if emit_info {
                self.send_info(depth, best_score);
            }

            // Only one legal move: it will be played whatever the score is, so
            // in a CLOCK-MANAGED search there is nothing to buy by searching on.
            //
            // Excluded for `infinite` and `ponder`, where the caller wants the
            // evaluation rather than the move: a GUI analysing a forced line
            // otherwise sees the search freeze at depth 2 with a meaningless
            // score. Stockfish behaves the same way — it moves instantly under
            // a clock but keeps searching under `go infinite`. Fixed-depth and
            // fixed-node searches keep the shortcut (bench relies on it, and
            // `go depth N` on a forced move is still a move request).
            if legal_moves.len() == 1 && depth >= 2 && !self.limits.analysis_mode {
                break;
            }

            // Update best-move instability and score EWMA for the soft-stop
            // formula. **This thread's own** best-move flips, deliberately:
            // 9.7.5(j) replaced this with the POOL's deepest-Exact move and
            // LOST at −5.54 ± 8.15 over 2,760 games at 4T. See PLAN 9.7.5(j)
            // for why the pool view is the noisier signal, not the better one.
            tot_best_move_changes /= 2.0;
            if bestmove != previous_bestmove {
                tot_best_move_changes += 1.0;
                #[cfg(feature = "diag")]
                {
                    best_since_depth = depth;
                }
            }

            // 4.7b: ONE snapshot of the completed iteration, built after the
            // instability update so it describes this iteration rather than the
            // previous one. Published before it is read, so a pooling thread
            // sees its own current value in the pool mean.
            if let Some(shared) = &self.shared_state {
                // KEEP-ALLOW: `tot_best_move_changes` is a decaying count that
                // converges to 2.0, so the thousandths are clamped into a small
                // range before the cast rather than merely saturating.
                #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let milli = (tot_best_move_changes * 1000.0).clamp(0.0, 1_000_000.0) as u64;
                shared.publish_instability(self.thread_id, milli);
            }
            let snapshot = self.root_confidence(
                depth,
                bestmove,
                best_score,
                tot_best_move_changes,
                fail_low_count,
                fail_high_count,
            );
            #[cfg(feature = "diag")]
            crate::diag::record_root_confidence(&crate::diag::RootConfidenceShadow {
                gap: snapshot.gap,
                deviation: snapshot.deviation,
                effort: snapshot.effort,
                best_changed: bestmove != previous_bestmove,
                // Recomputed rather than carried on the snapshot: production
                // has no use for it, and a diagnostic-only field inside the
                // model is exactly the kind of uncharged input 4.7b is trying
                // not to leave lying around.
                no_rival: !self
                    .root_move_records
                    .iter()
                    .any(|rm| rm.last_search_depth == depth && rm.mv != bestmove),
                scalar: snapshot.scalar(&self.params),
                instability: tot_best_move_changes,
                pooled_instability: self
                    .shared_state
                    .as_ref()
                    .and_then(|shared| shared.pooled_instability()),
                best_age: completed_depth.saturating_sub(best_since_depth),
                pv_len: self.pv_len[0],
                fails: fail_low_count + fail_high_count,
                // The two clock multipliers side by side, which is what sizes
                // the `RootConfTime` arm without a game. The live block below
                // keeps its own multiplication order so the arm-OFF path stays
                // bit-identical, so the baseline is recomposed here from the
                // same two shared factors rather than read back out of it.
                baseline_time: tm_instability_factor(&self.params, tot_best_move_changes)
                    * tm_effort_factor(&self.params, self.root_best_effort),
                candidate_time: snapshot.time_factor(&self.params),
            });
            confidence = Some(snapshot);
            // Phase 7.5 fix: `falling_eval` must compare this iteration's score
            // against the average of the *prior* iterations. At this point
            // `prev_avg_score` is still that prior average, so capture it here as
            // the baseline BEFORE folding the current score in below. The old
            // code read `prev_avg_score` only after the update, which made the
            // difference `(2/3)·(prior_avg − best_score)` — attenuating the
            // "score is falling → spend more time" signal to two-thirds and
            // contradicting the "feeds fallingEval next iteration" intent. On
            // the first iteration there is no prior average, so the baseline is
            // the current score → a neutral (zero) falling signal.
            let falling_baseline = if completed_depth <= 1 {
                best_score as f64
            } else {
                prev_avg_score
            };
            // Update the EWMA for the next iteration's baseline.
            prev_avg_score = if completed_depth <= 1 {
                best_score as f64
            } else {
                (prev_avg_score * 2.0 + best_score as f64) / 3.0
            };

            // SF-style between-iteration stop.
            // movetime mode: no soft stop — check_stop (every 2048 nodes) fires at maximum_ms.
            // clock mode: stop when elapsed exceeds the dynamically scaled optimum.
            let elapsed_ms = self.elapsed_ms();
            if elapsed_ms >= self.limits.maximum_ms {
                break;
            }
            if !self.limits.movetime_mode {
                // TM dynamic multipliers (Phase 5.1 TM group). Stored ×10000 in
                // SearchParams; `/ 10000.0` reconstructs the 2.2 SF seeds bit-exactly.
                let opt_scale = self.params.tm_opt_scale as f64 / 10_000.0;
                let fall_base = self.params.tm_fall_base as f64 / 10_000.0;
                let fall_slope = self.params.tm_fall_slope as f64 / 10_000.0;
                // fallingEval: ↑ when score is falling (want more time); seeds from SF.
                let falling_eval = (fall_base
                    + fall_slope * (falling_baseline - best_score as f64))
                    .clamp(0.572, 1.708);
                // bestMoveInstab: ↑ when best move changed recently.
                let best_move_instab = tm_instability_factor(&self.params, tot_best_move_changes);
                // effortFactor: linear interp — at effort≤0.79 → effort_high; at effort≥1.0 → effort_low.
                let effort_factor = tm_effort_factor(&self.params, self.root_best_effort);
                // 4.7b: with `RootConfTime` on, the confidence factor REPLACES
                // the instability × effort product — it does not multiply on
                // top of it, which is what would double-charge effort (it is
                // already a term inside the scalar) and instability (already
                // the snapshot's own time-only input).
                //
                // The arm-OFF branch keeps the original multiplication order
                // literally, so the shipped soft target is bit-identical.
                let total_time = match confidence {
                    Some(snapshot) if self.params.root_conf_time == 1 => {
                        self.limits.optimum_ms
                            * opt_scale
                            * falling_eval
                            * snapshot.time_factor(&self.params)
                    }
                    _ => {
                        self.limits.optimum_ms
                            * opt_scale
                            * falling_eval
                            * best_move_instab
                            * effort_factor
                    }
                };
                let soft_target = total_time.min(self.limits.maximum_ms);
                if self.pondering {
                    // While pondering: flag to stop immediately on ponderhit.
                    if elapsed_ms >= soft_target {
                        self.stop_on_ponderhit = true;
                    }
                } else if elapsed_ms >= soft_target {
                    // 8.13: in a parallel search the soft stop is a SYMMETRIC
                    // pool decision. A thread whose own soft target expires
                    // casts one vote (latched — re-voting each iteration would
                    // let a single thread reach the majority alone) and keeps
                    // searching; the search ends when a strict majority
                    // agrees. The main thread's expiry is just one vote like
                    // everyone else's, so the pool can EXTEND main past its
                    // noisy solo estimate as well as cut it short — N clamped
                    // opinions instead of 1. Bounded above by `maximum_ms`
                    // (checked before this block and inside the tree every
                    // poll), which the SMP-aware time reserve keeps
                    // forfeit-safe; measured 0 forfeits across every 8.13 run.
                    // Serial searches have no shared state and break at their
                    // own target exactly as before.
                    if let Some(shared) = &self.shared_state {
                        if !cast_stop_vote {
                            cast_stop_vote = true;
                            if shared.vote_to_stop() {
                                shared.request_stop();
                            }
                        }
                        if self.thread_id != 0 {
                            // Helpers keep searching until the pool agrees, so
                            // their remaining time still fills the shared TT.
                            continue;
                        }
                        if shared.stop_state.load(Ordering::Relaxed) == STOP_NONE {
                            // Main defers to the pool: no majority yet, so
                            // keep iterating. When the majority lands (any
                            // thread's vote, including the one just cast),
                            // the STOP_SEARCH state ends the search via the
                            // poll or this check next iteration.
                            continue;
                        }
                    }
                    break;
                }
            }
        }

        if pondermove.is_null() {
            pondermove = self.ponder_from_tt(&board, bestmove);
        }

        #[cfg(feature = "diag")]
        if (self.stopped || self.quit)
            && self
                .root_move_records
                .iter()
                .any(|rm| rm.last_search_depth > completed_depth)
        {
            crate::diag_count!(root_interrupted_fallback);
        }

        // Phase 4.1: dump per-search counters (no-op without `--features diag`).
        // 9.7.5(b): serial path only — see the reset note above. The parallel
        // dump lives in `search_parallel`, after the helpers are joined.
        crate::diag::record_thread_depth(self.thread_id, completed_depth);
        if self.shared_state.is_none() {
            crate::diag::dump();
        }

        SearchResult {
            bestmove,
            pondermove,
            score: best_score,
            depth: completed_depth,
            nodes: self.nodes,
            tb_hits: self.tb_hits,
            elapsed_ms: self.start.elapsed().as_millis(),
            exit: if self.quit {
                SearchExit::Quit
            } else {
                SearchExit::Stop
            },
            ponderhit: self.ponderhit,
        }
    }

    fn search_worker<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        root: Board,
        limits: &SearchLimits,
        engine_options: &EngineOptions,
        legal_moves: &[Move],
        poll: &mut P,
    ) -> SearchResult {
        let game_ply = 2 * root.fullmove.saturating_sub(1) as u32
            + (root.side_to_move() == Color::Black) as u32;
        // 8.13(a): helpers must NOT inherit the main thread's fixed depth.
        //
        // Under a clock this is invisible — every thread runs until the main
        // thread's time manager stops the pool. But under `go depth N` a helper
        // that reaches N returns and then sits idle for the rest of the search,
        // contributing nothing while the main thread finishes. Helpers exist to
        // widen the shared TT, so they should keep going until stopped; the
        // main thread alone owns the depth contract and the reported result.
        let mut helper_limits = limits.clone();
        helper_limits.depth = None;
        self.reset_search_state(
            &helper_limits,
            engine_options,
            root.side_to_move(),
            game_ply,
            false,
            true,
        );
        self.search_root(root, legal_moves, false, poll)
    }

    #[cold]
    #[inline(never)]
    fn search_parallel<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        root: Board,
        root_moves: &[Move],
        limits: &SearchLimits,
        engine_options: EngineOptions,
        threads: usize,
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        // 9.7.5(b): reset BEFORE any helper exists, so nothing already counted
        // gets wiped by a late-starting thread.
        crate::diag::reset();
        self.tt.make_shared(self.hash_mb);
        let helper_count = threads.saturating_sub(1);
        let root_len = root_moves.len();
        let shared_state = Arc::new(SharedSearchState::new(self.tb_hits, root_len, threads));
        let mut worker_engine_options = engine_options;
        worker_engine_options.threads = 1;
        self.worker_pool.set_helper_count(helper_count);
        let root_moves_shared: Arc<[Move]> = root_moves.to_vec().into();

        let (result_tx, result_rx) = mpsc::channel();
        let mut launched_helpers = 0usize;
        for index in 0..helper_count {
            // 8.13: stagger each helper's starting point in the root list so
            // the pool does not pile onto move 1. 9.7.5(e) tested removing this
            // (`RootRotation=false`, same binary both arms) and stopped at
            // −3.31 ± 10.62 over 1,682 games — inside the `[−5,0]` indifference
            // zone, i.e. unresolved but leaning toward rotation earning its
            // keep. Kept as the shipped behaviour; the switch was deleted
            // rather than shipped as a user-facing option.
            let offset = if threads <= root_len {
                ((index + 1) * root_len / threads).max(1) % root_len
            } else {
                (index + 1) % root_len
            };
            let job = WorkerJob {
                root: root.clone(),
                root_moves: Arc::clone(&root_moves_shared),
                limits: limits.clone(),
                engine_options: worker_engine_options.clone(),
                tt: self.tt.clone(),
                hash_mb: self.hash_mb,
                root_move_offset: offset,
                thread_id: index + 1,
                shared_state: Arc::clone(&shared_state),
                result_tx: result_tx.clone(),
            };
            if self.worker_pool.send_search(index, job) {
                launched_helpers += 1;
            }
        }
        drop(result_tx);

        self.root_move_offset = 0;
        self.thread_id = 0;
        self.shared_state = Some(Arc::clone(&shared_state));
        let root_for_ponder = root.clone();
        let mut main_poll = || match shared_state.stop_state.load(Ordering::Relaxed) {
            STOP_QUIT => SearchEvent::Quit,
            STOP_SEARCH => SearchEvent::Stop,
            _ => match poll() {
                SearchEvent::Quit => {
                    shared_state.request_quit();
                    SearchEvent::Quit
                }
                SearchEvent::Stop => {
                    shared_state.request_stop();
                    SearchEvent::Stop
                }
                SearchEvent::PonderHit => {
                    shared_state.ponderhit.store(true, Ordering::Relaxed);
                    SearchEvent::PonderHit
                }
                SearchEvent::None => SearchEvent::None,
            },
        };
        let main_result = self.search_root(root, root_moves, emit_info, &mut main_poll);
        shared_state.request_stop();

        let mut helper_results = Vec::with_capacity(launched_helpers + 1);
        helper_results.push(main_result);
        for _ in 0..launched_helpers {
            if let Ok(result) = result_rx.recv() {
                helper_results.push(result);
            }
        }
        self.root_move_offset = 0;

        #[cfg(feature = "diag")]
        if let Some(main) = helper_results.first() {
            let min_depth = helper_results
                .iter()
                .map(|result| result.depth)
                .min()
                .unwrap_or(0);
            let max_depth = helper_results
                .iter()
                .map(|result| result.depth)
                .max()
                .unwrap_or(0);
            let min_score = helper_results
                .iter()
                .map(|result| result.score)
                .min()
                .unwrap_or(0);
            let max_score = helper_results
                .iter()
                .map(|result| result.score)
                .max()
                .unwrap_or(0);
            let disagreements = helper_results
                .iter()
                .filter(|result| result.bestmove != main.bestmove)
                .count();
            crate::diag_add!(
                worker_best_disagreement,
                u64::try_from(disagreements).unwrap_or(u64::MAX)
            );
            crate::diag_add!(
                worker_depth_spread_sum,
                u64::try_from(max_depth.saturating_sub(min_depth)).unwrap_or(u64::MAX)
            );
            crate::diag_add!(
                worker_score_spread_sum,
                u64::from(max_score.saturating_sub(min_score).unsigned_abs())
            );
        }

        // 9.7.5(b): every helper has been joined above, so the counters are now
        // complete and this is the one legitimate dump point for a parallel go.
        crate::diag::dump();

        let total_nodes = helper_results.iter().map(|result| result.nodes).sum();
        let total_tb_hits = shared_state.tb_hits.load(Ordering::Relaxed);
        let quit = shared_state.stop_state.load(Ordering::Relaxed) == STOP_QUIT
            || helper_results
                .iter()
                .any(|result| result.exit == SearchExit::Quit);
        let mut best =
            select_parallel_result(&helper_results, root_moves).unwrap_or(SearchResult {
                bestmove: root_moves[0],
                pondermove: Move::NULL,
                score: -INF_SCORE,
                depth: 0,
                nodes: 0,
                tb_hits: 0,
                elapsed_ms: self.start.elapsed().as_millis(),
                exit: SearchExit::Stop,
                ponderhit: self.ponderhit,
            });
        self.nodes = total_nodes;
        self.tb_hits = total_tb_hits;
        self.quit = quit;
        self.stopped = true;
        best.nodes = total_nodes;
        best.tb_hits = total_tb_hits;
        best.elapsed_ms = self.start.elapsed().as_millis();
        if best.pondermove.is_null() {
            best.pondermove = self.ponder_from_tt(&root_for_ponder, best.bestmove);
        }
        best.ponderhit = self.ponderhit || helper_results.iter().any(|result| result.ponderhit);
        best.exit = if quit {
            SearchExit::Quit
        } else {
            SearchExit::Stop
        };
        self.shared_state = None;
        best
    }

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
        if self.params.nmp_min_non_pawn_pieces <= 1 {
            non_pawn.any()
        } else {
            infra::to_i32(non_pawn.count() as usize) >= self.params.nmp_min_non_pawn_pieces
        }
    }

    /// 4.6b: LMR reduction in 1024ths, as one formula shared by LMR itself and
    /// by the prospective depth the pruning consumers may use.
    ///
    /// Deliberately EXCLUDES two terms, both documented rather than hidden:
    ///
    /// * the per-thread jitter, because it mutates PRNG state and must be drawn
    ///   exactly once at the real reduction site — it is ±6% of one ply and is
    ///   not drawn at all at `Threads = 1`;
    /// * the singular extension, because the pruning consumers run BEFORE the
    ///   extension is known (pruning at the top of the move loop, extension
    ///   after it), so a shared pre-move depth cannot include it.
    ///
    /// Both callers pass the SAME `ReductionInputs` value, built once per move,
    /// so the prospective depth and the applied reduction cannot drift apart
    /// the way this arithmetic already did once. The `debug_assert` at the LMR
    /// site is kept as a cheap restatement of that, not as the guarantee.
    ///
    /// 4.5.1 removed this function's `#[expect(clippy::too_many_arguments)]`:
    /// the thirteen positional arguments are now one named struct, and the
    /// expectation went unfulfilled the moment they did. That is the
    /// self-cleaning property `#[expect]` is used for.
    #[inline(always)]
    fn lmr_reduction_units(&self, i: ReductionInputs) -> i32 {
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
            parent_move_count,
            parent_stat_score,
            tt_move_is_capture,
            tt_move_singular,
        } = i;
        let mut r = self.lmr_table[infra::to_usize(depth.min(63))][searched.min(63)];
        if tt_pv {
            r -= self.params.lmr_tt_pv_adj;
        } else if is_quiet {
            r += 1024;
        }
        if improving {
            r -= 1024;
        }
        if ev_is_exact {
            r += self.params.lmr_exact_bound;
        }
        // `lmr_shallow_tt` is a misnomer: it fires on TT-move PRESENCE and was
        // SPSA'd as such. See the `params.rs` note.
        if !tt_move_is_null && searched >= 4 {
            r += self.params.lmr_shallow_tt;
        }
        if cut_node {
            r += self.params.lmr_cut_node;
        }
        if !is_quiet && see < 0 {
            r += 1024;
        }
        if !tt_pv && !cut_node && quiet_hist > 4_000 {
            r -= 1024;
        }
        r -= quiet_hist * 1024 / self.params.lmr_hist_div;
        // 8.5(b): reduce less when the static eval is heavily corrected.
        r -= corr_abs * self.params.corr_lmr_scale / 128;
        // ─── 4.5.1 contract additions. All default to 0, so the accepted
        // fingerprint is preserved until they are fitted. Each is a MECHANISM
        // taken from the reference; none of its constants are, both because
        // the independence boundary forbids it and because its thresholds are
        // in its own history units, which are not Rarog's.
        //
        // (a) The TT move is a capture, so a quiet sibling is a worse bet.
        if is_quiet && tt_move_is_capture {
            r += self.params.lmr_tt_capture;
        }
        // (b) The TT move proved singular here, so this node's value rests on
        // one move and its siblings deserve a fairer look.
        if tt_move_singular {
            r -= self.params.lmr_singular_relief;
        }
        // (c) The parent was still searching late when it played into this
        // node, so this node is less likely to be a clean cut.
        if parent_move_count >= self.params.lmr_parent_movecount_min {
            r -= self.params.lmr_parent_movecount_relief;
        }
        // (d) Compare this move's history with the move that led here. The
        // absolute history already has a term above; this asks whether the
        // position is IMPROVING for the side to move, which the absolute
        // value cannot say.
        if is_quiet {
            let swing = quiet_hist - parent_stat_score;
            if swing > self.params.lmr_stat_swing_margin {
                r -= self.params.lmr_stat_swing;
            } else if swing < -self.params.lmr_stat_swing_margin {
                r += self.params.lmr_stat_swing;
            }
        }
        // 4.6.7: the root is where the answer is chosen, and it was the
        // one node type the reduction could not see.
        if is_root {
            r -= self.params.lmr_root_relief;
        }
        // 4.10 CANDIDATE: unconditional relief. Applied last, inside
        // `lmr_reduction_units`, so both call sites see it and the
        // prospective-depth estimate keeps agreeing with the real reduction —
        // the `debug_assert_eq!` between them is what enforces that.
        r -= self.params.lmr_relief;
        r
    }

    fn negamax<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        mut depth: i32,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        is_pv: bool,
        allow_null: bool,
        excluded: Move,
        cut_node: bool,
        poll: &mut P,
    ) -> i32 {
        if self.check_stop(poll) {
            return 0;
        }
        if ply >= MAX_PLY - 1 {
            return self.corrected_eval(board, ply);
        }
        self.pv_len[ply] = ply;
        self.seldepth = self.seldepth.max(ply);

        if ply > 0 && board.can_declare_draw_in_search() {
            return 0;
        }

        let in_check = board.is_in_check();
        if in_check {
            // Phase 8.2(a): the unconditional in-check extension (`depth += 1`)
            // is REMOVED. It was the first of five stacked protections around
            // checked nodes and the prime EBF suspect — every check bought a
            // full extra ply regardless of whether the check was forcing.
            // Checked nodes now search at their natural depth; a checked node
            // at depth 0 falls through to qsearch, which is safe because
            // qsearch generates the FULL legal movelist when in check (not just
            // captures) and detects mate, so evasions are never missed.
            // The `check_extensions` diag counter is intentionally left defined
            // in diag.rs and now reads 0 — an explicit confirmation the
            // extension is off. Restore this line to revert on H0.
        }

        let mate_alpha = -MATE_SCORE + infra::to_i32(ply);
        let mate_beta = MATE_SCORE - infra::to_i32(ply) - 1;
        alpha = alpha.max(mate_alpha);
        let beta = beta.min(mate_beta);
        if alpha >= beta {
            return alpha;
        }

        if depth <= 0 {
            return self.quiescence(board, alpha, beta, ply, 0, poll);
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
        let hash = board.hash;
        #[cfg(feature = "diag")]
        let diag_sample = crate::diag::sampled(hash, ply, crate::diag::SAMPLE_MAIN);
        #[cfg(feature = "diag")]
        if diag_sample {
            crate::diag_count!(sampled_main_nodes);
        }
        if let Some(score) = self.syzygy_wdl_score(board, depth, ply, excluded) {
            self.tt.store(TtStore {
                key: hash,
                depth,
                score,
                bound: Bound::Exact,
                mv: Move::NULL,
                ply,
                static_eval: VALUE_NONE,
                is_pv,
                kind: OutcomeKind::Tablebase,
            });
            return score;
        }
        let tt_entry = self.tt.probe(hash);
        // 9.7.5(b): main thread only. If helper work is reaching the thread
        // that owns the answer, this hit rate must RISE with thread count; a
        // flat rate means the helpers are filling a table nobody reads.
        if self.thread_id == 0 {
            crate::diag_count!(main_tt_probes);
            if tt_entry.is_some() {
                crate::diag_count!(main_tt_hits);
            }
        }
        // 4.2: one decode of the probe for the whole node. Mate distance and
        // rule-50 are resolved exactly once here — the pre-4.2 code decoded the
        // same entry twice, at `tt_score` and again inside the cutoff block.
        let ev = NodeEvidence::from_probe(tt_entry, ply, board.halfmove_clock);
        let tt_pv = ev.pv_line(is_pv);
        // 4.2b: captured at node entry, against the window this node was ASKED
        // to resolve. `alpha` is raised by the move loop below, so reading it
        // later would ask a different question.
        #[cfg(feature = "diag")]
        let diag_contradicts = ev.contradicts_window(alpha, beta);
        #[cfg(feature = "diag")]
        if diag_sample {
            if ev.hit {
                crate::diag_count!(tt_sample_hit);
                crate::diag_count!(shadow_4_2_evidence);
                if diag_contradicts {
                    crate::diag_count!(contradict_hits);
                }
                if !is_pv && excluded.is_null() && ev.depth >= depth {
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
                    if is_pv {
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
        if !is_pv
            && excluded.is_null()
            && let Some(score) = ev.cutoff_score(depth, alpha, beta)
        {
            // 8.4(a): the TT move just produced a beta cutoff without a
            // search - today it gets zero feedback. Reward it (quiet
            // moves only, main/low-ply/pawn histories) at a tunable
            // fraction of the cutoff bonus. Seed 0 = skip entirely.
            //
            // 4.2 note: still unconditional on provenance, so a depth-0 stand
            // pat can train quiet history through this path. 4.5 owns the
            // attribution guard; changing it here would be an ungated edit.
            if matches!(ev.bound, Some(Bound::Lower))
                && score >= beta
                && self.params.tt_cutoff_bonus_pct != 0
                && let Some(mv) = ev.mv.and_then(|m| board.legal_move(m))
                && !mv.is_capture()
                && !mv.is_promo()
            {
                let bonus = self.history_bonus(depth) * self.params.tt_cutoff_bonus_pct / 100;
                self.update_quiet_history(
                    board.side_to_move(),
                    mv,
                    board.moving_piece(mv),
                    board.pawn_key(),
                    ply,
                    bonus,
                );
            }
            return score;
        }
        let mut tt_move = ev
            .mv
            .and_then(|mv| board.legal_move(mv))
            .unwrap_or(Move::NULL);
        if ply == 0 && !self.root_moves.is_empty() && !self.root_moves.contains(&tt_move) {
            tt_move = Move::NULL;
        }

        #[cfg(feature = "diag")]
        let mut diag_iir_applied = false;
        // 4.2b: a contradicting entry that is deep enough to SUPPRESS IIR — the
        // search trusts it to order this node even though it resolved a
        // different window. A depth penalty would let IIR fire here instead.
        #[cfg(feature = "diag")]
        if diag_sample
            && diag_contradicts
            && excluded.is_null()
            && depth >= 4
            && !tt_move.is_null()
            && !(!is_pv && ev.too_shallow_to_order(depth))
        {
            crate::diag_count!(contradict_iir_suppressed);
        }
        // IIR: reduce depth when we lack a good TT entry to guide move ordering
        if !self.ablated(4)
            && excluded.is_null()
            && depth >= 4
            && (tt_move.is_null() || (!is_pv && ev.too_shallow_to_order(depth)))
        {
            #[cfg(feature = "diag")]
            if diag_sample {
                diag_iir_applied = true;
                crate::diag_count!(iir_applied);
                crate::diag_count!(shadow_4_4_selectivity);
                if is_pv {
                    crate::diag_count!(iir_pv);
                }
                if tt_move.is_null() {
                    crate::diag_count!(iir_no_tt_move);
                } else {
                    crate::diag_count!(iir_shallow_tt);
                }
            }
            depth -= 1;
        }

        // 4.2: the pre-4.2 form spelled out three branches whose two `else`
        // arms were identical, because a probe MISS and a hit carrying no
        // stored eval both fall back to a fresh raw eval. `NodeEvidence::MISS`
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
        self.stack[ply].static_eval = static_eval;
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
        // 4.5c: it is also applied to a number the correction may no longer be
        // part of. `eval_for_pruning` below can be REPLACED wholesale by a TT
        // bound (28.5% of sampled hits refine it, RAR-S30), and when that
        // happens the corrected eval is discarded — yet these margins are still
        // widened by the discarded correction's magnitude. That mismatch is what
        // `CorrSkipWhenTtRefined` measures and can switch off.
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
        let improving = !in_check
            && ply >= 2
            && self.stack[ply - 2].static_eval != VALUE_NONE
            && static_eval > self.stack[ply - 2].static_eval;
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
        // 4.5c: when a TT bound replaced the corrected eval, the correction is
        // no longer present in the number the margins test, so charging an
        // uncertainty penalty for it is charging for an adjustment that is not
        // there. At the seeded 0 this is exactly the prior behaviour.
        let corr_abs =
            if self.params.corr_skip_when_tt_refined != 0 && eval_for_pruning != static_eval {
                0
            } else {
                corr_abs
            };
        #[cfg(feature = "diag")]
        if corr_abs != 0 && eval_for_pruning != static_eval {
            crate::diag_count!(corr_applied_to_replaced_eval);
        }
        #[cfg(feature = "diag")]
        if diag_sample
            && eval_for_pruning != VALUE_NONE
            && static_eval != VALUE_NONE
            && eval_for_pruning != static_eval
        {
            crate::diag_count!(tt_eval_refined);
            let delta = u64::from(eval_for_pruning.saturating_sub(static_eval).unsigned_abs());
            crate::diag_add!(tt_eval_delta_sum, delta);
            // 4.2b: an entry that told this node nothing still moved the eval
            // its forward pruning runs on. Slack is measured against the knob
            // that actually gates the refinement.
            if diag_contradicts {
                crate::diag::record_contradiction_refine(ev.depth, delta);
            }
        }
        // 8.3 diagnostic: a non-PV, non-check node where the *stored* PV bit
        // (tt_pv true while is_pv false) is what keeps the whole forward-pruning
        // block below from running.
        if tt_pv && !is_pv && !in_check && excluded.is_null() {
            crate::diag_count!(tt_pv_veto);
            // 4.4a sizing: of the nodes this shared veto blocks, how many would
            // each mechanism actually reach if its own switch handed them back?
            // Depth preconditions only — the margin tests need the eval, which
            // is what the veto is denying them.
            #[cfg(feature = "diag")]
            {
                if depth <= 8 {
                    crate::diag_count!(tt_pv_veto_rfp_eligible);
                }
                if depth <= 3 {
                    crate::diag_count!(tt_pv_veto_razor_eligible);
                }
                if allow_null && depth >= 3 && board.has_non_pawn_material(board.side_to_move()) {
                    crate::diag_count!(tt_pv_veto_nmp_eligible);
                }
                if depth >= 4 {
                    crate::diag_count!(tt_pv_veto_probcut_eligible);
                }
            }
        }
        // 4.4a: the shared `!tt_pv` veto becomes four per-mechanism predicates.
        // At the seeded zeros `tt_pv_allows_any` is false, so this outer test is
        // exactly the old `!tt_pv && ...` — including the fast path, so a
        // `tt_pv` node still skips the margin arithmetic entirely.
        let rfp_tt_pv_ok = !tt_pv || self.params.rfp_allow_tt_pv != 0;
        let razor_tt_pv_ok = !tt_pv || self.params.razor_allow_tt_pv != 0;
        let nmp_tt_pv_ok = !tt_pv || self.params.nmp_allow_tt_pv != 0;
        let probcut_tt_pv_ok = !tt_pv || self.params.probcut_allow_tt_pv != 0;
        let tt_pv_allows_any = rfp_tt_pv_ok || razor_tt_pv_ok || nmp_tt_pv_ok || probcut_tt_pv_ok;
        if tt_pv_allows_any && !in_check && excluded.is_null() {
            let futility_margin = (self.params.futility_base
                + self.params.futility_not_improving * not_improving_i)
                * depth
                + corr_abs * self.params.corr_rfp_scale / 128; // 8.5(b)
            // 4.3 shadow, part 1. Evaluate all three forward-pruning predicates
            // twice — once on the refined eval the search will actually use, once
            // on the unrefined static eval — and count the disagreements. Placed
            // here, before RFP can return, so every consumer is covered by one
            // block and the sample set is identical for all three. Diagnostic
            // only: nothing below reads these, and `eval_for_pruning` is
            // untouched.
            #[cfg(feature = "diag")]
            if diag_sample && eval_for_pruning != static_eval {
                crate::diag_count!(refine_flip_nodes);
                let nmp_bar = beta
                    - self.params.nm_depth_coeff * depth
                    - self.params.nm_improving_bonus * improving_i;
                let nmp_gated =
                    allow_null && depth >= 3 && board.has_non_pawn_material(board.side_to_move());
                // Written out per consumer rather than as an array keyed by an
                // index: a `(refined, plain, which)` tuple plus a `_` arm is the
                // positional-sentinel shape the clean-code policy rules out, and
                // it would silently mislabel a fourth consumer as NMP.
                if depth <= 8 {
                    match (
                        eval_for_pruning - futility_margin >= beta,
                        static_eval - futility_margin >= beta,
                    ) {
                        (true, false) => crate::diag_count!(refine_flip_rfp_on),
                        (false, true) => crate::diag_count!(refine_flip_rfp_off),
                        _ => {}
                    }
                }
                if depth <= 3 {
                    let bar = self.params.razoring_coeff * depth;
                    match (eval_for_pruning + bar < alpha, static_eval + bar < alpha) {
                        (true, false) => crate::diag_count!(refine_flip_razor_on),
                        (false, true) => crate::diag_count!(refine_flip_razor_off),
                        _ => {}
                    }
                }
                if nmp_gated {
                    match (eval_for_pruning >= nmp_bar, static_eval >= nmp_bar) {
                        (true, false) => crate::diag_count!(refine_flip_nmp_on),
                        (false, true) => crate::diag_count!(refine_flip_nmp_off),
                        _ => {}
                    }
                }
            }
            if !self.ablated(1)
                && rfp_tt_pv_ok
                && depth <= 8
                && eval_for_pruning - futility_margin >= beta
            {
                crate::diag_count!(rfp_cut);
                return eval_for_pruning;
            }
            if !self.ablated(0)
                && razor_tt_pv_ok
                && depth <= 3
                && eval_for_pruning + self.params.razoring_coeff * depth < alpha
            {
                crate::diag_count!(razor_drop);
                return self.quiescence(board, alpha, beta, ply, 0, poll);
            }
            // 4.4b: which eval the null threshold may read. At the seeded 0
            // this is `eval_for_pruning`, exactly as before.
            let nmp_eval = if self.params.nmp_use_static_eval != 0 {
                static_eval
            } else {
                eval_for_pruning
            };
            if !self.ablated(2)
                && allow_null
                && nmp_tt_pv_ok
                // 4.4a: with the switch on, a null-verification subtree may not
                // null-prune anywhere inside itself, not merely at its root.
                && (self.params.nmp_suppress_null_in_verification == 0
                    || self.nmp_verify_nesting == 0)
                // 4.4b: restrict to nodes the caller expects to fail high.
                && (self.params.nmp_require_cut_node == 0 || cut_node)
                // 4.4c: a node that hinges on one move is the worst place to
                // trust a null refutation. Evidence-only, so it slightly
                // over-approximates - the conservative direction.
                && (self.params.nmp_singular_guard == 0
                    || !(depth >= 4
                        && ev.mv.is_some()
                        && ev.allows_singular(
                            depth,
                            self.params.singular_tt_depth_margin,
                            self.params.singular_reject_speculative != 0,
                        )))
                && depth >= 3
                && nmp_eval
                    >= beta
                        - self.params.nm_depth_coeff * depth
                        - self.params.nm_improving_bonus * improving_i
                && self.nmp_material_ok(board)
            {
                #[cfg(feature = "diag")]
                if self.nmp_verify_nesting > 0 {
                    // Exact, not sampled, and this IS the population
                    // `NmpSuppressNullInVerification` refuses: same predicate,
                    // so the counter and the switch cannot drift apart.
                    crate::diag_count!(nmp_nested_attempt);
                }
                // 4.10a: NMP running at a mate-range WINDOW. This was the
                // `NmpDecisiveGuard` population; the switch is gone (4.10a
                // removed it as an efficiency guard worth 0.004% of nodes) but
                // the count is kept, because it is the context for the unproven
                // -mate question below, which is a different condition.
                #[cfg(feature = "diag")]
                if beta.abs() >= MATE_SCORE - infra::to_i32(MAX_PLY) {
                    crate::diag_count!(nmp_decisive_population);
                }
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(nmp_attempt);
                    crate::diag_count!(shadow_4_4_selectivity);
                    if eval_for_pruning != static_eval {
                        crate::diag_count!(nmp_eval_tt);
                    } else if static_eval != raw_static_eval {
                        crate::diag_count!(nmp_eval_corrected);
                    } else {
                        crate::diag_count!(nmp_eval_raw);
                    }
                }
                let reduction = 4 + depth / 4 + ((nmp_eval - beta) / 200).clamp(0, 3);
                board.make_null_move();
                self.tt.prefetch(board.hash);
                let score = -self.negamax(
                    board,
                    depth - reduction,
                    -beta,
                    -beta + 1,
                    ply + 1,
                    false,
                    false,
                    Move::NULL,
                    true,
                    poll,
                );
                board.unmake_null_move();
                if self.stopped || self.quit {
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
                    #[cfg(feature = "diag")]
                    if diag_sample {
                        crate::diag_count!(nmp_sample_cut);
                    }
                    if depth >= 10 {
                        crate::diag_count!(nmp_verify_attempt);
                        let verify_depth = (depth - reduction).max(1);
                        self.nmp_verify_nesting += 1;
                        let verified = self.negamax(
                            board,
                            verify_depth,
                            beta - 1,
                            beta,
                            ply,
                            false,
                            false,
                            Move::NULL,
                            false,
                            poll,
                        );
                        self.nmp_verify_nesting -= 1;
                        if self.stopped || self.quit {
                            return 0;
                        }
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

            if !self.ablated(3) && probcut_tt_pv_ok && depth >= 4 {
                // Per NODE entering the block, before capture generation, so
                // nodes with no eligible capture are counted here too. This
                // carried the `probcut_attempt` name until 4.7c prep, and was
                // differenced against the oracle's per-MOVE counter of the same
                // name -- the RAR-S25 denominator shape. See the RAR-S55
                // correction.
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(probcut_nodes);
                    crate::diag_count!(shadow_4_4_selectivity);
                }
                let probcut_beta = beta + self.params.probcut_margin;
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
                    ((probcut_beta - static_eval) * self.params.probcut_see_gap_scale / 100).max(0);
                // The flat cap of 8 had no stated derivation. Scale it by the
                // node's own prediction instead: a cut node is where a fail-high
                // is expected and the speculative search is likeliest to pay.
                let move_cap = self.params.probcut_move_cap_base
                    + if cut_node {
                        self.params.probcut_move_cap_cut_bonus
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
                    self.push_move(ply, mv, probcut_piece, 0, 0);
                    board.make_move_unchecked(mv);
                    self.tt.prefetch(board.hash);
                    let score =
                        -self.quiescence(board, -probcut_beta, -probcut_beta + 1, ply + 1, 0, poll);
                    let score = if score >= probcut_beta {
                        #[cfg(feature = "diag")]
                        if diag_sample {
                            crate::diag_count!(probcut_qpass);
                        }
                        -self.negamax(
                            board,
                            depth - 4,
                            -probcut_beta,
                            -probcut_beta + 1,
                            ply + 1,
                            false,
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
                    if self.stopped || self.quit {
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
                        self.tt.store(TtStore {
                            key: hash,
                            depth: depth - 3,
                            // Which value to persist is an ablation, NOT part of
                            // the speculative contract: the producer bit keeps
                            // this result out of singular seeding either way.
                            // Storing the actual fail-high costs +5.55%
                            // time-to-depth on its own (RAR-S34), so the
                            // conservative margin-shifted value is the default.
                            score: if self.params.probcut_store_actual_score != 0 {
                                score
                            } else {
                                cutoff_score
                            },
                            bound: Bound::Lower,
                            mv,
                            ply,
                            static_eval: raw_static_eval,
                            is_pv: false,
                            kind: OutcomeKind::ProbCut,
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

        let mut move_picker = if in_check || ply == 0 || !excluded.is_null() {
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
            let legal_moves = if ply == 0 && !self.root_moves.is_empty() {
                root_moves = legal_moves
                    .iter()
                    .copied()
                    .filter(|mv| self.root_moves.contains(mv))
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
            if ply == 0 && scored.len() > 1 {
                // No-op serially: with no shared state there are no pool
                // scores to fold in.
                self.apply_shared_root_scores(legal_moves, &mut scored);
            }
            // Helpers rotate their root list on top of the pool ordering, so
            // the pool's shared view refines the ordering without collapsing
            // every thread onto the same tree.
            let rotate = self.root_move_offset > 0;
            if ply == 0 && rotate && scored.len() > 1 {
                let offset = self.root_move_offset % scored.len();
                diversify_root_scores(scored.as_mut_slice(), offset);
            }
            MovePicker::full(scored, tt_move)
        } else {
            MovePicker::staged(self, board, tt_move, ply)
        };
        let mut best_move = Move::NULL;
        let mut best_score = -INF_SCORE;
        let mut searched = 0usize;
        // 4.5.5: every move the loop LOOKED at, pruned or not. `searched`
        // counts only those actually searched and keeps that meaning, because
        // the PVS first-move logic and the mate/stalemate test depend on it.
        let mut considered = 0usize;
        #[cfg(feature = "diag")]
        let diag_order_sample = diag_sample && excluded.is_null();
        #[cfg(feature = "diag")]
        let mut diag_best_rank = 0usize;
        #[cfg(feature = "diag")]
        let mut diag_best_stage = MoveClass::BadCapture;
        #[cfg(feature = "diag")]
        let mut diag_best_reduced = false;
        let mut legal_move_seen = false;
        // 10.3: per-node check masks, built at most once and reused by every
        // move at this node — for the pruning-side `move_gives_check` calls
        // and for the `make_move` check hint below. `board` is restored by
        // `unmake_move` each iteration, so these stay valid for the whole loop.
        let mut node_ci: Option<CheckInfo> = None;
        // 4.5.1: set when the TT move is proved singular at THIS node. Read by
        // the node's later moves, which is safe because the TT move is searched
        // first, so the flag is settled before any move LMR can apply to.
        let mut tt_move_singular = false;
        // 4.6.5: latch so `skip_quiets_nodes` counts NODES, not calls.
        let mut skip_quiets_latched = false;
        // 4.7b: latch so `lmp_nodes` counts NODES, not moves -- the oracle can
        // only observe the per-node event, so that is the comparable unit.
        #[cfg(feature = "diag")]
        let mut diag_node_lmp_seen = false;
        let mut quiets = MoveList::new();
        let mut good_caps = BadCaptureList::new();
        let mut bad_caps = BadCaptureList::new();
        let previous_move = if ply > 0 {
            self.stack[ply - 1].mv
        } else {
            Move::NULL
        };
        while let Some(picked) = move_picker.next(self, board) {
            let mv = picked.mv;
            if mv == excluded {
                continue;
            }
            legal_move_seen = true;
            // 4.5.5: incremented HERE, where the reference increments, so a
            // move pruned below still advances the index.
            considered += 1;
            // The index the selectivity mechanisms use. Behind a switch, so
            // the accepted fingerprint holds while it is 0.
            let move_index = if self.params.selectivity_count_considered != 0 {
                considered - 1
            } else {
                searched
            };
            let is_capture = mv.is_capture();
            let is_quiet = board.is_quiet_move(mv);
            let mut see = if is_capture { picked.see as i32 } else { 0 };
            let moving_piece = board.moving_piece(mv);
            let captured_piece = board.captured_piece(mv);
            // 4.2: the pre-move evidence snapshot, taken at pick time. It
            // replaces a bare `0..3` stage integer, and 4.6 extends it with the
            // check/evasion taxonomy and the shared prospective depth. `see` is
            // captured here deliberately: the local below is refined for some
            // moves, and classification must not depend on where it is read.
            let move_ev = MoveEvidence::new(
                mv == tt_move,
                is_capture,
                is_quiet,
                see,
                if is_quiet { picked.quiet_history } else { 0 },
            );
            let quiet_hist = move_ev.quiet_history;
            let mut gives_check = None;
            #[cfg(feature = "diag")]
            if diag_order_sample {
                match move_ev.class {
                    MoveClass::TtMove => {
                        crate::diag_count!(move_seen_tt);
                    }
                    MoveClass::GoodCapture => {
                        crate::diag_count!(move_seen_good_capture);
                    }
                    MoveClass::Quiet => {
                        crate::diag_count!(move_seen_quiet);
                    }
                    MoveClass::BadCapture => {
                        crate::diag_count!(move_seen_bad_capture);
                    }
                }
            }
            #[cfg(feature = "diag")]
            let mut diag_move_reduced = false;

            // 4.6b: ONE prospective depth for LMP, futility, SEE pruning and
            // LMR. The audit's finding was that later pruning did not use the
            // depth the move would actually be searched at — LMR reduced it,
            // while the pruning tests all read raw `depth`, so a move about to
            // be searched 3 plies shallower was still judged as if it were not.
            //
            // `r_units_estimate` is the shared reduction; `prospective_depth` is
            // what remains after it. Both are computed pre-move so the pruning
            // consumers can see them, and the LMR site debug-asserts it derives
            // the same units.
            // 4.5.1: built ONCE and used at both sites, so the prospective
            // depth and the applied reduction cannot drift apart by
            // construction rather than by assertion. At the root there is no
            // parent, and 0 is the inert value for both parent inputs.
            let reduction_inputs = ReductionInputs {
                depth,
                searched: move_index,
                is_quiet,
                see,
                tt_pv,
                cut_node,
                quiet_hist,
                corr_abs,
                ev_is_exact: ev.is_exact(),
                tt_move_is_null: tt_move.is_null(),
                improving,
                is_root: ply == 0,
                parent_move_count: if ply > 0 {
                    self.stack[ply - 1].move_count
                } else {
                    0
                },
                parent_stat_score: if ply > 0 {
                    self.stack[ply - 1].stat_score
                } else {
                    0
                },
                tt_move_is_capture: !tt_move.is_null() && tt_move.is_capture(),
                tt_move_singular,
            };
            let r_units_estimate = self.lmr_reduction_units(reduction_inputs);
            // `depth - 1` is the child's nominal depth; subtract the estimated
            // reduction and floor at 1 so a consumer never reads a depth that
            // would make its own `depth <= N` guards nonsensical.
            let prospective_depth = if depth >= 3 && move_index >= 2 {
                (depth
                    - 1
                    - lmr_reduction(
                        r_units_estimate,
                        depth - 1,
                        self.params.lmr_min_reduced_depth,
                    ))
                .max(1)
            } else {
                depth
            };
            // At the seeded 0 every consumer keeps reading raw `depth`, so this
            // lands inert; 1 switches all four onto the shared depth together,
            // because switching them one at a time would recreate exactly the
            // incoherence 4.6 exists to remove.
            let sel_depth = if self.params.selectivity_prospective_depth != 0 {
                prospective_depth
            } else {
                depth
            };
            if !tt_pv && !in_check && searched > 0 {
                #[cfg(feature = "diag")]
                if diag_order_sample {
                    crate::diag_count!(prune_shadow_moves);
                    crate::diag_count!(shadow_4_6_prospective_depth);
                    if is_quiet {
                        let lmp_margin = (self.params.lmp_base
                            + self.params.lmp_not_improving * not_improving_i)
                            * depth;
                        let lmp = (depth <= 3 && eval_for_pruning + lmp_margin <= alpha)
                            || (depth <= 8
                                && move_index
                                    > late_move_prune_count(
                                        depth,
                                        improving,
                                        self.params.lmp_count_base,
                                    ))
                            || (depth <= 4 && quiet_hist < -10_000)
                            || (depth <= 7
                                && quiet_hist < -(self.params.quiet_hist_prune_coeff * depth));
                        let futility = depth <= 8
                            && eval_for_pruning
                                + self.params.fp_base
                                + self.params.fp_coeff * depth
                                + corr_abs * self.params.corr_fut_scale / 128
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
                            self.cap_history[moving_piece as usize][mv.to_sq().index()]
                                [cap as usize] as i32
                        });
                        let threshold = (-self.params.see_pruning_coeff * depth - cap_hist / 8)
                            .max(-self.params.see_pruning_max);
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
                    let prune_margin = (self.params.lmp_base
                        + self.params.lmp_not_improving * not_improving_i)
                        * sel_depth;
                    // 4.6.5: the move-count component ALONE, so it can be fed
                    // back to the picker the way the reference feeds its
                    // `moveCountPruning` flag into `next_move`.
                    let move_count_pruning = sel_depth <= 8
                        && move_index
                            > late_move_prune_count(
                                sel_depth,
                                improving,
                                self.params.lmp_count_base,
                            );
                    if move_count_pruning
                        && self.params.skip_quiets_on_move_count != 0
                        && !self.ablated(5)
                    {
                        if !skip_quiets_latched {
                            skip_quiets_latched = true;
                            crate::diag_count!(skip_quiets_nodes);
                        }
                        move_picker.skip_quiets();
                    }
                    let prune_candidate = !self.ablated(5)
                        && ((sel_depth <= 3 && eval_for_pruning + prune_margin <= alpha)
                            || move_count_pruning
                            || (sel_depth <= 4 && quiet_hist < -10_000)
                            || (sel_depth <= 7
                                && quiet_hist < -(self.params.quiet_hist_prune_coeff * sel_depth)));
                    if prune_candidate
                        && !move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                    {
                        crate::diag_count!(lmp_prune);
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
                    if sel_depth <= 8
                        && eval_for_pruning
                            + self.params.fp_base
                            + self.params.fp_coeff * sel_depth
                            + corr_abs * self.params.corr_fut_scale / 128 // 8.5(b)
                            <= alpha
                        && !move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                    {
                        crate::diag_count!(quiet_futility_prune);
                        continue;
                    }
                    // 4.6.4: a quiet move that hangs material. Rarog's only
                    // SEE prune is in the capture branch, so this population
                    // was never pruned for losing material. Gated OFF by a
                    // zero depth limit, which is why the accepted fingerprint
                    // survives.
                    if self.params.quiet_see_prune_depth != 0
                        && sel_depth <= self.params.quiet_see_prune_depth
                        && !board.see_ge_quiet_aware(
                            mv,
                            (-self.params.quiet_see_prune_coeff * sel_depth * sel_depth)
                                .max(-self.params.see_pruning_max),
                        )
                        && !move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                    {
                        crate::diag_count!(quiet_see_prune);
                        continue;
                    }
                } else if is_capture && see < 0 {
                    let cap_hist = captured_piece.map_or(0, |cap| {
                        self.cap_history[moving_piece as usize][mv.to_sq().index()][cap as usize]
                            as i32
                    });
                    let see_threshold = (-self.params.see_pruning_coeff * sel_depth - cap_hist / 8)
                        .max(-self.params.see_pruning_max);
                    if sel_depth <= 8
                        && !board.see_ge(mv, see_threshold)
                        && !move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                    {
                        crate::diag_count!(see_prune);
                        continue;
                    }
                }
            }

            let child_is_pv = is_pv && searched == 0;
            let mut extension = 0;
            let singular_move_candidate =
                !self.ablated(6) && ply > 0 && mv == tt_move && excluded.is_null() && depth >= 4;
            #[cfg(feature = "diag")]
            if singular_move_candidate
                && ev.speculative_singular_seed_blocked(depth, self.params.singular_tt_depth_margin)
            {
                crate::diag_count!(singular_speculative_seed_blocked);
            }
            if singular_move_candidate
                && ev.allows_singular(
                    depth,
                    self.params.singular_tt_depth_margin,
                    self.params.singular_reject_speculative != 0,
                )
            {
                #[cfg(feature = "diag")]
                if diag_sample {
                    crate::diag_count!(singular_attempt);
                    crate::diag_count!(shadow_4_4_selectivity);
                    if ev.depth == depth - 3 && matches!(ev.bound, Some(Bound::Lower)) {
                        // Since 4.3c this is explicitly only the historical
                        // ProbCut-shaped signature; tagged ProbCut producers
                        // have already been rejected above.
                        crate::diag_count!(singular_probcut_depth_match);
                    }
                    // 4.2b: the verification window is seeded from a score that
                    // resolved a different window.
                    if diag_contradicts {
                        crate::diag_count!(contradict_singular_attempt);
                    }
                }
                let singular_beta = ev.score - self.params.singular_beta_mult * depth;
                let singular_depth = (depth - 1) / 2;
                let singular_score = self.negamax(
                    board,
                    singular_depth,
                    singular_beta - 1,
                    singular_beta,
                    ply,
                    false,
                    false,
                    mv,
                    false,
                    poll,
                );
                if self.stopped || self.quit {
                    return 0;
                }
                if singular_score < singular_beta {
                    tt_move_singular = true;
                    extension = if !is_pv
                        && singular_score < singular_beta - self.params.singular_double_margin
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
                        // 4.2b: counted HERE, not below. This arm returns, so
                        // the post-block counter never sees a multi-cut — the
                        // single largest tree effect a contradicting seed can
                        // have was silently missing from the shadow until now.
                        if diag_contradicts {
                            crate::diag_count!(contradict_singular_multicut);
                        }
                    }
                    return singular_beta;
                } else if ev.score >= beta {
                    #[cfg(feature = "diag")]
                    if diag_sample {
                        crate::diag_count!(singular_negative_extension);
                    }
                    extension = -1;
                }
                #[cfg(feature = "diag")]
                if diag_sample && diag_iir_applied && extension != 0 {
                    crate::diag_count!(iir_extension_debt);
                }
                // 4.2b: did that seed change the DEPTH? Extensions and negative
                // extensions only — the multi-cut path returns above and is
                // counted there. Sum the two for total tree effect.
                #[cfg(feature = "diag")]
                if diag_sample && diag_contradicts && extension != 0 {
                    crate::diag_count!(contradict_singular_changed_depth);
                }
            }

            let checking_move =
                if depth >= 3 && move_index >= 2 && (is_quiet || see < 0) && !mv.is_promo() {
                    move_gives_check(board, &mut node_ci, mv, &mut gives_check)
                } else {
                    gives_check.unwrap_or(false)
                };

            // 4.5.1: the child reduces using its parent's move count and the
            // history of the move that led there. `searched` is the count
            // BEFORE this move, so +1 makes it this move's index.
            let searched_i32 = i32::try_from(searched).unwrap_or(i32::MAX);
            self.push_move(
                ply,
                mv,
                moving_piece,
                searched_i32.saturating_add(1),
                if is_quiet { quiet_hist } else { 0 },
            );
            let nodes_before_move = if ply == 0 { self.nodes } else { 0 };
            // 10.3: the check predicate is cheap here (node masks + two
            // bitboard tests) and lets `make_move` skip `calculate_checkers`
            // for the overwhelmingly common non-checking move.
            let mv_gives_check = move_gives_check(board, &mut node_ci, mv, &mut gives_check);
            board.make_move_with_check(mv, mv_gives_check);
            self.tt.prefetch(board.hash);
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
                score = -self.negamax(
                    board,
                    new_depth,
                    -beta,
                    -alpha,
                    ply + 1,
                    child_is_pv,
                    true,
                    Move::NULL,
                    !child_is_pv && !cut_node,
                    poll,
                );
            } else {
                // Late evasions are intentionally not reduced. The alternative
                // increased the deterministic tree by 14.83% and had no owner.
                let reducible = !self.ablated(7)
                    && depth >= 3
                    && move_index >= 2
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
                    // 4.6b: ONE formula, shared with the prospective depth the
                    // pruning consumers can use. Previously this arithmetic lived
                    // only here, so LMP/futility/SEE had no way to know how deep
                    // the move would actually be searched.
                    let mut r = self.lmr_reduction_units(reduction_inputs);
                    debug_assert_eq!(
                        r, r_units_estimate,
                        "4.6b: LMR and the prospective depth must share one formula"
                    );
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
                    // search — `shared_state` is None at Threads=1, which is
                    // what keeps bench identical.
                    // SMP diversification, unchanged: magnitude 64 reproduces
                    // the original expression exactly.
                    //
                    // 4.10 CANDIDATE — 1T selectivity jitter. Three independent
                    // results say perturbing this surface beats leaving it
                    // alone: RAR-S54 (+4.06 ± 3.71 over 14,196 games for a
                    // blind uniform de-selectivity shift), RAR-S62 (a ProbCut
                    // desync reading an arbitrary continuation row beat correct
                    // indexing by ~5 Elo) and RAR-S64 (a stale prior-reduction
                    // firing on a quasi-random subset beat the correct one by
                    // ~4.5). Twice a BUG that scattered noise into selectivity
                    // beat its own correction. The machinery already existed
                    // and was disabled at 1T only to keep bench deterministic
                    // for SMP work — the PRNG is re-seeded per search from a
                    // fixed seed, so 1T stays reproducible.
                    if self.shared_state.is_some() {
                        r += self.next_jitter(64);
                    } else if self.params.lmr_jitter_1t != 0 {
                        r += self.next_jitter(self.params.lmr_jitter_1t);
                    }
                    // 10.2.5 candidate: strong late moves may escape the old
                    // mandatory one-ply reduction. A zero reduction is a normal
                    // full-depth PVS search and must not trigger a redundant
                    // verification search at the same depth.
                    let reduction = lmr_reduction(r, new_depth, self.params.lmr_min_reduced_depth);
                    #[cfg(feature = "diag")]
                    {
                        if r < 0 {
                            crate::diag_count!(lmr_floor_clamped);
                        }
                        if new_depth > 0 && reduction == new_depth {
                            crate::diag_count!(lmr_qs_clamped);
                        }
                        if ply == 0 {
                            crate::diag_count!(lmr_root_applied);
                            crate::diag_add!(
                                lmr_root_reduction_sum,
                                u64::try_from(reduction).unwrap_or(0)
                            );
                        }
                    }
                    #[cfg(feature = "diag")]
                    {
                        diag_move_reduced = reduction > 0;
                        // 4.2: EXACT, because its denominator `lmr_applied` is
                        // exact. Sampling only the numerator made the mean
                        // reduction read 1024x low at the default stride.
                        crate::diag_add!(
                            reduction_depth_sum,
                            u64::try_from(reduction).unwrap_or(0)
                        );
                    }
                    if reduction == 0 {
                        crate::diag_count!(lmr_zero_reduction);
                    } else {
                        crate::diag_count!(lmr_applied);
                    }
                    score = -self.negamax(
                        board,
                        new_depth - reduction,
                        -alpha - 1,
                        -alpha,
                        ply + 1,
                        false,
                        true,
                        Move::NULL,
                        true,
                        poll,
                    );
                    if reduction > 0 && score > alpha {
                        crate::diag_count!(lmr_research);
                        // Full-depth verification re-search. (A do-deeper / do-shallower
                        // LMR re-search adjustment was tried as Phase 2.8 and dropped:
                        // do_shallower was proven dead, and SPSA-tuned do_deeper failed
                        // its SPRT gate at st=0.1 — -1.38 Elo for ~4% more nodes
                        // (bench 5,612,008 vs 5,401,662), a TC-transfer failure like the
                        // 2.4 LMR tune. See PLAN §5 2.8.)
                        score = -self.negamax(
                            board,
                            new_depth,
                            -alpha - 1,
                            -alpha,
                            ply + 1,
                            false,
                            true,
                            Move::NULL,
                            !cut_node,
                            poll,
                        );
                    }
                } else {
                    score = -self.negamax(
                        board,
                        new_depth,
                        -alpha - 1,
                        -alpha,
                        ply + 1,
                        false,
                        true,
                        Move::NULL,
                        true,
                        poll,
                    );
                }
                if score > alpha && score < beta {
                    score = -self.negamax(
                        board,
                        new_depth,
                        -beta,
                        -alpha,
                        ply + 1,
                        true,
                        true,
                        Move::NULL,
                        false,
                        poll,
                    );
                }
            }
            board.unmake_move(mv);
            self.clear_move(ply);

            if self.stopped || self.quit {
                return 0;
            }

            let move_nodes = if ply == 0 {
                self.nodes.saturating_sub(nodes_before_move)
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
            if ply == 0 {
                self.record_root_move_search(mv, depth, score, alpha, beta, move_nodes);
            }
            if score > best_score {
                best_score = score;
                best_move = mv;
                #[cfg(feature = "diag")]
                if diag_sample {
                    diag_best_rank = searched;
                    diag_best_stage = move_ev.class;
                    diag_best_reduced = diag_move_reduced;
                }
                if ply == 0 {
                    self.root_best_nodes = move_nodes;
                }
            }
            if score > alpha {
                alpha = score;
                self.pv_table[ply][ply] = mv;
                let child_len = self.pv_len[ply + 1].max(ply + 1);
                for next_ply in ply + 1..child_len {
                    self.pv_table[ply][next_ply] = self.pv_table[ply + 1][next_ply];
                }
                self.pv_len[ply] = child_len;

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
                            self.params.surprise_bonus_pct
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
                            if self.params.capture_malus_pct != 0 {
                                let xmalus = malus * self.params.capture_malus_pct / 100;
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
                        self.tt.store(TtStore {
                            key: hash,
                            depth,
                            score,
                            bound: Bound::Lower,
                            mv,
                            ply,
                            static_eval: raw_static_eval,
                            is_pv: tt_pv,
                            kind: OutcomeKind::Full,
                        });
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
                                crate::diag_count!(correction_on_capture);
                            }
                            let residual = self.attributed_residual(
                                score - static_eval,
                                is_capture,
                                board.halfmove_clock,
                            );
                            self.update_correction(board, residual, depth, ply);
                        }
                    }
                    #[cfg(feature = "diag")]
                    if diag_order_sample && diag_best_rank > 0 {
                        crate::diag::record_best_move(
                            diag_best_rank,
                            diag_best_stage,
                            diag_best_reduced,
                        );
                        if !tt_move.is_null() {
                            crate::diag::record_contradiction_ordering(
                                diag_contradicts,
                                ev.hit,
                                diag_best_stage == MoveClass::TtMove,
                            );
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
                    crate::diag_count!(correction_on_capture);
                }
                let residual =
                    self.attributed_residual(diff, best_move.is_capture(), board.halfmove_clock);
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
                && self.params.exact_bonus_pct != 0
                && !best_move.is_null()
                && !best_move.is_capture()
                && !best_move.is_promo()
            {
                let bonus = self.history_bonus(depth) * self.params.exact_bonus_pct / 100;
                self.update_quiet_history(
                    board.side_to_move(),
                    best_move,
                    board.moving_piece(best_move),
                    board.pawn_key(),
                    ply,
                    bonus,
                );
            }
            self.tt.store(TtStore {
                key: hash,
                depth,
                score: best_score,
                bound,
                mv: best_move,
                ply,
                static_eval: raw_static_eval,
                is_pv: tt_pv,
                kind: OutcomeKind::Full,
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
                // 4.3 shadow, part 2. This node refined its pruning eval and
                // then completed, so compare which estimate sat closer to the
                // score it reported. Only reachable when the node was NOT
                // pruned — see the counter docs for why that biases it.
                if eval_for_pruning != static_eval && static_eval != VALUE_NONE {
                    crate::diag::record_refine_agreement(static_eval, eval_for_pruning, best_score);
                }
            }
        }
        #[cfg(feature = "diag")]
        if diag_order_sample && diag_best_rank > 0 {
            crate::diag::record_best_move(diag_best_rank, diag_best_stage, diag_best_reduced);
            if !tt_move.is_null() {
                crate::diag::record_contradiction_ordering(
                    diag_contradicts,
                    ev.hit,
                    diag_best_stage == MoveClass::TtMove,
                );
            }
        }
        best_score
    }

    fn quiescence<P: FnMut() -> SearchEvent + ?Sized>(
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
        self.pv_len[ply] = ply;
        self.seldepth = self.seldepth.max(ply);

        if board.can_declare_draw_in_search() {
            return 0;
        }

        let in_check = board.is_in_check();
        crate::diag_count!(qnodes);
        let hash = board.hash;
        #[cfg(feature = "diag")]
        let diag_q_sample = crate::diag::sampled(hash, ply + qply, crate::diag::SAMPLE_QSEARCH);
        #[cfg(feature = "diag")]
        if diag_q_sample {
            crate::diag_count!(sampled_qnodes);
            crate::diag_count!(shadow_4_3_qsearch);
            if in_check {
                crate::diag_count!(q_in_check);
            }
        }
        let original_alpha = alpha;
        let tt_entry = self.tt.probe(hash);
        let ev = NodeEvidence::from_probe(tt_entry, ply, board.halfmove_clock);
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
                self.tt.store(TtStore {
                    key: hash,
                    depth: 0,
                    score: stand_pat,
                    bound: Bound::Lower,
                    mv: Move::NULL,
                    ply,
                    static_eval: q_raw_static_eval,
                    is_pv: false,
                    kind: OutcomeKind::StandPat,
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
                    let see_threshold = (alpha - stand_pat_for_pruning - self.params.qs_see_margin)
                        .clamp(self.params.qs_see_clamp_lo, self.params.qs_see_clamp_hi);
                    if !board.see_ge(mv, see_threshold) {
                        continue;
                    }
                }
                if picked.see < 0 && !board.see_ge(mv, self.params.qs_see_bad_floor) {
                    continue;
                }
            }
            let moving_piece = board.moving_piece(mv);
            // Quiescence does not reduce, so it has no reduction inputs to
            // hand its children.
            self.push_move(ply, mv, moving_piece, 0, 0);
            board.make_move_unchecked(mv);
            self.tt.prefetch(board.hash);
            let score = -self.quiescence(board, -beta, -alpha, ply + 1, qply + 1, poll);
            board.unmake_move(mv);
            self.clear_move(ply);
            if self.stopped || self.quit {
                return 0;
            }
            if score >= beta {
                #[cfg(feature = "diag")]
                if diag_q_sample {
                    crate::diag_count!(q_move_cut);
                    crate::diag_count!(q_move_store);
                }
                self.tt.store(TtStore {
                    key: hash,
                    depth: 0,
                    score,
                    bound: Bound::Lower,
                    mv,
                    ply,
                    static_eval: q_raw_static_eval,
                    is_pv: false,
                    kind: OutcomeKind::QsearchMove,
                });
                return score;
            }
            if score > alpha {
                alpha = score;
                best_move = mv;
                self.pv_table[ply][ply] = mv;
                let child_len = self.pv_len[ply + 1].max(ply + 1);
                for next_ply in ply + 1..child_len {
                    self.pv_table[ply][next_ply] = self.pv_table[ply + 1][next_ply];
                }
                self.pv_len[ply] = child_len;
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
        self.tt.store(TtStore {
            key: hash,
            depth: 0,
            score: alpha,
            bound,
            mv: best_move,
            ply,
            static_eval: q_raw_static_eval,
            is_pv: false,
            kind: OutcomeKind::QsearchTail,
        });
        alpha
    }

    fn score_moves(
        &self,
        board: &Board,
        moves: &[Move],
        tt_move: Move,
        ply: usize,
    ) -> ScoredMoveList {
        let mut scored = ScoredMoveList::new();
        self.append_scored_moves(board, moves, tt_move, ply, &mut scored);
        scored
    }

    /// [`Self::score_moves`] writing into a caller-owned buffer, so the
    /// staged picker can append quiets behind the captures already sitting
    /// in its single partitioned list (10.3(4)).
    fn append_scored_moves(
        &self,
        board: &Board,
        moves: &[Move],
        tt_move: Move,
        ply: usize,
        scored: &mut ScoredMoveList,
    ) {
        let previous = if ply > 0 {
            self.stack[ply - 1].mv
        } else {
            Move::NULL
        };
        let counter = if !previous.is_null() {
            self.countermove[previous.from_sq().index()][previous.to_sq().index()]
        } else {
            Move::NULL
        };
        // 10.3: check masks computed at most once per node, and only if a
        // quiet actually reaches history scoring — capture-only lists (the
        // common qsearch case) never pay for it. 8.12(g2): the history row
        // bases follow the same lazy once-per-node pattern.
        let mut check_info = None;
        let mut quiet_ctx = None;
        // 9.7.5(d): node-invariant, so read once rather than per move — the
        // killers were being loaded FOUR times per move (twice in the tier
        // chain, twice more in the quiet-history re-test below).
        let killer0 = self.killers[ply][0];
        let killer1 = self.killers[ply][1];
        let stm = board.side_to_move();

        for &mv in moves {
            let mut see = 0;
            // 9.7.5(d): captured where the quiet branch computes it. The old
            // form re-derived "was this a quiet?" afterwards, repeating six
            // comparisons (capture, promo, tt-move, both killers, countermove)
            // that the tier chain below has already resolved.
            let mut quiet_history = 0;
            let score = if mv == tt_move {
                30_000_000
            } else if mv.is_capture() {
                let attacker = board.moving_piece(mv);
                let victim = board.captured_piece(mv).unwrap_or(Piece::Pawn);
                see = board.see(mv);
                let hist =
                    self.cap_history[attacker as usize][mv.to_sq().index()][victim as usize] as i32;
                if see >= 0 {
                    20_000_000 + 32 * see + 10 * piece_value(victim) - piece_value(attacker) + hist
                } else {
                    -2_000_000 + see + hist
                }
            } else if mv.is_promo() {
                18_000_000 + piece_value(mv.promo_piece())
            } else if mv == killer0 {
                16_000_000
            } else if mv == killer1 {
                15_900_000
            } else if mv == counter {
                15_800_000
            } else {
                let ci = check_info.get_or_insert_with(|| board.check_info());
                let ctx = quiet_ctx.get_or_insert_with(|| self.quiet_history_ctx(board, ply));
                quiet_history = self.quiet_history_score(board, ci, ctx, stm, mv, ply);
                quiet_history
            };
            scored.push_with_history(mv, score, see, quiet_history);
        }
    }

    fn score_staged_captures(
        &self,
        board: &Board,
        moves: &[Move],
        tt_move: Move,
    ) -> (ScoredMoveList, usize, usize) {
        // Two passes so the partition lands in one buffer without moving
        // anything: good captures first, then bad ones appended behind them.
        // Scoring is pure, so scoring twice is only arithmetic — and the
        // second pass runs over the (usually small) bad-capture subset.
        let mut out = ScoredMoveList::new();
        for &mv in moves {
            if mv == tt_move {
                continue;
            }
            let scored = self.score_tactical_move(board, mv, tt_move);
            if scored.see >= 0 || mv.is_promo() {
                out.push(scored.mv, scored.score, scored.see as i32);
            }
        }
        let good_len = out.len();
        for &mv in moves {
            if mv == tt_move {
                continue;
            }
            let scored = self.score_tactical_move(board, mv, tt_move);
            if !(scored.see >= 0 || mv.is_promo()) {
                out.push(scored.mv, scored.score, scored.see as i32);
            }
        }
        let cap_len = out.len();
        (out, good_len, cap_len)
    }

    fn score_tactical_moves(&self, board: &Board, moves: &[Move], tt_move: Move) -> ScoredMoveList {
        let mut scored = ScoredMoveList::new();
        for &mv in moves {
            let scored_move = self.score_tactical_move(board, mv, tt_move);
            scored.push(scored_move.mv, scored_move.score, scored_move.see as i32);
        }
        scored
    }

    fn score_tactical_move(&self, board: &Board, mv: Move, tt_move: Move) -> ScoredMove {
        let mut see = 0;
        let score = if mv == tt_move {
            if mv.is_capture() && !board.see_ge(mv, 0) {
                see = -1;
            }
            30_000_000
        } else if mv.is_capture() {
            let attacker = board.moving_piece(mv);
            let victim = board.captured_piece(mv).unwrap_or(Piece::Pawn);
            let promo_gain = if mv.is_promo() {
                piece_value(mv.promo_piece()) - piece_value(Piece::Pawn)
            } else {
                0
            };
            let hist =
                self.cap_history[attacker as usize][mv.to_sq().index()][victim as usize] as i32;
            if board.see_ge(mv, 0) {
                20_000_000 + 16 * (piece_value(victim) + promo_gain) - piece_value(attacker) + hist
            } else {
                see = -1;
                -2_000_000 + 16 * (piece_value(victim) + promo_gain) - piece_value(attacker) + hist
            }
        } else if mv.is_promo() {
            18_000_000 + piece_value(mv.promo_piece())
        } else {
            0
        };

        ScoredMove {
            mv,
            score,
            see: crate::infra::saturating_i16(see),
            quiet_history: 0,
        }
    }

    /// Resolve the node-invariant half of quiet-history indexing once per
    /// node (8.12(g2), from the Basilisk cross-review — its 8.7.6(b+d) hoist,
    /// +3.03% NPS there). The continuation guards (`ply < back`, null
    /// previous move), the previous piece/square loads, and the pawn-key row
    /// lookup do not depend on the move being scored, yet `cont_score` used
    /// to redo all of them for every quiet in the list. 8.12(g) refuted the
    /// PREFETCH angle for these tables (all quiets share one row window per
    /// node) but never isolated the duplicated arithmetic; this removes it.
    /// Only `piece_to_index(piece, to)` remains per-move.
    fn quiet_history_ctx(&self, board: &Board, ply: usize) -> QuietHistoryCtx {
        let mut cont_bases = [None; CONT_TABLES];
        for (slot, &(back, _)) in CONT_PLY_BACK.iter().enumerate() {
            if ply < back {
                continue;
            }
            let prev = self.stack[ply - back].mv;
            if prev.is_null() {
                continue;
            }
            cont_bases[slot] = Some(self.stack[ply - back].cont_row_base());
        }
        QuietHistoryCtx {
            cont_bases,
            pawn_base: pawn_row_base(board.pawn_key()),
        }
    }

    fn quiet_history_score(
        &self,
        board: &Board,
        check_info: &CheckInfo,
        ctx: &QuietHistoryCtx,
        color: Color,
        mv: Move,
        ply: usize,
    ) -> i32 {
        let from = mv.from_sq().index();
        let to = mv.to_sq().index();
        let main = self.main_history[color as usize][from][to] as i32;
        let piece = board.moving_piece(mv) as usize;
        // The shared per-move offset into every (piece, to)-shaped row.
        let piece_to = piece_to_index(piece, to);
        let pawn = self.pawn_history[ctx.pawn_base + piece_to] as i32;
        let low_ply = if ply < LOW_PLY_HISTORY_SIZE {
            self.low_ply_history[ply][from][to] as i32 / (1 + infra::to_i32(ply))
        } else {
            0
        };
        let mut cont = 0;
        for (slot, base) in ctx.cont_bases.iter().enumerate() {
            if let Some(base) = base {
                cont += self.cont_history[slot][(base + piece_to).min(CONT_SIZE - 1)] as i32;
            }
        }
        // 4.6c: safe versus losing check classes. A check whose checker can be
        // taken at a material loss is usually refuted by taking it, so it does
        // not deserve the same enormous bonus as a safe one. When the two
        // bonuses are equal (the seeded state) the SEE probe is SKIPPED, so
        // ordering pays nothing for a distinction it is not making.
        let direct_check = if board.gives_check_with(mv, check_info) {
            // 4.6c: the safe/losing SPLIT WAS REVERTED. `see_ge(mv, 0)` returned
            // true for all 332,683 quiet checking moves on bench and `losing`
            // counted ZERO, so the split could never fire — `see_ge` is
            // evidently trivially satisfied for a non-capturing move, making it
            // the wrong predicate for "the checker can be taken at a loss".
            // Shipping a knob that cannot fire would be dead code dressed as a
            // tunable, so only the census survives; a correct classifier needs a
            // different test (is the destination defended, or the checker
            // attacked by a lesser piece) and belongs in 4.10's ordering work.
            #[cfg(feature = "diag")]
            if board.see_ge(mv, 0) {
                crate::diag_count!(check_order_safe);
            } else {
                crate::diag_count!(check_order_losing);
            }
            self.params.check_bonus_safe
        } else {
            0
        };
        2 * main + pawn + low_ply + cont + direct_check
    }

    /// Reward for the move that produced a beta cutoff (Phase 8.1: linear
    /// SF-shaped formula, split from the malus so SPSA can tune them apart).
    fn history_bonus(&self, depth: i32) -> i32 {
        (self.params.hist_bonus_mul * depth - self.params.hist_bonus_sub)
            .clamp(0, self.params.hist_bonus_max)
    }

    /// Penalty magnitude for searched moves that failed to cut (applied
    /// negated). Stored positive.
    fn history_malus(&self, depth: i32) -> i32 {
        (self.params.hist_malus_mul * depth - self.params.hist_malus_sub)
            .clamp(0, self.params.hist_malus_max)
    }

    fn update_cutoff_tables(
        &mut self,
        board: &Board,
        best: Move,
        best_piece: Piece,
        previous: Move,
        ply: usize,
        depth: i32,
        bonus_pct: i32,
        quiets: &[Move],
        good_caps: &BadCaptureList,
        bad_caps: &BadCaptureList,
    ) {
        if self.killers[ply][0] != best {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = best;
        }

        let color = board.side_to_move();
        let pawn_key = board.pawn_key();
        // 8.4(e): `bonus_pct` carries the surprise scale (100 = neutral); it
        // applies to every REWARD for the best move (main/pawn/low-ply and the
        // continuation entries below) but never to a malus.
        let bonus = self.history_bonus(depth) * bonus_pct / 100;
        let malus = self.history_malus(depth);
        self.update_quiet_history(color, best, best_piece, pawn_key, ply, bonus);
        // NOTE, 4.5.3: these quiets get a malus in main, low-ply and pawn
        // history but deliberately NOT in continuation history. That asymmetry
        // looks like an omission and was measured as a candidate: adding the
        // continuation malus leaves ordering flat (first-move cutoff 88.04% ->
        // 88.09%) while cutting the tree 7.5% and total cutoffs 9.6%. Cutoffs
        // fall FASTER than nodes, so it is not an ordering gain — continuation
        // history feeds `quiet_hist`, which drives two of LMP's four disjuncts
        // and the LMR reduction, so a broad negative push simply prunes more.
        // That is the one direction four independent readings say is wrong for
        // this engine (RAR-S53/S54/S55, and 4.7 paying +15.56 for pruning
        // LESS). Rejected on measurement, not left undone.
        for &quiet in quiets {
            let quiet_piece = board.moving_piece(quiet);
            self.update_quiet_history(color, quiet, quiet_piece, pawn_key, ply, -malus);
        }
        for good_cap in good_caps.as_slice() {
            self.update_capture_history(
                good_cap.attacker,
                good_cap.to as usize,
                good_cap.captured,
                -malus,
            );
        }
        for bad_cap in bad_caps.as_slice() {
            self.update_capture_history(
                bad_cap.attacker,
                bad_cap.to as usize,
                bad_cap.captured,
                -malus,
            );
        }

        if !previous.is_null() {
            self.countermove[previous.from_sq().index()][previous.to_sq().index()] = best;
        }

        let piece = best_piece as usize;
        let to = best.to_sq().index();
        for (slot, &(back, divisor)) in CONT_PLY_BACK.iter().enumerate() {
            if ply < back {
                continue;
            }
            let prev = self.stack[ply - back].mv;
            if prev.is_null() {
                continue;
            }
            let index = self.stack[ply - back].cont_row_base() + piece_to_index(piece, to);
            update_hist_entry(
                &mut self.cont_history[slot][index],
                bonus / divisor,
                HISTORY_MAX,
            );
        }
    }

    fn update_quiet_history(
        &mut self,
        color: Color,
        mv: Move,
        piece: Piece,
        pawn_key: u64,
        ply: usize,
        bonus: i32,
    ) {
        update_hist_entry(
            &mut self.main_history[color as usize][mv.from_sq().index()][mv.to_sq().index()],
            bonus,
            HISTORY_MAX,
        );
        if ply < LOW_PLY_HISTORY_SIZE {
            update_hist_entry(
                &mut self.low_ply_history[ply][mv.from_sq().index()][mv.to_sq().index()],
                bonus,
                HISTORY_MAX,
            );
        }
        update_hist_entry(
            &mut self.pawn_history
                [pawn_history_index(pawn_key, piece as usize, mv.to_sq().index())],
            bonus,
            HISTORY_MAX,
        );
    }

    fn update_capture_history(
        &mut self,
        attacker: Piece,
        to: usize,
        captured: Option<Piece>,
        bonus: i32,
    ) {
        if let Some(captured) = captured {
            update_hist_entry(
                &mut self.cap_history[attacker as usize][to][captured as usize],
                bonus,
                CAP_HISTORY_MAX,
            );
        }
    }

    fn age_history(&mut self) {
        for color in self.main_history.iter_mut() {
            for from in color.iter_mut() {
                for value in from.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for attacker in self.cap_history.iter_mut() {
            for to in attacker.iter_mut() {
                for value in to.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for ply in self.low_ply_history.iter_mut() {
            for from in ply.iter_mut() {
                for value in from.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for value in self.pawn_history.iter_mut() {
            *value /= 2;
        }
        for table in self.cont_history.iter_mut() {
            for value in table.iter_mut() {
                *value /= 2;
            }
        }
        for color in self.correction_history.iter_mut() {
            for value in color.iter_mut() {
                *value /= 2;
            }
        }
        for color in self.minor_correction_history.iter_mut() {
            for value in color.iter_mut() {
                *value /= 2;
            }
        }
        for stm in self.non_pawn_correction_history.iter_mut() {
            for color in stm.iter_mut() {
                for value in color.iter_mut() {
                    *value /= 2;
                }
            }
        }
        // 4.5b: all three continuation tables age through the same loop, which
        // is the "centralize saturation/aging" half of PLAN 4.5 - a table that
        // ages on a different schedule from its siblings drifts out of scale
        // with them and the weights stop meaning what they meant when fitted.
        for value in self
            .continuation_correction_history
            .iter_mut()
            .chain(self.continuation_correction_2ply.iter_mut())
            .chain(self.continuation_correction_4ply.iter_mut())
        {
            *value /= 2;
        }
    }

    /// 8.13: fold the pool's per-root-move knowledge into this thread's root
    /// ordering.
    ///
    /// A move that another thread has already searched deeper gets lifted
    /// above the local heuristic ordering, ranked by (depth, score). The
    /// shared TT already carries much of this implicitly, but root entries
    /// are overwritten under pressure while these slots are not, so the
    /// explicit channel survives exactly the case it is needed in.
    fn apply_shared_root_scores(&self, legal_moves: &[Move], scored: &mut ScoredMoveList) {
        let Some(shared) = &self.shared_state else {
            return;
        };
        for entry in scored.as_mut_slice() {
            let Some(index) = legal_moves.iter().position(|mv| *mv == entry.mv) else {
                continue;
            };
            let Some((depth, score, bound)) = shared.root_score(index) else {
                continue;
            };
            // Rank above every locally-scored quiet but below the TT move, so
            // the pool refines the ordering rather than overriding the one
            // move we already know is best here. An Upper-bound entry (a
            // proven fail-low) is demoted by half a depth step — the pool has
            // evidence AGAINST the move, so it should sort below same-depth
            // moves whose scores are trustworthy.
            let upper_penalty = if bound == RootBound::Upper { 2_048 } else { 0 };
            entry.score = 25_000_000 + depth * 4_096 + score.clamp(-30_000, 30_000) - upper_penalty;
        }
    }

    /// Publish and retain one completed root-move visit outside the hot node
    /// kernel. The single cold call replaces several root-only branches that
    /// the first 10.1 implementation placed directly in `negamax` and that
    /// measured about -0.8% best-of NPS despite running only at the root.
    #[cold]
    #[inline(never)]
    fn record_root_move_search(
        &mut self,
        mv: Move,
        depth: i32,
        score: i32,
        alpha: i32,
        beta: i32,
        nodes: u64,
    ) {
        let Some(index) = self
            .root_moves
            .iter()
            .position(|root_move| *root_move == mv)
        else {
            // Direct diagnostic/unit calls may enter root negamax without the
            // normal `search_root` initialization. Search remains valid; there
            // is simply no persistent table to update on that path.
            return;
        };
        let bound = if score >= beta {
            RootBound::Lower
        } else if score > alpha {
            RootBound::Exact
        } else {
            RootBound::Upper
        };
        if let Some(shared) = &self.shared_state {
            shared.publish_root_score(index, depth, score, bound);
        }

        let root_move = &mut self.root_move_records[index];
        // This is the cumulative search-wide seldepth at the time the move
        // completes (the same low-cost shape used by Basilisk), so a later move
        // may inherit a deeper earlier move's maximum. Exact per-move tracking
        // required extra branches in every recursive move loop and measured a
        // real speed loss; 10.2 should treat this field as a conservative max.
        root_move.record_search(infra::to_usize(depth), score, nodes, self.seldepth, bound);
        if score > alpha {
            let child_len = self.pv_len[1].clamp(1, MAX_PLY);
            root_move.pv[0] = mv;
            root_move.pv[1..child_len].copy_from_slice(&self.pv_table[1][1..child_len]);
            root_move.pv_len = child_len;
        }
    }

    fn corrected_eval(&mut self, board: &Board, ply: usize) -> i32 {
        let raw = self.raw_eval(board);
        self.corrected_eval_from_raw(board, raw, ply)
    }

    fn raw_eval(&mut self, board: &Board) -> i32 {
        self.evaluator.evaluate(board)
    }

    fn corrected_eval_from_raw(&self, board: &Board, raw: i32, ply: usize) -> i32 {
        raw + self.correction_value(board, ply)
    }

    fn correction_value(&self, board: &Board, ply: usize) -> i32 {
        let color = board.side_to_move();
        let us = color as usize;
        let them = (!color) as usize;
        let pawn =
            self.correction_history[us][infra::index(board.pawn_key()) & (CORR_SIZE - 1)] as i32;
        let minor = self.minor_correction_history[us]
            [infra::index(board.minor_key()) & (CORR_SIZE - 1)] as i32;
        let own_non_pawn = self.non_pawn_correction_history[us][us]
            [infra::index(board.non_pawn_key(color)) & (CORR_SIZE - 1)]
            as i32;
        let their_non_pawn = self.non_pawn_correction_history[us][them]
            [infra::index(board.non_pawn_key(!color)) & (CORR_SIZE - 1)]
            as i32;
        let continuation = if ply >= 1 {
            let prev = self.stack[ply - 1].mv;
            if prev.is_null() {
                0
            } else {
                self.continuation_correction_history[self.stack[ply - 1].cont_key] as i32
            }
        } else {
            0
        };
        // 8.5(c): per-source weights (seed 128 = the old unit weight; the
        // continuation term keeps its inherent `/2`). `Σ src·W / 16384`
        // reproduces the old `(pawn+minor+own_np+their_np+cont/2)/128` bit-for-
        // bit at seed, since `Σsrc·128/16384 == Σsrc/128` in integer division.
        // 4.5b: distance-2 and distance-4 continuation terms. Both reads are
        // skipped entirely at the seeded weight of 0, so the default costs not
        // even a table lookup.
        let cont2 = self.continuation_at(ply, 2, self.params.corr_w_cont2);
        let cont4 = self.continuation_at(ply, 4, self.params.corr_w_cont4);
        (pawn * self.params.corr_w_pawn
            + minor * self.params.corr_w_minor
            + own_non_pawn * self.params.corr_w_own_np
            + their_non_pawn * self.params.corr_w_their_np
            + (continuation / 2) * self.params.corr_w_cont
            + (cont2 / 2) * self.params.corr_w_cont2
            + (cont4 / 2) * self.params.corr_w_cont4)
            / 16384
    }

    fn syzygy_wdl_score(
        &mut self,
        board: &Board,
        depth: i32,
        ply: usize,
        excluded: Move,
    ) -> Option<i32> {
        if ply == 0 || !excluded.is_null() || !self.can_probe_syzygy(board, depth) {
            return None;
        }
        let wdl = syzygy::probe_wdl(board, self.syzygy_50_move_rule)?;
        self.record_tb_hit();
        Some(self.score_from_syzygy_wdl(wdl, ply))
    }

    fn can_probe_syzygy(&self, board: &Board, depth: i32) -> bool {
        self.syzygy_largest > 0
            && depth >= self.syzygy_probe_depth
            && board.castling.0 == 0
            && board.occupied_count() as usize <= self.syzygy_largest
    }

    fn can_probe_syzygy_root(&self, board: &Board) -> bool {
        self.syzygy_largest > 0
            && board.castling.0 == 0
            && board.occupied_count() as usize <= self.syzygy_largest
    }

    fn score_from_syzygy_wdl(&self, wdl: Wdl, ply: usize) -> i32 {
        match wdl {
            Wdl::Win => TB_WIN_SCORE - infra::to_i32(ply),
            Wdl::CursedWin if !self.syzygy_50_move_rule => TB_WIN_SCORE - infra::to_i32(ply),
            Wdl::Loss => -TB_WIN_SCORE + infra::to_i32(ply),
            Wdl::BlessedLoss if !self.syzygy_50_move_rule => -TB_WIN_SCORE + infra::to_i32(ply),
            Wdl::BlessedLoss | Wdl::Draw | Wdl::CursedWin => 0,
        }
    }

    /// 4.5: weight a correction residual by what produced it.
    ///
    /// At the seeded `CorrCaptureWeightPct = 100` this returns `diff` unchanged,
    /// so the default is exactly inert. Below 100 a capture-caused residual is
    /// down-weighted rather than discarded — the graded alternative to
    /// `corr_guard_capture`, whose binary exclusion RAR-S16 measured at −55.98
    /// Elo because it threw away 51.3% of all training.
    ///
    /// Also records the residual magnitude per attribution class, which is the
    /// measurement that decides whether down-weighting is justified at all: if
    /// capture-caused residuals are no noisier than quiet ones, the premise
    /// behind both this knob and `corr_guard_capture` is wrong.
    #[inline(always)]
    fn attributed_residual(&self, diff: i32, from_capture: bool, halfmove: u8) -> i32 {
        #[cfg(feature = "diag")]
        {
            let magnitude = u64::from(diff.unsigned_abs());
            if from_capture {
                crate::diag_count!(correction_resid_capture_n);
                crate::diag_add!(correction_resid_capture_sum, magnitude);
            } else {
                crate::diag_count!(correction_resid_quiet_n);
                crate::diag_add!(correction_resid_quiet_sum, magnitude);
            }
            // 4.5d: halfmove-clock context. PLAN 4.5 permits a new correction
            // context only where held-out UNIQUE signal is shown, so measure the
            // residual per bucket before proposing one. Rule-50 proximity is the
            // plausible mechanism: near the horizon a position's value stops
            // being a function of its structure at all.
            match halfmove {
                0..=19 => {
                    crate::diag_count!(correction_resid_hm_low_n);
                    crate::diag_add!(correction_resid_hm_low_sum, magnitude);
                }
                20..=49 => {
                    crate::diag_count!(correction_resid_hm_mid_n);
                    crate::diag_add!(correction_resid_hm_mid_sum, magnitude);
                }
                _ => {
                    crate::diag_count!(correction_resid_hm_high_n);
                    crate::diag_add!(correction_resid_hm_high_sum, magnitude);
                }
            }
        }
        // The clock is a diagnostic input only; production reads it nowhere, so
        // discard it explicitly rather than renaming the parameter to `_halfmove`
        // and losing the name at both call sites.
        #[cfg(not(feature = "diag"))]
        {
            let _ = halfmove;
        }
        if from_capture && self.params.corr_capture_weight_pct != 100 {
            diff * self.params.corr_capture_weight_pct / 100
        } else {
            diff
        }
    }

    fn update_correction(&mut self, board: &Board, diff: i32, depth: i32, ply: usize) {
        let color = board.side_to_move();
        let us = color as usize;
        let them = (!color) as usize;
        let scaled = (diff * depth.max(1)).clamp(-1024, 1024);
        #[cfg(feature = "diag")]
        if crate::diag::sampled(board.hash, ply, crate::diag::SAMPLE_CORRECTION) {
            crate::diag_count!(correction_sample_updates);
            crate::diag_count!(shadow_4_5_correction);
            crate::diag_add!(correction_sample_abs_sum, u64::from(diff.unsigned_abs()));
            let pawn_key = board.pawn_key();
            let minor_key = board.minor_key();
            let own_key = board.non_pawn_key(color);
            let other_key = board.non_pawn_key(!color);
            let pawn_index = infra::index(pawn_key) & (CORR_SIZE - 1);
            let minor_index = infra::index(minor_key) & (CORR_SIZE - 1);
            let own_index = infra::index(own_key) & (CORR_SIZE - 1);
            let other_index = infra::index(other_key) & (CORR_SIZE - 1);
            crate::diag::record_correction_slot(
                0,
                us * CORR_SIZE + pawn_index,
                pawn_key,
                self.correction_history[us][pawn_index],
            );
            crate::diag::record_correction_slot(
                1,
                us * CORR_SIZE + minor_index,
                minor_key,
                self.minor_correction_history[us][minor_index],
            );
            crate::diag::record_correction_slot(
                2,
                us * 2 * CORR_SIZE + us * CORR_SIZE + own_index,
                own_key,
                self.non_pawn_correction_history[us][us][own_index],
            );
            crate::diag::record_correction_slot(
                3,
                us * 2 * CORR_SIZE + them * CORR_SIZE + other_index,
                other_key,
                self.non_pawn_correction_history[us][them][other_index],
            );
        }
        update_hist_entry(
            &mut self.correction_history[us][infra::index(board.pawn_key()) & (CORR_SIZE - 1)],
            scaled,
            HISTORY_MAX,
        );
        update_hist_entry(
            &mut self.minor_correction_history[us]
                [infra::index(board.minor_key()) & (CORR_SIZE - 1)],
            scaled,
            HISTORY_MAX,
        );
        update_hist_entry(
            &mut self.non_pawn_correction_history[us][us]
                [infra::index(board.non_pawn_key(color)) & (CORR_SIZE - 1)],
            scaled,
            HISTORY_MAX,
        );
        update_hist_entry(
            &mut self.non_pawn_correction_history[us][them]
                [infra::index(board.non_pawn_key(!color)) & (CORR_SIZE - 1)],
            scaled,
            HISTORY_MAX,
        );
        if ply >= 1 {
            let prev = self.stack[ply - 1].mv;
            if !prev.is_null() {
                update_hist_entry(
                    &mut self.continuation_correction_history[self.stack[ply - 1].cont_key],
                    scaled / 2,
                    HISTORY_MAX,
                );
            }
        }
        // 4.5b: same keying at distance 2 and 4. Writes are skipped at weight 0
        // so an unused table is never touched; enabling a weight simply starts
        // from an empty table, exactly as a fresh `new_game` would.
        if self.params.corr_w_cont2 != 0
            && let Some(index) = self.continuation_index(ply, 2)
        {
            update_hist_entry(
                &mut self.continuation_correction_2ply[index],
                scaled / 2,
                HISTORY_MAX,
            );
        }
        if self.params.corr_w_cont4 != 0
            && let Some(index) = self.continuation_index(ply, 4)
        {
            update_hist_entry(
                &mut self.continuation_correction_4ply[index],
                scaled / 2,
                HISTORY_MAX,
            );
        }
    }

    /// Record the move being searched at `ply`, deriving its continuation key.
    ///
    /// The ONLY way to put a move on the stack. Writing `mv` and `piece`
    /// separately is what let ProbCut desynchronise them (see `NodeContext`).
    #[inline]
    fn push_move(&mut self, ply: usize, mv: Move, piece: Piece, move_count: i32, stat_score: i32) {
        self.stack[ply].mv = mv;
        self.stack[ply].piece = piece;
        self.stack[ply].cont_key = piece_to_index(piece as usize, mv.to_sq().index());
        self.stack[ply].move_count = move_count;
        self.stack[ply].stat_score = stat_score;
    }

    /// Clear the move at `ply`. The static eval is deliberately preserved: it
    /// belongs to the node, not to the move being tried at it.
    #[inline]
    fn clear_move(&mut self, ply: usize) {
        self.stack[ply].mv = Move::NULL;
        self.stack[ply].cont_key = 0;
        self.stack[ply].move_count = 0;
        self.stack[ply].stat_score = 0;
    }

    /// 4.5b: compact `(piece, to)` key for the move `distance` plies back, or
    /// `None` when that ply does not exist or held a null move.
    #[inline(always)]
    fn continuation_index(&self, ply: usize, distance: usize) -> Option<usize> {
        if ply < distance {
            return None;
        }
        let prev = self.stack[ply - distance].mv;
        if prev.is_null() {
            return None;
        }
        Some(self.stack[ply - distance].cont_key)
    }

    /// 4.5b: read the distance-`distance` continuation correction, or 0 when the
    /// term is switched off. Checking the weight FIRST is what keeps the seeded
    /// default free of a table access.
    #[inline(always)]
    fn continuation_at(&self, ply: usize, distance: usize, weight: i32) -> i32 {
        if weight == 0 {
            return 0;
        }
        let table = if distance == 2 {
            &self.continuation_correction_2ply
        } else {
            &self.continuation_correction_4ply
        };
        self.continuation_index(ply, distance)
            .map_or(0, |index| i32::from(table[index]))
    }

    /// True when mechanism `bit` is ablated. Const `false` without the
    /// feature, so every guard below folds away in a shipped build.
    #[cfg(feature = "ablate")]
    #[inline]
    fn ablated(&self, bit: u32) -> bool {
        (self.params.ablation_mask >> bit) & 1 == 1
    }

    #[cfg(not(feature = "ablate"))]
    #[inline]
    #[expect(clippy::unused_self, reason = "mirrors the ablate-feature signature")]
    fn ablated(&self, _bit: u32) -> bool {
        false
    }

    fn check_stop<P: FnMut() -> SearchEvent + ?Sized>(&mut self, poll: &mut P) -> bool {
        let total_nodes = self.record_node();
        if let Some(shared_state) = &self.shared_state
            && (self.limits.nodes > 0 || self.nodes & SHARED_NODE_BATCH_MASK == 0)
        {
            match shared_state.stop_state.load(Ordering::Relaxed) {
                STOP_QUIT => {
                    self.quit = true;
                    self.stopped = true;
                    return true;
                }
                STOP_SEARCH => {
                    self.stopped = true;
                    return true;
                }
                _ => {}
            }
        }
        if self.limits.nodes > 0 && total_nodes >= self.limits.nodes {
            self.stopped = true;
            if let Some(shared_state) = &self.shared_state {
                shared_state.request_stop();
            }
            return true;
        }
        if self.nodes & 2047 == 0 {
            match poll() {
                SearchEvent::Quit => {
                    self.quit = true;
                    self.stopped = true;
                }
                SearchEvent::Stop => {
                    self.stopped = true;
                }
                SearchEvent::PonderHit => {
                    self.pondering = false;
                    self.ponderhit = true;
                    if self.stop_on_ponderhit {
                        self.stopped = true;
                    }
                    if let Some(shared_state) = &self.shared_state {
                        shared_state.ponderhit.store(true, Ordering::Relaxed);
                    }
                }
                SearchEvent::None => {}
            }
            if !self.pondering && self.elapsed_ms() >= self.limits.maximum_ms {
                self.stopped = true;
            }
        }
        self.stopped
    }

    fn record_node(&mut self) -> u64 {
        self.nodes += 1;
        if let Some(shared_state) = &self.shared_state {
            let pending = self.nodes & SHARED_NODE_BATCH_MASK;
            if pending == 0 {
                shared_state
                    .nodes
                    .fetch_add(SHARED_NODE_BATCH, Ordering::Relaxed)
                    + SHARED_NODE_BATCH
            } else if self.limits.nodes > 0 {
                shared_state.nodes.load(Ordering::Relaxed) + pending
            } else {
                self.nodes
            }
        } else {
            self.nodes
        }
    }

    fn record_tb_hit(&mut self) {
        self.tb_hits += 1;
        if let Some(shared_state) = &self.shared_state {
            shared_state.tb_hits.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn reported_nodes(&self) -> u64 {
        self.shared_state
            .as_ref()
            .map_or(self.nodes, |shared_state| {
                shared_state.nodes.load(Ordering::Relaxed) + (self.nodes & SHARED_NODE_BATCH_MASK)
            })
    }

    fn reported_tb_hits(&self) -> u64 {
        self.shared_state
            .as_ref()
            .map_or(self.tb_hits, |shared_state| {
                shared_state.tb_hits.load(Ordering::Relaxed)
            })
    }

    fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    fn send_info(&self, depth: usize, score: i32) {
        let pv = self.pv_table[0][..self.pv_len[0].min(MAX_PLY)]
            .iter()
            .copied()
            .filter(|mv| !mv.is_null())
            .collect::<Vec<_>>();
        self.send_info_line(depth, score, &pv);
    }

    fn send_info_line(&self, depth: usize, score: i32, pv: &[Move]) {
        let elapsed_ms = self.start.elapsed().as_millis();
        let nodes = self.reported_nodes();
        let tb_hits = self.reported_tb_hits();
        let nps = (nodes as u128 * 1000)
            .checked_div(elapsed_ms)
            .unwrap_or(nodes as u128);
        let pv = pv
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "info depth {} seldepth {} score {} nodes {} nps {} hashfull {} tbhits {} time {} pv {}",
            depth,
            self.seldepth,
            format_score(score),
            nodes,
            nps,
            self.hashfull(),
            tb_hits,
            elapsed_ms,
            pv
        );
    }

    fn ponder_from_tt(&self, root: &Board, bestmove: Move) -> Move {
        if bestmove.is_null() {
            return Move::NULL;
        }
        let Some(bestmove) = root.legal_move(bestmove) else {
            return Move::NULL;
        };
        let mut child = root.clone();
        child.make_move_unchecked(bestmove);
        self.tt
            .probe(child.hash)
            .and_then(super::tt::TtEntry::best_move)
            .and_then(|mv| child.legal_move(mv))
            .unwrap_or(Move::NULL)
    }

    fn result_for_no_legal_moves(&self, board: &Board) -> GameResult {
        if board.is_in_check() {
            match board.side_to_move() {
                Color::White => GameResult::BlackCheckmates,
                Color::Black => GameResult::WhiteCheckmates,
            }
        } else {
            GameResult::Stalemate
        }
    }
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

fn select_parallel_result(results: &[SearchResult], root_moves: &[Move]) -> Option<SearchResult> {
    let root_results = results
        .iter()
        .enumerate()
        .filter(|(_, result)| is_root_result(result, root_moves))
        .collect::<Vec<_>>();
    let min_score = root_results.iter().map(|(_, result)| result.score).min()?;

    let mut votes: Vec<(Move, i64)> = Vec::new();
    for (_, result) in &root_results {
        let vote_value = parallel_vote_value(result, min_score);
        if let Some(vote) = votes.iter_mut().find(|(mv, _)| *mv == result.bestmove) {
            vote.1 += vote_value;
        } else {
            votes.push((result.bestmove, vote_value));
        }
    }

    root_results
        .into_iter()
        .max_by(|(left_index, left), (right_index, right)| {
            let left_vote = vote_for_move(&votes, left.bestmove);
            let right_vote = vote_for_move(&votes, right.bestmove);
            parallel_result_key(left, left_vote, *left_index == 0).cmp(&parallel_result_key(
                right,
                right_vote,
                *right_index == 0,
            ))
        })
        .map(|(_, result)| result.clone())
}

fn is_root_result(result: &SearchResult, root_moves: &[Move]) -> bool {
    result.depth > 0 && root_moves.contains(&result.bestmove)
}

fn parallel_vote_value(result: &SearchResult, min_score: i32) -> i64 {
    let score_weight = (result.score as i64 - min_score as i64 + 14).max(1);
    score_weight * i64::try_from(result.depth.max(1)).unwrap_or(i64::MAX)
}

fn vote_for_move(votes: &[(Move, i64)], mv: Move) -> i64 {
    votes
        .iter()
        .find_map(|(vote_move, vote)| (*vote_move == mv).then_some(*vote))
        .unwrap_or(0)
}

fn parallel_result_key(
    result: &SearchResult,
    vote: i64,
    main_thread: bool,
) -> (i32, i64, bool, usize, i32, bool) {
    let decisive_rank = if result.score >= TB_WIN_SCORE {
        2
    } else if result.score <= -TB_WIN_SCORE {
        0
    } else {
        1
    };
    (
        decisive_rank,
        vote,
        !result.pondermove.is_null(),
        result.depth,
        result.score,
        main_thread,
    )
}

fn format_score(score: i32) -> String {
    if score >= MATE_SCORE - infra::to_i32(MAX_PLY) {
        format!("mate {}", (MATE_SCORE - score + 1) / 2)
    } else if score <= -MATE_SCORE + infra::to_i32(MAX_PLY) {
        format!("mate -{}", (MATE_SCORE + score + 1) / 2)
    } else {
        format!("cp {score}")
    }
}

#[cfg(test)]
mod tests {
    /// The evaluator mirrors `MAX_PLY` to bound its mop-up drive below the mate
    /// band without taking a dependency on the search. This is the assertion
    /// that keeps the mirror honest, and it lives here because this module is
    /// the one that legitimately sees both constants (PLAN 4.10.11).
    #[test]
    fn mopup_mirror_matches_the_real_ply_horizon() {
        assert_eq!(
            crate::eval::MOPUP_ASSUMED_MAX_PLY as usize,
            super::MAX_PLY,
            "eval.rs mirrors MAX_PLY to bound the mop-up drive; the two have \
             drifted, so the compile-time bound in eval.rs is now checking \
             the wrong number"
        );
    }

    use super::*;
    use crate::board::Square;
    use std::time::{Duration, Instant};

    const LAZY_MARGIN_REGRESSION_FEN: &str = "5k2/5p1p/p3B1p1/Pp6/1P6/5P1P/4K1P1/8 b - - 0 1";

    /// A.3.3 (RAR-R11): the budget is measured from the instant `go` was
    /// parsed, not from the engine thread's start. A `go movetime 200`
    /// parsed 300 ms ago has already spent its budget and must return at
    /// once with a legal move from the first completed iteration.
    #[test]
    fn search_clock_starts_when_go_was_parsed() {
        let board = Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).unwrap();
        let legal = board.generate_legal_moves();
        let limits = SearchLimits {
            move_time: 200,
            issued: Some(Instant::now() - Duration::from_millis(300)),
            ..SearchLimits::default()
        };
        let mut searcher = Searcher::default();
        searcher.reset_search_state(
            &limits,
            &EngineOptions::default(),
            board.side_to_move(),
            0,
            true,
            true,
        );
        let started = Instant::now();
        let result = searcher.search_root(board, &legal, false, &mut || SearchEvent::None);
        let took = started.elapsed();
        assert!(
            took < Duration::from_millis(100),
            "a search whose budget expired before it began must stop at once, took {took:?}"
        );
        assert!(
            result.depth >= 1,
            "at least one iteration completes: {}",
            result.depth
        );
        assert!(legal.contains(&result.bestmove), "bestmove must be legal");
        assert!(
            result.elapsed_ms >= 300,
            "elapsed is measured from the parse instant: {} ms",
            result.elapsed_ms
        );
    }

    fn store_static_eval(searcher: &mut Searcher, board: &Board, static_eval: i32) {
        let mv = board
            .generate_legal_moves()
            .into_iter()
            .next()
            .expect("regression position has a legal move");
        searcher.tt.store(TtStore {
            key: board.hash,
            depth: 1,
            score: static_eval,
            bound: Bound::Exact,
            mv,
            ply: 0,
            static_eval,
            is_pv: false,
            kind: OutcomeKind::Full,
        });
    }

    #[test]
    fn lazy_margin_change_invalidates_local_and_shared_tt_evals() {
        const LOW_MARGIN: i32 = 200;
        const HIGH_MARGIN: i32 = 2_000;
        let board =
            Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).expect("valid lazy-eval regression FEN");
        let limits = SearchLimits::default();

        for shared in [false, true] {
            let mut searcher = Searcher::default();
            let mut options = EngineOptions::default();
            options.search_params.lazy_margin = LOW_MARGIN;
            searcher.reset_search_state(&limits, &options, board.side_to_move(), 0, true, false);
            let low_margin_score = searcher.raw_eval(&board);

            if shared {
                searcher.tt.make_shared(searcher.hash_mb);
            }
            store_static_eval(&mut searcher, &board, low_margin_score);
            assert_eq!(
                i32::from(
                    searcher
                        .tt
                        .probe(board.hash)
                        .expect("stored TT entry")
                        .static_eval
                ),
                low_margin_score
            );

            options.search_params.lazy_margin = HIGH_MARGIN;
            searcher.reset_search_state(&limits, &options, board.side_to_move(), 0, true, false);
            assert!(
                searcher.tt.probe(board.hash).is_none(),
                "LazyMargin change retained a {} TT evaluation",
                if shared { "shared" } else { "local" }
            );
            assert_ne!(searcher.raw_eval(&board), low_margin_score);
        }
    }

    #[test]
    fn helper_lazy_margin_sync_does_not_clear_live_shared_tt() {
        let board =
            Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).expect("valid lazy-eval regression FEN");
        let mut main = Searcher::default();
        main.tt.make_shared(main.hash_mb);
        store_static_eval(&mut main, &board, 123);

        let mut helper = Searcher::worker_default();
        helper.tt = main.tt.clone();
        let mut options = EngineOptions::default();
        options.search_params.lazy_margin = 2_000;
        helper.reset_search_state(
            &SearchLimits::default(),
            &options,
            board.side_to_move(),
            0,
            false,
            false,
        );

        assert!(
            main.tt.probe(board.hash).is_some(),
            "helper startup cleared the shared TT after another thread made it live"
        );
    }

    #[test]
    fn lmr_reduction_allows_strong_late_moves_to_reach_zero() {
        assert_eq!(lmr_reduction(1023, 8, 0), 0);
        assert_eq!(lmr_reduction(1024, 8, 0), 1);
        assert_eq!(lmr_reduction(4096, 3, 0), 3);
        assert_eq!(lmr_reduction(-1, 8, 0), 0);
        assert_eq!(lmr_reduction(1024, 0, 0), 0);
        // 4.8.1: with a floor of one ply the reduced search keeps a ply.
        // The 4096/3 case is exactly the one the audit counts: it consumed
        // the whole of new_depth and ran in quiescence.
        assert_eq!(lmr_reduction(4096, 3, 1), 2);
        assert_eq!(lmr_reduction(4096, 1, 1), 0);
        // new_depth already at or below the floor: leave it unreduced,
        // never extend it.
        assert_eq!(lmr_reduction(4096, 0, 1), 0);
    }

    #[test]
    fn quiescence_detects_mate_after_first_qply_check() {
        let mut searcher = Searcher::default();
        let mut board =
            Board::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
                .expect("valid fool's mate FEN");

        let score = searcher.quiescence(&mut board, -INF_SCORE, INF_SCORE, 0, 1, &mut || {
            SearchEvent::None
        });

        assert_eq!(score, -MATE_SCORE);
    }

    #[test]
    fn quiescence_stops_before_ply_stack_overflow() {
        let mut searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/8/8/3q4/8/8/4KQ2 w - - 0 1").expect("valid FEN");
        let before = board.to_fen();

        let score = searcher.quiescence(
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

    #[test]
    fn search_root_respects_restricted_root_moves() {
        let mut searcher = Searcher::default();
        let board = Board::default();
        let forced = board.parse_move("a2a3").expect("legal root move");
        let engine_options = EngineOptions::default();
        let limits = SearchLimits {
            depth: Some(1),
            ..SearchLimits::default()
        };
        searcher.reset_search_state(
            &limits,
            &engine_options,
            board.side_to_move(),
            0,
            true,
            true,
        );

        let result = searcher.search_root(board, &[forced], false, &mut || SearchEvent::None);

        assert_eq!(result.bestmove, forced);
    }

    #[test]
    fn root_move_record_keeps_iteration_and_distribution_state() {
        let mv = Move::from_uci("e2e4").expect("valid move");
        let mut root_move = RootMove::new(mv);

        root_move.record_search(1, 20, 100, 7, RootBound::Exact);
        root_move.complete_iteration();
        root_move.begin_iteration();
        root_move.record_search(2, 40, 250, 9, RootBound::Upper);
        root_move.record_search(2, -30, 50, 8, RootBound::Lower);
        root_move.complete_iteration();

        assert_eq!(root_move.mv, mv);
        assert_eq!(root_move.previous_score, 20);
        assert_eq!(root_move.score, -30);
        assert_eq!(root_move.samples, 2);
        assert!((root_move.average_score + 5.0).abs() < f64::EPSILON);
        assert!((root_move.mean_squared_score - 650.0).abs() < f64::EPSILON);
        assert_eq!(root_move.nodes, 400);
        assert_eq!(root_move.seldepth, 9);
        assert_eq!(root_move.fail_highs, 1);
        assert_eq!(root_move.fail_lows, 1);
        assert_eq!(root_move.pv_len, 1);
        assert_eq!(root_move.pv[0], mv);
    }

    #[test]
    fn search_populates_persistent_root_move_records() {
        let mut searcher = Searcher::default();
        let board = Board::default();
        let legal = board.generate_legal_moves();
        let limits = SearchLimits {
            depth: Some(3),
            ..SearchLimits::default()
        };
        searcher.reset_search_state(
            &limits,
            &EngineOptions::default(),
            board.side_to_move(),
            0,
            true,
            true,
        );

        let result = searcher.search_root(board, &legal, false, &mut || SearchEvent::None);

        assert_eq!(searcher.root_move_records.len(), legal.len());
        assert!(searcher.root_move_records.iter().all(|rm| {
            rm.samples >= 3
                && rm.previous_score > -INF_SCORE
                && rm.score > -INF_SCORE
                && rm.nodes > 0
                && rm.seldepth > 0
                && rm.pv[0] == rm.mv
                && rm.mean_squared_score + 1e-9 >= rm.average_score * rm.average_score
        }));
        let best = searcher
            .root_move_records
            .iter()
            .find(|rm| rm.mv == result.bestmove)
            .expect("best move has a persistent record");
        assert_eq!(best.last_best_depth, result.depth);
        assert!(best.pv_len > 1);
        assert!(
            searcher
                .root_move_records
                .iter()
                .map(|rm| rm.nodes)
                .sum::<u64>()
                <= result.nodes
        );
    }

    /// The single-legal-move shortcut must save clock time WITHOUT truncating
    /// an analysis search.
    ///
    /// Regression for 2026-07-23: on `1k3Q1r/pPpP2p1/P1P3P1/8/8/1p6/1P6/K6N b`
    /// the queen checks along the 8th rank and only `Rxf8` is legal (perft 1 =
    /// 1). `search_root` breaks after depth 2 on a single root move — correct
    /// under a clock, since that move gets played regardless of score, but it
    /// also fired for `go infinite`/`go ponder`, freezing a GUI's analysis at
    /// depth 2 on a meaningless score. Both halves are asserted: the shortcut
    /// still fires for a move request, and no longer fires in analysis mode.
    /// Verified to FAIL against the pre-fix condition.
    /// The 8.13 SMP machinery (pool root scores, stop voting, reduction
    /// jitter, pool-seeded aspiration) must be INERT at Threads=1.
    ///
    /// Every SMP feature gates on `shared_state`, which only a parallel
    /// search sets — this guards the property every 1-thread gate and the
    /// bench fingerprint rely on: a serial search must be deterministic and
    /// free of any pool machinery. If a gate ever leaks into the serial
    /// path, the run-to-run identity below breaks.
    #[test]
    fn smp_machinery_is_inert_on_a_single_thread() {
        let board = Board::default();
        let legal = board.generate_legal_moves();

        let run = || {
            let mut searcher = Searcher::default();
            let limits = SearchLimits {
                depth: Some(7),
                ..SearchLimits::default()
            };
            searcher.reset_search_state(
                &limits,
                &EngineOptions::default(),
                board.side_to_move(),
                0,
                true,
                true,
            );
            assert!(
                searcher.shared_state.is_none(),
                "a serial search must have no shared state"
            );
            let result =
                searcher.search_root(board.clone(), &legal, false, &mut || SearchEvent::None);
            (result.nodes, result.bestmove, result.score)
        };

        assert_eq!(
            run(),
            run(),
            "a serial search must be deterministic with all SMP gates closed"
        );
    }

    /// 4.9c: the skip pattern has to spread the pool across depths WITHOUT
    /// starving any helper. Three properties, each of which would be a real
    /// defect if violated.
    #[test]
    fn helper_skip_pattern_spreads_depth_without_starving_anyone() {
        // The main thread owns the reported result: it must run every
        // iteration whatever the pattern says.
        for depth in 1..=64 {
            assert!(!helper_skips_iteration(0, depth), "main skipped {depth}");
        }

        // No helper may skip everything, and none may skip nothing — either
        // would mean this thread contributes no depth diversity at all.
        for thread in 1..=32 {
            let searched = (1..=64)
                .filter(|d| !helper_skips_iteration(thread, *d))
                .count();
            assert!(
                (16..=48).contains(&searched),
                "thread {thread} searched {searched} of 64 iterations"
            );
        }

        // At any given depth the pool must DISAGREE — if every helper made the
        // same choice the mechanism would be a no-op with extra steps.
        for depth in 4..=32 {
            let skipping = (1..=16)
                .filter(|t| helper_skips_iteration(*t, depth))
                .count();
            assert!(
                (1..16).contains(&skipping),
                "at depth {depth}, {skipping} of 16 helpers skip — the pool is in lockstep"
            );
        }
    }

    /// A neutral snapshot to perturb one field at a time from.
    fn confidence_fixture() -> RootConfidence {
        RootConfidence {
            gap: 30,
            deviation: 30,
            effort: 0.9,
            fail_lows: 0,
            fail_highs: 0,
            instability: 0.5,
            pool_instability: None,
        }
    }

    /// Each retained term must move the scalar in the direction its name claims. A
    /// confidence model whose terms are wired backwards still compiles, still
    /// produces plausible numbers, and is worse than no model at all.
    ///
    #[test]
    fn root_confidence_scalar_moves_with_each_of_its_terms() {
        let params = SearchParams::default();
        let base = confidence_fixture().scalar(&params);

        let noisier = RootConfidence {
            deviation: 300,
            ..confidence_fixture()
        };
        let idler = RootConfidence {
            effort: 0.5,
            ..confidence_fixture()
        };
        let re_searched = RootConfidence {
            fail_highs: 2,
            ..confidence_fixture()
        };

        assert!(noisier.scalar(&params) < base, "a noisier score is weaker");
        assert!(idler.scalar(&params) < base, "less effort is weaker");
        assert!(
            re_searched.scalar(&params) < base,
            "aspiration re-searches are weaker"
        );
        assert!((0..=1000).contains(&base));
    }

    /// Root gap remains diagnostic-only. RAR-S47 found it exactly zero on 82.3%
    /// of completed bench iterations because rival root moves usually carry
    /// null-window bounds, not comparable values. Keep that structurally out of
    /// the scalar unless a future real-value root search creates new evidence.
    #[test]
    fn root_gap_is_diagnostic_only() {
        let params = SearchParams::default();

        assert_eq!(
            confidence_fixture().scalar(&params),
            RootConfidence {
                gap: 30_000,
                ..confidence_fixture()
            }
            .scalar(&params),
            "a gap must not move the shipped scalar at all"
        );
    }

    /// With every weight fitted to zero the model has no inputs, so it must
    /// report "unknown" rather than "no confidence" — the two would scale the
    /// clock in opposite directions.
    #[test]
    fn root_confidence_without_weights_is_neutral_not_zero() {
        let params = SearchParams {
            root_conf_w_deviation: 0,
            root_conf_w_effort: 0,
            root_conf_w_window: 0,
            ..SearchParams::default()
        };

        assert_eq!(confidence_fixture().scalar(&params), NEUTRAL_CONFIDENCE);
    }

    /// PLAN 4.7's hard rule: pooled worker instability may reach TIME and
    /// nothing else.
    ///
    /// Enforced structurally rather than by convention — `scalar` has no
    /// instability term, and `aspiration_deltas` is written on top of `scalar`
    /// — so this pins the property that keeps that true: two snapshots that
    /// differ ONLY in the pooled value must be indistinguishable to the window
    /// and distinguishable to the clock.
    ///
    /// Result ownership needs no case here because it cannot express the
    /// question: `select_parallel_result` chooses between `SearchResult`
    /// values, which carry no confidence at all.
    #[test]
    fn pooled_instability_reaches_the_clock_and_nothing_else() {
        let params = SearchParams {
            root_conf_pool_instability: 1,
            ..SearchParams::default()
        };
        let own = confidence_fixture();
        let pooled = RootConfidence {
            pool_instability: Some(4.0),
            ..confidence_fixture()
        };

        assert_eq!(own.scalar(&params), pooled.scalar(&params));
        assert_eq!(
            own.aspiration_deltas(21, &params),
            pooled.aspiration_deltas(21, &params),
            "pooled helper instability must not reach the aspiration window"
        );
        assert!(
            pooled.time_factor(&params) > own.time_factor(&params),
            "pooled helper instability must reach the clock"
        );
    }

    /// The window must widen on the side that was WRONG, stay bounded whatever
    /// the fail counts do, and narrow as confidence rises.
    #[test]
    fn root_confidence_aspiration_is_asymmetric_and_bounded() {
        let params = SearchParams::default();
        let failed_high = RootConfidence {
            fail_highs: 2,
            ..confidence_fixture()
        };
        let (alpha, beta) = failed_high.aspiration_deltas(21, &params);
        assert!(beta > alpha, "the failing side is the side that opens");

        let runaway = RootConfidence {
            fail_lows: 1_000,
            fail_highs: 1_000,
            ..confidence_fixture()
        };
        let (alpha, beta) = runaway.aspiration_deltas(INF_SCORE, &params);
        assert!((1..=INF_SCORE).contains(&alpha) && (1..=INF_SCORE).contains(&beta));

        let confident = RootConfidence {
            gap: 4_000,
            deviation: 0,
            effort: 1.0,
            ..confidence_fixture()
        };
        let unsure = RootConfidence {
            gap: 0,
            deviation: 4_000,
            effort: 0.0,
            ..confidence_fixture()
        };
        assert!(
            confident.aspiration_deltas(21, &params).0 < unsure.aspiration_deltas(21, &params).0,
            "confidence must narrow the window"
        );
    }

    /// Both arms must be INERT at their defaults and WIRED when enabled.
    ///
    /// The second half is the half that matters: 4.6c landed a switch that
    /// measured 0.00% node change both off AND on, which is indistinguishable
    /// from dead code. `RootConfAspiration` changes the tree, so `bench` can see
    /// it. `RootConfTime` cannot change a depth-limited search by construction —
    /// the soft target never binds — so what is asserted here is exactly that
    /// inertness, and its sizing is the TM shadow in the diagnostics.
    #[test]
    fn root_confidence_arms_are_inert_off_and_aspiration_is_wired_on() {
        let board = Board::default();
        let legal = board.generate_legal_moves();

        let run = |params: SearchParams| {
            let mut searcher = Searcher::default();
            let limits = SearchLimits {
                depth: Some(8),
                ..SearchLimits::default()
            };
            searcher.reset_search_state(
                &limits,
                &EngineOptions::default(),
                board.side_to_move(),
                0,
                true,
                true,
            );
            searcher.params = params;
            let result =
                searcher.search_root(board.clone(), &legal, false, &mut || SearchEvent::None);
            (result.nodes, result.bestmove)
        };

        let baseline = run(SearchParams::default());
        assert_eq!(
            baseline,
            run(SearchParams::default()),
            "the snapshot itself must not perturb a search"
        );
        assert_eq!(
            baseline,
            run(SearchParams {
                root_conf_time: 1,
                ..SearchParams::default()
            }),
            "a depth-limited search never binds the soft target, on or off"
        );
        assert_ne!(
            baseline.0,
            run(SearchParams {
                root_conf_aspiration: 1,
                ..SearchParams::default()
            })
            .0,
            "RootConfAspiration must reach the tree when enabled"
        );
    }

    #[test]
    fn single_legal_move_shortcut_skips_analysis_but_not_move_requests() {
        const FORCED: &str = "1k3Q1r/pPpP2p1/P1P3P1/8/8/1p6/1P6/K6N b - - 0 1";
        let board = Board::from_fen(FORCED).expect("valid fen");
        let legal = board.generate_legal_moves();
        assert_eq!(legal.len(), 1, "position must have exactly one legal move");

        let run = |infinite: bool| {
            let mut searcher = Searcher::default();
            let limits = SearchLimits {
                // Generous depth cap either way, so the STOP REASON under test
                // is the shortcut rather than the depth limit.
                depth: Some(8),
                infinite,
                ..SearchLimits::default()
            };
            searcher.reset_search_state(
                &limits,
                &EngineOptions::default(),
                board.side_to_move(),
                0,
                true,
                true,
            );
            searcher
                .search_root(board.clone(), &legal, false, &mut || SearchEvent::None)
                .depth
        };

        // Move request: stops at the shortcut, well short of the depth cap.
        let move_request_depth = run(false);
        assert_eq!(
            move_request_depth, 2,
            "clock/move-request search should stop at the single-move shortcut"
        );

        // Analysis: must keep going and honour the depth request instead.
        let analysis_depth = run(true);
        assert!(
            analysis_depth > move_request_depth,
            "analysis search must not be truncated by the single-move shortcut              (got depth {analysis_depth}, move-request depth {move_request_depth})"
        );
    }

    #[test]
    fn ponderhit_preserves_elapsed_time_budget() {
        let mut searcher = Searcher {
            nodes: 2047,
            pondering: true,
            start: Instant::now() - Duration::from_millis(10),
            limits: RuntimeLimits {
                depth: 64,
                nodes: 0,
                optimum_ms: 1.0,
                maximum_ms: 1.0,
                movetime_mode: false,
                analysis_mode: false,
            },
            ..Searcher::default()
        };

        let stopped = searcher.check_stop(&mut || SearchEvent::PonderHit);

        assert!(stopped);
        assert!(searcher.ponderhit);
        assert!(!searcher.pondering);
    }

    #[test]
    fn malformed_tt_move_is_not_searched_or_reported_in_pv() {
        let root = Board::from_fen("2k5/pp3pp1/5n2/2P5/bPP2P2/P3K3/6Pp/3Q1B1R w - - 0 23")
            .expect("valid tournament-derived FEN");
        let illegal = Move::from_uci("e3f4").expect("valid UCI move shape");
        let mut board = root.clone();
        let before_fen = board.to_fen();
        let before_hash = board.hash;
        let mut searcher = Searcher::default();
        let engine_options = EngineOptions::default();
        let limits = SearchLimits {
            depth: Some(3),
            ..SearchLimits::default()
        };
        searcher.reset_search_state(
            &limits,
            &engine_options,
            board.side_to_move(),
            0,
            true,
            true,
        );
        searcher.tt.store(TtStore {
            key: board.hash,
            depth: 8,
            score: 0,
            bound: Bound::Exact,
            mv: illegal,
            ply: 0,
            static_eval: VALUE_NONE,
            is_pv: false,
            kind: OutcomeKind::Full,
        });

        let _ = searcher.negamax(
            &mut board,
            3,
            -INF_SCORE,
            INF_SCORE,
            0,
            true,
            true,
            Move::NULL,
            false,
            &mut || SearchEvent::None,
        );

        assert_eq!(board.to_fen(), before_fen);
        assert_eq!(board.hash, before_hash);
        assert!(
            !searcher.pv_table[0][..searcher.pv_len[0].min(MAX_PLY)]
                .iter()
                .any(|&mv| mv.same_uci_move(illegal)),
            "malformed TT move must not appear in the root PV"
        );
        assert_legal_pv(
            &root,
            &searcher.pv_table[0][..searcher.pv_len[0].min(MAX_PLY)],
        );
    }

    #[test]
    fn parallel_result_selection_uses_weighted_helper_votes() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let g1f3 = Move::from_uci("g1f3").expect("valid move");
        let results = vec![
            test_search_result(e2e4, 20, 5),
            test_search_result(d2d4, 18, 5),
            test_search_result(d2d4, 16, 5),
        ];

        let selected =
            select_parallel_result(&results, &[e2e4, d2d4, g1f3]).expect("selected result");

        assert_eq!(selected.bestmove, d2d4);
    }

    #[test]
    fn parallel_result_selection_prefers_decisive_win() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let results = vec![
            test_search_result(e2e4, 900, 12),
            test_search_result(d2d4, TB_WIN_SCORE, 4),
        ];

        let selected = select_parallel_result(&results, &[e2e4, d2d4]).expect("selected result");

        assert_eq!(selected.bestmove, d2d4);
    }

    #[test]
    fn quiet_direct_checks_receive_ordering_bonus() {
        let searcher = Searcher::default();
        let board = Board::from_fen("4k3/8/8/8/8/8/8/R6K w - - 0 1").expect("valid FEN");
        let checking = board.parse_move("a1e1").expect("legal checking move");
        let quiet = board.parse_move("a1a2").expect("legal quiet move");
        assert!(board.gives_check(checking));
        assert!(!board.gives_check(quiet));

        let mut scored = searcher.score_moves(&board, &[checking, quiet], Move::NULL, 0);
        let moves = scored.as_mut_slice();
        let checking_score = moves
            .iter()
            .find(|scored| scored.mv == checking)
            .expect("checking move scored")
            .score;
        let quiet_score = moves
            .iter()
            .find(|scored| scored.mv == quiet)
            .expect("quiet move scored")
            .score;

        // 4.6c: read the live parameter, not a duplicate constant, so this
        // test cannot drift from the value ordering actually uses.
        let bonus = crate::params::SearchParams::default().check_bonus_safe;
        assert!(checking_score >= quiet_score + bonus);
    }

    #[test]
    fn quiet_history_uses_low_ply_slots_through_ply_seven() {
        let mut searcher = Searcher::default();
        let board = Board::default();
        let mv = board.parse_move("a2a3").expect("legal quiet move");
        let from = Square::A2.index();
        let to = Square::A3.index();

        searcher.low_ply_history[7][from][to] = 800;

        let ci = board.check_info();
        let ctx7 = searcher.quiet_history_ctx(&board, 7);
        assert_eq!(
            searcher.quiet_history_score(&board, &ci, &ctx7, Color::White, mv, 7),
            100
        );
        let ctx8 = searcher.quiet_history_ctx(&board, 8);
        assert_eq!(
            searcher.quiet_history_score(&board, &ci, &ctx8, Color::White, mv, 8),
            0
        );
    }

    #[test]
    fn quiet_history_updates_only_configured_low_ply_window() {
        let mut searcher = Searcher::default();
        let board = Board::default();
        let in_window = board.parse_move("a2a3").expect("legal quiet move");
        let outside_window = board.parse_move("h2h3").expect("legal quiet move");

        searcher.update_quiet_history(
            Color::White,
            in_window,
            Piece::Pawn,
            board.pawn_key(),
            LOW_PLY_HISTORY_SIZE - 1,
            400,
        );
        searcher.update_quiet_history(
            Color::White,
            outside_window,
            Piece::Pawn,
            board.pawn_key(),
            LOW_PLY_HISTORY_SIZE,
            400,
        );

        assert!(
            searcher.low_ply_history[LOW_PLY_HISTORY_SIZE - 1][Square::A2.index()]
                [Square::A3.index()]
                > 0
        );
        assert_eq!(
            searcher.low_ply_history[LOW_PLY_HISTORY_SIZE - 1][Square::H2.index()]
                [Square::H3.index()],
            0
        );
    }

    #[test]
    fn staged_picker_emits_valid_quiet_tt_move_first() {
        let searcher = Searcher::default();
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                .expect("valid FEN");
        let tt_move = board
            .legal_move(Move::from_uci("g1f3").expect("valid UCI move shape"))
            .expect("quiet TT move must be legal");

        let mut picker = MovePicker::staged(&searcher, &mut board, tt_move, 0);
        let picked = picker.next(&searcher, &mut board).expect("first move");

        assert_eq!(picked.mv, tt_move);
        assert!(!picked.mv.is_capture());
    }

    #[test]
    fn ponder_move_can_be_recovered_from_tt_child() {
        let mut searcher = Searcher::default();
        let root = Board::default();
        let bestmove = root.parse_move("a2a3").expect("legal root move");
        let mut child = root.clone();
        child.make_move_unchecked(bestmove);
        let ponder = child.parse_move("a7a6").expect("legal child move");
        searcher.tt.store(TtStore {
            key: child.hash,
            depth: 4,
            score: 0,
            bound: Bound::Exact,
            mv: ponder,
            ply: 1,
            static_eval: VALUE_NONE,
            is_pv: false,
            kind: OutcomeKind::Full,
        });

        assert_eq!(searcher.ponder_from_tt(&root, bestmove), ponder);
    }

    #[test]
    fn staged_picker_delays_bad_captures_until_after_quiets() {
        let searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/4p3/3p4/8/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let losing_capture = board
            .parse_move("c3d5")
            .expect("knight capture must be legal");
        assert!(losing_capture.is_capture());
        assert!(!board.see_ge(losing_capture, 0));

        let mut picker = MovePicker::staged(&searcher, &mut board, Move::NULL, 0);
        let mut quiet_seen = false;
        let mut losing_capture_seen = false;

        while let Some(picked) = picker.next(&searcher, &mut board) {
            if picked.mv == losing_capture {
                assert!(
                    quiet_seen,
                    "losing captures should be staged after quiet moves"
                );
                losing_capture_seen = true;
                break;
            }
            if board.is_quiet_move(picked.mv) {
                quiet_seen = true;
            }
        }

        assert!(
            quiet_seen,
            "test position must have at least one quiet move"
        );
        assert!(
            losing_capture_seen,
            "test position must include the losing capture"
        );
    }

    /// Collect everything a picker yields, for the contract tests below.
    fn drain_picker(picker: &mut MovePicker, searcher: &Searcher, board: &mut Board) -> Vec<Move> {
        let mut out = Vec::new();
        while let Some(picked) = picker.next(searcher, board) {
            out.push(picked.mv);
        }
        out
    }

    /// 4.5.2 CONTRACT — staged path: every legal move, exactly once.
    ///
    /// This is the legality and duplicate guarantee in one assertion. It holds
    /// with a TT move set, which is the case that can double-emit: the TT move
    /// goes out in `Stage::TtMove` and every later stage must filter it.
    #[test]
    fn staged_picker_emits_every_legal_move_exactly_once() {
        let searcher = Searcher::default();
        for fen in [
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "4k3/8/4p3/3p4/8/2N5/8/4K3 w - - 0 1",
        ] {
            let mut board = Board::from_fen(fen).expect("valid FEN");
            let mut legal: Vec<Move> = board.generate_legal_movelist().as_slice().to_vec();
            for tt in [Move::NULL, legal[0], legal[legal.len() - 1]] {
                let mut picker = MovePicker::staged(&searcher, &mut board, tt, 0);
                let mut got = drain_picker(&mut picker, &searcher, &mut board);
                got.sort_unstable_by_key(|m| m.0);
                legal.sort_unstable_by_key(|m| m.0);
                assert_eq!(
                    got, legal,
                    "staged picker must emit every legal move exactly once                      (fen {fen}, tt {tt})"
                );
            }
        }
    }

    /// 4.5.2 CONTRACT — full path (root and in-check): same guarantee.
    #[test]
    fn full_picker_emits_every_legal_move_exactly_once() {
        let searcher = Searcher::default();
        // Second FEN is a check position — rook on e2 checking Ke1, with
        // Kxe2/Kd1/Kf1 legal — which is exactly when the search takes the full
        // path. Not a mate: a mated position has no legal moves and would test
        // nothing here.
        for fen in [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1",
        ] {
            let mut board = Board::from_fen(fen).expect("valid FEN");
            let mut legal: Vec<Move> = board.generate_legal_movelist().as_slice().to_vec();
            assert!(!legal.is_empty(), "test FEN must have legal moves: {fen}");
            for tt in [Move::NULL, legal[0]] {
                let scored = searcher.score_moves(&board, legal.as_slice(), tt, 0);
                let mut picker = MovePicker::full(scored, tt);
                let mut got = drain_picker(&mut picker, &searcher, &mut board);
                got.sort_unstable_by_key(|m| m.0);
                legal.sort_unstable_by_key(|m| m.0);
                assert_eq!(
                    got, legal,
                    "full picker must emit every legal move exactly once                      (fen {fen}, tt {tt})"
                );
            }
        }
    }

    /// 4.5.2 CONTRACT — `Stage::Done` is terminal and stays terminal.
    #[test]
    fn picker_is_exhausted_permanently() {
        let searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");
        let mut picker = MovePicker::staged(&searcher, &mut board, Move::NULL, 0);
        while picker.next(&searcher, &mut board).is_some() {}
        for _ in 0..3 {
            assert!(
                picker.next(&searcher, &mut board).is_none(),
                "an exhausted picker must keep returning None"
            );
        }
    }

    fn test_search_result(bestmove: Move, score: i32, depth: usize) -> SearchResult {
        SearchResult {
            bestmove,
            pondermove: Move::NULL,
            score,
            depth,
            nodes: 0,
            tb_hits: 0,
            elapsed_ms: 0,
            exit: SearchExit::Stop,
            ponderhit: false,
        }
    }

    fn assert_legal_pv(root: &Board, pv: &[Move]) {
        let mut board = root.clone();
        for &mv in pv {
            if mv.is_null() {
                break;
            }
            let legal = board
                .parse_move(&mv.to_string())
                .unwrap_or_else(|| panic!("PV move {mv} is illegal in {}", board.to_fen()));
            board.make_move_unchecked(legal);
        }
    }
}
