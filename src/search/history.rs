//! Move-ordering histories: threat-aware quiet and noisy tables, pawn-structure
//! and continuation tables, their gravity update and the continuation helpers.

use crate::board::{Bitboard, Board, Color, FixedList, Move, Piece, Square};

use super::Searcher;

/// `(colour, piece type, square)` slots: the second index of every
/// piece-to table.
pub(super) const PIECE_TO_SIZE: usize = 12 * 64;
/// Continuation contexts: whether the earlier node was in check, whether its
/// move was noisy, and that move's coloured piece and destination.
pub(super) const CONT_CONTEXTS: usize = 2 * 2 * PIECE_TO_SIZE;
pub(super) const CONT_SIZE: usize = CONT_CONTEXTS * PIECE_TO_SIZE;
/// How many plies back each continuation term looks.
pub(super) const CONT_PLY_BACK: [usize; 4] = [1, 2, 4, 6];

const QUIET_MAX: i32 = 8_192;
const NOISY_MAX: i32 = 12_800;
/// The pawn table's limit, the kernel's seed; a tuning or pricing build
/// sets it for each search.
const PAWN_MAX: i32 = super::compact::rules::SEEDS.pawn_history_max;
const CONT_MAX: i32 = 15_320;
const PAWN_HISTORY_SIZE: usize = 512;
/// Captured-type slots of the noisy table: the six piece types and "none",
/// for a non-capturing promotion.
const CAPTURED_SLOTS: usize = 7;

/// Index of a coloured piece standing on `sq`.
#[inline(always)]
pub(super) fn piece_to(color: Color, piece: Piece, sq: Square) -> usize {
    (color as usize * 6 + piece as usize) * 64 + sq.index()
}

/// The continuation context a move made at a node selects, as a row base
/// into a `CONT_SIZE` table.
#[inline(always)]
pub(super) fn cont_context(
    in_check: bool,
    noisy: bool,
    color: Color,
    piece: Piece,
    to: Square,
) -> usize {
    ((usize::from(in_check) * 2 + usize::from(noisy)) * PIECE_TO_SIZE + piece_to(color, piece, to))
        * PIECE_TO_SIZE
}

/// Gravity update. The bonus is clamped to `±max` and the entry moves toward
/// its sign by `bonus - |bonus| * entry / max`, so an entry never leaves
/// `[-max, max]` and a saturated entry moves little.
#[inline(always)]
pub(super) fn apply_bonus(entry: &mut i16, bonus: i32, max: i32) {
    let bonus = bonus.clamp(-max, max);
    let current = i32::from(*entry);
    *entry = crate::infra::saturating_i16(current + bonus - bonus.abs() * current / max);
}

/// Heap-allocate a zeroed table without building it on the stack first.
fn zeroed<const N: usize>() -> Box<[i16; N]> {
    vec![0i16; N]
        .into_boxed_slice()
        .try_into()
        .unwrap_or_else(|_| unreachable!("length is N by construction"))
}

/// `[side to move][from threatened][to threatened][from][to]`.
type QuietTable = [[[[[i16; 64]; 64]; 2]; 2]; 2];

/// `[coloured piece and destination][captured type][destination threatened]`.
type NoisyTable = [[[i16; 2]; CAPTURED_SLOTS]; PIECE_TO_SIZE];

// Empty tables as statics: a box made from one is zeroed on the heap,
// where an array expression is built in the caller's frame and copied.
static EMPTY_QUIET: QuietTable = [[[[[0; 64]; 64]; 2]; 2]; 2];
static EMPTY_NOISY: NoisyTable = [[[0; 2]; CAPTURED_SLOTS]; PIECE_TO_SIZE];

