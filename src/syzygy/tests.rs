use super::*;

use crate::board::Square;
use std::fs;
use std::sync::{LazyLock, Mutex};

static TEST_SYZYGY_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn encoded_root_result(from: Square, to: Square, promotes: u32) -> u32 {
    (TB_WIN << TB_RESULT_WDL_SHIFT)
        | (u32::from(to.0) << TB_RESULT_TO_SHIFT)
        | (u32::from(from.0) << TB_RESULT_FROM_SHIFT)
        | (promotes << TB_RESULT_PROMOTES_SHIFT)
}

fn encoded_tb_move(from: Square, to: Square, promotes: u16) -> u16 {
    (u16::from(from.0) << 6) | u16::from(to.0) | (promotes << 12)
}

/// The value of `#define <name> <value>` in Fathom's header, where the
/// value is a number or a parenthesised sum of numbers.
fn fathom_define(header: &str, name: &str) -> u64 {
    let values: Vec<&str> = header
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("#define")?.trim_start();
            let rest = rest.strip_prefix(name)?;
            rest.starts_with(char::is_whitespace).then_some(rest)
        })
        .collect();
    assert_eq!(values.len(), 1, "{name} is defined once in tbprobe.h");
    let value = values[0].split("/*").next().unwrap_or_default().trim();
    value
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split('+')
        .map(|term| {
            let term = term.trim();
            match term.strip_prefix("0x") {
                Some(hex) => u64::from_str_radix(hex, 16),
                None => term.parse(),
            }
            .unwrap_or_else(|_| panic!("{name}: {term:?} is not a number"))
        })
        .sum()
}

#[test]
fn mirrored_constants_match_fathoms_header() {
    let header = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendor/fathom/src/tbprobe.h"
    ))
    .expect("read Fathom's header");
    let mirrored: [(&str, u64); 18] = [
        ("TB_MAX_MOVES", TB_MAX_MOVES as u64),
        ("TB_MAX_PLY", TB_MAX_PLY as u64),
        ("TB_LOSS", TB_LOSS.into()),
        ("TB_BLESSED_LOSS", TB_BLESSED_LOSS.into()),
        ("TB_DRAW", TB_DRAW.into()),
        ("TB_CURSED_WIN", TB_CURSED_WIN.into()),
        ("TB_WIN", TB_WIN.into()),
        ("TB_RESULT_FAILED", TB_RESULT_FAILED.into()),
        ("TB_RESULT_WDL_MASK", TB_RESULT_WDL_MASK.into()),
        ("TB_RESULT_TO_MASK", TB_RESULT_TO_MASK.into()),
        ("TB_RESULT_FROM_MASK", TB_RESULT_FROM_MASK.into()),
        ("TB_RESULT_PROMOTES_MASK", TB_RESULT_PROMOTES_MASK.into()),
        ("TB_RESULT_DTZ_MASK", TB_RESULT_DTZ_MASK.into()),
        ("TB_RESULT_WDL_SHIFT", TB_RESULT_WDL_SHIFT.into()),
        ("TB_RESULT_TO_SHIFT", TB_RESULT_TO_SHIFT.into()),
        ("TB_RESULT_FROM_SHIFT", TB_RESULT_FROM_SHIFT.into()),
        ("TB_RESULT_PROMOTES_SHIFT", TB_RESULT_PROMOTES_SHIFT.into()),
        ("TB_RESULT_DTZ_SHIFT", TB_RESULT_DTZ_SHIFT.into()),
    ];
    for (name, value) in mirrored {
        assert_eq!(fathom_define(&header, name), value, "{name}");
    }
}

#[test]
fn wdl_from_raw_maps_fathom_values() {
    assert_eq!(wdl_from_raw(TB_LOSS), Some(Wdl::Loss));
    assert_eq!(wdl_from_raw(TB_BLESSED_LOSS), Some(Wdl::BlessedLoss));
    assert_eq!(wdl_from_raw(TB_DRAW), Some(Wdl::Draw));
    assert_eq!(wdl_from_raw(TB_CURSED_WIN), Some(Wdl::CursedWin));
    assert_eq!(wdl_from_raw(TB_WIN), Some(Wdl::Win));
    assert_eq!(wdl_from_raw(TB_RESULT_FAILED), None);
    assert_eq!(wdl_from_raw(99), None);
}

