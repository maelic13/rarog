use crate::board::Color;
use crate::search_options::{EngineOptions, SearchLimits};

#[derive(Copy, Clone)]
pub(crate) struct RuntimeLimits {
    pub depth: usize,
    pub nodes: u64,
    /// Soft limit: between-iteration stop threshold (clock mode).
    /// In movetime mode this equals `maximum_ms`; the between-iteration
    /// soft-stop logic is skipped entirely for movetime — only `check_stop`
    /// (every 2048 nodes) fires at `maximum_ms`.
    pub optimum_ms: f64,
    /// Hard limit: mid-iteration abort threshold.
    pub maximum_ms: f64,
    /// True when the `go movetime T` command was used.
    pub movetime_mode: bool,
}

/// Compute time limits for one search.
///
/// `game_ply` is the number of half-moves played so far in the game
/// (≈ `2*(fullmove - 1) + (side_to_move == Black) as u32`).
/// It is used in Stockfish's clock formulas to allocate more time in the
/// opening (ply 0) and gradually less as the game progresses.
pub(crate) fn compute_runtime_limits(
    options: &SearchLimits,
    engine_options: &EngineOptions,
    side_to_move: Color,
    game_ply: u32,
    max_depth: usize,
) -> RuntimeLimits {
    let depth = if options.depth.is_finite() {
        options.depth.max(1.0) as usize
    } else {
        max_depth
    };
    let mut optimum_ms = f64::INFINITY;
    let mut maximum_ms = f64::INFINITY;
    let mut movetime_mode = false;

    if options.move_time > 0 {
        // Fixed movetime: pure hard limit, no Move Overhead subtraction.
        // SF and Reckless both use `soft = hard = T` here.
        // The every-2048-nodes check_stop aborts mid-iteration at maximum_ms.
        // If time forfeits appear in testing add `min(overhead, T/10)` as a
        // safety valve, but try the pure version first.
        optimum_ms = options.move_time as f64;
        maximum_ms = options.move_time as f64;
        movetime_mode = true;
    } else {
        let (time, increment) = match side_to_move {
            Color::White => (options.white_time, options.white_increment),
            Color::Black => (options.black_time, options.black_increment),
        };
        if time > 0 {
            let overhead = engine_options.move_overhead;
            let explicit_mtg = options.movestogo > 0;
            let mtg = if explicit_mtg {
                options.movestogo.min(50) as f64
            } else {
                50.0
            };

            // SF: timeLeft = max(1, time + inc*(mtg-1) - overhead*(2+mtg))
            let time_left = (time as f64 + increment as f64 * (mtg - 1.0)
                - overhead * (2.0 + mtg))
                .max(1.0);

            let ply = game_ply as f64;

            let (opt_scale, max_scale) = if explicit_mtg {
                // Explicit movestogo branch (SF timeman.cpp)
                let opt = ((0.88 + ply / 116.4) / mtg)
                    .min(0.88 * time as f64 / time_left);
                let max = 1.3 + 0.11 * mtg;
                (opt, max)
            } else {
                // Sudden death / increment (SF timeman.cpp)
                let log_t = (time_left / 1000.0).max(1e-9).log10();
                let opt_const = (0.0029869 + 0.00033554 * log_t).min(0.004905);
                let max_const = (3.3744 + 3.0608 * log_t).max(3.1441);
                let opt = (0.012112
                    + (ply + 3.22713_f64).max(0.0).powf(0.46866) * opt_const)
                    .min(0.19404 * time as f64 / time_left);
                let max = (6.873_f64).min(max_const + ply / 12.352);
                (opt, max)
            };

            optimum_ms = (opt_scale * time_left).max(1.0);
            // SF: maximum = max(optimum, min(0.8097*time - overhead, maxScale*optimum))
            maximum_ms = ((0.8097 * time as f64 - overhead)
                .min(max_scale * optimum_ms))
                .max(optimum_ms);
        }
    }

    RuntimeLimits {
        depth,
        nodes: options.nodes,
        optimum_ms,
        maximum_ms,
        movetime_mode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(game_ply: u32, options: SearchLimits) -> RuntimeLimits {
        compute_runtime_limits(&options, &EngineOptions::default(), Color::White, game_ply, 64)
    }

    #[test]
    fn movetime_is_pure_hard_limit_no_overhead() {
        let mut engine = EngineOptions::default();
        engine.move_overhead = 25.0;
        let options = SearchLimits {
            move_time: 250,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &engine, Color::White, 0, 64);

        // No overhead subtraction: optimum == maximum == move_time exactly.
        assert_eq!(lim.optimum_ms, 250.0);
        assert_eq!(lim.maximum_ms, 250.0);
        assert!(lim.movetime_mode);
    }

    #[test]
    fn movetime_small_budget_not_clamped() {
        // Previously clamped to 1.0 after overhead subtraction; now the raw T is used.
        let mut engine = EngineOptions::default();
        engine.move_overhead = 10.0;
        let options = SearchLimits {
            move_time: 5,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &engine, Color::Black, 0, 64);

        assert_eq!(lim.optimum_ms, 5.0);
        assert_eq!(lim.maximum_ms, 5.0);
        assert!(lim.movetime_mode);
    }

    #[test]
    fn clock_optimum_less_than_maximum() {
        // For any valid clock position optimum < maximum.
        let options = SearchLimits {
            white_time: 1000,
            white_increment: 100,
            ..SearchLimits::default()
        };
        let lim = limits(0, options);

        assert!(lim.optimum_ms < lim.maximum_ms);
        assert!(!lim.movetime_mode);
    }

    #[test]
    fn sudden_death_stays_within_clock() {
        let options = SearchLimits {
            white_time: 1000,
            white_increment: 100,
            ..SearchLimits::default()
        };
        let lim = limits(0, options);

        assert!(lim.optimum_ms < 150.0, "optimum_ms={}", lim.optimum_ms);
        assert!(lim.maximum_ms < 1000.0, "maximum_ms={}", lim.maximum_ms);
    }

    #[test]
    fn clock_selection_uses_side_to_move() {
        let options = SearchLimits {
            white_time: 1_000,
            white_increment: 0,
            black_time: 10_000,
            black_increment: 0,
            ..SearchLimits::default()
        };

        let white = compute_runtime_limits(
            &options, &EngineOptions::default(), Color::White, 0, 64,
        );
        let black = compute_runtime_limits(
            &options, &EngineOptions::default(), Color::Black, 0, 64,
        );

        assert!(white.optimum_ms < black.optimum_ms);
        assert!(white.maximum_ms < black.maximum_ms);
    }

    #[test]
    fn movestogo_uses_explicit_clock_horizon() {
        let options = SearchLimits {
            black_time: 10_000,
            black_increment: 0,
            movestogo: 10,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(
            &options, &EngineOptions::default(), Color::Black, 0, 64,
        );

        // mtg=10, timeLeft≈9880, optScale≈0.088 → optimum≈869
        assert!((800.0..=950.0).contains(&lim.optimum_ms), "optimum_ms={}", lim.optimum_ms);
        assert!(lim.maximum_ms <= 10_000.0);
    }

    #[test]
    fn depth_is_clamped_and_absent_clock_is_unbounded() {
        let shallow = limits(0, SearchLimits { depth: 0.25, ..SearchLimits::default() });
        assert_eq!(shallow.depth, 1);
        assert!(shallow.optimum_ms.is_infinite());
        assert!(shallow.maximum_ms.is_infinite());

        let unlimited = compute_runtime_limits(
            &SearchLimits::default(), &EngineOptions::default(), Color::Black, 0, 42,
        );
        assert_eq!(unlimited.depth, 42);
    }
}
