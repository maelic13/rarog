//! Lazy-SMP: the helper pool, per-thread job setup and the parallel result
//! vote.

use std::sync::mpsc::Sender;
use std::sync::{Arc, atomic::Ordering, mpsc};
use std::thread::{self, JoinHandle};

use crate::board::{Board, Color, Move};
use crate::eval::INF_SCORE;
use crate::infra;
use crate::search_options::{EngineOptions, SearchLimits};
use crate::tt::TranspositionTable;

use super::movepick::ScoredMoveList;
use super::shared::{RootBound, STOP_QUIT, STOP_SEARCH, SharedContext};
use super::{MAX_PLY, SearchEvent, SearchExit, SearchResult, Searcher, TB_WIN_SCORE};

struct WorkerJob {
    pub root: Board,
    pub(super) root_moves: Arc<[Move]>,
    pub limits: SearchLimits,
    pub(super) engine_options: EngineOptions,
    pub tt: TranspositionTable,
    pub hash_mb: usize,
    pub(super) root_move_offset: usize,
    /// 8.13: helper index (1-based); seeds the per-thread reduction jitter.
    pub(super) thread_id: usize,
    pub(super) shared_state: Arc<SharedContext>,
    result_tx: Sender<SearchResult>,
}

enum WorkerMessage {
    // 9.0: boxed — WorkerJob is ~712 B while the other variants are unit, so
    // every queued message paid the largest size. This is a per-search thread
    // handoff (not a hot path), so the indirection is free here.
    Search(Box<WorkerJob>),
    NewGame,
    Shutdown,
}

struct SearchWorkerHandle {
    sender: Sender<WorkerMessage>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Default)]
pub(super) struct WorkerPool {
    workers: Vec<SearchWorkerHandle>,
}

impl WorkerPool {
    /// Grow or shrink the pool to `helper_count` threads and return how many
    /// it holds, which is fewer when the operating system refuses a thread.
    pub(super) fn set_helper_count(&mut self, helper_count: usize) -> usize {
        while self.workers.len() > helper_count {
            if let Some(mut worker) = self.workers.pop() {
                let _ = worker.sender.send(WorkerMessage::Shutdown);
                if let Some(handle) = worker.handle.take() {
                    let _ = handle.join();
                }
            }
        }
        while self.workers.len() < helper_count {
            if let Some(worker) = spawn_search_worker(self.workers.len()) {
                self.workers.push(worker);
            } else {
                break;
            }
        }
        self.workers.len()
    }

    pub(crate) fn new_game(&self) {
        for worker in &self.workers {
            let _ = worker.sender.send(WorkerMessage::NewGame);
        }
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

fn spawn_search_worker(index: usize) -> Option<SearchWorkerHandle> {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::Builder::new()
        .name(format!("rarog-search-{index}"))
        .stack_size(infra::THREAD_STACK_SIZE)
        .spawn(move || {
            let mut worker = Searcher::default();
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
                    WorkerMessage::NewGame => worker.reset_worker_state_for_new_game(),
                    WorkerMessage::Shutdown => break,
                }
            }
        })
        .ok()?;
    Some(SearchWorkerHandle {
        sender,
        handle: Some(handle),
    })
}

