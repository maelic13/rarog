//! Move-ordering histories: threat-aware quiet and noisy tables, pawn-structure
//! and continuation tables, their gravity update and the interim update rules.

use crate::board::{Bitboard, Board, Color, Move, Piece, Square};

use super::Searcher;
use super::movepick::BadCaptureList;

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
const PAWN_MAX: i32 = 8_192;
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

/// One thread's move-ordering tables. None of them age between searches.
pub(super) struct HistoryTables {
    quiet: Box<QuietTable>,
    /// `[coloured piece and destination][captured type][destination threatened]`.
    noisy: Box<[[[i16; 2]; CAPTURED_SLOTS]; PIECE_TO_SIZE]>,
    /// `[pawn-key slot][coloured piece and destination]`, flat.
    pawn: Box<[i16; PAWN_HISTORY_SIZE * PIECE_TO_SIZE]>,
    /// `[context][coloured piece and destination]`, flat; a context is a row
    /// base from [`cont_context`].
    cont: Box<[i16; CONT_SIZE]>,
}

impl Default for HistoryTables {
    fn default() -> Self {
        Self {
            quiet: Box::new([[[[[0; 64]; 64]; 2]; 2]; 2]),
            noisy: Box::new([[[0; 2]; CAPTURED_SLOTS]; PIECE_TO_SIZE]),
            pawn: zeroed(),
            cont: zeroed(),
        }
    }
}

impl HistoryTables {
    /// Forget everything, for a new game.
    pub(super) fn clear(&mut self) {
        *self.quiet = [[[[[0; 64]; 64]; 2]; 2]; 2];
        *self.noisy = [[[0; 2]; CAPTURED_SLOTS]; PIECE_TO_SIZE];
        self.pawn.fill(0);
        self.cont.fill(0);
    }

