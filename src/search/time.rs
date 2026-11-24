//! Clock management: per-search limits, the between-iteration soft-stop
//! factors and the proved-mate stop.

use crate::board::{Color, Move};
use crate::score::{MATE_BOUND, MATE_SCORE};
use crate::search::options::{EngineOptions, SearchLimits};

use super::params::SearchParams;

#[derive(Copy, Clone)]
pub(super) struct RuntimeLimits {
    pub depth: usize,
    pub nodes: u64,
    /// Soft limit: between-iteration stop threshold (clock mode).
    /// In movetime mode this equals `maximum_ms`; the between-iteration
    /// soft-stop logic is skipped entirely for movetime — only `count_node`
    /// (every 2048 nodes) fires at `maximum_ms`.
    pub(super) optimum_ms: f64,
    /// Hard limit: mid-iteration abort threshold.
    pub(super) maximum_ms: f64,
    /// True when the `go movetime T` command was used.
    pub(super) movetime_mode: bool,
    /// The single-legal-move shortcut may end the search: a move request
    /// over the whole root. Not for `go infinite` or `go ponder`, where the
    /// caller wants an evaluation, not a move, and not under `searchmoves`,
    /// where one move left is the caller's choice of line, not a forced move.
    pub(super) forced_move_shortcut: bool,
    /// `go mate N` as a distance: a mate within this many plies, for either
    /// side, ends the search.
    pub(super) mate_plies: Option<i32>,
    /// The search may end on a proved mate ([`MateWatch`]): a clock search
    /// that asked for a move and nothing else. Analysis, ponder, movetime,
    /// depth, node and `searchmoves` requests run to their own limit.
    pub(super) mate_stop: bool,
    /// The tablebase PV extension's time box in milliseconds, shared by the
    /// reported lines; `None` without a clock, where it runs unbounded.
    pub(super) tb_extension_ms: Option<f64>,
    /// The latest elapsed time at which the extension may still start: the
    /// clock's hard ceiling less [`TB_EXTENSION_SLACK_OVERHEADS`] overheads.
    /// Infinite where no clock is known.
    pub(super) tb_extension_latest_start_ms: f64,
}

/// What the tablebase PV extension may spend once the search has ended.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum TbExtension {
    /// The clock is too short to risk a table read.
    Skip,
    /// This many milliseconds, shared by the reported lines.
    Boxed(f64),
    /// No clock: it runs to the end of the line.
    Unbounded,
}

/// Clock the extension must leave before the hard ceiling, in move overheads.
/// The box is checked between table probes, and one probe is a blocking read
/// no box can bound: a page the cache misses held one for 54 ms in a game and
/// 44 ms in a replay under load. Ten overheads, 100 ms at the default, is
/// about twice that, on top of the reserve the ceiling already keeps.
const TB_EXTENSION_SLACK_OVERHEADS: f64 = 10.0;

impl RuntimeLimits {
    /// The extension's allowance when the search ended `elapsed_ms` into the
    /// move, measured as the hard limit measures it.
    pub(super) fn tb_extension(&self, elapsed_ms: f64) -> TbExtension {
        match self.tb_extension_ms {
            None => TbExtension::Unbounded,
            Some(box_ms) if box_ms > 0.0 && elapsed_ms <= self.tb_extension_latest_start_ms => {
                TbExtension::Boxed(box_ms)
            }
            Some(_) => TbExtension::Skip,
        }
    }
}

impl Default for RuntimeLimits {
    /// No limit: full depth, no node cap, no clock.
    fn default() -> Self {
        Self {
            depth: super::MAX_DEPTH,
            nodes: 0,
            optimum_ms: f64::INFINITY,
            maximum_ms: f64::INFINITY,
            movetime_mode: false,
            forced_move_shortcut: true,
            mate_plies: None,
            mate_stop: false,
            tb_extension_ms: None,
            tb_extension_latest_start_ms: f64::INFINITY,
        }
    }
}

/// What both bounds leave for the search to stop and answer, as the donor
/// leaves it.
const TIME_OVERHEAD_MS: f64 = 15.0;

/// The budget's shape is the donor's, tuned with its search in a long-control
/// session and a six-thread one. It is not a fit coordinate: a fit at a
/// shorter control would move the long-control allocation without reading it. In sudden
/// death the soft bound's share of the clock grows with the move number from
/// about 1.2% towards [`SOFT_SCALE_LATE`].
const SOFT_SCALE_LATE: f64 = 0.0594;
const SOFT_SCALE_EARLY_CUT: f64 = 0.0492;
const SOFT_SCALE_RATE: f64 = 0.0386;
const HARD_SCALE: f64 = 0.7281;
/// The share of the increment both bounds add.
const INCREMENT_SHARE: f64 = 0.75;
/// The hard bound's multiple of the per-move base under `movestogo`.
const CYCLIC_HARD_SCALE: f64 = 5.0;

