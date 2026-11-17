//! The compact kernel: the search in its donor's form, beside the classic
//! kernel behind one switch ([`Kernel`](crate::search::options::Kernel)).
//!
//! Every decision the kernel takes is a pure function of an input struct and
//! the seed table ([`rules`]), so each can be checked against the donor's own
//! decision on the same inputs, and instrumentation reads the decisions
//! instead of sitting between them.
//!
//! The kernel works in its donor's evaluation units. Rarog's raw evaluation
//! is multiplied by [`SCALE`]/1024 where it enters, and a score is divided
//! back where it leaves, so the seeds hold the values the donor was fitted
//! with. The scale is a scaffold: folding the constants into Rarog's units
//! deletes it.

/// Count one asking of the rule `kind` and whether it fired, in a build with
/// the `probes` feature; nothing otherwise, where `fired` is not evaluated.
macro_rules! probe {
    ($searcher:expr, $kind:ident, $fired:expr) => {
        #[cfg(feature = "probes")]
        $searcher
            .td
            .rates
            .record(crate::search::compact::probes::Kind::$kind, $fired);
    };
}

mod node;
#[cfg(test)]
mod parity;
mod picker;
#[cfg(any(test, feature = "probes"))]
pub(super) mod probes;
mod quiescence;
pub(super) mod rules;
pub(super) mod window;

use crate::board::{Board, Color, Move, Piece, SeeValues};
use crate::eval::VALUE_NONE;
use crate::infra;
use crate::tt::TB_WIN_SCORE;

use super::Searcher;
use super::history::cont_context;
use rules::{SEEDS, WrapperInput, wrapped_eval};

/// The kernel's evaluation units per 1024 of Rarog's: the ratio of the
/// donor's mean raw evaluation to Rarog's on a common corpus.
pub(super) const SCALE: i32 = 2536;

/// The kernel's exchange values, the donor's: every SEE threshold and victim
/// term the seeds hold was fitted with them, and the material its evaluation
/// wrapper reads is counted in them. The exchange algorithm stays Rarog's.
pub(super) const KERNEL_SEE_VALUES: SeeValues = SeeValues::new(109, 403, 435, 679, 1242, 20_000);

/// A raw evaluation in kernel units, kept out of the decisive band.
pub(super) fn raw_to_kernel(raw: i32) -> i32 {
    let scaled = i64::from(raw) * i64::from(SCALE) / 1024;
    let bound = i64::from(TB_WIN_SCORE - 1);
    i32::try_from(scaled.clamp(-bound, bound))
        .unwrap_or_else(|_| unreachable!("clamped into i32 bounds"))
}

/// A kernel score in Rarog's units, for everything outside the kernel that
/// reads one: a decisive score is a distance, not an evaluation, and passes
/// unchanged, as do the window's infinities and the absent score.
pub(super) fn score_to_rarog(score: i32) -> i32 {
    if score.abs() >= TB_WIN_SCORE || score == VALUE_NONE {
        return score;
    }
    i32::try_from(i64::from(score) * 1024 / i64::from(SCALE))
        .unwrap_or_else(|_| unreachable!("a non-decisive score shrinks"))
}

/// A quiet move by the kernel's partition: any move but a capture or a
/// queen promotion, the rook, bishop and knight promotions among them. The
/// kernel's noisy tables, ordering and pruning read this partition. No move
/// is not quiet.
pub(super) fn is_quiet_move(mv: Move) -> bool {
    !mv.is_null() && !mv.is_capture() && !(mv.is_promo() && mv.promo_piece() == Piece::Queen)
}

/// The material on the board in the kernel's exchange values, kings at zero.
fn kernel_material(board: &Board) -> i32 {
    let values = KERNEL_SEE_VALUES.as_array();
    [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
    ]
    .into_iter()
    .map(|piece| {
        let both = board.pieces(Color::White, piece) | board.pieces(Color::Black, piece);
        infra::to_i32(both.count()) * values[piece as usize]
    })
    .sum()
}

