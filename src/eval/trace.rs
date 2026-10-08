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

// Texel trace recording (Phase 3.3). `tr_mg!`/`tr_eg!` add a net feature count
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
        // ---- Texel trace machinery (Phase 3.3) — `--features texel` only ----
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
                    mg += self.mg.$field[i] as i64 * p.$field[i] as i64;
                    eg += self.eg.$field[i] as i64 * p.$field[i] as i64;
                } )*
                crate::infra::saturating_i32(
                    (mg * self.phase as i64 + eg * (TOTAL_PHASE as i64 - self.phase as i64))
                        / TOTAL_PHASE as i64,
                )
            }

            /// Per-flat-index tapered coefficient `(count_mg·phase +
            /// count_eg·(24−phase))/24` — the linear sensitivity of the raw
            /// eval to each weight, in `EVAL_PARAM_NAMES` flat order.
            pub fn flat_coeffs(&self) -> Vec<f64> {
                let ph = self.phase as f64;
                let mut out = Vec::with_capacity(EvalParams::FLAT_SIZE);
                $( for i in 0..$len {
                    out.push(
                        (self.mg.$field[i] as f64 * ph
                            + self.eg.$field[i] as f64 * (TOTAL_PHASE as f64 - ph))
                            / TOTAL_PHASE as f64,
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
                $( for i in 0..$len { out.push(self.$field[i] as f64); } )*
                out
            }

            /// Inverse of `to_flat` (rounds to nearest integer weight).
            ///
            /// The float->int narrowing is intended and well defined: `as`
            /// from a float saturates at the target's bounds and maps NaN to
            /// 0 (guaranteed by the language since Rust 1.45), and `.round()`
            /// is exactly the rounding the tuner wants. There is no checked
            /// helper to reach for here.
            #[expect(clippy::cast_possible_truncation)]
            pub fn set_from_flat(&mut self, w: &[f64]) {
                let mut k = 0usize;
                $( for i in 0..$len { self.$field[i] = w[k].round() as i32; k += 1; } )*
            }
        }
    };
}

#[cfg(feature = "texel")]
eval_params!(define_eval_trace);
/// The multiplicative factor `scale_drawish_endgames` + the rule-50 damping
/// apply to the raw tapered score, as an `f64` (Phase 3.3, texel only). The
/// tuner uses it to scale per-position weight *deltas* the same way the engine
/// scales the eval, so a small weight change predicts the right score change.
/// Mirrors `scale_drawish_endgames` and the `score -= score*rule50/199` line.
#[cfg(feature = "texel")]
pub fn linear_delta_scale(board: &Board) -> f64 {
    // Specialised endgame scale factors (Phase 3.11) apply first, mirroring
    // `scale_endgame`. A dead-draw pattern zeroes the delta scale.
    if let Some(sf) = specialized_endgame_scale(board) {
        return sf as f64 / SCALE_NORMAL as f64 * (199.0 - board.halfmove_clock().min(100) as f64)
            / 199.0;
    }

    let mut scale = 1.0f64;

    if let Some(ocb) = opposite_bishop_scale(board) {
        scale *= ocb as f64 / OCB_SCALE_NORMAL as f64;
    }

    if (has_only_king(board, Color::White) && has_only_knights(board, Color::Black, 2))
        || (has_only_king(board, Color::Black) && has_only_knights(board, Color::White, 2))
    {
        return 0.0;
    }

    let rule50 = board.halfmove_clock().min(100) as f64;
    scale *= (199.0 - rule50) / 199.0;
    scale
}

// Phase 3.3 reconstruction acceptance gate: for a large, diverse set of
// positions, the trace must reconstruct the raw tapered eval `evaluate()`
// computed *exactly* (integer-for-integer). A mismatch means a trace count is
// wrong; that must be fixed before any tuning run, since the tuner's gradients
// are built from these counts.
#[cfg(all(test, feature = "texel"))]
mod texel_tests {
    use super::EvalTrace;
    use crate::board::Board;
    use crate::eval::{EVAL_PARAM_NAMES, EvalParams, Evaluator, TOTAL_PHASE};

    // Deterministic xorshift so the test is reproducible.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    #[test]
    fn trace_reconstructs_eval_exactly_over_random_playouts() {
        let defaults = EvalParams::default();
        let mut evaluator = Evaluator::default();
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let mut checked = 0usize;

        for _ in 0..400 {
            let mut board = Board::starting_position();
            for _ in 0..40 {
                let _ = evaluator.evaluate(&board);
                let trace = evaluator.last_trace();
                let recon = trace.reconstruct(&defaults);
                assert_eq!(
                    recon,
                    trace.raw,
                    "trace reconstruction mismatch in {} (recon {recon} != raw {})",
                    board.to_fen(),
                    trace.raw,
                );
                checked += 1;

                let moves = board.generate_legal_movelist();
                if moves.is_empty() {
                    break;
                }
                let mv = moves.as_slice()[crate::infra::index(rng.next()) % moves.len()];
                board.make_move(mv);
            }
        }
        assert!(
            checked > 5000,
            "expected a broad sample, only checked {checked}"
        );
    }

