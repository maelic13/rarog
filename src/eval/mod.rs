//! Static evaluation.
//!
//! `evaluate` accumulates every term into running `mg`/`eg` totals and tapers
//! them by phase. Three steps read a partial sum, so the order in which terms
//! are added is part of the evaluation: the lazy gate (the tapered sum after
//! the pawn and passer-advance terms), the mop-up (`(mg + eg) / 2` after piece
//! activity, before imbalance and tempo) and the initiative term (the sign of
//! the running `eg`, last in piece activity).
//!
//! `clippy::too_many_arguments` accepted crate-wide — see Cargo.toml. Here it
//! covers the eval terms whose `&mut mg` / `&mut eg` out-parameters keep the
//! evaluator single-pass and allocation-free.

mod attacks;
pub(crate) mod endgame;
mod initiative;
mod king;
mod material;
mod params;
mod passers;
mod pawns;
mod pieces;
mod space;
mod threats;
mod trace;

#[cfg(feature = "texel")]
use std::cell::RefCell;

use crate::board::{ATTACKS, Bitboard, Board, Color, GameResult, Piece, Square};
use crate::infra;
use attacks::AttackMaps;
use endgame::scale_endgame;
use material::PHASE_W;
use params::{EvalTables, build_tables};
use pawns::RELATIVE_RANKS;
use trace::{tr_eg, tr_mg};

// Read only by the search's test that ties it to `MAX_PLY`.
#[cfg_attr(not(test), expect(unused_imports))]
pub(crate) use endgame::MOPUP_ASSUMED_MAX_PLY;
pub use params::{EVAL_PARAM_NAMES, EvalParams};
#[cfg(feature = "texel")]
pub use trace::{EvalCounts, EvalTrace, linear_delta_scale};

pub const MATE_SCORE: i32 = 32_000;
pub(crate) const INF_SCORE: i32 = 32_001;
pub(crate) const VALUE_NONE: i32 = 32_002;

const PAWN_TABLE_SIZE: usize = 16_384;
const EVAL_TABLE_SIZE: usize = 32_768;
const TOTAL_PHASE: i32 = 24;
/// Lazy-eval threshold: if the tapered material + PST + pawn score already
/// exceeds it, the positional block is skipped. It must equal the search's
/// `LazyMargin` default: otherwise a fresh process's first search clears the
/// evaluation cache and the TT on its clock, and `texel` fits, which never set
/// the margin, describe a function the engine does not play.
const LAZY_MARGIN: i32 = 414;
const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, MATE_SCORE];

// Under `texel` the caches are written but never read (hits are bypassed so
// every position re-emits its trace), so the fields look dead to that build.
#[cfg_attr(feature = "texel", allow(dead_code))]
#[derive(Copy, Clone, Default)]
struct PawnEntry {
    key: u64,
    mg: i32,
    eg: i32,
    passed: [Bitboard; 2],
    attacks: [Bitboard; 2],
}

#[cfg_attr(feature = "texel", allow(dead_code))]
#[derive(Copy, Clone, Default)]
struct EvalEntry {
    key: u64,
    halfmove_clock: u8,
    value: i32,
    occupied: bool,
}

#[derive(Clone)]
pub struct Evaluator {
    pawn_table: Vec<PawnEntry>,
    eval_table: Vec<EvalEntry>,
    params: EvalParams,
    tables: Box<EvalTables>,
    /// Lazy-eval threshold. Seeded from `LAZY_MARGIN` and overridden by the
    /// `LazyMargin` UCI option (pushed in at every search start).
    lazy_margin: i32,
    /// The attack maps of the position being evaluated, filled at the start of
    /// piece activity and borrowed by its consumers. Kept across calls so the
    /// per-square slots are not re-zeroed every evaluation (see `AttackMaps`).
    attacks: AttackMaps,
    /// Per-call feature trace, recorded only under `--features texel`. Held in
    /// a `RefCell` so the `&self` eval helpers can append to it; the field does
    /// not exist in production builds.
    #[cfg(feature = "texel")]
    trace: RefCell<EvalTrace>,
}

impl Default for Evaluator {
    fn default() -> Self {
        #[cfg(feature = "tune")]
        let params = EvalParams::load_from_env();
        #[cfg(not(feature = "tune"))]
        let params = EvalParams::default();
        let tables = Box::new(build_tables(&params));
        Self {
            pawn_table: vec![PawnEntry::default(); PAWN_TABLE_SIZE],
            eval_table: vec![EvalEntry::default(); EVAL_TABLE_SIZE],
            params,
            tables,
            lazy_margin: LAZY_MARGIN,
            attacks: AttackMaps::new(),
            #[cfg(feature = "texel")]
            trace: RefCell::new(EvalTrace::default()),
        }
    }
}

