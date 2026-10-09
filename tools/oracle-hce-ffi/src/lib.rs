//! Native adapter exposing Rarog's HCE to the pre-NNUE Stockfish search.
//!
//! Same ABI as the `oracle/hybrid` tag's adapter; the position is rebuilt
//! through `Board::from_fen` (Rarog's public constructor) instead of the
//! tag's private snapshot constructor, which current Rarog does not have.

use std::cell::RefCell;

use rarog::board::{Board, CastlingRights, Color};
use rarog::eval::Evaluator;

const ABI_VERSION: u32 = 1;
const PIECE_LETTERS: [u8; 12] = *b"PNBRQKpnbrqk";

thread_local! {
    /// Stockfish owns one searcher per OS thread. Matching that ownership here
    /// keeps Rarog's mutable pawn/eval caches private without locks.
    static EVALUATOR: RefCell<Evaluator> = RefCell::new(Evaluator::default());
}

/// ABI handshake checked when the hybrid loads the DLL.
#[unsafe(no_mangle)]
pub extern "C" fn rarog_hce_abi_version() -> u32 {
    ABI_VERSION
}

/// The FEN of a snapshot: twelve bitboards in `[WP..WK, BP..BK]` order (bit 0
/// is a1), the side to move, Rarog's castling mask and the rule-50 clock.
/// En passant is absent, as in the tag's adapter: no HCE term reads it.
fn snapshot_fen(pieces: &[u64; 12], side: Color, castling: u8, halfmove: u8) -> Option<String> {
    if castling & !0xF != 0 {
        return None;
    }
    let mut board = [0u8; 64];
    for (index, &bits) in pieces.iter().enumerate() {
        let mut bits = bits;
        while bits != 0 {
            let square = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            if board[square] != 0 {
                return None;
            }
            board[square] = PIECE_LETTERS[index];
        }
    }
    if pieces[5].count_ones() != 1 || pieces[11].count_ones() != 1 {
        return None;
    }
    let mut fen = String::with_capacity(90);
    for rank in (0..8).rev() {
        let mut empty = 0;
        for file in 0..8 {
            match board[rank * 8 + file] {
                0 => empty += 1,
                letter => {
                    if empty > 0 {
                        fen.push(char::from(b'0' + empty));
                        empty = 0;
                    }
                    fen.push(char::from(letter));
                }
            }
        }
        if empty > 0 {
            fen.push(char::from(b'0' + empty));
        }
        if rank > 0 {
            fen.push('/');
        }
    }
    fen.push_str(if side == Color::White { " w " } else { " b " });
    let rights = CastlingRights(castling);
    let mut any = false;
    for (flag, letter) in [
        (CastlingRights::WHITE_KINGSIDE, 'K'),
        (CastlingRights::WHITE_QUEENSIDE, 'Q'),
        (CastlingRights::BLACK_KINGSIDE, 'k'),
        (CastlingRights::BLACK_QUEENSIDE, 'q'),
    ] {
        if rights.has(flag) {
            fen.push(letter);
            any = true;
        }
    }
    if !any {
        fen.push('-');
    }
    fen.push_str(&format!(" - {halfmove} 1"));
    Some(fen)
}

