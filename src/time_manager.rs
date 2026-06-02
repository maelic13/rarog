use crate::board::Color;
use crate::search_options::{EngineOptions, SearchLimits};

#[derive(Copy, Clone)]
pub(crate) struct RuntimeLimits {
    pub depth: usize,
    pub nodes: u64,
    pub soft_ms: f64,
    pub hard_ms: f64,
}

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct RootTimeSignals {
    pub stable_best_depths: usize,
    pub eval_stable_depths: usize,
    pub best_move_changes: usize,
    pub best_effort: f64,
    pub score_drop: i32,
}

pub(crate) fn soft_time_multiplier(signals: RootTimeSignals) -> f64 {
    let mut multiplier: f64 = 1.0;

    if signals.stable_best_depths >= 2 && signals.eval_stable_depths >= 1 {
        multiplier *= 0.82;
    } else if signals.stable_best_depths == 0 {
        multiplier *= 1.18;
    }

    if signals.best_move_changes >= 2 {
        multiplier *= 1.18;
    } else if signals.best_move_changes == 0 && signals.stable_best_depths >= 3 {
        multiplier *= 0.92;
    }

    if signals.score_drop > 90 {
        multiplier *= 1.28;
    } else if signals.score_drop > 45 {
        multiplier *= 1.12;
    }

    if signals.best_effort < 0.23 {
        multiplier *= 1.18;
    } else if signals.best_effort > 0.72 && signals.stable_best_depths >= 1 {
        multiplier *= 0.86;
    }

    multiplier.clamp(0.55, 1.75)
}

pub(crate) fn root_signals_ready_to_stop(signals: RootTimeSignals) -> bool {
    signals.stable_best_depths >= 1
        && signals.score_drop <= 55
        && (signals.eval_stable_depths >= 1 || signals.best_effort >= 0.62)
}

