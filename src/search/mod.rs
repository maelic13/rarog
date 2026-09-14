//! Search: iterative deepening and the root, the node kernels and their
//! producers. Evaluation lives in `crate::eval`.

// `clippy::too_many_arguments` is accepted crate-wide for search kernels —
// see the rationale in Cargo.toml's [lints.clippy] section.

/// Print one search decision with its inputs. In a `diag` build, under
/// `go searchmoves`, on the main thread and at plies one and two only, every
/// prune, reduction, extension and proof decision is written to the output
/// sink as `info string trace ply <n> line <moves> <decision>`. Counters say
/// how often a decision fires; only a trace says which move it dropped and on
/// what inputs. Expands to nothing without the feature.
macro_rules! trace_decision {
    ($searcher:expr, $ply:expr, $($arg:tt)*) => {
        #[cfg(feature = "diag")]
        if $searcher.td.trace_decisions && matches!($ply, 1 | 2) {
            $searcher.trace_line($ply, format_args!($($arg)*));
        }
    };
}

// The selectivity-core candidate replaces the node kernel and the tables it
// owns as one unit behind the `b2core` umbrella. The module names stay the
// same in both arms, so everything else in the search compiles against either.
#[cfg_attr(feature = "b2core", path = "core/correction.rs")]
mod correction;
#[cfg_attr(feature = "b2core", path = "core/history.rs")]
mod history;
#[cfg_attr(feature = "b2core", path = "core/movepick.rs")]
mod movepick;
#[cfg_attr(feature = "b2core", path = "core/node.rs")]
mod node;
pub mod params;
mod shared;
mod stack;
mod thread;
mod threads;
mod time;

use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::board::{Board, Color, GameResult, Move, MoveList};
use crate::eval::{INF_SCORE, MATE_SCORE};
use crate::infra;
use crate::search_options::{EngineOptions, MAX_THREADS, SearchLimits, SearchOptions};
use crate::syzygy::{self, Wdl};

use node::build_lmr_table;
use params::SearchParams;
use shared::{RootBound, STOP_NONE, STOP_QUIT, STOP_SEARCH, SearchShared};
use stack::{PlyArray, StackEntry};
use thread::ThreadData;
use threads::WorkerPool;
use time::{RuntimeLimits, compute_runtime_limits, tm_effort_factor, tm_instability_factor};

const MAX_DEPTH: usize = 100;

const MAX_PLY: usize = 128;
const MAX_QPLY: usize = 16;
const MIN_PARALLEL_DEPTH: usize = 4;
/// Jitter-PRNG seeding. Two odd 64-bit constants (SplitMix64's
/// increment and Xorshift*'s multiplier); the `| 1` at the use site guarantees
/// the state is never zero, xorshift's fixed point.
const JITTER_SEED: u64 = 0x9E37_79B9_7F4A_7C15;
const JITTER_STRIDE: u64 = 0x2545_F491_4F6C_DD1D;
const SHARED_NODE_BATCH: u64 = 128;
const SHARED_NODE_BATCH_MASK: u64 = SHARED_NODE_BATCH - 1;
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // const-evaluated; MAX_PLY = 128
const TB_WIN_SCORE: i32 = MATE_SCORE - (MAX_PLY as i32) * 2;
/// Where a searcher writes its protocol output: the `info` line of each
/// completed iteration and `info string` notices. The search formats lines and
/// never touches stdout itself; the engine layer decides where they go.
pub trait InfoSink: Send {
    /// One complete line, without the trailing newline.
    fn line(&self, line: &str);
}

/// Discards every line. The default, and what helper threads use.
struct SilentSink;

