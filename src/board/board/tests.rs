/// The noisy stage's list is the legal captures plus the legal
/// non-capturing queen promotions, each once, through checks and pins.
#[test]
fn the_noisy_list_is_the_captures_and_the_queen_pushes() {
    fn expected(board: &Board) -> Vec<String> {
        let mut moves: Vec<String> = board
            .generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.is_capture() || (mv.is_promo() && mv.promo_piece() == Piece::Queen))
            .map(|mv| mv.to_string())
            .collect();
        moves.sort_unstable();
        moves
    }
    fn noisy(board: &mut Board) -> Vec<String> {
        let mut list = MoveList::new();
        let threats = board.threats();
        board.generate_legal_noisy_staged_into(&threats, &mut list);
        let mut moves: Vec<String> = list.iter().map(|mv| mv.to_string()).collect();
        moves.sort_unstable();
        moves
    }
    fn walk(board: &mut Board, depth: u32, pushes: &mut usize) {
        let got = noisy(board);
        let mut unique = got.clone();
        unique.dedup();
        assert_eq!(got, unique, "a move twice in {}", board.to_fen());
        assert_eq!(got, expected(board), "noisy list in {}", board.to_fen());
        *pushes += got.iter().filter(|mv| mv.ends_with('q')).count();
        if depth == 0 {
            return;
        }
        for mv in board.generate_legal_moves() {
            board.make_move(mv);
            walk(board, depth - 1, pushes);
            board.unmake_move(mv);
        }
    }
    let mut pushes = 0;
    for fen in [
        "8/P6k/8/8/8/8/8/K7 w - - 0 1",
        "k7/8/8/8/8/8/p6K/8 b - - 0 1",
        "8/2p2P1P/2n2P2/1k6/p7/6N1/P5K1/8 b - - 0 44",
        "8/2p2P1P/2n2P2/1k6/p7/6N1/P5K1/8 w - - 0 44",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ] {
        let mut board = Board::from_fen(fen).expect("valid FEN");
        walk(&mut board, 2, &mut pushes);
    }
    assert!(pushes > 0, "no queen push reached");

    let list = |fen: &str| noisy(&mut Board::from_fen(fen).expect("valid FEN"));
    assert_eq!(list("8/P6k/8/8/8/8/8/K7 w - - 0 1"), ["a7a8q"]);
    assert!(
        list("n7/P6k/8/8/8/8/8/K7 w - - 0 1").is_empty(),
        "a blocked pawn"
    );
    // In check the push must block: on the checking rank, not off its file.
    assert!(list("K6r/2P5/8/8/8/8/8/7k w - - 0 1").contains(&"c7c8q".to_owned()));
    assert!(
        !list("K7/2P5/8/8/8/8/8/r6k w - - 0 1")
            .iter()
            .any(|mv| mv.ends_with('q'))
    );
    // Pinned on the diagonal, the pawn may take the pinner but not push.
    let pinned = list("2b4k/1P6/K7/8/8/8/8/8 w - - 0 1");
    assert!(pinned.contains(&"b7c8q".to_owned()) && !pinned.contains(&"b7b8q".to_owned()));
    // The captures list itself is unchanged.
    let mut board = Board::from_fen("8/P6k/8/8/8/8/8/K7 w - - 0 1").expect("valid FEN");
    assert!(board.generate_legal_captures().is_empty());
}

/// The partition, judged from what each legal move does on the board and
/// not from its flags: a piece taken, a pawn changing file, or a pawn
/// reaching the last rank is noisy, and everything else is quiet.
#[test]
fn every_legal_move_is_quiet_or_noisy_by_what_it_does() {
    let extra = [
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
        "1n6/P6k/8/8/8/8/8/K7 w - - 0 1",
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
    ];
    let (mut castles, mut en_passants, mut promotions, mut under_promotions) = (0, 0, 0, 0);
    for fen in crate::bench::BENCH_FENS.iter().chain(extra.iter()) {
        let board = Board::from_fen(fen).expect("valid FEN");
        for mv in board.generate_legal_moves() {
            let (from, to) = (mv.from_sq().index(), mv.to_sq().index());
            let pawn = board.moving_piece(mv) == Piece::Pawn;
            let noisy = board.piece_at(mv.to_sq()).is_some()
                || (pawn && from % 8 != to % 8)
                || (pawn && (to / 8 == 0 || to / 8 == 7));
            assert_eq!(mv.is_noisy(), noisy, "{mv} noisy in {fen}");
            assert_eq!(mv.is_quiet(), !noisy, "{mv} quiet in {fen}");
            castles += usize::from(mv.is_castling());
            en_passants += usize::from(mv.is_en_passant());
            if mv.is_promo() {
                promotions += 1;
                under_promotions += usize::from(mv.promo_piece() != Piece::Queen);
            }
        }
    }
    assert!(castles > 0, "no castling reached");
    assert!(en_passants > 0, "no en passant reached");
    assert!(promotions > 0, "no promotion reached");
    assert!(under_promotions > 0, "no under-promotion reached");
}

