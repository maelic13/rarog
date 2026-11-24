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
mod tests;
