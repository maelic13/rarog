//! Phase 3 gate — per-term eval regression assertions (the production half).
//!
//! **Colour symmetry.** A correct evaluation must be invariant under a
//! colour-flip + vertical mirror: mirroring every piece to the other colour,
//! flipping the board top-to-bottom, and swapping the side to move yields the
//! *same position from the other player's view*, so `evaluate()` (side-to-move
//! relative) must return the identical score. This single invariant exercises
//! every term at once — any colour-asymmetric bug (a sign error, a one-sided
//! table, the kind of inverted-corner mistake found in 3.11) breaks it.
//!
//! (The trace-side assertions — nonzero activation and seeded-zero liveness —
//! need the eval trace and live in `src/eval.rs` under `--features texel`.)

use rarog::board::Board;
use rarog::eval::Evaluator;

/// Build the colour-flipped, vertically-mirrored FEN of `fen`.
fn mirror_fen(fen: &str) -> String {
    let p: Vec<&str> = fen.split_whitespace().collect();

    // Piece placement: reverse the rank order (vertical flip) and swap the case
    // of every piece letter (colour flip).
    let placement = p[0]
        .split('/')
        .rev()
        .map(|rank| {
            rank.chars()
                .map(|c| {
                    if c.is_ascii_uppercase() {
                        c.to_ascii_lowercase()
                    } else if c.is_ascii_lowercase() {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("/");

    let stm = if p[1] == "w" { "b" } else { "w" };

    // Castling rights: swap colour (case) then re-emit in canonical KQkq order.
    let castling = if p[2] == "-" {
        "-".to_string()
    } else {
        let swapped: String = p[2]
            .chars()
            .map(|c| {
                if c.is_ascii_uppercase() {
                    c.to_ascii_lowercase()
                } else {
                    c.to_ascii_uppercase()
                }
            })
            .collect();
        let mut out: String = ['K', 'Q', 'k', 'q']
            .into_iter()
            .filter(|w| swapped.contains(*w))
            .collect();
        if out.is_empty() {
            out.push('-');
        }
        out
    };

    // En passant: file unchanged, rank mirrored (3<->6 etc.).
    let ep = if p[3] == "-" {
        "-".to_string()
    } else {
        let b = p[3].as_bytes();
        let new_rank = (b'8' - (b[1] - b'1')) as char;
        format!("{}{}", b[0] as char, new_rank)
    };

    let half = p.get(4).copied().unwrap_or("0");
    let full = p.get(5).copied().unwrap_or("1");
    format!("{placement} {stm} {castling} {ep} {half} {full}")
}

fn eval(fen: &str) -> i32 {
    let board = Board::from_fen(fen).unwrap_or_else(|e| panic!("bad FEN {fen}: {e}"));
    Evaluator::default().evaluate(&board)
}

const SEEDS: [&str; 10] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "r1bq1rk1/pp2bppp/2n2n2/2pp4/3P4/2N1PN2/PP2BPPP/R1BQ1RK1 w - - 0 8",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/p1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    "5k2/5p1p/p3B1p1/Pp6/1P6/5P1P/4K1P1/8 b - - 0 1",
    "2r3k1/1q2Rp1p/p2p2p1/1p1P4/1Pp1P3/2Q5/1P4PP/6K1 w - - 0 1",
    "r1bqkb1r/pp1p1ppp/2n1pn2/2p5/4P3/2NP1N2/PPP2PPP/R1BQKB1R w KQkq - 0 5",
    "8/8/p1p5/1p5p/1P5P/P1P5/8/K1k5 w - - 0 1",
    "6k1/p3q2p/1nr3p1/8/3Q4/7P/PP4P1/4R1K1 b - - 0 1",
];

#[test]
fn eval_is_colour_symmetric() {
    let mut checked = 0usize;
    for seed in SEEDS {
        let mut board = Board::from_fen(seed).unwrap();
        for _ in 0..40 {
            let fen = board.to_fen();
            let mirror = mirror_fen(&fen);
            assert_eq!(
                eval(&fen),
                eval(&mirror),
                "eval not colour-symmetric:\n  pos    = {fen}\n  mirror = {mirror}"
            );
            checked += 1;

            let moves = board.generate_legal_moves();
            if moves.is_empty() {
                break;
            }
            let mut moves = moves;
            moves.sort_by_key(|m| m.to_string());
            let mv = moves[0];
            board.make_move(mv);
        }
    }
    assert!(checked > 200, "expected a broad sample, checked {checked}");
}
