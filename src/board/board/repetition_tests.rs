use super::*;

fn played(fen: &str, moves: &[&str]) -> Board {
    let mut board = Board::from_fen(fen).expect("a legal FEN");
    for mv in moves {
        assert!(board.play_uci(mv), "{mv} is legal");
    }
    board
}

fn start(moves: &[&str]) -> Board {
    played(STARTING_FEN, moves)
}

#[test]
fn a_repetition_one_move_away_counts_inside_the_search_only() {
    // Black's Ng8 would restore the starting position, three plies back.
    let board = start(&["g1f3", "g8f6", "f3g1"]);
    assert!(
        board.upcoming_repetition(4),
        "the earlier position is in the search"
    );
    assert!(!board.upcoming_repetition(3), "it is the root");
    assert!(!board.upcoming_repetition(1));
    // After a first cycle the starting position has occurred twice, so
    // reaching it again draws wherever the root is.
    let twice = start(&["g1f3", "g8f6", "f3g1", "f6g8", "g1f3", "g8f6", "f3g1"]);
    assert!(twice.upcoming_repetition(1));
    // A pawn move cannot be undone.
    let pawn = start(&["e2e3", "g8f6", "e3e4", "f6g8", "g1f3"]);
    assert!(!pawn.upcoming_repetition(10));
}

#[test]
fn the_non_pawn_key_is_its_pieces_keys_king_included_xored() {
    fn from_scratch(board: &Board, color: Color) -> u64 {
        let mut key = 0;
        for piece in [
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            for sq in board.pieces(color, piece) {
                key ^= ZOBRIST.piece(color, piece, sq);
            }
        }
        key
    }
    fn check(board: &Board) {
        for color in [Color::White, Color::Black] {
            assert_eq!(
                board.non_pawn_key(color),
                from_scratch(board, color),
                "{color:?} in {}",
                board.to_fen()
            );
        }
    }
    for fen in [
        STARTING_FEN,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r1bq1rk1/pP3ppp/2n2n2/2b5/8/2N2N2/PPP2PPP/R1BQKB1R w KQ - 0 9",
        "4k3/8/8/8/8/8/8/4K3 b - - 0 1",
    ] {
        let mut board = Board::from_fen(fen).expect("a legal FEN");
        check(&board);
        // Every move, its key after it and after unmaking it: captures,
        // castling, promotions and king moves change the key.
        for mv in board.generate_legal_moves() {
            let before = [Color::White, Color::Black].map(|c| board.non_pawn_key(c));
            board.make_move(mv);
            check(&board);
            board.unmake_move(mv);
            let after = [Color::White, Color::Black].map(|c| board.non_pawn_key(c));
            assert_eq!(before, after, "{mv:?} in {fen}");
        }
    }
}

#[test]
fn a_null_move_ends_the_window_and_unmaking_restores_it() {
    let mut board = start(&["g1f3", "g8f6", "f3g1"]);
    assert_eq!(board.plies_from_null, 3);
    board.make_null_move();
    assert_eq!(board.plies_from_null, 0);
    assert!(!board.upcoming_repetition(10));
    board.unmake_null_move();
    assert_eq!(board.plies_from_null, 3);
    assert!(board.upcoming_repetition(4));
    let mv = board.parse_move("f6g8").expect("legal");
    board.make_move(mv);
    assert_eq!(board.plies_from_null, 4);
    board.unmake_move(mv);
    assert_eq!(board.plies_from_null, 3);
}

#[test]
fn every_legal_move_that_repeats_is_found() {
    // Shuffle pieces so that cycles occur, and at every position compare
    // the cuckoo test with trying each legal move. The cuckoo test does
    // not check legality, so it may find more, never fewer.
    let mut board = Board::starting_position();
    let mut rng = 0x9E37_79B9_7F4A_7C15_u64;
    let mut found = 0;
    for _ in 0..4_000 {
        let window = board.repetition_window();
        let moves = board.generate_legal_moves();
        if moves.is_empty() || board.halfmove_clock >= 90 {
            board = Board::starting_position();
            continue;
        }
        let repeats = moves.iter().any(|&mv| {
            let mut child = board.clone();
            child.make_move(mv);
            (2..=window + 1)
                .step_by(2)
                .any(|plies| plies <= child.history.len() && child.key_back(plies) == child.hash)
        });
        if repeats {
            found += 1;
            assert!(
                board.upcoming_repetition(usize::MAX),
                "a repeating move is missed in {}",
                board.to_fen()
            );
        }
        // Prefer moves that return pieces, so that cycles are common.
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        let quiet: Vec<Move> = moves
            .iter()
            .copied()
            .filter(|mv| !mv.is_capture() && board.moving_piece(*mv) != Piece::Pawn)
            .collect();
        let pool = if quiet.is_empty() { &moves } else { &quiet };
        let pick = pool[infra::index(rng) % pool.len()];
        board.make_move(pick);
    }
    assert!(found > 100, "only {found} repeating positions were tried");
}

#[test]
fn the_kernel_draw_needs_a_repetition_inside_the_search_or_two() {
    let once = start(&["g1f3", "g8f6", "f3g1", "f6g8"]);
    assert!(
        once.is_draw_at(5),
        "four plies back, inside a search five deep"
    );
    assert!(!once.is_draw_at(4), "four plies back is the root");
    let twice = start(&[
        "g1f3", "g8f6", "f3g1", "f6g8", "g1f3", "g8f6", "f3g1", "f6g8",
    ]);
    assert!(twice.is_draw_at(1), "a third occurrence");
    assert!(!Board::starting_position().is_draw_at(1));
}

#[test]
fn dead_material_is_the_kernel_rule() {
    let dead = |fen: &str| {
        Board::from_fen(fen)
            .expect("a legal FEN")
            .is_dead_material()
    };
    assert!(dead("8/8/4k3/8/8/4K3/8/8 w - - 0 1"), "bare kings");
    assert!(dead("8/8/4k3/8/8/4K3/4N3/8 w - - 0 1"), "a knight");
    assert!(dead("8/8/4k3/8/8/4K3/4B3/8 b - - 0 1"), "a bishop");
    assert!(dead("8/4n3/4k3/8/8/4K3/4B3/8 w - - 0 1"), "a minor each");
    assert!(
        !dead("k7/4n3/8/8/8/4K3/4B3/8 w - - 0 1"),
        "a king in a corner"
    );
    assert!(
        !dead("8/8/4k3/8/8/4K3/3BN3/8 w - - 0 1"),
        "bishop and knight"
    );
    assert!(
        dead("8/8/4k3/8/8/4K3/2B1B3/8 w - - 0 1"),
        "bishops on one colour"
    );
    assert!(
        !dead("8/8/4k3/8/8/4K3/3BB3/8 w - - 0 1"),
        "bishops on both colours"
    );
    assert!(!dead("8/8/4k3/8/8/4K3/4P3/8 w - - 0 1"), "a pawn");
}
