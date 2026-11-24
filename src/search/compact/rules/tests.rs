use super::super::quiescence::Q_DEPTH;
use super::*;

#[test]
fn a_draw_scores_within_the_kernels_spread_and_reaches_both_ends() {
    let spread = crate::search::DRAW_SPREAD;
    let scores: Vec<i32> = (0..5).map(draw_score).collect();
    assert!(
        scores.iter().all(|score| score.abs() <= spread),
        "{scores:?}"
    );
    assert!(
        scores.contains(&spread) && scores.contains(&-spread),
        "{scores:?}"
    );
}

/// The kernel plays the donor's form: the evaluation's distance to the
/// raised beta carried into the exchange values at the fold's scale.
#[test]
fn probcuts_exchange_bar_is_the_distance_to_its_beta_at_the_seeded_scale() {
    for (probcut_beta, eval) in [(300, 100), (100, 300), (0, 0), (-50, -1_250)] {
        for (table, scale) in [(SEEDS, 2_630), (&Seeds::DONOR, 2_536)] {
            let carried = i64::from(probcut_beta - eval) * scale / 1_024;
            assert_eq!(
                i64::from(probcut_see_bar(table, probcut_beta, eval)),
                carried
            );
        }
    }
    let mut unconverted = Seeds::FITTED;
    unconverted.probcut_see_scale = 1_024;
    assert_eq!(
        probcut_see_bar(&unconverted, 300, 100),
        200,
        "1,024 is the bare distance"
    );
    let far = TB_WIN_SCORE - 1;
    let mut wide = Seeds::FITTED;
    wide.probcut_see_scale = 4_096;
    assert_eq!(probcut_see_bar(&wide, far, -far), 8 * far, "no overflow");
}

#[test]
fn a_quiescence_entry_bounds_the_null_search_at_depth_one_only() {
    let (beta, stored) = (100, 40);
    let bound = |depth, tt_depth| null_bound(depth, beta, stored, Some(Bound::Lower), tt_depth);
    for (depth, tt_depth, expected) in [
        (1, Q_DEPTH, stored),
        (2, Q_DEPTH, beta),
        (3, Q_DEPTH, beta),
        (2, 1, stored),
        (3, 1, stored),
        (4, 1, beta),
    ] {
        assert_eq!(
            bound(depth, tt_depth),
            expected,
            "depth {depth}, stored at {tt_depth}"
        );
    }
}

#[test]
fn each_spread_term_offsets_a_thread_by_its_own_seed() {
    // A one-thread trace cannot see the per-thread offsets; this can.
    let s = &Seeds::DONOR;
    for (term, per_thread) in [
        (SpreadTerm::Lmr, s.lmr_spread_thread),
        (SpreadTerm::Unreduced, s.fds_spread_thread),
    ] {
        assert_eq!(spread(s, term, 1_000, 0), 1_000 % 128);
        for thread in 1..4_u64 {
            let offset = u64::try_from(per_thread).expect("a positive offset") * thread;
            let expected = i32::try_from((1_000 + offset) % 128).expect("below 128");
            assert_eq!(
                spread(s, term, 1_000, thread),
                expected,
                "{term:?} {thread}"
            );
        }
    }
    assert_ne!(
        spread(s, SpreadTerm::Lmr, 0, 1),
        spread(s, SpreadTerm::Unreduced, 0, 1),
        "the terms differ by thread"
    );
}

// Properties of the rules. Each holds for any sane seed table, so each is
// checked against both tables the kernel has searched with, the donor's
// and the fitted one, over deterministic random inputs: a later fit that
// breaks one has changed a rule's shape, not tuned it.

use crate::eval::MATE_SCORE;

const TABLES: [(&str, &Seeds); 2] = [("donor", &Seeds::DONOR), ("fitted", &Seeds::FITTED)];
const CASES: usize = 4_000;

/// A deterministic xorshift stream.
struct Draw(u64);

impl Draw {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn int(&mut self, lo: i32, hi: i32) -> i32 {
        let span = u64::try_from(i64::from(hi) - i64::from(lo) + 1).expect("lo <= hi");
        lo + i32::try_from(self.next() % span).expect("inside the span")
    }

    fn coin(&mut self) -> bool {
        self.next() & 1 == 1
    }

    /// An ordinary score, now and then a decisive one either way.
    fn score(&mut self) -> i32 {
        match self.int(0, 9) {
            0 => self.int(TB_WIN_SCORE, MATE_SCORE),
            1 => -self.int(TB_WIN_SCORE, MATE_SCORE),
            _ => self.int(-2_000, 2_000),
        }
    }

    /// A stored score: sometimes none.
    fn stored(&mut self) -> i32 {
        if self.int(0, 7) == 0 {
            VALUE_NONE
        } else {
            self.score()
        }
    }

    fn bound(&mut self) -> Option<Bound> {
        match self.int(0, 3) {
            0 => None,
            1 => Some(Bound::Upper),
            2 => Some(Bound::Lower),
            _ => Some(Bound::Exact),
        }
    }
}

