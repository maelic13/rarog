use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_uint};
use std::sync::{
    LazyLock, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use crate::board::{Board, Color, Move, Piece};

const TB_LOSS: u32 = 0;
const TB_BLESSED_LOSS: u32 = 1;
const TB_DRAW: u32 = 2;
const TB_CURSED_WIN: u32 = 3;
const TB_WIN: u32 = 4;
const TB_RESULT_FAILED: u32 = 0xFFFF_FFFF;
const TB_RESULT_WDL_MASK: u32 = 0x0000_000F;
const TB_RESULT_TO_MASK: u32 = 0x0000_03F0;
const TB_RESULT_FROM_MASK: u32 = 0x0000_FC00;
const TB_RESULT_PROMOTES_MASK: u32 = 0x0007_0000;
const TB_RESULT_DTZ_MASK: u32 = 0xFFF0_0000;
const TB_RESULT_DTZ_SHIFT: u32 = 20;
const TB_RESULT_WDL_SHIFT: u32 = 0;
const TB_RESULT_TO_SHIFT: u32 = 4;
const TB_RESULT_FROM_SHIFT: u32 = 10;
const TB_RESULT_PROMOTES_SHIFT: u32 = 16;
const TB_MAX_MOVES: usize = 193;
const TB_MAX_PLY: usize = 256;

static SYZYGY_PATH: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
static LARGEST: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" {
    static mut TB_LARGEST: c_uint;

    fn tb_init(path: *const c_char) -> bool;
    fn tb_probe_wdl_impl(
        white: u64,
        black: u64,
        kings: u64,
        queens: u64,
        rooks: u64,
        bishops: u64,
        knights: u64,
        pawns: u64,
        ep: c_uint,
        turn: bool,
    ) -> c_uint;
    fn tb_probe_root_impl(
        white: u64,
        black: u64,
        kings: u64,
        queens: u64,
        rooks: u64,
        bishops: u64,
        knights: u64,
        pawns: u64,
        rule50: c_uint,
        ep: c_uint,
        turn: bool,
        results: *mut c_uint,
    ) -> c_uint;
    fn tb_probe_root_wdl(
        white: u64,
        black: u64,
        kings: u64,
        queens: u64,
        rooks: u64,
        bishops: u64,
        knights: u64,
        pawns: u64,
        rule50: c_uint,
        castling: c_uint,
        ep: c_uint,
        turn: bool,
        use_rule50: bool,
        results: *mut TbRootMovesRaw,
    ) -> c_int;
}

#[repr(C)]
#[derive(Copy, Clone)]
struct TbRootMoveRaw {
    mv: u16,
    pv: [u16; TB_MAX_PLY],
    pv_size: c_uint,
    tb_score: i32,
    tb_rank: i32,
}

impl Default for TbRootMoveRaw {
    fn default() -> Self {
        Self {
            mv: 0,
            pv: [0; TB_MAX_PLY],
            pv_size: 0,
            tb_score: 0,
            tb_rank: 0,
        }
    }
}

#[repr(C)]
struct TbRootMovesRaw {
    size: c_uint,
    moves: [TbRootMoveRaw; TB_MAX_MOVES],
}