use super::*;

/// `threats()` agrees with the attacker query on every square of positions
/// reached by a perft walk, piece type by piece type, with the side to
/// move's king lifted from the occupancy.
#[test]
fn threats_match_the_attacker_query_on_every_square() {
    fn check(board: &Board) {
        let us = board.side_to_move();
        let them = !us;
        let occ = board.occupied() ^ board.pieces(us, Piece::King);
        let threats = board.threats();
        let mut union = Bitboard::EMPTY;
        for piece in Piece::ALL {
            let mut expected = Bitboard::EMPTY;
            for sq in (0..64).map(Square) {
                let attackers = board.attackers_to_color(sq, occ, them) & board.pieces(them, piece);
                if attackers.any() {
                    expected |= Bitboard::from(sq);
                }
            }
            assert_eq!(
                threats.by_piece[piece as usize],
                expected,
                "{piece:?} threats in {}",
                board.to_fen()
            );
            union |= expected;
        }
        assert_eq!(threats.all, union, "all threats in {}", board.to_fen());
        for sq in (0..64).map(Square) {
            assert_eq!(threats.all.contains(sq), union.contains(sq));
        }
    }
    fn walk(board: &mut Board, depth: u32) {
        check(board);
        if depth == 0 {
            return;
        }
        for mv in board.generate_legal_moves() {
            board.make_move(mv);
            walk(board, depth - 1);
            board.unmake_move(mv);
        }
    }
    for fen in [
        STARTING_FEN,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1",
    ] {
        let mut board = Board::from_fen(fen).expect("valid FEN");
        walk(&mut board, 2);
    }
}

/// The king is lifted from the occupancy: a rook checking along a file
/// also attacks the square behind the king, which it could not reach if
/// the king blocked the line.
#[test]
fn threats_see_through_the_side_to_move_king() {
    let board = Board::from_fen("4k3/8/8/8/4r3/8/4K3/8 w - - 0 1").expect("valid FEN");
    let rook = board.threats().by_piece[Piece::Rook as usize];
    assert!(rook.contains(Square::E2), "the checked king's square");
    assert!(rook.contains(Square::E1), "the square behind the king");
    let attackers = board.attackers_to_color(Square::E1, board.occupied(), Color::Black);
    assert!(
        attackers.is_empty(),
        "with the king in place e1 is shielded"
    );
}

/// Shuffle both knights out and back: four plies that always exist from the
/// starting position, so a walk of any even length can be built from them.
const SHUFFLE: [&str; 4] = ["g1f3", "g8f6", "f3g1", "f6g8"];

fn walk(board: &mut Board, plies: usize) -> Vec<Move> {
    let mut played = Vec::with_capacity(plies);
    for index in 0..plies {
        let uci = SHUFFLE[index % SHUFFLE.len()];
        let mv = board
            .parse_move(uci)
            .unwrap_or_else(|| panic!("{uci} must be legal at ply {index}"));
        board.make_move(mv);
        played.push(mv);
    }
    played
}

#[test]
fn reserve_history_provides_headroom_above_the_game_already_played() {
    let mut board = Board::starting_position();
    walk(&mut board, 40);
    let played = board.history.len();
    board.reserve_history(128);
    assert!(
        board.history.capacity() >= played + 128,
        "capacity {} < {} played + 128 reserved",
        board.history.capacity(),
        played
    );
}

#[test]
fn a_reserved_history_does_not_reallocate_through_a_full_depth_walk() {
    let mut board = Board::starting_position();
    walk(&mut board, 40);
    let plies = crate::score::MAX_PLY;
    board.reserve_history(plies);
    let reserved = board.history.capacity();

    // The search reserves its ply cap; the reservation must absorb a line
    // that long without a single reallocation.
    let played = walk(&mut board, plies);
    assert_eq!(
        board.history.capacity(),
        reserved,
        "history reallocated mid-walk: {reserved} -> {}",
        board.history.capacity()
    );

    for mv in played.into_iter().rev() {
        board.unmake_move(mv);
    }
    assert_eq!(board.history.len(), 40, "history depth not restored");
    assert_eq!(
        board.history.capacity(),
        reserved,
        "capacity changed on unwind"
    );
}

