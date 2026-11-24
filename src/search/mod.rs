//! Search: iterative deepening and the root, the node kernels and their
//! producers. Evaluation lives in `crate::eval`.

// `clippy::too_many_arguments` is accepted crate-wide for search kernels —
// see the rationale in Cargo.toml's [lints.clippy] section.

/// Whether a pricing probe switched `rule` off at `ply`: the one decision a
/// node-level probe reverses. Always false outside the `pricing` build.
macro_rules! pricing_off {
    ($searcher:expr, $ply:expr, $rule:ident) => {{
        #[cfg(feature = "pricing")]
        let off = $searcher.td.pricing.off == Some(($ply, crate::search::pricing::Rule::$rule));
        #[cfg(not(feature = "pricing"))]
        let off = false;
        off
    }};
}

mod compact;
mod correction;
mod history;
mod movepick;
mod node;
pub mod options;
pub mod params;
#[cfg(test)]
mod path_perft;
#[cfg(feature = "pricing")]
mod pricing;
mod report;
mod root;
mod shared;
mod stack;
mod tb;
mod thread;
mod threads;
mod time;

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::board::{Board, Color, GameResult, Move, MoveList, Piece};
use crate::infra;
use crate::score::{INF_SCORE, MATE_BOUND, MATE_SCORE, MAX_PLY, TB_VALUE, TB_WIN_SCORE};
use crate::search::options::{
    BOUND_LINES_FROM_NODES, EngineOptions, HashRequest, MAX_THREADS, SearchLimits, SearchOptions,
};
use crate::syzygy::{self, Wdl};
use crate::tt::TranspositionTable;

use params::SearchParams;
use shared::{RootBound, STOP_NONE, STOP_QUIT, STOP_SEARCH, SearchShared, TbRootDecision};
use stack::{PlyArray, StackEntry};
use thread::ThreadData;
use threads::WorkerPool;
use time::{IterationSignals, MateWatch, RuntimeLimits, TbExtension, compute_runtime_limits};

const MAX_DEPTH: usize = 100;

/// The most a draw's score lies from zero. The kernel scores a draw a unit
/// either side of it by the node count, so a line that can draw is not
/// scored level with every other draw.
pub const DRAW_SPREAD: i32 = 1;
const MIN_PARALLEL_DEPTH: usize = 4;
const SHARED_NODE_BATCH: u64 = 128;
const SHARED_NODE_BATCH_MASK: u64 = SHARED_NODE_BATCH - 1;
/// Where a searcher writes its protocol output: a report for each `info`
/// line, which the protocol layer writes (`crate::uci::info`), and `info
/// string` notices as text. The search never touches stdout itself; the
/// engine layer decides where its output goes.
pub trait InfoSink: Send {
    /// One complete line of text, without the trailing newline.
    fn line(&self, line: &str);
    /// One report of the search's state.
    fn report(&self, report: &InfoReport<'_>);
}

/// What a search reports on an `info` line.
pub enum InfoReport<'a> {
    /// A line of an iteration.
    Iteration(InfoRecord<'a>),
    /// The root has no legal move: mated in check, stalemated otherwise.
    NoLegalMove { in_check: bool },
}

/// One iteration's line, as the search knows it.
pub struct InfoRecord<'a> {
    pub depth: usize,
    pub seldepth: usize,
    /// The line's rank, from 1; only MultiPV reports more than one.
    pub multipv: usize,
    /// The score as shown: a tablebase result already placed in its
    /// display band.
    pub score: i32,
    pub bound: ScoreBound,
    pub nodes: u64,
    /// The table's fill, in permille.
    pub hashfull: usize,
    pub tb_hits: u64,
    pub elapsed_ms: u128,
    pub pv: &'a [Move],
    /// The root's material (`score_material`), which sets the scale an
    /// ordinary score prints at.
    pub material: i32,
    /// The line carries the win, draw and loss rates (`UCI_ShowWDL`).
    pub show_wdl: bool,
}

