//! Perft through the search's own move path: the staged picker in the
//! search's stage order, the search's make with its check hint, and unmake.
//!
//! The board's perft walks the plain legal generator, so a picker that drops,
//! repeats or invents a move, a TT move whose flags differ from the generated
//! move's, or a check hint that leaves the checker set wrong would pass it and
//! show up only as an unexplained loss in games. Here every node rebuilds the
//! board's derived fields and keys from scratch after each make and unmake,
//! every emitted move must pass the TT move's legality test unchanged, and the
//! leaf count must equal the reference count.

use crate::board::{Board, CheckInfo, Move, Square};

use super::Searcher;
use super::movepick::{MovePicker, Stage};

/// The picker stage a move came from.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Source {
    TtMove,
    GoodNoisy,
    Quiets,
    BadNoisy,
}

const SOURCES: [Source; 4] = [
    Source::TtMove,
    Source::GoodNoisy,
    Source::Quiets,
    Source::BadNoisy,
];

/// What one walk saw, and the fault planted in it, if any.
#[derive(Default)]
struct Walk {
    /// Moves emitted per [`Source`], so a walk proves every stage was reached.
    emitted: [u64; 4],
    /// Drop the first move this source emits, as a broken picker would.
    drop: Option<Source>,
    dropped: bool,
}

/// The state make/unmake must restore exactly, beyond what
/// [`Board::check_consistency`] rebuilds from the mailbox.
fn restorable(board: &Board) -> (u64, u8, Option<Square>, u8, u16) {
    (
        board.hash(),
        board.castling().0,
        board.ep_square(),
        board.halfmove_clock(),
        board.fullmove(),
    )
}

/// A TT move as a probe could hand the node, already through the legality
/// test the search applies: none, a legal move as stored, the same move
/// re-read from its UCI text (castling, en passant, double-push and capture
/// flags lost, so the test must restore them), or arbitrary bits, which a key
/// collision returns. Chosen from the key, so a walk reproduces exactly.
fn tt_move(board: &Board) -> Move {
    let mix = board.hash().wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32;
    let legal = board.generate_legal_movelist();
    let legal = legal.as_slice();
    let pick = |legal: &[Move]| legal[usize::try_from(mix >> 2).expect("32 bits") % legal.len()];
    let candidate = match mix % 4 {
        _ if legal.is_empty() => Move::NULL,
        0 => Move::NULL,
        1 => pick(legal),
        2 => Move::from_uci(&pick(legal).to_string()).expect("a generated move prints as UCI"),
        _ => Move(u16::try_from((mix >> 8) & 0xFFFF).expect("16 bits")),
    };
    board.legal_move(candidate).unwrap_or(Move::NULL)
}

fn node(
    searcher: &Searcher,
    board: &mut Board,
    depth: u32,
    ply: usize,
    walk: &mut Walk,
) -> Result<u64, String> {
    if depth == 0 {
        return Ok(1);
    }
    let tt_move = tt_move(board);
    let threats = board.threats();
    let mut picker = MovePicker::staged(tt_move, ply);
    picker.fill_noisy(searcher, board, &threats);
    let mut node_ci: Option<CheckInfo> = None;
    let mut tt_pending = !tt_move.is_null();
    let mut seen = Vec::new();
    let mut leaves = 0;
    while let Some(picked) = picker.next(searcher, board, &threats, false, &mut node_ci) {
        let mv = picked.mv;
        let source = if tt_pending {
            tt_pending = false;
            Source::TtMove
        } else {
            match picker.stage() {
                Stage::GoodNoisy => Source::GoodNoisy,
                Stage::Quiets => Source::Quiets,
                Stage::BadNoisy => Source::BadNoisy,
                other => return Err(format!("{}: {mv} emitted from {other:?}", board.to_fen())),
            }
        };
        if walk.drop == Some(source) && !walk.dropped {
            walk.dropped = true;
            continue;
        }
        walk.emitted[source as usize] += 1;
        let fen = || board.to_fen();
        if board.legal_move(mv) != Some(mv) {
            return Err(format!(
                "{}: {mv} from {source:?} fails the legality test",
                fen()
            ));
        }
        if seen.contains(&mv) {
            return Err(format!(
                "{}: {mv} emitted twice, again from {source:?}",
                fen()
            ));
        }
        seen.push(mv);
        let before = restorable(board);
        let ci = node_ci.get_or_insert_with(|| board.check_info());
        let gives_check = board.gives_check_with(mv, ci);
        board.make_move_with_check(mv, gives_check);
        if let Err(err) = board.check_consistency() {
            board.unmake_move(mv);
            return Err(format!("{}: after making {mv}: {err}", board.to_fen()));
        }
        let below = node(searcher, board, depth - 1, ply + 1, walk);
        board.unmake_move(mv);
        let below = below?;
        if let Err(err) = board.check_consistency() {
            return Err(format!("{}: after unmaking {mv}: {err}", board.to_fen()));
        }
        if restorable(board) != before {
            return Err(format!(
                "{}: unmaking {mv} did not restore the position",
                board.to_fen()
            ));
        }
        leaves += below;
    }
    Ok(leaves)
}

fn search_path_perft(fen: &str, depth: u32, walk: &mut Walk) -> Result<u64, String> {
    let searcher = Searcher::default();
    let mut board = Board::from_fen(fen).map_err(|err| format!("{fen}: {err}"))?;
    board
        .check_consistency()
        .map_err(|err| format!("{fen}: at the root: {err}"))?;
    node(&searcher, &mut board, depth, 0, walk)
}

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
const POSITION_6: &str = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

/// The standard perft positions with their published counts, shallower in a
/// debug build.
fn reference_cases() -> &'static [(&'static str, u32, u64)] {
    if cfg!(debug_assertions) {
        &[
            (STARTPOS, 3, 8_902),
            (KIWIPETE, 2, 2_039),
            (POSITION_3, 3, 2_812),
            (POSITION_4, 3, 9_467),
            (POSITION_5, 2, 1_486),
            (POSITION_6, 2, 2_079),
        ]
    } else {
        &[
            (STARTPOS, 5, 4_865_609),
            (KIWIPETE, 4, 4_085_603),
            (POSITION_3, 5, 674_624),
            (POSITION_4, 4, 422_333),
            (POSITION_5, 4, 2_103_487),
            (POSITION_6, 4, 3_894_594),
        ]
    }
}

#[test]
fn search_path_perft_matches_the_reference_counts() {
    let mut emitted = [0u64; 4];
    for &(fen, depth, expected) in reference_cases() {
        let mut walk = Walk::default();
        let leaves = search_path_perft(fen, depth, &mut walk).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(leaves, expected, "search-path perft({depth}) of {fen}");
        for (total, count) in emitted.iter_mut().zip(walk.emitted) {
            *total += count;
        }
    }
    for source in SOURCES {
        assert!(
            emitted[source as usize] > 0,
            "no move came from {source:?}, so the walk did not test that stage"
        );
    }
}

/// The planted fault: a picker that loses one move of any one stage fails
/// the perft.
#[test]
fn a_move_dropped_from_any_stage_fails_the_search_path_perft() {
    let (fen, depth, expected) = (KIWIPETE, 2, 2_039);
    for source in SOURCES {
        let mut walk = Walk {
            drop: Some(source),
            ..Walk::default()
        };
        let result = search_path_perft(fen, depth, &mut walk);
        assert!(walk.dropped, "{source:?} emitted nothing to drop");
        assert_ne!(
            result,
            Ok(expected),
            "a move dropped from {source:?} went unseen"
        );
    }
}