/// Evaluate a legal Stockfish position with Rarog's HCE.
///
/// # Safety
///
/// `pieces` must point to twelve readable `u64`s in `[WP..WK, BP..BK]` order.
/// Invalid input returns `i32::MIN`, which the C++ caller treats as fatal
/// rather than letting a corrupt score into the search.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rarog_hce_evaluate(
    pieces: *const u64,
    side_to_move: u8,
    castling: u8,
    halfmove_clock: u8,
) -> i32 {
    if pieces.is_null() {
        return i32::MIN;
    }
    let side = match side_to_move {
        0 => Color::White,
        1 => Color::Black,
        _ => return i32::MIN,
    };
    // SAFETY: the caller contract above requires twelve readable u64 values.
    let input = unsafe { std::slice::from_raw_parts(pieces, 12) };
    let mut snapshot = [0u64; 12];
    snapshot.copy_from_slice(input);
    let Some(fen) = snapshot_fen(&snapshot, side, castling, halfmove_clock) else {
        return i32::MIN;
    };
    // Chess960 castling rights (Stockfish's bench ends with such a position)
    // are not expressible in Rarog's FEN; that position is evaluated without
    // them. In standard chess the rights always parse.
    let board = match Board::from_fen(&fen) {
        Ok(board) => board,
        Err(_) => match snapshot_fen(&snapshot, side, 0, halfmove_clock)
            .and_then(|fen| Board::from_fen(&fen).ok())
        {
            Some(board) => board,
            None => return i32::MIN,
        },
    };
    EVALUATOR.with_borrow_mut(|evaluator| evaluator.evaluate(&board))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rarog::board::Piece;

    fn snapshot(board: &Board) -> [u64; 12] {
        let mut pieces = [0u64; 12];
        for color in [Color::White, Color::Black] {
            for piece in [
                Piece::Pawn,
                Piece::Knight,
                Piece::Bishop,
                Piece::Rook,
                Piece::Queen,
                Piece::King,
            ] {
                pieces[color as usize * 6 + piece as usize] = board.pieces(color, piece).0;
            }
        }
        pieces
    }

    /// The exported function against Rarog's own evaluator on the original
    /// position: they must agree exactly.
    fn assert_exact(fen: &str) {
        let original = Board::from_fen(fen).expect("valid test FEN");
        let fields: Vec<&str> = fen.split_whitespace().collect();
        let castling = fields[2].chars().fold(0u8, |mask, c| {
            mask | match c {
                'K' => 1,
                'Q' => 2,
                'k' => 4,
                'q' => 8,
                _ => 0,
            }
        });
        let halfmove: u8 = fields.get(4).map_or(0, |h| h.parse().expect("clock"));
        let expected = Evaluator::default().evaluate(&original);
        let exported = unsafe {
            rarog_hce_evaluate(
                snapshot(&original).as_ptr(),
                original.side_to_move() as u8,
                castling,
                halfmove,
            )
        };
        assert_eq!(exported, expected, "adapter mismatch for {fen}");
    }

    #[test]
    fn the_adapter_reproduces_the_evaluator_exactly() {
        for fen in [
            rarog::board::STARTING_FEN,
            "r3k2r/ppp2ppp/2n1bn2/3qp3/3P4/2N1BN2/PPP1QPPP/R3K2R w KQkq - 7 12",
            "8/5pk1/6p1/3P3p/4P3/5K2/8/8 b - - 73 54",
            "8/2p5/2P5/3K4/8/8/6k1/8 w - - 99 80",
        ] {
            assert_exact(fen);
        }
    }

    #[test]
    fn the_adapter_reproduces_the_evaluator_over_random_games() {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut checked = 0;
        for _ in 0..60 {
            let mut board = Board::from_fen(rarog::board::STARTING_FEN).expect("start");
            for _ in 0..90 {
                assert_exact(&board.to_fen());
                checked += 1;
                let moves = board.generate_legal_movelist();
                if moves.is_empty() {
                    break;
                }
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let index = usize::try_from(state % moves.len() as u64).expect("index");
                board.make_move(moves.as_slice()[index]);
            }
        }
        assert!(checked > 4000, "only {checked} positions");
    }

    #[test]
    fn chess960_castling_rights_are_dropped_not_fatal() {
        // Stockfish's last bench position, after its moves: the king on g8/g1
        // with rights to the h- and f-file rooks.
        let fen = "bbqnnrkr/pppppppp/8/8/8/8/PPPPPPPP/BBQNNRKR w - - 0 1";
        let board = Board::from_fen(fen).expect("valid without castling");
        let pieces = snapshot(&board);
        let with_rights = unsafe { rarog_hce_evaluate(pieces.as_ptr(), 0, 15, 0) };
        assert_ne!(with_rights, i32::MIN);
        assert_eq!(with_rights, Evaluator::default().evaluate(&board));
    }

    #[test]
    fn invalid_input_is_refused() {
        let board = Board::from_fen(rarog::board::STARTING_FEN).expect("start");
        let mut pieces = snapshot(&board);
        pieces[11] = 0; // no black king
        assert_eq!(
            unsafe { rarog_hce_evaluate(pieces.as_ptr(), 0, 15, 0) },
            i32::MIN
        );
        assert_eq!(
            unsafe { rarog_hce_evaluate(snapshot(&board).as_ptr(), 0, 16, 0) },
            i32::MIN
        );
        assert_eq!(
            unsafe { rarog_hce_evaluate(snapshot(&board).as_ptr(), 2, 15, 0) },
            i32::MIN
        );
    }
}