#[cfg(feature = "texel")]
impl Evaluator {
    /// Read-only view of the evaluation parameters (the tuner's defaults).
    pub fn params(&self) -> &EvalParams {
        &self.params
    }

    /// Swap in new parameters, rebuilding the derived tables so a changed
    /// material/PST/king-safety weight is fully reflected. Used by the
    /// nonlinear king-safety fit, which re-evaluates the dataset many times with
    /// perturbed danger-index weights (those weights select a table bucket
    /// nonlinearly, so the linear trace cannot see them). The whole-eval cache
    /// is never *read* under `texel` (hits are bypassed), so stale entries from
    /// a previous parameter set are harmless and need no clearing.
    pub fn set_params(&mut self, params: EvalParams) {
        *self.tables = build_tables(&params);
        self.params = params;
    }

    /// A clone of the trace captured by the most recent `evaluate()` call.
    pub fn last_trace(&self) -> EvalTrace {
        self.trace.borrow().clone()
    }
}

impl Evaluator {
    /// Override the lazy-eval margin (Phase 5.1b `LazyMargin` UCI option).
    /// Returns whether the evaluation semantics changed, so the search owner
    /// can invalidate cached raw evaluations outside this evaluator as well.
    pub fn set_lazy_margin(&mut self, margin: i32) -> bool {
        if self.lazy_margin != margin {
            self.lazy_margin = margin;
            self.eval_table.fill(EvalEntry::default());
            true
        } else {
            false
        }
    }

    pub(crate) fn clear_pawn_table(&mut self) {
        self.pawn_table.fill(PawnEntry::default());
        self.eval_table.fill(EvalEntry::default());
    }

    pub fn evaluate_result(&self, result: GameResult, color: Color, ply: usize) -> i32 {
        let mate = MATE_SCORE - infra::to_i32(ply);
        match (result, color) {
            (GameResult::WhiteCheckmates, Color::White)
            | (GameResult::BlackCheckmates, Color::Black) => mate,
            (GameResult::WhiteCheckmates, Color::Black)
            | (GameResult::BlackCheckmates, Color::White) => -mate,
            (GameResult::Stalemate, _) | (GameResult::Draw, _) => 0,
        }
    }

