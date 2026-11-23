//! SEE pin-awareness tests, checked against an *independent* legal
//! capture-tree oracle (not a `see_ge`-vs-`see` mirror).
//!
//! The oracle plays the exchange out with real legal moves via
//! `generate_legal_moves` + make/unmake, so pins, pin-ray-legal captures,
//! king-into-defended-square, en passant and promotions are all handled by the
//! move generator itself. `board.see()` must match it on these curated
//! positions. SEE's own material scale (P/N/B/R/Q = 100/320/330/500/900) is
//! replicated here so the two agree on value.

use rarog::board::{Board, Move, Piece, Square};

fn see_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20_000,
    }
}

fn capture_gain(board: &Board, mv: Move) -> i32 {
    let mut gain = board.captured_piece(mv).map_or(0, see_value);
    if mv.is_promo() {
        gain += see_value(mv.promo_piece()) - see_value(Piece::Pawn);
    }
    gain
}

/// Legal-move SEE oracle: value of `mv` followed by optimal least-valuable
/// legal recaptures on the same square by each side (each side stops once the
/// exchange turns unfavorable). Independent of `Board::see`'s swap algorithm.
fn oracle_see(board: &mut Board, mv: Move) -> i32 {
    let target = mv.to_sq();
    let gain = capture_gain(board, mv);
    board.make_move(mv);
    let reply = oracle_recapture(board, target);
    board.unmake_move(mv);
    gain - reply
}

fn oracle_recapture(board: &mut Board, target: Square) -> i32 {
    // Least-valuable legal capture landing on `target`.
    let lva = board
        .generate_legal_moves()
        .into_iter()
        .filter(|m| m.is_capture() && m.to_sq() == target)
        .min_by_key(|m| see_value(board.moving_piece(*m)));

    match lva {
        None => 0,
        Some(mv) => {
            let gain = capture_gain(board, mv);
            board.make_move(mv);
            let reply = oracle_recapture(board, target);
            board.unmake_move(mv);
            (gain - reply).max(0)
        }
    }
}

/// Assert `see(mv)` matches the oracle for the capture with the given UCI
/// string, and return the agreed value.
fn check(fen: &str, uci: &str) -> i32 {
    let mut board = Board::from_fen(fen).unwrap_or_else(|e| panic!("{fen}: {e}"));
    let mv = board
        .generate_legal_moves()
        .into_iter()
        .find(|m| m.to_string() == uci)
        .unwrap_or_else(|| panic!("{uci} not legal in {fen}"));
    let see = board.see(mv);
    let oracle = oracle_see(&mut board, mv);
    assert_eq!(
        see, oracle,
        "{fen}: see({uci}) = {see} but legal oracle = {oracle}"
    );
    see
}

// ---- A pinned recapturer must not defend the square ------

#[test]
fn pinned_pawn_cannot_recapture_off_its_file() {
    // Black d7 pawn is pinned to its king (d8) by the white rook on d1, so it
    // cannot play dxc6. White's Bxc6 therefore just wins the c6 pawn.
    let v = check("3k4/3p4/2p5/8/4B3/8/8/3RK3 w - - 0 1", "e4c6");
    assert_eq!(v, 100, "winning a free pawn, no legal recapture");
}

#[test]
fn pinned_by_bishop_diagonal() {
    // Black knight c6 pinned to e8 king by white bishop a4 (a4-b5-c6-d7-e8).
    // A white rook takes c6; the pinned knight cannot recapture off the
    // diagonal, and no other black piece defends c6.
    let v = check("4k3/8/2n5/8/B7/8/8/2R1K3 w - - 0 1", "c1c6");
    assert_eq!(v, 320);
}

// ---- pin-ray-legal: a pinned piece may still capture along its own ray -----

#[test]
fn pinned_pawn_may_capture_the_pinner_on_its_ray() {
    // Black b7 pawn is pinned to a8 along the a8-h1 diagonal by a bishop that
    // sits on c6. bxc6 captures the pinner and stays on the ray — legal — so
    // the recapture counts and the exchange is even-ish.
    let mut board = Board::from_fen("k7/1p6/2B5/8/8/8/8/4K3 b - - 0 1").unwrap();
    let mv = board
        .generate_legal_moves()
        .into_iter()
        .find(|m| m.to_string() == "b7c6")
        .expect("bxc6 is a legal pin-ray capture");
    assert_eq!(board.see(mv), oracle_see(&mut board, mv));
}

// ---- pinner removed mid-exchange re-enables the pinned piece ---------------

#[test]
fn a_pinner_leaving_its_line_frees_the_pinned_recapturer() {
    // The white rook on e2 pins the black bishop on e5 to its king. Rxb2 takes
    // a pawn and leaves the e-file, so the bishop is free and Bxb2 wins the
    // rook: 100 - 500.
    assert_eq!(check("4k3/8/8/4b3/8/8/1p2R3/7K w - - 0 1", "e2b2"), -400);
}

#[test]
fn a_queen_defended_along_its_file() {
    // The white rook on e4 takes the black queen on e6, which the rook on e7
    // defends along the file. Oracle and SEE must agree through the
    // whole sequence.
    check("4k3/4r3/4q3/8/4R3/8/8/4K2R w - - 0 1", "e4e6");
}

// ---- baselines the pin logic must not disturb ------------------------------

#[test]
fn plain_winning_and_losing_and_even_captures() {
    // Rook grabs an undefended pawn: +100.
    assert_eq!(check("4k3/8/8/3p4/8/8/3R4/4K3 w - - 0 1", "d2d5"), 100);
    // Rook takes a defended pawn: 100 - 500 < 0.
    assert!(check("4k3/8/2p5/3p4/8/8/3R4/4K3 w - - 0 1", "d2d5") < 0);
    // Queen takes a queen the enemy king defends: even trade, 0.
    assert_eq!(check("3qk3/8/8/8/8/8/3Q4/3K4 w - - 0 1", "d2d8"), 0);
}

#[test]
fn xray_recapture_still_seen_with_pin_logic() {
    // RxR on d5 backed by a second rook on d1 through the first — a whole rook.
    let v = check("3rk3/8/8/3r4/8/8/3R4/3RK3 w - - 0 1", "d2d5");
    assert!(v >= 500, "x-ray recapture must still be counted, got {v}");
}