/// The soft and hard bounds in milliseconds for a clock of `time` with
/// `increment`, before Rarog's reserve: the donor's budget. `movestogo` 0 is
/// sudden death; otherwise each of the moves to go gets the main clock's
/// share. Both bounds stay within the main clock, the clock less Move
/// Overhead, and are at least a millisecond.
fn budget(time: f64, increment: f64, movestogo: usize, overhead: f64, fullmove: f64) -> (f64, f64) {
    let main = (time - overhead).max(0.0);
    let (soft, hard) = if movestogo > 0 {
        let base = main / movestogo as f64 + INCREMENT_SHARE * increment;
        (
            base.min(main + increment),
            (CYCLIC_HARD_SCALE * base).min(main + increment),
        )
    } else {
        let soft_scale =
            SOFT_SCALE_LATE - SOFT_SCALE_EARLY_CUT * (-SOFT_SCALE_RATE * fullmove).exp();
        (
            (soft_scale * main + INCREMENT_SHARE * increment).min(main),
            (HARD_SCALE * main + INCREMENT_SHARE * increment).min(main),
        )
    };
    (
        (soft - TIME_OVERHEAD_MS).max(1.0),
        (hard - TIME_OVERHEAD_MS).max(1.0),
    )
}

/// Compute time limits for one search.
///
/// `game_ply` is the number of half-moves played so far in the game
/// (`2*(fullmove - 1) + (side_to_move == Black) as u32`); the budget reads
/// the full-move number it gives.
pub(super) fn compute_runtime_limits(
    options: &SearchLimits,
    engine_options: &EngineOptions,
    side_to_move: Color,
    game_ply: u32,
    max_depth: usize,
) -> RuntimeLimits {
    // `None` = no depth limit.
    let depth = options
        .depth
        .map_or(max_depth, |d| (d as usize).clamp(1, max_depth));
    let mut optimum_ms = f64::INFINITY;
    let mut maximum_ms = f64::INFINITY;
    let mut movetime_mode = false;
    let analysis_mode = options.infinite || options.ponder;
    let mut mate_stop = false;
    // The extension spends at most half of Move Overhead, as the donor does.
    let mut tb_extension_ms = None;
    let mut tb_extension_latest_start_ms = f64::INFINITY;

    if options.move_time > 0 {
        // Fixed movetime: use the full budget as the hard limit (the
        // SF/Reckless default). Time forfeits come from the *clock* path,
        // which the `2*overhead` reserve in the else-branch covers; movetime
        // never forfeited (`t=0` over a full 100 ms/move gauntlet), and a
        // reserve here cost ~10 % of thinking time. Stockfish measured
        // `tpm=110.2` with `t=0` at 100 ms/move, so the harness tolerates ~10 %
        // past the nominal time.
        // count_node (every 2048 nodes) aborts within ~1 ms of `maximum_ms`
        // and the pre/post latency is only ~3 ms, so the full budget lands
        // ~3 % over nominal — comfortably inside that tolerance.
        let movetime = (options.move_time as f64).max(1.0);
        optimum_ms = movetime;
        maximum_ms = movetime;
        movetime_mode = true;
        tb_extension_ms = Some(engine_options.move_overhead / 2.0);
    } else {
        let (time, increment) = match side_to_move {
            Color::White => (options.white_time, options.white_increment),
            Color::Black => (options.black_time, options.black_increment),
        };
        if let Some(time) = time {
            let overhead = engine_options.move_overhead;
            let fullmove = f64::from(game_ply / 2 + 1);
            let (soft, hard) = budget(
                time as f64,
                increment as f64,
                options.movestogo,
                overhead,
                fullmove,
            );

            // Rarog's reserve on top of the donor's bounds. The wall time the
            // GUI charges includes the latency before the search's clock
            // starts and after `bestmove` leaves, which spikes under a loaded
            // match, so the hard bound never passes `time - 2*overhead`. With
            // the donor's own 15 ms off a bound that is already below the main
            // clock this binds at one thread only for an overhead above 15 ms.
            // The reserve also covers scheduler starvation at several threads:
            // in Threads=4 games the search's 2048-node poll stretched to
            // 50-100 ms under contention and overran the hard bound by up to
            // about 34 ms, a time forfeit in about 1.3% of games with the 20 ms
            // reserve alone, against none in 3,460 at one thread. So
            // multi-threaded searches keep 30 ms more.
            let smp_reserve = if engine_options.threads > 1 {
                30.0
            } else {
                0.0
            };
            let min_reserve = 2.0 * overhead + smp_reserve;
            let hard_ceiling = (time as f64 - min_reserve).max(1.0);
            tb_extension_ms = Some(overhead / 2.0);
            tb_extension_latest_start_ms = hard_ceiling - TB_EXTENSION_SLACK_OVERHEADS * overhead;
            maximum_ms = hard.min(hard_ceiling);
            optimum_ms = soft.min(maximum_ms);
            mate_stop = !analysis_mode
                && options.depth.is_none()
                && options.mate.is_none()
                && options.nodes == 0
                && options.search_moves.is_empty();
        }
    }

    RuntimeLimits {
        depth,
        nodes: options.nodes,
        optimum_ms,
        maximum_ms,
        movetime_mode,
        forced_move_shortcut: !analysis_mode
            && options.search_moves.is_empty()
            && options.mate.is_none(),
        mate_plies: options
            .mate
            .map(|moves| i32::try_from(moves).unwrap_or(i32::MAX).saturating_mul(2)),
        mate_stop,
        tb_extension_ms,
        tb_extension_latest_start_ms,
    }
}

