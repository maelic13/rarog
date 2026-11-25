//! The tablebase at the root and below it: the root ranking, the line a
//! decided root extends, and the probes' scores and bounds.

use super::*;

impl Searcher {
    /// Before `bestmove`: when the reported line's displayed score is a
    /// tablebase result, extend it through the tables, take the ponder move
    /// from it and print it. Under a clock the line gets its share of the
    /// box, and nothing when the clock is too short to risk a table read.
    pub(super) fn finish_tb_line(
        &mut self,
        result: &mut SearchResult,
        lines: usize,
        emit_info: bool,
    ) {
        let Some(root) = self.td.tb_root_board.take() else {
            return;
        };
        let Some(&first) = result.pv.first() else {
            return;
        };
        let shown = self.displayed_score(Some(&first), result.score);
        if !is_tb_band(shown) {
            return;
        }
        let deadline = match self.cfg.limits.tb_extension(self.elapsed_ms()) {
            TbExtension::Skip => return,
            TbExtension::Unbounded => None,
            TbExtension::Boxed(box_ms) => {
                let per_line = box_ms / f64::from(u32::try_from(lines).unwrap_or(u32::MAX));
                Some(Instant::now() + Duration::from_secs_f64(per_line / 1000.0))
            }
        };
        let searched = result.pv.clone();
        let (shown, ran_out) = self.extend_tb_line(&root, &mut result.pv, shown, deadline);
        if result.pv.len() > 1 {
            result.pondermove = result.pv[1];
        }
        if emit_info {
            // Without a clock the last iteration already printed this line
            // extended; under a clock it is printed once here.
            if result.pv != searched && deadline.is_some() {
                self.send_shown_info_line(
                    result.depth.max(1),
                    result.seldepth,
                    shown,
                    RootBound::Exact,
                    &result.pv,
                );
            }
            if ran_out {
                self.notice(format_args!(
                    "Syzygy based PV extension requires more time, increase Move Overhead as needed."
                ));
            }
        }
    }

    /// Extend `pv` through the tables. First keep the searched moves while
    /// each keeps the best rank and no arbiter's draw or repetition appears;
    /// then follow the tables to mate, wins ordered by distance and ties
    /// broken toward the opponent's fewest replies (a capture reply counts a
    /// hundred). Returns the score to show, 0 if the line ends in a draw, and
    /// whether `deadline` cut it short.
    pub(super) fn extend_tb_line(
        &self,
        root: &Board,
        pv: &mut Vec<Move>,
        shown: i32,
        deadline: Option<Instant>,
    ) -> (i32, bool) {
        let out_of_time = || deadline.is_some_and(|deadline| Instant::now() >= deadline);
        let Some(&first) = pv.first() else {
            return (shown, false);
        };
        if out_of_time() {
            return (shown, true);
        }
        let rule50 = self.shared.syzygy.fifty_move_rule;
        let mut board = root.clone();
        board.make_move(first);
        let mut kept = 1;
        while kept < pv.len() {
            let Some(mv) = board.legal_move(pv[kept]) else {
                break;
            };
            if let Some(ranking) = syzygy::rank_root_moves(
                &board,
                syzygy::RootRankOptions {
                    use_rule50: rule50,
                    rank_dtz: false,
                    max_pieces: self.tb_probe_limit(),
                },
            ) {
                let best = ranking.moves[0].rank;
                if ranking
                    .moves
                    .iter()
                    .find(|ranked| ranked.mv == mv)
                    .map(|ranked| ranked.rank)
                    != Some(best)
                {
                    break;
                }
                board.make_move(mv);
                if board.is_arbiter_draw(rule50) || board.has_repeated_position() {
                    board.unmake_move(mv);
                    break;
                }
                kept += 1;
                if out_of_time() {
                    break;
                }
            } else {
                // Outside the tables the searched move stands.
                board.make_move(mv);
                kept += 1;
            }
        }
        pv.truncate(kept);
        while pv.len() < 4 * MAX_PLY && !board.is_arbiter_draw(rule50) && !out_of_time() {
            let Some(ranking) = syzygy::rank_root_moves(
                &board,
                syzygy::RootRankOptions {
                    use_rule50: rule50,
                    rank_dtz: true,
                    max_pieces: self.tb_probe_limit(),
                },
            ) else {
                break;
            };
            if !ranking.dtz {
                break;
            }
            let best = ranking.moves[0].rank;
            let mut chosen: Option<(i32, Move)> = None;
            for ranked in ranking
                .moves
                .iter()
                .take_while(|ranked| ranked.rank == best)
            {
                board.make_move(ranked.mv);
                let replies: i32 = board
                    .generate_legal_movelist()
                    .iter()
                    .map(|reply| if reply.is_capture() { 100 } else { 1 })
                    .sum();
                board.unmake_move(ranked.mv);
                if chosen.is_none_or(|(fewest, _)| replies < fewest) {
                    chosen = Some((replies, ranked.mv));
                }
            }
            let Some((_, mv)) = chosen else {
                break;
            };
            board.make_move(mv);
            pv.push(mv);
        }
        let shown = if board.is_arbiter_draw(rule50) {
            0
        } else {
            shown
        };
        (shown, out_of_time())
    }

