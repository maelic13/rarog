pub const DEFAULT_BENCH_DEPTH: u16 = 13;

// 40 positions covering openings, middlegames (quiet + tactical), a broad range
// of endgames, mates, and fortresses. The final "Nodes searched" total is a
// deterministic search fingerprint; the per-position spread also feeds the
// geometric-mean EBF / median / top-share diagnostics printed by `run_bench`.
//
// Positions 1-16 are the original curated suite. Positions 17-40 are legal
// self-play positions sampled across piece counts (30 down to 8) so that no
// single bushy middlegame dominates the node total — the 16-position suite had
// one position at ~35% of all nodes, which made the fingerprint lurch ~15% on
// sub-1-Elo parameter changes (Phase 5.1 finding). See PLAN.md §9.
pub const BENCH_FENS: [&str; 40] = [
    // --- 1-16: original curated suite -----------------------------------------
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
    "8/pp2k3/8/2p5/2P5/1P2K3/P7/8 w - - 0 1",
    "r1bq1r2/pp2n3/4N2k/3pPppP/1b1n2Q1/2N5/PP3PP1/R1B1K2R w KQ g6 0 20",
    "r4rk1/pp1n1pp1/2p1pn1p/q7/3P4/2NB4/PP3PPP/R2QR1K1 w - - 0 1",
    "5k2/5p1p/p3B1p1/Pp6/1P6/5P1P/4K1P1/8 b - - 0 1",
    "6k1/p3q2p/1nr3p1/8/3Q4/7P/PP4P1/4R1K1 b - - 0 1",
    "2r3k1/1q2Rp1p/p2p2p1/1p1P4/1Pp1P3/2Q5/1P4PP/6K1 w - - 0 1",
    "1r3rk1/p4ppp/2p5/3Nb3/1p1bP3/1B4P1/PP3P1P/R2R2K1 b - - 0 1",
    "r2qr1k1/p4ppp/1pn1bn2/2b1p3/4P3/1BN1BN2/PPP2PPP/R2QR1K1 b - - 6 10",
    "r1bqkb1r/pp1p1ppp/2n1pn2/2p5/4P3/2NP1N2/PPP2PPP/R1BQKB1R w KQkq - 0 5",
    "8/8/p1p5/1p5p/1P5P/P1P5/8/K1k5 w - - 0 1",
    "1k6/1b6/8/5p2/p1p2p2/P7/1P3P2/K7 b - - 0 1",
    // --- 17-40: self-play positions, opening/middlegame → endgame ------------
    "1k1rr3/pb3n2/1pnpqbp1/2pNp2p/2P1P2P/P2Q1P2/1PN1BBP1/1K1RR3 b - - 5 10",
    "3r1rk1/1ppb1pb1/p2npqnp/P5p1/3P4/1BN1BN1P/1PP2PP1/3RQR1K w - - 3 10",
    "1b2qrk1/rp3pp1/2p1p2p/p1Pp4/P2Pn3/1Q2PN1P/1P2BPP1/1KR2R2 w - - 3 11",
    "1r3rk1/1pqb1p2/pN1p1bp1/P1pPp3/2P1P2p/1P2QN1P/4RPP1/3R2K1 w - - 2 12",
    "1k1r1r2/pp3pp1/4qn1p/P1p1p3/2p1P3/3PPN1P/1PP3P1/2RQ1RK1 b - - 0 9",
    "1r1q1rk1/3np2p/1n1p2pb/pP1P4/4BP2/2p1BN1P/Q1P1N1P1/5RK1 b - - 0 11",
    "1k1r1n2/1pq1n1b1/2p1p1p1/p2p4/PP1P1PP1/2PB1N2/5B2/2Q1K2R w - - 0 12",
    "1kr1rq2/2p2pR1/p1n1pP2/n2pP3/3P3p/1P3N1P/2P1NQ2/1K1R4 w - - 4 13",
    "1b1rr1k1/1p4q1/1Qp1b1pp/p3p3/P7/2PBN3/1P1R1P2/3R2K1 b - - 1 9",
    "1kr2r2/1p1n1pp1/4p1p1/p2p2P1/3P3P/6P1/PPP3B1/1K1R1R2 b - - 0 10",
    "1Q2n1k1/4bp2/4p1p1/pB1pP2p/q2P1P1P/2P1KNP1/8/8 b - - 6 14",
    "1k2r3/1pp1bpKp/p7/8/2PNr3/1P2P1P1/P4P1P/3R3R b - - 2 9",
    "1B6/1p6/p1p2k2/P1P1p1p1/1P1nPp1p/5P1P/1K4P1/8 b - - 48 110",
    "1k1r4/1b4r1/p3P1n1/1p3pp1/2p5/2N5/1P2R1P1/4RBK1 b - - 4 11",
    "1Q4bk/3R2pp/p7/3p3P/1p6/1B6/P2q1PP1/6K1 w - - 2 17",
    "1R6/5ppk/2N1p2p/4P2P/P3P3/1P3KP1/1r2r3/8 b - - 1 25",
    "1B6/5p1k/3P1b2/p7/2r3Pp/2P4P/1P2R2K/8 b - - 0 9",
    "1R6/5pkp/4qp1b/3p4/7P/6P1/5Q1K/2r2B2 w - - 1 14",
    "1K6/1P1rkp2/B3p3/8/1R1Pb3/5p2/5P2/8 b - - 17 12",
    "1R6/5k2/3ppp2/4p3/P7/1P4P1/2P2r2/1K6 w - - 0 16",
    "1B6/8/1P1nk1p1/3b1p2/3K1P2/3B4/8/8 w - - 9 10",
    "1R6/3q1k2/6p1/7p/4p2P/6P1/5P2/6K1 b - - 3 37",
    "1Q4R1/5k2/4rpp1/3K4/8/7p/8/8 b - - 2 9",
    "1R6/8/4r3/6P1/1pk1b2P/8/3K4/8 b - - 0 11",
];

#[cfg(test)]
mod tests {
    use super::BENCH_FENS;
    use crate::board::Board;

    // Guards against corrupt bench FENs (a copy-paste typo once put 9 black
    // pawns on position 4, silently searched until Basilisk's stricter parser
    // flagged it). `from_fen` now rejects illegal material, so this catches it.
    #[test]
    fn all_bench_positions_are_legal() {
        for (i, fen) in BENCH_FENS.iter().enumerate() {
            assert!(
                Board::from_fen(fen).is_ok(),
                "bench position {} is illegal: {fen}",
                i + 1
            );
        }
    }
}
