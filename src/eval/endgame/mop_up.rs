//! The mate-drive mop-up: when one side is clearly winning, push the losing
//! king toward an edge or the right corner. It reads the running `(mg + eg)`
//! sum: on the full path after piece activity and before imbalance, on the
//! lazy path in place of both.

use super::super::pawns::{KING_DISTANCE, SQUARE_FILE, SQUARE_RANK};
use super::super::{Evaluator, MATE_SCORE, color_sign};
use crate::board::{Bitboard, Board, Color, Piece};
use crate::infra;

/// Manhattan (taxicab) distance between squares, used only by the mate drive.
///
/// 4.9a.4: the drive was pure Chebyshev, and Chebyshev is FLAT -- every square
/// in a ring around the target scores identically. Measured over 300
/// theoretically won KBNK positions, the 19 legal moves collapsed into a median
/// of 3 distinct mop-up values and **94% of positions had a tied best move**,
/// median best-vs-second gap 0 cp. A term that cannot order its own moves
/// cannot steer a search, which is why raising the magnitude alone would have
/// changed nothing: 40x a 0 cp gap is still 0. Manhattan breaks the rings.
const MANHATTAN_DISTANCE: [[u8; 64]; 64] = init_manhattan_distance();
/// Mate-drive weights (4.9a.4). Chebyshev carries the coarse pull and Manhattan
/// supplies the resolution that breaks its rings; the pair is what takes the
/// tied-best-move rate from 94% to 11% on won KBNK positions.
///
/// Three things were needed and they were found in this order: RESOLUTION (a
/// flat metric cannot order its own moves), MAGNITUDE (once ordered, the
/// difference must survive the pruning thresholds), and RATIO (the corner pull
/// must dominate the king pull, not merely exceed it). Swept on KBN-K
/// conversion at 100 positions, 60k nodes:
///
///   Chebyshev, 8/4 (accepted head)     19.4%    KBB-K  78.0%
///   Chebyshev+Manhattan 16/8/10/5      32.7%    KBB-K  96.0%
///   Chebyshev+Manhattan 32/16/20/10    57.1%    KBB-K 100.0%
///   diagonal 60  (~1:1 vs king terms)  56.1%    KBB-K 100.0%
///   diagonal 120 (~3:1)                83.7%    KBB-K 100.0%
///   diagonal 240 (~6:1)                94.9%    KBB-K 100.0%
///   diagonal 360 (THESE)               96.9%    KBB-K 100.0%
///   diagonal 480                       96.9%    KBB-K 100.0%
///   diagonal 720                       94.9%    KBB-K 100.0%
///
/// 360 and 480 tie at the peak; the smaller is taken. The term is gated on
/// `|approximate| > 200` and on minor-piece mates, so even at this size it only
/// applies to an already-won bare-king ending and `bench 13` is unchanged.
const MOPUP_DIAGONAL: i32 = 360;
const MOPUP_KING_CHEB: i32 = 20;
const MOPUP_KING_MAN: i32 = 10;

/// The largest score `apply_mop_up`'s minor-mate branch can add.
///
/// `diagonal` peaks at 7 (a corner), `7 - king_distance` at 7 (adjacent kings)
/// and `14 - king_man` at 14 (adjacent kings again), so this is the exact
/// supremum rather than a bound with slack.
const MOPUP_MAX: i32 = MOPUP_DIAGONAL * 7 + MOPUP_KING_CHEB * 7 + MOPUP_KING_MAN * 14;

/// The search's ply horizon, mirrored here on purpose.
///
/// `search::MAX_PLY` is module-private, and the evaluator should not depend on
/// the search to know its own safety bound. `search::tests::
/// mopup_mirror_matches_the_real_ply_horizon` ties the two together in a module
/// that legitimately sees both, so the mirror cannot drift silently.
pub(crate) const MOPUP_ASSUMED_MAX_PLY: i32 = 128;

/// A guidance term must never reach the band the search reads as a forced mate.
///
/// This is a `const` assertion rather than a debug check or a validator inside
/// an option-setter, and that is the point (PLAN 4.10.11): the SHIPPED default
/// is what plays the games, so the bound has to hold in every build type
/// including release, not only where a tuning build happens to compile a
/// setter. Basilisk shipped exactly that gap -- its bound was enforced only in
/// tuning builds and the release default was validated by nothing (BAS-E52).
///
/// If this fails, the drive can manufacture a mate score out of king geometry
/// and the search will believe it.
const _: () = assert!(
    MOPUP_MAX < MATE_SCORE - MOPUP_ASSUMED_MAX_PLY,
    "mop-up drive can reach the search's mate band; lower MOPUP_DIAGONAL"
);

