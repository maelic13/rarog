//! The compact kernel's decisions, each a pure function of an input struct
//! and the seed table.
//!
//! A rule reads nothing but its arguments, so the parity instrument can call
//! it on the donor's recorded inputs and compare its output with the donor's
//! recorded decision, and a probe can count its firings without sitting
//! inside the node. The seeds are the donor's fitted values in its units;
//! the fold into Rarog's units rewrites this table.

use crate::tt::TB_WIN_SCORE;

/// Every constant the kernel's rules read, in kernel units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Seeds {
    /// Evaluation wrapper: `(raw·(raw_base + material) + optimism·
    /// (optimism_base + material)) / divisor`, then damped linearly to zero
    /// at `fifty_horizon` plies of the fifty-move clock.
    pub(crate) wrap_raw_base: i32,
    pub(crate) wrap_optimism_base: i32,
    pub(crate) wrap_divisor: i32,
    pub(crate) wrap_fifty_horizon: i32,
}

impl Seeds {
    /// The donor's values.
    pub(crate) const DONOR: Self = Self {
        wrap_raw_base: 21_032,
        wrap_optimism_base: 1_548,
        wrap_divisor: 27_015,
        wrap_fifty_horizon: 200,
    };
}

/// The seeds the kernel searches with.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read by the compact kernel's nodes, not yet wired"
    )
)]
pub(crate) const SEEDS: &Seeds = &Seeds::DONOR;

/// The evaluation wrapper's inputs.
#[derive(Copy, Clone, Debug)]
pub(crate) struct WrapperInput {
    /// The raw evaluation in kernel units.
    pub(crate) raw: i32,
    /// Material on the board in the kernel's exchange values.
    pub(crate) material: i32,
    /// The side to move's optimism.
    pub(crate) optimism: i32,
    /// The fifty-move clock in plies.
    pub(crate) fifty: i32,
    /// The correction tables' value for the position.
    pub(crate) correction: i32,
}

/// The corrected evaluation: the raw evaluation weighed up with the material
/// on the board, optimism added the same way, both damped toward the
/// fifty-move horizon, then corrected and kept out of the decisive band.
#[inline(always)]
pub(crate) fn wrapped_eval(s: &Seeds, i: &WrapperInput) -> i32 {
    let raw = i64::from(i.raw) * i64::from(s.wrap_raw_base + i.material);
    let optimism = i64::from(i.optimism) * i64::from(s.wrap_optimism_base + i.material);
    let mut eval = (raw + optimism) / i64::from(s.wrap_divisor);
    eval = eval * i64::from(s.wrap_fifty_horizon - i.fifty) / i64::from(s.wrap_fifty_horizon);
    let bound = i64::from(TB_WIN_SCORE - 1);
    i32::try_from((eval + i64::from(i.correction)).clamp(-bound, bound))
        .unwrap_or_else(|_| unreachable!("clamped into i32 bounds"))
}
