//! Static-evaluation correction history: the tables' consumer and trainer.

use crate::board::Board;
use crate::infra;

use super::Searcher;
use super::history::{HISTORY_MAX, update_hist_entry};

pub(super) const CORR_SIZE: usize = 65_536;

impl Searcher {
    pub(super) fn corrected_eval(&mut self, board: &Board, ply: usize) -> i32 {
        let raw = self.raw_eval(board);
        self.corrected_eval_from_raw(board, raw, ply)
    }

    pub(super) fn raw_eval(&mut self, board: &Board) -> i32 {
        self.evaluator.evaluate(board)
    }

    pub(super) fn corrected_eval_from_raw(&self, board: &Board, raw: i32, ply: usize) -> i32 {
        raw + self.correction_value(board, ply)
    }

    pub(super) fn correction_value(&self, board: &Board, ply: usize) -> i32 {
        let color = board.side_to_move();
        let us = color as usize;
        let them = (!color) as usize;
        let pawn =
            self.td.correction_history[us][infra::index(board.pawn_key()) & (CORR_SIZE - 1)] as i32;
        let minor = self.td.minor_correction_history[us]
            [infra::index(board.minor_key()) & (CORR_SIZE - 1)] as i32;
        let own_non_pawn = self.td.non_pawn_correction_history[us][us]
            [infra::index(board.non_pawn_key(color)) & (CORR_SIZE - 1)]
            as i32;
        let their_non_pawn = self.td.non_pawn_correction_history[us][them]
            [infra::index(board.non_pawn_key(!color)) & (CORR_SIZE - 1)]
            as i32;
        let previous = self.td.stack.back(ply, 1);
        let continuation = if previous.mv.is_null() {
            0
        } else {
            self.td.continuation_correction_history[previous.cont_key] as i32
        };
        // 8.5(c): per-source weights (seed 128 = the old unit weight; the
        // continuation term keeps its inherent `/2`). `Σ src·W / 16384`
        // reproduces the old `(pawn+minor+own_np+their_np+cont/2)/128` bit-for-
        // bit at seed, since `Σsrc·128/16384 == Σsrc/128` in integer division.
        (pawn * self.params.corr_w_pawn
            + minor * self.params.corr_w_minor
            + own_non_pawn * self.params.corr_w_own_np
            + their_non_pawn * self.params.corr_w_their_np
            + (continuation / 2) * self.params.corr_w_cont)
            / 16384
    }

    /// 4.5: weight a correction residual by what produced it.
    ///
    /// At the seeded `CorrCaptureWeightPct = 100` this returns `diff` unchanged,
    /// so the default is exactly inert. Below 100 a capture-caused residual is
    /// down-weighted rather than discarded — the graded alternative to
    /// `corr_guard_capture`, whose binary exclusion RAR-S16 measured at −55.98
    /// Elo because it threw away 51.3% of all training.
    ///
    /// Also records the residual magnitude per attribution class, which is the
    /// measurement that decides whether down-weighting is justified at all: if
    /// capture-caused residuals are no noisier than quiet ones, the premise
    /// behind both this knob and `corr_guard_capture` is wrong.
    #[inline(always)]
    pub(super) fn attributed_residual(&self, diff: i32, from_capture: bool, halfmove: u8) -> i32 {
        #[cfg(feature = "diag")]
        {
            let magnitude = u64::from(diff.unsigned_abs());
            if from_capture {
                crate::diag_count!(corr_resid_capture_n);
                crate::diag_add!(corr_resid_capture_sum, magnitude);
            } else {
                crate::diag_count!(corr_resid_quiet_n);
                crate::diag_add!(corr_resid_quiet_sum, magnitude);
            }
            // 4.5d: halfmove-clock context. PLAN 4.5 permits a new correction
            // context only where held-out UNIQUE signal is shown, so measure the
            // residual per bucket before proposing one. Rule-50 proximity is the
            // plausible mechanism: near the horizon a position's value stops
            // being a function of its structure at all.
            match halfmove {
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
        // The clock is a diagnostic input only; production reads it nowhere, so
        // discard it explicitly rather than renaming the parameter to `_halfmove`
        // and losing the name at both call sites.
        #[cfg(not(feature = "diag"))]
        {
            let _ = halfmove;
        }
        if from_capture && self.params.corr_capture_weight_pct != 100 {
            diff * self.params.corr_capture_weight_pct / 100
        } else {
            diff
        }
    }

    pub(super) fn update_correction(&mut self, board: &Board, diff: i32, depth: i32, ply: usize) {
        let color = board.side_to_move();
        let us = color as usize;
        let them = (!color) as usize;
        let scaled = (diff * depth.max(1)).clamp(-1024, 1024);
        #[cfg(feature = "diag")]
        if crate::diag::sampled(board.hash, ply, crate::diag::SAMPLE_CORRECTION) {
            crate::diag_count!(corr_sample_updates);
            crate::diag_add!(corr_sample_abs_sum, u64::from(diff.unsigned_abs()));
            let pawn_key = board.pawn_key();
            let minor_key = board.minor_key();
            let own_key = board.non_pawn_key(color);
            let other_key = board.non_pawn_key(!color);
            let pawn_index = infra::index(pawn_key) & (CORR_SIZE - 1);
            let minor_index = infra::index(minor_key) & (CORR_SIZE - 1);
            let own_index = infra::index(own_key) & (CORR_SIZE - 1);
            let other_index = infra::index(other_key) & (CORR_SIZE - 1);
            crate::diag::record_correction_slot(
                0,
                us * CORR_SIZE + pawn_index,
                pawn_key,
                self.td.correction_history[us][pawn_index],
            );
            crate::diag::record_correction_slot(
                1,
                us * CORR_SIZE + minor_index,
                minor_key,
                self.td.minor_correction_history[us][minor_index],
            );
            crate::diag::record_correction_slot(
                2,
                us * 2 * CORR_SIZE + us * CORR_SIZE + own_index,
                own_key,
                self.td.non_pawn_correction_history[us][us][own_index],
            );
            crate::diag::record_correction_slot(
                3,
                us * 2 * CORR_SIZE + them * CORR_SIZE + other_index,
                other_key,
                self.td.non_pawn_correction_history[us][them][other_index],
            );
        }
        update_hist_entry(
            &mut self.td.correction_history[us][infra::index(board.pawn_key()) & (CORR_SIZE - 1)],
            scaled,
            HISTORY_MAX,
        );
        update_hist_entry(
            &mut self.td.minor_correction_history[us]
                [infra::index(board.minor_key()) & (CORR_SIZE - 1)],
            scaled,
            HISTORY_MAX,
        );
        update_hist_entry(
            &mut self.td.non_pawn_correction_history[us][us]
                [infra::index(board.non_pawn_key(color)) & (CORR_SIZE - 1)],
            scaled,
            HISTORY_MAX,
        );
        update_hist_entry(
            &mut self.td.non_pawn_correction_history[us][them]
                [infra::index(board.non_pawn_key(!color)) & (CORR_SIZE - 1)],
            scaled,
            HISTORY_MAX,
        );
        let previous = *self.td.stack.back(ply, 1);
        if !previous.mv.is_null() {
            update_hist_entry(
                &mut self.td.continuation_correction_history[previous.cont_key],
                scaled / 2,
                HISTORY_MAX,
            );
        }
    }
}