fn move_input(d: &mut Draw) -> MovePruneInput {
    let alpha = d.int(-1_500, 1_500);
    MovePruneInput {
        in_check: d.int(0, 4) == 0,
        direct_check: d.int(0, 4) == 0,
        quiet: d.coin(),
        depth: d.int(1, 14),
        history: d.int(-16_000, 16_000),
        eval: d.int(-2_000, 2_000),
        alpha,
        beta: if d.int(0, 9) == 0 {
            TB_WIN_SCORE
        } else {
            alpha + 1
        },
        improvement: d.int(-300, 300),
        correction: d.int(-400, 400),
        move_count: d.int(1, 40),
        bad_stage: d.coin(),
    }
}

fn reduction_input(d: &mut Draw) -> ReductionInput {
    let alpha = d.int(-1_000, 1_000);
    let pv = d.coin();
    ReductionInput {
        depth: d.int(1, 30),
        improvement: d.int(-300, 300),
        correction: d.int(-400, 400),
        exact: d.coin(),
        tt_score: d.stored(),
        tt_depth: d.int(-1, 30),
        alpha,
        beta: if pv { alpha + d.int(1, 200) } else { alpha + 1 },
        quiet: d.coin(),
        history: d.int(-16_000, 16_000),
        estimate: d.int(-2_000, 2_000),
        since_critical: d.int(0, 8),
        pv,
        root_delta: d.int(1, 400),
        laterality: d.int(0, 12),
        tt_pv: d.coin(),
        cut_node: !pv && d.coin(),
        tt_move_null: d.coin(),
        gives_check: d.coin(),
        child_cutoffs: d.int(0, 6),
        tt_move_score: if d.coin() {
            VALUE_NONE
        } else {
            d.int(-1_000, 1_000)
        },
        singular_score: if d.coin() {
            VALUE_NONE
        } else {
            d.int(-1_000, 1_000)
        },
        parent_reduction: 0,
        spread: d.int(0, 127),
        is_tt_move: d.coin(),
    }
}

#[test]
fn the_wrapper_keeps_evaluations_ordinary_and_damps_them_toward_the_fifty_move_horizon() {
    for (name, s) in TABLES {
        let mut d = Draw::new(1);
        for _ in 0..CASES {
            let input = WrapperInput {
                raw: d.int(-2 * MATE_SCORE, 2 * MATE_SCORE),
                material: d.int(0, 9_000),
                optimism: d.int(-400, 400),
                fifty: d.int(0, 100),
                correction: d.int(-30_000, 30_000),
            };
            let eval = wrapped_eval(s, &input);
            assert!(eval.abs() < TB_WIN_SCORE, "{name}: {input:?} -> {eval}");

            let plain = WrapperInput {
                raw: d.int(-3_000, 3_000),
                optimism: 0,
                correction: 0,
                ..input
            };
            let now = wrapped_eval(s, &plain);
            let later = wrapped_eval(
                s,
                &WrapperInput {
                    fifty: plain.fifty + 1,
                    ..plain
                },
            );
            assert!(
                later.abs() <= now.abs(),
                "{name}: {plain:?}: {now} then {later}"
            );
            assert!(
                now == 0 || now.signum() == plain.raw.signum(),
                "{name}: {plain:?}"
            );
            let richer = wrapped_eval(
                s,
                &WrapperInput {
                    material: plain.material + 500,
                    ..plain
                },
            );
            assert!(
                richer.abs() >= now.abs(),
                "{name}: more material shrank {plain:?}"
            );
            let corrected = wrapped_eval(
                s,
                &WrapperInput {
                    correction: 37,
                    ..plain
                },
            );
            assert_eq!(
                corrected,
                now + 37,
                "{name}: the correction is added one for one"
            );
        }
    }
}

#[test]
fn optimism_is_odd_bounded_and_grows_with_the_average() {
    for (name, s) in TABLES {
        for average in -3_000..=3_000 {
            let value = optimism(s, average);
            assert_eq!(optimism(s, -average), -value, "{name}: {average}");
            assert!(value.abs() <= s.optimism_scale.abs(), "{name}: {average}");
            assert!(optimism(s, average + 1) >= value, "{name}: {average}");
        }
    }
}

#[test]
fn a_stored_score_replaces_the_evaluation_only_where_its_bound_tightens_it() {
    let mut d = Draw::new(2);
    for _ in 0..CASES {
        let input = EstimateInput {
            in_check: d.int(0, 4) == 0,
            excluded: d.int(0, 4) == 0,
            tt_score: d.stored(),
            bound: d.bound(),
            eval: d.int(-2_000, 2_000),
        };
        let estimate = estimated_score(&input);
        assert!(
            estimate == input.eval || estimate == input.tt_score,
            "{input:?}"
        );
        if input.in_check || input.excluded || input.tt_score == VALUE_NONE {
            assert_eq!(estimate, input.eval, "{input:?}");
        }
        match input.bound {
            Some(Bound::Upper) => assert!(estimate <= input.eval, "{input:?}"),
            Some(Bound::Lower) => assert!(estimate >= input.eval, "{input:?}"),
            _ => {}
        }
    }
}