    /// The root set the tables allow: every candidate of the best rank, in
    /// the candidates' order, or `None` when the tables do not rank this root.
    /// The whole group stays, so the search chooses among result-equal moves;
    /// a move the tables prefer is not a forced move.
    pub(super) fn syzygy_root_moves(
        &mut self,
        board: &Board,
        candidates: &[Move],
    ) -> Option<Vec<Move>> {
        if !self.can_probe_syzygy_root(board)
            || board.can_declare_draw()
            || self.cfg.limits.nodes > 0
        {
            return None;
        }
        let ranking = syzygy::rank_root_moves(
            board,
            syzygy::RootRankOptions {
                use_rule50: self.shared.syzygy.fifty_move_rule,
                rank_dtz: false,
                max_pieces: self.tb_probe_limit(),
            },
        )?;
        self.record_tb_hit();
        let rank_of = |mv: Move| {
            ranking
                .moves
                .iter()
                .find(|ranked| ranked.mv == mv)
                .map(|ranked| ranked.rank)
        };
        let best_rank = candidates.iter().filter_map(|&mv| rank_of(mv)).max()?;
        let root_moves: Vec<Move> = candidates
            .iter()
            .copied()
            .filter(|&mv| rank_of(mv) == Some(best_rank))
            .collect();
        let use_rule50 = self.shared.syzygy.fifty_move_rule;
        self.td.tb_root_scores.clear();
        self.td
            .tb_root_scores
            .extend(ranking.moves.iter().map(|ranked| {
                (
                    ranked.mv,
                    syzygy::display_score(ranked.rank, ranking.dtz, use_rule50),
                )
            }));
        self.apply_tb_root(TbRootDecision {
            ranked_from: candidates.len(),
            search_probes_off: ranking.dtz || best_rank <= 0,
        });
        // DTZ has decided the result, and the search only chooses among
        // result-equal moves, so it may not run past the soft budget: a won
        // ending's fail-high cascade otherwise runs to the hard limit (30 s of
        // a 60 s clock measured on a rook ending).
        if ranking.dtz && self.cfg.limits.optimum_ms.is_finite() {
            self.cfg.limits.maximum_ms = self.cfg.limits.optimum_ms;
        }
        Some(root_moves)
    }

    /// Adopt the root's tablebase decision; every thread of the search calls
    /// it with the main thread's decision after its own reset.
    pub(super) fn apply_tb_root(&mut self, decision: TbRootDecision) {
        self.shared.syzygy.root = decision;
        if decision.search_probes_off {
            self.shared.syzygy.largest = 0;
        }
    }

    /// The tables' display score for a root move where it replaces the
    /// search's: at a tablebase root, unless the search proved a mate.
    pub(super) fn tb_root_score(&self, root_move: Option<&Move>, score: i32) -> Option<i32> {
        if score.abs() >= MATE_BOUND {
            return None;
        }
        root_move
            .and_then(|mv| {
                self.td
                    .tb_root_scores
                    .iter()
                    .find(|(tb_move, _)| tb_move == mv)
            })
            .map(|&(_, display)| display)
    }

    fn syzygy_wdl_score(
        &mut self,
        board: &Board,
        depth: i32,
        ply: usize,
        excluded: Move,
    ) -> Option<i32> {
        if ply == 0 || !excluded.is_null() || !self.can_probe_syzygy(board, depth) {
            return None;
        }
        let probed = syzygy::probe_wdl(board, self.shared.syzygy.fifty_move_rule);
        // The read may have been cold: the main thread checks the GUI and the
        // clock at the next node, as the donor's does, whatever it returned.
        if self.td.thread_id == 0 {
            self.td.poll_next_node = true;
        }
        let wdl = probed?;
        self.record_tb_hit();
        Some(self.score_from_syzygy_wdl(wdl, ply))
    }

    /// The in-search probe as a bound: a tablebase win is a lower bound and a
    /// loss an upper bound, since the search may find a faster mate or a
    /// slower loss than the tables' distance; a draw, cursed and blessed
    /// results included under the rule-50 option, is exact.
    pub(super) fn syzygy_wdl_bound(
        &mut self,
        board: &Board,
        depth: i32,
        ply: usize,
        excluded: Move,
    ) -> Option<(i32, crate::tt::Bound)> {
        let score = self.syzygy_wdl_score(board, depth, ply, excluded)?;
        let bound = match score.signum() {
            1 => crate::tt::Bound::Lower,
            -1 => crate::tt::Bound::Upper,
            _ => crate::tt::Bound::Exact,
        };
        Some((score, bound))
    }

    /// Loaded table size capped by `SyzygyProbeLimit`: the cardinality the
    /// root and PV extension may read, independent of `shared.syzygy.largest`,
    /// which `apply_tb_root` zeroes to stop in-search probes at a DTZ root.
    pub(super) fn tb_probe_limit(&self) -> usize {
        syzygy::largest().min(self.shared.syzygy.probe_limit)
    }

    fn can_probe_syzygy(&self, board: &Board, depth: i32) -> bool {
        let pieces = board.occupied_count() as usize;
        self.shared.syzygy.largest > 0
            && pieces <= self.shared.syzygy.largest
            && (pieces < self.shared.syzygy.largest || depth >= self.shared.syzygy.probe_depth)
            && board.castling().0 == 0
    }

    fn can_probe_syzygy_root(&self, board: &Board) -> bool {
        let limit = self.tb_probe_limit();
        limit > 0 && board.castling().0 == 0 && board.occupied_count() as usize <= limit
    }

    fn score_from_syzygy_wdl(&self, wdl: Wdl, ply: usize) -> i32 {
        match wdl {
            Wdl::Win => TB_VALUE - infra::to_i32(ply),
            Wdl::CursedWin if !self.shared.syzygy.fifty_move_rule => TB_VALUE - infra::to_i32(ply),
            Wdl::Loss => -TB_VALUE + infra::to_i32(ply),
            Wdl::BlessedLoss if !self.shared.syzygy.fifty_move_rule => {
                -TB_VALUE + infra::to_i32(ply)
            }
            Wdl::BlessedLoss | Wdl::Draw | Wdl::CursedWin => 0,
        }
    }
}