impl Default for TbRootMovesRaw {
    fn default() -> Self {
        Self {
            size: 0,
            moves: [TbRootMoveRaw::default(); TB_MAX_MOVES],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Wdl {
    Loss,
    BlessedLoss,
    Draw,
    CursedWin,
    Win,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RootMove {
    pub from: u8,
    pub to: u8,
    promotes: Option<Piece>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RootProbe {
    pub(crate) wdl: Wdl,
    pub best_move: Option<RootMove>,
}

#[derive(Copy, Clone)]
struct TbPosition {
    white: u64,
    black: u64,
    kings: u64,
    queens: u64,
    rooks: u64,
    bishops: u64,
    knights: u64,
    pawns: u64,
    rule50: u32,
    ep: u32,
    turn: bool,
}

pub fn initialize(path: &str) -> usize {
    let mut current_path = SYZYGY_PATH.lock().expect("syzygy path mutex poisoned");
    if *current_path == path {
        return largest();
    }

    if path.is_empty() {
        let empty = CString::new("").expect("empty string has no NUL");
        // SAFETY: FFI into Fathom. `empty` outlives the call and `as_ptr`
        // yields a valid NUL-terminated C string; `tb_init` copies what it
        // needs and does not retain the pointer.
        unsafe {
            tb_init(empty.as_ptr());
        }
        *current_path = String::new();
        LARGEST.store(0, Ordering::Relaxed);
        return 0;
    }

    let Ok(c_path) = CString::new(path) else {
        *current_path = String::new();
        LARGEST.store(0, Ordering::Relaxed);
        return 0;
    };

    // SAFETY: FFI into Fathom. `c_path` is a live NUL-terminated CString that
    // outlives the call; `tb_init` does not retain the pointer.
    let ok = unsafe { tb_init(c_path.as_ptr()) };
    let largest = if ok {
        // SAFETY: reading an `extern "C"` global that `tb_init` just
        // initialised; single-threaded at this point (guarded by the path
        // mutex held by the caller).
        unsafe { TB_LARGEST as usize }
    } else {
        0
    };
    *current_path = if ok { path.to_string() } else { String::new() };
    LARGEST.store(largest, Ordering::Relaxed);
    largest
}

pub(crate) fn current_path() -> String {
    SYZYGY_PATH
        .lock()
        .expect("syzygy path mutex poisoned")
        .clone()
}

#[inline(always)]
pub fn largest() -> usize {
    LARGEST.load(Ordering::Relaxed)
}

pub(crate) fn tablebase_file_counts(path: &str) -> (usize, usize) {
    let mut wdl = 0usize;
    let mut dtz = 0usize;
    for part in path
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let Ok(entries) = fs::read_dir(part) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            match extension.to_ascii_lowercase().as_str() {
                "rtbw" => wdl += 1,
                "rtbz" => dtz += 1,
                _ => {}
            }
        }
    }
    (wdl, dtz)
}

pub fn probe_wdl(board: &Board, use_rule50: bool) -> Option<Wdl> {
    if !can_probe(board, use_rule50, false) {
        return None;
    }

    let pos = tb_position(board);
    // SAFETY: FFI into Fathom with by-value bitboards from `tb_position`; no
    // pointers are passed and the tables were initialised by `tb_init`.
    let result = unsafe {
        tb_probe_wdl_impl(
            pos.white,
            pos.black,
            pos.kings,
            pos.queens,
            pos.rooks,
            pos.bishops,
            pos.knights,
            pos.pawns,
            pos.ep,
            pos.turn,
        )
    };
    wdl_from_raw(result)
}

pub fn probe_root(board: &Board, use_rule50: bool) -> Option<RootProbe> {
    if !can_probe(board, use_rule50, true) {
        return None;
    }

    let pos = tb_position(board);
    // SAFETY: FFI into Fathom, by-value position only (see `probe_wdl`).
    let result = unsafe {
        tb_probe_root_impl(
            pos.white,
            pos.black,
            pos.kings,
            pos.queens,
            pos.rooks,
            pos.bishops,
            pos.knights,
            pos.pawns,
            pos.rule50,
            pos.ep,
            pos.turn,
            std::ptr::null_mut(),
        )
    };
    if result == TB_RESULT_FAILED {
        return None;
    }

    let wdl = wdl_from_raw((result & TB_RESULT_WDL_MASK) >> TB_RESULT_WDL_SHIFT)?;
    Some(RootProbe {
        wdl,
        best_move: root_move_from_result(result),
    })
}

/// Rank scale of the root ranking: a clean win ranks `MAX_DTZ`, less its
/// distance to zeroing when moves are ordered by it; a win the rule-50
/// counter may spoil ranks below `MAX_DTZ / 2`; losses mirror wins.
pub(crate) const MAX_DTZ: i32 = 1 << 18;

/// One root move and its rank, higher better.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RankedMove {
    pub(crate) mv: Move,
    pub(crate) rank: i32,
}

/// The root's legal moves ranked by the tables, best first (stable among
/// equals). `dtz` says whether DTZ ranked them; otherwise WDL did, and the
/// search may still need its in-search probes to make progress.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RootRanking {
    pub(crate) dtz: bool,
    pub(crate) moves: Vec<RankedMove>,
}

/// Rank every legal root move. With `rank_dtz`, wins are ordered by distance
/// to zeroing; that is forced where distance to zeroing is distance to mate.
/// A move into a claimable threefold, or with `use_rule50` a rule-50 draw,
/// ranks as a draw: Fathom does not see the game history. `None` when the
/// root is not in the tables or a probe fails.
pub(crate) fn rank_root_moves(
    board: &Board,
    use_rule50: bool,
    rank_dtz: bool,
) -> Option<RootRanking> {
    if !can_probe(board, use_rule50, true) {
        return None;
    }
    let mut ranking = rank_by_dtz(board, use_rule50, rank_dtz || board.dtz_is_dtm())
        .or_else(|| rank_by_wdl(board, use_rule50))?;
    ranking
        .moves
        .sort_by_key(|ranked| std::cmp::Reverse(ranked.rank));
    Some(ranking)
}

/// The score a ranked root move shows, in search units: a clean win the
/// tablebase value at the root, a win the rule-50 counter may spoil 1 to 49
/// centipawns growing toward the real win, a draw 0, losses mirrored. A WDL
/// ranking knows no distance, so its cursed results show 2.
pub(crate) fn display_score(rank: i32, dtz: bool, use_rule50: bool) -> i32 {
    use crate::tt::TB_VALUE;
    const PAWN: i32 = 100;
    if !dtz {
        return match rank {
            0 => 0,
            rank if rank.abs() >= MAX_DTZ || !use_rule50 => rank.signum() * TB_VALUE,
            rank => rank.signum() * 2,
        };
    }
    let bound = if use_rule50 { MAX_DTZ / 2 - 100 } else { 1 };
    if rank >= bound {
        TB_VALUE
    } else if rank > 0 {
        (rank - (MAX_DTZ / 2 - 200)).max(3) * PAWN / 200
    } else if rank == 0 {
        0
    } else if rank > -bound {
        (rank + (MAX_DTZ / 2 - 200)).min(-3) * PAWN / 200
    } else {
        -TB_VALUE
    }
}

fn rank_by_dtz(board: &Board, use_rule50: bool, rank_dtz: bool) -> Option<RootRanking> {
    let pos = tb_position(board);
    let mut results = [TB_RESULT_FAILED; TB_MAX_MOVES + 1];
    // SAFETY: FFI into Fathom. `results` holds TB_MAX_MOVES + 1 entries, enough
    // for every legal move and the terminator Fathom writes after them.
    let root = unsafe {
        tb_probe_root_impl(
            pos.white,
            pos.black,
            pos.kings,
            pos.queens,
            pos.rooks,
            pos.bishops,
            pos.knights,
            pos.pawns,
            pos.rule50,
            pos.ep,
            pos.turn,
            results.as_mut_ptr(),
        )
    };
    if root == TB_RESULT_FAILED {
        return None;
    }
    let cnt50 = i32::from(board.halfmove_clock());
    let repeated = board.has_repetition_since_zeroing();
    let mut after = board.clone();
    let mut moves = Vec::new();
    for &result in results
        .iter()
        .take_while(|&&result| result != TB_RESULT_FAILED)
    {
        let mv = legal_move_from_root_probe(board, root_move_from_result(result)?)?;
        let distance = i32::try_from((result & TB_RESULT_DTZ_MASK) >> TB_RESULT_DTZ_SHIFT).ok()?;
        // Fathom reports the move's result from the root side's view and
        // the absolute distance; the sign follows the result.
        let mut dtz = match wdl_from_raw((result & TB_RESULT_WDL_MASK) >> TB_RESULT_WDL_SHIFT)? {
            Wdl::Win | Wdl::CursedWin => distance,
            Wdl::Loss | Wdl::BlessedLoss => -distance,
            Wdl::Draw => 0,
        };
        after.make_move(mv);
        if after.halfmove_clock() != 0 && after.is_arbiter_draw(use_rule50) {
            dtz = 0;
        }
        after.unmake_move(mv);
        moves.push(RankedMove {
            mv,
            rank: dtz_rank(dtz, cnt50, repeated, rank_dtz),
        });
    }
    (!moves.is_empty()).then_some(RootRanking { dtz: true, moves })
}

/// Better moves rank higher. Certain wins rank equally unless `rank_dtz`;
/// losses rank equally unless a rule-50 draw is in sight.
fn dtz_rank(dtz: i32, cnt50: i32, repeated: bool, rank_dtz: bool) -> i32 {
    let order = if rank_dtz { dtz } else { 0 };
    if dtz > 0 {
        if dtz + cnt50 <= 99 && !repeated {
            MAX_DTZ - order
        } else {
            MAX_DTZ / 2 - (dtz + cnt50)
        }
    } else if dtz < 0 {
        if -dtz * 2 + cnt50 < 100 {
            -MAX_DTZ - order
        } else {
            -MAX_DTZ / 2 + (-dtz + cnt50)
        }
    } else {
        0
    }
}

/// The fallback when DTZ tables are missing: Fathom's per-move WDL ranks,
/// mapped onto the DTZ scale (a win `MAX_DTZ`, a cursed win `MAX_DTZ − 101`).
fn rank_by_wdl(board: &Board, use_rule50: bool) -> Option<RootRanking> {
    let probe = probe_root_moves_wdl(board, use_rule50)?;
    let mut after = board.clone();
    let mut moves = Vec::new();
    for &(root_move, fathom_rank) in &probe {
        let mv = legal_move_from_root_probe(board, root_move)?;
        after.make_move(mv);
        let drawn = after.is_arbiter_draw(use_rule50);
        after.unmake_move(mv);
        let rank = match fathom_rank {
            _ if drawn => 0,
            0 => 0,
            rank if rank.abs() >= 1000 => rank.signum() * MAX_DTZ,
            rank => rank.signum() * (MAX_DTZ - 101),
        };
        moves.push(RankedMove { mv, rank });
    }
    (!moves.is_empty()).then_some(RootRanking { dtz: false, moves })
}

fn probe_root_moves_wdl(board: &Board, use_rule50: bool) -> Option<Vec<(RootMove, i32)>> {
    let pos = tb_position(board);
    let mut results = TbRootMovesRaw::default();
    // SAFETY: FFI into Fathom. `results` is a live, default-initialised
    // `TbRootMovesRaw` that outlives the call and is only written by Fathom.
    let ok = unsafe {
        tb_probe_root_wdl(
            pos.white,
            pos.black,
            pos.kings,
            pos.queens,
            pos.rooks,
            pos.bishops,
            pos.knights,
            pos.pawns,
            pos.rule50,
            0,
            pos.ep,
            pos.turn,
            use_rule50,
            &mut results,
        )
    };
    if ok == 0 {
        return None;
    }
    let len = (results.size as usize).min(TB_MAX_MOVES);
    results
        .moves
        .iter()
        .take(len)
        .map(|result| Some((root_move_from_tb_move(result.mv)?, result.tb_rank)))
        .collect()
}

pub(crate) fn legal_move_from_root_probe(board: &Board, root_move: RootMove) -> Option<Move> {
    board.generate_legal_movelist().iter().copied().find(|mv| {
        mv.from_sq().0 == root_move.from
            && mv.to_sq().0 == root_move.to
            && mv.promotion() == root_move.promotes
    })
}

fn can_probe(board: &Board, use_rule50: bool, root: bool) -> bool {
    if largest() == 0 || board.castling().0 != 0 {
        return false;
    }
    if use_rule50 && !root && board.halfmove_clock() != 0 {
        return false;
    }
    board.occupied_count() as usize <= largest()
}

fn tb_position(board: &Board) -> TbPosition {
    TbPosition {
        white: board.color_occ(Color::White).0,
        black: board.color_occ(Color::Black).0,
        kings: (board.pieces(Color::White, Piece::King) | board.pieces(Color::Black, Piece::King))
            .0,
        queens: (board.pieces(Color::White, Piece::Queen)
            | board.pieces(Color::Black, Piece::Queen))
        .0,
        rooks: (board.pieces(Color::White, Piece::Rook) | board.pieces(Color::Black, Piece::Rook))
            .0,
        bishops: (board.pieces(Color::White, Piece::Bishop)
            | board.pieces(Color::Black, Piece::Bishop))
        .0,
        knights: (board.pieces(Color::White, Piece::Knight)
            | board.pieces(Color::Black, Piece::Knight))
        .0,
        pawns: (board.pieces(Color::White, Piece::Pawn) | board.pieces(Color::Black, Piece::Pawn))
            .0,
        rule50: board.halfmove_clock() as u32,
        ep: board.ep_square().map_or(0, |sq| sq.0 as u32),
        turn: board.side_to_move() == Color::White,
    }
}

fn root_move_from_result(result: u32) -> Option<RootMove> {
    root_move(
        ((result & TB_RESULT_FROM_MASK) >> TB_RESULT_FROM_SHIFT) as u8,
        ((result & TB_RESULT_TO_MASK) >> TB_RESULT_TO_SHIFT) as u8,
        (result & TB_RESULT_PROMOTES_MASK) >> TB_RESULT_PROMOTES_SHIFT,
    )
}

fn root_move_from_tb_move(mv: u16) -> Option<RootMove> {
    root_move(
        ((mv >> 6) & 0x3F) as u8,
        (mv & 0x3F) as u8,
        u32::from((mv >> 12) & 0x7),
    )
}

/// Fathom's promotion code (0 none, 1 queen … 4 knight) is shared by both of
/// its move encodings. An unknown code or a null move decodes to `None`.
fn root_move(from: u8, to: u8, promotion_code: u32) -> Option<RootMove> {
    let promotes = match promotion_code {
        0 => None,
        1 => Some(Piece::Queen),
        2 => Some(Piece::Rook),
        3 => Some(Piece::Bishop),
        4 => Some(Piece::Knight),
        _ => return None,
    };
    (from != to).then_some(RootMove { from, to, promotes })
}

fn wdl_from_raw(value: u32) -> Option<Wdl> {
    match value {
        TB_LOSS => Some(Wdl::Loss),
        TB_BLESSED_LOSS => Some(Wdl::BlessedLoss),
        TB_DRAW => Some(Wdl::Draw),
        TB_CURSED_WIN => Some(Wdl::CursedWin),
        TB_WIN => Some(Wdl::Win),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::board::Square;
    use std::fs;
    use std::sync::{LazyLock, Mutex};

    static TEST_SYZYGY_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn encoded_root_result(from: Square, to: Square, promotes: u32) -> u32 {
        (TB_WIN << TB_RESULT_WDL_SHIFT)
            | ((to.0 as u32) << TB_RESULT_TO_SHIFT)
            | ((from.0 as u32) << TB_RESULT_FROM_SHIFT)
            | (promotes << TB_RESULT_PROMOTES_SHIFT)
    }

    fn encoded_tb_move(from: Square, to: Square, promotes: u16) -> u16 {
        ((from.0 as u16) << 6) | to.0 as u16 | (promotes << 12)
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
    fn tablebase_file_counts_scan_semicolon_separated_paths() {
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

        let path = format!("{};{}", first.display(), second.display());
        assert_eq!(tablebase_file_counts(&path), (2, 1));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn tb_position_exports_bitboards_side_ep_and_rule50_state() {
        let board = Board::from_fen("4k3/8/8/8/4Pp2/8/8/4K3 b - e3 7 42").expect("valid FEN");

        let pos = tb_position(&board);

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
        assert_eq!(pos.ep, Square::E3.0 as u32);
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
}