#[test]
fn clone_preserves_reserved_capacity_so_workers_inherit_it() {
    let mut board = Board::starting_position();
    // A multiple of SHUFFLE.len(), so the cycle restarts cleanly for the
    // clone's own walk below.
    walk(&mut board, 12);
    board.reserve_history(128);
    let reserved = board.history.capacity();

    let worker = board.clone();
    assert_eq!(
        worker.history.capacity(),
        reserved,
        "worker clone dropped the reservation"
    );
    assert_eq!(worker.history.len(), board.history.len());

    // And the clone must then be able to search without reallocating.
    let mut worker = worker;
    walk(&mut worker, 128);
    assert_eq!(
        worker.history.capacity(),
        reserved,
        "worker reallocated despite inheriting the reservation"
    );
}

#[test]
fn history_restoration_is_exact_through_a_deep_walk() {
    let mut board = Board::starting_position();
    board.reserve_history(128);
    let before = board.clone();

    let played = walk(&mut board, 64);
    for mv in played.into_iter().rev() {
        board.unmake_move(mv);
    }

    board
        .check_consistency()
        .unwrap_or_else(|err| panic!("inconsistent after unwind: {err}"));
    assert_eq!(board.hash, before.hash, "position hash not restored");
    assert_eq!(board.history.len(), before.history.len());
    assert_eq!(board.halfmove_clock, before.halfmove_clock);
    assert_eq!(board.fullmove, before.fullmove);
    assert_eq!(board.side_to_move, before.side_to_move);
}

#[test]
fn legal_move_canonicalizes_flags_that_from_uci_cannot_know() {
    // `Move::from_uci` always yields QUIET, because a UCI string carries no
    // flag information. `legal_move` must supply the real flags, and a
    // caller that plays its own input instead of the returned move would
    // push the wrong UnmakeInfo. This pins that difference.
    let cases = [
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "e2e4",
        ),
        (
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2",
            "e4d5",
        ),
        ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "e1g1"),
    ];
    for (fen, uci) in cases {
        let board = Board::from_fen(fen).unwrap_or_else(|err| panic!("{fen}: {err}"));
        let raw = Move::from_uci(uci).expect("valid UCI shape");
        assert_eq!(raw.flags(), QUIET, "{uci}: from_uci should be QUIET");
        let canonical = board
            .legal_move(raw)
            .unwrap_or_else(|| panic!("{uci} must be legal in {fen}"));
        assert_ne!(
            canonical.flags(),
            raw.flags(),
            "{uci}: legal_move did not canonicalize the flags"
        );
        assert_eq!(canonical.from_sq(), raw.from_sq());
        assert_eq!(canonical.to_sq(), raw.to_sq());
        assert!(
            board.is_legal(raw),
            "{uci}: is_legal must agree with legal_move"
        );
    }
}

/// The prefetch hint is the real key after the make for every plain move
/// or capture that changes no castling right and sets no en-passant
/// square, over every position of a depth-3 walk from several roots. The
/// walk must also reach the excluded moves, or the exclusions are untested.
#[test]
fn key_after_hint_is_exact_for_plain_moves_and_captures() {
    #[derive(Default)]
    struct Seen {
        exact: u64,
        castling_changed: u64,
        ep_set: u64,
    }
    fn walk(board: &mut Board, depth: u32, seen: &mut Seen) {
        if depth == 0 {
            return;
        }
        for mv in board.generate_legal_moves() {
            let hint = board.key_after_hint(mv);
            let castling = board.castling();
            let plain = matches!(mv.flags(), QUIET | DOUBLE_PUSH | CAPTURE);
            board.make_move(mv);
            if plain {
                if board.castling() != castling {
                    seen.castling_changed += 1;
                } else if board.ep_square().is_some() {
                    seen.ep_set += 1;
                } else {
                    assert_eq!(hint, board.hash(), "{mv} reaching {}", board.to_fen());
                    seen.exact += 1;
                }
            }
            walk(board, depth - 1, seen);
            board.unmake_move(mv);
        }
    }
    let mut seen = Seen::default();
    for fen in [
        STARTING_FEN,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        "8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    ] {
        let mut board = Board::from_fen(fen).expect("valid FEN");
        walk(&mut board, 3, &mut seen);
    }
    assert!(seen.exact > 50_000, "only {} exact cases", seen.exact);
    assert!(seen.castling_changed > 0 && seen.ep_set > 0);
}

