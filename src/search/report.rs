//! The `info` lines a search sends and the scores, bounds and lines they
//! show, the tablebase's display rules included.

use super::*;

impl Searcher {
    /// A line as it would be reported now: the move's record at `depth`, with
    /// its current score and PV.
    pub(super) fn reported_line(&self, mv: Move, depth: usize, bound: RootBound) -> ReportedLine {
        let Some(record) = self.root_record(mv) else {
            return ReportedLine {
                depth,
                seldepth: 0,
                score: -INF_SCORE,
                bound,
                pv: vec![mv],
            };
        };
        ReportedLine {
            depth,
            seldepth: record.seldepth,
            score: record.score,
            bound,
            pv: record.pv[..record.pv_len.clamp(1, MAX_PLY)]
                .iter()
                .copied()
                .take_while(|mv| !mv.is_null())
                .collect(),
        }
    }

    /// The score a line prints: the tables' where they replace the search's,
    /// otherwise the search's own.
    pub(super) fn displayed_score(&self, root_move: Option<&Move>, score: i32) -> i32 {
        // A move the search never scored, after a stop inside the first
        // iteration, prints 0: the marker would read as a mate.
        if score == -INF_SCORE {
            return 0;
        }
        self.tb_root_score(root_move, score).unwrap_or(score)
    }

    /// The bound a line prints: none beside the tables' score, which is exact
    /// whichever way the search's window failed.
    fn displayed_bound(&self, root_move: Option<&Move>, score: i32, bound: RootBound) -> RootBound {
        if self.tb_root_score(root_move, score).is_some() {
            RootBound::Exact
        } else {
            bound
        }
    }

    /// What a line prints: its displayed score and PV. Without a clock the PV
    /// is extended through the tables at every report; under a clock only the
    /// line reported with `bestmove` is, in [`Self::finish_tb_line`].
    fn shown_line(&self, pv: &[Move], score: i32) -> (i32, Vec<Move>) {
        let shown = self.displayed_score(pv.first(), score);
        let mut line = pv.to_vec();
        if self.cfg.limits.tb_extension_ms.is_none()
            && is_tb_band(shown)
            && let Some(root) = self.td.tb_root_board.as_ref()
        {
            let (shown, _) = self.extend_tb_line(root, &mut line, shown, None);
            return (shown, line);
        }
        (shown, line)
    }

    /// One `info` line per reported MultiPV line, numbered from 1.
    pub(super) fn send_multipv_info(&self, lines: &[ReportedLine]) {
        let elapsed_ms = self.cfg.start.elapsed().as_millis();
        let nodes = self.reported_nodes();
        let tb_hits = self.reported_tb_hits();
        let hashfull = self.hashfull();
        for (index, line) in lines.iter().enumerate() {
            if index == 0 {
                self.td
                    .last_reported_move
                    .set(line.pv.first().copied().unwrap_or(Move::NULL));
            }
            let bound = self.displayed_bound(line.pv.first(), line.score, line.bound);
            let (shown, line_pv) = self.shown_line(&line.pv, line.score);
            self.td.sink.report(&InfoReport::Iteration(InfoRecord {
                depth: line.depth.max(1),
                seldepth: line.seldepth,
                multipv: index + 1,
                score: shown,
                bound: bound.into(),
                nodes,
                hashfull,
                tb_hits,
                elapsed_ms,
                pv: &line_pv,
                material: self.cfg.score_material,
                show_wdl: self.cfg.show_wdl,
            }));
        }
    }

    fn reported_nodes(&self) -> u64 {
        self.shared.pool().map_or(self.td.nodes, |shared_state| {
            shared_state.nodes.load(Ordering::Relaxed) + (self.td.nodes & SHARED_NODE_BATCH_MASK)
        })
    }

    /// Whether a failed root window prints its bound line: past the floor
    /// on the search's count, every thread's nodes included.
    pub(super) fn shows_bound_lines(&self) -> bool {
        self.reported_nodes() > self.cfg.bound_lines_from_nodes
    }