/// Completed iterations a root mate must hold, with the same move and
/// distance, before a clock search ends on it.
const MATE_STOP_ITERATIONS: u32 = 3;

/// Watches the completed iterations of a clock search for a proved result
/// with nothing left to find: a mate for the side to move that has held its
/// move and distance for [`MATE_STOP_ITERATIONS`] iterations, or a loss in at
/// most two plies. Without it a found mate searched on until a deeper
/// iteration grew to the ply cap and the move ended at the hard maximum.
#[derive(Default)]
pub(super) struct MateWatch {
    line: Option<(Move, i32)>,
    held: u32,
}

impl MateWatch {
    /// Record one completed iteration; true when the search should end.
    pub(super) fn completed(&mut self, mv: Move, score: i32) -> bool {
        if score <= -MATE_SCORE + 2 {
            return true;
        }
        if score < MATE_BOUND {
            *self = Self::default();
            return false;
        }
        if self.line == Some((mv, score)) {
            self.held += 1;
        } else {
            self.line = Some((mv, score));
            self.held = 1;
        }
        self.held >= MATE_STOP_ITERATIONS
    }
}

/// The between-iteration soft stop as the last completed iteration left it:
/// each of the donor's five factors on the soft bound, and the target they
/// make.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(
    not(feature = "diag"),
    allow(
        dead_code,
        reason = "the factors are read by the diag log and the tests; the search reads the target"
    )
)]
pub(super) struct SoftStop {
    pub(super) nodes: f64,
    pub(super) score_trend: f64,
    pub(super) pv_stability: f64,
    pub(super) eval_stability: f64,
    pub(super) best_move_stability: f64,
    pub(super) target_ms: f64,
}

/// What the soft stop reads from the completed iterations.
#[derive(Copy, Clone, Debug)]
pub(super) struct IterationSignals {
    /// The best move's share of this thread's nodes in the search so far.
    pub(super) node_share: f64,
    /// The previous search's root score in this game less this iteration's:
    /// positive when the score fell.
    pub(super) score_fall: f64,
    /// Completed iterations in a row with the same best move.
    pub(super) pv_stability: i32,
    /// Completed iterations in a row whose score held near the running
    /// average (the root window's count).
    pub(super) eval_stability: i32,
    /// Moves other than the first that raised alpha on line one in this
    /// iteration.
    pub(super) best_move_changes: u32,
}

/// A clock multiplier stored in ten-thousandths, as `SearchParams` holds it.
fn ratio(value: i32) -> f64 {
    f64::from(value) / 10_000.0
}

/// `value` clamped to the two ends in either order, so a fit that crosses
/// them cannot panic `f64::clamp`.
fn clamp_between(value: f64, a: f64, b: f64) -> f64 {
    value.clamp(a.min(b), a.max(b))
}

/// The soft target: the soft bound scaled by the donor's five factors,
/// capped at the hard bound. Each factor reads one signal:
/// - the best move's node share: more time while the search is spread over
///   several moves, less once one move takes nearly all of it;
/// - the score's fall from the previous search: more time when it fell;
/// - the best move's stability and the score's: less time the longer each
///   has held, never below its floor;
/// - this iteration's best-move changes: more time, growing as their log.
pub(super) fn soft_stop(
    params: &SearchParams,
    limits: &RuntimeLimits,
    signals: IterationSignals,
) -> SoftStop {
    let nodes = (ratio(params.tm_nodes_base) - ratio(params.tm_nodes_slope) * signals.node_share)
        .max(ratio(params.tm_nodes_floor));
    let score_trend = clamp_between(
        ratio(params.tm_trend_base) + ratio(params.tm_trend_slope) * signals.score_fall,
        ratio(params.tm_trend_low),
        ratio(params.tm_trend_high),
    );
    let pv_stability = (ratio(params.tm_pv_base)
        - ratio(params.tm_pv_slope) * f64::from(signals.pv_stability))
    .max(ratio(params.tm_pv_floor));
    let eval_stability = (ratio(params.tm_eval_base)
        - ratio(params.tm_eval_slope) * f64::from(signals.eval_stability))
    .max(ratio(params.tm_eval_floor));
    let best_move_stability = ratio(params.tm_changes_base)
        + (ratio(params.tm_changes_scale) * f64::from(signals.best_move_changes)).ln_1p();
    let target_ms = limits.optimum_ms
        * nodes
        * score_trend
        * pv_stability
        * eval_stability
        * best_move_stability;
    SoftStop {
        nodes,
        score_trend,
        pv_stability,
        eval_stability,
        best_move_stability,
        target_ms: target_ms.min(limits.maximum_ms),
    }
}

#[cfg(test)]
mod tests;