/// A board built from its placement is the board its FEN builds without
/// the en-passant square: every derived key, the checkers and the
/// consistency check agree, over a tree of positions with castling,
/// promotions, checks and rule-50 clocks. Inconsistent placements are
/// refused.
#[test]
fn a_placement_builds_the_board_its_fen_builds() {
    fn placement(board: &Board) -> [Bitboard; 12] {
        let mut pieces = [Bitboard::EMPTY; 12];
        for color in [Color::White, Color::Black] {
            for piece in Piece::ALL {
                pieces[color as usize * 6 + piece as usize] = board.pieces(color, piece);
            }
        }
        pieces
    }
    fn check(board: &Board, checked: &mut usize) {
        let fen = board.to_fen();
        let mut fields: Vec<&str> = fen.split_whitespace().collect();
        fields[3] = "-";
        fields[5] = "1";
        let expected = Board::from_fen(&fields.join(" ")).expect("a FEN without its ep square");
        let built = Board::from_placement(
            placement(board),
            board.side_to_move(),
            board.castling(),
            board.halfmove_clock(),
        )
        .expect("a legal placement");
        assert_eq!(built.to_fen(), expected.to_fen());
        assert_eq!(built.hash(), expected.hash(), "hash in {fen}");
        assert_eq!(built.pawn_key(), expected.pawn_key(), "pawn key in {fen}");
        assert_eq!(
            built.minor_key(),
            expected.minor_key(),
            "minor key in {fen}"
        );
        for color in [Color::White, Color::Black] {
            assert_eq!(built.non_pawn_key(color), expected.non_pawn_key(color));
        }
        assert_eq!(built.checkers(), expected.checkers(), "checkers in {fen}");
        built.check_consistency().expect("a consistent board");
        *checked += 1;
    }
    fn walk(board: &mut Board, depth: u32, checked: &mut usize) {
        check(board, checked);
        if depth == 0 {
            return;
        }
        for mv in board.generate_legal_moves() {
            board.make_move(mv);
            walk(board, depth - 1, checked);
            board.unmake_move(mv);
        }
    }
    let mut checked = 0;
    for fen in [
        STARTING_FEN,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "8/5pk1/6p1/3P3p/4P3/5K2/8/8 b - - 73 54",
    ] {
        let mut board = Board::from_fen(fen).expect("valid test FEN");
        walk(&mut board, 2, &mut checked);
    }
    assert!(checked > 2_000, "only {checked} positions");

    let start = Board::starting_position();
    let mut overlap = placement(&start);
    overlap[1] |= Bitboard(1 << 12);
    assert!(Board::from_placement(overlap, Color::White, CastlingRights::NONE, 0).is_err());
    let mut back_rank_pawn = [Bitboard::EMPTY; 12];
    back_rank_pawn[5] = Bitboard(1 << 4);
    back_rank_pawn[11] = Bitboard(1 << 60);
    back_rank_pawn[0] = Bitboard(1 << 56);
    assert!(Board::from_placement(back_rank_pawn, Color::White, CastlingRights::NONE, 0).is_err());
    let mut no_king = placement(&start);
    no_king[5] = Bitboard::EMPTY;
    assert!(Board::from_placement(no_king, Color::White, CastlingRights::NONE, 0).is_err());
    let mut rookless = placement(&start);
    rookless[3] = Bitboard(rookless[3].0 & !(1 << 7));
    assert!(
        Board::from_placement(rookless, Color::White, CastlingRights(0xF), 0).is_err(),
        "castling rights without the rook"
    );
}

/// A queen pushed onto a defended square is taken: the compact kernel's
/// test plays that exchange, the gain form credits the promotion's gain,
/// and a capture or an undefended push reads the same in both.
#[test]
fn the_exchange_test_puts_a_promotion_push_at_risk() {
    let values = PRODUCTION_SEE_VALUES;
    let pawn = values.value(Piece::Pawn);
    let gain = values.value(Piece::Queen) - pawn;
    let defended = Board::from_fen("1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let push = defended.parse_move("a7a8q").unwrap();
    assert!(push.is_promo() && !push.is_capture());
    assert!(defended.see_ge_exchange_with_values(push, -pawn, values));
    assert!(!defended.see_ge_exchange_with_values(push, -pawn + 1, values));
    assert!(defended.see_ge_with_values(push, gain, values));

    let free = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let push = free.parse_move("a7a8q").unwrap();
    assert!(free.see_ge_exchange_with_values(push, gain, values));
    assert!(!free.see_ge_exchange_with_values(push, gain + 1, values));

    let taking = defended.parse_move("a7b8q").unwrap();
    let rook_and_gain = values.value(Piece::Rook) + gain;
    for see_ge in [
        Board::see_ge_exchange_with_values,
        Board::see_ge_with_values,
    ] {
        assert!(see_ge(&defended, taking, rook_and_gain, values));
        assert!(!see_ge(&defended, taking, rook_and_gain + 1, values));
    }
}
