//! 9.5 seeded random-walk differential test.
//!
//! `Board` keeps five redundant representations of one position: the 12 piece
//! bitboards, the two occupancies, `all_occ`, the mailbox, and five Zobrist
//! keys. Before this file, only two of them were ever checked — the existing
//! make/unmake test compares `to_fen()`, `hash`, `is_in_check()` and
//! `checkers`, and `to_fen()` reads the MAILBOX, so the FEN comparison
//! validates the mailbox and nothing else.
//!
//! The unchecked half is the dangerous half. `pawn_key`, `minor_key` and the
//! two `non_pawn_key`s are CACHE KEYS — they index the pawn cache, correction
//! history and pawn history. A drifted key does not crash; it silently
//! returns another position's cached evaluation, which surfaces much later as
//! an unexplained Elo loss.
//!
//! This walks random legal moves from several structurally different roots,
//! rebuilding every derived field from the mailbox after each make and each
//! unmake. Seeded, so a failure reproduces exactly.
//!
//! Phase 11 note: NNUE accumulators are this same bug class one level worse —
//! incrementally maintained state that must always equal a from-scratch
//! rebuild. This harness is deliberately shaped to extend to them.

use rarog::board::{Board, Move};

/// Roots chosen for structural variety, not difficulty: castling rights on
/// both sides, en-passant availability, promotion races, and a position with
/// no pawns at all (so the pawn key must stay 0 through every move).
const ROOTS: [(&str, &str); 6] = [
    (
        "startpos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    (
        "kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ),
    (
        "ep-available",
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
    ),
    ("promotion-race", "8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1"),
    ("no-pawns", "r3k2r/8/8/4b3/2N5/8/8/R3K2R w KQkq - 0 1"),
    ("endgame", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
];

/// xorshift64*, so a seed reproduces a walk exactly.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        let n64 = u64::try_from(n).expect("slice lengths fit u64");
        usize::try_from(self.next() % n64).expect("value below n fits usize")
    }
}

/// Everything a position is, including the fields `to_fen()` cannot express.
#[derive(Clone, PartialEq, Eq, Debug)]
struct FullState {
    fen: String,
    hash: u64,
    pawn_key: u64,
    minor_key: u64,
    non_pawn: [u64; 2],
    checkers: u64,
    all_occ: u64,
    pieces: [u64; 12],
}

impl FullState {
    fn of(board: &Board) -> Self {
        use rarog::board::{Color, Piece};
        let mut pieces = [0u64; 12];
        for (i, slot) in pieces.iter_mut().enumerate() {
            let color = if i < 6 { Color::White } else { Color::Black };
            let piece = match i % 6 {
                0 => Piece::Pawn,
                1 => Piece::Knight,
                2 => Piece::Bishop,
                3 => Piece::Rook,
                4 => Piece::Queen,
                _ => Piece::King,
            };
            *slot = board.pieces(color, piece).0;
        }
        Self {
            fen: board.to_fen(),
            hash: board.hash(),
            pawn_key: board.pawn_key(),
            minor_key: board.minor_key(),
            non_pawn: [
                board.non_pawn_key(Color::White),
                board.non_pawn_key(Color::Black),
            ],
            checkers: board.checkers().0,
            all_occ: board.occupied().0,
            pieces,
        }
    }
}

fn walk(name: &str, fen: &str, seed: u64, plies: usize) {
    let mut board = Board::from_fen(fen).expect("root FEN is legal");
    board
        .check_consistency()
        .unwrap_or_else(|e| panic!("[{name}] root position is already inconsistent: {e}"));

    let mut rng = Rng(seed);
    let mut played: Vec<(Move, FullState)> = Vec::new();

    for ply in 0..plies {
        let moves = board.generate_legal_moves();
        if moves.is_empty() {
            break;
        }
        let before = FullState::of(&board);
        let mv = moves[rng.below(moves.len())];
        board.make_move(mv);

        board.check_consistency().unwrap_or_else(|e| {
            panic!(
                "[{name} seed {seed} ply {ply}] after make of {mv}: {e}\nfrom FEN {}",
                before.fen
            )
        });

        played.push((mv, before));
    }

    // Unwind, asserting FULL field equivalence at every step — not just the
    // four fields the older snapshot compared.
    while let Some((mv, before)) = played.pop() {
        board.unmake_move(mv);
        board
            .check_consistency()
            .unwrap_or_else(|e| panic!("[{name} seed {seed}] after unmake of {mv}: {e}"));
        let after = FullState::of(&board);
        assert_eq!(
            after, before,
            "[{name} seed {seed}] unmake of {mv} did not restore the position"
        );
    }
}

#[test]
fn random_walks_keep_every_derived_field_in_sync() {
    for (name, fen) in ROOTS {
        for seed in 1..=250u64 {
            walk(name, fen, seed.wrapping_mul(0x9E37_79B9_7F4A_7C15), 60);
        }
    }
}

#[test]
fn deep_walk_from_startpos() {
    // One long walk, to reach positions a 40-ply walk rarely produces
    // (bare-king endings, mass promotion, exhausted castling rights).
    for seed in 1..=60u64 {
        walk(
            "startpos-deep",
            ROOTS[0].1,
            seed.wrapping_mul(0xD1B5_4A32_D192_ED03),
            400,
        );
    }
}
