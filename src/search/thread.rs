//! Per-thread search state.

use crate::board::Move;
use crate::eval::Evaluator;

use super::correction::CORR_SIZE;
use super::history::{
    CONT_SIZE, CONT_TABLES, LOW_PLY_HISTORY_SIZE, PAWN_HISTORY_SIZE, PIECE_TO_SIZE,
    boxed_cont_tables,
};
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
    pub(super) killers: PlyArray<[Move; 2]>,
    /// Compact root-order/index backbone, kept separate from the larger
    /// records below so move-membership and SMP hot reads stay cache-compact.
    pub(super) root_moves: Vec<Move>,
    pub(super) root_move_records: Vec<RootMove>,
    pub(super) main_history: Box<[[[i16; 64]; 64]; 2]>,
    pub(super) cap_history: Box<[[[i16; 6]; 64]; 6]>,
    pub(super) low_ply_history: Box<[[[i16; 64]; 64]; LOW_PLY_HISTORY_SIZE]>,
    /// Boxed const-size, NOT `Vec<i16>` — see [`ThreadData::cont_history`].
    pub(super) pawn_history: Box<[i16; PAWN_HISTORY_SIZE * PIECE_TO_SIZE]>,
    /// Continuation history, one table per look-back distance. Indexed by
    /// [`CONT_PLY_BACK`] position, NOT by ply distance — see that table.
    ///
    /// Boxed fixed-size arrays, not `Vec`s: the `Vec` form cost −2.1% NPS
    /// because runtime lengths defeat bounds-check elision in the hot loops.
    pub(super) cont_history: Box<[[i16; CONT_SIZE]; CONT_TABLES]>,
    pub(super) correction_history: Box<[[i16; CORR_SIZE]; 2]>,
    pub(super) minor_correction_history: Box<[[i16; CORR_SIZE]; 2]>,
    pub(super) non_pawn_correction_history: Box<[[[i16; CORR_SIZE]; 2]; 2]>,
    /// Boxed const-size, see [`ThreadData::pawn_history`].
    pub(super) continuation_correction_history: Box<[i16; PIECE_TO_SIZE]>,
    pub(super) countermove: Box<[[Move; 64]; 64]>,
    pub(super) root_move_offset: usize,
    /// 0 = main thread, 1.. = helper index. Seeds the reduction jitter.
    pub(super) thread_id: usize,
    /// Xorshift64 state for the per-thread LMR jitter. Re-seeded from
    /// `thread_id` on every `reset_search_state`; never zero.
    pub(super) jitter_state: u64,
    pub(super) root_iteration_nodes: u64,
    pub(super) root_best_nodes: u64,
    pub(super) root_best_effort: f64,
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
            killers: PlyArray::new([Move::NULL; 2]),
            root_moves: Vec::new(),
            root_move_records: Vec::new(),
            main_history: Box::new([[[0; 64]; 64]; 2]),
            cap_history: Box::new([[[0; 6]; 64]; 6]),
            low_ply_history: Box::new([[[0; 64]; 64]; LOW_PLY_HISTORY_SIZE]),
            pawn_history: Box::new([0; PAWN_HISTORY_SIZE * PIECE_TO_SIZE]),
            cont_history: boxed_cont_tables(),
            correction_history: Box::new([[0; CORR_SIZE]; 2]),
            minor_correction_history: Box::new([[0; CORR_SIZE]; 2]),
            non_pawn_correction_history: Box::new([[[0; CORR_SIZE]; 2]; 2]),
            continuation_correction_history: Box::new([0; PIECE_TO_SIZE]),
            countermove: Box::new([[Move::NULL; 64]; 64]),
            root_move_offset: 0,
            thread_id: 0,
            jitter_state: JITTER_SEED,
            root_iteration_nodes: 0,
            root_best_nodes: 0,
            root_best_effort: 0.0,
        }
    }
}
