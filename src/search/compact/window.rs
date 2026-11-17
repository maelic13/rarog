//! The compact kernel's root window: an aspiration window round a running
//! average of the root's scores, narrower the longer the score and the best
//! move have held, and the optimism that average gives the evaluation.

use crate::board::Move;

use super::rules::{self, SEEDS, Window};

/// What the root carries from one iteration to the next.
pub(in crate::search) struct Iterations {
    /// The running average of the iterations' exact root scores.
    average: i32,
    /// Iterations in a row whose score held near the average.
    eval_stability: i32,
    /// Iterations in a row with the same best move.
    pv_stability: i32,
    last_best: Move,
}

impl Iterations {
    /// The state at a search's start: the average is the previous search's
    /// score.
    pub(in crate::search) fn new(previous_score: i32) -> Self {
        Self {
            average: previous_score,
            eval_stability: 0,
            pv_stability: 0,
            last_best: Move::NULL,
        }
    }

    /// The window an iteration opens with, and the side to move's optimism:
    /// the average of this thread's running average and `pool_average`, the
    /// pool's deeper one when there is one.
    pub(in crate::search) fn open(&self, pool_average: Option<i32>) -> (Window, i32) {
        let delta =
            rules::window_delta(SEEDS, self.eval_stability, self.pv_stability, self.average);
        let best_average = pool_average.map_or(self.average, |pool| (pool + self.average) / 2);
        (
            rules::open_window(self.average, delta),
            rules::optimism(SEEDS, best_average),
        )
    }

    /// A window closed on `score`: fold it into the average, which starts
    /// from the previous search's score and so always holds one.
    pub(in crate::search) fn record_exact(&mut self, score: i32) {
        self.average = (self.average + score) / 2;
    }

    /// An iteration ended with `best_score` on `best_move`: count how long
    /// the score and the move have held.
    pub(in crate::search) fn end_iteration(&mut self, best_score: i32, best_move: Move) {
        self.eval_stability = if rules::score_held(SEEDS, best_score, self.average) {
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
        let mut iterations = Iterations::new(0);
        let (opening, optimism) = iterations.open(None);
        assert_eq!((opening.alpha, opening.beta, opening.delta), (-23, 23, 23));
        assert_eq!(optimism, 0);
        let mv = Move::from_uci("e2e4").expect("a move");
        for _ in 0..3 {
            iterations.record_exact(5);
            iterations.end_iteration(5, mv);
        }
        let (narrower, optimism) = iterations.open(None);
        assert_eq!(narrower.delta, 23 - 2, "held twice since the first");
        assert_eq!(optimism, 113 * 4 / (4 + 201));

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
}