#[test]
fn a_table_cutoff_needs_depth_an_agreeing_bound_and_a_node_off_the_pv() {
    for (name, s) in TABLES {
        let mut d = Draw::new(3);
        for _ in 0..CASES {
            let alpha = d.int(-1_000, 1_000);
            let input = TtCutInput {
                pv: d.int(0, 3) == 0,
                excluded: d.int(0, 5) == 0,
                tt_depth: d.int(-1, 20),
                depth: d.int(1, 20),
                tt_score: d.stored(),
                alpha,
                beta: alpha + 1,
                bound: d.bound(),
                cut_node: d.coin(),
            };
            if !tt_cuts(s, &input) {
                continue;
            }
            assert!(
                !input.pv && !input.excluded && input.tt_score != VALUE_NONE,
                "{name}: {input:?}"
            );
            assert!(
                input.tt_depth >= input.depth - 1,
                "{name}: too shallow {input:?}"
            );
            match input.bound {
                Some(Bound::Upper) => {
                    assert!(input.tt_score <= input.alpha, "{name}: {input:?}");
                }
                Some(Bound::Lower) => {
                    assert!(input.tt_score >= input.beta, "{name}: {input:?}");
                }
                _ => {}
            }
        }
    }
}

#[test]
fn rewards_for_a_cutting_table_move_are_capped_and_grow_with_depth() {
    for (name, s) in TABLES {
        for depth in 1..30 {
            let input = TtCutRewardInput {
                tt_quiet: true,
                tt_score: 100,
                beta: 50,
                parent_move_count: 0,
                depth,
            };
            let (quiet, cont) = tt_cut_rewards(s, &input).expect("a quiet cut early in the parent");
            let (deeper_quiet, deeper_cont) = tt_cut_rewards(
                s,
                &TtCutRewardInput {
                    depth: depth + 1,
                    ..input
                },
            )
            .expect("again");
            assert!(
                deeper_quiet >= quiet && deeper_cont >= cont,
                "{name}: depth {depth}"
            );
            assert!(
                quiet <= s.tt_bonus_quiet_cap && cont <= s.tt_bonus_cont_cap,
                "{name}"
            );
            assert!(
                tt_cut_rewards(
                    s,
                    &TtCutRewardInput {
                        tt_quiet: false,
                        ..input
                    }
                )
                .is_none()
            );
            assert!(
                tt_cut_rewards(
                    s,
                    &TtCutRewardInput {
                        tt_score: 49,
                        ..input
                    }
                )
                .is_none()
            );
            let late = TtCutRewardInput {
                parent_move_count: s.tt_bonus_parent_moves,
                ..input
            };
            assert!(
                tt_cut_rewards(s, &late).is_none(),
                "{name}: a late parent move earns nothing"
            );
        }
    }
}

#[test]
fn hindsight_moves_a_node_at_most_one_ply_and_never_below_one() {
    for (name, s) in TABLES {
        let mut d = Draw::new(4);
        for _ in 0..CASES {
            let depth = d.int(1, 30);
            let tt_pv = d.coin();
            let after = hindsight_depth(s, depth, d.int(-2_000, 6_000), d.int(-500, 500), tt_pv);
            assert!(
                (depth - 1..=depth + 1).contains(&after) && after >= 1,
                "{name}: {depth} -> {after}"
            );
            assert!(
                !tt_pv || after >= depth,
                "{name}: a PV line is never searched shallower"
            );
        }
    }
}

#[test]
fn razoring_keeps_off_the_pv_and_out_of_check_and_needs_more_room_deeper() {
    for (name, s) in TABLES {
        let mut d = Draw::new(5);
        for _ in 0..CASES {
            let input = RazorInput {
                pv: d.int(0, 3) == 0,
                in_check: d.int(0, 3) == 0,
                estimate: d.int(-3_000, 1_000),
                alpha: d.int(-1_000, 1_500),
                depth: d.int(1, 8),
                tt_quiet: d.int(0, 3) == 0,
                bound: d.bound(),
            };
            if razors(s, &input) {
                assert!(
                    !input.pv && !input.in_check && !input.tt_quiet,
                    "{name}: {input:?}"
                );
                assert_ne!(input.bound, Some(Bound::Lower), "{name}: {input:?}");
                assert!(input.alpha < s.razor_alpha_limit, "{name}: {input:?}");
                assert!(
                    razors(
                        s,
                        &RazorInput {
                            estimate: input.estimate - 1,
                            ..input
                        }
                    ),
                    "{name}"
                );
                if input.depth > 1 {
                    assert!(
                        razors(
                            s,
                            &RazorInput {
                                depth: input.depth - 1,
                                ..input
                            }
                        ),
                        "{name}"
                    );
                }
            }
        }
    }
}