/// The material a reported score's scale is read at: pawn 1, knight and
/// bishop 3, rook 5, queen 9, both sides; 78 at the start.
pub(crate) fn score_material(board: &Board) -> i32 {
    [
        (Piece::Pawn, 1),
        (Piece::Knight, 3),
        (Piece::Bishop, 3),
        (Piece::Rook, 5),
        (Piece::Queen, 9),
    ]
    .into_iter()
    .map(|(piece, value)| {
        let both = board.pieces(Color::White, piece) | board.pieces(Color::Black, piece);
        infra::to_i32(both.count()) * value
    })
    .sum()
}

/// What a reported score is: exact, or the bound of a window that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreBound {
    Exact,
    /// The window failed high: the score is at least this.
    Lower,
    /// The window failed low: the score is at most this.
    Upper,
}

impl From<RootBound> for ScoreBound {
    fn from(bound: RootBound) -> Self {
        match bound {
            RootBound::Exact => Self::Exact,
            RootBound::Lower => Self::Lower,
            RootBound::Upper => Self::Upper,
        }
    }
}

/// Discards every line. The default, and what helper threads use.
struct SilentSink;

impl InfoSink for SilentSink {
    fn line(&self, _line: &str) {}
    fn report(&self, _report: &InfoReport<'_>) {}
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
    /// The principal variation this thread would report, and the selective
    /// depth beside it. Carried because a parallel search picks its move by
    /// vote: when the winner is a helper, the main thread's last `info` line
    /// describes a different move, so the pool prints the winner's own line
    /// before `bestmove` (Stockfish's `output_pv(*bestThread)`).
    pub pv: Vec<Move>,
    pub seldepth: usize,
    pub nodes: u64,
    pub tb_hits: u64,
    pub(crate) elapsed_ms: u128,
    pub exit: SearchExit,
    pub ponderhit: bool,
}

/// Persistent state for one legal root move across iterative-deepening passes.
/// The last completed iteration that proved a win, kept against a later
/// iteration that no longer proves one.
struct ProvedWin {
    mv: Move,
    ponder: Move,
    score: i32,
    pv: [Move; MAX_PLY],
    pv_len: usize,
}

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
    /// point/statistics block cold prevents it from perturbing the node
    /// kernel's hot code layout (inlining it measured a real NPS loss).
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
    /// overweight volatile iterations in the per-move score statistics.
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
    /// Root, clock and quiescence settings, which integration tests set to
    /// exercise switches production leaves off, so they stay runtime values.
    params: SearchParams,
    /// The compact kernel's seeds as the tuning options set them.
    #[cfg(feature = "tune")]
    compact_seeds: &'static compact::rules::Seeds,
    /// Their exchange values, built when they change.
    #[cfg(all(feature = "tune", not(feature = "pricing")))]
    compact_exchange: crate::board::SeeValues,
    limits: RuntimeLimits,
    start: Instant,
    /// The search's node count past which a failed root window prints its
    /// bound line; below it only exact lines and the line a search stops on
    /// are printed.
    bound_lines_from_nodes: u64,
    /// The root's material, the scale its reported scores print at; output
    /// only, no search decision reads it.
    score_material: i32,
    /// `info` lines carry the win, draw and loss rates.
    show_wdl: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            params: SearchParams::default(),
            #[cfg(feature = "tune")]
            compact_seeds: compact::rules::SEEDS,
            #[cfg(all(feature = "tune", not(feature = "pricing")))]
            compact_exchange: compact::KERNEL_SEE_VALUES,
            limits: RuntimeLimits::default(),
            start: Instant::now(),
            bound_lines_from_nodes: BOUND_LINES_FROM_NODES,
            score_material: 78,
            show_wdl: false,
        }
    }
}