    pub fn evaluate(&mut self, board: &Board) -> i32 {
        // 4.9a search-tree occurrence, counted BEFORE the cache lookup: a
        // cache hit is still the search reaching that family, and counting
        // only misses would undercount exactly the families the tree revisits
        // most. Compiled out entirely without `--features diag`.
        #[cfg(feature = "diag")]
        {
            let counts = |c: Color| {
                [
                    board.pieces(c, Piece::Pawn).count(),
                    board.pieces(c, Piece::Knight).count(),
                    board.pieces(c, Piece::Bishop).count(),
                    board.pieces(c, Piece::Rook).count(),
                    board.pieces(c, Piece::Queen).count(),
                ]
            };
            crate::diag::record_endgame_family(counts(Color::White), counts(Color::Black));
        }
        // The whole-eval cache must be bypassed under `texel`: a cache hit
        // returns without re-emitting trace counts, which would poison the
        // per-position trace the tuner records.
        let eval_slot = infra::index(board.hash()) & (EVAL_TABLE_SIZE - 1);
        #[cfg(not(feature = "texel"))]
        {
            let cached = self.eval_table[eval_slot];
            if cached.occupied
                && cached.key == board.hash()
                && cached.halfmove_clock == board.halfmove_clock()
            {
                return cached.value;
            }
        }
        #[cfg(feature = "texel")]
        self.trace.borrow_mut().reset();

        let atk = &*ATTACKS;
        let mut mg = 0;
        let mut eg = 0;
        let mut phase = 0;

        for color in [Color::White, Color::Black] {
            let sign = color_sign(color);
            for piece in Piece::ALL {
                let mut bb = board.pieces(color, piece);
                let phase_weight = PHASE_W[piece as usize];
                while bb.any() {
                    let sq = bb.pop_lsb();
                    phase += phase_weight;
                    mg += sign * self.tables.mg[color as usize][piece as usize][sq.index()];
                    eg += sign * self.tables.eg[color as usize][piece as usize][sq.index()];
                    // Material and PST are separate tunable params; the cooked
                    // table folds them, so trace each separately. The PST index
                    // mirrors build_tables: sq for white, sq^56 for black.
                    #[cfg_attr(not(feature = "texel"), allow(unused_variables))]
                    let pst_sq = if color == Color::White {
                        sq.index()
                    } else {
                        sq.index() ^ 56
                    };
                    tr_mg!(self, mg_val, piece as usize, sign);
                    tr_eg!(self, eg_val, piece as usize, sign);
                    tr_mg!(self, pst_mg, piece as usize * 64 + pst_sq, sign);
                    tr_eg!(self, pst_eg, piece as usize * 64 + pst_sq, sign);
                }
            }
        }
        phase = phase.min(TOTAL_PHASE);
        #[cfg(feature = "texel")]
        {
            self.trace.borrow_mut().phase = phase;
        }

        let mut passed = [Bitboard::EMPTY; 2];
        let mut pawn_attacks = [Bitboard::EMPTY; 2];
        let (pawn_mg, pawn_eg) = self.eval_pawns(board, atk, &mut passed, &mut pawn_attacks);
        mg += pawn_mg;
        eg += pawn_eg;
        // Passed-pawn free-stop / safe-stop bonuses (Phase 3.14): occupancy- and
        // attack-dependent, so they run every evaluation rather than living in
        // the pawn-structure cache. Applied here — immediately after `eval_pawns`
        // and before `eval_piece_activity` — so the running `mg`/`eg` totals seen
        // by downstream nonlinear terms (e.g. the mop-up's `(mg+eg)/2` test)
        // match the pre-3.14 ordering exactly; only the cache key changes.
        self.eval_passed_pawn_advance(board, &passed, &mut mg, &mut eg);

        // Lazy eval (Phase 3.16): if the cheap material + PST + pawn margin
        // already decides the position by more than any positional term could
        // flip, skip the expensive block (piece activity = mobility / threats /
        // king-safety / hanging / small-terms, plus imbalance). The mop-up still
        // runs, so mating technique (KBNK, KXK) survives a lazy skip. The gate
        // applies in every build, `texel` included, so the tuner traces and
        // fits the function the engine plays: above the gate a position's
        // trace holds only the terms evaluated before it, plus mop-up and
        // tempo. The eval stays a pure function of the position, so the eval
        // cache and `tests/eval_cache.rs` remain exact.
        let lazy =
            ((mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE).abs() > self.lazy_margin;

        if lazy {
            self.apply_mop_up(board, &mut mg, &mut eg);
        } else {
            self.eval_piece_activity(board, atk, &mut mg, &mut eg, &passed, &pawn_attacks, phase);
            // Mop-up keeps its pre-3.16 position (after activity, before
            // imbalance) so the full-eval path is byte-identical.
            self.apply_mop_up(board, &mut mg, &mut eg);
            self.eval_imbalance(board, &mut mg, &mut eg);
        }

        let tempo_sign = if board.side_to_move() == Color::White {
            1
        } else {
            -1
        };
        mg += tempo_sign * self.params.tempo[0];
        tr_mg!(self, tempo, 0, tempo_sign);

        let mut score = (mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
        #[cfg(feature = "texel")]
        {
            // Reconstruct the *linear* tapered score (frozen mop-up / passer
            // proximity constants removed) so it matches `reconstruct()` exactly.
            let (fmg, feg) = {
                let t = self.trace.borrow();
                (t.frozen_mg, t.frozen_eg)
            };
            let lin = ((mg - fmg) * phase + (eg - feg) * (TOTAL_PHASE - phase)) / TOTAL_PHASE;
            self.trace.borrow_mut().raw = lin;
        }
        score = scale_endgame(board, score);
        // No rule-50 damping here: the transposition table stores this eval
        // keyed without the clock, so the search damps it with the current
        // clock instead.
        let value = if board.side_to_move() == Color::White {
            score
        } else {
            -score
        };
        self.eval_table[eval_slot] = EvalEntry {
            key: board.hash(),
            halfmove_clock: board.halfmove_clock(),
            value,
            occupied: true,
        };
        value
    }
}
#[inline(always)]
pub fn piece_value(piece: Piece) -> i32 {
    // 9.0: safe indexing — `piece as usize` from a 6-variant enum is provably
    // 0..=5, so LLVM elides the bounds check; identical codegen, no unsafe.
    PIECE_VALUES[piece as usize]
}

#[inline(always)]
fn color_sign(color: Color) -> i32 {
    if color == Color::White { 1 } else { -1 }
}

#[inline(always)]
fn relative_rank(color: Color, sq: Square) -> u8 {
    RELATIVE_RANKS[color as usize][sq.index()]
}

fn forward_square(color: Color, sq: Square) -> Option<Square> {
    match color {
        Color::White => sq.0.checked_add(8).filter(|to| *to < 64).map(Square),
        Color::Black => sq.0.checked_sub(8).map(Square),
    }
}
