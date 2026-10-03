//! Static-evaluation correction: the tables, the corrected-eval formula that
//! reads them and the update that trains them.

use crate::board::{Board, Color, Piece};
use crate::eval::piece_value;
use crate::infra;

use super::history::{CONT_SIZE, apply_bonus, piece_to};
use super::{Searcher, TB_WIN_SCORE};

/// Key slots per side to move in a keyed correction table.
const KEY_SLOTS: usize = 65_536;
/// Fifty-move buckets: the clock below 16 plies shares bucket 0, then one
/// bucket per 8 plies up to bucket 15.
const RULE50_BUCKETS: usize = 16;
const KEYED_SIZE: usize = RULE50_BUCKETS * 2 * KEY_SLOTS;
const KEYED_MAX: i32 = 14_605;
const CONT_MAX: i32 = 16_418;
/// Table units per evaluation unit: a correction is read as `sum / 64`.
const TABLE_SCALE: i64 = 64;
/// Weights are in 128ths.
const WEIGHT_SCALE: i64 = 128;
/// Material of the starting position in evaluation units, the point at which
/// the material term of the corrected eval is neutral.
const MATERIAL_REFERENCE: i64 = 8_000;

fn saturate(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(if value < 0 { i32::MIN } else { i32::MAX })
}

fn zeroed<const N: usize>() -> Box<[i16; N]> {
    vec![0i16; N]
        .into_boxed_slice()
        .try_into()
        .unwrap_or_else(|_| unreachable!("length is N by construction"))
}

#[inline(always)]
fn rule50_bucket(board: &Board) -> usize {
    (usize::from(board.halfmove_clock()).saturating_sub(8) / 8).min(RULE50_BUCKETS - 1)
}

#[inline(always)]
fn keyed_index(bucket: usize, stm: Color, key: u64) -> usize {
    (bucket * 2 + stm as usize) * KEY_SLOTS + (infra::index(key) & (KEY_SLOTS - 1))
}

/// One thread's correction tables.
pub(super) struct CorrectionTables {
    /// `[rule-50 bucket][side to move][pawn key]`.
    pawn: Box<[i16; KEYED_SIZE]>,
    /// `[rule-50 bucket][side to move][minor-piece key]`.
    minor: Box<[i16; KEYED_SIZE]>,
    /// `[colour][rule-50 bucket][side to move][that colour's non-pawn key]`.
    non_pawn: [Box<[i16; KEYED_SIZE]>; 2],
    /// Continuation corrections: `[context two or four plies back][previous
    /// move's coloured piece and destination]`.
    continuation_2: Box<[i16; CONT_SIZE]>,
    continuation_4: Box<[i16; CONT_SIZE]>,
}

impl Default for CorrectionTables {
    fn default() -> Self {
        Self {
            pawn: zeroed(),
            minor: zeroed(),
            non_pawn: [zeroed(), zeroed()],
            continuation_2: zeroed(),
            continuation_4: zeroed(),
        }
    }
}

impl CorrectionTables {
    /// Every entry of every table is zero: nothing has been trained.
    #[cfg(test)]
    pub(super) fn untouched(&self) -> bool {
        self.pawn.iter().all(|&v| v == 0)
            && self.minor.iter().all(|&v| v == 0)
            && self
                .non_pawn
                .iter()
                .all(|table| table.iter().all(|&v| v == 0))
            && self.continuation_2.iter().all(|&v| v == 0)
            && self.continuation_4.iter().all(|&v| v == 0)
    }

    /// Forget everything, for a new game.
    pub(super) fn clear(&mut self) {
        self.pawn.fill(0);
        self.minor.fill(0);
        for table in &mut self.non_pawn {
            table.fill(0);
        }
        self.continuation_2.fill(0);
        self.continuation_4.fill(0);
    }

    /// Between searches the tables keep their values.
    #[expect(
        clippy::unused_self,
        reason = "the search calls age() on either arm's tables"
    )]
    pub(super) fn age(&mut self) {}
}

/// The slots one position reads and trains.
struct CorrectionSlots {
    pawn: usize,
    minor: usize,
    non_pawn: [usize; 2],
    /// `(context, slot)` at two and four plies back, when both moves exist.
    continuation: [Option<usize>; 2],
}

