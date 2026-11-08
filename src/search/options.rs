//! What a search is asked to do: the engine's settings, one search's limits
//! and the position. The protocol fills these (`crate::uci`); the search
//! reads them.

use crate::board::{Board, UciMove};
use crate::search::params::CoreParams;
use crate::search::params::ProofParams;
use crate::search::params::QuietParams;
use crate::search::params::SearchParams;

pub(crate) const MAX_THREADS: usize = 1024;
/// Most principal variations `MultiPV` can ask for.
pub(crate) const MAX_MULTI_PV: usize = 256;

#[derive(Clone, PartialEq, Eq)]
pub struct SyzygyOptions {
    pub path: String,
    pub probe_depth: i32,
    pub probe_limit: usize,
    pub fifty_move_rule: bool,
}

impl Default for SyzygyOptions {
    fn default() -> Self {
        Self {
            path: String::new(),
            probe_depth: 1,
            probe_limit: 7,
            fifty_move_rule: true,
        }
    }
}

#[derive(Clone)]
pub struct EngineOptions {
    pub move_overhead: f64,
    pub hash_mb: usize,
    pub ponder: bool,
    pub threads: usize,
    /// Principal variations reported per depth; 1 is the ordinary search.
    pub multi_pv: usize,
    /// Back the big tables with large pages where Windows grants them.
    pub large_pages: bool,
    pub syzygy: SyzygyOptions,
    pub search_params: SearchParams,
    /// The selectivity core's coordinates.
    pub core_params: CoreParams,
    /// The proof-search cluster's coordinates.
    pub proof_params: ProofParams,
    /// The quiescence cluster's coordinates and switches.
    pub quiet_params: QuietParams,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            move_overhead: 10.0,
            hash_mb: 64,
            ponder: false,
            threads: 1,
            multi_pv: 1,
            large_pages: true,
            syzygy: SyzygyOptions::default(),
            search_params: SearchParams::default(),
            core_params: CoreParams::default(),
            proof_params: ProofParams::default(),
            quiet_params: QuietParams::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct SearchLimits {
    pub move_time: usize,
    /// Each side's clock in milliseconds when `go` sent one. A present clock
    /// is at least 1 ms, so a zero, negative or unreadable one moves at once
    /// rather than reading as no clock.
    pub white_time: Option<usize>,
    pub white_increment: usize,
    pub black_time: Option<usize>,
    pub black_increment: usize,
    /// Fixed-depth limit from `go depth N`. `None` = no depth limit (the
    /// search runs to the internal MAX_DEPTH ceiling).
    pub depth: Option<u32>,
    /// `go mate N`: the search ends once a completed iteration on any thread
    /// proves a mate within N moves, for either side, and otherwise runs until
    /// stopped. It is no depth limit, so the pool's vote still chooses.
    pub mate: Option<u32>,
    pub movestogo: usize,
    pub nodes: u64,
    pub infinite: bool,
    pub ponder: bool,
    pub search_moves: Vec<UciMove>,
    /// The instant the `go` command was parsed on the UCI thread.
    ///
    /// The harness charges the clock from the moment it writes `go`, so the
    /// search budget starts there too, as Stockfish's `limits.startTime` and
    /// Reckless's parse-time `TimeManager` do. Stamping on the engine thread
    /// instead would take any hand-off latency under a loaded host straight
    /// off the harness margin. `None` (tests, bench) means the search stamps
    /// its own start.
    pub(crate) issued: Option<std::time::Instant>,
}

/// Whether a reconfiguration carries a `Hash` the GUI has just sent. Such a
/// value is tried whenever it differs from the table in use, and a refusal
/// is reported every time; any other change keeps the table at the size in
/// use and reports nothing about `Hash`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashRequest {
    Sent,
    NotSent,
}

#[derive(Clone, Default)]
pub struct SearchOptions {
    pub board: Board,
    pub engine: EngineOptions,
    pub limits: SearchLimits,
}

impl SearchOptions {
    /// A new game: the starting position and no limits; the engine's settings
    /// stay.
    pub fn reset(&mut self) {
        self.board = Board::default();
        self.limits = SearchLimits::default();
    }
}
