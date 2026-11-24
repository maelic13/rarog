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
mod shared;
mod stack;
mod thread;
mod threads;
mod time;

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::board::{Board, Color, GameResult, Move, MoveList, Piece};
use crate::eval::{INF_SCORE, MATE_SCORE};
use crate::infra;
use crate::search::options::{
    BOUND_LINES_FROM_NODES, EngineOptions, HashRequest, MAX_THREADS, SearchLimits, SearchOptions,
};
use crate::syzygy::{self, Wdl};
use crate::tt::{TB_VALUE, TB_WIN_SCORE, TranspositionTable};

use params::SearchParams;
use shared::{RootBound, STOP_NONE, STOP_QUIT, STOP_SEARCH, SearchShared, TbRootDecision};
use stack::{PlyArray, StackEntry};
use thread::ThreadData;
use threads::WorkerPool;
use time::{IterationSignals, MateWatch, RuntimeLimits, TbExtension, compute_runtime_limits};

const MAX_DEPTH: usize = 100;

/// The deepest ply a line reaches, the donor's 240: from iteration 16 the
/// line bound's eight times the depth passes a 128-ply cap, and long
/// searches in endings met it.
pub(crate) const MAX_PLY: usize = 240;
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
pub fn score_material(board: &Board) -> i32 {
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

    pub fn clear_hash(&mut self) {
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

        let game_ply = 2 * root.fullmove().saturating_sub(1) as u32
            + (root.side_to_move() == Color::Black) as u32;
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

    /// Before `bestmove`: when the reported line's displayed score is a
    /// tablebase result, extend it through the tables, take the ponder move
    /// from it and print it. Under a clock the line gets its share of the
    /// box, and nothing when the clock is too short to risk a table read.
    fn finish_tb_line(&mut self, result: &mut SearchResult, lines: usize, emit_info: bool) {
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
    fn extend_tb_line(
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
            if let Some(ranking) = syzygy::rank_root_moves(&board, rule50, false) {
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
            let Some(ranking) = syzygy::rank_root_moves(&board, rule50, true) else {
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

    /// The root set the tables allow: every candidate of the best rank, in
    /// the candidates' order, or `None` when the tables do not rank this root.
    /// The whole group stays, so the search chooses among result-equal moves;
    /// a move the tables prefer is not a forced move.
    fn syzygy_root_moves(&mut self, board: &Board, candidates: &[Move]) -> Option<Vec<Move>> {
        if !self.can_probe_syzygy_root(board)
            || board.can_declare_draw()
            || self.cfg.limits.nodes > 0
        {
            return None;
        }
        let ranking = syzygy::rank_root_moves(board, self.shared.syzygy.fifty_move_rule, false)?;
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
    fn apply_tb_root(&mut self, decision: TbRootDecision) {
        self.shared.syzygy.root = decision;
        if decision.search_probes_off {
            self.shared.syzygy.largest = 0;
        }
    }

    fn search_root<P: FnMut() -> SearchEvent + ?Sized>(
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
                if best_score >= MATE_SCORE - infra::to_i32(MAX_PLY) {
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
            && best_score < MATE_SCORE - infra::to_i32(MAX_PLY)
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
        }
        let best_score = best_score;
        let reported = self.reported_line(bestmove, completed_depth, RootBound::Exact);
        SearchResult {
            bestmove,
            pondermove,
            score: best_score,
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
    fn search_root_multipv<P: FnMut() -> SearchEvent + ?Sized>(
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
        let best_score = best_score;
        let reported = self.reported_line(bestmove, completed_depth, RootBound::Exact);
        SearchResult {
            bestmove,
            pondermove,
            score: best_score,
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

    fn root_record(&self, mv: Move) -> Option<&RootMove> {
        self.root_record_index(mv)
            .map(|index| &self.td.root_move_records[index])
    }

    /// A line as it would be reported now: the move's record at `depth`, with
    /// its current score and PV.
    fn reported_line(&self, mv: Move, depth: usize, bound: RootBound) -> ReportedLine {
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

    /// The tables' display score for a root move where it replaces the
    /// search's: at a tablebase root, unless the search proved a mate.
    fn tb_root_score(&self, root_move: Option<&Move>, score: i32) -> Option<i32> {
        if score.abs() >= MATE_SCORE - infra::to_i32(MAX_PLY) {
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

    /// The score a line prints: the tables' where they replace the search's,
    /// otherwise the search's own.
    fn displayed_score(&self, root_move: Option<&Move>, score: i32) -> i32 {
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
    fn send_multipv_info(&self, lines: &[ReportedLine]) {
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

    /// The in-search probe as a bound: a tablebase win is a lower bound and a
    /// loss an upper bound, since the search may find a faster mate or a
    /// slower loss than the tables' distance; a draw, cursed and blessed
    /// results included under the rule-50 option, is exact.
    fn syzygy_wdl_bound(
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
            Wdl::Win => TB_VALUE - infra::to_i32(ply),
            Wdl::CursedWin if !self.shared.syzygy.fifty_move_rule => TB_VALUE - infra::to_i32(ply),
            Wdl::Loss => -TB_VALUE + infra::to_i32(ply),
            Wdl::BlessedLoss if !self.shared.syzygy.fifty_move_rule => {
                -TB_VALUE + infra::to_i32(ply)
            }
            Wdl::BlessedLoss | Wdl::Draw | Wdl::CursedWin => 0,
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

    fn reported_nodes(&self) -> u64 {
        self.shared.pool().map_or(self.td.nodes, |shared_state| {
            shared_state.nodes.load(Ordering::Relaxed) + (self.td.nodes & SHARED_NODE_BATCH_MASK)
        })
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

    /// Whether a failed root window prints its bound line: past the floor
    /// on the search's count, every thread's nodes included.
    fn shows_bound_lines(&self) -> bool {
        self.reported_nodes() > self.cfg.bound_lines_from_nodes
    }

    fn reported_tb_hits(&self) -> u64 {
        self.shared.pool().map_or(self.td.tb_hits, |shared_state| {
            shared_state.tb_hits.load(Ordering::Relaxed)
        })
    }

    fn elapsed_ms(&self) -> f64 {
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

    fn send_info(&self, depth: usize, score: i32) {
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
    fn send_bounded_info_line(&self, depth: usize, score: i32, bound: RootBound, fallback: Move) {
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
    fn send_full_info_line(
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
    fn send_shown_info_line(
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
    pub(super) fn report_result_line(&self, result: &SearchResult) {
        if self.td.last_reported_move.get() != result.bestmove {
            self.report_line(result);
        }
    }

    /// Print `result`'s line whatever was printed last.
    pub(super) fn report_line(&self, result: &SearchResult) {
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