#[test]
fn reverse_futility_keeps_its_guards_and_returns_between_beta_and_the_estimate() {
    for (name, s) in TABLES {
        let mut d = Draw::new(6);
        for _ in 0..CASES {
            let input = RfpInput {
                tt_pv: d.int(0, 3) == 0,
                in_check: d.int(0, 3) == 0,
                excluded: d.int(0, 5) == 0,
                estimate: d.score(),
                beta: d.score(),
                depth: d.int(1, 12),
                improvement: d.int(-300, 300),
                correction: d.int(-400, 400),
                unthreatened: d.coin(),
            };
            assert!(rfp_margin(s, &input) >= s.rfp_min_16, "{name}: {input:?}");
            if rfp_cuts(s, &input) {
                assert!(
                    !input.tt_pv && !input.in_check && !input.excluded,
                    "{name}: {input:?}"
                );
                assert!(!is_decisive(input.estimate), "{name}: {input:?}");
                assert!(
                    !is_loss(input.beta) && input.estimate >= input.beta,
                    "{name}: {input:?}"
                );
                assert!(
                    rfp_cuts(
                        s,
                        &RfpInput {
                            estimate: input.estimate + 1,
                            ..input
                        }
                    ) || is_win(input.estimate + 1)
                );
                let value = rfp_value(s, input.estimate, input.beta);
                assert!(
                    (input.beta..=input.estimate).contains(&value),
                    "{name}: {input:?} -> {value}"
                );
            }
        }
    }
}

#[test]
fn the_null_move_keeps_its_gate_and_a_reduction_that_grows_with_depth_and_surplus() {
    for (name, s) in TABLES {
        let mut d = Draw::new(7);
        for _ in 0..CASES {
            let input = NullGateInput {
                cut_node: d.int(0, 3) != 0,
                in_check: d.int(0, 3) == 0,
                excluded: d.int(0, 5) == 0,
                singular: d.int(0, 5) == 0,
                estimate: d.score(),
                beta: d.score(),
                depth: d.int(1, 20),
                tt_pv: d.coin(),
                improvement: d.int(-300, 300),
                child_cutoffs: d.int(0, 4),
                ply: d.int(1, 40),
                nmp_min_ply: if d.coin() { 0 } else { d.int(1, 40) },
                material: d.int(0, 8_000),
                tt_capture_refutes: d.int(0, 5) == 0,
            };
            assert!(nmp_margin(s, &input) >= s.nmp_min_16, "{name}");
            if nmp_gate(s, &input) {
                assert!(
                    input.cut_node && !input.in_check && !input.excluded && !input.singular,
                    "{name}: {input:?}"
                );
                assert!(
                    input.ply >= input.nmp_min_ply && input.material > s.nmp_material,
                    "{name}: {input:?}"
                );
                assert!(
                    !input.tt_capture_refutes && input.estimate >= input.beta,
                    "{name}: {input:?}"
                );
                assert!(
                    !is_loss(input.beta) && !is_win(input.estimate),
                    "{name}: {input:?}"
                );
            }
        }
        for depth in 1..30 {
            for surplus in [0, 50, 200, 1_000, 10_000] {
                let r = nmp_reduction(s, false, depth, 100 + surplus, 100);
                assert!(r >= 1, "{name}: a null move is searched shallower");
                assert!(
                    nmp_reduction(s, false, depth + 1, 100 + surplus, 100) >= r,
                    "{name}"
                );
                assert!(
                    nmp_reduction(s, false, depth, 101 + surplus, 100) >= r,
                    "{name}"
                );
            }
        }
    }
}

#[test]
fn a_null_search_cuts_only_at_beta_and_never_on_a_decisive_score() {
    for (name, s) in TABLES {
        let mut d = Draw::new(8);
        for _ in 0..CASES {
            let beta = d.int(-1_000, 1_000);
            let (depth, tt_depth) = (d.int(1, 20), d.int(-1, 20));
            let null_beta = null_bound(depth, beta, d.stored(), d.bound(), tt_depth);
            assert!(null_beta <= beta, "{name}: the bound is never above beta");
            let score = d.score();
            let reduction = d.int(1, 8);
            match null_outcome(s, score, null_beta, beta, depth, reduction, d.int(0, 3)) {
                NullOutcome::Fail => {}
                NullOutcome::Cut => {
                    assert!(
                        score >= beta && !is_decisive(score),
                        "{name}: cut on {score} at {beta}"
                    );
                }
                NullOutcome::Verify(verify) => {
                    assert!(score >= null_beta && !is_decisive(score), "{name}");
                    assert!(verify < depth, "{name}: verified at {verify} from {depth}");
                }
            }
            if is_decisive(score) || score < null_beta {
                assert_eq!(
                    null_outcome(s, score, null_beta, beta, depth, reduction, 0),
                    NullOutcome::Fail
                );
            }
        }
    }
}

