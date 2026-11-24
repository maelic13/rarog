use std::{fmt, mem::MaybeUninit, slice};

use super::piece::Piece;
use super::square::Square;

/// Move flags — classic CPW 4-bit encoding.
///
/// bits 0-5:  from square
/// bits 6-11: to square
/// bits 12-15: flag (see constants below)
#[derive(Copy, Clone, PartialEq, Eq, Default, Hash, Debug)]
pub struct Move(pub u16);

// KEEP-UNSAFE (measured): a plain `[Move; 256]` with `Move::NULL` init
// (and the analogous scored lists) was tried on 2026-07-19 and cost −10% NPS
// on both the pext and magic builds: the per-node initialization of list
// memory is exactly what the `MaybeUninit` pattern avoids. The unsafe is
// confined to `as_slice` and `as_mut_slice`, whose invariant is local: only
// the `..len` prefix is exposed and every element below `len` was written by
// `push` or `retain`.
/// A list of at most `N` items on the stack, nothing written until pushed:
/// the move lists, the scored lists and a node's searched moves.
pub struct FixedList<T: Copy, const N: usize> {
    items: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> FixedList<T, N> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            items: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, item: T) {
        debug_assert!(self.len < N);
        self.items[self.len].write(item);
        self.len += 1;
    }

    /// Reset the list to empty without touching any element: nothing below
    /// the new `len` is read, so a caller-owned list is reusable at no cost.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: only the prefix below `len` is exposed and each element
        // below `len` was initialized by `push` or `retain`.
        unsafe { slice::from_raw_parts(self.items.as_ptr().cast::<T>(), self.len) }
    }

    #[inline(always)]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: only the initialized prefix below `len` is exposed.
        unsafe { slice::from_raw_parts_mut(self.items.as_mut_ptr().cast::<T>(), self.len) }
    }

    /// Keep the items `keep` accepts, in their order, without allocating.
    pub fn retain(&mut self, mut keep: impl FnMut(T) -> bool) {
        let mut kept = 0;
        for read in 0..self.len {
            let item = self.as_slice()[read];
            if keep(item) {
                // `kept <= read`, so every slot below the new `len` holds an
                // item written by `push` or by this loop.
                self.items[kept].write(item);
                kept += 1;
            }
        }
        self.len = kept;
    }
}

impl<T: Copy, const N: usize> Clone for FixedList<T, N> {
    #[inline(always)]
    fn clone(&self) -> Self {
        let mut cloned = Self::new();
        for &item in self.as_slice() {
            cloned.push(item);
        }
        cloned
    }
}

impl<T: Copy, const N: usize> Default for FixedList<T, N> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: Copy, const N: usize> IntoIterator for &'a FixedList<T, N> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The moves of one generation, at most 256.
pub type MoveList = FixedList<Move, 256>;

// Move flag constants (upper 4 bits of a Move)
pub(crate) const QUIET: u16 = 0;
pub(super) const DOUBLE_PUSH: u16 = 1;
pub(crate) const CASTLE_KINGSIDE: u16 = 2;
pub(crate) const CASTLE_QUEENSIDE: u16 = 3;
pub(crate) const CAPTURE: u16 = 4;
pub(super) const EN_PASSANT: u16 = 5;
// 6, 7 unused
pub(crate) const PROMO_KNIGHT: u16 = 8;
pub(super) const PROMO_BISHOP: u16 = 9;
pub(super) const PROMO_ROOK: u16 = 10;
pub(super) const PROMO_QUEEN: u16 = 11;
pub(super) const PROMO_CAPTURE_KNIGHT: u16 = 12;
pub(super) const PROMO_CAPTURE_BISHOP: u16 = 13;
pub(super) const PROMO_CAPTURE_ROOK: u16 = 14;
pub(super) const PROMO_CAPTURE_QUEEN: u16 = 15;

impl Move {
    pub const NULL: Self = Self(0);

    #[inline(always)]
    pub fn new(from: Square, to: Square, flags: u16) -> Self {
        Self(u16::from(from.0) | (u16::from(to.0) << 6) | (flags << 12))
    }

    #[inline(always)]
    pub fn from_sq(self) -> Square {
        Square((self.0 & 0x3F) as u8)
    }

    #[inline(always)]
    pub fn to_sq(self) -> Square {
        Square(((self.0 >> 6) & 0x3F) as u8)
    }

    #[inline(always)]
    pub(crate) fn flags(self) -> u16 {
        self.0 >> 12
    }

    #[inline(always)]
    pub fn is_capture(self) -> bool {
        // flags 4,5,12-15 are captures
        let f = self.flags();
        f == CAPTURE || f == EN_PASSANT || f >= PROMO_CAPTURE_KNIGHT
    }

    #[inline(always)]
    pub fn is_promo(self) -> bool {
        self.flags() >= PROMO_KNIGHT
    }

    /// A capture or any promotion: the board's own split of the moves, which
    /// only tests read. The search reads the kernel's partition instead
    /// (`is_quiet_move`), in which a rook, bishop or knight push is quiet.
    #[inline(always)]
    pub fn is_noisy(self) -> bool {
        self.is_capture() || self.is_promo()
    }

    /// Any move that is not noisy, castling included.
    #[inline(always)]
    pub fn is_quiet(self) -> bool {
        !self.is_noisy()
    }

    #[inline(always)]
    pub fn is_castling(self) -> bool {
        let f = self.flags();
        f == CASTLE_KINGSIDE || f == CASTLE_QUEENSIDE
    }

    #[inline(always)]
    pub fn is_en_passant(self) -> bool {
        self.flags() == EN_PASSANT
    }

