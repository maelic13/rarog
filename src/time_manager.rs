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
        // Fixed movetime: use the full budget as the hard limit (the
        // SF/Reckless default). 2.9.1 originally reserved
        // `min(MoveOverhead, T/10)` here, but that was a misattribution: the
        // 28 time forfeits that motivated 2.9.1 were all in the *clock* path
        // (tc=3+0.03 → wtime/btime/winc/binc), fixed by the `2*overhead`
        // reserve in the else-branch below. Movetime mode never forfeited
        // (`t=0` over a full 100 ms/move gauntlet), yet the reserve cost ~10 %
        // of thinking time: at 100 ms/move Rarog measured `tpm=92.9` (90 ms
        // budget + ~3 ms GUI latency) while Stockfish used `tpm=110.2` with
        // `t=0` — proving the harness tolerates ~10 % past the nominal time.
        // check_stop (every 2048 nodes) aborts within ~1 ms of `maximum_ms`
        // and the pre/post latency is only ~3 ms, so the full budget lands
        // ~3 % over nominal — comfortably inside that tolerance.
        let movetime = (options.move_time as f64).max(1.0);
        optimum_ms = movetime;
        maximum_ms = movetime;
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
            let time_left =
                (time as f64 + increment as f64 * (mtg - 1.0) - overhead * (2.0 + mtg)).max(1.0);

            let ply = game_ply as f64;

            let (opt_scale, max_scale) = if explicit_mtg {
                // Explicit movestogo branch (SF timeman.cpp)
                let opt = ((0.88 + ply / 116.4) / mtg).min(0.88 * time as f64 / time_left);
                let max = 1.3 + 0.11 * mtg;
                (opt, max)
            } else {
                // Sudden death / increment (SF timeman.cpp)
                let log_t = (time_left / 1000.0).max(1e-9).log10();
                let opt_const = (0.0029869 + 0.00033554 * log_t).min(0.004905);
                let max_const = (3.3744 + 3.0608 * log_t).max(3.1441);
                let opt = (0.012112 + (ply + 3.22713_f64).max(0.0).powf(0.46866) * opt_const)
                    .min(0.19404 * time as f64 / time_left);
                let max = (6.873_f64).min(max_const + ply / 12.352);
                (opt, max)
            };

            optimum_ms = (opt_scale * time_left).max(1.0);
            // SF: maximum = max(optimum, min(0.8097*time - overhead, maxScale*optimum))
            maximum_ms =
                ((0.8097 * time as f64 - overhead).min(max_scale * optimum_ms)).max(optimum_ms);

            // Time-safety reserve (Phase 2.9.1). The SF maximum above leaves
            // only ~19% of the clock plus one Move Overhead unused; at low
            // remaining time that slack is just a few ms. The clock is polled
            // (and the iteration aborted) within ~1 ms of `maximum_ms`, but the
            // wall time the GUI actually charges also includes the latency
            // *before* our clock starts (`go` received → `self.start`) and the
            // latency for `bestmove` to reach the GUI. Under a loaded gauntlet
            // those spike well past the thin low-time slack, which is why the
            // 2.2 SF-style TM rewrite introduced time forfeits (Rarog 2.0.2's
            // old, more conservative TM forfeited 0) that the SF formula's thin
            // low-time slack does not cover.
            //
            // Guarantee an absolute reserve of `2*overhead` on top of the
            // percentage reserve: never schedule a hard limit past
            // `time - 2*overhead`. This only binds when `time < ~52*overhead`
            // (≈520 ms at the default 10 ms overhead) — i.e. only in genuine
            // time scrambles, where playing a hair faster costs ~no Elo — and
            // leaves normal-time allocation (the +81 Elo from 2.2) untouched.
            let min_reserve = 2.0 * overhead;
            let hard_ceiling = (time as f64 - min_reserve).max(1.0);
            maximum_ms = maximum_ms.min(hard_ceiling);
            optimum_ms = optimum_ms.min(maximum_ms);
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
        compute_runtime_limits(
            &options,
            &EngineOptions::default(),
            Color::White,
            game_ply,
            64,
        )
    }

    #[test]
    fn movetime_uses_full_budget_as_hard_limit() {
        // Fixed movetime uses the entire budget (SF/Reckless default); the
        // MoveOverhead is NOT subtracted in movetime mode (the 2.9.1 forfeit
        // fix lives in the clock path, not here).
        let mut engine = EngineOptions::default();
        engine.move_overhead = 25.0;
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
        let mut engine = EngineOptions::default();
        engine.move_overhead = 200.0;
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
        let mut engine = EngineOptions::default();
        engine.move_overhead = 10.0;
        let options = SearchLimits {
            move_time: 1,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &engine, Color::Black, 0, 64);

        assert_eq!(lim.optimum_ms, 1.0);
        assert_eq!(lim.maximum_ms, 1.0);
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

        let white =
            compute_runtime_limits(&options, &EngineOptions::default(), Color::White, 0, 64);
        let black =
            compute_runtime_limits(&options, &EngineOptions::default(), Color::Black, 0, 64);

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
        let lim = compute_runtime_limits(&options, &EngineOptions::default(), Color::Black, 0, 64);

        // mtg=10, timeLeft≈9880, optScale≈0.088 → optimum≈869
        assert!(
            (800.0..=950.0).contains(&lim.optimum_ms),
            "optimum_ms={}",
            lim.optimum_ms
        );
        assert!(lim.maximum_ms <= 10_000.0);
    }

    #[test]
    fn clock_maximum_reserves_two_move_overheads_at_low_time() {
        // At low remaining time the percentage reserve in the SF maximum
        // formula is only a few ms; the absolute `2*overhead` reserve must
        // keep maximum_ms at or below `time - 2*overhead` so startup/output
        // latency under load cannot flag. Chosen so the reserve actually
        // binds (low time, sizeable optimum via a large increment).
        let mut engine = EngineOptions::default();
        engine.move_overhead = 10.0;
        let options = SearchLimits {
            white_time: 50,
            white_increment: 50,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &engine, Color::White, 2, 64);

        let hard_ceiling = 50.0 - 2.0 * 10.0;
        assert!(
            lim.maximum_ms <= hard_ceiling,
            "maximum_ms={} exceeds hard_ceiling={}",
            lim.maximum_ms,
            hard_ceiling
        );
        assert!(lim.optimum_ms <= lim.maximum_ms);
    }

    #[test]
    fn clock_normal_time_allocation_is_not_throttled_by_reserve() {
        // At normal remaining time the `2*overhead` reserve must NOT bind:
        // maximum_ms stays at the SF percentage cap (0.8097*time - overhead),
        // which is the smaller (binding) limit whenever time > ~52*overhead.
        let mut engine = EngineOptions::default();
        engine.move_overhead = 10.0;
        let options = SearchLimits {
            white_time: 3_000,
            white_increment: 30,
            ..SearchLimits::default()
        };
        let lim = compute_runtime_limits(&options, &engine, Color::White, 20, 64);

        let percentage_cap = 0.8097 * 3_000.0 - 10.0;
        let reserve_cap = 3_000.0 - 2.0 * 10.0;
        // The percentage cap is far below the reserve cap here, so it binds.
        assert!(percentage_cap < reserve_cap);
        assert!(
            lim.maximum_ms <= percentage_cap,
            "maximum_ms={} exceeds percentage_cap={}",
            lim.maximum_ms,
            percentage_cap
        );
    }

    #[test]
    fn depth_is_clamped_and_absent_clock_is_unbounded() {
        let shallow = limits(
            0,
            SearchLimits {
                depth: 0.25,
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
}
