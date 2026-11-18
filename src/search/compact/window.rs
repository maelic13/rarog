//! The compact kernel's root window: an aspiration window round a running
//! average of the root's scores, narrower the longer the score and the best
//! move have held, and the optimism that average gives the evaluation.

use crate::board::Move;

use super::rules::{self, SEEDS, Window};

/// What the root carries from one iteration to the next.
pub(in crate::search) struct Iterations {
    /// Each line's running average of its exact root scores; line 1's
    /// decides whether the score held.
    averages: Vec<i32>,
    /// Iterations in a row whose score held near the average.
    eval_stability: i32,
    /// Iterations in a row with the same best move.
    pv_stability: i32,
    last_best: Move,
}

impl Iterations {
    /// The state at a search of `lines` lines' start: every line's average
    /// is the previous search's score.
    pub(in crate::search) fn new(previous_score: i32, lines: usize) -> Self {
        Self {
            averages: vec![previous_score; lines.max(1)],
            eval_stability: 0,
            pv_stability: 0,
            last_best: Move::NULL,
        }
    }

    /// The window `line` opens with this iteration, and the side to move's
    /// optimism: the average of the line's running average and
    /// `pool_average`, the pool's deeper one when there is one. Every line
    /// opens afresh, its width from line 1's stability and its own average.
    pub(in crate::search) fn open(&self, line: usize, pool_average: Option<i32>) -> (Window, i32) {
        let average = self.averages[line];
        let delta = rules::window_delta(SEEDS, self.eval_stability, self.pv_stability, average);
        let best_average = pool_average.map_or(average, |pool| (pool + average) / 2);
        (
            rules::open_window(average, delta),
            rules::optimism(SEEDS, best_average),
        )
    }

    /// `line`'s window closed on `score`: fold it into the line's average,
    /// which starts from the previous search's score and so always holds one.
    pub(in crate::search) fn record_exact(&mut self, line: usize, score: i32) {
        self.averages[line] = (self.averages[line] + score) / 2;
    }

    /// An iteration ended with `best_score` on `best_move`: count how long
    /// the score and the move have held.
    pub(in crate::search) fn end_iteration(&mut self, best_score: i32, best_move: Move) {
        self.eval_stability = if rules::score_held(SEEDS, best_score, self.averages[0]) {
            self.eval_stability + 1
        } else {
            0
        };
        self.pv_stability = if best_move == self.last_best {
            self.pv_stability + 1
        } else {
            0
        };
        self.last_best = best_move;
    }
}

/// The window state while an iteration's root searches fail.
#[derive(Copy, Clone, Debug)]
pub(in crate::search) struct CompactWindow {
    pub(in crate::search) window: Window,
    fails: i32,
}

impl CompactWindow {
    pub(in crate::search) fn new(window: Window) -> Self {
        Self { window, fails: 0 }
    }

    /// The depth the root searches at: one ply less per fail-high so far,
    /// never below one.
    pub(in crate::search) fn depth(&self, depth: i32) -> i32 {
        (depth - self.window.reduction).max(1)
    }

    /// A fail low; past `max_fails` the window opens below fully, a guard
    /// the widening already makes redundant.
    pub(in crate::search) fn fail_low(&mut self, score: i32, max_fails: i32) {
        self.fails += 1;
        self.window = rules::window_fail_low(SEEDS, self.window, score);
        if self.fails >= max_fails {
            self.window.alpha = -crate::eval::INF_SCORE;
        }
    }

    /// A fail high; past `max_fails` the window opens above fully.
    pub(in crate::search) fn fail_high(&mut self, score: i32, max_fails: i32) {
        self.fails += 1;
        self.window = rules::window_fail_high(SEEDS, self.window, score);
        if self.fails >= max_fails {
            self.window.beta = crate::eval::INF_SCORE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_narrows_as_the_score_and_move_hold_and_widens_on_failures() {
        let mut iterations = Iterations::new(0, 1);
        let (opening, optimism) = iterations.open(0, None);
        // The base half-width, 9510 1024ths, to the unit below.
        assert_eq!((opening.alpha, opening.beta, opening.delta), (-9, 9, 9));
        assert_eq!(optimism, 0);
        let mv = Move::from_uci("e2e4").expect("a move");
        for _ in 0..3 {
            iterations.record_exact(0, 5);
            iterations.end_iteration(5, mv);
        }
        let (narrower, optimism) = iterations.open(0, None);
        assert_eq!(
            narrower.delta,
            (9_510 - 2 * 413 + 1024 * 4 * 4 / 10_658) / 1024,
            "held twice since the first"
        );
        assert_eq!(optimism, 46 * 4 / (4 + 81));

        let mut window = CompactWindow::new(narrower);
        window.fail_high(200, 20);
        assert_eq!(window.depth(10), 9, "one ply less after a fail high");
        assert!(window.window.beta > 200 && window.window.delta > narrower.delta);
        window.fail_low(-300, 20);
        assert!(window.window.alpha < -300);
        let mut guarded = CompactWindow::new(narrower);
        guarded.fail_low(0, 1);
        assert_eq!(
            guarded.window.alpha,
            -crate::eval::INF_SCORE,
            "the guard opens it"
        );
    }

    #[test]
    fn each_line_opens_round_its_own_average_with_line_ones_stability() {
        let mut iterations = Iterations::new(0, 3);
        let mv = Move::from_uci("e2e4").expect("a move");
        for _ in 0..4 {
            iterations.record_exact(0, 5);
            iterations.record_exact(2, -400);
            iterations.end_iteration(5, mv);
        }
        let (first, _) = iterations.open(0, None);
        let (second, _) = iterations.open(1, None);
        let (third, optimism) = iterations.open(2, None);
        assert_eq!(first.delta, second.delta, "line 2 still averages 0");
        assert_eq!((second.alpha + second.beta) / 2, 0);
        assert!(third.delta > first.delta, "a far average widens its line");
        assert!((third.alpha + third.beta) / 2 < -300);
        assert!(optimism < 0, "a line's optimism follows its own average");
        // A line's average never sets the stability: line 3's scores are
        // far from line 1's, yet the window narrowed as line 1 held.
        assert_eq!(
            first.delta,
            (9_510 - 3 * 413 + 1024 * 4 * 4 / 10_658) / 1024
        );
    }
}
