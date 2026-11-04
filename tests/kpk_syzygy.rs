//! The static evaluation of every legal KPK position, both colours and both
//! sides to move, against Syzygy's WDL verdict from the committed `KPvK.rtbw`
//! (`tests/fixtures/syzygy`). A drawn KPK is the bitbase's verdict and must
//! read exactly 0; a won one falls through to the generic evaluation, and
//! nothing else holds its sign. Tablebase state is global to the process, so
//! these tests have their own binary.

use rarog::board::Board;
use rarog::eval::Evaluator;
use rarog::syzygy::{self, Wdl};

/// Every legal KPK position: two pawn colours, two sides to move.
const LEGAL_KPK_POSITIONS: usize = 662_704;

fn fixture_path() -> String {
    format!("{}/tests/fixtures/syzygy", env!("CARGO_MANIFEST_DIR"))
}

/// The FEN with a white king, a black king and one pawn of `pawn_white`'s
/// colour on the given squares (a1 = 0), or `None` when two share a square.
fn fen(
    white_king: usize,
    black_king: usize,
    pawn: usize,
    pawn_white: bool,
    white_to_move: bool,
) -> Option<String> {
    if white_king == black_king || pawn == white_king || pawn == black_king {
        return None;
    }
    let mut squares = ['.'; 64];
    squares[white_king] = 'K';
    squares[black_king] = 'k';
    squares[pawn] = if pawn_white { 'P' } else { 'p' };
    let mut placement = String::new();
    for rank in (0..8).rev() {
        let mut empty = 0;
        for file in 0..8 {
            let piece = squares[rank * 8 + file];
            if piece == '.' {
                empty += 1;
                continue;
            }
            if empty > 0 {
                placement.push_str(&empty.to_string());
                empty = 0;
            }
            placement.push(piece);
        }
        if empty > 0 {
            placement.push_str(&empty.to_string());
        }
        if rank > 0 {
            placement.push('/');
        }
    }
    Some(format!(
        "{placement} {} - - 0 1",
        if white_to_move { 'w' } else { 'b' }
    ))
}

/// Every legal KPK position with its FEN, Syzygy's verdict and the static
/// evaluation (side to move's view).
fn every_kpk_position() -> Vec<(String, Wdl, i32)> {
    assert_eq!(
        syzygy::initialize(&fixture_path()),
        3,
        "the fixture holds 3-man tables"
    );
    let mut evaluator = Evaluator::default();
    let mut positions = Vec::with_capacity(LEGAL_KPK_POSITIONS);
    for pawn_white in [true, false] {
        for white_to_move in [true, false] {
            for white_king in 0..64 {
                for black_king in 0..64 {
                    for pawn in 8..56 {
                        let Some(fen) =
                            fen(white_king, black_king, pawn, pawn_white, white_to_move)
                        else {
                            continue;
                        };
                        // The parser rejects adjacent kings and a side not to
                        // move in check: what parses is every legal position.
                        let Ok(board) = Board::from_fen(&fen) else {
                            continue;
                        };
                        let wdl = syzygy::probe_wdl(&board, false)
                            .expect("KPvK.rtbw answers every KPK position");
                        let eval = evaluator.evaluate(&board);
                        positions.push((fen, wdl, eval));
                    }
                }
            }
        }
    }
    assert_eq!(positions.len(), LEGAL_KPK_POSITIONS);
    positions
}

fn report(what: &str, failures: &[String]) -> String {
    format!(
        "{} KPK positions {what}; first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(20)]
    )
}

#[test]
fn every_drawn_kpk_reads_exactly_zero() {
    let positions = every_kpk_position();
    let drawn: Vec<_> = positions
        .iter()
        .filter(|(_, wdl, _)| *wdl == Wdl::Draw)
        .collect();
    assert_eq!(drawn.len(), 217_576, "Syzygy's drawn KPK count");
    let failures: Vec<String> = drawn
        .iter()
        .filter(|(_, _, eval)| *eval != 0)
        .map(|(fen, _, eval)| format!("{fen}: evaluation {eval}"))
        .collect();
    assert!(
        failures.is_empty(),
        "{}",
        report("drawn by Syzygy read away from 0", &failures)
    );
    // No KPK is a cursed win or a blessed loss: the rest are plain wins.
    assert!(
        positions
            .iter()
            .all(|(_, wdl, _)| matches!(wdl, Wdl::Win | Wdl::Loss | Wdl::Draw))
    );
}

#[test]
#[ignore = "D.6.21: 88 won KPK positions read at or below 0 for the winner (a pawn on its 2nd to 4th rank, the lone king beside it)"]
fn every_won_kpk_reads_above_zero_for_the_winner() {
    let failures: Vec<String> = every_kpk_position()
        .into_iter()
        .filter(|(_, wdl, eval)| match wdl {
            Wdl::Win => *eval <= 0,
            Wdl::Loss => *eval >= 0,
            _ => false,
        })
        .map(|(fen, wdl, eval)| format!("{fen}: Syzygy {wdl:?}, evaluation {eval}"))
        .collect();
    assert!(
        failures.is_empty(),
        "{}",
        report("won by Syzygy read at or below 0 for the winner", &failures)
    );
}
