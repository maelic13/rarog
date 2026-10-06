//! Material values, phase weights, the piece-square tables and the quadratic
//! material imbalance.

#[cfg(feature = "texel")]
use super::trace::{tr_eg, tr_mg};
use super::{Evaluator, color_sign};
use crate::board::{Board, Color, Piece};
use crate::infra;

// Material values (Phase 4.6 fitted). King pinned 0. mg values rescaled up ~×1.1
// vs the old PeSTO seeds to match the lower fitted K (1.70) — ratios-to-pawn are
// essentially unchanged, so this is a benign scale shift, not a distortion.
pub(super) const MG_VAL: [i32; 6] = [88, 394, 418, 537, 1131, 0];
pub(super) const EG_VAL: [i32; 6] = [123, 239, 290, 486, 930, 0];
pub(super) const PHASE_W: [i32; 6] = [0, 1, 1, 2, 4, 0];
const MG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -37, -25, -13, 16, -5, 47, 47, -39, -55, -42, -25, -12, -2, -7, 0, -35,
    -47, -48, -17, -17, 3, -5, -41, -44, -28, -18, -14, -13, 6, 9, -9, -17, -1, 21, 49, 32, 79, 75,
    102, 32, 221, 256, 180, 216, 192, 228, 139, -29, 0, 0, 0, 0, 0, 0, 0, 0,
];
const EG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 27, 16, 5, -32, 5, -2, -15, -9, 28, 16, -3, -10, -1, 1, -12, -5, 41,
    29, -5, -10, -15, -5, 12, 6, 58, 28, -2, -27, -17, -12, 7, 13, 83, 62, 30, 8, -16, 4, 9, 35,
    85, 64, 92, 36, 23, 28, 72, 89, 0, 0, 0, 0, 0, 0, 0, 0,
];
const MG_KNIGHT_PST: [i32; 64] = [
    -142, -130, -110, -21, -17, -16, 11, -56, -45, -106, -17, 12, 12, -7, -46, 26, -79, 18, 18, 38,
    72, 43, 43, -32, 20, 31, 63, 57, 77, 88, 114, 53, 11, -6, 44, 72, 54, 99, 97, 87, -101, -26,
    38, 62, 120, 109, 119, -35, -26, 20, 62, 95, 80, 48, 39, 16, -184, -43, -62, 35, 24, -17, -6,
    19,
];
const EG_KNIGHT_PST: [i32; 64] = [
    -98, 17, -6, -31, -21, -59, -40, -40, -73, 3, -24, -23, -28, -13, 5, -73, 17, -37, -20, -21,
    -29, -48, -44, -7, -5, -17, -6, -6, -14, -33, -36, -24, 21, -18, -8, -13, -9, -34, -44, -4, 23,
    -18, -6, -14, -28, -33, -57, 13, 21, 3, -24, -35, -36, -26, -7, 10, -102, -54, 1, -14, -8, -49,
    11, -47,
];
const MG_BISHOP_PST: [i32; 64] = [
    -65, -37, -4, -65, 3, -24, -32, -68, 52, 0, -7, -5, 0, 20, 21, -54, 2, 41, 26, 27, 17, 45, 16,
    11, 34, 13, 74, 53, 62, 38, 31, 15, 42, 27, 19, 73, 47, 52, 21, -11, -14, 11, 22, 33, 66, 68,
    26, 7, 14, 36, 6, -26, -60, 40, -35, -80, -6, 4, -45, -1, -60, -36, 40, -79,
];
const EG_BISHOP_PST: [i32; 64] = [
    1, -10, -16, -17, -22, -28, 0, -50, -10, 4, -7, -5, -21, -22, -40, -10, -1, -15, 4, -16, 5,
    -35, -6, -21, 2, 18, -12, 8, -16, -6, -14, -5, 13, 8, 11, -8, 5, -1, 1, 3, 17, 25, 10, 11, -11,
    3, 19, 23, 29, 11, 15, 9, 38, 12, 31, 10, 60, 24, 20, -3, 23, 14, -4, 19,
];
const MG_ROOK_PST: [i32; 64] = [
    -17, -5, -9, 5, 19, 26, 23, -21, -35, -12, -18, -3, 18, 24, 35, -1, -78, -75, -42, -10, -14, 1,
    -25, -35, -86, -4, -22, -11, 12, 24, 65, -62, -64, 23, 6, 65, 34, 98, 48, 4, -24, 24, -9, 45,
    72, 127, 88, 50, -17, 14, 61, 66, 83, 108, 119, 110, -32, 15, 55, 45, 25, 53, 58, 82,
];
const EG_ROOK_PST: [i32; 64] = [
    -28, -36, -19, -31, -48, -47, -32, -50, -13, -11, -3, -11, -30, -26, -30, -29, 20, 12, 5, -12,
    -6, -9, 10, 11, 45, 6, 12, 4, -10, -11, -7, 18, 39, -1, -2, -24, -23, -36, -15, 2, 26, 0, 1,
    -21, -27, -37, -17, -22, 33, 20, -6, -9, -8, -15, -17, -15, 53, 39, 19, 12, 24, 17, 17, 21,
];
const MG_QUEEN_PST: [i32; 64] = [
    40, 0, 4, 14, 25, 6, -24, 2, -3, 1, -4, 6, 21, 51, -5, 52, 5, 7, -14, -16, 11, 54, 39, -8, 12,
    -2, -2, 42, 41, 33, 40, 20, -19, -55, -54, -19, 44, 7, 58, -16, 2, -7, -24, -52, 34, 63, 67,
    51, -72, -69, -42, -22, -25, 16, 5, 11, -27, -51, -24, 0, 22, 51, 33, 28,
];
const EG_QUEEN_PST: [i32; 64] = [
    -37, -1, -15, -10, -13, -28, -25, -32, 10, 48, 47, 23, 20, -28, 16, -23, -38, 22, 74, 83, 45,
    18, 44, 55, 51, 64, 82, 65, 51, 56, 85, 64, -39, 16, -3, -7, -20, 17, -35, 12, -87, -60, -25,
    15, 22, -15, -48, -48, -37, -31, 13, 12, 48, 25, 1, -17, -38, 2, 1, 10, -4, 19, 3, 6,
];
const MG_KING_PST: [i32; 64] = [
    -36, 47, -2, -122, -34, -80, 31, 43, 29, 17, -53, -106, -70, -33, 38, 36, -21, -42, -26, -39,
    -72, 14, 56, -8, -60, 35, -93, -143, -135, -54, -16, -44, 24, 19, -116, -144, -126, -42, -6,
    -34, 19, 61, 48, -59, -66, 43, 61, 21, 83, 51, 42, 49, 20, 63, 24, 20, -14, 75, 74, 40, -9, 31,
    66, 32,
];
const EG_KING_PST: [i32; 64] = [
    -14, -39, -6, 23, 11, -3, -47, -68, -6, -3, 21, 35, 22, 11, -13, -21, 12, 12, 25, 32, 38, 11,
    -15, -1, 21, 10, 48, 69, 63, 45, 23, 16, 19, 49, 67, 70, 71, 68, 61, 32, 36, 56, 55, 50, 63,
    74, 76, 57, 43, 19, 54, 39, 40, 80, 93, 48, -67, 17, 26, 18, 28, 64, 52, -3,
];

