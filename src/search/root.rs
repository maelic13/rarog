//! The root drivers, single-line and MultiPV, and the signals each
//! iteration reads to stop or go on.

use super::*;

impl Searcher {
    pub(super) fn search_root<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        mut board: Board,
        legal_moves: &[Move],
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        #[cfg(feature = "donor-eval")]
        self.td.evaluator.set_donor_root(None);
        // The SERIAL path owns the diag lifecycle here. In a parallel
        // search `search_parallel` resets before spawning and dumps after
        // joining — helpers reach this function too, so a reset/dump left
        // unconditional would run once PER THREAD, wiping earlier threads'
        // counts on the way in and emitting N competing dumps on the way out.
        if self.shared.threads == 1 {
            crate::diag::reset();
        }
        self.td.root_moves.clear();
        self.td.root_moves.extend_from_slice(legal_moves);
        self.td.root_move_records.clear();
        self.td
            .root_move_records
            .extend(legal_moves.iter().copied().map(RootMove::new));
        let mut bestmove = legal_moves[0];
        let mut pondermove = Move::NULL;
        let mut best_score = -INF_SCORE;
        let mut completed_depth = 0;
        let max_depth = self.cfg.limits.depth.min(MAX_DEPTH - 1);
        let mut mate_watch = MateWatch::default();
        let mut proved_win: Option<ProvedWin> = None;
        // The soft target the last completed iteration set; a root window
        // that fails high this far past it ends the search.
        let mut soft_target_ms = f64::INFINITY;
        let fail_high_target = f64::from(self.cfg.params.tm_fail_high_target) / 10_000.0;
        // This thread's soft-stop vote is cast at most ONCE per search.
        // Without the latch a thread that keeps iterating past its own soft
        // target votes again every iteration and can reach the majority
        // single-handedly — which is the opposite of pooling the decision.
        let mut cast_stop_vote = false;
        let mut iterations =
            compact::window::Iterations::new(self.td.compact_previous_score, 1, self.seeds());
        crate::diag_count!(root_searches);