#[test]
fn probcut_raises_beta_checks_its_gate_and_returns_a_score_between_beta_and_its_own() {
    for (name, s) in TABLES {
        let mut d = Draw::new(9);
        for _ in 0..CASES {
            let beta = d.int(-1_000, 1_000);
            let improving = d.coin();
            let raised = probcut_beta(s, beta, improving);
            assert!(raised > beta, "{name}: ProbCut asks for more than beta");
            let gate = ProbcutGateInput {
                cut_node: d.int(0, 3) != 0,
                beta,
                probcut_beta: raised,
                tt_score: d.stored(),
                eval: d.int(-1_500, 1_500),
                tt_quiet: d.int(0, 3) == 0,
            };
            if probcut_gate(&gate) {
                assert!(
                    gate.cut_node && !gate.tt_quiet && !is_win(beta),
                    "{name}: {gate:?}"
                );
            }
            let depth = d.int(1, 20);
            let score = raised + d.int(0, 2_000);
            let plan = probcut_depth(s, depth, improving, score, raised);
            assert!(
                plan.depth >= 0 && plan.depth <= plan.base && plan.base <= depth,
                "{name}: {plan:?}"
            );
            assert!(plan.adjusted_beta >= raised, "{name}: {plan:?}");
            let more = probcut_depth(s, depth, improving, score + 500, raised);
            assert!(
                more.depth <= plan.depth,
                "{name}: a larger surplus verifies no deeper"
            );
            let value = probcut_value(s, score, beta);
            assert!(
                (beta..=score).contains(&value),
                "{name}: {score} at {beta} -> {value}"
            );
            let mate = MATE_SCORE - 7;
            assert_eq!(
                probcut_value(s, mate, beta),
                mate,
                "{name}: a mate passes unchanged"
            );
        }
    }
}

#[test]
fn a_singular_search_extends_one_to_three_plies_and_cuts_only_on_an_ordinary_score() {
    for (name, s) in TABLES {
        let mut d = Draw::new(10);
        for depth in 1..30 {
            for exact in [false, true] {
                let margin = singular_margin(s, depth, exact, false);
                assert!(margin >= 0, "{name}");
                assert!(
                    singular_margin(s, depth + 1, exact, false) >= margin,
                    "{name}: {depth}"
                );
                assert!(
                    singular_margin(s, depth, exact, true) >= margin,
                    "{name}: {depth}"
                );
            }
            assert!(
                singular_margin(s, depth, true, false) <= singular_margin(s, depth, false, false)
            );
        }
        for _ in 0..CASES {
            let tt_score = d.int(-1_000, 1_000);
            let singular_beta = tt_score - d.int(0, 100);
            let input = SingularInput {
                singular_score: d.score(),
                singular_beta,
                beta: singular_beta + d.int(1, 300),
                tt_score,
                pv: d.coin(),
                tt_was_pv: d.coin(),
                tt_quiet: d.coin(),
                correction: d.int(-400, 400),
                cut_node: d.coin(),
                searched_a_move: d.coin(),
            };
            match singular_outcome(s, &input) {
                SingularOutcome::Extend(plies) if input.singular_score < singular_beta => {
                    assert!((1..=3).contains(&plies), "{name}: {plies}");
                    let lower = SingularInput {
                        singular_score: input.singular_score - 50,
                        ..input
                    };
                    let SingularOutcome::Extend(more) = singular_outcome(s, &lower) else {
                        panic!("{name}: lower is still singular")
                    };
                    assert!(more >= plies, "{name}: further below extends no less");
                }
                SingularOutcome::Extend(plies) => {
                    assert_eq!(plies, s.negative_extension, "{name}");
                }
                SingularOutcome::MultiCut(value) => {
                    assert!(!is_decisive(input.singular_score), "{name}: {input:?}");
                    assert!(
                        (input.beta..=input.singular_score).contains(&value),
                        "{name}: {input:?}"
                    );
                }
                SingularOutcome::Demote => {
                    assert!(
                        input.singular_score > tt_score && input.searched_a_move,
                        "{name}"
                    );
                }
                SingularOutcome::None => {}
            }
        }
    }
}