/// The search reads everything from the tablebase-win score up as decisive,
/// below the mate band, so the drive must stay under that lower line too.
const _: () = assert!(
    MOPUP_MAX < crate::tt::TB_WIN_SCORE,
    "mop-up drive can reach the search's decisive band; lower MOPUP_DIAGONAL"
);

// KBNK mate (Phase 3.11): the bare king is driven to a corner the winning
// bishop can actually reach — i.e. a corner of the bishop's own square colour.
// A bishop can only reach corners that share its colour, so these sets are
// exactly the corner squares contained in `Bitboard::LIGHT_SQUARES` /
// `DARK_SQUARES`. NB this engine's colour convention puts a1 in LIGHT_SQUARES
// (see bitboard.rs), so the "light" corners are a1(0) and h8(63).
const KBNK_LIGHT_CORNERS: [usize; 2] = [0, 63]; // a1, h8 — on LIGHT_SQUARES
const KBNK_DARK_CORNERS: [usize; 2] = [7, 56]; // h1, a8 — on DARK_SQUARES

// Const-evaluated init, same justification as `init_relative_ranks` above: the
// `infra` helpers are not `const fn`, both operands are file/rank differences in
// 0..=7 so the sum is 0..=14, and any out-of-range would surface at COMPILE
// time. Sign loss cannot occur because both differences are taken larger-minus-
// smaller.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
const fn init_manhattan_distance() -> [[u8; 64]; 64] {
    let mut table = [[0u8; 64]; 64];
    let mut a = 0usize;
    while a < 64 {
        let af = (a & 7) as i32;
        let ar = (a >> 3) as i32;
        let mut b = 0usize;
        while b < 64 {
            let bf = (b & 7) as i32;
            let br = (b >> 3) as i32;
            let df = if af > bf { af - bf } else { bf - af };
            let dr = if ar > br { ar - br } else { br - ar };
            table[a][b] = (df + dr) as u8;
            b += 1;
        }
        a += 1;
    }
    table
}