    fn trace_of(ev: &mut Evaluator, fen: &str) -> super::EvalTrace {
        let board = Board::from_fen(fen).unwrap_or_else(|e| panic!("bad FEN {fen}: {e}"));
        let _ = ev.evaluate(&board);
        ev.last_trace()
    }

    /// An evaluator with the lazy gate out of reach. The term tests below read
    /// terms the gate skips, so they must see the full evaluation whatever the
    /// gate does with their positions.
    fn full_evaluator() -> Evaluator {
        let mut ev = Evaluator::default();
        ev.set_lazy_margin(i32::MAX);
        ev
    }

    /// Phase 3 gate — **nonzero activation**: every new term must actually fire
    /// on a position designed to trigger it, otherwise the Phase-4 feature-support
    /// diagnostics would tune a dead term blind.
    #[test]
    fn new_terms_activate_on_curated_positions() {
        let mut ev = full_evaluator();

        // Passers (3.8) + free-stop / safe-stop (3.14): a clear, unattacked passer.
        let t = trace_of(&mut ev, "4k3/8/8/3P4/8/8/8/4K3 w - - 0 1");
        assert!(t.eg.passed_eg.iter().any(|&c| c != 0), "passed_eg dead");
        assert_ne!(t.eg.passed_freestop_eg_per_rank[0], 0, "free-stop dead");
        assert_ne!(t.eg.passed_safestop_eg_per_rank[0], 0, "safe-stop dead");

        // Per-count knight mobility one-hot (3.7).
        let t = trace_of(&mut ev, "4k3/8/8/8/4N3/8/8/4K3 w - - 0 1");
        assert!(t.mg.mob_n_mg.iter().any(|&c| c != 0), "mob_n_mg dead");

        // Material imbalance (3.9): 3 white pawns vs none → nonzero net products.
        let t = trace_of(&mut ev, "4k3/8/8/8/8/8/PPP5/4K3 w - - 0 1");
        assert!(
            t.mg.imbalance_ours.iter().any(|&c| c != 0),
            "imbalance dead"
        );

        // Threat-by-minor (3.6): white knight e4 attacks the black rook on d6.
        let t = trace_of(&mut ev, "4k3/8/3r4/8/4N3/8/8/4K3 w - - 0 1");
        assert!(
            t.mg.threat_by_minor_mg.iter().any(|&c| c != 0),
            "threat_by_minor dead"
        );

        // Minor-behind-pawn (3.12): white knight d3 shielded by the pawn on d4.
        let t = trace_of(&mut ev, "4k3/8/8/8/3P4/3N4/8/4K3 w - - 0 1");
        assert_ne!(t.mg.minor_behind_pawn_mg[0], 0, "minor_behind_pawn dead");
    }

    /// Phase 7.4 HCE semantics — counterexample tests for the four fixes.
    #[test]
    fn phase_7_4_semantic_fixes() {
        let mut ev = full_evaluator();

        // (iv) Phalanx fires for pawns abreast on the same rank (d4+e4)...
        let t = trace_of(&mut ev, "4k3/8/8/8/3PP3/8/8/4K3 w - - 0 1");
        assert!(
            t.mg.pawn_phalanx_mg.iter().any(|&c| c != 0),
            "phalanx dead on d4+e4"
        );
        // ...but NOT for pawns two files apart (d4, f4 — not adjacent).
        let t = trace_of(&mut ev, "4k3/8/8/8/3P1P2/8/8/4K3 w - - 0 1");
        assert!(
            t.mg.pawn_phalanx_mg.iter().all(|&c| c == 0),
            "phalanx should not fire on non-adjacent pawns"
        );
        // Support (the old "connected") still fires for a diagonally-defended
        // pawn (d4 defends e5), and phalanx does not.
        let t = trace_of(&mut ev, "4k3/8/8/4P3/3P4/8/8/4K3 w - - 0 1");
        assert!(
            t.mg.pawn_connected_mg.iter().any(|&c| c != 0),
            "support dead on d4->e5"
        );

        // (ii) Enemy rook behind a passer is now counted even with NO friendly
        // rook on the file (pre-fix it required a friendly rook to be seen).
        let t = trace_of(&mut ev, "4k3/8/8/3P4/8/8/8/3rK3 w - - 0 1");
        assert_ne!(
            t.mg.enemy_rook_behind_passer_mg[0], 0,
            "enemy rook behind passer missed without a friendly rook on the file"
        );

        // (iii) Square rule is SUPPRESSED when the defender has a piece (a
        // knight can interpose), but fires when the king is the sole defender.
        let t = trace_of(&mut ev, "8/P7/1K6/8/8/8/6n1/7k w - - 0 1");
        assert_eq!(
            t.eg.unstoppable_passer_eg[0], 0,
            "square rule must not fire while the defender has a piece"
        );
        let t = trace_of(&mut ev, "8/P7/1K6/8/8/8/8/7k w - - 0 1");
        assert_ne!(
            t.eg.unstoppable_passer_eg[0], 0,
            "square rule should fire with a king-only defender"
        );
    }