/// One thread's move-ordering tables. None of them age between searches.
pub(super) struct HistoryTables {
    quiet: Box<QuietTable>,
    noisy: Box<NoisyTable>,
    /// `[pawn-key slot][coloured piece and destination]`, flat.
    pawn: Box<[i16; PAWN_HISTORY_SIZE * PIECE_TO_SIZE]>,
    /// `[context][coloured piece and destination]`, flat; a context is a row
    /// base from [`cont_context`].
    cont: Box<[i16; CONT_SIZE]>,
    /// The pawn table's limit, set from the seed table for each search.
    #[cfg(any(feature = "tune", feature = "pricing"))]
    pawn_max: i32,
    /// Set while a pricing probe runs: no update writes, so the measured
    /// search finds the tables as it left them.
    #[cfg(feature = "pricing")]
    pub(super) frozen: bool,
    /// Updates since the search began, for the rates line.
    #[cfg(feature = "probes")]
    pub(super) updates: u64,
}

impl Default for HistoryTables {
    fn default() -> Self {
        Self {
            quiet: Box::new(EMPTY_QUIET),
            noisy: Box::new(EMPTY_NOISY),
            pawn: zeroed(),
            cont: zeroed(),
            #[cfg(any(feature = "tune", feature = "pricing"))]
            pawn_max: PAWN_MAX,
            #[cfg(feature = "pricing")]
            frozen: false,
            #[cfg(feature = "probes")]
            updates: 0,
        }
    }
}

impl HistoryTables {
    /// The pawn table's limit for the next search.
    #[cfg(any(feature = "tune", feature = "pricing"))]
    pub(super) fn set_pawn_max(&mut self, max: i32) {
        self.pawn_max = max;
    }

    /// Forget everything, for a new game.
    pub(super) fn clear(&mut self) {
        *self.quiet = EMPTY_QUIET;
        *self.noisy = EMPTY_NOISY;
        self.pawn.fill(0);
        self.cont.fill(0);
    }

    #[inline(always)]
    pub(super) fn quiet(&self, threats: Bitboard, stm: Color, mv: Move) -> i32 {
        let (from, to) = (mv.from_sq(), mv.to_sq());
        i32::from(
            self.quiet[stm as usize][usize::from(threats.contains(from))]
                [usize::from(threats.contains(to))][from.index()][to.index()],
        )
    }

    #[inline(always)]
    pub(super) fn update_quiet(&mut self, threats: Bitboard, stm: Color, mv: Move, bonus: i32) {
        #[cfg(feature = "pricing")]
        if self.frozen {
            return;
        }
        let (from, to) = (mv.from_sq(), mv.to_sq());
        #[cfg(feature = "probes")]
        {
            self.updates += 1;
        }
        apply_bonus(
            &mut self.quiet[stm as usize][usize::from(threats.contains(from))]
                [usize::from(threats.contains(to))][from.index()][to.index()],
            bonus,
            QUIET_MAX,
        );
    }

    #[inline(always)]
    pub(super) fn noisy(
        &self,
        threats: Bitboard,
        color: Color,
        piece: Piece,
        to: Square,
        captured: Option<Piece>,
    ) -> i32 {
        i32::from(
            self.noisy[piece_to(color, piece, to)][captured.map_or(6, |p| p as usize)]
                [usize::from(threats.contains(to))],
        )
    }

    #[inline(always)]
    pub(super) fn update_noisy(
        &mut self,
        threats: Bitboard,
        color: Color,
        piece: Piece,
        to: Square,
        captured: Option<Piece>,
        bonus: i32,
    ) {
        #[cfg(feature = "pricing")]
        if self.frozen {
            return;
        }
        #[cfg(feature = "probes")]
        {
            self.updates += 1;
        }
        apply_bonus(
            &mut self.noisy[piece_to(color, piece, to)][captured.map_or(6, |p| p as usize)]
                [usize::from(threats.contains(to))],
            bonus,
            NOISY_MAX,
        );
    }

    /// Row base of the pawn table for this pawn structure.
    #[inline(always)]
    pub(super) fn pawn_row(pawn_key: u64) -> usize {
        (crate::infra::index(pawn_key) & (PAWN_HISTORY_SIZE - 1)) * PIECE_TO_SIZE
    }