impl Searcher {
    fn correction_slots(&self, board: &Board, ply: usize) -> CorrectionSlots {
        let stm = board.side_to_move();
        let bucket = rule50_bucket(board);
        let previous = self.td.stack.back(ply, 1);
        let mut continuation = [None; 2];
        if !previous.mv.is_null() {
            let slot = piece_to(!stm, previous.piece, previous.mv.to_sq());
            for (index, back) in [2, 4].into_iter().enumerate() {
                continuation[index] = self.cont_context_back(ply, back).map(|ctx| ctx + slot);
            }
        }
        CorrectionSlots {
            pawn: keyed_index(bucket, stm, board.pawn_key()),
            minor: keyed_index(bucket, stm, board.minor_key()),
            non_pawn: [
                keyed_index(bucket, stm, board.non_pawn_key(Color::White)),
                keyed_index(bucket, stm, board.non_pawn_key(Color::Black)),
            ],
            continuation,
        }
    }

    /// The correction for the position at `ply`, in evaluation units: the
    /// weighted sum of the six tables.
    pub(super) fn correction_value(&self, board: &Board, ply: usize) -> i32 {
        let slots = self.correction_slots(board, ply);
        let tables = &self.td.corr;
        let p = &self.cfg.core;
        let read = |table: &[i16], slot: usize| i64::from(table[slot]);
        let mut sum = i64::from(p.corr_weight_pawn) * read(&tables.pawn[..], slots.pawn)
            + i64::from(p.corr_weight_minor) * read(&tables.minor[..], slots.minor)
            + i64::from(p.corr_weight_non_pawn_white)
                * read(&tables.non_pawn[0][..], slots.non_pawn[0])
            + i64::from(p.corr_weight_non_pawn_black)
                * read(&tables.non_pawn[1][..], slots.non_pawn[1]);
        if let Some(slot) = slots.continuation[0] {
            sum += i64::from(p.corr_weight_cont2) * read(&tables.continuation_2[..], slot);
        }
        if let Some(slot) = slots.continuation[1] {
            sum += i64::from(p.corr_weight_cont4) * read(&tables.continuation_4[..], slot);
        }
        saturate(sum / (TABLE_SCALE * WEIGHT_SCALE))
    }

    pub(super) fn raw_eval(&mut self, board: &Board) -> i32 {
        self.td.evaluator.evaluate(board)
    }

    pub(super) fn corrected_eval(&mut self, board: &Board, ply: usize) -> i32 {
        let raw = self.raw_eval(board);
        self.corrected_eval_from_raw(board, raw, ply)
    }

    pub(super) fn corrected_eval_from_raw(&self, board: &Board, raw: i32, ply: usize) -> i32 {
        self.corrected_eval_parts(board, raw, ply).0
    }

    /// The corrected eval and the correction it includes. The raw eval is
    /// scaled by the material on the board (a coordinate neutral at zero) and
    /// damped toward the fifty-move horizon with the current clock, then
    /// corrected and kept out of the tablebase-win band. The damping lives
    /// here, not in the evaluator, because the raw eval is stored in the
    /// transposition table without the clock.
    pub(super) fn corrected_eval_parts(&self, board: &Board, raw: i32, ply: usize) -> (i32, i32) {
        let p = &self.cfg.core;
        let mut eval = i64::from(raw);
        if p.eval_material_scale != 0 {
            let material = i64::from(material(board));
            eval += eval * i64::from(p.eval_material_scale) * (material - MATERIAL_REFERENCE)
                / (64 * MATERIAL_REFERENCE);
        }
        if p.eval_rule50_damping != 0 {
            let clock = i64::from(board.halfmove_clock().min(100));
            eval -= eval * i64::from(p.eval_rule50_damping) * clock / (100 * 199);
        }
        let correction = self.correction_value(board, ply);
        let eval = (eval + i64::from(correction))
            .clamp(i64::from(-TB_WIN_SCORE + 1), i64::from(TB_WIN_SCORE - 1));
        (saturate(eval), correction)
    }