pub(crate) fn compute_runtime_limits(
    options: &SearchLimits,
    engine_options: &EngineOptions,
    side_to_move: Color,
    max_depth: usize,
) -> RuntimeLimits {
    let depth = if options.depth.is_finite() {
        options.depth.max(1.0) as usize
    } else {
        max_depth
    };
    let mut soft_ms = f64::INFINITY;
    let mut hard_ms = f64::INFINITY;

    if options.move_time > 0 {
        let available = (options.move_time as f64 - engine_options.move_overhead).max(1.0);
        soft_ms = available;
        hard_ms = available;
    } else {
        let (time, increment) = match side_to_move {
            Color::White => (options.white_time, options.white_increment),
            Color::Black => (options.black_time, options.black_increment),
        };
        if time > 0 {
            let remaining = (time as f64 - engine_options.move_overhead).max(1.0);
            let explicit_moves_to_go = options.movestogo > 0;
            let moves_to_go = if explicit_moves_to_go {
                options.movestogo.min(50) as f64
            } else if remaining < 1_000.0 {
                14.0
            } else if increment > 0 {
                30.0
            } else {
                40.0
            };
            if explicit_moves_to_go {
                let usable = (time as f64 + increment as f64 * (moves_to_go - 1.0)
                    - engine_options.move_overhead * (moves_to_go + 1.0))
                    .max(1.0);
                soft_ms = (usable / (moves_to_go + 1.2))
                    .min(remaining * 0.88)
                    .max(1.0);
                hard_ms = (soft_ms * (2.20 + 0.08 * moves_to_go))
                    .min(remaining * 0.82)
                    .max(soft_ms);
            } else {
                let increment_scale = if remaining < 2_000.0 { 0.20 } else { 0.70 };
                let clock_scale = if increment > 0 { 1.0 } else { 0.85 };
                soft_ms = (remaining * clock_scale / moves_to_go
                    + increment as f64 * increment_scale)
                    .min(remaining * 0.22)
                    .max(1.0);
                let reserve_cap = if remaining < 2_000.0 { 0.30 } else { 0.76 };
                hard_ms = (soft_ms * 3.0).min(remaining * reserve_cap).max(soft_ms);
            }
        }
    }

    RuntimeLimits {
        depth,
        nodes: options.nodes,
        soft_ms,
        hard_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < f64::EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn movetime_uses_exact_budget_after_overhead() {
        let mut engine = EngineOptions::default();
        engine.move_overhead = 25.0;
        let options = SearchLimits {
            move_time: 250,
            white_time: 1,
            black_time: 1,
            white_increment: 1_000,
            black_increment: 1_000,
            ..SearchLimits::default()
        };
        let limits = compute_runtime_limits(&options, &engine, Color::White, 64);

        assert_close(limits.soft_ms, 225.0);
        assert_close(limits.hard_ms, 225.0);
    }

    #[test]
    fn movetime_never_allocates_less_than_one_millisecond() {
        let mut engine = EngineOptions::default();
        engine.move_overhead = 10.0;
        let options = SearchLimits {
            move_time: 5,
            ..SearchLimits::default()
        };
        let limits = compute_runtime_limits(&options, &engine, Color::Black, 64);

        assert_close(limits.soft_ms, 1.0);
        assert_close(limits.hard_ms, 1.0);
    }

    #[test]
    fn sudden_death_time_control_keeps_a_hard_reserve() {
        let options = SearchLimits {
            white_time: 1000,
            white_increment: 100,
            ..SearchLimits::default()
        };
        let limits = compute_runtime_limits(&options, &EngineOptions::default(), Color::White, 64);

        assert!(limits.soft_ms < 100.0);
        assert!(limits.hard_ms < 300.0);
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

        let white_limits =
            compute_runtime_limits(&options, &EngineOptions::default(), Color::White, 64);
        let black_limits =
            compute_runtime_limits(&options, &EngineOptions::default(), Color::Black, 64);

        assert!(white_limits.soft_ms < black_limits.soft_ms);
        assert!(white_limits.hard_ms < black_limits.hard_ms);
    }

    #[test]
    fn movestogo_uses_explicit_clock_horizon() {
        let options = SearchLimits {
            black_time: 10_000,
            black_increment: 0,
            movestogo: 10,
            ..SearchLimits::default()
        };
        let limits = compute_runtime_limits(&options, &EngineOptions::default(), Color::Black, 64);

        assert!((850.0..=900.0).contains(&limits.soft_ms));
        assert!(limits.hard_ms <= 6_000.0);
    }

    #[test]
    fn depth_is_clamped_and_absent_clock_is_unbounded() {
        let shallow = SearchLimits {
            depth: 0.25,
            ..SearchLimits::default()
        };
        let shallow_limits =
            compute_runtime_limits(&shallow, &EngineOptions::default(), Color::White, 64);

        assert_eq!(shallow_limits.depth, 1);
        assert!(shallow_limits.soft_ms.is_infinite());
        assert!(shallow_limits.hard_ms.is_infinite());

        let unlimited = SearchLimits::default();
        let unlimited_limits =
            compute_runtime_limits(&unlimited, &EngineOptions::default(), Color::Black, 42);

        assert_eq!(unlimited_limits.depth, 42);
    }

    #[test]
    fn stable_root_signals_reduce_soft_time() {
        let stable = RootTimeSignals {
            stable_best_depths: 3,
            eval_stable_depths: 2,
            best_move_changes: 0,
            best_effort: 0.78,
            score_drop: 0,
        };
        let unstable = RootTimeSignals {
            stable_best_depths: 0,
            eval_stable_depths: 0,
            best_move_changes: 3,
            best_effort: 0.15,
            score_drop: 120,
        };

        assert!(soft_time_multiplier(stable) < 1.0);
        assert!(soft_time_multiplier(unstable) > 1.0);
        assert!(soft_time_multiplier(stable) < soft_time_multiplier(unstable));
    }

    #[test]
    fn root_signals_stop_only_when_best_and_eval_are_reliable() {
        let stable = RootTimeSignals {
            stable_best_depths: 1,
            eval_stable_depths: 1,
            best_move_changes: 0,
            best_effort: 0.50,
            score_drop: 20,
        };
        let low_confidence = RootTimeSignals {
            stable_best_depths: 1,
            eval_stable_depths: 0,
            best_move_changes: 1,
            best_effort: 0.20,
            score_drop: 20,
        };
        let score_drop = RootTimeSignals {
            score_drop: 90,
            ..stable
        };

        assert!(root_signals_ready_to_stop(stable));
        assert!(!root_signals_ready_to_stop(low_confidence));
        assert!(!root_signals_ready_to_stop(score_drop));
    }
}
