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

/// **Nonzero activation**: every term must actually fire on a position
/// designed to trigger it, otherwise the fit's feature-support diagnostics
/// would tune a dead term blind.
#[test]
fn new_terms_activate_on_curated_positions() {
    let mut ev = full_evaluator();

    // Passers and the free-stop / safe-stop terms: a clear, unattacked passer.
    let t = trace_of(&mut ev, "4k3/8/8/3P4/8/8/8/4K3 w - - 0 1");
    assert!(t.eg.passed_eg.iter().any(|&c| c != 0), "passed_eg dead");
    assert_ne!(t.eg.passed_freestop_eg_per_rank[0], 0, "free-stop dead");
    assert_ne!(t.eg.passed_safestop_eg_per_rank[0], 0, "safe-stop dead");

    // Per-count knight mobility one-hot.
    let t = trace_of(&mut ev, "4k3/8/8/8/4N3/8/8/4K3 w - - 0 1");
    assert!(t.mg.mob_n_mg.iter().any(|&c| c != 0), "mob_n_mg dead");

    // Material imbalance: 3 white pawns vs none → nonzero net products.
    let t = trace_of(&mut ev, "4k3/8/8/8/8/8/PPP5/4K3 w - - 0 1");
    assert!(
        t.mg.imbalance_ours.iter().any(|&c| c != 0),
        "imbalance dead"
    );

    // Threat-by-minor: white knight e4 attacks the black rook on d6.
    let t = trace_of(&mut ev, "4k3/8/3r4/8/4N3/8/8/4K3 w - - 0 1");
    assert!(
        t.mg.threat_by_minor_mg.iter().any(|&c| c != 0),
        "threat_by_minor dead"
    );

    // King threat: the white king on d5 attacks the undefended pawn on e5.
    let t = trace_of(&mut ev, "4k3/8/8/3Kp3/8/8/8/8 w - - 0 1");
    assert_ne!(t.mg.threat_by_king_mg[0], 0, "threat_by_king dead");

    // Slider threat on the queen: the rook on h5 and the knight on f4
    // both attack d5, on the black queen's file.
    let t = trace_of(&mut ev, "3q4/8/k7/7R/5N2/8/8/K7 w - - 0 1");
    assert_ne!(
        t.mg.threat_slider_on_queen_mg[0], 0,
        "threat_slider_on_queen dead"
    );

    // Minor-behind-pawn: white knight d3 shielded by the pawn on d4.
    let t = trace_of(&mut ev, "4k3/8/8/8/3P4/3N4/8/4K3 w - - 0 1");
    assert_ne!(t.mg.minor_behind_pawn_mg[0], 0, "minor_behind_pawn dead");
}

/// The fit's base score is the evaluator's, which ignores the halfmove
/// clock, so the factor on its weight deltas must ignore it too, while
/// keeping the endgame scale factors.
#[test]
fn delta_scale_follows_the_evaluator_not_the_clock() {
    use super::linear_delta_scale;
    // Opposite bishops with pawns (scaled), a plain position (unscaled)
    // and a drawn KPK (zeroed), each at a fresh and a late clock.
    const CASES: &[(&str, &str, Option<f64>)] = &[
        (
            "4k3/8/2b5/3p4/3P4/8/5B2/4K3 w - - 0 1",
            "4k3/8/2b5/3p4/3P4/8/5B2/4K3 w - - 90 80",
            None,
        ),
        (
            "4k3/8/8/8/8/2N5/PPP5/4K3 w - - 0 1",
            "4k3/8/8/8/8/2N5/PPP5/4K3 w - - 90 80",
            Some(1.0),
        ),
        (
            "k7/8/K7/P7/8/8/8/8 w - - 0 1",
            "k7/8/K7/P7/8/8/8/8 w - - 90 80",
            Some(0.0),
        ),
    ];
    let mut ev = full_evaluator();
    for &(fresh, late, expected) in CASES {
        let fresh = Board::from_fen(fresh).expect("fresh FEN");
        let late = Board::from_fen(late).expect("late FEN");
        let factor = linear_delta_scale(&fresh);
        assert_eq!(
            factor,
            linear_delta_scale(&late),
            "clock moved the delta scale"
        );
        assert_eq!(
            ev.evaluate(&fresh),
            ev.evaluate(&late),
            "clock moved the score"
        );
        match expected {
            Some(value) => assert_eq!(factor, value),
            None => assert!(factor > 0.0 && factor < 1.0, "OCB factor {factor}"),
        }
    }
}

