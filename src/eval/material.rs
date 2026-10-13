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
pub(super) const MG_VAL: [i32; 6] = [98, 392, 420, 538, 1136, 0];
pub(super) const EG_VAL: [i32; 6] = [135, 255, 312, 539, 944, 0];
pub(super) const PHASE_W: [i32; 6] = [0, 1, 1, 2, 4, 0];
const MG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -27, -23, -11, 8, 9, 26, 25, -30, -49, -45, -25, -18, -6, -3, -1, -39,
    -34, -40, -20, -14, -4, 0, -27, -39, -19, -18, -20, -9, 11, 5, -6, -17, -1, 16, 41, 26, 75, 70,
    98, 34, 221, 256, 180, 216, 192, 228, 139, -29, 0, 0, 0, 0, 0, 0, 0, 0,
];
const EG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 25, 11, -2, -32, 9, -1, -9, -9, 27, 13, -6, -8, -4, -1, -9, -6, 35, 29,
    -5, -11, -16, -2, 11, 4, 56, 28, -1, -25, -13, -9, 8, 14, 81, 62, 28, 8, -14, 6, 13, 41, 89,
    65, 92, 36, 23, 33, 72, 95, 0, 0, 0, 0, 0, 0, 0, 0,
];
const MG_KNIGHT_PST: [i32; 64] = [
    -142, -99, -103, -18, -10, -11, -5, -56, -45, -104, -17, 10, 11, -6, -41, 9, -71, 13, 18, 38,
    58, 35, 39, -30, 12, 33, 64, 65, 78, 88, 108, 44, 11, 5, 44, 74, 62, 93, 83, 81, -101, -20, 38,
    62, 120, 109, 114, -35, -26, 20, 62, 95, 80, 48, 39, 16, -184, -43, -62, 35, 24, -17, -6, 19,
];
const EG_KNIGHT_PST: [i32; 64] = [
    -98, 17, -6, -31, -21, -59, -41, -40, -73, 3, -25, -23, -28, -13, 5, -73, 17, -38, -22, -21,
    -29, -49, -44, -7, -6, -17, -6, -6, -11, -33, -36, -24, 21, -16, -8, -8, -9, -34, -44, -4, 23,
    -18, -6, -14, -28, -33, -57, 13, 21, 3, -24, -35, -36, -26, -7, 10, -102, -54, 1, -14, -8, -49,
    11, -47,
];
const MG_BISHOP_PST: [i32; 64] = [
    -65, -29, -3, -57, 3, -14, -32, -68, 44, 6, 1, -5, 1, 20, 26, -47, 2, 43, 24, 26, 17, 35, 23,
    11, 29, 17, 60, 45, 55, 39, 31, 19, 34, 27, 19, 66, 43, 52, 21, -11, -14, 9, 22, 33, 66, 68,
    26, 14, 14, 36, 6, -26, -60, 40, -35, -80, -6, 4, -45, -1, -60, -36, 40, -79,
];
const EG_BISHOP_PST: [i32; 64] = [
    1, -10, -16, -14, -22, -26, 0, -50, -10, 3, -7, -5, -21, -22, -37, -10, -1, -13, 4, -16, 5,
    -35, -6, -21, 2, 18, -12, 8, -16, -6, -14, -5, 8, 8, 11, -8, 5, -1, 1, 3, 17, 25, 10, 11, -11,
    3, 19, 23, 29, 11, 15, 9, 38, 12, 31, 10, 60, 24, 20, -3, 23, 14, -4, 19,
];
const MG_ROOK_PST: [i32; 64] = [
    -17, -7, -8, 16, 22, 25, 24, -22, -36, -19, -20, -1, 15, 24, 35, -1, -71, -67, -36, -3, -6, 4,
    -17, -30, -84, -4, -22, -5, 12, 24, 65, -54, -57, 18, 6, 62, 40, 98, 48, 4, -24, 24, -5, 45,
    72, 127, 88, 50, -21, 14, 54, 66, 83, 108, 119, 110, -32, 15, 55, 45, 25, 53, 58, 82,
];
const EG_ROOK_PST: [i32; 64] = [
    -28, -35, -20, -34, -48, -47, -38, -50, -13, -11, -3, -11, -30, -26, -30, -29, 20, 12, 5, -12,
    -6, -9, 10, 11, 45, 6, 12, 4, -10, -11, -7, 18, 41, -1, -1, -24, -20, -36, -15, 2, 27, 5, 7,
    -21, -27, -37, -17, -22, 33, 20, -6, -9, -8, -15, -17, -15, 53, 39, 19, 12, 24, 17, 17, 21,
];
const MG_QUEEN_PST: [i32; 64] = [
    40, -4, 0, 3, 25, 6, -24, 2, -4, -9, 0, 16, 24, 47, -4, 52, 5, 6, -6, -6, 15, 46, 39, 0, 4, -2,
    4, 34, 42, 39, 45, 28, -21, -51, -48, -18, 39, 7, 56, -15, -1, -7, -22, -46, 34, 63, 67, 46,
    -72, -69, -42, -22, -25, 16, 5, 11, -27, -51, -24, 0, 22, 51, 33, 28,
];
const EG_QUEEN_PST: [i32; 64] = [
    -37, -1, -15, -10, -13, -28, -25, -32, 10, 45, 44, 23, 20, -28, 16, -23, -38, 22, 74, 83, 45,
    18, 44, 55, 51, 64, 82, 65, 51, 56, 85, 64, -39, 16, -3, -7, -20, 17, -35, 12, -87, -60, -25,
    15, 22, -15, -48, -48, -37, -31, 13, 12, 48, 25, 1, -17, -38, 2, 1, 10, -4, 19, 3, 6,
];
const MG_KING_PST: [i32; 64] = [
    -36, 51, 9, -122, -47, -74, 32, 32, 29, 17, -53, -106, -70, -33, 34, 36, -21, -42, -26, -39,
    -72, 14, 54, -8, -60, 35, -93, -143, -135, -54, -16, -44, 24, 19, -116, -144, -126, -42, -6,
    -34, 19, 61, 48, -59, -66, 43, 61, 21, 83, 51, 42, 49, 20, 63, 24, 20, -14, 75, 74, 40, -9, 31,
    66, 32,
];
const EG_KING_PST: [i32; 64] = [
    -14, -39, -6, 21, 5, 3, -44, -77, -6, -3, 21, 34, 22, 15, -14, -29, 12, 12, 25, 31, 38, 14,
    -11, -2, 21, 10, 48, 69, 63, 45, 25, 15, 19, 49, 71, 74, 73, 68, 61, 32, 36, 56, 55, 58, 65,
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
