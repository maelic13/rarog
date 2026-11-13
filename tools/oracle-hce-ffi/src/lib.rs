//! Native adapter exposing Rarog's HCE to a foreign engine's search.
//!
//! Same ABI as the `oracle/hybrid` tag's adapter. The position arrives as
//! bitboards and is rebuilt by `Board::from_placement`, which neither writes
//! nor parses text: an oracle pays for the evaluation it calls, not for a
//! FEN round trip on every node.

use std::cell::RefCell;
use std::hint::black_box;
use std::sync::OnceLock;

use rarog::board::{Bitboard, Board, CastlingRights, Color};
use rarog::eval::Evaluator;

const ABI_VERSION: u32 = 1;

thread_local! {
    /// A foreign search owns one searcher per OS thread. Matching that
    /// ownership here keeps Rarog's mutable pawn/king caches private without
    /// locks.
    static EVALUATOR: RefCell<Evaluator> = RefCell::new(Evaluator::default());
}

/// How many times each call builds the board, read once from
/// `RAROG_HCE_BRIDGE_REPEAT` (default 1). An oracle's time per node at two
/// repeat counts on the same fixed-depth tree prices the bridge in that
/// search: the extra builds change no score, only the time.
fn bridge_repeat() -> u32 {
    static REPEAT: OnceLock<u32> = OnceLock::new();
    *REPEAT.get_or_init(|| {
        std::env::var("RAROG_HCE_BRIDGE_REPEAT")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|&repeat| repeat >= 1)
            .unwrap_or(1)
    })
}

/// ABI handshake checked when the hybrid loads the DLL.
#[unsafe(no_mangle)]
pub extern "C" fn rarog_hce_abi_version() -> u32 {
    ABI_VERSION
}

/// The board of a snapshot: twelve bitboards in `[WP..WK, BP..BK]` order
/// (bit 0 is a1), the side to move, Rarog's castling mask and the rule-50
/// clock. En passant is absent, as in the tag's adapter: no HCE term reads
/// it. Castling rights the placement cannot hold (Chess960 rights, which
/// Stockfish's bench ends with) are dropped rather than refused; in standard
/// chess they always hold.
fn snapshot_board(pieces: [u64; 12], side: Color, castling: u8, halfmove: u8) -> Option<Board> {
    if castling & !0xF != 0 {
        return None;
    }
    let placement = pieces.map(Bitboard);
    Board::from_placement(placement, side, CastlingRights(castling), halfmove)
        .or_else(|_| Board::from_placement(placement, side, CastlingRights(0), halfmove))
        .ok()
}

/// Evaluate a legal position with Rarog's HCE, in centipawns from the side
/// to move.
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
    for _ in 1..bridge_repeat() {
        black_box(snapshot_board(
            black_box(snapshot),
            side,
            castling,
            halfmove_clock,
        ));
    }
    let Some(board) = snapshot_board(snapshot, side, castling, halfmove_clock) else {
        return i32::MIN;
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

    /// The bridge's own price per call: the exported function against
    /// `Evaluator::evaluate` on prebuilt boards, over the same positions in
    /// the same order. Ignored by default; run on an idle host with
    /// `cargo test --release -- --ignored --nocapture bridge_cost`.
    #[test]
    #[ignore]
    fn bridge_cost() {
        use std::time::Instant;

        let mut state = 0x2545_F491_4F6C_DD1D_u64;
        let mut boards = Vec::new();
        while boards.len() < 50_000 {
            let mut board = Board::from_fen(rarog::board::STARTING_FEN).expect("start");
            for _ in 0..120 {
                let moves = board.generate_legal_movelist();
                if moves.is_empty() {
                    break;
                }
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let index = usize::try_from(state % moves.len() as u64).expect("index");
                board.make_move(moves.as_slice()[index]);
                boards.push(Board::from_fen(&board.to_fen()).expect("replay"));
            }
        }
        let inputs: Vec<([u64; 12], u8, u8, u8)> = boards
            .iter()
            .map(|board| {
                (
                    snapshot(board),
                    board.side_to_move() as u8,
                    board.castling().0,
                    board.halfmove_clock(),
                )
            })
            .collect();
        let rounds = 20;
        let mut direct_ns = Vec::new();
        let mut bridged_ns = Vec::new();
        for _ in 0..7 {
            let mut evaluator = Evaluator::default();
            let start = Instant::now();
            let mut sum = 0i64;
            for _ in 0..rounds {
                for board in &boards {
                    sum += i64::from(evaluator.evaluate(black_box(board)));
                }
            }
            direct_ns.push(start.elapsed().as_nanos() as f64 / (rounds * boards.len()) as f64);
            let start = Instant::now();
            let mut bridged = 0i64;
            for _ in 0..rounds {
                for (pieces, side, castling, clock) in &inputs {
                    bridged += i64::from(unsafe {
                        rarog_hce_evaluate(black_box(pieces.as_ptr()), *side, *castling, *clock)
                    });
                }
            }
            bridged_ns.push(start.elapsed().as_nanos() as f64 / (rounds * boards.len()) as f64);
            assert_eq!(sum, bridged, "the bridge changed a score");
        }
        direct_ns.sort_by(f64::total_cmp);
        bridged_ns.sort_by(f64::total_cmp);
        let (direct, bridged) = (direct_ns[3], bridged_ns[3]);
        println!(
            "positions {} rounds {rounds} | evaluate {direct:.1} ns | bridged {bridged:.1} ns | bridge {:.1} ns a call ({:.1}% of a bridged call)",
            boards.len(),
            bridged - direct,
            100.0 * (bridged - direct) / bridged
        );
    }
}