        for depth in 1..=max_depth {
            // Per iteration, as Stockfish resets `selDepth`: carried across a
            // whole search the number reports a deeper earlier iteration and
            // never falls, which is not what a GUI plots against depth.
            self.td.seldepth = 0;
            // The clock counts this iteration's best-move changes alone.
            self.td.best_move_changes = 0;
            for root_move in &mut self.td.root_move_records {
                root_move.begin_iteration();
            }
            let previous_bestmove = bestmove;
            let mut failed_high_past_target = false;
            // The window opens on this thread's own average, joined by the
            // pool's estimate when the pool has already proven an Exact root
            // score deeper than this thread's progress (fewer re-searches
            // when joining the pool's view). Serial searches have no shared
            // state.
            let pool_average = self
                .shared
                .pool()
                .and_then(|shared| shared.pool_best_exact())
                .filter(|&(pool_depth, _)| pool_depth > infra::to_i32(completed_depth))
                .map(|(_, pool_score)| pool_score);
            let (opening, optimism) = iterations.open(0, pool_average);
            let stm = board.side_to_move() as usize;
            self.td.optimism[stm] = optimism;
            self.td.optimism[1 - stm] = -optimism;
            #[cfg(feature = "probes")]
            {
                self.td.rates.record(compact::probes::Kind::Optimism, true);
                self.td.rates.record(compact::probes::Kind::Window, true);
            }
            let mut window = compact::window::CompactWindow::new(opening, self.seeds());
            // The move an earlier window of this iteration failed high on.
            #[cfg(feature = "diag")]
            let mut iteration_fail_high_move = Move::NULL;

            loop {
                #[cfg(feature = "diag")]
                let window_nodes_before = self.td.nodes;
                let (alpha, beta, search_depth) = window.bounds(infra::to_i32(depth));
                // Each window searches from a clean stack, its cutoff counts
                // included.
                self.td.stack = PlyArray::new(StackEntry::default());
                self.td.root_report_depth = infra::to_i32(depth);
                let score = self.search_root_window(&mut board, search_depth, alpha, beta, poll);
                #[cfg(feature = "diag")]
                let window_nodes = self.td.nodes.saturating_sub(window_nodes_before);
                if self.td.stopped || self.td.quit {
                    #[cfg(feature = "diag")]
                    self.count_interrupted_iteration(bestmove, iteration_fail_high_move);
                    break;
                }
                #[cfg(feature = "diag")]
                if depth >= 4 {
                    crate::diag_add!(asp_search_nodes, window_nodes);
                }
                // A failed window re-centres on the failing score and grows;
                // after `asp_max_fails` failures the failing side opens fully,
                // and each fail high's growth opens both sides in a bounded
                // number of fails.
                if score <= alpha {
                    crate::diag_count!(asp_fail_low);
                    crate::diag_add!(asp_fail_low_nodes, window_nodes);
                    if emit_info && self.shows_bound_lines() {
                        self.send_bounded_info_line(depth, score, RootBound::Upper, bestmove);
                    }
                    window.fail_low(score, self.cfg.params.asp_max_fails);
                    #[cfg(feature = "probes")]
                    self.td.rates.record(compact::probes::Kind::Window, true);
                    continue;
                }
                if score >= beta {
                    crate::diag_count!(asp_fail_high);
                    crate::diag_add!(asp_fail_high_nodes, window_nodes);
                    #[cfg(feature = "diag")]
                    if self.td.pv_len[0] > 0 {
                        iteration_fail_high_move = self.td.pv_table[0][0];
                    }
                    let stops = self.td.thread_id == 0
                        && !self.td.pondering
                        && self.elapsed_ms() >= soft_target_ms * fail_high_target;
                    // The line a search stops on is its report of the move
                    // it plays, so it is printed however short the search.
                    if emit_info && (stops || self.shows_bound_lines()) {
                        self.send_bounded_info_line(depth, score, RootBound::Lower, bestmove);
                    }
                    // Past a multiple of the soft target (twice it by default)
                    // a fail-high ends the search on the move that failed high,
                    // with its lower bound as the score: the re-searches that
                    // would confirm it are what ran won endings to the hard
                    // maximum. At the target itself the stop cut ordinary
                    // fail-highs and cost Elo. It is checked only when a window
                    // returns, so one window that never fails high still runs
                    // to the maximum. Nothing better is discarded: the
                    // bound reaches beta, which lies above the score the window
                    // centred on or, after a fail-low pulled it down, above
                    // every move's bound from that fail-low. The main thread
                    // decides; helpers keep their vote.
                    if stops {
                        crate::diag_count!(tm_fail_high_stop);
                        if self.td.pv_len[0] > 0 {
                            bestmove = self.td.pv_table[0][0];
                            pondermove = if self.td.pv_len[0] > 1 {
                                self.td.pv_table[0][1]
                            } else {
                                Move::NULL
                            };
                        }
                        best_score = score;
                        failed_high_past_target = true;
                        break;
                    }
                    window.fail_high(score, self.cfg.params.asp_max_fails);
                    #[cfg(feature = "probes")]
                    self.td.rates.record(compact::probes::Kind::Window, true);
                    continue;
                }
                best_score = score;
                completed_depth = depth;
                iterations.record_exact(0, score);
                if self.td.pv_len[0] > 0 {
                    bestmove = self.td.pv_table[0][0];
                    pondermove = if self.td.pv_len[0] > 1 {
                        self.td.pv_table[0][1]
                    } else {
                        Move::NULL
                    };
                }
                // A deeper iteration loses a proven win to pruning or a table
                // artefact far more often than it refutes it, and playing its
                // move only delays the mate: keep the proven win's move, score
                // and line until another iteration proves a win.
                if best_score >= MATE_BOUND {
                    crate::diag_count!(iter_mate_win);
                    let pv_len = self.td.pv_len[0].min(MAX_PLY);
                    proved_win = Some(ProvedWin {
                        mv: bestmove,
                        ponder: pondermove,
                        score: best_score,
                        pv: self.td.pv_table[0],
                        pv_len,
                    });
                } else if let Some(proved) = &proved_win {
                    crate::diag_count!(iter_mate_win_lost);
                    #[cfg(feature = "diag")]
                    if bestmove != proved.mv {
                        crate::diag_count!(iter_mate_win_lost_move);
                    }
                    bestmove = proved.mv;
                    pondermove = proved.ponder;
                    best_score = proved.score;
                    self.td.pv_table[0] = proved.pv;
                    self.td.pv_len[0] = proved.pv_len;
                }
                if let Some(root_move) = self
                    .td
                    .root_move_records
                    .iter_mut()
                    .find(|rm| rm.mv == bestmove)
                {
                    root_move.last_best_depth = depth;
                }
                for root_move in &mut self.td.root_move_records {
                    if root_move.last_search_depth == depth {
                        root_move.complete_iteration();
                    }
                }
                break;
            }

            if self.td.stopped || self.td.quit || failed_high_past_target {
                break;
            }
            iterations.end_iteration(best_score, bestmove);

            if emit_info {
                self.send_info(depth, best_score);
            }

            if self.mate_limit_proved(best_score) {
                self.stop_the_pool();
                break;
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
            // `go depth N` on a forced move is still a move request). Under
            // `searchmoves` one move left is the caller's line, not a forced
            // move, so the shortcut counts the whole root or does not fire.
            // A tablebase root keeps its best-ranked group; the shortcut reads
            // the moves it could choose from before that cut.
            let root_choices = match self.shared.syzygy.root.ranked_from {
                0 => legal_moves.len(),
                ranked_from => ranked_from,
            };
            if root_choices == 1 && depth >= 2 && self.cfg.limits.forced_move_shortcut {
                break;
            }

            if bestmove != previous_bestmove {
                crate::diag_count!(root_best_changes);
            }
            crate::diag_count!(root_iterations);

            // The between-iteration stop.
            // movetime mode: no soft stop — count_node (every 2048 nodes) fires at maximum_ms.
            // clock mode: stop when elapsed exceeds the dynamically scaled optimum.
            let elapsed_ms = self.elapsed_ms();
            if elapsed_ms >= self.cfg.limits.maximum_ms {
                break;
            }
            if self.cfg.limits.mate_stop
                && self.td.thread_id == 0
                && mate_watch.completed(bestmove, best_score)
            {
                crate::diag_count!(tm_mate_stop);
                break;
            }
            if !self.cfg.limits.movetime_mode {
                let signals = self.iteration_signals(&iterations, bestmove, best_score);
                let soft = time::soft_stop(&self.cfg.params, &self.cfg.limits, signals);
                soft_target_ms = soft.target_ms;
                #[cfg(feature = "diag")]
                if emit_info && self.cfg.limits.optimum_ms.is_finite() {
                    self.log_soft_stop(depth, signals, soft, elapsed_ms);
                }
                let soft_target = soft.target_ms;
                if self.td.pondering {
                    // While pondering: flag to stop immediately on ponderhit.
                    if elapsed_ms >= soft_target {
                        self.td.stop_on_ponderhit = true;
                    }
                } else if elapsed_ms >= soft_target {
                    // In a parallel search the soft stop is a SYMMETRIC
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
                    // forfeit-safe (measured 0 forfeits). Serial searches have
                    // no shared state and break at their own target.
                    if let Some(shared) = self.shared.pool() {
                        if !cast_stop_vote {
                            cast_stop_vote = true;
                            if shared.vote_to_stop() {
                                shared.request_stop();
                            }
                        }
                        if self.td.thread_id != 0 {
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
        if (self.td.stopped || self.td.quit)
            && self
                .td
                .root_move_records
                .iter()
                .any(|rm| rm.last_search_depth > completed_depth)
        {
            crate::diag_count!(root_interrupted_fallback);
        }

        #[cfg(feature = "diag")]
        if let Some(proved) = &proved_win
            && best_score < MATE_BOUND
        {
            crate::diag_count!(search_mate_win_dropped);
            if bestmove != proved.mv {
                crate::diag_count!(search_mate_win_dropped_move);
            }
        }

        // Dump per-search counters (no-op without `--features diag`).
        // Serial path only — see the reset note above. The parallel
        // dump lives in `search_parallel`, after the helpers are joined.
        if self.shared.threads == 1 {
            crate::diag::dump();
            #[cfg(feature = "pricing")]
            self.pricing_dump();
        }

        if best_score > -INF_SCORE {
            self.td.compact_previous_score = best_score;
        }
        #[cfg(feature = "probes")]
        if emit_info && self.td.thread_id == 0 {
            self.td
                .rates
                .add_fired(compact::probes::Kind::Gravity, self.td.hist.updates);
            self.notice(format_args!("{}", self.td.rates.line()));
            self.notice(format_args!("{}", compact::probes::seed_line()));
        }
        self.root_result(bestmove, pondermove, best_score, completed_depth)
    }

    /// What the clock reads after a completed iteration ended on
    /// `best_score` with `bestmove`: the best move's node share, the score's
    /// fall from the previous search in the game, the two stability counts
    /// the root window keeps, and this iteration's best-move changes. Each is
    /// this thread's own, as in the donor.
    fn iteration_signals(
        &self,
        iterations: &compact::window::Iterations,
        bestmove: Move,
        best_score: i32,
    ) -> IterationSignals {
        IterationSignals {
            node_share: self.effort_share(bestmove),
            score_fall: f64::from(self.td.compact_previous_score - best_score),
            pv_stability: iterations.pv_stability(),
            eval_stability: iterations.eval_stability(),
            best_move_changes: self.td.best_move_changes,
        }
    }

    /// Whether `score`, from a completed iteration, is a mate within `go mate`'s
    /// limit, for either side. Only a completed iteration's score is exact.
    fn mate_limit_proved(&self, score: i32) -> bool {
        let distance = MATE_SCORE - score.abs();
        self.cfg
            .limits
            .mate_plies
            .is_some_and(|plies| distance < infra::to_i32(MAX_PLY) && distance <= plies)
    }

    /// End every thread's search; a serial search has no pool to tell.
    fn stop_the_pool(&self) {
        if let Some(shared) = self.shared.pool() {
            shared.request_stop();
        }
    }

    /// `mv`'s share of this thread's nodes since the search began: the
    /// effort the soft stop reads.
    fn effort_share(&self, mv: Move) -> f64 {
        self.root_record(mv).map_or(0.0, |record| {
            record.nodes as f64 / self.td.nodes.max(1) as f64
        })
    }

    /// One `info string tm` line per completed iteration of a clock search:
    /// the soft stop's inputs and factors, for reading which of them move.
    #[cfg(feature = "diag")]
    #[cold]
    #[inline(never)]
    fn log_soft_stop(
        &self,
        depth: usize,
        signals: IterationSignals,
        soft: time::SoftStop,
        elapsed_ms: f64,
    ) {
        self.td.sink.line(&format!(
            "info string tm depth {depth} share {:.4} fall {:.1} pv {} eval {} changes {} \
             nodes {:.4} trend {:.4} pvstab {:.4} evalstab {:.4} bms {:.4} optimum {:.1} \
             target {:.1} elapsed {elapsed_ms:.1}",
            signals.node_share,
            signals.score_fall,
            signals.pv_stability,
            signals.eval_stability,
            signals.best_move_changes,
            soft.nodes,
            soft.score_trend,
            soft.pv_stability,
            soft.eval_stability,
            soft.best_move_stability,
            self.cfg.limits.optimum_ms,
            soft.target_ms,
        ));
    }
    /// The search's result from the root's last completed iteration, as
    /// both drivers report it.
    fn root_result(
        &self,
        bestmove: Move,
        pondermove: Move,
        score: i32,
        completed_depth: usize,
    ) -> SearchResult {
        let reported = self.reported_line(bestmove, completed_depth, RootBound::Exact);
        SearchResult {
            bestmove,
            pondermove,
            score,
            depth: completed_depth,
            pv: reported.pv,
            seldepth: reported.seldepth,
            nodes: self.td.nodes,
            tb_hits: self.td.tb_hits,
            elapsed_ms: self.cfg.start.elapsed().as_millis(),
            exit: if self.td.quit {
                SearchExit::Quit
            } else {
                SearchExit::Stop
            },
            ponderhit: self.td.ponderhit,
        }
    }

    /// Iterative deepening with more than one principal variation.
    ///
    /// Each depth searches line 1 over the whole root set, then line `i` over
    /// the moves not yet ranked at this depth, by narrowing the root
    /// restriction the kernel already honours for `searchmoves` (it also drops
    /// a TT move outside the restriction). A line's aspiration window centres
    /// on the previous-depth score of the move expected at its rank. After a
    /// line completes, its best move takes that rank and the unranked moves
    /// keep their previous-depth order, since a move that never raised alpha
    /// proved only an upper bound. `bestmove`, the ponder move and the time
    /// manager read line 1 only.
    #[cold]
    #[inline(never)]
    pub(super) fn search_root_multipv<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        mut board: Board,
        root_set: &[Move],
        lines: usize,
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        #[cfg(feature = "donor-eval")]
        self.td.evaluator.set_donor_root(None);
        if self.shared.threads == 1 {
            crate::diag::reset();
        }
        let lines = lines.clamp(1, root_set.len());
        self.td.root_move_records.clear();
        self.td
            .root_move_records
            .extend(root_set.iter().copied().map(RootMove::new));
        // The root set in rank order: moves ranked at this depth first.
        let mut order = root_set.to_vec();
        let mut bestmove = root_set[0];
        let mut pondermove = Move::NULL;
        let mut best_score = -INF_SCORE;
        let mut completed_depth = 0;
        let max_depth = self.cfg.limits.depth.min(MAX_DEPTH - 1);
        let mut mate_watch = MateWatch::default();
        let mut cast_stop_vote = false;
        // The lines of the last depth whose every line completed, best first.
        // A record's score alone is not a line: a move that never raised alpha
        // keeps the bound of its last attempt.
        let mut settled: Vec<ReportedLine> = Vec::new();
        let mut iterations =
            compact::window::Iterations::new(self.td.compact_previous_score, lines, self.seeds());

        for depth in 1..=max_depth {
            // Per iteration, as in the single-line path above.
            self.td.seldepth = 0;
            self.td.best_move_changes = 0;
            for root_move in &mut self.td.root_move_records {
                root_move.begin_iteration();
            }
            // Last depth's lines are the first candidates, in their rank; the
            // rest follow by previous score. Stable, so ties keep their rank.
            order.sort_by_key(|&mv| {
                (
                    settled
                        .iter()
                        .position(|line| line.pv[0] == mv)
                        .unwrap_or(usize::MAX),
                    std::cmp::Reverse(
                        self.root_record(mv)
                            .map_or(-INF_SCORE, |rm| rm.previous_score),
                    ),
                )
            });
            let previous_bestmove = bestmove;
            let mut ranked = 0usize;
            // The line being searched when the search stopped, with the bound
            // of its last aspiration attempt at this depth, if it had one.
            let mut interrupted: Option<ReportedLine> = None;

            for line in 0..lines {
                self.td.root_moves.clear();
                self.td.root_moves.extend_from_slice(&order[line..]);
                self.td.multipv_line = line;
                // Line 1 joins the pool's deeper average, as the one-line
                // search does.
                let pool_average = self
                    .shared
                    .pool()
                    .and_then(|shared| shared.pool_best_exact())
                    .filter(|&(pool_depth, _)| {
                        line == 0 && pool_depth > infra::to_i32(completed_depth)
                    })
                    .map(|(_, pool_score)| pool_score);
                let (opening, optimism) = iterations.open(line, pool_average);
                let stm = board.side_to_move() as usize;
                self.td.optimism[stm] = optimism;
                self.td.optimism[1 - stm] = -optimism;
                let mut window =
                    compact::window::CompactWindow::at_iteration_depth(opening, self.seeds());
                let mut attempt: Option<ReportedLine> = None;
                let completed = loop {
                    let (alpha, beta, search_depth) = window.bounds(infra::to_i32(depth));
                    // Each window searches from a clean stack.
                    self.td.stack = PlyArray::new(StackEntry::default());
                    self.td.root_report_depth = infra::to_i32(depth);
                    let score =
                        self.search_root_window(&mut board, search_depth, alpha, beta, poll);
                    if self.td.stopped || self.td.quit {
                        break false;
                    }
                    if score <= alpha {
                        crate::diag_count!(asp_fail_low);
                        // No move raised alpha: the line's bound is the best
                        // upper bound among the moves this attempt searched.
                        let bounded_move = order[line..]
                            .iter()
                            .copied()
                            .filter_map(|mv| {
                                self.root_record(mv)
                                    .filter(|record| record.last_search_depth == depth)
                                    .map(|record| (record.score, mv))
                            })
                            .max_by_key(|&(score, _)| score)
                            .map_or(order[line], |(_, mv)| mv);
                        attempt = Some(self.reported_line(bounded_move, depth, RootBound::Upper));
                        window.fail_low(score, self.cfg.params.asp_max_fails);
                        continue;
                    }
                    if score >= beta {
                        crate::diag_count!(asp_fail_high);
                        let fail_high_move = if self.td.pv_len[0] > 0 {
                            self.td.pv_table[0][0]
                        } else {
                            order[line]
                        };
                        attempt = Some(self.reported_line(fail_high_move, depth, RootBound::Lower));
                        window.fail_high(score, self.cfg.params.asp_max_fails);
                        continue;
                    }
                    iterations.record_exact(line, score);
                    break true;
                };
                if !completed {
                    interrupted = attempt;
                    break;
                }
                let line_best = if self.td.pv_len[0] > 0 {
                    self.td.pv_table[0][0]
                } else {
                    order[line]
                };
                if let Some(position) = order[line..].iter().position(|&mv| mv == line_best) {
                    let mv = order.remove(line + position);
                    order.insert(line, mv);
                }
                ranked = line + 1;
                // A later line searched in its own window can score above an
                // earlier one; the lines ranked at this depth are kept in score
                // order, so line 1, and the move played, is the best of them.
                order[..ranked].sort_by_key(|&mv| {
                    std::cmp::Reverse(self.root_record(mv).map_or(-INF_SCORE, |rm| rm.score))
                });
                if line == 0 {
                    completed_depth = depth;
                }
                if let Some(record) = self.root_record(order[0]) {
                    best_score = record.score;
                    bestmove = record.mv;
                    pondermove = if record.pv_len > 1 {
                        record.pv[1]
                    } else {
                        Move::NULL
                    };
                }
            }
            self.td.multipv_line = 0;
            if ranked > 0
                && let Some(index) = self.root_record_index(bestmove)
            {
                self.td.root_move_records[index].last_best_depth = depth;
            }

            let stopped = self.td.stopped || self.td.quit;
            if !stopped {
                for root_move in &mut self.td.root_move_records {
                    if root_move.last_search_depth == depth {
                        root_move.complete_iteration();
                    }
                }
            }
            // A stop during line 1 reports nothing new, as the one-line search
            // does; a stop after it reports the lines ranked at this depth, the
            // interrupted line's last bound, and the previous depth for the
            // rest.
            if emit_info && ranked > 0 {
                let mut report: Vec<ReportedLine> = order[..ranked]
                    .iter()
                    .map(|&mv| self.reported_line(mv, depth, RootBound::Exact))
                    .collect();
                if let Some(bounded) = interrupted.take() {
                    report.push(bounded);
                }
                for previous in &settled {
                    if report.len() >= lines {
                        break;
                    }
                    if report.iter().all(|line| line.pv[0] != previous.pv[0]) {
                        report.push(previous.clone());
                    }
                }
                self.send_multipv_info(&report);
            }
            if stopped {
                break;
            }
            iterations.end_iteration(best_score, bestmove);
            settled = order[..ranked]
                .iter()
                .map(|&mv| self.reported_line(mv, depth, RootBound::Exact))
                .collect();

            if self.mate_limit_proved(best_score) {
                self.stop_the_pool();
                break;
            }

            if bestmove != previous_bestmove {
                crate::diag_count!(root_best_changes);
            }
            crate::diag_count!(root_iterations);
            let elapsed_ms = self.elapsed_ms();
            if elapsed_ms >= self.cfg.limits.maximum_ms {
                break;
            }
            // Line 1's mate ends the search as in the one-line path. A root
            // fail-high does not: it would end the depth before the other
            // lines were searched at it.
            if self.cfg.limits.mate_stop
                && self.td.thread_id == 0
                && mate_watch.completed(bestmove, best_score)
            {
                crate::diag_count!(tm_mate_stop);
                break;
            }
            if !self.cfg.limits.movetime_mode {
                let signals = self.iteration_signals(&iterations, bestmove, best_score);
                let soft_target =
                    time::soft_stop(&self.cfg.params, &self.cfg.limits, signals).target_ms;
                if self.td.pondering {
                    if elapsed_ms >= soft_target {
                        self.td.stop_on_ponderhit = true;
                    }
                } else if elapsed_ms >= soft_target {
                    // The pool votes as in the one-line search: every thread
                    // keeps searching until a majority agrees.
                    if let Some(shared) = self.shared.pool() {
                        if !cast_stop_vote {
                            cast_stop_vote = true;
                            if shared.vote_to_stop() {
                                shared.request_stop();
                            }
                        }
                        if shared.stop_state.load(Ordering::Relaxed) == STOP_NONE {
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
        if self.shared.threads == 1 {
            crate::diag::dump();
            #[cfg(feature = "pricing")]
            self.pricing_dump();
        }
        if best_score > -INF_SCORE {
            self.td.compact_previous_score = best_score;
        }
        self.root_result(bestmove, pondermove, best_score, completed_depth)
    }

    pub(super) fn root_record(&self, mv: Move) -> Option<&RootMove> {
        self.root_record_index(mv)
            .map(|index| &self.td.root_move_records[index])
    }
}
