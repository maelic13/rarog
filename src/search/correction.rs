//! Static-evaluation correction: the tables, the corrected-eval formula that
//! reads them and the update that trains them.

use crate::board::{Board, Color};
use crate::infra;

use super::Searcher;
use super::history::{CONT_SIZE, apply_bonus, piece_to};

/// Key slots per side to move in a keyed correction table.
const KEY_SLOTS: usize = 65_536;
/// Fifty-move buckets: the clock below 16 plies shares bucket 0, then one
/// bucket per 8 plies up to bucket 15.
const RULE50_BUCKETS: usize = 16;
const KEYED_SIZE: usize = RULE50_BUCKETS * 2 * KEY_SLOTS;
/// Table units per evaluation unit: a correction is read as `sum / 64`.
const TABLE_SCALE: i64 = 64;

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
    /// `[colour][rule-50 bucket][side to move][that colour's non-pawn key]`.
    non_pawn: [Box<[i16; KEYED_SIZE]>; 2],
    /// Continuation corrections: `[context two or four plies back][previous
    /// move's coloured piece and destination]`, one table for both.
    continuation_2: Box<[i16; CONT_SIZE]>,
}

impl Default for CorrectionTables {
    fn default() -> Self {
        Self {
            pawn: zeroed(),
            non_pawn: [zeroed(), zeroed()],
            continuation_2: zeroed(),
        }
    }
}

impl CorrectionTables {
    /// Forget everything, for a new game.
    pub(super) fn clear(&mut self) {
        self.pawn.fill(0);
        for table in &mut self.non_pawn {
            table.fill(0);
        }
        self.continuation_2.fill(0);
    }
}

/// The slots one position reads and trains.
struct CorrectionSlots {
    pawn: usize,
    non_pawn: [usize; 2],
    /// `(context, slot)` at two and four plies back, when both moves exist.
    continuation: [Option<usize>; 2],
}

impl Searcher {
    /// The compact kernel's slots: its non-pawn keys include each side's king.
    fn compact_correction_slots(&self, board: &Board, ply: usize) -> CorrectionSlots {
        let non_pawn = [Color::White, Color::Black].map(|color| board.non_pawn_key(color));
        self.correction_slots_keyed(board, ply, non_pawn)
    }

