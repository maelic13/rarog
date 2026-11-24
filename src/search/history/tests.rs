use super::*;

/// Each table's entries stay inside `±max` under any sequence of
/// clamped bonuses, and the update is gravity: repeated identical bonuses
/// converge on the bound without crossing it, and a saturated entry moves
/// less than an empty one.
#[test]
fn every_table_is_bounded_and_updates_by_gravity() {
    // The pawn table's largest limit a tuning option reaches is twice
    // its seed, 16,802, inside an `i16`.
    for max in [QUIET_MAX, NOISY_MAX, PAWN_MAX, 2 * PAWN_MAX, CONT_MAX] {
        let mut entry = 0i16;
        let mut previous = 0;
        for _ in 0..200 {
            apply_bonus(&mut entry, 3 * max, max);
            let value = i32::from(entry);
            assert!(value <= max && value >= previous, "max {max}: {value}");
            previous = value;
        }
        assert!(i32::from(entry) >= max - max / 64, "max {max} converges");
        for _ in 0..200 {
            apply_bonus(&mut entry, -3 * max, max);
            assert!(i32::from(entry) >= -max, "max {max}: {entry}");
        }
        let mut empty = 0i16;
        let mut high = i16::try_from(max * 3 / 4).expect("fits");
        apply_bonus(&mut empty, 1_000, max);
        let before = high;
        apply_bonus(&mut high, 1_000, max);
        assert!(high - before < empty, "saturated entry moves less");
    }
}

#[test]
fn tables_index_by_threat_and_colour() {
    let mut tables = HistoryTables::default();
    let mv = Move::from_uci("g1f3").expect("valid move");
    let threatened_to = Bitboard::from(Square::F3);
    tables.update_quiet(threatened_to, Color::White, mv, 500);
    assert_eq!(tables.quiet(threatened_to, Color::White, mv), 500);
    assert_eq!(tables.quiet(Bitboard::EMPTY, Color::White, mv), 0);
    assert_eq!(tables.quiet(threatened_to, Color::Black, mv), 0);

    tables.update_noisy(
        Bitboard::EMPTY,
        Color::Black,
        Piece::Queen,
        Square::D4,
        None,
        700,
    );
    assert_eq!(
        tables.noisy(
            Bitboard::EMPTY,
            Color::Black,
            Piece::Queen,
            Square::D4,
            None
        ),
        700
    );
    assert_eq!(
        tables.noisy(
            Bitboard::EMPTY,
            Color::Black,
            Piece::Queen,
            Square::D4,
            Some(Piece::Pawn)
        ),
        0
    );
    assert_eq!(
        tables.noisy(
            Bitboard::from(Square::D4),
            Color::Black,
            Piece::Queen,
            Square::D4,
            None
        ),
        0
    );

    let row = HistoryTables::pawn_row(0xDEAD_BEEF);
    tables.update_pawn(row, Color::White, Piece::Knight, Square::F3, -300);
    assert_eq!(
        tables.pawn(row, Color::White, Piece::Knight, Square::F3),
        -300
    );
    assert_eq!(tables.pawn(row, Color::Black, Piece::Knight, Square::F3), 0);

    let context = cont_context(true, false, Color::Black, Piece::Pawn, Square::E5);
    tables.update_cont(context, Color::White, Piece::Knight, Square::F3, 900);
    assert_eq!(
        tables.cont(context, Color::White, Piece::Knight, Square::F3),
        900
    );
    let other = cont_context(false, false, Color::Black, Piece::Pawn, Square::E5);
    assert_eq!(
        tables.cont(other, Color::White, Piece::Knight, Square::F3),
        0
    );

    tables.clear();
    assert_eq!(tables.quiet(threatened_to, Color::White, mv), 0);
    assert_eq!(
        tables.cont(context, Color::White, Piece::Knight, Square::F3),
        0
    );
}

/// The largest context plus the largest piece-to slot is the last entry.
#[test]
fn continuation_indexes_cover_the_table_exactly() {
    let last = cont_context(true, true, Color::Black, Piece::King, Square::H8)
        + piece_to(Color::Black, Piece::King, Square::H8);
    assert_eq!(last, CONT_SIZE - 1);
    assert_eq!(
        HistoryTables::pawn_row(u64::MAX) + piece_to(Color::Black, Piece::King, Square::H8),
        PAWN_HISTORY_SIZE * PIECE_TO_SIZE - 1
    );
}