#[test]
fn move_pruning_spares_checks_and_evasions_and_eases_with_a_better_move() {
    for (name, s) in TABLES {
        let mut d = Draw::new(11);
        for _ in 0..CASES {
            let i = move_input(&mut d);
            let any = lmp_prunes(s, &i)
                || futility_prunes(s, &i)
                || bad_noisy_prunes(s, &i)
                || history_prunes(s, &i);
            if i.in_check {
                assert!(!any, "{name}: pruned an evasion {i:?}");
            }
            if i.direct_check {
                assert!(
                    !lmp_prunes(s, &i) && !futility_prunes(s, &i) && !bad_noisy_prunes(s, &i),
                    "{name}: {i:?}"
                );
            }
            if lmp_prunes(s, &i) {
                assert!(i.quiet && !is_win(i.beta), "{name}: {i:?}");
                assert!(
                    lmp_prunes(
                        s,
                        &MovePruneInput {
                            move_count: i.move_count + 1,
                            ..i
                        }
                    ),
                    "{name}"
                );
            }
            if futility_prunes(s, &i) {
                assert!(i.quiet, "{name}: {i:?}");
                assert!(
                    futility_prunes(
                        s,
                        &MovePruneInput {
                            eval: i.eval - 1,
                            ..i
                        }
                    ),
                    "{name}: {i:?}"
                );
                assert!(futility_value(s, &i) <= i.alpha, "{name}: {i:?}");
            }
            if bad_noisy_prunes(s, &i) {
                assert!(i.bad_stage, "{name}: only the losing noisy moves");
                assert!(bad_noisy_value(s, &i) <= i.alpha, "{name}: {i:?}");
            }
            if history_prunes(s, &i) {
                assert!(i.quiet, "{name}");
                assert!(
                    history_prunes(
                        s,
                        &MovePruneInput {
                            history: i.history - 1,
                            ..i
                        }
                    ),
                    "{name}"
                );
            }
            assert!(
                lmp_limit(
                    s,
                    &MovePruneInput {
                        depth: i.depth + 1,
                        ..i
                    }
                ) >= lmp_limit(s, &i),
                "{name}"
            );
            for quiet in [false, true] {
                let threshold = see_threshold(s, quiet, i.depth, i.history);
                assert!(
                    threshold <= 0,
                    "{name}: a move is never asked to win material"
                );
                assert!(
                    see_threshold(s, quiet, i.depth, i.history + 1_000) <= threshold,
                    "{name}"
                );
            }
        }
    }
}

#[test]
fn reductions_never_drop_below_a_ply_and_ease_for_good_history_and_checks() {
    for (name, s) in TABLES {
        let mut d = Draw::new(12);
        for _ in 0..CASES {
            let i = reduction_input(&mut d);
            let r = late_move_reduction(s, &i);
            let deeper = late_move_reduction(
                s,
                &ReductionInput {
                    depth: i.depth + 1,
                    ..i
                },
            );
            assert!(deeper >= r, "{name}: deeper reduced less {i:?}");
            let better = late_move_reduction(
                s,
                &ReductionInput {
                    history: i.history + 2_000,
                    ..i
                },
            );
            assert!(better <= r, "{name}: better history reduced more {i:?}");
            if !i.gives_check {
                let checking = late_move_reduction(
                    s,
                    &ReductionInput {
                        gives_check: true,
                        ..i
                    },
                );
                assert!(checking <= r, "{name}: a check reduced more {i:?}");
            }
            let new_depth = d.int(1, 30);
            for pv in [false, true] {
                let depth = lmr_depth(new_depth, r, pv);
                let floor = 1 + 2 * i32::from(pv);
                assert!(
                    (floor..=new_depth + 2 + 2 * i32::from(pv)).contains(&depth),
                    "{name}: {depth}"
                );
                assert!(lmr_depth(new_depth, r + 1_024, pv) <= depth, "{name}");
            }
            let unreduced = unreduced_reduction(s, &i);
            let full = unreduced_depth(s, new_depth, unreduced);
            assert!(
                (new_depth - 2..=new_depth).contains(&full),
                "{name}: {full} from {new_depth}"
            );
            assert!(
                unreduced_depth(s, new_depth, unreduced + 1_024) <= full,
                "{name}"
            );
            let root = research_depth(s, true, d.score(), d.score(), new_depth);
            assert_eq!(root, new_depth, "{name}: the root keeps its depth");
            let research = research_depth(
                s,
                false,
                d.int(-1_000, 1_000),
                d.int(-1_000, 1_000),
                new_depth,
            );
            assert!(
                (new_depth - 1..=new_depth + 1).contains(&research),
                "{name}"
            );
        }
    }
}

#[test]
fn history_amounts_grow_with_depth_and_maluses_fade_with_the_moves_sharing_them() {
    for (name, s) in TABLES {
        for depth in 1..40 {
            for cut_node in [false, true] {
                let now = update_amounts(s, depth, cut_node, 3, 5);
                let deeper = update_amounts(s, depth + 1, cut_node, 3, 5);
                for (a, b) in [
                    (now.quiet_bonus, deeper.quiet_bonus),
                    (now.noisy_bonus, deeper.noisy_bonus),
                    (now.cont_bonus, deeper.cont_bonus),
                    (now.quiet_malus, deeper.quiet_malus),
                    (now.parent_malus, deeper.parent_malus),
                    (now.research_bonus, deeper.research_bonus),
                ] {
                    assert!(b >= a, "{name}: depth {depth}");
                }
                let crowded = update_amounts(s, depth, cut_node, 10, 20);
                assert!(crowded.quiet_malus <= now.quiet_malus, "{name}");
                assert!(crowded.noisy_malus <= now.noisy_malus, "{name}");
            }
            let open = update_amounts(s, depth, false, 3, 5);
            let cut = update_amounts(s, depth, true, 3, 5);
            assert!(
                cut.quiet_bonus <= open.quiet_bonus && cut.cont_bonus <= open.cont_bonus,
                "{name}"
            );
        }
        assert_eq!(
            malus_fade(s, 0),
            1_024,
            "{name}: the first quiet takes its whole malus"
        );
        for index in 0..64 {
            assert!(
                malus_fade(s, index + 1) <= malus_fade(s, index),
                "{name}: {index}"
            );
            assert!(malus_fade(s, index) > 0, "{name}: {index}");
        }
    }
}

