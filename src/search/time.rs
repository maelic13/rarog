//! Clock management: per-search limits, the between-iteration soft-stop
//! factors and the proved-mate stop.

use crate::board::{Color, Move};
use crate::eval::MATE_SCORE;
use crate::infra;
use crate::search::options::{EngineOptions, SearchLimits};

use super::MAX_PLY;
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

/// The budget's shape is the donor's, tuned with its search at `40+0.4` and
/// at six threads. It is not a fit coordinate: a fit at a shorter control
/// would move the long-control allocation without reading it. In sudden
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
        if score < MATE_SCORE - infra::to_i32(MAX_PLY) {
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
mod tests {
    use super::*;

    fn limits(game_ply: u32, options: &SearchLimits) -> RuntimeLimits {
        compute_runtime_limits(
            options,
            &EngineOptions::default(),
            Color::White,
            game_ply,
            64,
        )
    }

    /// The clock coordinates' seeds, the values these tests'
    /// expectations are written in, so a fit that moves the defaults moves
    /// nothing here.
    fn seeds() -> SearchParams {
        SearchParams {
            tm_nodes_base: 31_838,
            tm_nodes_slope: 26_554,
            tm_nodes_floor: 5_460,
            tm_trend_base: 7_426,
            tm_trend_slope: 1_189,
            tm_trend_low: 7_214,
            tm_trend_high: 14_031,
            tm_pv_base: 12_881,
            tm_pv_slope: 440,
            tm_pv_floor: 7_160,
            tm_eval_base: 12_664,
            tm_eval_slope: 416,
            tm_eval_floor: 8_642,
            tm_changes_base: 11_500,
            tm_changes_scale: 2_526,
            ..SearchParams::default()
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// The donor's sudden-death bounds at `fullmove`, before the reserve,
    /// written out from its formula: 0.0594 - 0.0492 * e^(-0.0386 * fullmove)
    /// of the main clock plus three quarters of the increment, 0.7281 of it
    /// plus the same, each within the main clock, less 15 ms.
    fn donor_bounds(time: f64, increment: f64, overhead: f64, fullmove: f64) -> (f64, f64) {
        let main = time - overhead;
        let scale = 0.0594 - 0.0492 * (-0.0386 * fullmove).exp();
        (
            (scale * main + 0.75 * increment).min(main) - 15.0,
            (0.7281 * main + 0.75 * increment).min(main) - 15.0,
        )
    }

    #[test]
    fn a_sudden_death_clock_gets_the_donors_bounds() {
        let clock = |time: usize, increment: usize, game_ply: u32| {
            limits(
                game_ply,
                &SearchLimits {
                    white_time: Some(time),
                    white_increment: increment,
                    ..SearchLimits::default()
                },
            )
        };
        // `3+0.03` at the first move, ply 0 being full move 1.
        let (soft, hard) = donor_bounds(3_000.0, 30.0, 10.0, 1.0);
        let first = clock(3_000, 30, 0);
        assert!(close(first.optimum_ms, soft), "{}", first.optimum_ms);
        assert!(close(first.maximum_ms, hard), "{}", first.maximum_ms);
        assert!((43.0..44.0).contains(&soft), "{soft}");
        // The share grows with the move number: ply 120 is full move 61.
        let (late_soft, _) = donor_bounds(3_000.0, 30.0, 10.0, 61.0);
        assert!(close(clock(3_000, 30, 120).optimum_ms, late_soft));
        assert!(late_soft > 3.0 * soft);
        // CCRL's Blitz control.
        let (soft, hard) = donor_bounds(120_000.0, 1_000.0, 10.0, 21.0);
        let blitz = clock(120_000, 1_000, 40);
        assert!(close(blitz.optimum_ms, soft) && close(blitz.maximum_ms, hard));
    }

    #[test]
    fn movestogo_gives_each_move_its_share_of_the_main_clock() {
        let options = SearchLimits {
            black_time: Some(10_000),
            black_increment: 0,
            movestogo: 10,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &EngineOptions::default(), Color::Black, 0, 64);
        // A tenth of the 9,990 ms main clock, and five times it, less 15 ms.
        assert!(close(lim.optimum_ms, 999.0 - 15.0), "{}", lim.optimum_ms);
        assert!(close(lim.maximum_ms, 4_995.0 - 15.0), "{}", lim.maximum_ms);
        // On the last move before the control the donor allows the main
        // clock and its increment (1,050 and 1,075 ms here), which passes the
        // clock; the reserve holds both at `time - 2 * overhead`.
        let last = SearchLimits {
            black_time: Some(1_000),
            black_increment: 100,
            movestogo: 1,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&last, &EngineOptions::default(), Color::Black, 0, 64);
        assert!(close(lim.maximum_ms, 980.0), "{}", lim.maximum_ms);
        assert!(close(lim.optimum_ms, 980.0), "{}", lim.optimum_ms);
    }

    #[test]
    fn movetime_uses_full_budget_as_hard_limit() {
        // Fixed movetime uses the entire budget (SF/Reckless default); the
        // MoveOverhead is NOT subtracted in movetime mode (the forfeit
        // reserve lives in the clock path, not here).
        let engine = EngineOptions {
            move_overhead: 25.0,
            ..EngineOptions::default()
        };
        let options = SearchLimits {
            move_time: 250,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &engine, Color::White, 0, 64);

        assert_eq!(lim.optimum_ms, 250.0);
        assert_eq!(lim.maximum_ms, 250.0);
        assert!(lim.movetime_mode);
    }

    #[test]
    fn movetime_ignores_move_overhead() {
        // A large MoveOverhead must not shrink the movetime budget: `go
        // movetime T` is an explicit "think exactly this long" instruction.
        let engine = EngineOptions {
            move_overhead: 200.0,
            ..EngineOptions::default()
        };
        let options = SearchLimits {
            move_time: 100,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &engine, Color::Black, 0, 64);

        assert_eq!(lim.optimum_ms, 100.0);
        assert_eq!(lim.maximum_ms, 100.0);
    }

    #[test]
    fn movetime_tiny_budget_is_at_least_one_ms() {
        let engine = EngineOptions {
            move_overhead: 10.0,
            ..EngineOptions::default()
        };
        let options = SearchLimits {
            move_time: 1,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &engine, Color::Black, 0, 64);

        assert_eq!(lim.optimum_ms, 1.0);
        assert_eq!(lim.maximum_ms, 1.0);
        assert!(lim.movetime_mode);
    }

    /// A present clock of 1 ms, what a zero, negative or unreadable `wtime`
    /// becomes, moves at once; only an absent clock leaves the search
    /// without a clock limit.
    #[test]
    fn a_one_millisecond_clock_moves_at_once() {
        let options = SearchLimits {
            white_time: Some(1),
            black_time: Some(1),
            ..SearchLimits::default()
        };
        let lim = limits(0, &options);

        assert!(lim.maximum_ms <= 1.0, "maximum_ms={}", lim.maximum_ms);
        assert!(lim.optimum_ms <= lim.maximum_ms);
    }

    #[test]
    fn clock_optimum_less_than_maximum() {
        // For any valid clock position optimum < maximum.
        let options = SearchLimits {
            white_time: Some(1000),
            white_increment: 100,
            ..SearchLimits::default()
        };
        let lim = limits(0, &options);

        assert!(lim.optimum_ms < lim.maximum_ms);
        assert!(!lim.movetime_mode);
    }

    #[test]
    fn sudden_death_stays_within_clock() {
        let options = SearchLimits {
            white_time: Some(1000),
            white_increment: 100,
            ..SearchLimits::default()
        };
        let lim = limits(0, &options);

        assert!(lim.optimum_ms < 150.0, "optimum_ms={}", lim.optimum_ms);
        assert!(lim.maximum_ms < 1000.0, "maximum_ms={}", lim.maximum_ms);
    }

    #[test]
    fn clock_selection_uses_side_to_move() {
        let options = SearchLimits {
            white_time: Some(1_000),
            white_increment: 0,
            black_time: Some(10_000),
            black_increment: 0,
            ..SearchLimits::default()
        };

        let white =
            compute_runtime_limits(&options, &EngineOptions::default(), Color::White, 0, 64);
        let black =
            compute_runtime_limits(&options, &EngineOptions::default(), Color::Black, 0, 64);

        assert!(white.optimum_ms < black.optimum_ms);
        assert!(white.maximum_ms < black.maximum_ms);
    }

    #[test]
    fn clock_maximum_reserves_two_move_overheads_at_low_time() {
        // The wall time charged includes latency outside the search, so the
        // hard bound never passes `time - 2*overhead`. With a 20 ms overhead
        // the donor's bound, 15 ms under a 30 ms main clock, would pass it.
        let engine = EngineOptions {
            move_overhead: 20.0,
            ..EngineOptions::default()
        };
        let options = SearchLimits {
            white_time: Some(50),
            white_increment: 50,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &engine, Color::White, 2, 64);

        assert!(
            close(lim.maximum_ms, 50.0 - 2.0 * 20.0),
            "{}",
            lim.maximum_ms
        );
        assert!(lim.optimum_ms <= lim.maximum_ms);
    }

    #[test]
    fn at_one_thread_the_donors_bound_lies_inside_the_reserve() {
        // At the default overhead the donor's own 15 ms off a bound already
        // within the main clock keeps the hard bound under `time - 20`, at a
        // normal clock and at a scramble alike.
        for (time, increment, game_ply) in [(3_000, 30, 20), (50, 300, 126), (100, 30, 80)] {
            let lim = limits(
                game_ply,
                &SearchLimits {
                    white_time: Some(time),
                    white_increment: increment,
                    ..SearchLimits::default()
                },
            );
            let (soft, hard) = donor_bounds(
                time as f64,
                increment as f64,
                10.0,
                f64::from(game_ply / 2 + 1),
            );
            assert!(close(lim.maximum_ms, hard), "{time}: {}", lim.maximum_ms);
            assert!(
                close(lim.optimum_ms, soft.min(hard)),
                "{time}: {}",
                lim.optimum_ms
            );
            assert!(lim.maximum_ms <= time as f64 - 20.0);
        }
    }

    #[test]
    fn several_threads_keep_thirty_milliseconds_more() {
        let clock = |time: usize, threads: usize| {
            compute_runtime_limits(
                &SearchLimits {
                    white_time: Some(time),
                    white_increment: 300,
                    ..SearchLimits::default()
                },
                &EngineOptions {
                    threads,
                    ..EngineOptions::default()
                },
                Color::White,
                126,
                64,
            )
        };
        // The donor's hard bound at 300 ms is the 290 ms main clock less 15.
        assert!(close(clock(300, 1).maximum_ms, 275.0));
        // Four threads stop 50 ms before the clock.
        assert!(close(clock(300, 4).maximum_ms, 250.0));
        // At a normal clock the reserve does not bind at four threads either.
        assert_eq!(clock(3_000, 4).maximum_ms, clock(3_000, 1).maximum_ms);
    }

    #[test]
    fn the_tablebase_extension_starts_only_with_clock_to_spare() {
        let clock = |time: usize, threads: usize| {
            compute_runtime_limits(
                &SearchLimits {
                    white_time: Some(time),
                    white_increment: 30,
                    ..SearchLimits::default()
                },
                &EngineOptions {
                    move_overhead: 10.0,
                    threads,
                    ..EngineOptions::default()
                },
                Color::White,
                126,
                64,
            )
        };
        // A game was lost on time here: 58 ms on the clock, the search over at
        // 25 ms, then one table read that took 54 ms.
        assert_eq!(clock(58, 1).tb_extension(25.0), TbExtension::Skip);
        // With clock to spare the line gets half an overhead.
        assert_eq!(clock(3_000, 1).tb_extension(100.0), TbExtension::Boxed(5.0));
        // The boundary is ten overheads before the hard ceiling: 300 - 20 - 100.
        assert_eq!(clock(300, 1).tb_extension(180.0), TbExtension::Boxed(5.0));
        assert_eq!(clock(300, 1).tb_extension(180.5), TbExtension::Skip);
        // More threads keep a larger reserve, so the boundary moves with it.
        assert_eq!(clock(300, 4).tb_extension(150.0), TbExtension::Boxed(5.0));
        assert_eq!(clock(300, 4).tb_extension(150.5), TbExtension::Skip);
    }

    #[test]
    fn the_tablebase_extension_is_unbounded_without_a_clock_and_boxed_under_movetime() {
        let depth_only = limits(
            0,
            &SearchLimits {
                depth: Some(12),
                ..SearchLimits::default()
            },
        );
        assert_eq!(depth_only.tb_extension(1e9), TbExtension::Unbounded);
        // `go movetime` names no clock to protect; the box alone applies.
        let movetime = limits(
            0,
            &SearchLimits {
                move_time: 100,
                ..SearchLimits::default()
            },
        );
        assert_eq!(movetime.tb_extension(100.0), TbExtension::Boxed(5.0));
        // No overhead, no box.
        let no_overhead = compute_runtime_limits(
            &SearchLimits {
                move_time: 100,
                ..SearchLimits::default()
            },
            &EngineOptions {
                move_overhead: 0.0,
                ..EngineOptions::default()
            },
            Color::White,
            0,
            64,
        );
        assert_eq!(no_overhead.tb_extension(0.0), TbExtension::Skip);
    }

    fn signals(
        node_share: f64,
        score_fall: f64,
        pv_stability: i32,
        eval_stability: i32,
        best_move_changes: u32,
    ) -> IterationSignals {
        IterationSignals {
            node_share,
            score_fall,
            pv_stability,
            eval_stability,
            best_move_changes,
        }
    }

    #[test]
    fn the_soft_target_is_the_soft_bound_times_the_five_factors_capped_at_the_hard_bound() {
        let params = seeds();
        let mut clock = RuntimeLimits {
            optimum_ms: 1_000.0,
            maximum_ms: 5_000.0,
            ..RuntimeLimits::default()
        };
        let read = signals(0.6, 3.0, 3, 2, 2);
        let soft = soft_stop(&params, &clock, read);
        assert!(close(soft.nodes, 3.1838 - 2.6554 * 0.6));
        assert!(close(soft.score_trend, 0.7426 + 0.1189 * 3.0));
        assert!(close(soft.pv_stability, 1.2881 - 0.0440 * 3.0));
        assert!(close(soft.eval_stability, 1.2664 - 0.0416 * 2.0));
        assert!(close(
            soft.best_move_stability,
            1.15 + (0.2526_f64 * 2.0).ln_1p()
        ));
        let product = soft.nodes
            * soft.score_trend
            * soft.pv_stability
            * soft.eval_stability
            * soft.best_move_stability;
        assert!(close(soft.target_ms, 1_000.0 * product));
        clock.maximum_ms = 1_200.0;
        assert_eq!(soft_stop(&params, &clock, read).target_ms, 1_200.0);
    }

    #[test]
    fn each_factor_stops_at_its_floor_or_clamp() {
        let params = seeds();
        let clock = RuntimeLimits {
            optimum_ms: 1_000.0,
            maximum_ms: f64::INFINITY,
            ..RuntimeLimits::default()
        };
        let soft = |read: IterationSignals| soft_stop(&params, &clock, read);
        // A search spent on one move is cut to the floor; spread over several
        // it is extended, past three times at none.
        assert!(close(soft(signals(1.0, 0.0, 0, 0, 0)).nodes, 0.546));
        assert!(close(soft(signals(0.0, 0.0, 0, 0, 0)).nodes, 3.1838));
        // A fall buys time up to 1.4031, a rise cuts it to 0.7214.
        assert!(close(
            soft(signals(0.5, 100.0, 0, 0, 0)).score_trend,
            1.4031
        ));
        assert!(close(
            soft(signals(0.5, -100.0, 0, 0, 0)).score_trend,
            0.7214
        ));
        // Long-held moves and scores cut to their floors.
        assert!(close(
            soft(signals(0.5, 0.0, 100, 0, 0)).pv_stability,
            0.716
        ));
        assert!(close(
            soft(signals(0.5, 0.0, 0, 100, 0)).eval_stability,
            0.8642
        ));
        // No change keeps the base; changes add their log.
        assert!(close(
            soft(signals(0.5, 0.0, 0, 0, 0)).best_move_stability,
            1.15
        ));
        assert!(soft(signals(0.5, 0.0, 0, 0, 9)).best_move_stability < 1.15 + 9.0 * 0.2526);
    }

    #[test]
    fn crossed_trend_clamps_from_a_fit_do_not_panic() {
        let crossed = SearchParams {
            tm_trend_low: 15_000,
            tm_trend_high: 7_000,
            ..seeds()
        };
        let clock = RuntimeLimits {
            optimum_ms: 1_000.0,
            maximum_ms: f64::INFINITY,
            ..RuntimeLimits::default()
        };
        let trend = soft_stop(&crossed, &clock, signals(0.5, 1_000.0, 0, 0, 0)).score_trend;
        assert!(close(trend, 1.5));
    }

    #[test]
    fn the_score_trend_slope_is_the_donors_in_rarogs_units() {
        // The donor reads 0.0480 a unit of its evaluation; the kernel's unit
        // is 1024/2536 of it, so a per-unit slope is divided by that.
        let folded = (0.0480_f64 * 2536.0 / 1024.0 * 10_000.0).round();
        assert_eq!(f64::from(SearchParams::default().tm_trend_slope), folded);
        assert_eq!(
            seeds().tm_trend_slope,
            SearchParams::default().tm_trend_slope
        );
    }

    #[test]
    fn only_a_clock_search_that_asks_for_a_move_may_end_on_a_mate() {
        let clock = SearchLimits {
            white_time: Some(60_000),
            ..SearchLimits::default()
        };
        assert!(limits(0, &clock).mate_stop);
        let refused = [
            SearchLimits {
                infinite: true,
                ..clock.clone()
            },
            SearchLimits {
                ponder: true,
                ..clock.clone()
            },
            SearchLimits {
                depth: Some(20),
                ..clock.clone()
            },
            SearchLimits {
                mate: Some(3),
                ..clock.clone()
            },
            SearchLimits {
                nodes: 1_000_000,
                ..clock.clone()
            },
            SearchLimits {
                search_moves: vec![Move::from_uci("e2e4").expect("valid move").into()],
                ..clock
            },
            SearchLimits {
                move_time: 1_000,
                ..SearchLimits::default()
            },
            SearchLimits::default(),
        ];
        for (index, request) in refused.iter().enumerate() {
            assert!(!limits(0, request).mate_stop, "request {index}");
        }
    }

    #[test]
    fn a_mate_ends_the_search_after_three_iterations_with_the_same_move_and_distance() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let mate = MATE_SCORE - 5;

        let mut watch = MateWatch::default();
        assert!(!watch.completed(e2e4, mate));
        assert!(!watch.completed(e2e4, mate));
        assert!(watch.completed(e2e4, mate));

        // A new distance or a new move starts the count again, as does a
        // score outside the mate band.
        let mut watch = MateWatch::default();
        assert!(!watch.completed(e2e4, mate));
        assert!(!watch.completed(e2e4, mate - 2));
        assert!(!watch.completed(d2d4, mate - 2));
        assert!(!watch.completed(d2d4, mate - 2));
        assert!(!watch.completed(d2d4, 300));
        assert!(!watch.completed(d2d4, mate - 2));
        assert!(!watch.completed(d2d4, mate - 2));
        assert!(watch.completed(d2d4, mate - 2));

        // A tablebase win is not a proved mate.
        let mut watch = MateWatch::default();
        for _ in 0..5 {
            assert!(!watch.completed(e2e4, crate::tt::TB_VALUE));
        }

        // A loss in two plies has nothing left to find; one in three has.
        assert!(MateWatch::default().completed(e2e4, -MATE_SCORE + 2));
        assert!(!MateWatch::default().completed(e2e4, -MATE_SCORE + 3));
    }

    #[test]
    fn depth_is_clamped_and_absent_clock_is_unbounded() {
        let shallow = limits(
            0,
            &SearchLimits {
                // A fractional depth is now unrepresentable (Option<u32>);
                // Some(0) exercises the same lower clamp this test was written for.
                depth: Some(0),
                ..SearchLimits::default()
            },
        );
        assert_eq!(shallow.depth, 1);
        assert!(shallow.optimum_ms.is_infinite());
        assert!(shallow.maximum_ms.is_infinite());

        let unlimited = compute_runtime_limits(
            &SearchLimits::default(),
            &EngineOptions::default(),
            Color::Black,
            0,
            42,
        );
        assert_eq!(unlimited.depth, 42);
    }

    /// A mate search has no ply limit of its own, wants the mate rather than a
    /// move, and reads N as a distance in plies.
    #[test]
    fn a_mate_search_is_bounded_only_by_its_proof() {
        let mate = SearchLimits {
            mate: Some(3),
            ..SearchLimits::default()
        };
        let plain = limits(0, &SearchLimits::default());
        let limited = limits(0, &mate);
        assert_eq!(limited.depth, plain.depth);
        assert_eq!(limited.mate_plies, Some(6));
        assert!(plain.forced_move_shortcut && !limited.forced_move_shortcut);
        assert_eq!(plain.mate_plies, None);

        let deep = SearchLimits {
            mate: Some(u32::MAX),
            ..SearchLimits::default()
        };
        assert_eq!(limits(0, &deep).mate_plies, Some(i32::MAX));
    }
}