impl Searcher {
    /// Put the move about to be searched at `ply` on the stack: its piece,
    /// its victim, its continuation context by the kernel's partition, and
    /// its order index and the line's lateness.
    #[inline(always)]
    fn compact_push_move(
        &mut self,
        board: &Board,
        ply: usize,
        mv: Move,
        piece: Piece,
        move_count: i32,
    ) {
        let captured = board.captured_piece(mv);
        let entry = &mut self.td.stack[ply];
        entry.mv = mv;
        entry.piece = piece;
        entry.captured = captured;
        entry.cont_key = cont_context(
            board.is_in_check(),
            !is_quiet_move(mv),
            board.side_to_move(),
            piece,
            mv.to_sq(),
        );
        self.record_move_order(ply, move_count);
    }

    /// `mv` heads the line at `ply`, followed by the child's line.
    #[inline(always)]
    fn compact_update_pv(&mut self, ply: usize, mv: Move) {
        self.td.pv_table[ply][ply] = mv;
        let child_len = self.td.pv_len[ply + 1].max(ply + 1);
        for next_ply in ply + 1..child_len {
            self.td.pv_table[ply][next_ply] = self.td.pv_table[ply + 1][next_ply];
        }
        self.td.pv_len[ply] = child_len;
    }

    /// The compact kernel's corrected evaluation from a raw evaluation in
    /// kernel units and the board's `material` in exchange values, and the
    /// correction it includes. A draw verdict is a classification, not an
    /// estimate, and passes through as 0, with no correction and no optimism.
    pub(super) fn compact_eval(
        &self,
        board: &Board,
        raw: i32,
        material: i32,
        ply: usize,
    ) -> (i32, i32) {
        probe!(self, Wrapper, true);
        if raw == 0 && crate::eval::endgame::is_draw_verdict(board) {
            return (0, 0);
        }
        let correction = self.compact_correction(board, ply);
        let eval = wrapped_eval(
            SEEDS,
            &WrapperInput {
                raw,
                material,
                optimism: self.td.optimism[board.side_to_move() as usize],
                fifty: i32::from(board.halfmove_clock()),
                correction,
            },
        );
        (eval, correction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn the_unit_boundary_scales_evaluations_and_keeps_distances() {
        assert_eq!(raw_to_kernel(100), 247, "100 * 2536 / 1024, truncated");
        assert_eq!(raw_to_kernel(-100), -247, "truncated toward zero");
        assert_eq!(
            raw_to_kernel(20_000),
            TB_WIN_SCORE - 1,
            "held below the band"
        );
        assert_eq!(raw_to_kernel(-20_000), -TB_WIN_SCORE + 1);
        assert_eq!(score_to_rarog(247), 99);
        assert_eq!(score_to_rarog(-2536), -1024);
        for distance in [
            TB_WIN_SCORE,
            31_990,
            -31_990,
            -TB_WIN_SCORE,
            32_001,
            -32_001,
        ] {
            assert_eq!(score_to_rarog(distance), distance);
        }
        assert_eq!(score_to_rarog(VALUE_NONE), VALUE_NONE);
    }

    #[test]
    fn the_wrapper_weighs_material_optimism_and_keeps_verdicts() {
        let mut searcher = Searcher::default();
        let board = Board::starting_position();
        // 200 * (21032 + 10296) / 27015, no correction trained.
        let material = kernel_material(&board);
        assert_eq!(searcher.compact_eval(&board, 200, material, 0), (231, 0));
        searcher.td.optimism = [10, -10];
        // (200 * 31328 + 10 * (1548 + 10296)) / 27015.
        assert_eq!(searcher.compact_eval(&board, 200, material, 0), (236, 0));
        let bare_kings = Board::from_fen("8/8/8/4k3/8/8/8/4K3 w - - 0 1").expect("a legal FEN");
        assert_eq!(searcher.compact_eval(&bare_kings, 0, 0, 0), (0, 0));
    }

    #[test]
    fn the_start_position_counts_the_donors_material() {
        let board = Board::starting_position();
        assert_eq!(
            kernel_material(&board),
            16 * 109 + 4 * 403 + 4 * 435 + 4 * 679 + 2 * 1242
        );
    }
}
