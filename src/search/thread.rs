//! Per-thread search state.

use crate::board::Move;
use crate::eval::Evaluator;

use super::correction::CorrectionTables;
use super::history::HistoryTables;
use super::stack::{PlyArray, StackEntry};
use super::{InfoSink, JITTER_SEED, MAX_PLY, RootMove, SilentSink};

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
    /// Print search decisions at plies one and two; see `trace_decision!`.
    #[cfg(feature = "diag")]
    pub(super) trace_decisions: bool,
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
    /// Index of the MultiPV line being searched; 0 outside MultiPV, where the
    /// root reads and writes the table as an ordinary node.
    pub(super) multipv_line: usize,
    /// Move-ordering histories.
    pub(super) hist: HistoryTables,
    /// Static-evaluation correction tables.
    pub(super) corr: CorrectionTables,
    pub(super) root_move_offset: usize,
    /// 0 = main thread, 1.. = helper index. Seeds the reduction jitter.
    pub(super) thread_id: usize,
    /// Xorshift64 state for the per-thread LMR jitter. Re-seeded from
    /// `thread_id` on every `reset_search_state`; never zero.
    pub(super) jitter_state: u64,
    pub(super) root_iteration_nodes: u64,
    pub(super) root_best_nodes: u64,
    pub(super) root_best_effort: f64,
    /// Width of the root window of the current aspiration step.
    #[cfg(feature = "b2core")]
    pub(super) root_delta: i32,
    /// Late-move reductions applied at a root node and at a node in check,
    /// for the tests of the reduction scope.
    #[cfg(all(test, feature = "b2core"))]
    pub(super) lmr_at_root: u64,
    #[cfg(all(test, feature = "b2core"))]
    pub(super) lmr_in_check: u64,
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
            #[cfg(feature = "diag")]
            trace_decisions: false,
            nodes: 0,
            tb_hits: 0,
            seldepth: 0,
            pv_table: PlyArray::new([Move::NULL; MAX_PLY]),
            pv_len: PlyArray::new(0),
            stack: PlyArray::new(StackEntry::default()),
            root_moves: Vec::new(),
            root_move_records: Vec::new(),
            multipv_line: 0,
            hist: HistoryTables::default(),
            corr: CorrectionTables::default(),
            root_move_offset: 0,
            thread_id: 0,
            jitter_state: JITTER_SEED,
            root_iteration_nodes: 0,
            root_best_nodes: 0,
            root_best_effort: 0.0,
            #[cfg(feature = "b2core")]
            root_delta: 1,
            #[cfg(all(test, feature = "b2core"))]
            lmr_at_root: 0,
            #[cfg(all(test, feature = "b2core"))]
            lmr_in_check: 0,
        }
    }
}
