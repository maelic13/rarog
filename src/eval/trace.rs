//! Texel trace machinery. `tr_mg!`/`tr_eg!` record each weight's net feature
//! count at its scoring site; under `--features texel` the trace types below
//! reconstruct the linear tapered score from those counts, which is what the
//! tuner differentiates.

#[cfg(feature = "texel")]
use super::TOTAL_PHASE;
#[cfg(feature = "texel")]
use super::endgame::{
    OCB_SCALE_NORMAL, SCALE_NORMAL, has_only_king, has_only_knights, opposite_bishop_scale,
    specialized_endgame_scale,
};
#[cfg(feature = "texel")]
use super::params::{EvalParams, eval_params};
#[cfg(feature = "texel")]
use crate::board::{Board, Color};

// Texel trace recording. `tr_mg!`/`tr_eg!` add a net feature count
// to the current `Evaluator::trace` at every `mg += sign·W·n` / `eg += ...`
// site. Without `--features texel` they expand to nothing, so production builds
// compile to byte-identical code and `self.trace` is never referenced.
#[cfg(feature = "texel")]
macro_rules! tr_mg {
    ($self:ident, $field:ident, $idx:expr, $v:expr) => {
        $self.trace.borrow_mut().mg.$field[$idx] += ($v) as i32;
    };
}
#[cfg(not(feature = "texel"))]
macro_rules! tr_mg {
    ($($t:tt)*) => {};
}
#[cfg(feature = "texel")]
macro_rules! tr_eg {
    ($self:ident, $field:ident, $idx:expr, $v:expr) => {
        $self.trace.borrow_mut().eg.$field[$idx] += ($v) as i32;
    };
}
#[cfg(not(feature = "texel"))]
macro_rules! tr_eg {
    ($($t:tt)*) => {};
}
pub(super) use tr_eg;
pub(super) use tr_mg;

