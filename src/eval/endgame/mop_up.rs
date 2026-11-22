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
/// Chebyshev distance alone is FLAT: every square in a ring around the target
/// scores identically, so over 300 won KBNK positions 94% had a tied best move.
/// A term that cannot order its own moves cannot steer a search, whatever its
/// magnitude. Manhattan breaks the rings.
const MANHATTAN_DISTANCE: [[u8; 64]; 64] = init_manhattan_distance();
/// Mate-drive weights. Chebyshev carries the coarse pull and Manhattan
/// supplies the resolution that breaks its rings. The diagonal weight sets the
/// ratio of the corner pull to the king pull, which must dominate, not merely
/// exceed it: swept on KBN-K conversion (100 positions, 60k nodes), Chebyshev
/// alone converted 19.4%, with Manhattan 57.1%, a diagonal pull at about 1:1
/// against the king terms 56.1%, and 96.9% at 360 and at 480 (720 gave 94.9%);
/// the smaller of the tied peak is taken.
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
pub(crate) const MOPUP_ASSUMED_MAX_PLY: i32 = 240;

/// A guidance term must never reach the band the search reads as a forced mate.
///
/// This is a `const` assertion rather than a debug check or a validator inside
/// an option-setter, and that is the point: the SHIPPED default is what plays
/// the games, so the bound has to hold in every build type including release,
/// not only where a tuning build happens to compile a setter.
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

// KBNK mate: the bare king is driven to a corner the winning
// bishop can actually reach — i.e. a corner of the bishop's own square colour.
// A bishop can only reach corners that share its colour, so these sets are
// exactly the corner squares contained in `Bitboard::DARK_SQUARES` /
// `LIGHT_SQUARES`.
const KBNK_DARK_CORNERS: [usize; 2] = [0, 63]; // a1, h8
const KBNK_LIGHT_CORNERS: [usize; 2] = [7, 56]; // h1, a8

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
    /// king toward the edge/corner (the bishop's corner for KBNK). It also runs
    /// on the lazy-eval path, so mating technique survives a lazy skip. Frozen
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
            // The corner drive applies ONLY to MINOR-PIECE mates: a losing bare
            // king against a winner with no pawn, rook or queen, read from the
            // position evaluated (a pawn-root family can reach this material
            // after an under-promotion). KBNK needs the bishop's corner, which
            // the generic nearest-corner drive cannot find. Every other won
            // ending keeps the mild edge push: the gate `|approximate| > 200`
            // is "up two pawns" and fires in plenty of middlegames, where the
            // corner drive perturbed positions it was never meant to steer
            // (bench nodes +7.9%), and the search already solves queen and rook
            // mates, so a drive there only churns aspiration windows.
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
                // Chebyshev+Manhattan metric (see `MANHATTAN_DISTANCE`).
                let corners = match kbnk_winner_bishop(board, winning) {
                    Some(true) => KBNK_DARK_CORNERS,
                    Some(false) => KBNK_LIGHT_CORNERS,
                    // Generic KXK: drive toward whichever corner is already
                    // nearest. A corner is on the edge, so this subsumes the edge
                    // push while giving the search an ordering to follow.
                    None => {
                        if KING_DISTANCE[lksq.index()][KBNK_DARK_CORNERS[0]]
                            .min(KING_DISTANCE[lksq.index()][KBNK_DARK_CORNERS[1]])
                            <= KING_DISTANCE[lksq.index()][KBNK_LIGHT_CORNERS[0]]
                                .min(KING_DISTANCE[lksq.index()][KBNK_LIGHT_CORNERS[1]])
                        {
                            KBNK_DARK_CORNERS
                        } else {
                            KBNK_LIGHT_CORNERS
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
                // corner. Mirroring the file serves the h1/a8 pair with
                // one formula. Structure and scale come from the reference HCE,
                // whose corner term outweighs its king term by roughly 24 to 1;
                // that ratio is the mechanism, and at about 1:1 the pull
                // converted worse than Chebyshev+Manhattan alone.
                let lfile = infra::to_i32(SQUARE_FILE[lksq.index()]);
                let lrank = infra::to_i32(SQUARE_RANK[lksq.index()]);
                let diag_file = if target == KBNK_DARK_CORNERS[0] || target == KBNK_DARK_CORNERS[1]
                {
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
/// mate). Returns whether the winning bishop is dark-squared.
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
        Some((bishops & Bitboard::DARK_SQUARES).any())
    } else {
        None
    }
}