impl Evaluator {
    /// Mate-drive "mop-up": when one side is clearly winning, nudge the losing
    /// king toward the edge/corner (the bishop's corner for KBNK). Extracted
    /// from `eval_piece_activity` so it also runs on the lazy-eval early-return
    /// path (Phase 3.16) — mating technique must survive a lazy skip. Frozen
    /// (non-tunable) term, so it lands in the tuner's `rest`.
    pub(in crate::eval) fn apply_mop_up(&self, board: &Board, mg: &mut i32, eg: &mut i32) {
        let approximate = (*mg + *eg) / 2;
        if approximate.abs() > 200 {
            let winning = if approximate > 0 {
                Color::White
            } else {
                Color::Black
            };
            let losing = !winning;
            let sign = color_sign(winning);
            let lksq = board.king_sq(losing);
            let wksq = board.king_sq(winning);
            let king_distance = KING_DISTANCE[wksq.index()][lksq.index()] as i32;
            // KBNK (Phase 3.11): the generic corner-drive cannot win K+B+N vs K
            // because it pushes the bare king to the nearest corner, not the
            // bishop-coloured one. For that exact material pattern, drive the
            // losing king to a corner matching the winning bishop's colour
            // instead; keep the generic drive for every other won ending.
            // 4.9a.4. The finer drive applies ONLY when the losing side is a
            // bare king at the point of evaluation. KXK/KBNK match directly,
            // but a pawn-root family can enter this material shape after an
            // under-promotion and exchange; its real scope is the dispatcher's
            // promotion closure (4.11.9), never root material alone. The enclosing
            // gate is `|approximate| > 200`, which is "up two pawns" and fires
            // in plenty of middlegames; scaling the drive there moved `bench 13`
            // by +7.9% (7,226,051 -> 7,800,345) purely by perturbing positions
            // this term was never meant to steer. Everything else keeps the old
            // mild edge push, avoiding a broad middlegame blast radius.
            // Scoped to MINOR-PIECE mates, following Basilisk BAS-E34. Their
            // first version drove every bare-king mate, queen and rook
            // included, and cost +20.5% bench nodes; restricting it to
            // minor-piece mates returned bench to baseline byte for byte. The
            // reason generalises: a recogniser is worth adding only where
            // SEARCH CANNOT ALREADY SOLVE THE CLASS, and overriding a class it
            // solves only churns aspiration windows wherever a deep line
            // touches a won ending. Their KQ-K/KR-K were already 100/100.
            let minor_mate = board.color_occ(losing) == Bitboard::from(lksq)
                && !board.pieces(winning, Piece::Pawn).any()
                && !board.pieces(winning, Piece::Rook).any()
                && !board.pieces(winning, Piece::Queen).any();
            let mopup = if !minor_mate {
                let lfile = infra::to_i32(SQUARE_FILE[lksq.index()]);
                let lrank = infra::to_i32(SQUARE_RANK[lksq.index()]);
                let file_push = (3 - lfile).max(lfile - 4);
                let rank_push = (3 - lrank).max(lrank - 4);
                sign * (5 * (file_push + rank_push) + (14 - king_distance) * 4)
            } else {
                // Both branches below drive to a CORNER through the same
                // Chebyshev+Manhattan metric. Chebyshev alone is flat -- it scores
                // every square in a ring identically -- and the measured cost was
                // total: over 300 won KBNK positions, 94% had a TIED best move and
                // the whole term spanned 8 cp across all 19 legal moves. The same
                // metric serves KXK and KBNK because they share that one defect,
                // which is why one change fixes both.
                let corners = match kbnk_winner_bishop(board, winning) {
                    Some(true) => KBNK_LIGHT_CORNERS,
                    Some(false) => KBNK_DARK_CORNERS,
                    // Generic KXK: drive toward whichever corner is already
                    // nearest. A corner is on the edge, so this subsumes the old
                    // edge push while giving the search an ordering to follow.
                    None => {
                        if KING_DISTANCE[lksq.index()][KBNK_LIGHT_CORNERS[0]]
                            .min(KING_DISTANCE[lksq.index()][KBNK_LIGHT_CORNERS[1]])
                            <= KING_DISTANCE[lksq.index()][KBNK_DARK_CORNERS[0]]
                                .min(KING_DISTANCE[lksq.index()][KBNK_DARK_CORNERS[1]])
                        {
                            KBNK_LIGHT_CORNERS
                        } else {
                            KBNK_DARK_CORNERS
                        }
                    }
                };
                // Pick the corner by Chebyshev first, Manhattan as the tie-break,
                // so the target itself does not flip between equally distant
                // corners and undo the gradient we just created.
                let target = if (
                    KING_DISTANCE[lksq.index()][corners[0]],
                    MANHATTAN_DISTANCE[lksq.index()][corners[0]],
                ) <= (
                    KING_DISTANCE[lksq.index()][corners[1]],
                    MANHATTAN_DISTANCE[lksq.index()][corners[1]],
                ) {
                    corners[0]
                } else {
                    corners[1]
                };
                let king_man = i32::from(MANHATTAN_DISTANCE[wksq.index()][lksq.index()]);
                // DIAGONAL pull. `|7 - rank - file|` is 0 on the a8-h1
                // anti-diagonal and rises to 7 at the a1/h8 corners, so it is
                // plateau-free by construction and describes the actual
                // technique: walk the king DOWN A DIAGONAL rather than at a
                // corner. Mirroring the file serves the dark corner pair with
                // one formula. Structure and scale both come from the reference
                // HCE, whose corner term outweighs its king term by roughly 24
                // to 1 -- and that RATIO is the whole mechanism. An earlier
                // sweep tried this shape at roughly 1:1 and measured it WORSE
                // than the Chebyshev version it replaced (33-50% against
                // 57.1%), which is why it was wrongly rejected once already.
                let lfile = infra::to_i32(SQUARE_FILE[lksq.index()]);
                let lrank = infra::to_i32(SQUARE_RANK[lksq.index()]);
                let diag_file =
                    if target == KBNK_LIGHT_CORNERS[0] || target == KBNK_LIGHT_CORNERS[1] {
                        lfile
                    } else {
                        7 - lfile
                    };
                let diagonal = (7 - lrank - diag_file).abs();
                sign * (MOPUP_DIAGONAL * diagonal
                    + MOPUP_KING_CHEB * (7 - king_distance)
                    + MOPUP_KING_MAN * (14 - king_man))
            };
            *eg += mopup;
            // Frozen mate-drive term — not a tunable weight; goes into `rest`.
            #[cfg(feature = "texel")]
            {
                self.trace.borrow_mut().frozen_eg += mopup;
            }
        }
    }
}

/// True iff `winner` holds exactly king + one bishop + one knight while the
/// loser has a bare king and neither side has pawns/rooks/queens (the KBNK
/// mate). Returns whether the winning bishop is light-squared.
fn kbnk_winner_bishop(board: &Board, winner: Color) -> Option<bool> {
    let loser = !winner;
    if board.color_occ(loser) != Bitboard::from(board.king_sq(loser)) {
        return None;
    }
    if board.pieces(winner, Piece::Pawn).any()
        || board.pieces(winner, Piece::Rook).any()
        || board.pieces(winner, Piece::Queen).any()
    {
        return None;
    }
    let bishops = board.pieces(winner, Piece::Bishop);
    let knights = board.pieces(winner, Piece::Knight);
    if bishops.count() == 1 && knights.count() == 1 {
        Some((bishops & Bitboard::LIGHT_SQUARES).any())
    } else {
        None
    }
}
