//! Lazy-SMP: the helper pool, per-thread job setup and the parallel result
//! vote.

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, atomic::Ordering, mpsc};
use std::thread::{self, JoinHandle};

use crate::board::{Board, Color, Move};
use crate::eval::INF_SCORE;
use crate::infra;
use crate::search::options::{EngineOptions, SearchLimits};
use crate::tt::{SharedTt, TranspositionTable};

use super::movepick::ScoredMoveList;
use super::shared::{RootBound, STOP_QUIT, STOP_SEARCH, SharedContext, TbRootDecision};
use super::{MAX_PLY, SearchEvent, SearchExit, SearchResult, Searcher, TB_WIN_SCORE};

struct WorkerJob {
    pub root: Board,
    pub(super) root_moves: Arc<[Move]>,
    pub limits: SearchLimits,
    pub(super) engine_options: EngineOptions,
    pub tt: SharedTt,
    pub hash_mb: usize,
    pub(super) root_move_offset: usize,
    /// Helper index (1-based); seeds the per-thread reduction jitter.
    pub(super) thread_id: usize,
    /// The main thread's tablebase decision for this root.
    pub(super) tb_root: TbRootDecision,
    pub(super) shared_state: Arc<SharedContext>,
    result_tx: Sender<SearchResult>,
}

enum WorkerMessage {
    // Boxed — WorkerJob is large while the other variants are unit, so
    // every queued message paid the largest size. This is a per-search thread
    // handoff (not a hot path), so the indirection is free here.
    Search(Box<WorkerJob>),
    NewGame,
    Shutdown,
}

struct SearchWorkerHandle {
    sender: Sender<WorkerMessage>,
    /// One message per finished start-up and `NewGame`.
    acks: Receiver<()>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Default)]
pub(super) struct WorkerPool {
    workers: Vec<SearchWorkerHandle>,
}

impl WorkerPool {
    /// Grow or shrink the pool to `helper_count` threads and return how many
    /// it holds, which is fewer when the operating system refuses a thread.
    /// New helpers have finished starting on return.
    pub(super) fn set_helper_count(&mut self, helper_count: usize) -> usize {
        self.set_helper_count_with(helper_count, spawn_search_worker)
    }

    fn set_helper_count_with(
        &mut self,
        helper_count: usize,
        spawn: impl Fn(usize) -> Option<SearchWorkerHandle>,
    ) -> usize {
        while self.workers.len() > helper_count {
            if let Some(mut worker) = self.workers.pop() {
                let _ = worker.sender.send(WorkerMessage::Shutdown);
                if let Some(handle) = worker.handle.take() {
                    let _ = handle.join();
                }
            }
        }
        let started = self.workers.len();
        while self.workers.len() < helper_count {
            if let Some(worker) = spawn(self.workers.len()) {
                self.workers.push(worker);
            } else {
                break;
            }
        }
        // After every spawn, so the new helpers start up together.
        wait_for_acks(&self.workers[started..]);
        self.workers.len()
    }

    pub(crate) fn start_new_game(&self) {
        for worker in &self.workers {
            let _ = worker.sender.send(WorkerMessage::NewGame);
        }
    }

    pub(crate) fn finish_new_game(&self) {
        wait_for_acks(&self.workers);
    }

    fn send_search(&self, index: usize, job: WorkerJob) -> bool {
        self.workers.get(index).is_some_and(|worker| {
            worker
                .sender
                .send(WorkerMessage::Search(Box::new(job)))
                .is_ok()
        })
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.set_helper_count(0);
    }
}

/// One acknowledgement from each of `workers`. A helper that died has dropped
/// its sender, so the wait ends with it and `isready` cannot hang.
fn wait_for_acks(workers: &[SearchWorkerHandle]) {
    for worker in workers {
        let _ = worker.acks.recv();
    }
}

