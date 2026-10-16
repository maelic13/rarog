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
pub(super) const MG_VAL: [i32; 6] = [103, 392, 420, 539, 1140, 0];
pub(super) const EG_VAL: [i32; 6] = [138, 261, 317, 557, 952, 0];
pub(super) const PHASE_W: [i32; 6] = [0, 1, 1, 2, 4, 0];
const MG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -26, -21, -11, 7, 7, 23, 23, -25, -44, -40, -23, -18, -7, -3, -1, -40,
    -33, -36, -21, -16, -5, 1, -23, -35, -17, -20, -22, -11, 11, 3, -3, -17, 1, 12, 37, 27, 75, 64,
    94, 38, 221, 254, 180, 216, 192, 228, 139, -29, 0, 0, 0, 0, 0, 0, 0, 0,
];
const EG_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 22, 12, -2, -32, 9, 3, -4, -8, 21, 13, -7, -7, -4, -2, -5, -6, 33, 28,
    -5, -13, -16, -2, 10, 4, 51, 27, 0, -22, -13, -9, 10, 15, 80, 59, 28, 11, -11, 6, 13, 44, 87,
    64, 92, 36, 23, 33, 72, 95, 0, 0, 0, 0, 0, 0, 0, 0,
];
const MG_KNIGHT_PST: [i32; 64] = [
    -142, -88, -99, -18, -10, -11, -10, -56, -45, -102, -17, 8, 6, -6, -37, 7, -67, 11, 19, 37, 51,
    34, 37, -26, 8, 33, 65, 66, 76, 88, 103, 42, 8, 10, 46, 77, 65, 93, 79, 77, -101, -16, 42, 63,
    120, 109, 112, -35, -26, 20, 62, 95, 80, 48, 39, 16, -184, -43, -62, 35, 24, -17, -6, 19,
];
const EG_KNIGHT_PST: [i32; 64] = [
    -98, 17, -6, -31, -21, -59, -41, -40, -73, 3, -25, -24, -28, -13, 5, -73, 17, -38, -22, -21,
    -29, -49, -44, -7, -6, -17, -6, -6, -8, -31, -36, -24, 18, -16, -8, -6, -9, -32, -44, -4, 23,
    -17, -6, -14, -28, -33, -57, 13, 21, 3, -24, -35, -36, -26, -7, 10, -102, -54, 1, -14, -8, -49,
    11, -47,
];
const MG_BISHOP_PST: [i32; 64] = [
    -65, -24, -3, -53, 3, -11, -32, -67, 40, 6, 6, -3, 3, 19, 23, -43, 2, 43, 22, 28, 14, 32, 25,
    14, 29, 17, 56, 40, 53, 39, 31, 19, 29, 26, 19, 62, 41, 52, 23, -11, -14, 9, 22, 33, 66, 68,
    26, 15, 10, 36, 6, -26, -60, 40, -35, -80, -6, 4, -45, -1, -60, -36, 40, -79,
];
const EG_BISHOP_PST: [i32; 64] = [
    1, -10, -16, -13, -22, -24, 0, -50, -10, 2, -7, -5, -20, -22, -37, -10, -1, -13, 4, -16, 5,
    -32, -8, -21, 2, 18, -12, 6, -16, -6, -14, -5, 7, 8, 11, -8, 5, -1, 1, 3, 17, 24, 10, 11, -11,
    3, 19, 23, 29, 11, 15, 9, 38, 12, 31, 10, 60, 24, 20, -3, 23, 14, -4, 19,
];
const MG_ROOK_PST: [i32; 64] = [
    -17, -8, -6, 18, 24, 23, 22, -22, -36, -20, -18, 1, 13, 25, 35, -3, -69, -62, -33, -3, -1, 6,
    -13, -27, -82, -4, -22, -5, 14, 24, 65, -52, -57, 18, 6, 62, 41, 97, 48, 4, -24, 24, -1, 45,
    72, 127, 88, 50, -21, 12, 50, 66, 82, 108, 119, 110, -32, 15, 55, 45, 25, 53, 58, 82,
];
const EG_ROOK_PST: [i32; 64] = [
    -28, -35, -18, -36, -46, -45, -40, -50, -13, -11, -3, -11, -30, -27, -30, -29, 20, 12, 5, -12,
    -6, -9, 10, 11, 45, 6, 12, 4, -8, -11, -7, 18, 41, -1, 1, -24, -20, -36, -15, 2, 27, 5, 10,
    -21, -27, -37, -17, -22, 33, 20, -6, -9, -8, -15, -17, -15, 53, 39, 19, 12, 24, 17, 17, 21,
];
const MG_QUEEN_PST: [i32; 64] = [
    40, -4, 0, 1, 25, 6, -24, 2, -5, -12, 2, 15, 24, 44, -2, 52, 5, 7, -2, -4, 17, 49, 41, 3, 0, 0,
    6, 33, 44, 41, 45, 29, -23, -48, -44, -18, 39, 7, 51, -13, -3, -7, -22, -43, 34, 63, 67, 44,
    -72, -65, -42, -22, -25, 16, 5, 11, -27, -51, -24, 0, 22, 51, 33, 28,
];
const EG_QUEEN_PST: [i32; 64] = [
    -37, -1, -15, -10, -13, -28, -25, -32, 10, 45, 44, 23, 20, -28, 16, -23, -38, 22, 74, 83, 45,
    18, 44, 55, 51, 64, 82, 65, 52, 57, 85, 64, -39, 16, -2, -7, -20, 17, -35, 12, -87, -60, -25,
    15, 22, -15, -48, -48, -37, -31, 13, 12, 48, 25, 1, -17, -38, 2, 1, 10, -4, 19, 3, 6,
];
const MG_KING_PST: [i32; 64] = [
    -36, 51, 11, -122, -57, -74, 30, 34, 29, 19, -52, -103, -70, -33, 33, 34, -21, -42, -26, -39,
    -72, 14, 54, -8, -60, 35, -93, -143, -135, -54, -16, -44, 24, 19, -116, -144, -126, -42, -6,
    -34, 19, 61, 48, -59, -66, 43, 61, 21, 83, 51, 42, 49, 20, 63, 24, 20, -14, 75, 74, 40, -9, 31,
    66, 32,
];
const EG_KING_PST: [i32; 64] = [
    -14, -39, -6, 19, 1, 3, -43, -80, -6, -3, 21, 31, 20, 15, -12, -30, 12, 12, 25, 31, 38, 15, -8,
    -2, 21, 10, 48, 69, 63, 45, 25, 15, 19, 49, 71, 74, 74, 68, 61, 32, 36, 56, 55, 59, 65, 74, 76,
    57, 43, 19, 54, 39, 40, 80, 93, 48, -67, 17, 26, 18, 28, 64, 52, -3,
];