#[test]
fn a_fail_low_rewards_the_parents_move_more_for_its_table_move_and_with_depth() {
    for (name, s) in TABLES {
        let mut d = Draw::new(13);
        for _ in 0..CASES {
            let input = RewardInput {
                parent_move_count: d.int(0, 40),
                parent_tt: false,
                in_check: d.coin(),
                best_score: d.int(-1_500, 1_500),
                eval: d.int(-1_500, 1_500),
                parent_eval: d.stored(),
                depth: d.int(1, 30),
            };
            let reward = fail_low_reward(s, &input);
            assert!(reward.factor >= s.reward_base, "{name}");
            let tt = fail_low_reward(
                s,
                &RewardInput {
                    parent_tt: true,
                    ..input
                },
            );
            assert!(tt.factor >= reward.factor, "{name}");
            let deeper = fail_low_reward(
                s,
                &RewardInput {
                    depth: input.depth + 1,
                    ..input
                },
            );
            assert!(
                deeper.bonus >= reward.bonus && deeper.grandparent >= reward.grandparent,
                "{name}"
            );
        }
        for depth in 1..30 {
            assert!(
                noisy_fail_low_reward(s, depth + 1) >= noisy_fail_low_reward(s, depth),
                "{name}"
            );
        }
    }
}

#[test]
fn the_correction_learns_the_residuals_sign_only_where_the_evaluation_is_to_blame() {
    for (name, s) in TABLES {
        let mut d = Draw::new(14);
        for _ in 0..CASES {
            let input = CorrectionInput {
                in_check: d.int(0, 4) == 0,
                noisy_best: d.int(0, 4) == 0,
                bound: match d.int(0, 2) {
                    0 => Bound::Upper,
                    1 => Bound::Lower,
                    _ => Bound::Exact,
                },
                best_score: d.int(-3_000, 3_000),
                eval: d.int(-3_000, 3_000),
                depth: d.int(1, 60),
            };
            let residual = input.best_score - input.eval;
            match correction_update(s, &input) {
                None => assert!(
                    input.in_check
                        || input.noisy_best
                        || (input.bound == Bound::Upper && residual >= 0)
                        || (input.bound == Bound::Lower && residual <= 0),
                    "{name}: {input:?}"
                ),
                Some(update) => {
                    assert!(!input.in_check && !input.noisy_best, "{name}: {input:?}");
                    assert!(
                        update == 0 || update.signum() == residual.signum(),
                        "{name}: {input:?}"
                    );
                    assert!(
                        (s.corr_min..=s.corr_max).contains(&update),
                        "{name}: {update}"
                    );
                }
            }
        }
    }
}

#[test]
fn quiescence_cuts_and_stands_pat_by_the_bound_and_softens_only_ordinary_fail_highs() {
    for (name, s) in TABLES {
        let mut d = Draw::new(15);
        for _ in 0..CASES {
            let (pv, tt_score, bound) = (d.coin(), d.stored(), d.bound());
            let alpha = d.int(-1_000, 1_000);
            let beta = alpha + d.int(1, 200);
            if q_tt_cuts(pv, tt_score, bound, alpha, beta) {
                assert!(
                    tt_score != VALUE_NONE && !(pv && is_decisive(tt_score)),
                    "{name}"
                );
                match bound {
                    Some(Bound::Upper) => assert!(tt_score <= alpha, "{name}"),
                    Some(Bound::Lower) => assert!(tt_score >= beta, "{name}"),
                    _ => {}
                }
            }
            let eval = d.int(-1_500, 1_500);
            assert_eq!(
                stand_pat(true, pv, eval, tt_score, bound),
                -INF_SCORE,
                "{name}"
            );
            let standing = stand_pat(false, pv, eval, tt_score, bound);
            assert!(standing == eval || standing == tt_score, "{name}");
            let score = d.score();
            for value in [
                q_stand_pat_value(s, score, beta),
                q_tail_value(s, score, beta),
            ] {
                if score < beta || is_decisive(score) {
                    assert_eq!(value, score, "{name}: {score} at {beta}");
                } else {
                    assert!(
                        (beta..=score).contains(&value),
                        "{name}: {score} at {beta} -> {value}"
                    );
                }
            }
            let count = d.int(0, 10);
            assert!(
                !q_stops(s, count, true),
                "{name}: a direct check is always searched"
            );
            if q_stops(s, count, false) {
                assert!(q_stops(s, count + 1, false), "{name}");
            }
        }
    }
}

