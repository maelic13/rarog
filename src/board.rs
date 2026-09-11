// 9.0: `clippy::module_inception` allowed. `board::board` holds the `Board`
// type itself while the sibling modules hold its supporting concepts
// (bitboard, movegen, moves, piece, square, zobrist). Renaming it would churn
// every `use crate::board::board::…` path and every commit that references
// them, for a naming-style lint with no functional effect. The re-exports
// below mean callers write `crate::board::Board` regardless.
#![allow(clippy::module_inception)]

pub mod attacks;
pub mod bitboard;
pub mod board;
pub mod movegen;
pub mod moves;
pub mod piece;
pub mod square;
pub mod zobrist;

// Convenient re-exports of the most commonly used types.
pub use attacks::ATTACKS;
pub use bitboard::Bitboard;
pub use board::{
    Board, CROSS_ENGINE_SEE_VALUES, CheckInfo, GameResult, PRODUCTION_SEE_VALUES, STARTING_FEN,
    SeeValues,
};
pub use movegen::{
    generate_captures, generate_captures_into, generate_legal_into, generate_legal_movelist,
    generate_legal_moves, perft,
};
pub use moves::{Move, MoveList};
pub use piece::{CastlingRights, Color, Piece};
pub use square::{File, Rank, Square};
pub use zobrist::ZOBRIST;
