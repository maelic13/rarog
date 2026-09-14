//! Move-ordering histories: the tables' index math, their update rules and
//! ageing.

use crate::board::{Board, CheckInfo, Color, Move, Piece};
use crate::infra;

use super::Searcher;
use super::movepick::BadCaptureList;

pub const HISTORY_MAX: i32 = 16_384;
pub const CAP_HISTORY_MAX: i32 = 16_384;
pub const CONT_SIZE: usize = 6 * 64 * 6 * 64;
pub const LOW_PLY_HISTORY_SIZE: usize = 8;
pub const PAWN_HISTORY_SIZE: usize = 4_096;
pub const PIECE_TO_SIZE: usize = 6 * 64;

/// Continuation-history look-back distances and their bonus divisors.
///
/// 9.0a: replaces four parallel `cont_history_N` fields and four copy-pasted
/// blocks in each of the read / update / age paths (twelve near-identical
/// stanzas). `(plies_back, bonus_divisor)` — slot order is the array order in
/// [`Searcher::cont_history`], so adding a look-back distance is one entry
/// here rather than a field plus three new blocks.
pub(super) const CONT_PLY_BACK: [(usize, i32); 4] = [(1, 1), (2, 1), (4, 2), (6, 3)];
pub(super) const CONT_TABLES: usize = CONT_PLY_BACK.len();

/// Node-invariant half of quiet-history scoring, resolved once per node by
/// [`Searcher::quiet_history_ctx`] (8.12(g2)): the continuation rows that
/// apply at this ply (`None` = guard failed — too shallow or null previous
/// move) and the pawn-history row for this pawn structure. Per move, scoring
/// adds only `piece_to_index(piece, to)` to each base.
pub(super) struct QuietHistoryCtx {
    cont_bases: [Option<usize>; CONT_TABLES],
    pawn_base: usize,
}
/// Heap-allocate the continuation tables without a ~1.1 MB stack temporary
/// (`Box::new([[0; CONT_SIZE]; N])` would materialize the array on the stack
/// first). Startup-only.
pub(super) fn boxed_cont_tables() -> Box<[[i16; CONT_SIZE]; CONT_TABLES]> {
    let tables: Box<[[i16; CONT_SIZE]]> = vec![[0; CONT_SIZE]; CONT_TABLES].into_boxed_slice();
    tables
        .try_into()
        .unwrap_or_else(|_| unreachable!("length is CONT_TABLES by construction"))
}

pub(crate) fn update_hist_entry(entry: &mut i16, bonus: i32, max_value: i32) {
    let current = *entry as i32;
    let updated = current + bonus - current * bonus.abs() / max_value;
    *entry = crate::infra::saturating_i16(updated);
}

/// Node-invariant prefix of [`pawn_history_index`]: the pawn-key row base,
/// same per-node hoist as [`cont_row_base`].
pub(crate) fn pawn_row_base(pawn_key: u64) -> usize {
    (infra::index(pawn_key) & (PAWN_HISTORY_SIZE - 1)) * PIECE_TO_SIZE
}

/// Flat `(piece, square)` index. Same reasoning as [`cont_index`].
pub(crate) fn piece_to_index(piece: usize, to: usize) -> usize {
    debug_assert!(piece < 6, "piece index out of range");
    debug_assert!(to < 64, "square index out of range");
    (piece * 64 + to).min(PIECE_TO_SIZE - 1)
}

pub(crate) fn pawn_history_index(pawn_key: u64, piece: usize, to: usize) -> usize {
    let slot = infra::index(pawn_key) & (PAWN_HISTORY_SIZE - 1);
    slot * PIECE_TO_SIZE + piece_to_index(piece, to)
}

impl Searcher {
    /// Resolve the node-invariant half of quiet-history indexing once per
    /// node (8.12(g2), from the Basilisk cross-review — its 8.7.6(b+d) hoist,
    /// +3.03% NPS there). The continuation guards (`ply < back`, null
    /// previous move), the previous piece/square loads, and the pawn-key row
    /// lookup do not depend on the move being scored, yet `cont_score` used
    /// to redo all of them for every quiet in the list. 8.12(g) refuted the
    /// PREFETCH angle for these tables (all quiets share one row window per
    /// node) but never isolated the duplicated arithmetic; this removes it.
    /// Only `piece_to_index(piece, to)` remains per-move.
    pub(super) fn quiet_history_ctx(&self, board: &Board, ply: usize) -> QuietHistoryCtx {
        let mut cont_bases = [None; CONT_TABLES];
        for (slot, &(back, _)) in CONT_PLY_BACK.iter().enumerate() {
            if ply < back {
                continue;
            }
            let prev = self.stack[ply - back].mv;
            if prev.is_null() {
                continue;
            }
            cont_bases[slot] = Some(self.stack[ply - back].cont_row_base());
        }
        QuietHistoryCtx {
            cont_bases,
            pawn_base: pawn_row_base(board.pawn_key()),
        }
    }