    /// Between searches the tables keep their values: gravity alone bounds
    /// them.
    #[expect(
        clippy::unused_self,
        reason = "the search calls age() on either arm's tables"
    )]
    pub(super) fn age(&mut self) {}

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
        let (from, to) = (mv.from_sq(), mv.to_sq());
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
        apply_bonus(
            &mut self.pawn[pawn_row + piece_to(color, piece, to)],
            bonus,
            PAWN_MAX,
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

    /// Reward for the move that produced a beta cutoff.
    pub(super) fn history_bonus(&self, depth: i32) -> i32 {
        (self.cfg.params.hist_bonus_mul * depth - self.cfg.params.hist_bonus_sub)
            .clamp(0, self.cfg.params.hist_bonus_max)
    }

    /// Penalty magnitude for searched moves that failed to cut (applied
    /// negated). Stored positive.
    pub(super) fn history_malus(&self, depth: i32) -> i32 {
        (self.cfg.params.hist_malus_mul * depth - self.cfg.params.hist_malus_sub)
            .clamp(0, self.cfg.params.hist_malus_max)
    }

    /// Quiet and pawn tables for one quiet move, plus the continuation terms
    /// when `with_continuation` is set.
    pub(super) fn update_quiet_history(
        &mut self,
        board: &Board,
        threats: Bitboard,
        ply: usize,
        mv: Move,
        bonus: i32,
        with_continuation: bool,
    ) {
        let stm = board.side_to_move();
        let piece = board.moving_piece(mv);
        let to = mv.to_sq();
        self.td.hist.update_quiet(threats, stm, mv, bonus);
        self.td.hist.update_pawn(
            HistoryTables::pawn_row(board.pawn_key()),
            stm,
            piece,
            to,
            bonus,
        );
        if with_continuation {
            for (index, back) in CONT_PLY_BACK.into_iter().enumerate() {
                if let Some(context) = self.cont_context_back(ply, back) {
                    // Interim divisors 1, 1, 2, 3 by look-back distance.
                    let divisor = [1, 1, 2, 3][index];
                    self.td
                        .hist
                        .update_cont(context, stm, piece, to, bonus / divisor);
                }
            }
        }
    }

    pub(super) fn update_noisy_history(
        &mut self,
        board: &Board,
        threats: Bitboard,
        attacker: Piece,
        to: Square,
        captured: Option<Piece>,
        bonus: i32,
    ) {
        self.td
            .hist
            .update_noisy(threats, board.side_to_move(), attacker, to, captured, bonus);
    }

    /// The interim update at a quiet beta cutoff: bonus to the move, malus to
    /// the quiets and captures searched before it.
    pub(super) fn update_cutoff_tables(
        &mut self,
        board: &Board,
        threats: Bitboard,
        best: Move,
        ply: usize,
        depth: i32,
        bonus_pct: i32,
        quiets: &[Move],
        good_caps: &BadCaptureList,
        bad_caps: &BadCaptureList,
    ) {
        let bonus = self.history_bonus(depth) * bonus_pct / 100;
        let malus = self.history_malus(depth);
        self.update_quiet_history(board, threats, ply, best, bonus, true);
        for &quiet in quiets {
            self.update_quiet_history(board, threats, ply, quiet, -malus, false);
        }
        for capture in good_caps.as_slice().iter().chain(bad_caps.as_slice()) {
            self.update_noisy_history(
                board,
                threats,
                capture.attacker,
                Square(capture.to),
                capture.captured,
                -malus,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each table's entries stay inside `±max` under any sequence of
    /// clamped bonuses, and the update is gravity: repeated identical bonuses
    /// converge on the bound without crossing it, and a saturated entry moves
    /// less than an empty one.
    #[test]
    fn every_table_is_bounded_and_updates_by_gravity() {
        for max in [QUIET_MAX, NOISY_MAX, PAWN_MAX, CONT_MAX] {
            let mut entry = 0i16;
            let mut previous = 0;
            for _ in 0..200 {
                apply_bonus(&mut entry, 3 * max, max);
                let value = i32::from(entry);
                assert!(value <= max && value >= previous, "max {max}: {value}");
                previous = value;
            }
            assert!(i32::from(entry) >= max - max / 64, "max {max} converges");
            for _ in 0..200 {
                apply_bonus(&mut entry, -3 * max, max);
                assert!(i32::from(entry) >= -max, "max {max}: {entry}");
            }
            let mut empty = 0i16;
            let mut high = i16::try_from(max * 3 / 4).expect("fits");
            apply_bonus(&mut empty, 1_000, max);
            let before = high;
            apply_bonus(&mut high, 1_000, max);
            assert!(high - before < empty, "saturated entry moves less");
        }
    }

    #[test]
    fn tables_index_by_threat_and_colour() {
        let mut tables = HistoryTables::default();
        let mv = Move::from_uci("g1f3").expect("valid move");
        let threatened_to = Bitboard::from(Square::F3);
        tables.update_quiet(threatened_to, Color::White, mv, 500);
        assert_eq!(tables.quiet(threatened_to, Color::White, mv), 500);
        assert_eq!(tables.quiet(Bitboard::EMPTY, Color::White, mv), 0);
        assert_eq!(tables.quiet(threatened_to, Color::Black, mv), 0);

        tables.update_noisy(
            Bitboard::EMPTY,
            Color::Black,
            Piece::Queen,
            Square::D4,
            None,
            700,
        );
        assert_eq!(
            tables.noisy(
                Bitboard::EMPTY,
                Color::Black,
                Piece::Queen,
                Square::D4,
                None
            ),
            700
        );
        assert_eq!(
            tables.noisy(
                Bitboard::EMPTY,
                Color::Black,
                Piece::Queen,
                Square::D4,
                Some(Piece::Pawn)
            ),
            0
        );
        assert_eq!(
            tables.noisy(
                Bitboard::from(Square::D4),
                Color::Black,
                Piece::Queen,
                Square::D4,
                None
            ),
            0
        );

        let row = HistoryTables::pawn_row(0xDEAD_BEEF);
        tables.update_pawn(row, Color::White, Piece::Knight, Square::F3, -300);
        assert_eq!(
            tables.pawn(row, Color::White, Piece::Knight, Square::F3),
            -300
        );
        assert_eq!(tables.pawn(row, Color::Black, Piece::Knight, Square::F3), 0);

        let context = cont_context(true, false, Color::Black, Piece::Pawn, Square::E5);
        tables.update_cont(context, Color::White, Piece::Knight, Square::F3, 900);
        assert_eq!(
            tables.cont(context, Color::White, Piece::Knight, Square::F3),
            900
        );
        let other = cont_context(false, false, Color::Black, Piece::Pawn, Square::E5);
        assert_eq!(
            tables.cont(other, Color::White, Piece::Knight, Square::F3),
            0
        );

        tables.clear();
        assert_eq!(tables.quiet(threatened_to, Color::White, mv), 0);
        assert_eq!(
            tables.cont(context, Color::White, Piece::Knight, Square::F3),
            0
        );
    }

    /// The largest context plus the largest piece-to slot is the last entry.
    #[test]
    fn continuation_indexes_cover_the_table_exactly() {
        let last = cont_context(true, true, Color::Black, Piece::King, Square::H8)
            + piece_to(Color::Black, Piece::King, Square::H8);
        assert_eq!(last, CONT_SIZE - 1);
        assert_eq!(
            HistoryTables::pawn_row(u64::MAX) + piece_to(Color::Black, Piece::King, Square::H8),
            PAWN_HISTORY_SIZE * PIECE_TO_SIZE - 1
        );
    }
}