    fn reported_tb_hits(&self) -> u64 {
        self.shared.pool().map_or(self.td.tb_hits, |shared_state| {
            shared_state.tb_hits.load(Ordering::Relaxed)
        })
    }

    pub(super) fn elapsed_ms(&self) -> f64 {
        #[cfg(test)]
        if let Some((from_nodes, elapsed_ms)) = self.td.test_clock {
            return if self.td.nodes >= from_nodes {
                elapsed_ms
            } else {
                0.0
            };
        }
        self.cfg.start.elapsed().as_secs_f64() * 1000.0
    }

    pub(super) fn send_info(&self, depth: usize, score: i32) {
        let pv = self.td.pv_table[0][..self.td.pv_len[0].min(MAX_PLY)]
            .iter()
            .copied()
            .filter(|mv| !mv.is_null())
            .collect::<Vec<_>>();
        self.send_info_line(depth, score, &pv);
    }

    fn send_info_line(&self, depth: usize, score: i32, pv: &[Move]) {
        self.send_full_info_line(depth, self.td.seldepth, score, RootBound::Exact, pv);
    }

    /// An aspiration re-search's line: the score is only a bound, so it carries
    /// `lowerbound` or `upperbound`. It is printed before the window closes so
    /// a long iteration is not silent and a stopped one reports its own depth.
    pub(super) fn send_bounded_info_line(
        &self,
        depth: usize,
        score: i32,
        bound: RootBound,
        fallback: Move,
    ) {
        let mut pv: Vec<Move> = self.td.pv_table[0][..self.td.pv_len[0].min(MAX_PLY)]
            .iter()
            .copied()
            .filter(|mv| !mv.is_null())
            .collect();
        if pv.is_empty() && !fallback.is_null() {
            pv.push(fallback);
        }
        if pv.is_empty() {
            return;
        }
        self.send_full_info_line(depth, self.td.seldepth, score, bound, &pv);
    }

    /// One full `info` line, from values the caller owns. The pool uses it to
    /// report the winning thread's line, which is not this thread's state.
    pub(super) fn send_full_info_line(
        &self,
        depth: usize,
        seldepth: usize,
        score: i32,
        bound: RootBound,
        pv: &[Move],
    ) {
        let bound = self.displayed_bound(pv.first(), score, bound);
        let (shown, pv) = self.shown_line(pv, score);
        self.send_shown_info_line(depth, seldepth, shown, bound, &pv);
    }

    /// One full `info` line with the score already in its displayed form.
    pub(super) fn send_shown_info_line(
        &self,
        depth: usize,
        seldepth: usize,
        shown: i32,
        bound: RootBound,
        pv: &[Move],
    ) {
        self.td
            .last_reported_move
            .set(pv.first().copied().unwrap_or(Move::NULL));
        self.td.sink.report(&InfoReport::Iteration(InfoRecord {
            depth,
            seldepth,
            multipv: 1,
            score: shown,
            bound: bound.into(),
            nodes: self.reported_nodes(),
            hashfull: self.hashfull(),
            tb_hits: self.reported_tb_hits(),
            elapsed_ms: self.cfg.start.elapsed().as_millis(),
            pv,
            material: self.cfg.score_material,
            show_wdl: self.cfg.show_wdl,
        }));
    }

    /// Before `bestmove`: print the result's own line unless the last `info`
    /// line already describes its move. That line can name another move: an
    /// iteration stopped after a window failed high or low on a move it never
    /// confirmed leaves that move's bound line last, and the pool's vote can
    /// choose a helper's move no line mentioned.
    pub(crate) fn report_result_line(&self, result: &SearchResult) {
        if self.td.last_reported_move.get() != result.bestmove {
            self.report_line(result);
        }
    }

    /// Print `result`'s line whatever was printed last.
    pub(crate) fn report_line(&self, result: &SearchResult) {
        if result.pv.is_empty() {
            return;
        }
        self.send_full_info_line(
            result.depth.max(1),
            result.seldepth,
            result.score,
            RootBound::Exact,
            &result.pv,
        );
    }
}
