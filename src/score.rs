//! Scores and their bands. A mate scores `MATE_SCORE` less its ply, so
//! every mate the search can find lies from `MATE_BOUND` up; a tablebase
//! result lies from `TB_WIN_SCORE` up to `TB_VALUE`, below the mate band;
//! `INF_SCORE` and `VALUE_NONE` lie above every score a search returns.

use crate::infra;

/// A mate on the board: a line that mates at ply `p` scores `MATE_SCORE - p`.
pub const MATE_SCORE: i32 = 32_000;
/// A window's open end, above every score a search returns.
pub(crate) const INF_SCORE: i32 = 32_001;
/// No score: an empty table slot, or an evaluation not taken.
pub(crate) const VALUE_NONE: i32 = 32_002;

/// The deepest ply a line reaches, the donor's 240: from iteration 16 the
/// line bound's eight times the depth passes a 128-ply cap, and long
/// searches in endings met it.
pub(crate) const MAX_PLY: usize = 240;

/// `MAX_PLY` in score units, for the bands below.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "the assertion below keeps MAX_PLY far inside i32"
)]
const MAX_PLY_SCORE: i32 = MAX_PLY as i32;
const _: () = assert!(MAX_PLY < 1 << 12 && MAX_PLY_SCORE * 2 < MATE_SCORE);

/// The mate band's floor: every mate within `MAX_PLY` plies scores at or
/// above it.
pub(crate) const MATE_BOUND: i32 = MATE_SCORE - MAX_PLY_SCORE;
/// The decisive band's floor: a tablebase win or anything better. Below
/// the mate band lies the tablebase band, `TB_VALUE - ply` for a win found
/// `ply` plies from the root. Window arithmetic near mate scores also lands
/// in it (a fail-hard quiescence stores ProbCut's `beta + margin` at a
/// losing beta), so the table keeps the band's values as stored: moving
/// them by ply changes the search without tablebases.
pub(crate) const TB_WIN_SCORE: i32 = MATE_SCORE - 2 * MAX_PLY_SCORE;
/// A tablebase win at the root; one below the mate band.
pub(crate) const TB_VALUE: i32 = MATE_BOUND - 1;

/// The score of mating `ply` plies from the root.
#[inline(always)]
pub(crate) fn mate_in(ply: usize) -> i32 {
    MATE_SCORE - infra::to_i32(ply)
}

/// A score at or beyond the tablebase win band.
#[inline(always)]
pub(crate) const fn is_win(score: i32) -> bool {
    score >= TB_WIN_SCORE
}

/// A score at or beyond the tablebase loss band.
#[inline(always)]
pub(crate) const fn is_loss(score: i32) -> bool {
    score <= -TB_WIN_SCORE
}

/// A won or lost score: a mate or a tablebase result, either side.
#[inline(always)]
pub(crate) const fn is_decisive(score: i32) -> bool {
    is_win(score) || is_loss(score)
}