    /// Train every correction table for the node at `ply` from the residual
    /// `diff` = searched score minus static eval. The caller owns the
    /// admission rule: not in check, a quiet best move, a bound that agrees
    /// with the residual's sign.
    pub(super) fn train_correction(&mut self, board: &Board, depth: i32, diff: i32, ply: usize) {
        crate::diag_count!(correction_updates);
        #[cfg(feature = "diag")]
        {
            let magnitude = u64::from(diff.unsigned_abs());
            crate::diag_count!(corr_resid_quiet_n);
            crate::diag_add!(corr_resid_quiet_sum, magnitude);
            match board.halfmove_clock() {
                0..=19 => {
                    crate::diag_count!(corr_resid_hm_low_n);
                    crate::diag_add!(corr_resid_hm_low_sum, magnitude);
                }
                20..=49 => {
                    crate::diag_count!(corr_resid_hm_mid_n);
                    crate::diag_add!(corr_resid_hm_mid_sum, magnitude);
                }
                _ => {
                    crate::diag_count!(corr_resid_hm_high_n);
                    crate::diag_add!(corr_resid_hm_high_sum, magnitude);
                }
            }
        }
        let p = &self.cfg.core;
        let bonus =
            (p.corr_update_slope * depth * diff / 128).clamp(p.corr_update_min, p.corr_update_max);
        let slots = self.correction_slots(board, ply);
        let tables = &mut self.td.corr;
        #[cfg(feature = "diag")]
        if crate::diag::sampled(board.hash(), ply, crate::diag::SAMPLE_CORRECTION) {
            crate::diag_count!(corr_sample_updates);
            crate::diag_add!(corr_sample_abs_sum, u64::from(diff.unsigned_abs()));
            crate::diag::record_correction_slot(
                0,
                slots.pawn,
                board.pawn_key(),
                tables.pawn[slots.pawn],
            );
            crate::diag::record_correction_slot(
                1,
                slots.minor,
                board.minor_key(),
                tables.minor[slots.minor],
            );
            for (source, index, color) in [(2, 0, Color::White), (3, 1, Color::Black)] {
                crate::diag::record_correction_slot(
                    source,
                    slots.non_pawn[index],
                    board.non_pawn_key(color),
                    tables.non_pawn[index][slots.non_pawn[index]],
                );
            }
        }
        apply_bonus(&mut tables.pawn[slots.pawn], bonus, KEYED_MAX);
        apply_bonus(&mut tables.minor[slots.minor], bonus, KEYED_MAX);
        apply_bonus(&mut tables.non_pawn[0][slots.non_pawn[0]], bonus, KEYED_MAX);
        apply_bonus(&mut tables.non_pawn[1][slots.non_pawn[1]], bonus, KEYED_MAX);
        if let Some(slot) = slots.continuation[0] {
            crate::diag_count!(corr_cont2_admitted);
            #[cfg(feature = "diag")]
            match depth {
                ..=1 => crate::diag_count!(corr_cont2_admitted_d1),
                2 => crate::diag_count!(corr_cont2_admitted_d2),
                3 => crate::diag_count!(corr_cont2_admitted_d3),
                4..=6 => crate::diag_count!(corr_cont2_admitted_d4_6),
                _ => crate::diag_count!(corr_cont2_admitted_d7_plus),
            }
            apply_bonus(&mut tables.continuation_2[slot], bonus, CONT_MAX);
        }
        if let Some(slot) = slots.continuation[1] {
            crate::diag_count!(corr_cont4_admitted);
            #[cfg(feature = "diag")]
            match depth {
                ..=1 => crate::diag_count!(corr_cont4_admitted_d1),
                2 => crate::diag_count!(corr_cont4_admitted_d2),
                3 => crate::diag_count!(corr_cont4_admitted_d3),
                4..=6 => crate::diag_count!(corr_cont4_admitted_d4_6),
                _ => crate::diag_count!(corr_cont4_admitted_d7_plus),
            }
            apply_bonus(&mut tables.continuation_4[slot], bonus, CONT_MAX);
        }
    }
}