#[test]
fn root_move_from_result_decodes_square_and_promotion_fields() {
    assert_eq!(
        root_move_from_result(encoded_root_result(Square::A7, Square::A8, 1)),
        Some(RootMove {
            from: Square::A7.0,
            to: Square::A8.0,
            promotes: Some(Piece::Queen),
        })
    );
    assert_eq!(
        root_move_from_result(encoded_root_result(Square::B2, Square::B1, 4)),
        Some(RootMove {
            from: Square::B2.0,
            to: Square::B1.0,
            promotes: Some(Piece::Knight),
        })
    );
    assert_eq!(
        root_move_from_result(encoded_root_result(Square::E2, Square::E4, 0)),
        Some(RootMove {
            from: Square::E2.0,
            to: Square::E4.0,
            promotes: None,
        })
    );
}

#[test]
fn root_move_from_result_rejects_no_move_and_unknown_promotion() {
    assert_eq!(
        root_move_from_result(encoded_root_result(Square::E2, Square::E2, 0)),
        None
    );
    assert_eq!(
        root_move_from_result(encoded_root_result(Square::A7, Square::A8, 7)),
        None
    );
}

#[test]
fn root_move_from_tb_move_decodes_fathom_move_fields() {
    assert_eq!(
        root_move_from_tb_move(encoded_tb_move(Square::C6, Square::C7, 0)),
        Some(RootMove {
            from: Square::C6.0,
            to: Square::C7.0,
            promotes: None,
        })
    );
    assert_eq!(
        root_move_from_tb_move(encoded_tb_move(Square::A7, Square::A8, 1)),
        Some(RootMove {
            from: Square::A7.0,
            to: Square::A8.0,
            promotes: Some(Piece::Queen),
        })
    );
    assert_eq!(
        root_move_from_tb_move(encoded_tb_move(Square::A7, Square::A8, 7)),
        None
    );
}

#[test]
fn tablebase_file_counts_scan_platform_separated_paths() {
    let base =
        std::env::temp_dir().join(format!("rarog-syzygy-counts-{}-{}", std::process::id(), 1));
    let first = base.join("a");
    let second = base.join("b");
    fs::create_dir_all(&first).expect("create first temp tablebase directory");
    fs::create_dir_all(&second).expect("create second temp tablebase directory");
    fs::write(first.join("KQvK.rtbw"), []).expect("write WDL file");
    fs::write(first.join("KQvK.rtbz"), []).expect("write DTZ file");
    fs::write(second.join("KRvK.RTBW"), []).expect("write upper-case WDL file");
    fs::write(second.join("README.txt"), []).expect("write ignored file");

    let path = std::env::join_paths([&first, &second])
        .expect("join temp tablebase paths")
        .into_string()
        .expect("UTF-8 temp paths");
    assert_eq!(tablebase_file_counts(&path), (2, 1));

    let _ = fs::remove_dir_all(base);
}

#[test]
fn tb_position_exports_bitboards_side_ep_and_rule50_state() {
    let board = Board::from_fen("4k3/8/8/8/4Pp2/8/8/4K3 b - e3 7 42").expect("valid FEN");

    let pos = TbPosition::of(&board);

    assert_eq!(pos.white, board.color_occ(Color::White).0);
    assert_eq!(pos.black, board.color_occ(Color::Black).0);
    assert_eq!(
        pos.kings,
        (board.pieces(Color::White, Piece::King) | board.pieces(Color::Black, Piece::King)).0
    );
    assert_eq!(
        pos.pawns,
        (board.pieces(Color::White, Piece::Pawn) | board.pieces(Color::Black, Piece::Pawn)).0
    );
    assert_eq!(pos.rule50, 7);
    assert_eq!(pos.ep, u32::from(Square::E3.0));
    assert!(!pos.turn);
}

#[test]
fn legal_move_from_root_probe_requires_exact_promotion_match() {
    let board = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");

    let queen_promo = legal_move_from_root_probe(
        &board,
        RootMove {
            from: Square::A7.0,
            to: Square::A8.0,
            promotes: Some(Piece::Queen),
        },
    )
    .expect("queen promotion must be legal");

    assert_eq!(queen_promo.to_string(), "a7a8q");
    assert!(
        legal_move_from_root_probe(
            &board,
            RootMove {
                from: Square::A7.0,
                to: Square::A8.0,
                promotes: None,
            },
        )
        .is_none()
    );
}

#[test]
fn initialize_rejects_path_with_nul_without_calling_fathom() {
    let _guard = TEST_SYZYGY_LOCK.lock().expect("syzygy test lock poisoned");
    assert_eq!(initialize("bad\0path"), 0);
    assert_eq!(largest(), 0);
}