impl InfoSink for SilentSink {
    fn line(&self, _line: &str) {}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SearchEvent {
    None,
    Stop,
    Quit,
    PonderHit,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SearchExit {
    Stop,
    Quit,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub bestmove: Move,
    pub pondermove: Move,
    pub score: i32,
    pub depth: usize,
    pub nodes: u64,
    pub tb_hits: u64,
    pub(crate) elapsed_ms: u128,
    pub exit: SearchExit,
    pub ponderhit: bool,
}

/// Persistent state for one legal root move across iterative-deepening passes.
#[derive(Debug, Clone)]
struct RootMove {
    mv: Move,
    score: i32,
    previous_score: i32,
    average_score: f64,
    mean_squared_score: f64,
    samples: u32,
    last_search_depth: usize,
    pv: [Move; MAX_PLY],
    pv_len: usize,
    nodes: u64,
    seldepth: usize,
    fail_highs: u32,
    fail_lows: u32,
    last_best_depth: usize,
}

impl RootMove {
    fn new(mv: Move) -> Self {
        let mut pv = [Move::NULL; MAX_PLY];
        pv[0] = mv;
        Self {
            mv,
            score: -INF_SCORE,
            previous_score: -INF_SCORE,
            average_score: 0.0,
            mean_squared_score: 0.0,
            samples: 0,
            last_search_depth: 0,
            pv,
            pv_len: 1,
            nodes: 0,
            seldepth: 0,
            fail_highs: 0,
            fail_lows: 0,
            last_best_depth: 0,
        }
    }

    /// Freeze the last iteration's score before this iteration starts.
    ///
    /// If the new iteration is interrupted, `previous_score` remains the last
    /// completed-iteration fallback while `score` may contain newer partial
    /// information, matching the distinction later consumers need.
    fn begin_iteration(&mut self) {
        self.previous_score = self.score;
    }

    /// Root-only bookkeeping must not be inlined into the node kernel. Besides
    /// executing only a few hundred times per search, keeping the floating
    /// point/statistics block cold prevents it from perturbing `negamax`'s hot
    /// code layout (inlining it measured a real NPS loss).
    #[cold]
    #[inline(never)]
    fn record_search(
        &mut self,
        depth: usize,
        score: i32,
        nodes: u64,
        seldepth: usize,
        bound: RootBound,
    ) {
        self.score = score;
        self.last_search_depth = depth;
        self.nodes = self.nodes.saturating_add(nodes);
        self.seldepth = self.seldepth.max(seldepth);
        match bound {
            RootBound::Lower => self.fail_highs = self.fail_highs.saturating_add(1),
            RootBound::Upper => self.fail_lows = self.fail_lows.saturating_add(1),
            RootBound::Exact => {}
        }
    }

    /// Add exactly one distribution sample for a COMPLETED iteration. Failed
    /// aspiration visits still update score/bounds/effort above, but must not
    /// overweight volatile iterations in the statistics consumed by 10.2.
    fn complete_iteration(&mut self) {
        self.samples = self.samples.saturating_add(1);
        let weight = 1.0 / f64::from(self.samples);
        let score = f64::from(self.score);
        self.average_score += (score - self.average_score) * weight;
        let squared_score = score * score;
        self.mean_squared_score += (squared_score - self.mean_squared_score) * weight;
    }
}

/// Configuration fixed for one `go`: the parameters, the reduction table built
/// from them, the resolved limits and the instant the clock started.
struct SearchConfig {
    params: SearchParams,
    lmr_table: Box<[[i32; 64]; 64]>,
    /// The `(base, div)` pair `lmr_table` was built from, so a search rebuilds
    /// it only when the parameters change.
    lmr_table_key: (i32, i32),
    limits: RuntimeLimits,
    start: Instant,
}

impl Default for SearchConfig {
    fn default() -> Self {
        let params = SearchParams::default();
        Self {
            lmr_table: build_lmr_table(params.lmr_table_base, params.lmr_table_div),
            lmr_table_key: (params.lmr_table_base, params.lmr_table_div),
            params,
            limits: RuntimeLimits::default(),
            start: Instant::now(),
        }
    }
}

/// One search thread: what every thread shares, the configuration of the
/// current search and the state only this thread mutates. The engine owns the
/// main thread's `Searcher`, which also owns the helper pool; each helper owns
/// its own with an empty pool.
#[derive(Default)]
pub struct Searcher {
    worker_pool: WorkerPool,
    shared: SearchShared,
    cfg: SearchConfig,
    td: ThreadData,
}

fn format_score(score: i32) -> String {
    if score >= MATE_SCORE - infra::to_i32(MAX_PLY) {
        format!("mate {}", (MATE_SCORE - score + 1) / 2)
    } else if score <= -MATE_SCORE + infra::to_i32(MAX_PLY) {
        format!("mate -{}", (MATE_SCORE + score + 1) / 2)
    } else {
        format!("cp {score}")
    }
}

impl Searcher {
    /// A searcher that writes its output to `sink`.
    pub fn with_sink(sink: Box<dyn InfoSink>) -> Self {
        let mut searcher = Self::default();
        searcher.td.sink = sink;
        searcher
    }

    fn notice(&self, message: std::fmt::Arguments<'_>) {
        self.td.sink.line(&format!("info string {message}"));
    }

    pub fn configure(&mut self, options: &EngineOptions) {
        if options.hash_mb != self.shared.hash_mb {
            if self.shared.tt.resize(options.hash_mb) {
                self.shared.hash_mb = options.hash_mb;
            } else {
                self.notice(format_args!(
                    "Unable to allocate Hash value {}; keeping {} MiB.",
                    options.hash_mb, self.shared.hash_mb
                ));
            }
        }
        let old_path = syzygy::current_path();
        let largest = syzygy::initialize(&options.syzygy.path);
        if old_path != options.syzygy.path && !options.syzygy.path.is_empty() {
            if largest == 0 {
                self.notice(format_args!("SyzygyPath loaded no usable tablebases."));
            } else {
                let (wdl, dtz) = syzygy::tablebase_file_counts(&options.syzygy.path);
                self.notice(format_args!(
                    "Found {wdl} WDL and {dtz} DTZ tablebase files (up to {largest}-man)."
                ));
            }
        }
        let wanted = options.threads.saturating_sub(1);
        let helpers = self.worker_pool.set_helper_count(wanted);
        if helpers < wanted {
            self.notice(format_args!(
                "Unable to create helper search thread {}; using {} search threads.",
                helpers + 1,
                helpers + 1
            ));
        }
    }

    pub fn clear_hash(&mut self) {
        self.shared.tt.clear();
    }

    pub fn new_game(&mut self) {
        self.shared.tt.clear();
        self.clear_history();
        self.td.evaluator.clear_pawn_table();
        self.worker_pool.new_game();
    }

    pub(crate) fn clear_history(&mut self) {
        self.td.hist.clear();
        self.td.corr.clear();
    }

    pub fn hashfull(&self) -> usize {
        self.shared.tt.hashfull()
    }

    pub fn search(
        &mut self,
        root: Board,
        options: &SearchOptions,
        emit_info: bool,
        mut poll: impl FnMut() -> SearchEvent,
    ) -> SearchResult {
        self.search_impl::<true, _>(root, &options.limits, &options.engine, emit_info, &mut poll)
    }

    fn search_impl<const ALLOW_PARALLEL: bool, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        mut root: Board,
        limits: &SearchLimits,
        engine_options: &EngineOptions,
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
        if ALLOW_PARALLEL
            && engine_options.threads <= 1
            && !self.shared.tt.ensure_local(self.shared.hash_mb)
        {
            self.notice(format_args!(
                "Unable to restore local transposition table at {} MiB.",
                self.shared.hash_mb
            ));
        }
        self.shared.leave_pool();
        self.td.root_move_offset = 0;
        self.td.thread_id = 0;

        // Reserve the whole search's history headroom once, before any hot
        // path or helper exists. Search pushes one UnmakeInfo per ply and pops
        // it again, so MAX_PLY bounds the peak above the game history already
        // present. Board::clone preserves capacity, so every worker's
        // root.clone() inherits this and no thread reallocates while searching.
        root.reserve_history(MAX_PLY);

        let game_ply = 2 * root.fullmove().saturating_sub(1) as u32
            + (root.side_to_move() == Color::Black) as u32;
        self.reset_search_state(
            limits,
            engine_options,
            root.side_to_move(),
            game_ply,
            true,
            true,
        );

        let board = root;
        let mut legal_moves = MoveList::new();
        board.generate_legal_movelist_into(&mut legal_moves);
        if legal_moves.is_empty() {
            return self.no_legal_moves_result(&board);
        }

        let filtered_root_moves;
        let root_candidates = if limits.search_moves.is_empty() {
            legal_moves.as_slice()
        } else {
            filtered_root_moves = legal_moves
                .iter()
                .copied()
                .filter(|mv| {
                    limits
                        .search_moves
                        .iter()
                        .any(|requested| mv.same_uci_move(*requested))
                })
                .collect::<Vec<_>>();
            if filtered_root_moves.is_empty() {
                legal_moves.as_slice()
            } else {
                filtered_root_moves.as_slice()
            }
        };

        let syzygy_root_moves = self.syzygy_root_moves(&board, root_candidates);
        let root_moves = syzygy_root_moves.as_deref().unwrap_or(root_candidates);

        if ALLOW_PARALLEL {
            let threads = engine_options.threads.clamp(1, MAX_THREADS);
            if threads > 1 && self.cfg.limits.depth.min(MAX_DEPTH - 1) >= MIN_PARALLEL_DEPTH {
                return self.search_parallel(
                    board,
                    root_moves,
                    limits,
                    engine_options.clone(),
                    threads,
                    emit_info,
                    poll,
                );
            }
        }

        self.search_root(board, root_moves, emit_info, poll)
    }

    fn reset_search_state(
        &mut self,
        limits: &SearchLimits,
        engine_options: &EngineOptions,
        side_to_move: Color,
        game_ply: u32,
        age_tt: bool,
        age_history: bool,
    ) {
        // LazyMargin changes the raw evaluation function. The evaluator owns
        // its whole-eval cache, while the main searcher owns TT lifecycle and
        // must also discard stored raw evals and bounds from the old function.
        // Helpers receive the already-cleared shared TT, so they clear only
        // their private evaluator cache; clearing shared storage here would
        // race with helpers or main already searching.
        let lazy_margin_changed = self
            .td
            .evaluator
            .set_lazy_margin(engine_options.search_params.lazy_margin);
        if lazy_margin_changed && age_tt {
            self.shared.tt.clear();
        }

        // The clock starts when `go` was parsed, as the harness measures it;
        // configuration invalidation and thread hand-off are on the clock
        // because they are on the harness's clock.
        self.cfg.start = limits.issued.unwrap_or_else(Instant::now);
        self.td.nodes = 0;
        self.td.tb_hits = 0;
        self.td.seldepth = 0;
        self.td.stopped = false;
        self.td.quit = false;
        self.td.pondering = limits.ponder;
        self.td.ponderhit = false;
        self.td.stop_on_ponderhit = false;
        self.cfg.limits =
            compute_runtime_limits(limits, engine_options, side_to_move, game_ply, MAX_DEPTH);
        self.shared.syzygy.probe_depth = engine_options.syzygy.probe_depth;
        self.shared.syzygy.probe_limit = engine_options.syzygy.probe_limit;
        self.shared.syzygy.fifty_move_rule = engine_options.syzygy.fifty_move_rule;
        self.cfg.params = engine_options.search_params.clone();
        let table_key = (
            self.cfg.params.lmr_table_base,
            self.cfg.params.lmr_table_div,
        );
        if table_key != self.cfg.lmr_table_key {
            self.cfg.lmr_table = build_lmr_table(table_key.0, table_key.1);
            self.cfg.lmr_table_key = table_key;
        }
        self.shared.syzygy.largest = syzygy::largest().min(self.shared.syzygy.probe_limit);
        self.td.root_iteration_nodes = 0;
        self.td.root_best_nodes = 0;
        self.td.root_best_effort = 0.0;
        if age_tt {
            self.shared.tt.new_search();
        }
        if age_history {
            self.td.hist.age();
            self.td.corr.age();
        }
        self.td.pv_table = PlyArray::new([Move::NULL; MAX_PLY]);
        self.td.pv_len = PlyArray::new(0);
        self.td.stack = PlyArray::new(StackEntry::default());
        #[cfg(feature = "diag")]
        {
            self.td.trace_decisions = self.td.thread_id == 0 && !limits.search_moves.is_empty();
        }
        // Re-seed the LMR-jitter PRNG per search, per thread, so each
        // thread walks a different sequence and a given thread's sequence does
        // not depend on how the previous search happened to end. `thread_id` is
        // bounded by MAX_THREADS so the conversion always succeeds; a fallback
        // seed would only pick a different sequence, never break anything.
        let thread_seed = u64::try_from(self.td.thread_id).unwrap_or(0);
        self.td.jitter_state = JITTER_SEED ^ thread_seed.wrapping_mul(JITTER_STRIDE) | 1;
    }

    fn no_legal_moves_result(&mut self, board: &Board) -> SearchResult {
        let result = self.result_for_no_legal_moves(board);
        SearchResult {
            bestmove: Move::NULL,
            pondermove: Move::NULL,
            score: self
                .td
                .evaluator
                .evaluate_result(result, board.side_to_move(), 0),
            depth: 0,
            nodes: 0,
            tb_hits: self.td.tb_hits,
            elapsed_ms: self.cfg.start.elapsed().as_millis(),
            exit: SearchExit::Stop,
            ponderhit: self.td.ponderhit,
        }
    }

    fn syzygy_root_moves(&mut self, board: &Board, legal_moves: &[Move]) -> Option<Vec<Move>> {
        if !self.can_probe_syzygy_root(board)
            || board.can_declare_draw()
            || self.cfg.limits.nodes > 0
        {
            return None;
        }

        let probe = syzygy::probe_root_moves(
            board,
            self.shared.syzygy.fifty_move_rule,
            board.has_repeated_position(),
        )?;
        self.record_tb_hit();

        let mut tb_moves = Vec::new();
        for probe_move in &probe.moves {
            let Some(mv) = syzygy::legal_move_from_root_probe(board, probe_move.root_move) else {
                continue;
            };
            if legal_moves.contains(&mv) {
                tb_moves.push((mv, probe_move.rank, probe_move.score));
            }
        }

        let best_rank = tb_moves.iter().map(|(_, rank, _)| *rank).max()?;
        let preferred_move = if probe.used_dtz && best_rank != 0 {
            syzygy::probe_root(board, self.shared.syzygy.fifty_move_rule)
                .and_then(|probe| probe.best_move)
                .and_then(|root_move| syzygy::legal_move_from_root_probe(board, root_move))
        } else {
            None
        };

        if best_rank != 0
            && let Some(preferred_move) = preferred_move
            && tb_moves
                .iter()
                .any(|(tb_move, rank, _)| *tb_move == preferred_move && *rank == best_rank)
        {
            self.record_tb_hit();
            return Some(vec![preferred_move]);
        }

        let mut root_moves = Vec::with_capacity(legal_moves.len());
        for &legal_move in legal_moves {
            if tb_moves
                .iter()
                .any(|(tb_move, rank, _)| *tb_move == legal_move && *rank == best_rank)
            {
                root_moves.push(legal_move);
            }
        }

        if root_moves.is_empty() {
            None
        } else {
            Some(root_moves)
        }
    }

    fn search_root<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        mut board: Board,
        legal_moves: &[Move],
        emit_info: bool,
        poll: &mut P,
    ) -> SearchResult {
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
        let mut prev_avg_score = 0.0_f64; // EWMA of completed root scores (SF bestPreviousAverageScore)
        let mut tot_best_move_changes = 0.0_f64; // decaying count of best-move changes
        // This thread's soft-stop vote is cast at most ONCE per search.
        // Without the latch a thread that keeps iterating past its own soft
        // target votes again every iteration and can reach the majority
        // single-handedly — which is the opposite of pooling the decision.
        let mut cast_stop_vote = false;

        for depth in 1..=max_depth {
            for root_move in &mut self.td.root_move_records {
                root_move.begin_iteration();
            }
            let previous_bestmove = bestmove;
            self.td.root_iteration_nodes = self.td.nodes;
            self.td.root_best_nodes = 0;
            self.td.root_best_effort = 0.0;
            // The aspiration window centers on this thread's own last
            // completed score — unless the pool has already proven an Exact
            // root score DEEPER than this thread's progress, in which case it
            // centers on the pool's estimate (fewer fail-high/low re-searches
            // when joining the pool's view). Serial searches have no shared
            // state and keep `best_score` bit-for-bit.
            let mut window_center = best_score;
            if let Some(shared) = self.shared.pool()
                && let Some((pool_depth, pool_score)) = shared.pool_best_exact()
                && pool_depth > infra::to_i32(completed_depth)
            {
                window_center = pool_score;
            }
            let use_aspiration =
                depth >= 4 && window_center.abs() < MATE_SCORE - infra::to_i32(MAX_PLY);
            let mut alpha_delta = self.cfg.params.aspiration_delta;
            let mut beta_delta = self.cfg.params.aspiration_delta;
            let mut alpha = if use_aspiration {
                (window_center - alpha_delta).max(-INF_SCORE)
            } else {
                -INF_SCORE
            };
            let mut beta = if use_aspiration {
                (window_center + beta_delta).min(INF_SCORE)
            } else {
                INF_SCORE
            };
            // TERMINATION BY CONSTRUCTION: once a side has failed
            // `asp_max_fails` times it is opened to ±INF and cannot fail again,
            // so this loop runs at most `2 * asp_max_fails` times whatever the
            // scores do.
            let mut fail_low_count = 0i32;
            let mut fail_high_count = 0i32;

            loop {
                let score =
                    self.search_root_window(&mut board, infra::to_i32(depth), alpha, beta, poll);
                if self.td.stopped || self.td.quit {
                    break;
                }
                // Termination guard. The widened window re-centers
                // on the previous iteration's best_score; with the delta
                // clamped to INF_SCORE that caps the reachable bound at
                // best_score ± INF_SCORE, which can never contain a mate score
                // found *this* iteration when best_score is negative-ish
                // (prev + 32001 < mate) — the fail-high loop then never
                // terminates (WAC.005 hung every fixed-depth search ≥ 4; games
                // masked it because the clock aborts the iteration). Force the
                // failing side fully open once a mate-magnitude score appears
                // or the delta saturates; every other re-search keeps the
                // best_score-centered dynamics — the SF-style "re-center on the
                // failing score" variant measured −4.52 ± 4.80 Elo, because
                // AspirationDelta and the pruning group were tuned around these
                // dynamics.
                if score <= alpha {
                    crate::diag_count!(asp_fail_low);
                    fail_low_count += 1;
                    alpha_delta = (alpha_delta * self.cfg.params.asp_growth_pct / 100
                        + self.cfg.params.asp_growth_add)
                        .min(INF_SCORE);
                    alpha = if fail_low_count >= self.cfg.params.asp_max_fails
                        || alpha_delta >= INF_SCORE
                        || score <= -(MATE_SCORE - infra::to_i32(MAX_PLY))
                    {
                        -INF_SCORE
                    } else {
                        (window_center - alpha_delta).max(-INF_SCORE)
                    };
                    beta = (alpha + beta) / 2;
                    continue;
                }
                if score >= beta {
                    crate::diag_count!(asp_fail_high);
                    fail_high_count += 1;
                    beta_delta = (beta_delta * self.cfg.params.asp_growth_high_pct / 100
                        + self.cfg.params.asp_growth_add)
                        .min(INF_SCORE);
                    beta = if fail_high_count >= self.cfg.params.asp_max_fails
                        || beta_delta >= INF_SCORE
                        || score >= MATE_SCORE - infra::to_i32(MAX_PLY)
                    {
                        INF_SCORE
                    } else {
                        (window_center + beta_delta).min(INF_SCORE)
                    };
                    continue;
                }
                best_score = score;
                completed_depth = depth;
                let iteration_nodes = self
                    .td
                    .nodes
                    .saturating_sub(self.td.root_iteration_nodes)
                    .max(1);
                self.td.root_best_effort = self.td.root_best_nodes as f64 / iteration_nodes as f64;
                if self.td.pv_len[0] > 0 {
                    bestmove = self.td.pv_table[0][0];
                    pondermove = if self.td.pv_len[0] > 1 {
                        self.td.pv_table[0][1]
                    } else {
                        Move::NULL
                    };
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

            if self.td.stopped || self.td.quit {
                break;
            }

            if emit_info {
                self.send_info(depth, best_score);
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
            // `go depth N` on a forced move is still a move request).
            if legal_moves.len() == 1 && depth >= 2 && !self.cfg.limits.analysis_mode {
                break;
            }

            // Update best-move instability and score EWMA for the soft-stop
            // formula. **This thread's own** best-move flips, deliberately:
            // the POOL's deepest-Exact move measured −5.54 ± 8.15 Elo over
            // 2,760 games at 4T, the noisier signal rather than the better one.
            tot_best_move_changes /= 2.0;
            if bestmove != previous_bestmove {
                tot_best_move_changes += 1.0;
                crate::diag_count!(root_best_changes);
            }
            crate::diag_count!(root_iterations);

            // `falling_eval` compares this iteration's score against the
            // average of the *prior* iterations. At this point `prev_avg_score`
            // is still that prior average, so capture it here as the baseline
            // BEFORE folding the current score in below; reading it after the
            // update would attenuate the signal to two-thirds. On the first
            // iteration there is no prior average, so the baseline is the
            // current score → a neutral (zero) falling signal.
            let falling_baseline = if completed_depth <= 1 {
                best_score as f64
            } else {
                prev_avg_score
            };
            // Update the EWMA for the next iteration's baseline.
            prev_avg_score = if completed_depth <= 1 {
                best_score as f64
            } else {
                (prev_avg_score * 2.0 + best_score as f64) / 3.0
            };

            // SF-style between-iteration stop.
            // movetime mode: no soft stop — check_stop (every 2048 nodes) fires at maximum_ms.
            // clock mode: stop when elapsed exceeds the dynamically scaled optimum.
            let elapsed_ms = self.elapsed_ms();
            if elapsed_ms >= self.cfg.limits.maximum_ms {
                break;
            }
            if !self.cfg.limits.movetime_mode {
                // TM dynamic multipliers, stored ×10000 in SearchParams;
                // `/ 10000.0` reconstructs the Stockfish seeds bit-exactly.
                let opt_scale = self.cfg.params.tm_opt_scale as f64 / 10_000.0;
                let fall_base = self.cfg.params.tm_fall_base as f64 / 10_000.0;
                let fall_slope = self.cfg.params.tm_fall_slope as f64 / 10_000.0;
                // fallingEval: ↑ when score is falling (want more time); seeds from SF.
                let falling_eval = (fall_base
                    + fall_slope * (falling_baseline - best_score as f64))
                    .clamp(0.572, 1.708);
                // bestMoveInstab: ↑ when best move changed recently.
                let best_move_instab =
                    tm_instability_factor(&self.cfg.params, tot_best_move_changes);
                // effortFactor: linear interp — at effort≤0.79 → effort_high; at effort≥1.0 → effort_low.
                let effort_factor = tm_effort_factor(&self.cfg.params, self.td.root_best_effort);
                let total_time = self.cfg.limits.optimum_ms
                    * opt_scale
                    * falling_eval
                    * best_move_instab
                    * effort_factor;
                let soft_target = total_time.min(self.cfg.limits.maximum_ms);
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

        // Dump per-search counters (no-op without `--features diag`).
        // Serial path only — see the reset note above. The parallel
        // dump lives in `search_parallel`, after the helpers are joined.
        if self.shared.threads == 1 {
            crate::diag::dump();
        }

        SearchResult {
            bestmove,
            pondermove,
            score: best_score,
            depth: completed_depth,
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
        let wdl = syzygy::probe_wdl(board, self.shared.syzygy.fifty_move_rule)?;
        self.record_tb_hit();
        Some(self.score_from_syzygy_wdl(wdl, ply))
    }

    fn can_probe_syzygy(&self, board: &Board, depth: i32) -> bool {
        self.shared.syzygy.largest > 0
            && depth >= self.shared.syzygy.probe_depth
            && board.castling().0 == 0
            && board.occupied_count() as usize <= self.shared.syzygy.largest
    }

    fn can_probe_syzygy_root(&self, board: &Board) -> bool {
        self.shared.syzygy.largest > 0
            && board.castling().0 == 0
            && board.occupied_count() as usize <= self.shared.syzygy.largest
    }

    fn score_from_syzygy_wdl(&self, wdl: Wdl, ply: usize) -> i32 {
        match wdl {
            Wdl::Win => TB_WIN_SCORE - infra::to_i32(ply),
            Wdl::CursedWin if !self.shared.syzygy.fifty_move_rule => {
                TB_WIN_SCORE - infra::to_i32(ply)
            }
            Wdl::Loss => -TB_WIN_SCORE + infra::to_i32(ply),
            Wdl::BlessedLoss if !self.shared.syzygy.fifty_move_rule => {
                -TB_WIN_SCORE + infra::to_i32(ply)
            }
            Wdl::BlessedLoss | Wdl::Draw | Wdl::CursedWin => 0,
        }
    }

    /// True when mechanism `bit` is ablated. Const `false` without the
    /// feature, so every guard below folds away in a shipped build.
    #[cfg(feature = "ablate")]
    #[inline]
    fn ablated(&self, bit: u32) -> bool {
        (self.cfg.params.ablation_mask >> bit) & 1 == 1
    }

    #[cfg(not(feature = "ablate"))]
    #[inline]
    #[expect(clippy::unused_self, reason = "mirrors the ablate-feature signature")]
    fn ablated(&self, _bit: u32) -> bool {
        false
    }

    fn check_stop<P: FnMut() -> SearchEvent + ?Sized>(&mut self, poll: &mut P) -> bool {
        let total_nodes = self.record_node();
        if let Some(shared_state) = self.shared.pool()
            && (self.cfg.limits.nodes > 0 || self.td.nodes & SHARED_NODE_BATCH_MASK == 0)
        {
            match shared_state.stop_state.load(Ordering::Relaxed) {
                STOP_QUIT => {
                    self.td.quit = true;
                    self.td.stopped = true;
                    return true;
                }
                STOP_SEARCH => {
                    self.td.stopped = true;
                    return true;
                }
                _ => {}
            }
        }
        if self.cfg.limits.nodes > 0 && total_nodes >= self.cfg.limits.nodes {
            self.td.stopped = true;
            if let Some(shared_state) = self.shared.pool() {
                shared_state.request_stop();
            }
            return true;
        }
        if self.td.nodes & 2047 == 0 {
            match poll() {
                SearchEvent::Quit => {
                    self.td.quit = true;
                    self.td.stopped = true;
                }
                SearchEvent::Stop => {
                    self.td.stopped = true;
                }
                SearchEvent::PonderHit => {
                    self.td.pondering = false;
                    self.td.ponderhit = true;
                    if self.td.stop_on_ponderhit {
                        self.td.stopped = true;
                    }
                    if let Some(shared_state) = self.shared.pool() {
                        shared_state.ponderhit.store(true, Ordering::Relaxed);
                    }
                }
                SearchEvent::None => {}
            }
            if !self.td.pondering && self.elapsed_ms() >= self.cfg.limits.maximum_ms {
                self.td.stopped = true;
            }
        }
        self.td.stopped
    }

    fn record_node(&mut self) -> u64 {
        self.td.nodes += 1;
        if let Some(shared_state) = self.shared.pool() {
            let pending = self.td.nodes & SHARED_NODE_BATCH_MASK;
            if pending == 0 {
                shared_state
                    .nodes
                    .fetch_add(SHARED_NODE_BATCH, Ordering::Relaxed)
                    + SHARED_NODE_BATCH
            } else if self.cfg.limits.nodes > 0 {
                shared_state.nodes.load(Ordering::Relaxed) + pending
            } else {
                self.td.nodes
            }
        } else {
            self.td.nodes
        }
    }

    fn record_tb_hit(&mut self) {
        self.td.tb_hits += 1;
        if let Some(shared_state) = self.shared.pool() {
            shared_state.tb_hits.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn reported_nodes(&self) -> u64 {
        self.shared.pool().map_or(self.td.nodes, |shared_state| {
            shared_state.nodes.load(Ordering::Relaxed) + (self.td.nodes & SHARED_NODE_BATCH_MASK)
        })
    }

    fn reported_tb_hits(&self) -> u64 {
        self.shared.pool().map_or(self.td.tb_hits, |shared_state| {
            shared_state.tb_hits.load(Ordering::Relaxed)
        })
    }

    /// Write one decision-trace line; see `trace_decision!`.
    #[cfg(feature = "diag")]
    #[cold]
    #[inline(never)]
    fn trace_line(&self, ply: usize, decision: std::fmt::Arguments<'_>) {
        let line = (0..ply)
            .map(|p| self.td.stack[p].mv.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        self.td.sink.line(&format!(
            "info string trace ply {ply} line {line} {decision}"
        ));
    }

    fn elapsed_ms(&self) -> f64 {
        self.cfg.start.elapsed().as_secs_f64() * 1000.0
    }

    fn send_info(&self, depth: usize, score: i32) {
        let pv = self.td.pv_table[0][..self.td.pv_len[0].min(MAX_PLY)]
            .iter()
            .copied()
            .filter(|mv| !mv.is_null())
            .collect::<Vec<_>>();
        self.send_info_line(depth, score, &pv);
    }

    fn send_info_line(&self, depth: usize, score: i32, pv: &[Move]) {
        let elapsed_ms = self.cfg.start.elapsed().as_millis();
        let nodes = self.reported_nodes();
        let tb_hits = self.reported_tb_hits();
        let nps = (nodes as u128 * 1000)
            .checked_div(elapsed_ms)
            .unwrap_or(nodes as u128);
        let pv = pv
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        self.td.sink.line(&format!(
            "info depth {} seldepth {} score {} nodes {} nps {} hashfull {} tbhits {} time {} pv {}",
            depth,
            self.td.seldepth,
            format_score(score),
            nodes,
            nps,
            self.hashfull(),
            tb_hits,
            elapsed_ms,
            pv
        ));
    }

    fn ponder_from_tt(&self, root: &Board, bestmove: Move) -> Move {
        if bestmove.is_null() {
            return Move::NULL;
        }
        let Some(bestmove) = root.legal_move(bestmove) else {
            return Move::NULL;
        };
        let mut child = root.clone();
        child.make_move(bestmove);
        self.shared
            .tt
            .probe(child.hash())
            .and_then(super::tt::TtEntry::best_move)
            .and_then(|mv| child.legal_move(mv))
            .unwrap_or(Move::NULL)
    }

    fn result_for_no_legal_moves(&self, board: &Board) -> GameResult {
        if board.is_in_check() {
            match board.side_to_move() {
                Color::White => GameResult::BlackCheckmates,
                Color::Black => GameResult::WhiteCheckmates,
            }
        } else {
            GameResult::Stalemate
        }
    }
}

#[cfg(test)]
mod tests {

    /// The evaluator mirrors `MAX_PLY` to bound its mop-up drive below the mate
    /// band without taking a dependency on the search. This is the assertion
    /// that keeps the mirror honest, and it lives here because this module is
    /// the one that legitimately sees both constants.
    #[test]
    fn mopup_mirror_matches_the_real_ply_horizon() {
        assert_eq!(
            crate::eval::MOPUP_ASSUMED_MAX_PLY as usize,
            super::MAX_PLY,
            "eval.rs mirrors MAX_PLY to bound the mop-up drive; the two have \
             drifted, so the compile-time bound in eval.rs is now checking \
             the wrong number"
        );
    }

    use super::*;
    use crate::eval::VALUE_NONE;
    use crate::tt::{Bound, TtStore};

    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    const LAZY_MARGIN_REGRESSION_FEN: &str = "5k2/5p1p/p3B1p1/Pp6/1P6/5P1P/4K1P1/8 b - - 0 1";

    struct Recorder(Arc<Mutex<Vec<String>>>);

    impl InfoSink for Recorder {
        fn line(&self, line: &str) {
            self.0.lock().unwrap().push(line.to_string());
        }
    }

    #[test]
    fn search_writes_one_info_line_per_iteration_through_its_sink() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
        let mut options = SearchOptions::default();
        options.limits.depth = Some(3);

        let result = searcher.search(options.board.clone(), &options, true, || SearchEvent::None);

        let lines = lines.lock().unwrap();
        assert_eq!(result.depth, 3);
        assert_eq!(lines.len(), 3, "{lines:?}");
        for (index, line) in lines.iter().enumerate() {
            assert!(
                line.starts_with(&format!("info depth {} ", index + 1)),
                "{line}"
            );
            assert!(line.contains(" pv "), "{line}");
        }

        drop(lines);

        let quiet_lines = Arc::new(Mutex::new(Vec::new()));
        let mut quiet = Searcher::with_sink(Box::new(Recorder(Arc::clone(&quiet_lines))));
        quiet.search(options.board.clone(), &options, false, || SearchEvent::None);
        assert!(
            quiet_lines.lock().unwrap().is_empty(),
            "emit_info = false writes nothing"
        );
    }

    /// The decision trace fires only under `searchmoves`, and only at plies one
    /// and two.
    #[cfg(feature = "diag")]
    #[test]
    fn decision_trace_is_bounded_to_searchmoves_and_plies_one_and_two() {
        let run = |search_moves: Vec<Move>| {
            let lines = Arc::new(Mutex::new(Vec::new()));
            let mut searcher = Searcher::with_sink(Box::new(Recorder(Arc::clone(&lines))));
            let mut options = SearchOptions::default();
            options.limits.depth = Some(6);
            options.limits.search_moves = search_moves;
            searcher.search(options.board.clone(), &options, false, || SearchEvent::None);
            let lines = lines.lock().unwrap();
            lines
                .iter()
                .filter(|line| line.starts_with("info string trace "))
                .cloned()
                .collect::<Vec<_>>()
        };

        assert!(run(Vec::new()).is_empty(), "no searchmoves, no trace");
        let traced = run(vec![
            Move::from_uci("e2e4").expect("valid move"),
            Move::from_uci("d2d4").expect("valid move"),
        ]);
        assert!(!traced.is_empty(), "searchmoves must produce a trace");
        for line in &traced {
            let (ply, rest) = line
                .strip_prefix("info string trace ply ")
                .and_then(|rest| rest.split_once(" line "))
                .unwrap_or_else(|| panic!("malformed trace line: {line}"));
            let path = rest.split_whitespace().take(2).collect::<Vec<_>>();
            assert!(matches!(ply, "1" | "2"), "{line}");
            assert!(matches!(path[0], "e2e4" | "d2d4"), "{line}");
        }
    }

    /// The budget is measured from the instant `go` was
    /// parsed, not from the engine thread's start. A `go movetime 200`
    /// parsed 300 ms ago has already spent its budget and must return at
    /// once with a legal move from the first completed iteration.
    #[test]
    fn search_clock_starts_when_go_was_parsed() {
        let board = Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).unwrap();
        let legal = board.generate_legal_moves();
        let limits = SearchLimits {
            move_time: 200,
            issued: Some(Instant::now() - Duration::from_millis(300)),
            ..SearchLimits::default()
        };
        let mut searcher = Searcher::default();
        searcher.reset_search_state(
            &limits,
            &EngineOptions::default(),
            board.side_to_move(),
            0,
            true,
            true,
        );
        let started = Instant::now();
        let result = searcher.search_root(board, &legal, false, &mut || SearchEvent::None);
        let took = started.elapsed();
        assert!(
            took < Duration::from_millis(100),
            "a search whose budget expired before it began must stop at once, took {took:?}"
        );
        assert!(
            result.depth >= 1,
            "at least one iteration completes: {}",
            result.depth
        );
        assert!(legal.contains(&result.bestmove), "bestmove must be legal");
        assert!(
            result.elapsed_ms >= 300,
            "elapsed is measured from the parse instant: {} ms",
            result.elapsed_ms
        );
    }

    fn store_static_eval(searcher: &mut Searcher, board: &Board, static_eval: i32) {
        let mv = board
            .generate_legal_moves()
            .into_iter()
            .next()
            .expect("regression position has a legal move");
        searcher.shared.tt.store(TtStore {
            key: board.hash(),
            depth: 1,
            score: static_eval,
            bound: Bound::Exact,
            mv,
            ply: 0,
            static_eval,
            is_pv: false,
        });
    }

    #[test]
    fn lazy_margin_change_invalidates_local_and_shared_tt_evals() {
        const LOW_MARGIN: i32 = 200;
        const HIGH_MARGIN: i32 = 2_000;
        let board =
            Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).expect("valid lazy-eval regression FEN");
        let limits = SearchLimits::default();

        for shared in [false, true] {
            let mut searcher = Searcher::default();
            let mut options = EngineOptions::default();
            options.search_params.lazy_margin = LOW_MARGIN;
            searcher.reset_search_state(&limits, &options, board.side_to_move(), 0, true, false);
            let low_margin_score = searcher.raw_eval(&board);

            if shared {
                searcher.shared.tt.make_shared(searcher.shared.hash_mb);
            }
            store_static_eval(&mut searcher, &board, low_margin_score);
            assert_eq!(
                i32::from(
                    searcher
                        .shared
                        .tt
                        .probe(board.hash())
                        .expect("stored TT entry")
                        .static_eval
                ),
                low_margin_score
            );

            options.search_params.lazy_margin = HIGH_MARGIN;
            searcher.reset_search_state(&limits, &options, board.side_to_move(), 0, true, false);
            assert!(
                searcher.shared.tt.probe(board.hash()).is_none(),
                "LazyMargin change retained a {} TT evaluation",
                if shared { "shared" } else { "local" }
            );
            assert_ne!(searcher.raw_eval(&board), low_margin_score);
        }
    }

    #[test]
    fn helper_lazy_margin_sync_does_not_clear_live_shared_tt() {
        let board =
            Board::from_fen(LAZY_MARGIN_REGRESSION_FEN).expect("valid lazy-eval regression FEN");
        let mut main = Searcher::default();
        main.shared.tt.make_shared(main.shared.hash_mb);
        store_static_eval(&mut main, &board, 123);

        let mut helper = Searcher::default();
        helper.shared.tt = main.shared.tt.clone();
        let mut options = EngineOptions::default();
        options.search_params.lazy_margin = 2_000;
        helper.reset_search_state(
            &SearchLimits::default(),
            &options,
            board.side_to_move(),
            0,
            false,
            false,
        );

        assert!(
            main.shared.tt.probe(board.hash()).is_some(),
            "helper startup cleared the shared TT after another thread made it live"
        );
    }

    #[test]
    fn search_root_respects_restricted_root_moves() {
        let mut searcher = Searcher::default();
        let board = Board::default();
        let forced = board.parse_move("a2a3").expect("legal root move");
        let engine_options = EngineOptions::default();
        let limits = SearchLimits {
            depth: Some(1),
            ..SearchLimits::default()
        };
        searcher.reset_search_state(
            &limits,
            &engine_options,
            board.side_to_move(),
            0,
            true,
            true,
        );

        let result = searcher.search_root(board, &[forced], false, &mut || SearchEvent::None);

        assert_eq!(result.bestmove, forced);
    }

    #[test]
    fn root_move_record_keeps_iteration_and_distribution_state() {
        let mv = Move::from_uci("e2e4").expect("valid move");
        let mut root_move = RootMove::new(mv);

        root_move.record_search(1, 20, 100, 7, RootBound::Exact);
        root_move.complete_iteration();
        root_move.begin_iteration();
        root_move.record_search(2, 40, 250, 9, RootBound::Upper);
        root_move.record_search(2, -30, 50, 8, RootBound::Lower);
        root_move.complete_iteration();

        assert_eq!(root_move.mv, mv);
        assert_eq!(root_move.previous_score, 20);
        assert_eq!(root_move.score, -30);
        assert_eq!(root_move.samples, 2);
        assert!((root_move.average_score + 5.0).abs() < f64::EPSILON);
        assert!((root_move.mean_squared_score - 650.0).abs() < f64::EPSILON);
        assert_eq!(root_move.nodes, 400);
        assert_eq!(root_move.seldepth, 9);
        assert_eq!(root_move.fail_highs, 1);
        assert_eq!(root_move.fail_lows, 1);
        assert_eq!(root_move.pv_len, 1);
        assert_eq!(root_move.pv[0], mv);
    }

    #[test]
    fn search_populates_persistent_root_move_records() {
        let mut searcher = Searcher::default();
        let board = Board::default();
        let legal = board.generate_legal_moves();
        let limits = SearchLimits {
            depth: Some(3),
            ..SearchLimits::default()
        };
        searcher.reset_search_state(
            &limits,
            &EngineOptions::default(),
            board.side_to_move(),
            0,
            true,
            true,
        );

        let result = searcher.search_root(board, &legal, false, &mut || SearchEvent::None);

        assert_eq!(searcher.td.root_move_records.len(), legal.len());
        assert!(searcher.td.root_move_records.iter().all(|rm| {
            rm.samples >= 3
                && rm.previous_score > -INF_SCORE
                && rm.score > -INF_SCORE
                && rm.nodes > 0
                && rm.seldepth > 0
                && rm.pv[0] == rm.mv
                && rm.mean_squared_score + 1e-9 >= rm.average_score * rm.average_score
        }));
        let best = searcher
            .td
            .root_move_records
            .iter()
            .find(|rm| rm.mv == result.bestmove)
            .expect("best move has a persistent record");
        assert_eq!(best.last_best_depth, result.depth);
        assert!(best.pv_len > 1);
        assert!(
            searcher
                .td
                .root_move_records
                .iter()
                .map(|rm| rm.nodes)
                .sum::<u64>()
                <= result.nodes
        );
    }

    /// The single-legal-move shortcut must save clock time WITHOUT truncating
    /// an analysis search.
    ///
    /// On `1k3Q1r/pPpP2p1/P1P3P1/8/8/1p6/1P6/K6N b` only `Rxf8` is legal.
    /// `search_root` breaks after depth 2 on a single root move — correct under
    /// a clock, since that move gets played regardless of score, but under
    /// `go infinite`/`go ponder` it would freeze a GUI's analysis at depth 2 on
    /// a meaningless score. Both halves are asserted: the shortcut fires for a
    /// move request and not in analysis mode.
    /// The SMP machinery (pool root scores, stop voting, reduction
    /// jitter, pool-seeded aspiration) must be INERT at Threads=1.
    ///
    /// Every SMP feature gates on the thread count, which only a parallel
    /// search raises above one. This guards the property every 1-thread gate
    /// and the bench fingerprint rely on: a serial search must be
    /// deterministic and free of any pool machinery. If a gate ever leaks into the serial
    /// path, the run-to-run identity below breaks.
    #[test]
    fn smp_machinery_is_inert_on_a_single_thread() {
        let board = Board::default();
        let legal = board.generate_legal_moves();

        let run = || {
            let mut searcher = Searcher::default();
            let limits = SearchLimits {
                depth: Some(7),
                ..SearchLimits::default()
            };
            searcher.reset_search_state(
                &limits,
                &EngineOptions::default(),
                board.side_to_move(),
                0,
                true,
                true,
            );
            assert!(
                searcher.shared.pool().is_none(),
                "a serial search must have every pool gate closed"
            );
            let result =
                searcher.search_root(board.clone(), &legal, false, &mut || SearchEvent::None);
            (result.nodes, result.bestmove, result.score)
        };

        assert_eq!(
            run(),
            run(),
            "a serial search must be deterministic with all SMP gates closed"
        );
    }

    #[test]
    fn single_legal_move_shortcut_skips_analysis_but_not_move_requests() {
        const FORCED: &str = "1k3Q1r/pPpP2p1/P1P3P1/8/8/1p6/1P6/K6N b - - 0 1";
        let board = Board::from_fen(FORCED).expect("valid fen");
        let legal = board.generate_legal_moves();
        assert_eq!(legal.len(), 1, "position must have exactly one legal move");

        let run = |infinite: bool| {
            let mut searcher = Searcher::default();
            let limits = SearchLimits {
                // Generous depth cap either way, so the STOP REASON under test
                // is the shortcut rather than the depth limit.
                depth: Some(8),
                infinite,
                ..SearchLimits::default()
            };
            searcher.reset_search_state(
                &limits,
                &EngineOptions::default(),
                board.side_to_move(),
                0,
                true,
                true,
            );
            searcher
                .search_root(board.clone(), &legal, false, &mut || SearchEvent::None)
                .depth
        };

        // Move request: stops at the shortcut, well short of the depth cap.
        let move_request_depth = run(false);
        assert_eq!(
            move_request_depth, 2,
            "clock/move-request search should stop at the single-move shortcut"
        );

        // Analysis: must keep going and honour the depth request instead.
        let analysis_depth = run(true);
        assert!(
            analysis_depth > move_request_depth,
            "analysis search must not be truncated by the single-move shortcut              (got depth {analysis_depth}, move-request depth {move_request_depth})"
        );
    }

    #[test]
    fn ponderhit_preserves_elapsed_time_budget() {
        let mut searcher = Searcher::default();
        searcher.td.pondering = true;
        searcher.cfg.start = Instant::now() - Duration::from_millis(10);
        searcher.cfg.limits = RuntimeLimits {
            depth: 64,
            optimum_ms: 1.0,
            maximum_ms: 1.0,
            ..RuntimeLimits::default()
        };
        searcher.td.nodes = 2047;

        let stopped = searcher.check_stop(&mut || SearchEvent::PonderHit);

        assert!(stopped);
        assert!(searcher.td.ponderhit);
        assert!(!searcher.td.pondering);
    }

    #[test]
    fn malformed_tt_move_is_not_searched_or_reported_in_pv() {
        let root = Board::from_fen("2k5/pp3pp1/5n2/2P5/bPP2P2/P3K3/6Pp/3Q1B1R w - - 0 23")
            .expect("valid tournament-derived FEN");
        let illegal = Move::from_uci("e3f4").expect("valid UCI move shape");
        let mut board = root.clone();
        let before_fen = board.to_fen();
        let before_hash = board.hash();
        let mut searcher = Searcher::default();
        let engine_options = EngineOptions::default();
        let limits = SearchLimits {
            depth: Some(3),
            ..SearchLimits::default()
        };
        searcher.reset_search_state(
            &limits,
            &engine_options,
            board.side_to_move(),
            0,
            true,
            true,
        );
        searcher.shared.tt.store(TtStore {
            key: board.hash(),
            depth: 8,
            score: 0,
            bound: Bound::Exact,
            mv: illegal,
            ply: 0,
            static_eval: VALUE_NONE,
            is_pv: false,
        });

        let _ = searcher.search_root_window(&mut board, 3, -INF_SCORE, INF_SCORE, &mut || {
            SearchEvent::None
        });

        assert_eq!(board.to_fen(), before_fen);
        assert_eq!(board.hash(), before_hash);
        assert!(
            !searcher.td.pv_table[0][..searcher.td.pv_len[0].min(MAX_PLY)]
                .iter()
                .any(|&mv| mv.same_uci_move(illegal)),
            "malformed TT move must not appear in the root PV"
        );
        assert_legal_pv(
            &root,
            &searcher.td.pv_table[0][..searcher.td.pv_len[0].min(MAX_PLY)],
        );
    }

    #[test]
    fn ponder_move_can_be_recovered_from_tt_child() {
        let mut searcher = Searcher::default();
        let root = Board::default();
        let bestmove = root.parse_move("a2a3").expect("legal root move");
        let mut child = root.clone();
        child.make_move(bestmove);
        let ponder = child.parse_move("a7a6").expect("legal child move");
        searcher.shared.tt.store(TtStore {
            key: child.hash(),
            depth: 4,
            score: 0,
            bound: Bound::Exact,
            mv: ponder,
            ply: 1,
            static_eval: VALUE_NONE,
            is_pv: false,
        });

        assert_eq!(searcher.ponder_from_tt(&root, bestmove), ponder);
    }

    fn assert_legal_pv(root: &Board, pv: &[Move]) {
        let mut board = root.clone();
        for &mv in pv {
            if mv.is_null() {
                break;
            }
            let legal = board
                .parse_move(&mv.to_string())
                .unwrap_or_else(|| panic!("PV move {mv} is illegal in {}", board.to_fen()));
            board.make_move(legal);
        }
    }
}