/// Non-king material of both sides, in evaluation units.
fn material(board: &Board) -> i32 {
    let mut total = 0;
    for color in [Color::White, Color::Black] {
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
        ] {
            total +=
                infra::to_i32(board.pieces(color, piece).count() as usize) * piece_value(piece);
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Move;

    const FEN: &str = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";

    /// The table stores the raw eval under a key that ignores the halfmove
    /// clock, so the raw eval must not depend on the clock: a value stored at
    /// clock 0 and read at clock 80 corrects to exactly what a fresh
    /// evaluation at clock 80 does, and that is damped toward the draw.
    #[test]
    fn a_stored_raw_eval_serves_any_halfmove_clock() {
        let mut searcher = Searcher::default();
        let early = Board::from_fen("4k3/8/8/8/8/8/3Q4/4K3 w - - 0 60").expect("valid FEN");
        let late = Board::from_fen("4k3/8/8/8/8/8/3Q4/4K3 w - - 80 60").expect("valid FEN");
        assert_eq!(early.hash(), late.hash(), "the table key ignores the clock");
        let raw = searcher.raw_eval(&early);
        searcher.shared.tt.store_eval(early.hash(), raw, false);
        let stored = crate::tt::TtProbe::from_entry(
            searcher.shared.tt.probe(late.hash()),
            0,
            late.halfmove_clock(),
        )
        .raw_static_eval;
        let fresh = searcher.raw_eval(&late);
        assert_eq!(stored, fresh);
        let (late_eval, _) = searcher.corrected_eval_parts(&late, stored, 0);
        assert_eq!(late_eval, searcher.corrected_eval_parts(&late, fresh, 0).0);
        let (early_eval, _) = searcher.corrected_eval_parts(&early, raw, 0);
        assert!(
            late_eval.abs() < early_eval.abs(),
            "{late_eval} vs {early_eval}"
        );
    }

    #[test]
    fn rule50_buckets_cover_the_clock() {
        let bucket = |clock: u8| {
            let fen = format!("4k3/8/8/8/8/8/8/4K3 w - - {clock} 60");
            rule50_bucket(&Board::from_fen(&fen).expect("valid FEN"))
        };
        assert_eq!(bucket(0), 0);
        assert_eq!(bucket(15), 0);
        assert_eq!(bucket(16), 1);
        assert_eq!(bucket(23), 1);
        assert_eq!(bucket(100), 11);
        assert_eq!(bucket(255), RULE50_BUCKETS - 1);
    }

    /// A positive residual trained at a position raises the correction read
    /// back there, and a negative one lowers it; the corrected eval moves by
    /// the correction and nothing else at the neutral formula coordinates.
    /// The update is gravity toward the table maximum, not toward the
    /// residual: a residual that keeps its sign keeps pushing, and only the
    /// table bounds cap the correction.
    #[test]
    fn training_moves_the_correction_toward_the_residual() {
        let mut searcher = Searcher::default();
        let board = Board::from_fen(FEN).expect("valid FEN");
        let raw = 37;
        assert_eq!(searcher.correction_value(&board, 0), 0);
        assert_eq!(searcher.corrected_eval_parts(&board, raw, 0), (raw, 0));
        for _ in 0..20 {
            searcher.train_correction(&board, 8, 60, 0);
        }
        let correction = searcher.correction_value(&board, 0);
        let p = &searcher.cfg.core;
        let bound = (KEYED_MAX
            * (p.corr_weight_pawn
                + p.corr_weight_minor
                + p.corr_weight_non_pawn_white
                + p.corr_weight_non_pawn_black))
            / (64 * 128);
        assert!(
            correction > 0 && correction <= bound,
            "{correction} vs {bound}"
        );
        assert_eq!(
            searcher.corrected_eval_parts(&board, raw, 0),
            (raw + correction, correction)
        );
        for _ in 0..200 {
            searcher.train_correction(&board, 20, -400, 0);
        }
        assert!(searcher.correction_value(&board, 0) < 0);
    }

    /// Continuation corrections are keyed by the earlier context and the
    /// previous move, and read only when both moves exist.
    #[test]
    fn continuation_corrections_need_both_moves() {
        let mut searcher = Searcher::default();
        let board = Board::from_fen(FEN).expect("valid FEN");
        let slots = searcher.correction_slots(&board, 4);
        assert_eq!(slots.continuation, [None, None]);
        let mv = Move::from_uci("e7e5").expect("valid move");
        searcher.td.stack[3].mv = mv;
        searcher.td.stack[3].piece = Piece::Pawn;
        searcher.td.stack[2].mv = Move::from_uci("g1f3").expect("valid move");
        searcher.td.stack[2].cont_key = 1234 * super::super::history::PIECE_TO_SIZE;
        let slots = searcher.correction_slots(&board, 4);
        assert!(slots.continuation[0].is_some());
        assert_eq!(slots.continuation[1], None, "no move four plies back");
    }

    #[test]
    fn corrected_eval_stays_out_of_the_tablebase_band() {
        let mut searcher = Searcher::default();
        let board = Board::from_fen(FEN).expect("valid FEN");
        for _ in 0..500 {
            searcher.train_correction(&board, 60, 30_000, 0);
        }
        let (eval, _) = searcher.corrected_eval_parts(&board, TB_WIN_SCORE, 0);
        assert!(eval < TB_WIN_SCORE);
    }

    #[test]
    fn rule50_damping_is_the_evaluators_and_material_is_neutral_at_zero() {
        let mut searcher = Searcher::default();
        let board = Board::from_fen("4k3/8/8/8/8/8/3Q4/4K3 w - - 90 60").expect("valid FEN");
        // The material scale is an SPSA coordinate and is fitted away from
        // zero, so the damping line is read with it switched off; the term's
        // own effect is asserted below.
        searcher.cfg.core.eval_material_scale = 0;
        assert_eq!(
            searcher.corrected_eval_parts(&board, 500, 0).0,
            500 - 500 * 90 / 199,
            "the default damping is the evaluator's former rule-50 line"
        );
        searcher.cfg.core.eval_rule50_damping = 0;
        assert_eq!(searcher.corrected_eval_parts(&board, 500, 0).0, 500);
        searcher.cfg.core.eval_material_scale = 64;
        let expected = 500 + 500 * (900 - 8_000) / 8_000;
        assert_eq!(searcher.corrected_eval_parts(&board, 500, 0).0, expected);
    }
}
