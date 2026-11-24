//! Per-thread search state.

use std::cell::Cell;

use crate::board::{Board, Move};
use crate::eval::Evaluator;

use super::correction::CorrectionTables;
use super::history::HistoryTables;
use super::stack::{PlyArray, StackEntry};
use super::{InfoSink, RootMove, SilentSink};
use crate::score::MAX_PLY;

/// Everything one search thread owns and mutates while it searches: the
/// per-ply stack and PV, the move-ordering histories and correction tables,
/// the root-move records, the node counters, the evaluator, the stop flags and
/// the output sink. Every table is per thread.
pub(super) struct ThreadData {
    pub(super) evaluator: Evaluator,
    /// The search must unwind: a limit, a stop request or a quit.
    pub(super) stopped: bool,
    pub(super) quit: bool,
    pub(super) pondering: bool,
    pub(super) ponderhit: bool,
    /// The soft target expired while pondering: stop at `ponderhit`.
    pub(super) stop_on_ponderhit: bool,
    pub(super) sink: Box<dyn InfoSink>,
    pub(super) nodes: u64,
    pub(super) tb_hits: u64,
    pub(super) seldepth: usize,
    pub(super) pv_table: PlyArray<[Move; MAX_PLY]>,
    pub(super) pv_len: PlyArray<usize>,
    /// Per-ply search context with sentinel entries below the root. See
    /// `StackEntry`.
    pub(super) stack: PlyArray<StackEntry>,
    /// Compact root-order/index backbone, kept separate from the larger
    /// records below so move-membership and SMP hot reads stay cache-compact.
    pub(super) root_moves: Vec<Move>,
    pub(super) root_move_records: Vec<RootMove>,
    /// At a tablebase root, each legal move's displayed score; empty
    /// elsewhere. Only the reporting thread fills it.
    pub(super) tb_root_scores: Vec<(Move, i32)>,
    /// The root, kept by the reporting thread when tables are loaded, for
    /// extending reported lines through them.
    pub(super) tb_root_board: Option<Board>,
    /// The first move of line 1 in the last `info` line this thread printed,
    /// so the search can tell whether that line describes its `bestmove`.
    pub(super) last_reported_move: Cell<Move>,
    /// Index of the MultiPV line being searched; 0 outside MultiPV, where the
    /// root reads and writes the table as an ordinary node.
    pub(super) multipv_line: usize,
    /// Move-ordering histories.
    pub(super) hist: HistoryTables,
    /// Static-evaluation correction tables.
    pub(super) corr: CorrectionTables,
    pub(super) root_move_offset: usize,
    /// 0 = main thread, 1.. = helper index.
    pub(super) thread_id: usize,
    /// Width of the root window of the current aspiration step.
    pub(super) root_delta: i32,
    /// While a null-move verification search runs, the first ply at which the
    /// null move is allowed again; zero outside a verification, and at every
    /// search start. Per thread: a helper verifies its own null moves.
    pub(super) nmp_min_ply: i32,
    /// The depth of the iteration this thread is searching; positive
    /// extensions stop at twice it. Zero before any root search.
    pub(super) root_depth: i32,
    /// The iteration depth the compact kernel's root records carry. Its
    /// root window searches a window that failed high a ply shallower, but
    /// the root driver completes a move's iteration only from a record at
    /// the iteration's depth. Zero outside the driver.
    pub(super) root_report_depth: i32,
    /// The compact kernel's optimism, by colour: what its evaluation
    /// adds toward the root's running score for the side to move; zero
    /// outside a compact search.
    pub(super) optimism: [i32; 2],
    /// The compact kernel's rule counts for the rates line.
    #[cfg(feature = "probes")]
    pub(super) rates: super::compact::probes::Rates,
    /// Moves other than the first that raised alpha on line one in the
    /// current iteration: the clock's best-move change count, cleared at
    /// every iteration as in the donor.
    pub(super) best_move_changes: u32,
    /// The root score at the previous search's end in the game, zero at a
    /// new game: the next search's running average starts there, and the
    /// clock's score trend reads the fall from it.
    pub(super) compact_previous_score: i32,
    /// Moves made in the tree, for the test of the node count.
    #[cfg(test)]
    pub(super) moves_made: u64,
    /// The call a parent is about to make and its own node type, taken by the
    /// child it starts, for the test of the kernel's child types.
    #[cfg(test)]
    pub(super) child_call: Option<(ChildCall, bool)>,
    /// Children counted by call, by the parent's node type and by their own.
    #[cfg(test)]
    pub(super) child_types: [[[u64; 2]; 2]; ChildCall::COUNT],
    /// A clock for the tests of the clock's stops: from this node count on,
    /// the search reads this many elapsed milliseconds, and none before it.
    #[cfg(test)]
    pub(super) test_clock: Option<(u64, f64)>,
    /// The pricing instrument's probe state and records.
    #[cfg(feature = "pricing")]
    pub(super) pricing: super::pricing::Pricing,
}

impl Default for ThreadData {
    fn default() -> Self {
        Self {
            evaluator: Evaluator::default(),
            stopped: false,
            quit: false,
            pondering: false,
            ponderhit: false,
            stop_on_ponderhit: false,
            sink: Box::new(SilentSink),
            nodes: 0,
            tb_hits: 0,
            seldepth: 0,
            pv_table: PlyArray::new([Move::NULL; MAX_PLY]),
            pv_len: PlyArray::new(0),
            stack: PlyArray::new(StackEntry::default()),
            root_moves: Vec::new(),
            root_move_records: Vec::new(),
            tb_root_scores: Vec::new(),
            tb_root_board: None,
            last_reported_move: Cell::new(Move::NULL),
            multipv_line: 0,
            hist: HistoryTables::default(),
            corr: CorrectionTables::default(),
            root_move_offset: 0,
            thread_id: 0,
            root_delta: 1,
            nmp_min_ply: 0,
            root_depth: 0,
            root_report_depth: 0,
            optimism: [0; 2],
            #[cfg(feature = "probes")]
            rates: super::compact::probes::Rates::default(),
            best_move_changes: 0,
            compact_previous_score: 0,
            #[cfg(test)]
            moves_made: 0,
            #[cfg(test)]
            child_call: None,
            #[cfg(test)]
            child_types: [[[0; 2]; 2]; ChildCall::COUNT],
            #[cfg(test)]
            test_clock: None,
            #[cfg(feature = "pricing")]
            pricing: super::pricing::Pricing::default(),
        }
    }
}

impl ThreadData {
    /// This thread's big tables allocated again, empty, so that they take the
    /// backing the `LargePages` option now asks for.
    pub(super) fn reallocate_tables(&mut self) {
        self.hist = HistoryTables::default();
        self.corr = CorrectionTables::default();
        self.evaluator.reallocate_pawn_table();
    }
}

/// Each recursive call of the kernel's node search, for the test that pins
/// the node type its child receives.
#[cfg(test)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum ChildCall {
    Null,
    NullVerify,
    ProbCut,
    Singular,
    Reduced,
    Research,
    FullDepth,
    Pv,
}

#[cfg(test)]
impl ChildCall {
    pub(super) const COUNT: usize = 8;
    pub(super) const ALL: [ChildCall; Self::COUNT] = [
        ChildCall::Null,
        ChildCall::NullVerify,
        ChildCall::ProbCut,
        ChildCall::Singular,
        ChildCall::Reduced,
        ChildCall::Research,
        ChildCall::FullDepth,
        ChildCall::Pv,
    ];
}