/// One root line as MultiPV reports it.
#[derive(Clone, Debug)]
struct ReportedLine {
    depth: usize,
    seldepth: usize,
    score: i32,
    bound: RootBound,
    pv: Vec<Move>,
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

/// A tablebase result, not a mate: what the PV extension follows.
fn is_tb_band(score: i32) -> bool {
    (TB_WIN_SCORE..=TB_VALUE).contains(&score.abs())
}

impl Searcher {
    /// A searcher that writes its output to `sink`.
    pub fn with_sink(sink: Box<dyn InfoSink>) -> Self {
        let mut searcher = Self::default();
        searcher.td.sink = sink;
        searcher
    }

    /// A helper's searcher. It searches the pool's table, held only for each
    /// job, so what it keeps of its own between jobs is the smallest table.
    fn helper() -> Self {
        Self {
            worker_pool: WorkerPool::default(),
            shared: SearchShared::with_table(TranspositionTable::new(1)),
            cfg: SearchConfig::default(),
            td: ThreadData::default(),
        }
    }

    fn notice(&self, message: std::fmt::Arguments<'_>) {
        self.td.sink.line(&format!("info string {message}"));
    }

    pub fn configure(&mut self, options: &EngineOptions, hash: HashRequest) {
        let in_use = self.shared.hash_mb;
        // The option decides each big table's backing when it is allocated,
        // so a change allocates them all again: the helpers with their own,
        // this thread's, and the transposition table, released before its
        // replacement so that two are never held at once.
        let backing_changed = options.large_pages != self.shared.large_pages;
        if backing_changed {
            crate::large_pages::set_enabled(options.large_pages);
            self.shared.large_pages = options.large_pages;
            self.worker_pool.set_helper_count(0);
            self.td.reallocate_tables();
            self.shared.tt = TranspositionTable::new(1);
            self.shared.hash_mb = 1;
        }
        // A Hash the GUI sent is tried whenever it differs from the table in
        // use, and a refusal said every time; any other change only keeps the
        // size in use, across a change of backing too, so it never reports on
        // a Hash nobody sent.
        let target = match hash {
            HashRequest::Sent => options.hash_mb,
            HashRequest::NotSent => in_use,
        };
        if target != self.shared.hash_mb {
            let resized = if options.threads > 1 {
                self.shared.tt.resize_shared(target)
            } else {
                self.shared.tt.resize(target)
            };
            if resized {
                self.shared.hash_mb = target;
            } else {
                self.notice(format_args!(
                    "Unable to allocate Hash value {target}; keeping {} MiB.",
                    self.shared.hash_mb
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
        // Put the table in the form the next search uses now, not in that
        // search: converting allocates and clears the whole table (128 MiB
        // cost 20 ms and 33,000 page faults on a fresh process's first move).
        if options.threads > 1 {
            self.shared.tt.make_shared(self.shared.hash_mb);
        } else if !self.shared.tt.ensure_local(self.shared.hash_mb) {
            self.notice(format_args!(
                "Unable to restore local transposition table at {} MiB.",
                self.shared.hash_mb
            ));
        }
        if backing_changed {
            self.notice(format_args!(
                "Large pages: {}",
                crate::large_pages::large_pages()
            ));
        }
    }

    pub(crate) fn clear_hash(&mut self) {
        self.shared.tt.clear();
    }

    /// The helpers clear alongside this thread, and all are done on return, so
    /// a `readyok` that follows means the whole pool is ready.
    pub fn new_game(&mut self) {
        self.worker_pool.start_new_game();
        self.shared.tt.clear();
        self.reset_thread_for_new_game();
        self.worker_pool.finish_new_game();
    }

    /// What one thread carries from game to game, cleared: its histories,
    /// pawn cache and previous search scores. The main thread and every
    /// helper run this one reset, so a new game's first window opens at
    /// the same score on each.
    fn reset_thread_for_new_game(&mut self) {
        self.clear_history();
        self.td.evaluator.clear_pawn_table();
        self.td.compact_previous_score = 0;
    }

    fn clear_history(&mut self) {
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
        self.search_impl::<true>(root, &options.limits, &options.engine, emit_info, &mut poll)
    }

    // The poll is a trait object from here down, and the main thread's and
    // the helpers' polls are too, so the kernel exists once for every thread:
    // a copy per closure type left the multi-thread copies outside the PGO
    // profile, which trains at one thread. It is called once per 2,048 nodes.
    fn search_impl<const ALLOW_PARALLEL: bool>(
        &mut self,
        mut root: Board,
        limits: &SearchLimits,
        engine_options: &EngineOptions,
        emit_info: bool,
        poll: &mut dyn FnMut() -> SearchEvent,
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

        let game_ply = 2 * u32::from(root.fullmove().saturating_sub(1))
            + u32::from(root.side_to_move() == Color::Black);
        self.reset_search_state(limits, engine_options, root.side_to_move(), game_ply, true);
        self.cfg.score_material = score_material(&root);

        let board = root;
        let mut legal_moves = MoveList::new();
        board.generate_legal_movelist_into(&mut legal_moves);
        if legal_moves.is_empty() {
            return self.no_legal_moves_result(&board, emit_info);
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
                        .any(|requested| requested.names(*mv))
                })
                .collect::<Vec<_>>();
            if filtered_root_moves.is_empty() {
                legal_moves.as_slice()
            } else {
                filtered_root_moves.as_slice()
            }
        };

        let syzygy_root_moves = self.syzygy_root_moves(&board, root_candidates);
        if syzygy::largest() > 0 {
            self.td.tb_root_board = Some(board.clone());
        }
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

        let lines = engine_options.multi_pv.clamp(1, root_moves.len());
        let mut result = if lines > 1 {
            self.search_root_multipv(board, root_moves, lines, emit_info, poll)
        } else {
            self.search_root(board, root_moves, emit_info, poll)
        };
        if emit_info {
            self.report_result_line(&result);
        }
        self.finish_tb_line(&mut result, lines, emit_info);
        result
    }

    fn reset_search_state(
        &mut self,
        limits: &SearchLimits,
        engine_options: &EngineOptions,
        side_to_move: Color,
        game_ply: u32,
        age_tt: bool,
    ) {
        // LazyMargin changes the raw evaluation function. The main searcher
        // owns TT lifecycle and must discard stored raw evals and bounds from
        // the old function. Helpers receive the already-cleared shared TT;
        // clearing shared storage here would race with helpers or main
        // already searching.
        let lazy_margin_changed = self
            .td
            .evaluator
            .set_lazy_margin(engine_options.search_params.lazy_margin);
        #[cfg(feature = "probes")]
        {
            self.td.rates.clear();
            compact::probes::seed_clear();
            self.td.hist.updates = 0;
        }
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
        self.shared.syzygy.root = TbRootDecision::default();
        self.cfg.params = engine_options.search_params.clone();
        self.cfg.bound_lines_from_nodes = engine_options.bound_lines_from_nodes;
        self.cfg.show_wdl = engine_options.show_wdl;
        // The kernel reads its seeds by a static reference, as it reads the
        // donor's in every other build, so a node borrows nothing from the
        // searcher. A table is allocated only when the options change it,
        // about a kilobyte for each tuning iteration a process plays.
        #[cfg(feature = "tune")]
        if *self.cfg.compact_seeds != engine_options.compact_seeds {
            self.cfg.compact_seeds = Box::leak(Box::new(engine_options.compact_seeds.clone()));
            #[cfg(not(feature = "pricing"))]
            {
                self.cfg.compact_exchange = compact::exchange_values_of(self.cfg.compact_seeds);
            }
        }
        // In a build whose seed table can change, the pawn history's limit
        // follows it, set once a search as the seeds are.
        #[cfg(any(feature = "tune", feature = "pricing"))]
        self.td.hist.set_pawn_max(self.seeds().pawn_history_max);
        self.td.nmp_min_ply = 0;
        self.shared.syzygy.largest = syzygy::largest().min(self.shared.syzygy.probe_limit);
        self.td.tb_root_scores.clear();
        self.td.tb_root_board = None;
        self.td.last_reported_move.set(Move::NULL);
        if age_tt {
            self.shared.tt.new_search();
        }
        self.td.pv_table = PlyArray::new([Move::NULL; MAX_PLY]);
        self.td.pv_len = PlyArray::new(0);
        self.td.stack = PlyArray::new(StackEntry::default());
    }

    /// A root with no legal move: report the position's value and then
    /// `bestmove`, as both donors do, so a GUI has a score to show.
    fn no_legal_moves_result(&mut self, board: &Board, emit_info: bool) -> SearchResult {
        let result = self.result_for_no_legal_moves(board);
        if emit_info {
            self.td.sink.report(&InfoReport::NoLegalMove {
                in_check: board.is_in_check(),
            });
        }
        SearchResult {
            bestmove: Move::NULL,
            pondermove: Move::NULL,
            score: self
                .td
                .evaluator
                .evaluate_result(result, board.side_to_move(), 0),
            depth: 0,
            pv: Vec::new(),
            seldepth: 0,
            nodes: 0,
            tb_hits: self.td.tb_hits,
            elapsed_ms: self.cfg.start.elapsed().as_millis(),
            exit: SearchExit::Stop,
            ponderhit: self.td.ponderhit,
        }
    }

    /// Count the position a move just reached, then look for a reason to
    /// stop: another thread's stop, the node limit, and on every 2048th
    /// count the GUI and the clock. A count is a move made, as Stockfish
    /// and Reckless keep it: a null move, a re-search, the hand-off to
    /// quiescence, razoring's quiescence search, a null-move verification
    /// and an exclusion search add nothing, so `nodes`, `nps`, `go nodes`
    /// and the bench count one thing, the same thing those engines count. A reason to stop sets
    /// `td.stopped`, which every node reads on entry.
    fn count_node<P: FnMut() -> SearchEvent + ?Sized>(&mut self, poll: &mut P) {
        let total_nodes = self.record_node();
        if let Some(shared_state) = self.shared.pool()
            && (self.cfg.limits.nodes > 0 || self.td.nodes & SHARED_NODE_BATCH_MASK == 0)
        {
            match shared_state.stop_state.load(Ordering::Relaxed) {
                STOP_QUIT => {
                    self.td.quit = true;
                    self.td.stopped = true;
                    return;
                }
                STOP_SEARCH => {
                    self.td.stopped = true;
                    return;
                }
                _ => {}
            }
        }
        // Not before the first iteration has completed, as for the clock
        // below: a search stopped inside it has no score and no searched move.
        if self.cfg.limits.nodes > 0
            && total_nodes >= self.cfg.limits.nodes
            && self.td.root_depth > 1
        {
            self.td.stopped = true;
            if let Some(shared_state) = self.shared.pool() {
                shared_state.request_stop();
            }
            return;
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
            // Not before the first iteration has completed: an aborted first
            // iteration has no move to fall back on.
            if !self.td.pondering && self.elapsed_ms() >= self.cfg.limits.maximum_ms {
                if self.td.root_depth > 1 {
                    self.td.stopped = true;
                } else {
                    crate::diag_count!(tm_first_iteration_guard);
                }
            }
        }
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

    /// What a stop inside an iteration discards when the root falls back to
    /// `fallback`, the last completed iteration's move: the interrupted
    /// window's root resets its line and sets it only when a fully searched
    /// move raises alpha.
    #[cfg(feature = "diag")]
    #[cold]
    fn count_interrupted_iteration(&self, fallback: Move, fail_high_move: Move) {
        crate::diag_count!(root_stop_mid_iteration);
        let new_best = self.td.pv_len[0] > 0 && self.td.pv_table[0][0] != fallback;
        let failed_high_new = !fail_high_move.is_null() && fail_high_move != fallback;
        if new_best {
            crate::diag_count!(root_stop_partial_new_best);
        }
        if failed_high_new {
            crate::diag_count!(root_stop_after_fail_high_new);
        }
        if new_best || failed_high_new {
            crate::diag_count!(root_stop_discards);
        }
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
mod tests;