    pub(super) fn quiet_history_score(
        &self,
        board: &Board,
        check_info: &CheckInfo,
        ctx: &QuietHistoryCtx,
        color: Color,
        mv: Move,
        ply: usize,
    ) -> i32 {
        let from = mv.from_sq().index();
        let to = mv.to_sq().index();
        let main = self.main_history[color as usize][from][to] as i32;
        let piece = board.moving_piece(mv) as usize;
        // The shared per-move offset into every (piece, to)-shaped row.
        let piece_to = piece_to_index(piece, to);
        let pawn = self.pawn_history[ctx.pawn_base + piece_to] as i32;
        let low_ply = if ply < LOW_PLY_HISTORY_SIZE {
            self.low_ply_history[ply][from][to] as i32 / (1 + infra::to_i32(ply))
        } else {
            0
        };
        let mut cont = 0;
        for (slot, base) in ctx.cont_bases.iter().enumerate() {
            if let Some(base) = base {
                cont += self.cont_history[slot][(base + piece_to).min(CONT_SIZE - 1)] as i32;
            }
        }
        // 4.6c: safe versus losing check classes. A check whose checker can be
        // taken at a material loss is usually refuted by taking it, so it does
        // not deserve the same enormous bonus as a safe one. When the two
        // bonuses are equal (the seeded state) the SEE probe is SKIPPED, so
        // ordering pays nothing for a distinction it is not making.
        let direct_check = if board.gives_check_with(mv, check_info) {
            // 4.6c: a safe/losing split was measured non-functional (RAR-S44:
            // `see_ge(mv, 0)` is trivially true for a non-capture) and reverted.
            self.params.check_bonus_safe
        } else {
            0
        };
        2 * main + pawn + low_ply + cont + direct_check
    }

    /// Reward for the move that produced a beta cutoff (Phase 8.1: linear
    /// SF-shaped formula, split from the malus so SPSA can tune them apart).
    pub(super) fn history_bonus(&self, depth: i32) -> i32 {
        (self.params.hist_bonus_mul * depth - self.params.hist_bonus_sub)
            .clamp(0, self.params.hist_bonus_max)
    }

    /// Penalty magnitude for searched moves that failed to cut (applied
    /// negated). Stored positive.
    pub(super) fn history_malus(&self, depth: i32) -> i32 {
        (self.params.hist_malus_mul * depth - self.params.hist_malus_sub)
            .clamp(0, self.params.hist_malus_max)
    }

    pub(super) fn update_cutoff_tables(
        &mut self,
        board: &Board,
        best: Move,
        best_piece: Piece,
        previous: Move,
        ply: usize,
        depth: i32,
        bonus_pct: i32,
        quiets: &[Move],
        good_caps: &BadCaptureList,
        bad_caps: &BadCaptureList,
    ) {
        if self.killers[ply][0] != best {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = best;
        }

        let color = board.side_to_move();
        let pawn_key = board.pawn_key();
        // 8.4(e): `bonus_pct` carries the surprise scale (100 = neutral); it
        // applies to every REWARD for the best move (main/pawn/low-ply and the
        // continuation entries below) but never to a malus.
        let bonus = self.history_bonus(depth) * bonus_pct / 100;
        let malus = self.history_malus(depth);
        self.update_quiet_history(color, best, best_piece, pawn_key, ply, bonus);
        // NOTE, 4.5.3: these quiets get a malus in main, low-ply and pawn
        // history but deliberately NOT in continuation history. That asymmetry
        // looks like an omission and was measured as a candidate: adding the
        // continuation malus leaves ordering flat (first-move cutoff 88.04% ->
        // 88.09%) while cutting the tree 7.5% and total cutoffs 9.6%. Cutoffs
        // fall FASTER than nodes, so it is not an ordering gain — continuation
        // history feeds `quiet_hist`, which drives two of LMP's four disjuncts
        // and the LMR reduction, so a broad negative push simply prunes more.
        // That is the one direction four independent readings say is wrong for
        // this engine (RAR-S53/S54/S55, and 4.7 paying +15.56 for pruning
        // LESS). Rejected on measurement, not left undone.
        for &quiet in quiets {
            let quiet_piece = board.moving_piece(quiet);
            self.update_quiet_history(color, quiet, quiet_piece, pawn_key, ply, -malus);
        }
        for good_cap in good_caps.as_slice() {
            self.update_capture_history(
                good_cap.attacker,
                good_cap.to as usize,
                good_cap.captured,
                -malus,
            );
        }
        for bad_cap in bad_caps.as_slice() {
            self.update_capture_history(
                bad_cap.attacker,
                bad_cap.to as usize,
                bad_cap.captured,
                -malus,
            );
        }

        if !previous.is_null() {
            self.countermove[previous.from_sq().index()][previous.to_sq().index()] = best;
        }

        let piece = best_piece as usize;
        let to = best.to_sq().index();
        for (slot, &(back, divisor)) in CONT_PLY_BACK.iter().enumerate() {
            if ply < back {
                continue;
            }
            let prev = self.stack[ply - back].mv;
            if prev.is_null() {
                continue;
            }
            let index = self.stack[ply - back].cont_row_base() + piece_to_index(piece, to);
            update_hist_entry(
                &mut self.cont_history[slot][index],
                bonus / divisor,
                HISTORY_MAX,
            );
        }
    }