fn spawn_search_worker(index: usize) -> Option<SearchWorkerHandle> {
    let (sender, receiver) = mpsc::channel();
    let (ack, acks) = mpsc::channel();
    let handle = thread::Builder::new()
        .name(format!("rarog-search-{index}"))
        .stack_size(infra::THREAD_STACK_SIZE)
        .spawn(move || {
            // Ready before the first search, which runs on the clock: the
            // helper's history and cache pages are touched here, not by that
            // search.
            let mut worker = Searcher::helper();
            worker.reset_worker_state_for_new_game();
            let _ = ack.send(());
            while let Ok(message) = receiver.recv() {
                match message {
                    WorkerMessage::Search(job) => {
                        let result_tx = job.result_tx.clone();
                        let shared_state = Arc::clone(&job.shared_state);
                        let mut helper_poll =
                            || match shared_state.stop_state.load(Ordering::Relaxed) {
                                STOP_QUIT => SearchEvent::Quit,
                                STOP_SEARCH => SearchEvent::Stop,
                                _ if shared_state.ponderhit.load(Ordering::Relaxed) => {
                                    SearchEvent::PonderHit
                                }
                                _ => SearchEvent::None,
                            };
                        let result = worker.run_worker_job(*job, &mut helper_poll);
                        let _ = result_tx.send(result);
                    }
                    WorkerMessage::NewGame => {
                        worker.reset_worker_state_for_new_game();
                        let _ = ack.send(());
                    }
                    WorkerMessage::Shutdown => break,
                }
            }
        })
        .ok()?;
    Some(SearchWorkerHandle {
        sender,
        acks,
        handle: Some(handle),
    })
}

/// The voted result and the index of the thread that produced it; index 0 is
/// the main thread, whose line has already been printed.
fn select_parallel_result(
    results: &[SearchResult],
    root_moves: &[Move],
) -> Option<(usize, SearchResult)> {
    let root_results = results
        .iter()
        .enumerate()
        .filter(|(_, result)| is_root_result(result, root_moves))
        .collect::<Vec<_>>();
    let min_score = root_results.iter().map(|(_, result)| result.score).min()?;

    let mut votes: Vec<(Move, i64)> = Vec::new();
    for (_, result) in &root_results {
        let vote_value = parallel_vote_value(result, min_score);
        if let Some(vote) = votes.iter_mut().find(|(mv, _)| *mv == result.bestmove) {
            vote.1 += vote_value;
        } else {
            votes.push((result.bestmove, vote_value));
        }
    }

    root_results
        .into_iter()
        .max_by(|(left_index, left), (right_index, right)| {
            let left_vote = vote_for_move(&votes, left.bestmove);
            let right_vote = vote_for_move(&votes, right.bestmove);
            parallel_result_key(left, left_vote, *left_index == 0).cmp(&parallel_result_key(
                right,
                right_vote,
                *right_index == 0,
            ))
        })
        .map(|(index, result)| (index, result.clone()))
}

/// The result that answers for the pool and the index of its thread; index 0
/// is the main thread.
///
/// With more than one line every thread searches them all, and the main
/// thread owns the reported lines and the move, so they stay one thread's
/// consistent list. Under a depth
/// limit only the main thread searches to that depth, and a helper's depth says
/// only how far it got before the main thread finished. Anything else votes,
/// `go mate` included.
fn pool_answer(
    results: &[SearchResult],
    root_moves: &[Move],
    lines: usize,
    limits: &SearchLimits,
) -> Option<(usize, SearchResult)> {
    if lines > 1 || limits.depth.is_some() {
        return results.first().cloned().map(|result| (0, result));
    }
    select_parallel_result(results, root_moves)
}

fn is_root_result(result: &SearchResult, root_moves: &[Move]) -> bool {
    result.depth > 0 && root_moves.contains(&result.bestmove)
}

fn parallel_vote_value(result: &SearchResult, min_score: i32) -> i64 {
    let score_weight = (result.score as i64 - min_score as i64 + 14).max(1);
    score_weight * i64::try_from(result.depth.max(1)).unwrap_or(i64::MAX)
}

fn vote_for_move(votes: &[(Move, i64)], mv: Move) -> i64 {
    votes
        .iter()
        .find_map(|(vote_move, vote)| (*vote_move == mv).then_some(*vote))
        .unwrap_or(0)
}

fn parallel_result_key(
    result: &SearchResult,
    vote: i64,
    main_thread: bool,
) -> (i32, i32, i64, bool, usize, i32, bool) {
    let decisive_rank = if result.score >= TB_WIN_SCORE {
        2
    } else if result.score <= -TB_WIN_SCORE {
        0
    } else {
        1
    };
    // Among decisive results the shortest mate or conversion decides: each is
    // proved, so a vote for a longer one would only delay the end.
    let decisive_length = if decisive_rank == 1 {
        0
    } else {
        result.score.abs()
    };
    (
        decisive_rank,
        decisive_length,
        vote,
        !result.pondermove.is_null(),
        result.depth,
        result.score,
        main_thread,
    )
}

impl Searcher {
    fn reset_worker_state_for_new_game(&mut self) {
        self.clear_history();
        self.td.evaluator.clear_pawn_table();
        self.td.previous_search_score = None;
    }