    #[inline(always)]
    pub(super) fn pawn(&self, pawn_row: usize, color: Color, piece: Piece, to: Square) -> i32 {
        i32::from(self.pawn[pawn_row + piece_to(color, piece, to)])
    }

    #[inline(always)]
    pub(super) fn update_pawn(
        &mut self,
        pawn_row: usize,
        color: Color,
        piece: Piece,
        to: Square,
        bonus: i32,
    ) {
        #[cfg(feature = "pricing")]
        if self.frozen {
            return;
        }
        #[cfg(feature = "probes")]
        {
            self.updates += 1;
        }
        #[cfg(any(feature = "tune", feature = "pricing"))]
        let max = self.pawn_max;
        #[cfg(not(any(feature = "tune", feature = "pricing")))]
        let max = PAWN_MAX;
        apply_bonus(
            &mut self.pawn[pawn_row + piece_to(color, piece, to)],
            bonus,
            max,
        );
    }

    #[inline(always)]
    pub(super) fn cont(&self, context: usize, color: Color, piece: Piece, to: Square) -> i32 {
        i32::from(self.cont[context + piece_to(color, piece, to)])
    }

    #[inline(always)]
    pub(super) fn update_cont(
        &mut self,
        context: usize,
        color: Color,
        piece: Piece,
        to: Square,
        bonus: i32,
    ) {
        #[cfg(feature = "pricing")]
        if self.frozen {
            return;
        }
        #[cfg(feature = "probes")]
        {
            self.updates += 1;
        }
        apply_bonus(
            &mut self.cont[context + piece_to(color, piece, to)],
            bonus,
            CONT_MAX,
        );
    }
}

impl Searcher {
    /// The continuation context `back` plies before `ply`, or `None` when no
    /// move was made there.
    #[inline(always)]
    pub(super) fn cont_context_back(&self, ply: usize, back: usize) -> Option<usize> {
        let entry = self.td.stack.back(ply, back);
        (!entry.mv.is_null()).then_some(entry.cont_key)
    }

    /// The history a pruning or reduction decision reads for a quiet move:
    /// the quiet table plus the one- and two-ply continuations.
    pub(super) fn quiet_pruning_history(
        &self,
        board: &Board,
        threats: Bitboard,
        ply: usize,
        mv: Move,
    ) -> i32 {
        let stm = board.side_to_move();
        let piece = board.moving_piece(mv);
        let mut history = self.td.hist.quiet(threats, stm, mv);
        for back in [1, 2] {
            if let Some(context) = self.cont_context_back(ply, back) {
                history += self.td.hist.cont(context, stm, piece, mv.to_sq());
            }
        }
        history
    }

    /// Update the continuation histories of the move `piece` to `to`, made
    /// at `ply` by `color`, in every context it follows.
    pub(super) fn update_continuations(
        &mut self,
        ply: usize,
        color: Color,
        piece: Piece,
        to: Square,
        bonus: i32,
    ) {
        for back in CONT_PLY_BACK {
            if let Some(context) = self.cont_context_back(ply, back) {
                self.td.hist.update_cont(context, color, piece, to, bonus);
            }
        }
    }
}

/// Up to 32 moves of one class a node searched without raising alpha; the
/// moves past 32 are not kept. Nothing is written until a move is pushed, as
/// every node makes two.
pub(super) struct SearchedMoves(FixedList<Move, 32>);

impl SearchedMoves {
    pub(super) const fn new() -> Self {
        Self(FixedList::new())
    }

    pub(super) fn push(&mut self, mv: Move) {
        if self.0.len() < 32 {
            self.0.push(mv);
        }
    }

    pub(super) fn as_slice(&self) -> &[Move] {
        self.0.as_slice()
    }
}

#[cfg(test)]
mod tests;
