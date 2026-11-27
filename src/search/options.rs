//! What a search is asked to do: the engine's settings, one search's limits
//! and the position. The protocol fills these (`crate::uci`); the search
//! reads them.

use crate::board::{Board, UciMove};
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
    /// `info` lines carry the win, draw and loss rates (`UCI_ShowWDL`).
    pub show_wdl: bool,
    /// Back the big tables with large pages where Windows grants them.
    pub large_pages: bool,
    pub syzygy: SyzygyOptions,
    pub search_params: SearchParams,
    /// The search's node count past which a failed root window prints its
    /// bound line, `BOUND_LINES_FROM_NODES` by default. No UCI option sets
    /// it: a caller that wants every bound line, as a test, sets 0.
    pub bound_lines_from_nodes: u64,
    /// The compact kernel's seeds, which only a tuning build can set.
    #[cfg(feature = "tune")]
    pub(crate) compact_seeds: super::compact::rules::Seeds,
}

#[cfg(feature = "tune")]
impl EngineOptions {
    /// The compact kernel's seeds as tuning options, the donor's values as
    /// defaults.
    pub(crate) fn compact_seed_option_lines() -> Vec<String> {
        super::compact::tune::uci_option_strings()
    }

    /// Sets a compact kernel seed by its option name; false when the name is
    /// no tunable seed.
    pub(crate) fn set_compact_seed(&mut self, name: &str, value: &str) -> bool {
        super::compact::tune::set_uci_option(&mut self.compact_seeds, name, value)
    }
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            move_overhead: 10.0,
            hash_mb: 64,
            ponder: false,
            threads: 1,
            multi_pv: 1,
            show_wdl: false,
            large_pages: true,
            syzygy: SyzygyOptions::default(),
            search_params: SearchParams::default(),
            bound_lines_from_nodes: BOUND_LINES_FROM_NODES,
            #[cfg(feature = "tune")]
            compact_seeds: super::compact::rules::Seeds::FITTED,
        }
    }
}

/// The search's node count below which a failed root window prints no bound
/// line, the donor's: the kernel's narrow windows fail often by design, and
/// in a short search a line for each buries the exact ones.
pub(crate) const BOUND_LINES_FROM_NODES: u64 = 10_000_000;

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

/// Whether a reconfiguration carries a `SyzygyPath` the GUI has just sent.
/// Re-sending an unchanged path still retries loading when no tables are
/// currently loaded, while unrelated options skip tablebase initialisation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyzygyPathRequest {
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