    pub(super) fn update_quiet_history(
        &mut self,
        color: Color,
        mv: Move,
        piece: Piece,
        pawn_key: u64,
        ply: usize,
        bonus: i32,
    ) {
        update_hist_entry(
            &mut self.main_history[color as usize][mv.from_sq().index()][mv.to_sq().index()],
            bonus,
            HISTORY_MAX,
        );
        if ply < LOW_PLY_HISTORY_SIZE {
            update_hist_entry(
                &mut self.low_ply_history[ply][mv.from_sq().index()][mv.to_sq().index()],
                bonus,
                HISTORY_MAX,
            );
        }
        update_hist_entry(
            &mut self.pawn_history
                [pawn_history_index(pawn_key, piece as usize, mv.to_sq().index())],
            bonus,
            HISTORY_MAX,
        );
    }

    pub(super) fn update_capture_history(
        &mut self,
        attacker: Piece,
        to: usize,
        captured: Option<Piece>,
        bonus: i32,
    ) {
        if let Some(captured) = captured {
            update_hist_entry(
                &mut self.cap_history[attacker as usize][to][captured as usize],
                bonus,
                CAP_HISTORY_MAX,
            );
        }
    }

    pub(super) fn age_history(&mut self) {
        for color in self.main_history.iter_mut() {
            for from in color.iter_mut() {
                for value in from.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for attacker in self.cap_history.iter_mut() {
            for to in attacker.iter_mut() {
                for value in to.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for ply in self.low_ply_history.iter_mut() {
            for from in ply.iter_mut() {
                for value in from.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for value in self.pawn_history.iter_mut() {
            *value /= 2;
        }
        for table in self.cont_history.iter_mut() {
            for value in table.iter_mut() {
                *value /= 2;
            }
        }
        for color in self.correction_history.iter_mut() {
            for value in color.iter_mut() {
                *value /= 2;
            }
        }
        for color in self.minor_correction_history.iter_mut() {
            for value in color.iter_mut() {
                *value /= 2;
            }
        }
        for stm in self.non_pawn_correction_history.iter_mut() {
            for color in stm.iter_mut() {
                for value in color.iter_mut() {
                    *value /= 2;
                }
            }
        }
        for value in self.continuation_correction_history.iter_mut() {
            *value /= 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Square;

    /// The continuation index SPECIFICATION: the search reads a per-ply
    /// `cont_key` instead, and the test below proves the identity
    /// `StackEntry::cont_row_base` depends on. Test-only since B.1.
    /// Flat index into a continuation-history table.
    ///
    /// 9.0a: the inputs are structurally bounded — `piece`/`prev_piece` come from
    /// a 6-variant `Piece` cast and the squares from `Square::index()` (masked to
    /// 0..=63), so the result is always < `CONT_SIZE`. The `.min()` remains as the
    /// release-mode backstop (it keeps the function total), but it used to be the
    /// ONLY thing here: an out-of-range index was silently folded into the last
    /// bucket, so a logic bug would quietly corrupt one history cell instead of
    /// surfacing. The `debug_assert!`s now state the invariant and fail loudly in
    /// debug and under `cargo test`.
    ///
    /// NB `CONT_SIZE` (147,456) and `PIECE_TO_SIZE` (384) are NOT powers of two,
    /// so `& (SIZE - 1)` is *not* a valid substitute for `.min()` here — masking
    /// would remap valid in-range indices (65,536 would fold to 0). Only
    /// `pawn_history_index`'s 4,096-entry slot table may use a mask.
    fn cont_index(prev_piece: usize, prev_to: usize, piece: usize, to: usize) -> usize {
        debug_assert!(prev_piece < 6 && piece < 6, "piece index out of range");
        debug_assert!(prev_to < 64 && to < 64, "square index out of range");
        (((prev_piece * 64 + prev_to) * 6 + piece) * 64 + to).min(CONT_SIZE - 1)
    }

    /// Node-invariant prefix of [`cont_index`]: the row base for a
    /// `(prev_piece, prev_to)` pair, such that
    /// `cont_index(pp, pt, piece, to) == (cont_row_base(pp, pt) + piece_to_index(piece, to)).min(CONT_SIZE - 1)`
    /// (equivalence pinned by a test below). Move scoring resolves this once per
    /// node instead of once per quiet move — the 8.12(g2) hoist from the Basilisk
    /// cross-review (its 8.7.6(b+d), +3.03% NPS there).
    fn cont_row_base(prev_piece: usize, prev_to: usize) -> usize {
        debug_assert!(prev_piece < 6, "piece index out of range");
        debug_assert!(prev_to < 64, "square index out of range");
        (prev_piece * 64 + prev_to) * PIECE_TO_SIZE
    }

    /// The hoisted row-base + per-move offset decomposition must agree with
    /// the original single-shot index everywhere — this is what makes the
    /// 8.12(g2) scoring hoist a pure refactor.
    #[test]
    fn row_base_decomposition_matches_cont_and_pawn_indexes() {
        for prev_piece in 0..6 {
            for prev_to in (0..64).step_by(7) {
                for piece in 0..6 {
                    for to in (0..64).step_by(5) {
                        assert_eq!(
                            (cont_row_base(prev_piece, prev_to) + piece_to_index(piece, to))
                                .min(CONT_SIZE - 1),
                            cont_index(prev_piece, prev_to, piece, to),
                        );
                    }
                }
            }
        }
        for key in [0u64, 1, 0xFFFF, 0xDEAD_BEEF_CAFE_F00D, u64::MAX] {
            for piece in 0..6 {
                for to in (0..64).step_by(9) {
                    assert_eq!(
                        pawn_row_base(key) + piece_to_index(piece, to),
                        pawn_history_index(key, piece, to),
                    );
                }
            }
        }
    }

    #[test]
    fn quiet_history_uses_low_ply_slots_through_ply_seven() {
        let mut searcher = Searcher::default();
        let board = Board::default();
        let mv = board.parse_move("a2a3").expect("legal quiet move");
        let from = Square::A2.index();
        let to = Square::A3.index();

        searcher.low_ply_history[7][from][to] = 800;

        let ci = board.check_info();
        let ctx7 = searcher.quiet_history_ctx(&board, 7);
        assert_eq!(
            searcher.quiet_history_score(&board, &ci, &ctx7, Color::White, mv, 7),
            100
        );
        let ctx8 = searcher.quiet_history_ctx(&board, 8);
        assert_eq!(
            searcher.quiet_history_score(&board, &ci, &ctx8, Color::White, mv, 8),
            0
        );
    }

    #[test]
    fn quiet_history_updates_only_configured_low_ply_window() {
        let mut searcher = Searcher::default();
        let board = Board::default();
        let in_window = board.parse_move("a2a3").expect("legal quiet move");
        let outside_window = board.parse_move("h2h3").expect("legal quiet move");

        searcher.update_quiet_history(
            Color::White,
            in_window,
            Piece::Pawn,
            board.pawn_key(),
            LOW_PLY_HISTORY_SIZE - 1,
            400,
        );
        searcher.update_quiet_history(
            Color::White,
            outside_window,
            Piece::Pawn,
            board.pawn_key(),
            LOW_PLY_HISTORY_SIZE,
            400,
        );

        assert!(
            searcher.low_ply_history[LOW_PLY_HISTORY_SIZE - 1][Square::A2.index()]
                [Square::A3.index()]
                > 0
        );
        assert_eq!(
            searcher.low_ply_history[LOW_PLY_HISTORY_SIZE - 1][Square::H2.index()]
                [Square::H3.index()],
            0
        );
    }
}