    fn run_worker_job(
        &mut self,
        job: WorkerJob,
        poll: &mut dyn FnMut() -> SearchEvent,
    ) -> SearchResult {
        let idle_table =
            std::mem::replace(&mut self.shared.tt, TranspositionTable::attached(job.tt));
        self.shared.hash_mb = job.hash_mb;
        self.shared.join_pool(Arc::clone(&job.shared_state));
        self.td.root_move_offset = job.root_move_offset;
        self.td.thread_id = job.thread_id;
        let result = self.search_worker(
            job.root,
            &job.limits,
            &job.engine_options,
            job.root_moves.as_ref(),
            job.tb_root,
            poll,
        );
        self.shared.leave_pool();
        // The handle goes before the result does: the main thread holds the
        // table alone once it has every result, so a `Hash` change that
        // follows frees the old table rather than leaving it to an idle helper.
        self.shared.tt = idle_table;
        result
    }

    fn search_worker<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        root: Board,
        limits: &SearchLimits,
        engine_options: &EngineOptions,
        legal_moves: &[Move],
        tb_root: TbRootDecision,
        poll: &mut P,
    ) -> SearchResult {
        let game_ply = 2 * root.fullmove().saturating_sub(1) as u32
            + (root.side_to_move() == Color::Black) as u32;
        // Helpers must NOT inherit the main thread's fixed depth.
        //
        // Under a clock this is invisible — every thread runs until the main
        // thread's time manager stops the pool. But under `go depth N` a helper
        // that reaches N returns and then sits idle for the rest of the search,
        // contributing nothing while the main thread finishes. Helpers exist to
        // widen the shared TT, so they should keep going until stopped; the
        // main thread alone owns the depth contract and the reported result.
        let mut helper_limits = limits.clone();
        helper_limits.depth = None;
        self.reset_search_state(
            &helper_limits,
            engine_options,
            root.side_to_move(),
            game_ply,
            false,
        );
        self.apply_tb_root(tb_root);
        let lines = engine_options.multi_pv.clamp(1, legal_moves.len());
        if lines > 1 {
            self.search_root_multipv(root, legal_moves, lines, false, poll)
        } else {
            self.search_root(root, legal_moves, false, poll)
        }
    }

    #[cold]
    #[inline(never)]
    pub(super) fn search_parallel(
        &mut self,
        root: Board,
        root_moves: &[Move],
        limits: &SearchLimits,
        engine_options: EngineOptions,
        threads: usize,
        emit_info: bool,
        poll: &mut dyn FnMut() -> SearchEvent,
    ) -> SearchResult {
        // Reset BEFORE any helper exists, so nothing already counted
        // gets wiped by a late-starting thread.
        crate::diag::reset();
        let table = self.shared.tt.make_shared(self.shared.hash_mb);
        let helper_count = threads.saturating_sub(1);
        let root_len = root_moves.len();
        let shared_state = Arc::new(SharedContext::new(self.td.tb_hits, root_len, threads));
        let lines = engine_options.multi_pv.clamp(1, root_moves.len());
        let mut worker_engine_options = engine_options;
        worker_engine_options.threads = 1;
        self.worker_pool.set_helper_count(helper_count);
        let root_moves_shared: Arc<[Move]> = root_moves.to_vec().into();

        let (result_tx, result_rx) = mpsc::channel();
        let mut launched_helpers = 0usize;
        for index in 0..helper_count {
            // Stagger each helper's starting point in the root list so
            // the pool does not pile onto move 1. Removing the stagger measured
            // −3.31 ± 10.62 over 1,682 games: unresolved, leaning toward the
            // rotation earning its keep.
            let offset = if threads <= root_len {
                ((index + 1) * root_len / threads).max(1) % root_len
            } else {
                (index + 1) % root_len
            };
            let job = WorkerJob {
                root: root.clone(),
                root_moves: Arc::clone(&root_moves_shared),
                limits: limits.clone(),
                engine_options: worker_engine_options.clone(),
                tt: table.clone(),
                hash_mb: self.shared.hash_mb,
                root_move_offset: offset,
                thread_id: index + 1,
                tb_root: self.shared.syzygy.root,
                shared_state: Arc::clone(&shared_state),
                result_tx: result_tx.clone(),
            };
            if self.worker_pool.send_search(index, job) {
                launched_helpers += 1;
            }
        }
        drop(result_tx);
        drop(table);

        self.td.root_move_offset = 0;
        self.td.thread_id = 0;
        self.shared.join_pool(Arc::clone(&shared_state));
        let root_for_ponder = root.clone();
        let mut main_poll = || match shared_state.stop_state.load(Ordering::Relaxed) {
            STOP_QUIT => SearchEvent::Quit,
            STOP_SEARCH => SearchEvent::Stop,
            _ => match poll() {
                SearchEvent::Quit => {
                    shared_state.request_quit();
                    SearchEvent::Quit
                }
                SearchEvent::Stop => {
                    shared_state.request_stop();
                    SearchEvent::Stop
                }
                SearchEvent::PonderHit => {
                    shared_state.ponderhit.store(true, Ordering::Relaxed);
                    SearchEvent::PonderHit
                }
                SearchEvent::None => SearchEvent::None,
            },
        };
        let main_poll: &mut dyn FnMut() -> SearchEvent = &mut main_poll;
        let main_result = if lines > 1 {
            self.search_root_multipv(root, root_moves, lines, emit_info, main_poll)
        } else {
            self.search_root(root, root_moves, emit_info, main_poll)
        };
        shared_state.request_stop();

        let mut helper_results = Vec::with_capacity(launched_helpers + 1);
        helper_results.push(main_result);
        for _ in 0..launched_helpers {
            if let Ok(result) = result_rx.recv() {
                helper_results.push(result);
            }
        }
        self.td.root_move_offset = 0;

        // Every helper has been joined above, so the counters are now
        // complete and this is the one legitimate dump point for a parallel go.
        crate::diag::dump();

        let total_nodes = helper_results.iter().map(|result| result.nodes).sum();
        let total_tb_hits = shared_state.tb_hits.load(Ordering::Relaxed);
        let quit = shared_state.stop_state.load(Ordering::Relaxed) == STOP_QUIT
            || helper_results
                .iter()
                .any(|result| result.exit == SearchExit::Quit);
        let answer = pool_answer(&helper_results, root_moves, lines, limits);
        let answering_thread = answer.as_ref().map_or(0, |(index, _)| *index);
        let mut best = answer.map(|(_, result)| result).unwrap_or(SearchResult {
            bestmove: root_moves[0],
            pondermove: Move::NULL,
            score: -INF_SCORE,
            depth: 0,
            pv: Vec::new(),
            seldepth: 0,
            nodes: 0,
            tb_hits: 0,
            elapsed_ms: self.cfg.start.elapsed().as_millis(),
            exit: SearchExit::Stop,
            ponderhit: self.td.ponderhit,
        });
        self.td.nodes = total_nodes;
        self.td.tb_hits = total_tb_hits;
        self.td.quit = quit;
        self.td.stopped = true;
        best.nodes = total_nodes;
        best.tb_hits = total_tb_hits;
        best.elapsed_ms = self.cfg.start.elapsed().as_millis();
        if best.pondermove.is_null() {
            best.pondermove = self.ponder_from_tt(&root_for_ponder, best.bestmove);
        }
        best.ponderhit = self.td.ponderhit || helper_results.iter().any(|result| result.ponderhit);
        best.exit = if quit {
            SearchExit::Quit
        } else {
            SearchExit::Stop
        };
        if emit_info {
            self.report_pool_answer(answering_thread, &best);
        }
        self.finish_tb_line(&mut best, lines, emit_info);
        self.shared.leave_pool();
        best
    }

    /// Only the main thread prints, so a helper's answer is printed here even
    /// for the move the main thread's last line names: its score and line are
    /// the helper's, a mate the main thread never saw among them. The main
    /// thread's own answer prints only if its last line names another move.
    fn report_pool_answer(&self, answering_thread: usize, result: &SearchResult) {
        if answering_thread == 0 {
            self.report_result_line(result);
        } else {
            self.report_line(result);
        }
    }

    /// A root move's position in the search's full root list: the index of
    /// its persistent record and of its slot in the pool's root scores. It
    /// stays fixed while `td.root_moves` narrows for later MultiPV lines, and
    /// every thread builds its records from the same list.
    #[inline]
    pub(super) fn root_record_index(&self, mv: Move) -> Option<usize> {
        self.td
            .root_move_records
            .iter()
            .position(|record| record.mv == mv)
    }

    /// Fold the pool's per-root-move knowledge into this thread's root
    /// ordering.
    ///
    /// A move that another thread has already searched deeper gets lifted
    /// above the local heuristic ordering, ranked by (depth, score). The
    /// shared TT already carries much of this implicitly, but root entries
    /// are overwritten under pressure while these slots are not, so the
    /// explicit channel survives exactly the case it is needed in.
    pub(super) fn apply_shared_root_scores(&self, scored: &mut ScoredMoveList) {
        let Some(shared) = self.shared.pool() else {
            return;
        };
        for entry in scored.as_mut_slice() {
            let Some(index) = self.root_record_index(entry.mv) else {
                continue;
            };
            let Some((depth, score, bound)) = shared.root_score(index) else {
                continue;
            };
            // Rank above every locally-scored quiet but below the TT move, so
            // the pool refines the ordering rather than overriding the one
            // move we already know is best here. An Upper-bound entry (a
            // proven fail-low) is demoted by half a depth step — the pool has
            // evidence AGAINST the move, so it should sort below same-depth
            // moves whose scores are trustworthy.
            let upper_penalty = if bound == RootBound::Upper { 2_048 } else { 0 };
            entry.score = 25_000_000 + depth * 4_096 + score.clamp(-30_000, 30_000) - upper_penalty;
        }
    }

    /// Publish and retain one completed root-move visit outside the hot node
    /// kernel: root-only branches placed in `negamax` measured about -0.8%
    /// best-of NPS despite running only at the root.
    #[cold]
    #[inline(never)]
    pub(super) fn record_root_move_search(
        &mut self,
        mv: Move,
        depth: i32,
        score: i32,
        alpha: i32,
        beta: i32,
        nodes: u64,
    ) {
        let Some(index) = self.root_record_index(mv) else {
            // Direct diagnostic/unit calls may enter root negamax without the
            // normal `search_root` initialization. Search remains valid; there
            // is simply no persistent table to update on that path.
            return;
        };
        let bound = if score >= beta {
            RootBound::Lower
        } else if score > alpha {
            RootBound::Exact
        } else {
            RootBound::Upper
        };
        if let Some(shared) = self.shared.pool() {
            shared.publish_root_score(index, depth, score, bound);
        }

        let root_move = &mut self.td.root_move_records[index];
        // This is the cumulative search-wide seldepth at the time the move
        // completes (the same low-cost shape used by Basilisk), so a later move
        // may inherit a deeper earlier move's maximum. Exact per-move tracking
        // required extra branches in every recursive move loop and measured a
        // real speed loss, so consumers treat this field as a conservative max.
        root_move.record_search(
            infra::to_usize(depth),
            score,
            nodes,
            self.td.seldepth,
            bound,
        );
        if score > alpha {
            let child_len = self.td.pv_len[1].clamp(1, MAX_PLY);
            root_move.pv[0] = mv;
            root_move.pv[1..child_len].copy_from_slice(&self.td.pv_table[1][1..child_len]);
            root_move.pv_len = child_len;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::eval::MATE_SCORE;
    use crate::search::options::{HashRequest, SearchOptions};
    use crate::search::{InfoReport, InfoSink};
    use crate::uci::info::report_line;

    const SELF_STOP_LIMIT: Duration = Duration::from_secs(30);

    struct Recorder(Arc<Mutex<Vec<String>>>);

    impl InfoSink for Recorder {
        fn line(&self, line: &str) {
            self.0.lock().unwrap().push(line.to_string());
        }
        fn report(&self, report: &InfoReport<'_>) {
            self.line(&report_line(report));
        }
    }

    #[test]
    fn parallel_result_selection_uses_weighted_helper_votes() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let g1f3 = Move::from_uci("g1f3").expect("valid move");
        let results = vec![
            test_search_result(e2e4, 20, 5),
            test_search_result(d2d4, 18, 5),
            test_search_result(d2d4, 16, 5),
        ];

        let (index, selected) =
            select_parallel_result(&results, &[e2e4, d2d4, g1f3]).expect("selected result");

        assert_eq!(selected.bestmove, d2d4);
        assert_ne!(
            index, 0,
            "the winner here is a helper, and the caller must know"
        );
    }

    #[test]
    fn parallel_result_selection_prefers_decisive_win() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let results = vec![
            test_search_result(e2e4, 900, 12),
            test_search_result(d2d4, TB_WIN_SCORE, 4),
        ];

        let (index, selected) =
            select_parallel_result(&results, &[e2e4, d2d4]).expect("selected result");

        assert_eq!(selected.bestmove, d2d4);
        assert_eq!(index, 1);
    }

    /// A helper that takes `delay` over its start-up and each `NewGame`,
    /// counting each one it finishes.
    fn slow_worker(delay: Duration, finished: &Arc<AtomicUsize>) -> SearchWorkerHandle {
        let finished = Arc::clone(finished);
        let (sender, receiver) = mpsc::channel();
        let (ack, acks) = mpsc::channel();
        let handle = thread::spawn(move || {
            let finish = || {
                thread::sleep(delay);
                finished.fetch_add(1, Ordering::SeqCst);
                let _ = ack.send(());
            };
            finish();
            while let Ok(message) = receiver.recv() {
                match message {
                    WorkerMessage::NewGame => finish(),
                    WorkerMessage::Shutdown => break,
                    WorkerMessage::Search(_) => {}
                }
            }
        });
        SearchWorkerHandle {
            sender,
            acks,
            handle: Some(handle),
        }
    }

    /// `readyok` follows these calls, so a slow helper must hold them back.
    #[test]
    fn the_pool_waits_for_slow_helpers_to_start_and_to_clear() {
        const DELAY: Duration = Duration::from_millis(100);
        let finished = Arc::new(AtomicUsize::new(0));
        let mut pool = WorkerPool::default();

        let helpers = pool.set_helper_count_with(3, |_| Some(slow_worker(DELAY, &finished)));
        assert_eq!(helpers, 3);
        assert_eq!(finished.load(Ordering::SeqCst), 3, "started");

        pool.start_new_game();
        pool.finish_new_game();
        assert_eq!(finished.load(Ordering::SeqCst), 6, "cleared");

        pool.set_helper_count_with(4, |_| Some(slow_worker(DELAY, &finished)));
        assert_eq!(
            finished.load(Ordering::SeqCst),
            7,
            "only the new helper is waited for"
        );
    }

    #[test]
    fn a_helper_that_died_does_not_hang_the_pool() {
        let dead = || {
            let (sender, _) = mpsc::channel();
            let (_, acks) = mpsc::channel();
            Some(SearchWorkerHandle {
                sender,
                acks,
                handle: None,
            })
        };
        let mut pool = WorkerPool::default();

        assert_eq!(pool.set_helper_count_with(2, |_| dead()), 2);
        pool.start_new_game();
        pool.finish_new_game();
    }

    #[test]
    fn a_helper_keeps_only_the_smallest_table() {
        assert_eq!(Searcher::helper().shared.tt.allocated_bytes(), 1 << 20);
    }

    /// Three threads on a longer mate outvote the one that found the shortest,
    /// which decides all the same; so does the nearest mate when all are lost.
    #[test]
    fn the_shortest_decisive_result_wins_whatever_the_vote_says() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        for sign in [1, -1] {
            let results = vec![
                test_search_result(d2d4, sign * (MATE_SCORE - 9), 12),
                test_search_result(d2d4, sign * (MATE_SCORE - 9), 12),
                test_search_result(d2d4, sign * (MATE_SCORE - 9), 12),
                test_search_result(e2e4, sign * (MATE_SCORE - 3), 8),
            ];

            let (index, selected) =
                select_parallel_result(&results, &[e2e4, d2d4]).expect("selected result");

            assert_eq!(index, 3, "sign {sign}");
            assert_eq!(selected.bestmove, e2e4);
        }
    }

    /// The winner's own line is what the pool reports, so a result carries the
    /// PV and seldepth of the thread that produced it.
    #[test]
    fn a_helper_result_carries_the_line_the_pool_would_report() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let mut helper = test_search_result(d2d4, 30, 9);
        helper.pv = vec![d2d4, e2e4];
        helper.seldepth = 17;
        let results = vec![test_search_result(e2e4, 10, 9), helper];

        let (index, selected) =
            select_parallel_result(&results, &[e2e4, d2d4]).expect("selected result");

        assert_eq!(index, 1);
        assert_eq!(selected.pv, vec![d2d4, e2e4]);
        assert_eq!(selected.seldepth, 17);
    }

    fn depth_limit(depth: u32) -> SearchLimits {
        SearchLimits {
            depth: Some(depth),
            ..SearchLimits::default()
        }
    }

    /// Helpers ahead of the main thread and behind it both outvote it, and
    /// under a depth limit neither answers.
    #[test]
    fn a_depth_limit_is_answered_by_the_main_thread_whatever_the_vote_says() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let root_moves = [e2e4, d2d4];
        for helper_depth in [3, 7] {
            let results = vec![
                test_search_result(e2e4, 10, 6),
                test_search_result(d2d4, 300, helper_depth),
                test_search_result(d2d4, 300, helper_depth),
                test_search_result(d2d4, 300, helper_depth),
            ];
            let (voted, _) =
                pool_answer(&results, &root_moves, 1, &SearchLimits::default()).expect("answered");
            assert_ne!(voted, 0, "on the clock the vote goes to a helper");

            let (index, selected) =
                pool_answer(&results, &root_moves, 1, &depth_limit(6)).expect("answered");
            assert_eq!(index, 0);
            assert_eq!((selected.bestmove, selected.depth), (e2e4, 6));
        }
    }

    #[test]
    fn more_than_one_line_is_answered_by_the_main_thread() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let results = vec![
            test_search_result(e2e4, 10, 6),
            test_search_result(d2d4, 300, 9),
        ];

        let (index, _) =
            pool_answer(&results, &[e2e4, d2d4], 2, &SearchLimits::default()).expect("answered");

        assert_eq!(index, 0);
    }

    /// The depth limit decides alone, as in Stockfish: a main thread stopped
    /// inside its first iteration still answers, with the first root move.
    #[test]
    fn a_depth_limit_is_the_main_threads_even_before_its_first_iteration_ends() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let results = vec![
            test_search_result(e2e4, -INF_SCORE, 0),
            test_search_result(d2d4, 20, 2),
        ];

        let (index, selected) =
            pool_answer(&results, &[e2e4, d2d4], 1, &depth_limit(6)).expect("answered");

        assert_eq!(index, 0);
        assert_eq!(selected.bestmove, e2e4);
    }

    /// `go mate` is not a depth limit: a mate only a helper found is the answer.
    #[test]
    fn a_mate_search_is_answered_by_the_vote() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let results = vec![
            test_search_result(e2e4, 40, 5),
            test_search_result(d2d4, TB_WIN_SCORE, 4),
        ];
        let limits = SearchLimits {
            mate: Some(3),
            ..SearchLimits::default()
        };

        let (index, selected) = pool_answer(&results, &[e2e4, d2d4], 1, &limits).expect("answered");

        assert_eq!(index, 1);
        assert_eq!(selected.bestmove, d2d4);
    }

    const MATE_IN_2: &str = "k7/8/2K5/8/8/8/8/7R w - - 0 1";
    const MATED_IN_1: &str = "7k/8/5K2/8/8/8/6Q1/8 b - - 0 1";

    /// A search meant to end itself: it stops here instead of hanging the
    /// suite, and the elapsed time says which it was.
    fn within_a_limit() -> impl FnMut() -> SearchEvent {
        let start = Instant::now();
        move || {
            if start.elapsed() > SELF_STOP_LIMIT {
                SearchEvent::Stop
            } else {
                SearchEvent::None
            }
        }
    }

    fn mate_search(fen: &str, mate: u32, threads: usize) -> SearchResult {
        let mut options = SearchOptions::default();
        options.engine.hash_mb = 1;
        options.engine.threads = threads;
        options.limits.mate = Some(mate);
        let mut searcher = Searcher::default();
        searcher.configure(&options.engine, HashRequest::Sent);
        let board = Board::from_fen(fen).expect("valid FEN");
        searcher.search(board, &options, false, within_a_limit())
    }

    #[test]
    fn a_mate_search_ends_itself_on_the_proof_for_either_side() {
        for threads in [1, 4] {
            let start = Instant::now();
            assert_eq!(
                mate_search(MATE_IN_2, 2, threads).score,
                MATE_SCORE - 3,
                "Threads {threads}"
            );
            assert_eq!(
                mate_search(MATED_IN_1, 1, threads).score,
                2 - MATE_SCORE,
                "Threads {threads}"
            );
            assert!(start.elapsed() < SELF_STOP_LIMIT, "Threads {threads}");
        }
    }

    /// Either side's mate within the limit ends the search at once. The same
    /// mate beyond a tighter limit ends nothing: the search goes on past it,
    /// here as far as a search goes in so small a position.
    #[test]
    fn only_a_mate_within_the_limit_ends_the_search() {
        let won = mate_search(MATE_IN_2, 2, 1);
        let lost = mate_search(MATED_IN_1, 1, 1);
        let beyond = mate_search(MATE_IN_2, 1, 1);
        assert!(won.depth < 10, "{}", won.depth);
        assert!(lost.depth < 10, "{}", lost.depth);
        assert!(beyond.depth > 50, "{}", beyond.depth);
        assert_eq!(beyond.score, MATE_SCORE - 3);
    }

    /// A helper's proof ends the pool's search: the main thread need not find
    /// the mate itself.
    #[test]
    fn a_helpers_proof_stops_the_pool() {
        let board = Board::from_fen(MATE_IN_2).expect("valid FEN");
        let moves = board.generate_legal_moves();
        let context = Arc::new(SharedContext::new(0, moves.len(), 2));
        let mut helper = Searcher::default();
        helper.shared.join_pool(Arc::clone(&context));
        helper.td.thread_id = 1;
        let limits = SearchLimits {
            mate: Some(2),
            ..SearchLimits::default()
        };

        let result = helper.search_worker(
            board,
            &limits,
            &EngineOptions::default(),
            &moves,
            TbRootDecision::default(),
            &mut within_a_limit(),
        );

        assert_eq!(result.score, MATE_SCORE - 3);
        assert_eq!(context.stop_state.load(Ordering::Relaxed), STOP_SEARCH);
    }

    /// A mate search ends on a completed iteration, where each of the helper's
    /// lines holds an exact score at that depth; one line would hold one.
    #[test]
    fn a_helper_searches_every_line() {
        let board = Board::from_fen(MATE_IN_2).expect("valid FEN");
        let moves = board.generate_legal_moves();
        let context = Arc::new(SharedContext::new(0, moves.len(), 2));
        let mut helper = Searcher::default();
        helper.shared.join_pool(Arc::clone(&context));
        helper.td.thread_id = 1;
        let limits = SearchLimits {
            mate: Some(2),
            ..SearchLimits::default()
        };
        let engine = EngineOptions {
            multi_pv: 3,
            ..EngineOptions::default()
        };

        let result = helper.search_worker(
            board,
            &limits,
            &engine,
            &moves,
            TbRootDecision::default(),
            &mut within_a_limit(),
        );

        let depth = i32::try_from(result.depth).expect("a small depth");
        let exact = (0..moves.len())
            .filter(|&index| {
                matches!(
                    context.root_score(index),
                    Some((at, _, RootBound::Exact)) if at == depth
                )
            })
            .count();
        assert_eq!(exact, 3);
    }

    #[test]
    fn a_helpers_answer_is_printed_even_for_the_move_already_shown() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let lines = Arc::new(Mutex::new(Vec::new()));
        let searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
        searcher.td.last_reported_move.set(e2e4);
        let answer = test_search_result(e2e4, MATE_SCORE - 3, 5);

        searcher.report_pool_answer(0, &answer);
        assert!(lines.lock().unwrap().is_empty(), "the main thread said so");

        searcher.report_pool_answer(2, &answer);
        let printed = lines.lock().unwrap();
        assert_eq!(printed.len(), 1);
        assert!(printed[0].contains("score mate 2"), "{}", printed[0]);
    }

    /// The whole path at four threads: the result and the last line printed
    /// both name the depth asked for, whatever the helpers reached. Helpers
    /// race, so this catches a wrong answer only some of the time; the tests
    /// on `pool_answer` above are the ones that always do.
    #[test]
    fn go_depth_at_four_threads_reports_that_depth() {
        const DEPTH: usize = 6;
        let lines = Arc::new(Mutex::new(Vec::new()));
        let mut options = SearchOptions::default();
        options.engine.hash_mb = 1;
        options.engine.threads = 4;
        options.limits.depth = Some(6);
        let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
        searcher.configure(&options.engine, HashRequest::Sent);

        for run in 0..30 {
            lines.lock().unwrap().clear();
            searcher.new_game();
            let result = searcher.search(Board::default(), &options, true, || SearchEvent::None);
            assert_eq!(result.depth, DEPTH, "run {run}");
            let printed = lines.lock().unwrap();
            let last = printed
                .iter()
                .rfind(|line| line.starts_with("info depth"))
                .expect("a depth line");
            assert!(
                last.starts_with(&format!("info depth {DEPTH} ")),
                "run {run}: {last}"
            );
        }
    }

    /// Every helper has let go of the table by the time the search returns,
    /// so the main thread's handle is the only one, and a `Hash` change frees
    /// the old table at once.
    #[test]
    fn a_helper_holds_the_table_only_for_its_job() {
        let mut options = SearchOptions::default();
        options.engine.hash_mb = 1;
        options.engine.threads = 4;
        options.limits.depth = Some(4);
        let mut searcher = Searcher::default();
        searcher.configure(&options.engine, HashRequest::Sent);

        for _ in 0..3 {
            let result = searcher.search(Board::default(), &options, false, || SearchEvent::None);
            assert!(!result.bestmove.is_null());
            assert_eq!(searcher.shared.tt.shared_handles(), 1);
        }

        options.engine.hash_mb = 2;
        searcher.configure(&options.engine, HashRequest::Sent);
        let result = searcher.search(Board::default(), &options, false, || SearchEvent::None);
        assert!(!result.bestmove.is_null(), "the resized table searches");
        assert_eq!(searcher.shared.tt.shared_handles(), 1);
    }

    fn test_search_result(bestmove: Move, score: i32, depth: usize) -> SearchResult {
        SearchResult {
            bestmove,
            pondermove: Move::NULL,
            score,
            depth,
            pv: vec![bestmove],
            seldepth: depth,
            nodes: 0,
            tb_hits: 0,
            elapsed_ms: 0,
            exit: SearchExit::Stop,
            ponderhit: false,
        }
    }
}