/// Flatten the six per-piece PST consts into one `[i32; 384]` array in
/// `Piece::ALL` order (Pawn,Knight,Bishop,Rook,Queen,King), white POV — the
/// uniform `[i32; N]` shape every `EvalParams` field uses (Phase 3.1).
pub(super) const fn build_default_pst(mg: bool) -> [i32; 384] {
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
    let mut piece = 0;
    while piece < 6 {
        let mut sq = 0;
        while sq < 64 {
            out[piece * 64 + sq] = tables[piece][sq];
            sq += 1;
        }
        piece += 1;
    }
    out
}

impl Evaluator {
    /// SF-style quadratic material imbalance (Phase 3.9). White-POV net value,
    /// added phase-independently to mg and eg. Imbalance "pieces" are indexed
    /// `[0]=bishop pair, [1]=pawn, [2]=knight, [3]=bishop, [4]=rook, [5]=queen`;
    /// the coefficient matrices use the lower triangle (`pt2 <= pt1`). All
    /// coefficients are seeded 0, so this contributes nothing today.
    pub(super) fn eval_imbalance(&mut self, board: &Board, mg: &mut i32, eg: &mut i32) {
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

        // The imbalance is a function of the piece counts alone, and a search
        // meets few distinct counts, so it is cached by them. The texel build
        // traces it, so it computes every time.
        #[cfg(not(feature = "texel"))]
        let imb = {
            let material = material_signature(&cnt);
            let slot = imbalance_slot(material);
            match self.imbalance_table[slot] {
                Some(entry) if entry.material == material => entry.value,
                _ => {
                    let value = self.imbalance_of(&cnt);
                    self.imbalance_table[slot] = Some(ImbalanceEntry { material, value });
                    value
                }
            }
        };
        #[cfg(feature = "texel")]
        let imb = self.imbalance_of(&cnt);
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

    /// The imbalance of the counts `cnt`, White's point of view.
    #[inline(always)]
    fn imbalance_of(&self, cnt: &[[i32; 6]; 2]) -> i32 {
        // Branch-free: a piece type a side lacks multiplies its row by zero,
        // and with compiled-in coefficients the loops unroll into a fixed sum.
        let mut imb = 0i32; // white − black
        for c in [Color::White, Color::Black] {
            let sign = color_sign(c);
            let us = cnt[c as usize];
            let them = cnt[!c as usize];
            for pt1 in 0..6 {
                let mut row = 0;
                for pt2 in 0..=pt1 {
                    let k = pt1 * 6 + pt2;
                    row += self.params.imbalance_ours[k] * us[pt2]
                        + self.params.imbalance_theirs[k] * them[pt2];
                }
                imb += sign * us[pt1] * row;
            }
        }
        imb
    }
}

/// Slots in the imbalance cache.
pub(super) const IMBALANCE_TABLE_SIZE: usize = 1024;

/// One cached imbalance and the piece counts it is for. The texel build
/// writes no entry, so there the fields look dead.
#[cfg_attr(feature = "texel", allow(dead_code))]
#[derive(Copy, Clone)]
pub(super) struct ImbalanceEntry {
    material: u64,
    value: i32,
}

/// Both sides' pawn, knight, bishop, rook and queen counts, four bits each
/// (no count exceeds ten), which determine the imbalance.
#[cfg_attr(feature = "texel", allow(dead_code))]
fn material_signature(cnt: &[[i32; 6]; 2]) -> u64 {
    cnt.iter()
        .flat_map(|side| side[1..].iter())
        .fold(0u64, |signature, &n| {
            (signature << 4) | u64::from(n.unsigned_abs())
        })
}

#[cfg_attr(feature = "texel", allow(dead_code))]
fn imbalance_slot(material: u64) -> usize {
    infra::index(material.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 54) & (IMBALANCE_TABLE_SIZE - 1)
}
