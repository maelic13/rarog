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

    let white = compute_runtime_limits(&options, &EngineOptions::default(), Color::White, 0, 64);
    let black = compute_runtime_limits(&options, &EngineOptions::default(), Color::Black, 0, 64);

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
        assert!(!watch.completed(e2e4, crate::score::TB_VALUE));
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
            // Some(0) exercises the lower clamp to depth 1.
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