#[test]
fn a_failed_root_window_moves_to_the_failing_score_and_grows() {
    for (name, s) in TABLES {
        let mut d = Draw::new(16);
        for _ in 0..CASES {
            let delta = window_delta(s, d.int(0, 20), d.int(0, 20), d.int(-3_000, 3_000));
            assert!(delta >= 1, "{name}: an empty window cannot hold a score");
            let average = d.int(-2_000, 2_000);
            let window = open_window(average, delta);
            assert!(
                window.alpha < average && average < window.beta,
                "{name}: {window:?}"
            );

            let low = window.alpha - d.int(0, 3_000);
            let after = window_fail_low(s, window, low);
            assert!(
                after.alpha < low && low <= after.beta,
                "{name}: {low} not inside {after:?}"
            );
            assert!(
                after.beta <= window.beta && after.delta >= window.delta,
                "{name}: {after:?}"
            );

            let high = if d.int(0, 9) == 0 {
                MATE_SCORE - 9
            } else {
                window.beta + d.int(0, 3_000)
            };
            let after = window_fail_high(s, window, high);
            assert!(
                after.alpha < high && high < after.beta,
                "{name}: {high} not inside {after:?}"
            );
            assert!(
                after.alpha >= window.alpha && after.delta >= window.delta,
                "{name}: {after:?}"
            );
            let expected = if is_decisive(high) {
                window.reduction.min(1)
            } else {
                window.reduction + 1
            };
            assert_eq!(
                after.reduction, expected,
                "{name}: a mate is not searched shallower"
            );
        }
        for score in -500..=500 {
            assert_eq!(
                score_held(s, score, 0),
                score_held(s, -score, 0),
                "{name}: {score}"
            );
        }
    }
}

#[test]
fn move_order_puts_cheaper_evaders_bigger_victims_and_safe_quiets_first() {
    for (name, s) in TABLES {
        let mut d = Draw::new(17);
        for _ in 0..CASES {
            let i = NoisyOrderInput {
                victim: d.int(0, 1_300),
                history: d.int(-8_000, 8_000),
                queen_promotion: d.coin(),
                in_check: d.coin(),
                piece: d.int(0, 4),
            };
            let score = noisy_order(s, &i);
            assert!(
                noisy_order(
                    s,
                    &NoisyOrderInput {
                        victim: i.victim + 100,
                        ..i
                    }
                ) >= score,
                "{name}"
            );
            if i.in_check {
                assert!(
                    noisy_order(
                        s,
                        &NoisyOrderInput {
                            piece: i.piece + 1,
                            ..i
                        }
                    ) <= score,
                    "{name}"
                );
            }
            let q = QuietOrderInput {
                quiet: d.int(-8_000, 8_000),
                pawn: d.int(-8_000, 8_000),
                cont: [
                    d.int(-8_000, 8_000),
                    d.int(-8_000, 8_000),
                    d.int(-8_000, 8_000),
                    d.int(-8_000, 8_000),
                ],
                piece: usize::try_from(d.int(0, 5)).expect("a piece index"),
                escape: false,
                check_square: false,
                threatened_to: false,
                offense: false,
                wall: false,
            };
            let plain = quiet_order(s, &q);
            assert!(
                quiet_order(s, &QuietOrderInput { escape: true, ..q }) >= plain,
                "{name}"
            );
            assert!(
                quiet_order(
                    s,
                    &QuietOrderInput {
                        check_square: true,
                        ..q
                    }
                ) >= plain,
                "{name}"
            );
            assert!(
                quiet_order(s, &QuietOrderInput { offense: true, ..q }) >= plain,
                "{name}"
            );
            assert!(
                quiet_order(
                    s,
                    &QuietOrderInput {
                        threatened_to: true,
                        ..q
                    }
                ) <= plain,
                "{name}"
            );
            assert!(
                quiet_order(s, &QuietOrderInput { wall: true, ..q }) <= plain,
                "{name}"
            );
            assert!(
                quiet_order(
                    s,
                    &QuietOrderInput {
                        quiet: q.quiet + 1_024,
                        ..q
                    }
                ) >= plain,
                "{name}"
            );
            let ordered = d.int(-20_000, 20_000);
            let bar = good_noisy_threshold(s, ordered, false, 0);
            assert!(
                good_noisy_threshold(s, ordered + 1_000, false, 0) <= bar,
                "{name}"
            );
            assert_eq!(
                good_noisy_threshold(s, ordered, true, s.pick_late_good + 1),
                1,
                "{name}"
            );
        }
    }
}
