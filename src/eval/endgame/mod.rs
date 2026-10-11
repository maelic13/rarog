//! Endgame knowledge: the scale-factor dispatch and its recognisers, the KPK
//! bitbase and the mate-drive mop-up.

pub(crate) mod kpk;
mod mop_up;

use super::pawns::{FILE_BBS, KING_DISTANCE, PASSED_PAWN_MASKS, SQUARE_FILE};
use super::relative_rank;
use crate::board::{Bitboard, Board, Color, Piece, Square};
use crate::infra;

pub(crate) use mop_up::MOPUP_ASSUMED_MAX_PLY;

/// Endgame scale-factor framework. A scale of `SCALE_NORMAL` leaves the
/// tapered endgame score untouched; specialised functions return a smaller
/// scale (down to 0 = dead draw) for known material patterns, on a `/64` basis.
/// The opposite-coloured-bishop rule has its own `/48` basis.
pub(super) const SCALE_NORMAL: i32 = 64;
/// The `/48` basis of `opposite_bishop_scale`.
pub(super) const OCB_SCALE_NORMAL: i32 = 48;

/// Endgame scale-factor dispatch. Specialised material patterns are checked
/// first; everything else falls through to the opposite-coloured-bishop rule
/// and the KNNK draw.
pub(super) fn scale_endgame(board: &Board, mut score: i32) -> i32 {
    if let Some(sf) = specialized_endgame_scale(board) {
        return score * sf / SCALE_NORMAL;
    }

    if let Some(scale) = opposite_bishop_scale(board) {
        score = score * scale / OCB_SCALE_NORMAL;
    }

    if has_only_king(board, Color::White) && has_only_knights(board, Color::Black, 2) {
        return 0;
    }
    if has_only_king(board, Color::Black) && has_only_knights(board, Color::White, 2) {
        return 0;
    }

    score
}

/// Specialised known-endgame scale factor (Phase 3.11), in `0..=SCALE_NORMAL`.
/// Returns `Some(sf)` for a recognised material pattern (the caller multiplies
/// the tapered score by `sf / SCALE_NORMAL`), or `None` to fall through to the
/// general scaling. Each pattern fires only on exact material absent from the
/// bench suite, so `bench 13` is unchanged.
///
/// Implemented: KPK bitbase, KBP wrong-corner draw, pawnless
/// insufficient-mating-material draws (KK/KNK/KBK/minor-vs-minor), and the
/// Phase-3.11c patterns — the KQKP fortress draw (rook/bishop pawn only) and a
/// conservative KRKP partial scale. KNNK keeps its existing handling in
/// `scale_endgame`. Deliberately omitted: KQ-vs-KR (a win — never scaled toward
/// draw) and broad rook-endgame drawishness (a tunable Phase-4 term, not a
/// hardcoded rule that could wrongly draw a won R+P vs R).
pub(super) fn specialized_endgame_scale(board: &Board) -> Option<i32> {
    // KPK (king + single pawn vs lone king): exact bitbase verdict. A drawn KPK
    // is forced to 0; a won one falls through (`None`) so normal eval scores it.
    if let Some((white_to_move, wk, bk, p)) = kpk_normalized(board) {
        return if kpk::probe(white_to_move, wk, bk, p) {
            None
        } else {
            Some(0)
        };
    }
    // KBP with a wrong-coloured bishop and a rook pawn, defender on the corner:
    // a textbook dead draw the bishop cannot break.
    if kbp_wrong_corner_draw(board) {
        return Some(0);
    }
    // KQ vs KP fortress (Phase 3.11c): only the textbook drawn case — a rook or
    // bishop pawn on the 7th, its king guarding the queening square, the queen's
    // king too far to break through. Knight/centre pawns are wins and are left
    // untouched.
    if let Some(sf) = kqkp_fortress_scale(board) {
        return Some(sf);
    }
    // KR vs KP (Phase 3.11c): conservative *partial* scale toward draw in the
    // clear draw zone (pawn on the 7th, escorted by its king, rook's king far).
    // Never a forced draw, so an actually-won KRKP keeps a clearly winning score.
    if let Some(sf) = krkp_drawish_scale(board) {
        return Some(sf);
    }
    // KRP vs KR (Phase 4.9a.7): the highest-value open reference family, at
    // 10.04% of real games.
    if let Some(sf) = krpkr_scale(board) {
        return Some(sf);
    }
    // KRP vs KB (Phase 4.9a.8): rook-pawn fortresses only; partial scales.
    if let Some(sf) = krpkb_scale(board) {
        return Some(sf);
    }

    let no_pawns = board.pieces(Color::White, Piece::Pawn).is_empty()
        && board.pieces(Color::Black, Piece::Pawn).is_empty();
    if !no_pawns {
        return None;
    }
    let majors = board.pieces(Color::White, Piece::Rook)
        | board.pieces(Color::Black, Piece::Rook)
        | board.pieces(Color::White, Piece::Queen)
        | board.pieces(Color::Black, Piece::Queen);
    if majors.any() {
        return None;
    }
    let white_minors = board.pieces(Color::White, Piece::Knight).count()
        + board.pieces(Color::White, Piece::Bishop).count();
    let black_minors = board.pieces(Color::Black, Piece::Knight).count()
        + board.pieces(Color::Black, Piece::Bishop).count();
    // KK / KNK / KBK (at most one minor on the board) and minor-vs-minor are
    // dead draws. K+B+N vs K (one side has two minors, the other none) is a
    // WIN and is deliberately *not* matched here — it falls through so the
    // KBNK corner-drive scores it.
    if white_minors + black_minors <= 1 || (white_minors == 1 && black_minors == 1) {
        return Some(0);
    }
    None
}