    /// The texel build takes the lazy path by the engine's rule, so a fit
    /// describes the function the engine plays. Above the gate the trace holds
    /// only what `evaluate` adds before the gate (material, piece-square,
    /// pawn structure, passer advance) and tempo; the same position with the
    /// gate out of reach traces the piece-activity terms, so the empty set
    /// above the gate is the gate's doing. Both traces reconstruct exactly.
    #[test]
    fn lazy_gate_applies_under_texel() {
        // A queen and three pawns against a rook: far above the 600 cp gate.
        const FEN: &str = "3rk3/8/8/8/8/8/PPP5/3QK3 w - - 0 1";
        const BEFORE_GATE: &[&str] = &[
            "mg_val",
            "eg_val",
            "pst_mg",
            "pst_eg",
            "passed_mg",
            "passed_eg",
            "passed_supported_mg",
            "passed_supported_eg_base",
            "passed_supported_eg_per_rank",
            "passed_candidate_mg",
            "passed_candidate_eg",
            "pawn_doubled_mg",
            "pawn_doubled_eg",
            "pawn_isolated_mg",
            "pawn_isolated_eg",
            "pawn_connected_mg",
            "pawn_connected_eg",
            "pawn_phalanx_mg",
            "pawn_phalanx_eg",
            "pawn_backward_mg",
            "pawn_backward_eg",
            "pawn_lever_mg",
            "pawn_lever_eg",
            "pawn_doubled_isolated_mg",
            "pawn_doubled_isolated_eg",
            "pawn_islands_mg",
            "pawn_islands_eg",
            "passed_freestop_mg_per_rank",
            "passed_freestop_eg_per_rank",
            "passed_safestop_eg_per_rank",
            "passed_freepath_mg_per_rank",
            "passed_freepath_eg_per_rank",
            "passed_safepath_eg_per_rank",
            "tempo",
        ];
        for name in BEFORE_GATE {
            assert!(
                EVAL_PARAM_NAMES.iter().any(|&(n, _)| n == *name),
                "{name} is not an eval param"
            );
        }

        // Fields outside BEFORE_GATE with a nonzero mg or eg count. A trace
        // at phase 24 (resp. 0) has the mg (resp. eg) counts as coefficients.
        let after_gate = |t: &EvalTrace| -> Vec<&'static str> {
            let mut at = t.clone();
            at.phase = TOTAL_PHASE;
            let mg = at.flat_coeffs();
            at.phase = 0;
            let eg = at.flat_coeffs();
            let mut out = Vec::new();
            let mut k = 0usize;
            for &(name, len) in EVAL_PARAM_NAMES {
                let fired = (k..k + len).any(|i| mg[i] != 0.0 || eg[i] != 0.0);
                if fired && !BEFORE_GATE.contains(&name) {
                    out.push(name);
                }
                k += len;
            }
            out
        };

        let defaults = EvalParams::default();
        let lazy = trace_of(&mut Evaluator::default(), FEN);
        let mut full_evaluator = Evaluator::default();
        full_evaluator.set_lazy_margin(i32::MAX);
        let full = trace_of(&mut full_evaluator, FEN);

        assert_eq!(
            lazy.reconstruct(&defaults),
            lazy.raw,
            "lazy trace does not reconstruct"
        );
        assert_eq!(
            full.reconstruct(&defaults),
            full.raw,
            "full trace does not reconstruct"
        );
        assert!(
            after_gate(&lazy).is_empty(),
            "terms after the gate traced above it: {:?}",
            after_gate(&lazy)
        );
        assert!(
            !after_gate(&full).is_empty(),
            "the full evaluation traced no term after the gate; the check is vacuous"
        );
        assert_ne!(lazy.raw, full.raw, "the gate did not change the evaluation");
    }
}