    /// Promotion piece (only valid when `is_promo()` is true).
    #[inline(always)]
    pub fn promo_piece(self) -> Piece {
        match self.flags() & 3 {
            0 => Piece::Knight,
            1 => Piece::Bishop,
            2 => Piece::Rook,
            _ => Piece::Queen,
        }
    }

    #[inline(always)]
    pub(crate) fn promotion(self) -> Option<Piece> {
        if self.is_promo() {
            Some(self.promo_piece())
        } else {
            None
        }
    }

    /// Test-only: the move UCI text names with its flags unset, which the
    /// board must canonicalise before it is played. The engine reads text as
    /// a [`UciMove`].
    #[cfg(test)]
    pub(crate) fn from_uci(input: &str) -> Option<Move> {
        UciMove::parse(input).map(|mv| mv.0)
    }

    #[inline(always)]
    pub fn same_uci_move(self, other: Move) -> bool {
        self.from_sq() == other.from_sq()
            && self.to_sq() == other.to_sq()
            && self.promotion() == other.promotion()
    }

    /// Is this a null move sentinel?
    #[inline(always)]
    pub fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// A move as UCI text names it: its squares and promotion piece, without
/// the capture, double-push, castling and en-passant flags the position
/// gives it. Played as it stands it would corrupt make/unmake, so only
/// [`Board::legal_uci_move`](super::Board::legal_uci_move) turns one into a
/// [`Move`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UciMove(pub(in crate::board) Move);

impl UciMove {
    pub fn parse(input: &str) -> Option<UciMove> {
        // UCI moves are ASCII. Check that before byte-indexing: a four-byte
        // UTF-8 token such as `aé1` has no valid [0..2] character boundary.
        if !input.is_ascii() || (input.len() != 4 && input.len() != 5) {
            return None;
        }
        let input = input.to_ascii_lowercase();
        let from = Square::from_algebraic(&input[0..2])?;
        let to = Square::from_algebraic(&input[2..4])?;
        let flags = if input.len() == 5 {
            match Piece::from_promo_char(input.as_bytes()[4] as char)? {
                Piece::Knight => PROMO_KNIGHT,
                Piece::Bishop => PROMO_BISHOP,
                Piece::Rook => PROMO_ROOK,
                Piece::Queen => PROMO_QUEEN,
                _ => return None,
            }
        } else {
            QUIET
        };
        Some(UciMove(Move::new(from, to, flags)))
    }

    /// Whether `mv` is the move this text names: the same squares and
    /// promotion.
    #[inline(always)]
    pub fn names(self, mv: Move) -> bool {
        self.0.same_uci_move(mv)
    }
}

impl From<Move> for UciMove {
    /// The text form of a move: its squares and promotion, its other flags
    /// dropped.
    fn from(mv: Move) -> Self {
        // A promotion's flags are `PROMO_KNIGHT` plus its piece in the low
        // two bits, with or without the capture bit.
        let flags = if mv.is_promo() {
            PROMO_KNIGHT | (mv.flags() & 3)
        } else {
            QUIET
        };
        UciMove(Move::new(mv.from_sq(), mv.to_sq(), flags))
    }
}

impl fmt::Display for UciMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            return write!(f, "0000");
        }
        write!(f, "{}{}", self.from_sq(), self.to_sq())?;
        if self.is_promo() {
            write!(f, "{}", self.promo_piece().promo_char())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `as_slice` exposes `..len` as initialized, so `len` must never pass the
    /// last slot `push` wrote: the push after a full list panics
    /// instead (the debug assertion or, in release, the checked index).
    #[test]
    #[should_panic]
    fn a_push_past_the_last_slot_panics() {
        let mut list = MoveList::new();
        let mv = Move::from_uci("e2e4").expect("valid UCI move shape");
        for _ in 0..256 {
            list.push(mv);
        }
        assert_eq!(list.as_slice().len(), 256);
        list.push(mv);
    }

    /// Every legal move survives the trip through its text form: printed,
    /// parsed or converted, the board gives it back with its flags.
    #[test]
    fn a_legal_move_round_trips_through_uci_move() {
        let mut flagged = std::collections::BTreeSet::new();
        for fen in [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
            "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
            "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1",
        ] {
            let board = crate::board::Board::from_fen(fen).expect("valid FEN");
            let legal = board.generate_legal_movelist();
            flagged.extend(legal.iter().map(|mv| mv.flags()));
            for &mv in &legal {
                let text = UciMove::from(mv);
                assert_eq!(text.to_string(), mv.to_string(), "{fen}");
                assert_eq!(UciMove::parse(&mv.to_string()), Some(text), "{fen} {mv}");
                assert!(text.names(mv), "{fen} {mv}");
                assert_eq!(board.legal_uci_move(text), Some(mv), "{fen} {mv}");
            }
        }
        // Every flag a legal move can carry occurred.
        let all = [0, 1, 2, 3, 4, 5, 8, 9, 10, 11, 12, 13, 14, 15];
        assert_eq!(flagged, std::collections::BTreeSet::from(all));
    }

    #[test]
    fn retain_keeps_the_accepted_moves_in_order() {
        let mut list = MoveList::new();
        for raw in 1..=8 {
            list.push(Move(raw));
        }
        list.retain(|mv| mv.0 % 3 != 0);
        assert_eq!(list.as_slice(), [1, 2, 4, 5, 7, 8].map(Move).as_slice());
        list.retain(|mv| mv.0 > 4);
        assert_eq!(list.as_slice(), [5, 7, 8].map(Move).as_slice());
        list.retain(|_| false);
        assert!(list.is_empty());
    }
}