/// Flatten the six per-piece PST consts into one `[i32; 384]` array in
/// `Piece::ALL` order (Pawn,Knight,Bishop,Rook,Queen,King), white POV — the
/// uniform `[i32; N]` shape every `EvalParams` field uses (Phase 3.1).
pub(super) fn build_default_pst(mg: bool) -> [i32; 384] {
    let tables: [&[i32; 64]; 6] = if mg {
        [
            &MG_PAWN_PST,
            &MG_KNIGHT_PST,
            &MG_BISHOP_PST,
            &MG_ROOK_PST,
            &MG_QUEEN_PST,
            &MG_KING_PST,
        ]
    } else {
        [
            &EG_PAWN_PST,
            &EG_KNIGHT_PST,
            &EG_BISHOP_PST,
            &EG_ROOK_PST,
            &EG_QUEEN_PST,
            &EG_KING_PST,
        ]
    };
    let mut out = [0i32; 384];
    for (piece, table) in tables.iter().enumerate() {
        out[piece * 64..piece * 64 + 64].copy_from_slice(table.as_slice());
    }
    out
}

impl Evaluator {
    /// SF-style quadratic material imbalance (Phase 3.9). White-POV net value,
    /// added phase-independently to mg and eg. Imbalance "pieces" are indexed
    /// `[0]=bishop pair, [1]=pawn, [2]=knight, [3]=bishop, [4]=rook, [5]=queen`;
    /// the coefficient matrices use the lower triangle (`pt2 <= pt1`). All
    /// coefficients are seeded 0, so this contributes nothing today.
    pub(super) fn eval_imbalance(&self, board: &Board, mg: &mut i32, eg: &mut i32) {
        let count = |c: Color| -> [i32; 6] {
            let bishops = infra::to_i32(board.pieces(c, Piece::Bishop).count());
            [
                (bishops >= 2) as i32,
                infra::to_i32(board.pieces(c, Piece::Pawn).count()),
                infra::to_i32(board.pieces(c, Piece::Knight).count()),
                bishops,
                infra::to_i32(board.pieces(c, Piece::Rook).count()),
                infra::to_i32(board.pieces(c, Piece::Queen).count()),
            ]
        };
        let cnt = [count(Color::White), count(Color::Black)];

        let mut imb = 0i32; // white − black
        for c in [Color::White, Color::Black] {
            let sign = color_sign(c);
            let us = cnt[c as usize];
            let them = cnt[!c as usize];
            for pt1 in 0..6 {
                if us[pt1] == 0 {
                    continue;
                }
                for pt2 in 0..=pt1 {
                    let k = pt1 * 6 + pt2;
                    imb += sign
                        * us[pt1]
                        * (self.params.imbalance_ours[k] * us[pt2]
                            + self.params.imbalance_theirs[k] * them[pt2]);
                }
            }
        }
        *mg += imb;
        *eg += imb;

        // Trace each coefficient's net (white − black) count product, in both
        // mg and eg (phase-independent weight). `imbalance_ours[pt1][pt2]`
        // multiplies count[pt1]*count[pt2]; `imbalance_theirs[pt1][pt2]`
        // multiplies our[pt1]*their[pt2].
        #[cfg(feature = "texel")]
        {
            let w = &cnt[Color::White as usize];
            let b = &cnt[Color::Black as usize];
            for pt1 in 0..6 {
                for pt2 in 0..=pt1 {
                    let k = pt1 * 6 + pt2;
                    let ours = w[pt1] * w[pt2] - b[pt1] * b[pt2];
                    let theirs = w[pt1] * b[pt2] - b[pt1] * w[pt2];
                    tr_mg!(self, imbalance_ours, k, ours);
                    tr_eg!(self, imbalance_ours, k, ours);
                    tr_mg!(self, imbalance_theirs, k, theirs);
                    tr_eg!(self, imbalance_theirs, k, theirs);
                }
            }
        }
    }
}