/// The trace types, generated from `params.rs`'s weight list.
#[cfg(feature = "texel")]
macro_rules! define_eval_trace {
    ( $( $field:ident : $len:literal = $default:expr; )* ) => {
        // ---- Texel trace machinery — `--features texel` only ----
        // EvalCounts mirrors EvalParams field-for-field but holds *net feature
        // counts* (white − black). The trace records, per scoring site, how
        // many times each weight entered the mg and/or eg accumulator, so the
        // raw tapered eval can be reconstructed as Σ count·weight and the Texel
        // tuner can differentiate the loss w.r.t. each weight analytically.
        #[cfg(feature = "texel")]
        #[derive(Clone)]
        pub struct EvalCounts {
            $( pub $field: [i32; $len], )*
        }

        #[cfg(feature = "texel")]
        impl Default for EvalCounts {
            fn default() -> Self {
                Self { $( $field: [0; $len], )* }
            }
        }

        #[cfg(feature = "texel")]
        #[derive(Clone, Default)]
        pub struct EvalTrace {
            pub mg: EvalCounts,
            pub eg: EvalCounts,
            pub phase: i32,
            /// Untraced (frozen, non-tunable) contributions to the pre-taper mg
            /// and eg accumulators — the mate-drive mop-up and the passer-king
            /// proximity `rel_rank` constant. They are excluded from the linear
            /// reconstruction (they end up in the tuner's per-position `rest`).
            pub frozen_mg: i32,
            pub frozen_eg: i32,
            /// The king-danger map penalised at least one king. Its output is
            /// untraced, so this is how a reader sees the term was active.
            pub king_danger: bool,
            /// White-POV raw tapered score of the *linear* part only (mg/eg with
            /// the frozen contributions removed). The reconstruction gate asserts
            /// `reconstruct(defaults) == raw`, validating every traced count.
            pub raw: i32,
        }

        #[cfg(feature = "texel")]
        impl EvalTrace {
            pub fn reset(&mut self) {
                *self = Self::default();
            }

            /// White-POV raw tapered eval reconstructed from the counts:
            /// `(Σcount_mg·w · phase + Σcount_eg·w · (24−phase)) / 24`. At the
            /// default weights this must equal the pre-scaling tapered score
            /// `evaluate()` computed (the reconstruction acceptance gate).
            pub fn reconstruct(&self, p: &EvalParams) -> i32 {
                let mut mg: i64 = 0;
                let mut eg: i64 = 0;
                $( for i in 0..$len {
                    mg += i64::from(self.mg.$field[i]) * i64::from(p.$field[i]);
                    eg += i64::from(self.eg.$field[i]) * i64::from(p.$field[i]);
                } )*
                crate::infra::saturating_i32(
                    (mg * i64::from(self.phase) + eg * (i64::from(TOTAL_PHASE) - i64::from(self.phase)))
                        / i64::from(TOTAL_PHASE),
                )
            }

            /// Per-flat-index tapered coefficient `(count_mg·phase +
            /// count_eg·(24−phase))/24` — the linear sensitivity of the raw
            /// eval to each weight, in `EVAL_PARAM_NAMES` flat order.
            pub fn flat_coeffs(&self) -> Vec<f64> {
                let ph = f64::from(self.phase);
                let mut out = Vec::with_capacity(EvalParams::FLAT_SIZE);
                $( for i in 0..$len {
                    out.push(
                        (f64::from(self.mg.$field[i]) * ph
                            + f64::from(self.eg.$field[i]) * (f64::from(TOTAL_PHASE) - ph))
                            / f64::from(TOTAL_PHASE),
                    );
                } )*
                out
            }
        }

        #[cfg(feature = "texel")]
        impl EvalParams {
            /// Total number of scalar weights across all fields.
            pub const FLAT_SIZE: usize = 0 $( + $len )*;

            /// Flatten the weights into a contiguous `f64` vector in
            /// `EVAL_PARAM_NAMES` order (the tuner's working representation).
            pub fn to_flat(&self) -> Vec<f64> {
                let mut out = Vec::with_capacity(Self::FLAT_SIZE);
                $( for i in 0..$len { out.push(f64::from(self.$field[i])); } )*
                out
            }

            /// Inverse of `to_flat` (rounds to nearest integer weight).
            ///
            /// The float->int narrowing is intended and well defined: `as`
            /// from a float saturates at the target's bounds and maps NaN to
            /// 0 (guaranteed by the language since Rust 1.45), and `.round()`
            /// is exactly the rounding the tuner wants. There is no checked
            /// helper to reach for here.
            #[expect(
                clippy::cast_possible_truncation,
                reason = "rounding a fitted weight to the nearest integer is the intent"
            )]
            pub fn set_from_flat(&mut self, w: &[f64]) {
                let mut k = 0usize;
                $( for i in 0..$len { self.$field[i] = w[k].round() as i32; k += 1; } )*
            }
        }
    };
}

#[cfg(feature = "texel")]
eval_params!(define_eval_trace);
/// The multiplicative factor `scale_endgame` applies to the raw tapered score,
/// as an `f64` (texel only). The tuner scales each position's weight *deltas*
/// by it, so a small weight change predicts the score change the evaluator
/// would make. The evaluator applies no rule-50 damping (the search does,
/// outside the evaluation), so neither does this factor: the fit's base score
/// and its deltas must be on the same footing.
#[cfg(feature = "texel")]
pub fn linear_delta_scale(board: &Board) -> f64 {
    // Specialised endgame scale factors apply first, as in `scale_endgame`.
    // A dead-draw pattern zeroes the delta scale.
    if let Some(sf) = specialized_endgame_scale(board) {
        return f64::from(sf) / f64::from(SCALE_NORMAL);
    }

    if (has_only_king(board, Color::White) && has_only_knights(board, Color::Black, 2))
        || (has_only_king(board, Color::Black) && has_only_knights(board, Color::White, 2))
    {
        return 0.0;
    }

    opposite_bishop_scale(board).map_or(1.0, |ocb| f64::from(ocb) / f64::from(OCB_SCALE_NORMAL))
}

// The reconstruction acceptance gate: for a large, diverse set of
// positions, the trace must reconstruct the raw tapered eval `evaluate()`
// computed *exactly* (integer-for-integer). A mismatch means a trace count is
// wrong; that must be fixed before any tuning run, since the tuner's gradients
// are built from these counts.
#[cfg(all(test, feature = "texel"))]
mod texel_tests;