/// If `board` is exactly K+P vs K, return the KPK bitbase probe arguments
/// normalised so the pawn is White's (mirroring vertically when the pawn is
/// Black's). Returns `None` for any other material.
fn kpk_normalized(board: &Board) -> Option<(bool, usize, usize, usize)> {
    let wp = board.pieces(Color::White, Piece::Pawn);
    let bp = board.pieces(Color::Black, Piece::Pawn);
    if wp.count() + bp.count() != 1 {
        return None;
    }
    for color in [Color::White, Color::Black] {
        for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
            if board.pieces(color, piece).any() {
                return None;
            }
        }
    }
    let wk = board.king_sq(Color::White).index();
    let bk = board.king_sq(Color::Black).index();
    if wp.any() {
        let p = wp.lsb().index();
        Some((board.side_to_move() == Color::White, wk, bk, p))
    } else {
        // Black has the pawn: mirror vertically (square ^ 56) so the pawn is
        // White's, swapping the kings' roles and the side to move.
        let p = bp.lsb().index();
        Some((
            board.side_to_move() == Color::Black,
            bk ^ 56,
            wk ^ 56,
            p ^ 56,
        ))
    }
}

/// True for the wrong-bishop rook-pawn draw. All of the following hold:
/// the strong side has K + one bishop + one or more pawns all on a single rook
/// file; the bishop is the wrong colour to control the queening square; and the
/// bare defending king holds that corner (within one square of it). Such
/// positions are dead draws.
fn kbp_wrong_corner_draw(board: &Board) -> bool {
    for strong in [Color::White, Color::Black] {
        let weak = !strong;
        if board.color_occ(weak) != Bitboard::from(board.king_sq(weak)) {
            continue;
        }
        if board.pieces(strong, Piece::Knight).any()
            || board.pieces(strong, Piece::Rook).any()
            || board.pieces(strong, Piece::Queen).any()
        {
            continue;
        }
        let bishops = board.pieces(strong, Piece::Bishop);
        if bishops.count() != 1 {
            continue;
        }
        let pawns = board.pieces(strong, Piece::Pawn);
        if pawns.is_empty() {
            continue;
        }
        let on_a = (pawns & FILE_BBS[0]) == pawns;
        let on_h = (pawns & FILE_BBS[7]) == pawns;
        if !on_a && !on_h {
            continue;
        }
        let file = if on_a { 0 } else { 7 };
        let queening = if strong == Color::White {
            file + 56
        } else {
            file
        };
        let queening_sq = Square(infra::to_u8(queening));
        // A bishop can only guard the queening square if it shares that square's
        // colour. Wrong colour → it can never evict the king from the corner.
        let bishop_light = (bishops & Bitboard::LIGHT_SQUARES).any();
        let queen_light = (Bitboard::from(queening_sq) & Bitboard::LIGHT_SQUARES).any();
        if bishop_light == queen_light {
            continue;
        }
        let weak_king = board.king_sq(weak);
        if KING_DISTANCE[weak_king.index()][queening_sq.index()] as i32 <= 1 {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Phase 3.11c — narrow, high-confidence endgame knowledge. Each fires only on
// an exact material pattern (verified absent from the bench suite, so the
// fingerprint is unchanged). Two deliberate omissions vs the original plan list:
//   * KQ-vs-KR is a *win*, so it is never scaled toward draw; and
//   * general rook-endgame drawishness is a tunable Phase-4 term, not a
//     hardcoded rule that could wrongly draw a won R+P vs R.
// ---------------------------------------------------------------------------

/// KQ vs KP fortress draw. The weak side has K+P with the pawn on the 7th, and
/// — crucially — it is a **rook or bishop pawn** (a/c/f/h), the only files where
/// KQ-vs-KP is a fortress draw. Its king must guard the queening square and the
/// queen's king be too far to break through. Knight and centre pawns are wins
/// and return `None`. Returns `Some(0)` (dead draw) for the recognised fortress.
fn kqkp_fortress_scale(board: &Board) -> Option<i32> {
    for strong in [Color::White, Color::Black] {
        let weak = !strong;
        if !has_exact_material(board, strong, 0, 0, 0, 0, 1)
            || !has_exact_material(board, weak, 1, 0, 0, 0, 0)
        {
            continue;
        }
        let pawn = board.pieces(weak, Piece::Pawn).lsb();
        if relative_rank(weak, pawn) != 6 || !is_rook_or_bishop_pawn(pawn) {
            continue;
        }
        let queening = promotion_square(weak, pawn);
        let weak_king = board.king_sq(weak);
        let strong_king = board.king_sq(strong);
        if KING_DISTANCE[weak_king.index()][queening.index()] <= 1
            && KING_DISTANCE[strong_king.index()][queening.index()] > 2
        {
            return Some(0);
        }
    }
    None
}

/// KR vs KP drawish heuristic. When the weak side's pawn is on the 7th escorted
/// by its king and the rook's king is far (> 4 — deliberately conservative), the
/// ending is usually drawn (the rook must give itself for the pawn). A
/// *partial* scale (≈¼) only — never a forced draw — so an actually-won KRKP
/// keeps a clearly winning score and a wrong guess cannot throw the game.
/// KRP vs KB (Phase 4.9a.8). The rook side is winning almost everywhere, so
/// the drawn subset is small -- 94 of 4,000 sampled positions, 2.35% -- and the
/// evaluator was wrong about nearly all of it: **95.74% of drawn KRP-KB
/// positions scored above +100 cp, mean +347.2, max +771**.
///
/// **The reference only addresses ROOK pawns**, so it can reach at most about a
/// quarter of that cohort by construction; a non-rook-pawn KRP-KB draw is
/// outside its case analysis entirely. Ported from `sf_11` `endgame.cpp`, which
/// returns partial scales here (24, 48, 8 on the same `/64` basis) and never a
/// forced draw -- appropriate for a family this close to won.
///
/// Rank-normalised so the strong side is White and the pawn pushes `+8`. Every
/// predicate used below -- square colour agreement, diagonal alignment,
/// Chebyshev distance -- is preserved under a rank flip, since both squares in
/// each comparison flip together.
fn krpkb_scale(board: &Board) -> Option<i32> {
    for strong in [Color::White, Color::Black] {
        let weak = !strong;
        if !has_exact_material(board, strong, 1, 0, 0, 1, 0)
            || !has_exact_material(board, weak, 0, 0, 1, 0, 0)
        {
            continue;
        }
        let pawn_sq = board.pieces(strong, Piece::Pawn).lsb();
        let pawn_file = SQUARE_FILE[pawn_sq.index()];
        // Rook pawn only; the reference has no case for anything else.
        if pawn_file != 0 && pawn_file != 7 {
            continue;
        }

        let flip: u8 = if strong == Color::Black { 56 } else { 0 };
        let psq = pawn_sq.0 ^ flip;
        let ksq = board.king_sq(weak).0 ^ flip;
        let bsq = board.pieces(weak, Piece::Bishop).lsb().0 ^ flip;
        let strong_ksq = board.king_sq(strong).0 ^ flip;

        let rank = |s: u8| s / 8;
        let file = |s: u8| s % 8;
        let dist = |a: u8, b: u8| i32::from(KING_DISTANCE[usize::from(a)][usize::from(b)]);
        let same_colour = |a: u8, b: u8| (rank(a) + file(a)) % 2 == (rank(b) + file(b)) % 2;
        let on_diagonal = |a: u8, b: u8| {
            (i32::from(file(a)) - i32::from(file(b))).abs()
                == (i32::from(rank(a)) - i32::from(rank(b))).abs()
        };

        let rk = rank(psq);
        // Pawn on the 5th with the bishop on the pawn's own colour: a fortress
        // is possible, and how good it is depends on where the defending king
        // stands relative to the queening square.
        if rk == 4 && same_colour(bsq, psq) {
            let d = dist(psq + 24, ksq);
            return if d <= 2 && !(d == 0 && ksq == strong_ksq + 16) {
                Some(24)
            } else {
                Some(48)
            };
        }
        // Pawn on the 6th, defending king beside the queening square, bishop
        // covering the square in front of the pawn from a distance.
        if rk == 5
            && dist(psq + 16, ksq) <= 1
            && on_diagonal(bsq, psq + 8)
            && (i32::from(file(bsq)) - i32::from(file(psq))).abs() >= 2
        {
            return Some(8);
        }
    }
    None
}

/// KRP vs KR (Phase 4.9a.7), the highest-expected-value open reference family:
/// 10.04% of real games by RAR-M15 occurrence.
///
/// WHAT THE DEFECT ACTUALLY IS. The conversion number looks alarming and is
/// not the problem: RAR-E11 measured Stockfish 18 converting this family at
/// 47.9% against Rarog's 43.8% at the same node budget, so the reachable mark
/// is about four points away, not fifty-five. The measured defect is the
/// complementary cohort -- **37.1% of theoretically DRAWN KRP-KR positions
/// scored above +100 cp** (46 of 124, mean +93.8, max +473), which makes the
/// engine steer into dead-drawn rook endings from positions it could have won
/// another way. No measurement taken inside the ending can see that happen;
/// `tools/diag/endgame_drawn.py` is the instrument for it.
///
/// The case analysis is ported from the final pre-NNUE Stockfish (`sf_11`
/// `endgame.cpp`), whose own comment calls it "far from perfect" and notes it
/// descends from Glaurung. Both engines use a `/64` scale basis, so the
/// constants transfer without rescaling -- a seed, not a result.
///
/// **The reference's two WINNING branches are deliberately NOT ported.** They
/// return `SCALE_FACTOR_MAX - k*distance`, i.e. above the neutral 64, which
/// AMPLIFIES the score. Rarog's surface is Texel-fitted and its magnitudes are
/// not the reference's, so an untested amplifier interacts with a fitted
/// evaluation in a way the drawn cohort cannot measure. Only the draw
/// detection is taken, which is what the measured defect calls for.
fn krpkr_scale(board: &Board) -> Option<i32> {
    for strong in [Color::White, Color::Black] {
        let weak = !strong;
        if !has_exact_material(board, strong, 1, 0, 0, 1, 0)
            || !has_exact_material(board, weak, 0, 0, 0, 1, 0)
        {
            continue;
        }
        // Normalise to the reference frame: strong side is White, pawn on
        // files A-D. Rank flip for a Black strong side, file mirror for a
        // pawn on the east half; both are index XORs, so the whole case
        // analysis below reads exactly as the reference does.
        // Indices stay `u8` throughout, so every widening below is infallible
        // (`i32::from`) and the function needs no cast suppressions.
        let pawn_sq = board.pieces(strong, Piece::Pawn).lsb();
        let flip: u8 = if strong == Color::Black { 56 } else { 0 };
        let mirror: u8 = if SQUARE_FILE[pawn_sq.index()] >= 4 {
            7
        } else {
            0
        };
        let norm = |sq: Square| sq.0 ^ flip ^ mirror;

        let wk = norm(board.king_sq(strong));
        let bk = norm(board.king_sq(weak));
        let wr = norm(board.pieces(strong, Piece::Rook).lsb());
        let br = norm(board.pieces(weak, Piece::Rook).lsb());
        let wp = norm(pawn_sq);

        let rank = |s: u8| s / 8;
        let file = |s: u8| s % 8;
        let dist = |a: u8, b: u8| i32::from(KING_DISTANCE[usize::from(a)][usize::from(b)]);
        let file_dist = |a: u8, b: u8| (i32::from(file(a)) - i32::from(file(b))).abs();

        let r = rank(wp);
        let queening = 56 + file(wp);
        // The reference's `tempo` is 1 when the strong side is to move.
        let tempo = i32::from(board.side_to_move() == strong);

        // Third-rank defence: pawn not far advanced, defending king on the
        // queening square, defending rook cutting on the 6th.
        if r <= 4
            && dist(bk, queening) <= 1
            && wk <= 39
            && (rank(br) == 5 || (r <= 2 && rank(wr) != 5))
        {
            return Some(0);
        }
        // Checking from behind once the pawn reaches the 6th.
        if r == 5
            && dist(bk, queening) <= 1
            && i32::from(rank(wk)) + tempo <= 5
            && (rank(br) == 0 || (tempo == 0 && file_dist(br, wp) >= 3))
        {
            return Some(0);
        }
        if r >= 5 && bk == queening && rank(br) == 0 && (tempo == 0 || dist(wk, wp) >= 2) {
            return Some(0);
        }
        // Pawn a7, rook a8, defending king boxed on g7/h7 with its rook behind.
        if wp == 48
            && wr == 56
            && (bk == 55 || bk == 54)
            && file(br) == 0
            && (rank(br) <= 2 || file(wk) >= 3 || rank(wk) <= 4)
        {
            return Some(0);
        }
        // Defending king blockades the pawn and the attacking king is far.
        if r <= 4 && bk == wp + 8 && dist(wk, wp) - tempo >= 2 && dist(wk, br) - tempo >= 2 {
            return Some(0);
        }
        // Not a forced draw, but drawish: the defending king sits in the
        // pawn's path with the pawn still short of the 5th. Partial scales, so
        // an actually-won position keeps a winning score.
        if r <= 3 && bk > wp {
            if file(bk) == file(wp) {
                return Some(10);
            }
            if file_dist(bk, wp) == 1 && dist(wk, bk) > 2 {
                return Some(24 - 2 * dist(wk, bk));
            }
        }
    }
    None
}

fn krkp_drawish_scale(board: &Board) -> Option<i32> {
    for strong in [Color::White, Color::Black] {
        let weak = !strong;
        if !has_exact_material(board, strong, 0, 0, 0, 1, 0)
            || !has_exact_material(board, weak, 1, 0, 0, 0, 0)
        {
            continue;
        }
        let pawn = board.pieces(weak, Piece::Pawn).lsb();
        if relative_rank(weak, pawn) != 6 {
            continue;
        }
        let queening = promotion_square(weak, pawn);
        let weak_king = board.king_sq(weak);
        let strong_king = board.king_sq(strong);
        if KING_DISTANCE[weak_king.index()][queening.index()] <= 1
            && KING_DISTANCE[strong_king.index()][queening.index()] > 4
        {
            return Some(16); // ×0.25 of SCALE_NORMAL
        }
    }
    None
}

/// Pure bishop endings with at least this many pawns take the fitted rule.
const PURE_OCB_MIN_PAWNS: i32 = 3;

/// Opposite-coloured-bishop scaling (one bishop each, on opposite colours), on
/// the `/48` basis, applied to the whole score.
///
/// A pure bishop ending (no knight, rook or queen) with three pawns or more is
/// mostly drawn whatever the pawn balance, so its scale starts near zero and
/// rises with passed pawns: `1 + 2·pawns + 10·passers`, fitted against game
/// outcomes. Below three pawns, and with other pieces on the board, the older
/// `32 + 4·pawns + 4·passers` stays, because the fitted rule applied there
/// cost conversion of won two-pawn endings at a fixed node budget. The two
/// regimes do not meet continuously: an exchange from three pawns to two can
/// raise the score.
pub(super) fn opposite_bishop_scale(board: &Board) -> Option<i32> {
    let white_bishops = board.pieces(Color::White, Piece::Bishop);
    let black_bishops = board.pieces(Color::Black, Piece::Bishop);
    if white_bishops.is_empty()
        || white_bishops.more_than_one()
        || black_bishops.is_empty()
        || black_bishops.more_than_one()
    {
        return None;
    }
    let white_dark = (white_bishops & Bitboard::DARK_SQUARES).any();
    let black_dark = (black_bishops & Bitboard::DARK_SQUARES).any();
    if white_dark == black_dark {
        return None;
    }
    let pawns = infra::to_i32(
        (board.pieces(Color::White, Piece::Pawn) | board.pieces(Color::Black, Piece::Pawn)).count(),
    );
    let passers = count_passed_pawns(board);
    let pieces = board.pieces(Color::White, Piece::Knight)
        | board.pieces(Color::Black, Piece::Knight)
        | board.pieces(Color::White, Piece::Rook)
        | board.pieces(Color::Black, Piece::Rook)
        | board.pieces(Color::White, Piece::Queen)
        | board.pieces(Color::Black, Piece::Queen);
    if pieces.is_empty() && pawns >= PURE_OCB_MIN_PAWNS {
        return Some((1 + pawns * 2 + passers * 10).min(OCB_SCALE_NORMAL));
    }
    Some((32 + pawns * 4 + passers * 4).min(OCB_SCALE_NORMAL))
}

fn count_passed_pawns(board: &Board) -> i32 {
    let mut count = 0;
    for color in [Color::White, Color::Black] {
        let enemy_pawns = board.pieces(!color, Piece::Pawn);
        let mut pawns = board.pieces(color, Piece::Pawn);
        while pawns.any() {
            let sq = pawns.pop_lsb();
            if (PASSED_PAWN_MASKS[color as usize][sq.index()] & enemy_pawns).is_empty() {
                count += 1;
            }
        }
    }
    count
}

fn has_exact_material(
    board: &Board,
    color: Color,
    pawns: u32,
    knights: u32,
    bishops: u32,
    rooks: u32,
    queens: u32,
) -> bool {
    board.pieces(color, Piece::Pawn).count() == pawns
        && board.pieces(color, Piece::Knight).count() == knights
        && board.pieces(color, Piece::Bishop).count() == bishops
        && board.pieces(color, Piece::Rook).count() == rooks
        && board.pieces(color, Piece::Queen).count() == queens
}

fn promotion_square(color: Color, pawn: Square) -> Square {
    let file = SQUARE_FILE[pawn.index()];
    match color {
        Color::White => Square(infra::to_u8(file + 56)),
        Color::Black => Square(infra::to_u8(file)),
    }
}

/// A rook (a/h) or bishop (c/f) pawn — the files on which KQ-vs-KP is a fortress
/// draw.
fn is_rook_or_bishop_pawn(pawn: Square) -> bool {
    matches!(SQUARE_FILE[pawn.index()], 0 | 2 | 5 | 7)
}

pub(super) fn has_only_king(board: &Board, color: Color) -> bool {
    board.color_occ(color) == Bitboard::from(board.king_sq(color))
}

pub(super) fn has_only_knights(board: &Board, color: Color, count: u32) -> bool {
    board.pieces(color, Piece::Pawn).is_empty()
        && board.pieces(color, Piece::Bishop).is_empty()
        && board.pieces(color, Piece::Rook).is_empty()
        && board.pieces(color, Piece::Queen).is_empty()
        && board.pieces(color, Piece::Knight).count() == count
}

#[cfg(test)]
mod endgame_311c_tests {
    use super::*;
    use crate::eval::Evaluator;

    fn board(fen: &str) -> Board {
        Board::from_fen(fen).unwrap_or_else(|e| panic!("bad test FEN {fen}: {e}"))
    }

    fn static_eval(fen: &str) -> i32 {
        Evaluator::default().evaluate(&board(fen))
    }

    /// The chess rule, not a hardcoded constant: KQ-vs-KP is a fortress draw
    /// only for rook/bishop pawns — knight and centre pawns are wins and must
    /// not be scaled.
    #[test]
    fn kqkp_fortress_only_draws_rook_and_bishop_pawns() {
        // Bishop pawn (c) on the 7th, king guarding c1, queen's king far: drawn.
        assert_eq!(
            kqkp_fortress_scale(&board("8/8/6K1/8/8/8/1kp5/7Q w - - 0 1")),
            Some(0)
        );
        // Knight pawn (b) on the 7th — a win for the queen — must NOT scale.
        assert_eq!(
            kqkp_fortress_scale(&board("8/8/6K1/8/8/8/kp6/7Q w - - 0 1")),
            None
        );
    }

    /// KR-vs-KP draw zone gets a conservative *partial* scale, never a forced 0.
    #[test]
    fn krkp_partial_scale_in_the_draw_zone() {
        assert_eq!(
            krkp_drawish_scale(&board("8/8/6K1/8/8/8/1kp5/7R w - - 0 1")),
            Some(16)
        );
    }

    /// KRP-vs-KB rook-pawn fortresses (4.9a.8). Both FENs are Syzygy DRAW.
    ///
    /// This is also the **live-wire proof** the null result needs: the drawn
    /// cohort's overclaim rate did not move at all (0.9574 before and after),
    /// because the reference addresses only rook pawns -- at most a quarter of
    /// that cohort -- and its partial scales leave a +350 score above the
    /// 100 cp threshold even where they fire. A null from a dead wire and a
    /// null from a narrow mechanism look identical in the aggregate, so the
    /// mechanism is asserted directly here instead.
    #[test]
    fn krpkb_scales_rook_pawn_fortresses() {
        // Pawn a5, bishop on the pawn's own colour, defending king next to the
        // queening square: the reference's moderate fortress reduction.
        assert_eq!(
            krpkb_scale(&board("8/1kb5/8/P7/8/8/8/6KR w - - 0 1")),
            Some(24)
        );
        // A non-rook pawn is outside the reference's case analysis entirely,
        // and must fall through rather than be scaled on a guess.
        assert_eq!(
            krpkb_scale(&board("8/2kb4/8/1P6/8/8/8/6KR w - - 0 1")),
            None
        );
    }

    /// KRP-vs-KR draw recognition (4.9a.7). Every FEN here was probed against
    /// Syzygy before it was written down, so these assert chess truth rather
    /// than the recognizer's own opinion of itself.
    #[test]
    fn krpkr_recognizes_textbook_draws() {
        // Philidor third-rank defence: defending rook on the 6th cuts the
        // attacking king while the pawn is still short of it. Syzygy: DRAW.
        assert_eq!(
            krpkr_scale(&board("8/3k4/r7/3PK3/8/8/8/7R w - - 0 1")),
            Some(0)
        );
        // Pawn a7 with the rook in front on a8 and the defending king boxed on
        // h7, its rook behind the pawn. Syzygy: DRAW.
        assert_eq!(
            krpkr_scale(&board("R7/P6k/8/8/8/8/8/r5K1 w - - 0 1")),
            Some(0)
        );
    }

    /// The failure mode that matters: a hard `Some(0)` on a position that is
    /// actually WON would turn wins into claimed draws. The reference calls its
    /// own case analysis "far from perfect", so this is the guard that earns
    /// porting it. All three are Syzygy WIN, so the endgame scaling must also
    /// leave a winning score at least half its size. The scaling is tested on
    /// a fixed score, not the evaluation's, whose size every refit moves.
    #[test]
    fn krpkr_never_zeroes_a_won_position() {
        const WINNING: i32 = 300;
        for fen in [
            "1R6/1P6/8/8/8/7k/r7/6K1 w - - 0 1",
            "8/8/1PK5/8/8/7k/r7/1R6 w - - 0 1",
            "8/8/8/1PK5/8/7k/r7/1R6 w - - 0 1",
        ] {
            let position = board(fen);
            assert_ne!(
                krpkr_scale(&position),
                Some(0),
                "won KRP-KR must not be scaled to a forced draw: {fen}"
            );
            let scaled = scale_endgame(&position, WINNING);
            assert!(
                scaled >= WINNING / 2,
                "won KRP-KR should stay clearly winning, scaled {WINNING} to {scaled} for {fen}"
            );
        }
    }

    /// Won endings must keep a clearly winning static score — guards against the
    /// earlier bug of scaling wins (knight-pawn KQKP, and KQ-vs-KR) toward draw.
    #[test]
    fn won_endings_are_not_scaled_toward_draw() {
        let kqkp_knight_pawn = static_eval("8/8/6K1/8/8/8/kp6/7Q w - - 0 1");
        assert!(
            kqkp_knight_pawn > 300,
            "won KQ vs knight-pawn should stay clearly winning, got {kqkp_knight_pawn}"
        );
        let kqkr = static_eval("8/8/6KQ/8/3k4/8/8/3r4 w - - 0 1");
        assert!(
            kqkr > 150,
            "won KQ vs KR should stay clearly winning, got {kqkr}"
        );
    }

    /// Below three pawns a pure bishop ending keeps the older rule, which
    /// passed pawns relax upward.
    #[test]
    fn opposite_bishop_scale_relaxed_by_passers() {
        assert_eq!(
            opposite_bishop_scale(&board("4k3/p7/8/3b4/8/8/P7/2B1K3 w - - 0 1")),
            Some(40) // 2 pawns, no passers: 32 + 2*4
        );
        assert_eq!(
            opposite_bishop_scale(&board("4k3/7p/P7/3b4/8/8/8/2B1K3 w - - 0 1")),
            Some(48) // 2 passed pawns relax to the /48 cap
        );
    }

    /// From three pawns a pure bishop ending takes the fitted rule; other
    /// pieces on the board keep the older one at any pawn count.
    #[test]
    fn opposite_bishop_scale_pure_endings_from_three_pawns() {
        assert_eq!(
            opposite_bishop_scale(&board("4k3/p7/8/3b4/8/8/PP6/2B1K3 w - - 0 1")),
            Some(7) // 3 pawns, no passers: 1 + 3*2
        );
        assert_eq!(
            opposite_bishop_scale(&board("4k3/7p/P7/3b4/8/8/P7/2B1K3 w - - 0 1")),
            Some(37) // 3 pawns, all passed: 1 + 3*2 + 3*10
        );
        assert_eq!(
            opposite_bishop_scale(&board("r3k3/p7/8/3b4/8/8/PP6/2B1K2R w - - 0 1")),
            Some(44) // rooks on: 32 + 3*4
        );
    }
}