fn select_parallel_result(results: &[SearchResult], root_moves: &[Move]) -> Option<SearchResult> {
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
        .map(|(_, result)| result.clone())
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
) -> (i32, i64, bool, usize, i32, bool) {
    let decisive_rank = if result.score >= TB_WIN_SCORE {
        2
    } else if result.score <= -TB_WIN_SCORE {
        0
    } else {
        1
    };
    (
        decisive_rank,
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
        self.evaluator.clear_pawn_table();
    }

    fn run_worker_job<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        job: WorkerJob,
        poll: &mut P,
    ) -> SearchResult {
        self.tt = job.tt;
        self.hash_mb = job.hash_mb;
        self.shared_state = Some(Arc::clone(&job.shared_state));
        self.td.root_move_offset = job.root_move_offset;
        self.td.thread_id = job.thread_id;
        let result = self.search_worker(
            job.root,
            &job.limits,
            &job.engine_options,
            job.root_moves.as_ref(),
            poll,
        );
        self.shared_state = None;
        result
    }

    /// One xorshift64 step, mapped to LMR-reduction jitter in 1024ths of a ply.
    ///
    /// 9.7.5(k) replaces `(nodes + id·27) % 128 − 59`, which had two defects a
    /// PRNG does not: it was **correlated with the node counter** (consecutive
    /// nodes got consecutive jitter, so "random" perturbation moved in ramps),
    /// and it was **biased +4.5/1024**, quietly raising every thread's mean
    /// reduction rather than only spreading it. The range here is [−64, 63], the
    /// same amplitude as before, with mean −0.5/1024 — nine times closer to
    /// zero, so the jitter now diversifies without also pruning harder.
    #[inline(always)]
    pub(super) fn next_jitter(&mut self, magnitude: i32) -> i32 {
        let mut x = self.td.jitter_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.td.jitter_state = x;
        // Top 7 bits, not the bottom ones: xorshift64's low bits are its
        // weakest (they carry the least mixing), and taking them measurably
        // skewed the mean. `>> 57` yields 0..=127, so the result is [−64, 63].
        // `magnitude` in 1024ths of a ply; the result is [−magnitude,
        // +magnitude]. At magnitude 64 this is EXACTLY the pre-4.5 expression
        // `(x >> 57) - 64`, since `bits * 64 / 64 - 64 == bits - 64`, so the
        // SMP path is unchanged by construction rather than by measurement.
        let bits = i32::try_from(x >> 57).expect("7-bit shift fits i32");
        bits * magnitude / 64 - magnitude
    }

    fn search_worker<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        root: Board,
        limits: &SearchLimits,
        engine_options: &EngineOptions,
        legal_moves: &[Move],
        poll: &mut P,
    ) -> SearchResult {
        let game_ply = 2 * root.fullmove().saturating_sub(1) as u32
            + (root.side_to_move() == Color::Black) as u32;
        // 8.13(a): helpers must NOT inherit the main thread's fixed depth.
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
            true,
        );
        self.search_root(root, legal_moves, false, poll)
    }

    #[cold]
    #[inline(never)]
    pub(super) fn search_parallel<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        root: Board,
        root_moves: &[Move],
        limits: &SearchLimits,
        engine_options: EngineOptions,
        threads: usize,
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        // 9.7.5(b): reset BEFORE any helper exists, so nothing already counted
        // gets wiped by a late-starting thread.
        crate::diag::reset();
        self.tt.make_shared(self.hash_mb);
        let helper_count = threads.saturating_sub(1);
        let root_len = root_moves.len();
        let shared_state = Arc::new(SharedContext::new(self.td.tb_hits, root_len, threads));
        let mut worker_engine_options = engine_options;
        worker_engine_options.threads = 1;
        self.worker_pool.set_helper_count(helper_count);
        let root_moves_shared: Arc<[Move]> = root_moves.to_vec().into();

        let (result_tx, result_rx) = mpsc::channel();
        let mut launched_helpers = 0usize;
        for index in 0..helper_count {
            // 8.13: stagger each helper's starting point in the root list so
            // the pool does not pile onto move 1. 9.7.5(e) tested removing this
            // (`RootRotation=false`, same binary both arms) and stopped at
            // −3.31 ± 10.62 over 1,682 games — inside the `[−5,0]` indifference
            // zone, i.e. unresolved but leaning toward rotation earning its
            // keep. Kept as the shipped behaviour; the switch was deleted
            // rather than shipped as a user-facing option.
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
                tt: self.tt.clone(),
                hash_mb: self.hash_mb,
                root_move_offset: offset,
                thread_id: index + 1,
                shared_state: Arc::clone(&shared_state),
                result_tx: result_tx.clone(),
            };
            if self.worker_pool.send_search(index, job) {
                launched_helpers += 1;
            }
        }
        drop(result_tx);

        self.td.root_move_offset = 0;
        self.td.thread_id = 0;
        self.shared_state = Some(Arc::clone(&shared_state));
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
        let main_result = self.search_root(root, root_moves, emit_info, &mut main_poll);
        shared_state.request_stop();

        let mut helper_results = Vec::with_capacity(launched_helpers + 1);
        helper_results.push(main_result);
        for _ in 0..launched_helpers {
            if let Ok(result) = result_rx.recv() {
                helper_results.push(result);
            }
        }
        self.td.root_move_offset = 0;

        #[cfg(feature = "diag")]
        if let Some(main) = helper_results.first() {
            let min_depth = helper_results
                .iter()
                .map(|result| result.depth)
                .min()
                .unwrap_or(0);
            let max_depth = helper_results
                .iter()
                .map(|result| result.depth)
                .max()
                .unwrap_or(0);
            let min_score = helper_results
                .iter()
                .map(|result| result.score)
                .min()
                .unwrap_or(0);
            let max_score = helper_results
                .iter()
                .map(|result| result.score)
                .max()
                .unwrap_or(0);
            let disagreements = helper_results
                .iter()
                .filter(|result| result.bestmove != main.bestmove)
                .count();
            crate::diag_add!(
                worker_best_disagreement,
                u64::try_from(disagreements).unwrap_or(u64::MAX)
            );
            crate::diag_add!(
                worker_depth_spread_sum,
                u64::try_from(max_depth.saturating_sub(min_depth)).unwrap_or(u64::MAX)
            );
            crate::diag_add!(
                worker_score_spread_sum,
                u64::from(max_score.saturating_sub(min_score).unsigned_abs())
            );
        }

        // 9.7.5(b): every helper has been joined above, so the counters are now
        // complete and this is the one legitimate dump point for a parallel go.
        crate::diag::dump();

        let total_nodes = helper_results.iter().map(|result| result.nodes).sum();
        let total_tb_hits = shared_state.tb_hits.load(Ordering::Relaxed);
        let quit = shared_state.stop_state.load(Ordering::Relaxed) == STOP_QUIT
            || helper_results
                .iter()
                .any(|result| result.exit == SearchExit::Quit);
        let mut best =
            select_parallel_result(&helper_results, root_moves).unwrap_or(SearchResult {
                bestmove: root_moves[0],
                pondermove: Move::NULL,
                score: -INF_SCORE,
                depth: 0,
                nodes: 0,
                tb_hits: 0,
                elapsed_ms: self.start.elapsed().as_millis(),
                exit: SearchExit::Stop,
                ponderhit: self.ponderhit,
            });
        self.td.nodes = total_nodes;
        self.td.tb_hits = total_tb_hits;
        self.quit = quit;
        self.stopped = true;
        best.nodes = total_nodes;
        best.tb_hits = total_tb_hits;
        best.elapsed_ms = self.start.elapsed().as_millis();
        if best.pondermove.is_null() {
            best.pondermove = self.ponder_from_tt(&root_for_ponder, best.bestmove);
        }
        best.ponderhit = self.ponderhit || helper_results.iter().any(|result| result.ponderhit);
        best.exit = if quit {
            SearchExit::Quit
        } else {
            SearchExit::Stop
        };
        self.shared_state = None;
        best
    }

    /// 8.13: fold the pool's per-root-move knowledge into this thread's root
    /// ordering.
    ///
    /// A move that another thread has already searched deeper gets lifted
    /// above the local heuristic ordering, ranked by (depth, score). The
    /// shared TT already carries much of this implicitly, but root entries
    /// are overwritten under pressure while these slots are not, so the
    /// explicit channel survives exactly the case it is needed in.
    pub(super) fn apply_shared_root_scores(
        &self,
        legal_moves: &[Move],
        scored: &mut ScoredMoveList,
    ) {
        let Some(shared) = &self.shared_state else {
            return;
        };
        for entry in scored.as_mut_slice() {
            let Some(index) = legal_moves.iter().position(|mv| *mv == entry.mv) else {
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
    /// kernel. The single cold call replaces several root-only branches that
    /// the first 10.1 implementation placed directly in `negamax` and that
    /// measured about -0.8% best-of NPS despite running only at the root.
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
        let Some(index) = self
            .td
            .root_moves
            .iter()
            .position(|root_move| *root_move == mv)
        else {
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
        if let Some(shared) = &self.shared_state {
            shared.publish_root_score(index, depth, score, bound);
        }

        let root_move = &mut self.td.root_move_records[index];
        // This is the cumulative search-wide seldepth at the time the move
        // completes (the same low-cost shape used by Basilisk), so a later move
        // may inherit a deeper earlier move's maximum. Exact per-move tracking
        // required extra branches in every recursive move loop and measured a
        // real speed loss; 10.2 should treat this field as a conservative max.
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
    use super::*;

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

        let selected =
            select_parallel_result(&results, &[e2e4, d2d4, g1f3]).expect("selected result");

        assert_eq!(selected.bestmove, d2d4);
    }

    #[test]
    fn parallel_result_selection_prefers_decisive_win() {
        let e2e4 = Move::from_uci("e2e4").expect("valid move");
        let d2d4 = Move::from_uci("d2d4").expect("valid move");
        let results = vec![
            test_search_result(e2e4, 900, 12),
            test_search_result(d2d4, TB_WIN_SCORE, 4),
        ];

        let selected = select_parallel_result(&results, &[e2e4, d2d4]).expect("selected result");

        assert_eq!(selected.bestmove, d2d4);
    }

    fn test_search_result(bestmove: Move, score: i32, depth: usize) -> SearchResult {
        SearchResult {
            bestmove,
            pondermove: Move::NULL,
            score,
            depth,
            nodes: 0,
            tb_hits: 0,
            elapsed_ms: 0,
            exit: SearchExit::Stop,
            ponderhit: false,
        }
    }
}
