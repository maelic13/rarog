//! Upcoming-repetition detection: every reversible piece move stored under
//! its key, so one probe says whether two positions in the key history lie a
//! single move apart.
//!
//! The method is M. N. J. van Kervinck's ("The design and implementation of
//! the upcoming repetition detection", 2013), which has one natural form: a
//! knight, bishop, rook, queen or king move between two squares the piece
//! attacks on an empty board is stored under the XOR of its two
//! piece-square keys and the side key, in a cuckoo hash table probed by two
//! functions of the key's high bits. A position's key XOR an earlier one's
//! names the move that would join them, if one exists.

use super::attacks::{bishop_attacks_slow, king_attacks_of, knight_attacks_of, rook_attacks_slow};
use super::bitboard::Bitboard;
use super::piece::{Color, Piece};
use super::square::Square;
use super::zobrist::ZobristKeys;
use crate::infra;

const SLOTS: usize = 8192;
/// A slot's squares when it holds no move.
const NO_SQUARE: u8 = 64;
/// Moves stored: 2 colours times the knight's 168, the bishop's 280, the
/// rook's 448, the queen's 728 and the king's 210 square pairs.
const REVERSIBLE_MOVES: usize = 3_668;

/// The table: each slot's key and the move's two squares.
pub(super) struct Cuckoo {
    keys: [u64; SLOTS],
    from: [u8; SLOTS],
    to: [u8; SLOTS],
}

/// Built by the compiler from the same keys as the board's hash.
pub(super) static CUCKOO: Cuckoo = Cuckoo::build();

const fn first_slot(key: u64) -> usize {
    infra::index((key >> 32) & 0x1FFF)
}

const fn second_slot(key: u64) -> usize {
    infra::index((key >> 48) & 0x1FFF)
}

/// The squares `piece` attacks from `sq` on an empty board.
const fn empty_board_attacks(piece: Piece, sq: u8) -> u64 {
    let square = Square(sq);
    let single = Bitboard(1 << sq);
    match piece {
        Piece::Knight => knight_attacks_of(single).0,
        Piece::Bishop => bishop_attacks_slow(square, 0),
        Piece::Rook => rook_attacks_slow(square, 0),
        Piece::Queen => bishop_attacks_slow(square, 0) | rook_attacks_slow(square, 0),
        Piece::King => king_attacks_of(single).0,
        Piece::Pawn => 0,
    }
}

impl Cuckoo {
    const fn build() -> Self {
        let zobrist = ZobristKeys::init();
        let mut table = Self {
            keys: [0; SLOTS],
            from: [NO_SQUARE; SLOTS],
            to: [NO_SQUARE; SLOTS],
        };
        let pieces = [
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ];
        let mut stored = 0;
        let mut color = 0;
        while color < 2 {
            let side = if color == 0 {
                Color::White
            } else {
                Color::Black
            };
            let mut p = 0;
            while p < pieces.len() {
                let piece = pieces[p];
                let mut a = 0u8;
                while a < 64 {
                    let attacks = empty_board_attacks(piece, a);
                    let mut b = a + 1;
                    while b < 64 {
                        if attacks & (1 << b) != 0 {
                            table.insert(zobrist.move_key(side, piece, a, b), a, b);
                            stored += 1;
                        }
                        b += 1;
                    }
                    a += 1;
                }
                p += 1;
            }
            color += 1;
        }
        assert!(stored == REVERSIBLE_MOVES);
        table
    }

    /// Store a move, displacing an occupant to its other slot until one
    /// lands in an empty slot.
    const fn insert(&mut self, key: u64, from: u8, to: u8) {
        let (mut key, mut from, mut to) = (key, from, to);
        let mut slot = first_slot(key);
        loop {
            let held = (self.keys[slot], self.from[slot], self.to[slot]);
            self.keys[slot] = key;
            self.from[slot] = from;
            self.to[slot] = to;
            (key, from, to) = held;
            if from == NO_SQUARE {
                return;
            }
            slot = if slot == first_slot(key) {
                second_slot(key)
            } else {
                first_slot(key)
            };
        }
    }

    /// The two squares of the reversible move whose key change is `key`,
    /// if there is one.
    #[inline(always)]
    pub(super) fn reversible_move(&self, key: u64) -> Option<(Square, Square)> {
        let mut slot = first_slot(key);
        if self.keys[slot] != key {
            slot = second_slot(key);
            if self.keys[slot] != key {
                return None;
            }
        }
        Some((Square(self.from[slot]), Square(self.to[slot])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::zobrist::ZOBRIST;

    #[test]
    fn every_reversible_move_is_found_by_its_key_and_nothing_else_is() {
        let held = CUCKOO.keys.iter().filter(|&&key| key != 0).count();
        assert_eq!(held, REVERSIBLE_MOVES);
        let knight = ZOBRIST.piece(Color::White, Piece::Knight, Square(6))
            ^ ZOBRIST.piece(Color::White, Piece::Knight, Square(21))
            ^ ZOBRIST.side();
        assert_eq!(
            CUCKOO.reversible_move(knight),
            Some((Square(6), Square(21))),
            "g1-f3"
        );
        let pawn = ZOBRIST.piece(Color::White, Piece::Pawn, Square(12))
            ^ ZOBRIST.piece(Color::White, Piece::Pawn, Square(20))
            ^ ZOBRIST.side();
        assert_eq!(CUCKOO.reversible_move(pawn), None, "a pawn never returns");
        let leap = ZOBRIST.piece(Color::Black, Piece::Rook, Square(0))
            ^ ZOBRIST.piece(Color::Black, Piece::Rook, Square(9))
            ^ ZOBRIST.side();
        assert_eq!(CUCKOO.reversible_move(leap), None, "a1-b2 is no rook move");
    }
}