/// Passer king proximity at the block square: counts are capped distance
/// times `5·rank − 13`, our king's next-square count skipped on the
/// seventh rank.
#[test]
fn passer_king_proximity_counts() {
    let mut ev = full_evaluator();
    // e5 (w = 7): black king two from e6 (14), ours five from e6 (35) and
    // six, capped at five, from e7 (35).
    let t = trace_of(&mut ev, "4k3/8/8/4P3/8/8/8/4K3 w - - 0 1");
    assert_eq!(t.eg.passer_king_them_eg[0], 14);
    assert_eq!(t.eg.passer_king_us_eg[0], -35);
    assert_eq!(t.eg.passer_king_next_eg[0], -35);
    // a7 (w = 17): the block square is the promotion square, no next.
    let t = trace_of(&mut ev, "4k3/P7/8/8/8/8/8/K7 w - - 0 1");
    assert_eq!(t.eg.passer_king_them_eg[0], 4 * 17);
    assert_eq!(t.eg.passer_king_us_eg[0], -5 * 17);
    assert_eq!(t.eg.passer_king_next_eg[0], 0);
    // A passer on its third rank takes no proximity term.
    let t = trace_of(&mut ev, "4k3/8/8/8/8/4P3/8/4K3 w - - 0 1");
    assert_eq!(t.eg.passer_king_them_eg[0], 0);
    assert_eq!(t.eg.passer_king_us_eg[0], 0);
    // Colour symmetry.
    let t = trace_of(&mut ev, "4k3/8/8/8/4p3/8/8/4K3 b - - 0 1");
    assert_eq!(t.eg.passer_king_them_eg[0], -14);
    assert_eq!(t.eg.passer_king_us_eg[0], 35);
}

/// Weak unopposed: an isolated or backward pawn with no enemy pawn ahead on
/// its file, counted once per pawn.
#[test]
fn weak_unopposed_pawns() {
    let mut ev = full_evaluator();
    let count = |ev: &mut Evaluator, fen: &str| {
        let t = trace_of(ev, fen);
        (
            t.mg.pawn_weak_unopposed_mg[0],
            t.eg.pawn_weak_unopposed_eg[0],
        )
    };
    // An isolated unopposed pawn.
    assert_eq!(count(&mut ev, "4k3/8/8/8/4P3/8/8/4K3 w - - 0 1"), (-1, -1));
    // Both pawns isolated and opposed: nothing.
    assert_eq!(count(&mut ev, "4k3/4p3/8/8/4P3/8/8/4K3 w - - 0 1"), (0, 0));
    // e3 backward (d5 attacks e4, d4 is ahead of it) and unopposed; d4 and
    // d5 face each other.
    assert_eq!(
        count(&mut ev, "4k3/8/8/3p4/3P4/4P3/8/4K3 w - - 0 1"),
        (-1, -1)
    );
    // e3 both isolated and backward: counted once; Black's c6-d5 chain is
    // neither.
    assert_eq!(
        count(&mut ev, "4k3/8/2p5/3p4/8/4P3/8/4K3 w - - 0 1"),
        (-1, -1)
    );
}

/// The unstoppable passer follows the rule of the square, counting the
/// king's capture of the new queen.
#[test]
fn unstoppable_passer_square_rule() {
    let mut ev = full_evaluator();
    // a5 against a king on e5: moving first, the king catches it (1...Kd6
    // 2.a6 Kc7 3.a7 Kb7 4.a8=Q+ Kxa8); moving second, it does not.
    let t = trace_of(&mut ev, "8/8/8/P3k3/8/8/8/K7 b - - 0 1");
    assert_eq!(t.eg.unstoppable_passer_eg[0], 0);
    let t = trace_of(&mut ev, "8/8/8/P3k3/8/8/8/K7 w - - 0 1");
    assert_eq!(t.eg.unstoppable_passer_eg[0], 1);
    // One square further, it queens whoever moves.
    let t = trace_of(&mut ev, "8/8/8/P4k2/8/8/8/K7 b - - 0 1");
    assert_eq!(t.eg.unstoppable_passer_eg[0], 1);
    // With the pawn's side to move, a king three from a8 still catches it.
    let t = trace_of(&mut ev, "8/8/8/P2k4/8/8/8/K7 w - - 0 1");
    assert_eq!(t.eg.unstoppable_passer_eg[0], 0);
}

/// Four pawn and passer terms fire on the positions they describe and on
/// no others.
#[test]
fn pawn_and_passer_terms_hold_on_their_counterexamples() {
    let mut ev = full_evaluator();

    // Phalanx fires for pawns abreast on the same rank (d4+e4)...
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
    // Support fires for a diagonally-defended pawn (d4 defends e5), and
    // phalanx does not.
    let t = trace_of(&mut ev, "4k3/8/8/4P3/3P4/8/8/4K3 w - - 0 1");
    assert!(
        t.mg.pawn_connected_mg.iter().any(|&c| c != 0),
        "support dead on d4->e5"
    );

    // An enemy rook behind a passer counts even with NO friendly rook on
    // the file.
    let t = trace_of(&mut ev, "4k3/8/8/3P4/8/8/8/3rK3 w - - 0 1");
    assert_ne!(
        t.mg.enemy_rook_behind_passer_mg[0], 0,
        "enemy rook behind passer missed without a friendly rook on the file"
    );

    // The square rule is SUPPRESSED when the defender has a piece (a
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
    // A queen and three pawns against a rook: far above the lazy margin.
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