    fn correction_slots_keyed(
        &self,
        board: &Board,
        ply: usize,
        non_pawn: [u64; 2],
    ) -> CorrectionSlots {
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
            non_pawn: non_pawn.map(|key| keyed_index(bucket, stm, key)),
            continuation,
        }
    }

    /// The compact kernel's correction for the position at `ply`: the plain
    /// sum of the pawn, both non-pawn and two continuation
    /// readings, over the table scale. Its non-pawn keys include the kings,
    /// its continuation readings, two and four plies back, share one table,
    /// and it reads no minor table.
    pub(super) fn compact_correction(&self, board: &Board, ply: usize) -> i32 {
        let slots = self.compact_correction_slots(board, ply);
        let tables = &self.td.corr;
        let mut sum = i64::from(tables.pawn[slots.pawn])
            + i64::from(tables.non_pawn[0][slots.non_pawn[0]])
            + i64::from(tables.non_pawn[1][slots.non_pawn[1]]);
        for slot in slots.continuation.into_iter().flatten() {
            sum += i64::from(tables.continuation_2[slot]);
        }
        saturate(sum / TABLE_SCALE)
    }

    /// Train the compact kernel's tables for the node at `ply` by `bonus`,
    /// already scaled and clamped: the pawn and both non-pawn tables, and
    /// the one continuation table at the contexts two and four plies back.
    pub(super) fn compact_train_correction(&mut self, board: &Board, ply: usize, bonus: i32) {
        // A pricing probe leaves the corrections as the measured search had them.
        #[cfg(feature = "pricing")]
        if self.td.pricing.probing {
            return;
        }
        let slots = self.compact_correction_slots(board, ply);
        let seeds = self.seeds();
        let (keyed_max, cont_max) = (seeds.corr_keyed_max, seeds.corr_cont_max);
        let tables = &mut self.td.corr;
        apply_bonus(&mut tables.pawn[slots.pawn], bonus, keyed_max);
        apply_bonus(&mut tables.non_pawn[0][slots.non_pawn[0]], bonus, keyed_max);
        apply_bonus(&mut tables.non_pawn[1][slots.non_pawn[1]], bonus, keyed_max);
        for slot in slots.continuation.into_iter().flatten() {
            apply_bonus(&mut tables.continuation_2[slot], bonus, cont_max);
            #[cfg(feature = "probes")]
            self.td
                .rates
                .record(super::compact::probes::Kind::Gravity, true);
        }
    }

    pub(super) fn raw_eval(&mut self, board: &Board) -> i32 {
        self.td.evaluator.evaluate(board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Move, Piece};
    use crate::tt::TB_WIN_SCORE;

    const FEN: &str = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";

    /// The table stores the raw eval under a key that ignores the halfmove
    /// clock, so the raw eval must not depend on the clock: a value stored at
    /// clock 0 and read at clock 80 evaluates to exactly what a fresh
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
        let material = 1_242;
        let (late_eval, _) = searcher.compact_eval(&late, stored, material, 0);
        assert_eq!(
            late_eval,
            searcher.compact_eval(&late, fresh, material, 0).0
        );
        let (early_eval, _) = searcher.compact_eval(&early, raw, material, 0);
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

    /// A positive bonus trained at a position raises the correction read
    /// back there and a negative one lowers it, and the corrected eval moves
    /// by the correction.
    #[test]
    fn training_moves_the_correction_toward_the_residual() {
        let mut searcher = Searcher::default();
        let board = Board::from_fen(FEN).expect("valid FEN");
        let (raw, material) = (37, 8_000);
        assert_eq!(searcher.compact_correction(&board, 0), 0);
        let (plain, _) = searcher.compact_eval(&board, raw, material, 0);
        for _ in 0..20 {
            searcher.compact_train_correction(&board, 0, 600);
        }
        let correction = searcher.compact_correction(&board, 0);
        assert!(correction > 0, "{correction}");
        assert_eq!(
            searcher.compact_eval(&board, raw, material, 0),
            (plain + correction, correction)
        );
        for _ in 0..200 {
            searcher.compact_train_correction(&board, 0, -4_000);
        }
        assert!(searcher.compact_correction(&board, 0) < 0);
    }

    /// Continuation corrections are keyed by the earlier context and the
    /// previous move, and read only when both moves exist.
    #[test]
    fn continuation_corrections_need_both_moves() {
        let mut searcher = Searcher::default();
        let board = Board::from_fen(FEN).expect("valid FEN");
        let slots = searcher.compact_correction_slots(&board, 4);
        assert_eq!(slots.continuation, [None, None]);
        let mv = Move::from_uci("e7e5").expect("valid move");
        searcher.td.stack[3].mv = mv;
        searcher.td.stack[3].piece = Piece::Pawn;
        searcher.td.stack[2].mv = Move::from_uci("g1f3").expect("valid move");
        searcher.td.stack[2].cont_key = 1234 * super::super::history::PIECE_TO_SIZE;
        let slots = searcher.compact_correction_slots(&board, 4);
        assert!(slots.continuation[0].is_some());
        assert_eq!(slots.continuation[1], None, "no move four plies back");
    }

    /// A draw verdict is the evaluator's classification: tables trained on its
    /// own keys leave it at exactly 0, while the same tables move an ordinary
    /// raw 0, so the guard is the verdict and not the value.
    #[test]
    fn a_draw_verdict_takes_no_correction() {
        let mut searcher = Searcher::default();
        // The wrong-bishop rook-pawn draw: a light bishop cannot cover h8.
        let verdict = Board::from_fen("7k/8/6KP/8/8/8/8/5B2 w - - 0 1").expect("valid FEN");
        assert!(crate::eval::endgame::is_draw_verdict(&verdict));
        assert_eq!(searcher.raw_eval(&verdict), 0, "the recogniser's verdict");
        for _ in 0..50 {
            searcher.compact_train_correction(&verdict, 0, 600);
        }
        assert_ne!(
            searcher.compact_correction(&verdict, 0),
            0,
            "the tables are trained"
        );
        assert_eq!(searcher.compact_eval(&verdict, 0, 545, 0), (0, 0));

        let ordinary = Board::from_fen(FEN).expect("valid FEN");
        assert!(!crate::eval::endgame::is_draw_verdict(&ordinary));
        for _ in 0..50 {
            searcher.compact_train_correction(&ordinary, 0, 600);
        }
        let correction = searcher.compact_correction(&ordinary, 0);
        assert_ne!(correction, 0);
        assert_eq!(searcher.compact_eval(&ordinary, 0, 8_000, 0).1, correction);
    }

    #[test]
    fn corrected_eval_stays_out_of_the_tablebase_band() {
        let mut searcher = Searcher::default();
        let board = Board::from_fen(FEN).expect("valid FEN");
        for _ in 0..500 {
            searcher.compact_train_correction(&board, 0, 30_000);
        }
        let raw = crate::search::compact::bounded_raw(TB_WIN_SCORE);
        let (eval, _) = searcher.compact_eval(&board, raw, 8_000, 0);
        assert!(eval < TB_WIN_SCORE);
    }
}
