//! Layer A of the parity instrument: each of the compact kernel's rules
//! against the donor's own decisions on the same inputs.
//!
//! The donor's tracing twin writes one line per decision it takes at a
//! sample of nodes, `<kind> fire=<0|1> <name>=<int> ...`, its inputs and
//! outputs as named integers. The test reads every trace file in the
//! directory `RAROG_DONOR_TRACE` names and calls the rule for each record of
//! its kind. A trace is the donor's behaviour as numbers, so it stays in
//! ignored storage and this test is a local check, not CI's.
//!
//! The trace is in the donor's evaluation units and the rules in Rarog's,
//! their seeds converted by dimension. A record's evaluation fields are
//! converted the same way, rounded to the nearest unit, and the rule must
//! take every decision the donor took and form every score to the rounding
//! of its inputs. A decision that differs only within a unit of its
//! threshold is an exception; exceptions stay under 2% of each kind and
//! mismatches at zero. A margin or a reduction in 1024ths is checked
//! through the decision it feeds.
//!
//! Every input a rule reads is recorded, among them `nmp_gate`'s capture
//! refutation, `singular_result`'s searched move and `q_stand_pat`'s node
//! type, so the test needs the extended twin's trace.
//!
//! Two outputs are the board's, not a rule's: whether a move passes its SEE
//! threshold (`see_margin`, `q_see`). Their thresholds are checked.
//!
//! Each plant changes one seed; it must fail the kind that reads it and no
//! other, which proves both that the kind's check reads its rule and that
//! no other kind's check sees a change it should not. Three plants convert
//! a seed by the wrong dimension, which its kind must fail. Six kinds read no
//! seed (`estimate`, `nmp_result`, `q_tt_cut`, `gravity`, `null_bound`,
//! `draw_score`) and have none; `spread` reads only the per-thread offsets,
//! which a one-thread trace multiplies by zero.
//!
//! The seed-coverage pass changes every seed in turn by a quarter of its
//! value, at least one, up and then down, and lists each seed whose change
//! moves no decision or score of any matching record; the list must hold
//! only the seeds named with their reason.
//!
//! The picker's two kinds score a move from its recorded components; the
//! components themselves are table reads.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;

use crate::score::{TB_WIN_SCORE, VALUE_NONE};
use crate::tt::Bound;

use super::super::history::apply_bonus;
use super::rules::{self, SEED_FIELDS, Seeds};

/// A kind's records must come from at least this many positions, and
/// number at least this many, for its zero mismatches to count.
const MIN_POSITIONS: usize = 200;
const MIN_RECORDS: usize = 2_000;
/// The floor for a kind written only when a rare event happens: a ProbCut
/// cutoff, a null search, a completed iteration.
const MIN_RARE_RECORDS: usize = 200;
const RARE: &[&str] = &["null_bound", "probcut_value", "score_held"];

/// A kind's exceptions, decisions within a unit of their threshold, stay
/// below this share of its records.
const MAX_EXCEPTION_SHARE: f64 = 0.02;

/// The kinds the rules cover, each held to zero mismatches.
const COVERED: &[&str] = &[
    "bad_noisy",
    "corr_update",
    "draw_score",
    "estimate",
    "eval_bonus",
    "eval_bonus_gate",
    "fail_low_reward",
    "futility",
    "gravity",
    "hindsight",
    "history_prune",
    "ldse",
    "lmp",
    "lmr",
    "malus_fade",
    "nmp_gate",
    "nmp_r",
    "nmp_result",
    "null_bound",
    "optimism",
    "pick_noisy",
    "pick_quiet",
    "pick_threshold",
    "probcut_depth",
    "probcut_gate",
    "probcut_value",
    "q_cutoff_bonus",
    "q_lmp",
    "q_see",
    "q_stand_pat",
    "q_tail",
    "q_tt_cut",
    "razor",
    "research",
    "rfp",
    "score_held",
    "see_margin",
    "singular_gate",
    "singular_result",
    "spread",
    "tt_cut",
    "tt_cut_bonus",
    "tt_cut_return",
    "unreduced",
    "updates",
    "window",
    "wrapper",
];

/// One decision record.
struct Record<'a> {
    kind: &'a str,
    fields: Vec<(&'a str, i64)>,
}

impl Record<'_> {
    fn parse(line: &str) -> Option<Record<'_>> {
        let mut words = line.split_ascii_whitespace();
        let kind = words.next()?;
        let fields = words
            .map(|word| {
                let (name, value) = word.split_once('=')?;
                Some((name, value.parse().ok()?))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Record { kind, fields })
    }

    fn has(&self, name: &str) -> bool {
        self.fields.iter().any(|(field, _)| *field == name)
    }

    fn int(&self, name: &str) -> i32 {
        let value = self
            .fields
            .iter()
            .find(|(field, _)| *field == name)
            .map(|&(_, value)| value)
            .unwrap_or_else(|| panic!("a {} record has no {name}", self.kind));
        i32::try_from(value).unwrap_or_else(|_| panic!("{name}={value} is out of range"))
    }

    /// A field wider than a score: a node count.
    fn count(&self, name: &str) -> u64 {
        let value = self
            .fields
            .iter()
            .find(|(field, _)| *field == name)
            .map(|&(_, value)| value)
            .unwrap_or_else(|| panic!("a {} record has no {name}", self.kind));
        u64::try_from(value).unwrap_or_else(|_| panic!("{name}={value} is negative"))
    }

    /// A field this kind may not record, as zero.
    fn int_or_zero(&self, name: &str) -> i32 {
        if self.has(name) { self.int(name) } else { 0 }
    }

    fn flag(&self, name: &str) -> bool {
        self.int(name) != 0
    }

    /// The donor's bound code: none, exact, lower, upper.
    fn bound(&self, name: &str) -> Option<Bound> {
        match self.int(name) {
            0 => None,
            1 => Some(Bound::Exact),
            2 => Some(Bound::Lower),
            3 => Some(Bound::Upper),
            code => panic!("bound code {code}"),
        }
    }
}

/// How an output is held to its record.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Class {
    /// What the search does with the result: a cut, a depth, an extension,
    /// an ordering score, a history amount read from depth alone.
    Decision,
    /// A score or an amount formed from scores.
    Quantity,
    /// A value only its kind's decision reads: a margin, a reduction in
    /// 1024ths, a move-count limit.
    Intermediate,
}

/// One output a rule formed, beside the value its record holds.
#[derive(Copy, Clone, Debug)]
struct Out {
    class: Class,
    rule: i64,
    record: i64,
}

/// The outputs a record's rule formed.
#[derive(Default)]
struct Outs(Vec<Out>);

impl Outs {
    fn push(&mut self, class: Class, rule: impl Into<i64>, record: impl Into<i64>) {
        self.0.push(Out {
            class,
            rule: rule.into(),
            record: record.into(),
        });
    }

    fn decision(&mut self, rule: impl Into<i64>, record: impl Into<i64>) {
        self.push(Class::Decision, rule, record);
    }

    fn quantity(&mut self, rule: impl Into<i64>, record: impl Into<i64>) {
        self.push(Class::Quantity, rule, record);
    }

    fn intermediate(&mut self, rule: impl Into<i64>, record: impl Into<i64>) {
        self.push(Class::Intermediate, rule, record);
    }
}

/// How a record compared with its rule.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Verdict {
    Match,
    /// A decision that differs only within a unit of its threshold.
    Exception,
    /// A decision on a decisive score, which an evaluation margin separates
    /// from another: the fold scales the margin, and a distance has no Rarog
    /// value to keep it against.
    Distance,
    Mismatch,
}

fn prune_input(r: &Record<'_>) -> rules::MovePruneInput {
    rules::MovePruneInput {
        in_check: r.int_or_zero("in_check") != 0,
        direct_check: r.int_or_zero("direct_check") != 0,
        quiet: r.int_or_zero("quiet") != 0,
        depth: r.int_or_zero("depth"),
        history: r.int_or_zero("history"),
        eval: r.int_or_zero("eval"),
        alpha: r.int_or_zero("alpha"),
        beta: r.int_or_zero("beta"),
        improvement: r.int_or_zero("improvement"),
        correction: r.int_or_zero("correction"),
        move_count: r.int_or_zero("move_count"),
        bad_stage: r.int_or_zero("bad_stage") != 0,
    }
}

fn reduction_input(r: &Record<'_>, lmr: bool) -> rules::ReductionInput {
    // The unreduced record carries whether a score was stored, not the
    // score; its formula reads nothing else of it.
    let tt_score = if lmr {
        r.int("tt_score")
    } else if r.flag("tt_valid") {
        0
    } else {
        VALUE_NONE
    };
    rules::ReductionInput {
        depth: r.int("depth"),
        improvement: r.int("improvement"),
        correction: r.int("correction"),
        exact: r.int_or_zero("exact") != 0,
        tt_score,
        tt_depth: r.int("tt_depth"),
        alpha: r.int_or_zero("alpha"),
        beta: r.int_or_zero("beta"),
        quiet: r.flag("quiet"),
        history: r.int("history"),
        estimate: r.int_or_zero("estimate"),
        since_critical: r.int_or_zero("since_critical"),
        pv: r.flag("pv"),
        root_delta: if lmr { r.int("root_delta") } else { 1 },
        laterality: r.int_or_zero("laterality"),
        tt_pv: r.flag("tt_pv"),
        cut_node: r.flag("cut_node"),
        tt_move_null: r.flag("tt_null"),
        gives_check: r.int_or_zero("gives_check") != 0,
        child_cutoffs: r.int("child_cutoffs"),
        tt_move_score: r.int("tt_move_score"),
        singular_score: r.int("singular_score"),
        parent_reduction: r.int("parent_reduction"),
        spread: r.int("spread"),
        is_tt_move: r.int_or_zero("is_tt_move") != 0,
    }
}

/// The outputs the rule forms for the record; `None` for a kind no rule
/// covers.
fn outputs(s: &Seeds, r: &Record<'_>) -> Option<Outs> {
    let mut o = Outs::default();
    let fire = r.flag("fire");
    match r.kind {
        "wrapper" => {
            let input = rules::WrapperInput {
                raw: r.int("raw"),
                material: r.int("material"),
                optimism: r.int("optimism"),
                fifty: r.int("fifty"),
                correction: r.int("correction"),
            };
            o.quantity(rules::wrapped_eval(s, &input), r.int("eval"));
        }
        "optimism" => o.quantity(rules::optimism(s, r.int("best_avg")), r.int("optimism")),
        "estimate" => {
            let input = rules::EstimateInput {
                in_check: r.flag("in_check"),
                excluded: r.flag("excluded"),
                tt_score: r.int("tt_score"),
                bound: r.bound("bound"),
                eval: r.int("eval"),
            };
            o.quantity(rules::estimated_score(&input), r.int("estimate"));
        }
        "eval_bonus" => o.quantity(
            rules::eval_difference_bonus(s, r.int("eval"), r.int("parent_eval")),
            r.int("bonus"),
        ),
        "tt_cut" => {
            let input = rules::TtCutInput {
                pv: r.flag("pv"),
                excluded: r.flag("excluded"),
                tt_depth: r.int("tt_depth"),
                depth: r.int("depth"),
                tt_score: r.int("tt_score"),
                alpha: r.int("alpha"),
                beta: r.int("beta"),
                bound: r.bound("bound"),
                cut_node: r.flag("cut_node"),
            };
            o.decision(rules::tt_cuts(s, &input), fire);
        }
        "hindsight" => o.decision(
            rules::hindsight_depth(
                s,
                r.int("depth_in"),
                r.int("reduction"),
                r.int("eval_delta"),
                r.flag("tt_pv"),
            ),
            r.int("depth"),
        ),
        "razor" => {
            let input = rules::RazorInput {
                pv: r.flag("pv"),
                in_check: r.flag("in_check"),
                estimate: r.int("estimate"),
                alpha: r.int("alpha"),
                depth: r.int("depth"),
                tt_quiet: r.flag("tt_quiet"),
                bound: r.bound("bound"),
            };
            o.decision(rules::razors(s, &input), fire);
        }
        "rfp" => {
            let input = rules::RfpInput {
                tt_pv: r.flag("tt_pv"),
                in_check: r.flag("in_check"),
                excluded: r.flag("excluded"),
                estimate: r.int("estimate"),
                beta: r.int("beta"),
                depth: r.int("depth"),
                improvement: r.int("improvement"),
                correction: r.int("correction"),
                unthreatened: r.flag("unthreatened"),
            };
            o.intermediate(rules::rfp_margin(s, &input), r.int("margin"));
            o.decision(rules::rfp_cuts(s, &input), fire);
            o.quantity(
                rules::rfp_value(s, input.estimate, input.beta),
                r.int("value"),
            );
        }
        "nmp_gate" => {
            let input = rules::NullGateInput {
                cut_node: r.flag("cut_node"),
                in_check: r.flag("in_check"),
                excluded: r.flag("excluded"),
                singular: r.flag("singular"),
                estimate: r.int("estimate"),
                beta: r.int("beta"),
                depth: r.int("depth"),
                tt_pv: r.flag("tt_pv"),
                improvement: r.int("improvement"),
                child_cutoffs: r.int("child_cutoffs"),
                ply: r.int("ply"),
                nmp_min_ply: r.int("nmp_min_ply"),
                material: r.int("material"),
                tt_capture_refutes: r.flag("tt_capture_refutes"),
            };
            o.intermediate(rules::nmp_margin(s, &input), r.int("margin"));
            o.decision(rules::nmp_gate(s, &input), fire);
        }
        "nmp_r" => o.decision(
            rules::nmp_reduction(
                s,
                r.flag("improving"),
                r.int("depth"),
                r.int("estimate"),
                r.int("beta"),
            ),
            r.int("r"),
        ),
        "nmp_result" => {
            let outcome = rules::null_outcome(
                s,
                r.int("score"),
                r.int("null_beta"),
                r.int("beta"),
                r.int("depth"),
                r.int("r"),
                r.int("nmp_min_ply"),
            );
            let (code, verify_depth) = match outcome {
                rules::NullOutcome::Fail => (0, None),
                rules::NullOutcome::Cut => (1, None),
                rules::NullOutcome::Verify(depth) => (2, Some(depth)),
            };
            let recorded = r.int("outcome");
            assert!((0..=2).contains(&recorded), "null outcome {recorded}");
            o.decision(code, recorded);
            if recorded == 2 {
                o.decision(verify_depth.unwrap_or(-1), r.int("verify_depth"));
            }
            o.decision(outcome != rules::NullOutcome::Fail, fire);
        }
        "probcut_gate" => {
            let probcut_beta = rules::probcut_beta(s, r.int("beta"), r.flag("improving"));
            let input = rules::ProbcutGateInput {
                cut_node: r.flag("cut_node"),
                beta: r.int("beta"),
                probcut_beta,
                tt_score: r.int("tt_score"),
                eval: r.int("eval"),
                tt_quiet: r.flag("tt_quiet"),
            };
            o.quantity(probcut_beta, r.int("probcut_beta"));
            o.decision(rules::probcut_gate(&input), fire);
        }
        "probcut_depth" => {
            let score = r.int("score");
            let probcut_beta = r.int("probcut_beta");
            let d =
                rules::probcut_depth(s, r.int("depth"), r.flag("improving"), score, probcut_beta);
            o.decision(d.base, r.int("base_depth"));
            o.decision(d.depth, r.int("probcut_depth"));
            o.quantity(d.adjusted_beta, r.int("adjusted_beta"));
            o.decision(score >= probcut_beta && d.depth > 0, fire);
        }
        "singular_gate" => o.decision(
            rules::singular_candidate(
                s,
                r.int("depth"),
                r.flag("tt_pv"),
                r.int("tt_depth"),
                r.bound("bound"),
                r.int("tt_score"),
            ),
            fire,
        ),
        "singular_result" => singular_outputs(s, r, &mut o),
        "ldse" => o.decision(
            rules::ldse_extends(
                s,
                r.flag("singular_searched"),
                r.int("depth"),
                r.flag("in_check"),
                r.flag("cut_node"),
                r.int("estimate"),
                r.int("alpha"),
            ),
            fire,
        ),
        "lmp" => {
            let input = prune_input(r);
            o.intermediate(rules::lmp_limit(s, &input), r.int("limit"));
            o.decision(rules::lmp_prunes(s, &input), fire);
        }
        "futility" => {
            let input = prune_input(r);
            o.quantity(rules::futility_value(s, &input), r.int("value"));
            o.decision(rules::futility_prunes(s, &input), fire);
        }
        "bad_noisy" => {
            let input = prune_input(r);
            o.quantity(rules::bad_noisy_value(s, &input), r.int("value"));
            o.decision(rules::bad_noisy_prunes(s, &input), fire);
        }
        "history_prune" => o.decision(rules::history_prunes(s, &prune_input(r)), fire),
        "see_margin" => o.decision(
            rules::see_threshold(s, r.flag("quiet"), r.int("depth"), r.int("history")),
            r.int("threshold"),
        ),
        "lmr" => {
            let input = reduction_input(r, true);
            let reduction = rules::late_move_reduction(s, &input);
            o.intermediate(reduction, r.int("reduction"));
            o.decision(
                rules::lmr_depth(r.int("new_depth"), reduction, input.pv),
                r.int("reduced_depth"),
            );
        }
        "research" => {
            let new_depth = rules::research_depth(
                s,
                r.flag("root"),
                r.int("score"),
                r.int("best_score"),
                r.int("new_depth_in"),
            );
            o.decision(new_depth, r.int("new_depth"));
            o.decision(new_depth > r.int("reduced_depth"), fire);
        }
        "unreduced" => {
            let input = reduction_input(r, false);
            let reduction = rules::unreduced_reduction(s, &input);
            let new_depth = r.int("new_depth");
            let reduced = rules::unreduced_depth(s, new_depth, reduction);
            o.intermediate(reduction, r.int("reduction"));
            o.decision(reduced, r.int("reduced_depth"));
            o.decision(reduced < new_depth, fire);
        }
        "updates" => {
            let a = rules::update_amounts(
                s,
                r.int("depth"),
                r.flag("cut_node"),
                r.int("noisy"),
                r.int("quiet"),
            );
            o.decision(a.noisy_bonus, r.int("noisy_bonus"));
            o.decision(a.noisy_malus, r.int("noisy_malus"));
            o.decision(a.quiet_bonus, r.int("quiet_bonus"));
            o.decision(a.quiet_malus, r.int("quiet_malus"));
            o.decision(a.cont_bonus, r.int("cont_bonus"));
            o.decision(a.cont_malus, r.int("cont_malus"));
            o.decision(a.parent_malus, r.int("parent_malus"));
            o.decision(a.research_bonus, r.int("research_bonus"));
        }
        "fail_low_reward" => {
            if r.flag("quiet") {
                let reward = rules::fail_low_reward(
                    s,
                    &rules::RewardInput {
                        parent_move_count: r.int("parent_move_count"),
                        parent_tt: r.flag("parent_tt"),
                        in_check: r.flag("in_check"),
                        best_score: r.int("best_score"),
                        eval: r.int("eval"),
                        parent_eval: r.int("parent_eval"),
                        depth: r.int("depth"),
                    },
                );
                o.decision(reward.factor, r.int("factor"));
                o.decision(reward.bonus, r.int("bonus"));
                o.decision(reward.grandparent, r.int("grandparent"));
            } else {
                o.decision(
                    rules::noisy_fail_low_reward(s, r.int("depth")),
                    r.int("bonus"),
                );
            }
        }
        "corr_update" => {
            let update = rules::correction_update(
                s,
                &rules::CorrectionInput {
                    in_check: r.flag("in_check"),
                    noisy_best: r.flag("noisy_best"),
                    bound: r.bound("bound").expect("a node's result has a bound"),
                    best_score: r.int("best_score"),
                    eval: r.int("eval"),
                    depth: r.int("depth"),
                },
            );
            // The donor writes the amount for every node; it is formed only
            // when the update fires, and only then compared.
            o.decision(update.is_some(), fire);
            if let Some(bonus) = update {
                o.quantity(bonus, r.int("bonus"));
            }
        }
        "gravity" => {
            // The history tables' own update, which every kernel shares.
            let mut entry = i16::try_from(r.int("entry")).expect("a table entry");
            apply_bonus(&mut entry, r.int("bonus"), r.int("max"));
            o.quantity(entry, r.int("result"));
        }
        "q_tt_cut" => o.decision(
            rules::q_tt_cuts(
                r.flag("pv"),
                r.int("tt_score"),
                r.bound("bound"),
                r.int("alpha"),
                r.int("beta"),
            ),
            fire,
        ),
        "q_stand_pat" => {
            let best_score = r.int("best_score");
            let beta = r.int("beta");
            o.quantity(
                rules::q_stand_pat_value(s, best_score, beta),
                r.int("value"),
            );
            o.decision(best_score >= beta, fire);
            o.quantity(
                rules::stand_pat(
                    r.flag("in_check"),
                    r.flag("pv"),
                    r.int("eval"),
                    r.int("tt_score"),
                    r.bound("bound"),
                ),
                best_score,
            );
        }
        "q_lmp" => o.decision(
            rules::q_stops(s, r.int("move_count"), r.flag("direct_check")),
            fire,
        ),
        "q_see" => o.quantity(
            rules::q_see_threshold(
                s,
                r.int("alpha"),
                r.int("eval"),
                r.int("correction"),
                r.int("history"),
            ),
            r.int("threshold"),
        ),
        "q_tail" => {
            let best_score = r.int("best_score");
            let beta = r.int("beta");
            o.quantity(rules::q_tail_value(s, best_score, beta), r.int("value"));
            o.decision(best_score >= beta, fire);
        }
        "window" => window_outputs(s, r, &mut o),
        "pick_noisy" => o.decision(
            rules::noisy_order(
                s,
                &rules::NoisyOrderInput {
                    victim: r.int("victim"),
                    history: r.int("history"),
                    queen_promotion: r.flag("queen_promotion"),
                    in_check: r.flag("in_check"),
                    piece: r.int("piece"),
                },
            ),
            r.int("score"),
        ),
        "pick_quiet" => o.decision(
            rules::quiet_order(
                s,
                &rules::QuietOrderInput {
                    quiet: r.int("quiet"),
                    pawn: r.int("pawn"),
                    cont: [
                        r.int("cont1"),
                        r.int("cont2"),
                        r.int("cont4"),
                        r.int("cont6"),
                    ],
                    piece: usize::try_from(r.int("piece")).expect("a piece index"),
                    escape: r.flag("escape"),
                    check_square: r.flag("check_square"),
                    threatened_to: r.flag("threatened_to"),
                    offense: r.flag("offense"),
                    wall: r.flag("wall"),
                },
            ),
            r.int("score"),
        ),
        "tt_cut_bonus" => {
            let rewards = rules::tt_cut_rewards(
                s,
                &rules::TtCutRewardInput {
                    tt_quiet: r.flag("tt_quiet"),
                    tt_score: r.int("tt_score"),
                    beta: r.int("beta"),
                    parent_move_count: r.int("parent_move_count"),
                    depth: r.int("depth"),
                },
            );
            // The donor writes the amounts at every cutoff; they are
            // formed only when the reward fires, and only then compared.
            o.decision(rewards.is_some(), fire);
            if let Some((quiet, cont)) = rewards {
                o.decision(quiet, r.int("quiet_bonus"));
                o.decision(cont, r.int("cont_bonus"));
            }
        }
        "tt_cut_return" => o.decision(rules::tt_cut_returns(s, r.int("fifty")), fire),
        "eval_bonus_gate" => o.decision(
            rules::eval_bonus_applies(
                s,
                &rules::EvalBonusGate {
                    // The donor writes this record off the root only.
                    root: false,
                    in_check: r.flag("in_check"),
                    excluded: r.flag("excluded"),
                    parent_quiet: r.flag("parent_quiet"),
                    parent_eval_valid: r.flag("parent_eval_valid"),
                    depth: r.int("depth"),
                    has_entry: r.flag("has_entry"),
                },
            ),
            fire,
        ),
        "null_bound" => {
            let bound = rules::null_bound(
                r.int("depth"),
                r.int("beta"),
                r.int("tt_score"),
                r.bound("bound"),
                r.int("tt_depth"),
            );
            o.quantity(bound, r.int("null_beta"));
            o.decision(bound != r.int("beta"), fire);
        }
        "probcut_value" => o.quantity(
            rules::probcut_value(s, r.int("score"), r.int("beta")),
            r.int("value"),
        ),
        "spread" => {
            let term = match r.int("term") {
                0 => rules::SpreadTerm::Lmr,
                1 => rules::SpreadTerm::Unreduced,
                term => panic!("spread term {term}"),
            };
            o.decision(
                rules::spread(s, term, r.count("nodes"), r.count("thread")),
                r.int("spread"),
            );
        }
        "malus_fade" => o.decision(rules::malus_fade(s, r.int("index")), r.int("scale")),
        "q_cutoff_bonus" => {
            let reward =
                rules::q_cutoff_reward(s, r.int("best_score"), r.int("beta"), r.flag("noisy_best"));
            o.decision(reward.is_some(), fire);
            if let Some(bonus) = reward {
                o.decision(bonus, r.int("bonus"));
            }
        }
        "score_held" => o.decision(rules::score_held(s, r.int("score"), r.int("average")), fire),
        "pick_threshold" => o.decision(
            rules::good_noisy_threshold(
                s,
                r.int("score"),
                r.flag("tt_quiet"),
                r.int("good_so_far"),
            ),
            r.int("threshold"),
        ),
        "draw_score" => o.quantity(rules::draw_score(r.count("nodes")), r.int("value")),
        _ => return None,
    }
    Some(o)
}

fn singular_outputs(s: &Seeds, r: &Record<'_>, o: &mut Outs) {
    let singular_score = r.int("singular_score");
    let singular_beta = r.int("singular_beta");
    let beta = r.int("beta");
    if r.int("outcome") == 1 {
        // A multi-cut record carries what that branch reads.
        let input = rules::SingularInput {
            singular_score,
            singular_beta,
            beta,
            tt_score: VALUE_NONE,
            pv: false,
            tt_was_pv: false,
            tt_quiet: false,
            correction: 0,
            cut_node: false,
            searched_a_move: false,
        };
        let multicut = match rules::singular_outcome(s, &input) {
            rules::SingularOutcome::MultiCut(value) => Some(value),
            _ => None,
        };
        o.decision(multicut.is_some(), true);
        if let Some(value) = multicut {
            o.quantity(value, r.int("value"));
        }
        return;
    }
    let depth = r.int("depth");
    let pv = r.flag("pv");
    let tt_pv = r.flag("tt_pv");
    let tt_score = r.int("tt_score");
    let margin = rules::singular_margin(s, depth, r.flag("exact"), tt_pv && !pv);
    o.intermediate(margin, r.int("margin"));
    o.quantity(tt_score - margin, singular_beta);
    o.decision((depth - 1) / 2, r.int("singular_depth"));
    let input = rules::SingularInput {
        singular_score,
        singular_beta,
        beta,
        tt_score,
        pv,
        tt_was_pv: r.flag("tt_was_pv"),
        tt_quiet: r.flag("tt_quiet"),
        correction: r.int("correction"),
        cut_node: r.flag("cut_node"),
        searched_a_move: r.flag("searched"),
    };
    let (extension, cleared, multicut) = match rules::singular_outcome(s, &input) {
        rules::SingularOutcome::None => (0, false, false),
        rules::SingularOutcome::Extend(plies) => (plies, false, false),
        rules::SingularOutcome::Demote => (0, true, false),
        rules::SingularOutcome::MultiCut(_) => (0, false, true),
    };
    o.decision(multicut, false);
    o.decision(extension, r.int("extension"));
    o.decision(cleared, r.flag("tt_cleared"));
    o.decision(extension != 0, r.flag("fire"));
}

fn window_outputs(s: &Seeds, r: &Record<'_>, o: &mut Outs) {
    let window = match r.int("phase") {
        0 => {
            let delta = rules::window_delta(
                s,
                r.int("eval_stability"),
                r.int("pv_stability"),
                r.int("average"),
            );
            rules::open_window(r.int("average"), delta)
        }
        phase => {
            let before = rules::Window {
                alpha: r.int("alpha_in"),
                beta: r.int("beta_in"),
                delta: r.int("delta_in"),
                reduction: if phase == 2 { r.int("reduction_in") } else { 0 },
            };
            if phase == 1 {
                rules::window_fail_low(s, before, r.int("score"))
            } else {
                let after = rules::window_fail_high(s, before, r.int("score"));
                o.decision(after.reduction, r.int("reduction"));
                after
            }
        }
    };
    o.quantity(window.alpha, r.int("alpha"));
    o.quantity(window.beta, r.int("beta"));
    o.quantity(window.delta, r.int("delta"));
}

/// The donor's evaluation units per 1024 of Rarog's: the trace is in the
/// donor's units, the rules in Rarog's.
const DONOR_UNITS: i64 = 2536;

/// The donor's saturation of the continuation and keyed correction tables,
/// which tell a correction table's update from a history table's.
const DONOR_CORR_CONT_MAX: i64 = 16_418;
const DONOR_CORR_KEYED_MAX: i64 = 14_605;

/// The fields of each kind in evaluation units: its inputs, then its
/// outputs. Every other field is a ply, a count, a flag, a history amount or
/// an exchange value, which the fold leaves as it is.
fn eval_fields(kind: &str) -> (&'static [&'static str], &'static [&'static str]) {
    match kind {
        "wrapper" => (&["raw", "optimism", "correction"], &["eval"]),
        "optimism" => (&["best_avg"], &["optimism"]),
        "estimate" => (&["tt_score", "eval"], &["estimate"]),
        "eval_bonus" => (&["eval", "parent_eval"], &[]),
        "tt_cut" | "q_tt_cut" => (&["tt_score", "alpha", "beta"], &[]),
        "hindsight" => (&["eval_delta"], &[]),
        "razor" | "ldse" => (&["estimate", "alpha"], &[]),
        "rfp" => (
            &["estimate", "beta", "improvement", "correction"],
            &["margin", "value"],
        ),
        "nmp_gate" => (&["estimate", "beta", "improvement"], &["margin"]),
        "nmp_r" => (&["estimate", "beta"], &[]),
        "nmp_result" => (&["score", "null_beta", "beta"], &[]),
        "probcut_gate" => (&["beta", "tt_score", "eval"], &["probcut_beta"]),
        "probcut_depth" => (&["score", "probcut_beta"], &["adjusted_beta"]),
        "singular_gate" => (&["tt_score"], &[]),
        "singular_result" => (
            &[
                "singular_score",
                "singular_beta",
                "beta",
                "tt_score",
                "correction",
            ],
            &["margin", "value"],
        ),
        "lmp" | "futility" | "bad_noisy" | "history_prune" => (
            &["eval", "alpha", "beta", "improvement", "correction"],
            &["value"],
        ),
        "lmr" => (
            &[
                "improvement",
                "correction",
                "tt_score",
                "alpha",
                "beta",
                "estimate",
                "root_delta",
                "tt_move_score",
                "singular_score",
            ],
            &[],
        ),
        "research" => (&["score", "best_score"], &[]),
        "unreduced" => (
            &[
                "improvement",
                "correction",
                "tt_move_score",
                "singular_score",
            ],
            &[],
        ),
        "fail_low_reward" => (&["best_score", "eval", "parent_eval"], &[]),
        "corr_update" => (&["best_score", "eval"], &["bonus"]),
        "q_stand_pat" => (&["best_score", "beta", "eval", "tt_score"], &["value"]),
        "q_see" => (&["alpha", "eval", "correction"], &[]),
        "q_tail" => (&["best_score", "beta"], &["value"]),
        "probcut_value" => (&["score", "beta"], &["value"]),
        "window" => (
            &["average", "alpha_in", "beta_in", "delta_in", "score"],
            &["alpha", "beta", "delta"],
        ),
        "tt_cut_bonus" => (&["tt_score", "beta"], &[]),
        "null_bound" => (&["beta", "tt_score"], &["null_beta"]),
        "q_cutoff_bonus" => (&["best_score", "beta"], &[]),
        "score_held" => (&["score", "average"], &[]),
        "draw_score" => (&[], &["value"]),
        _ => (&[], &[]),
    }
}

/// A score the fold moves: neither absent nor decisive.
fn is_evaluation(value: i64) -> bool {
    value != i64::from(VALUE_NONE) && value.abs() < i64::from(TB_WIN_SCORE)
}

/// A value in the donor's evaluation units in Rarog's, rounded to the
/// nearest; an absent or decisive score passes unchanged.
fn to_rarog(value: i64) -> i64 {
    if !is_evaluation(value) {
        return value;
    }
    let twice = 2 * value * 1024;
    (twice + twice.signum() * DONOR_UNITS) / (2 * DONOR_UNITS)
}

impl<'a> Record<'a> {
    /// A table update of a correction table, whose entries are evaluation
    /// units, not of a history table.
    fn is_correction_update(&self) -> bool {
        self.kind == "gravity"
            && self.fields.iter().any(|&(name, value)| {
                name == "max" && (value == DONOR_CORR_CONT_MAX || value == DONOR_CORR_KEYED_MAX)
            })
    }

    /// The record in Rarog's units, each evaluation input moved by its
    /// `jitter` step. A correction table's update also takes the folded
    /// table's saturation.
    fn folded(&self, s: &Seeds, jitter: &[(&str, i64)]) -> Record<'a> {
        let (inputs, outputs) = eval_fields(self.kind);
        let correction_table = self.is_correction_update();
        let fields = self
            .fields
            .iter()
            .map(|&(name, value)| {
                let value = if correction_table {
                    match name {
                        "max" if value == DONOR_CORR_CONT_MAX => i64::from(s.corr_cont_max),
                        "max" => i64::from(s.corr_keyed_max),
                        "entry" | "bonus" | "result" => to_rarog(value),
                        _ => value,
                    }
                } else if inputs.contains(&name) || outputs.contains(&name) {
                    to_rarog(value)
                } else {
                    value
                };
                let step = jitter
                    .iter()
                    .find(|&&(field, _)| field == name)
                    .map_or(0, |&(_, step)| step);
                (name, value + step)
            })
            .collect();
        Record {
            kind: self.kind,
            fields,
        }
    }

    /// The evaluation inputs a jitter may move: those this record holds as
    /// a score the fold moves.
    fn jitter_fields(&self) -> Vec<&'static str> {
        let names: &[&'static str] = if self.is_correction_update() {
            &["entry", "bonus"]
        } else {
            eval_fields(self.kind).0
        };
        names
            .iter()
            .copied()
            .filter(|name| {
                self.fields
                    .iter()
                    .any(|&(field, value)| field == *name && is_evaluation(value))
            })
            .collect()
    }
}

/// Every move of the inputs by one unit: none, each alone either way, all
/// together either way, and each pair either way.
fn jitters(fields: &[&'static str]) -> Vec<Vec<(&'static str, i64)>> {
    let mut all = vec![Vec::new()];
    for step in [1, -1] {
        for &field in fields {
            all.push(vec![(field, step)]);
        }
        if fields.len() > 1 {
            all.push(fields.iter().map(|&field| (field, step)).collect());
        }
    }
    for (a, &first) in fields.iter().enumerate() {
        for &second in &fields[a + 1..] {
            for (x, y) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                all.push(vec![(first, x), (second, y)]);
            }
        }
    }
    all
}

/// The decisions a set of outputs takes, in order.
fn decisions(o: &Outs) -> impl Iterator<Item = &Out> {
    o.0.iter().filter(|out| out.class == Class::Decision)
}

/// Whether the record's scores convert: every score it holds in evaluation
/// units is an evaluation. One that is absent or decisive, beside an
/// evaluation offset or inside a score formed from one, has no value in
/// Rarog's units, so such a record is held to its decisions only.
fn scores_convert(r: &Record<'_>) -> bool {
    if r.is_correction_update() {
        return true;
    }
    let (inputs, outputs) = eval_fields(r.kind);
    r.fields
        .iter()
        .filter(|(name, _)| inputs.contains(name) || outputs.contains(name))
        .all(|&(_, value)| is_evaluation(value))
}

/// The rule's outputs on the record in Rarog's units, each evaluation input
/// moved by its `jitter` step, without the scores when the record's scores
/// do not convert.
fn comparable_outputs(s: &Seeds, r: &Record<'_>, jitter: &[(&str, i64)]) -> Option<Outs> {
    let mut o = outputs(s, &r.folded(s, jitter))?;
    if !scores_convert(r) {
        o.0.retain(|out| out.class != Class::Quantity);
    }
    Some(o)
}

/// Whether the record holds a decisive score in a field the fold converts.
/// A decision on one, which an evaluation margin separates from another
/// score, cannot be kept: the fold scales the margin and leaves the distance.
/// An absent score passes the fold unchanged and every rule tests for it, so
/// it excuses no decision.
fn holds_distance(r: &Record<'_>) -> bool {
    if r.is_correction_update() {
        return false;
    }
    let (inputs, outputs) = eval_fields(r.kind);
    r.fields
        .iter()
        .filter(|(name, _)| inputs.contains(name) || outputs.contains(name))
        .any(|&(_, value)| value != i64::from(VALUE_NONE) && !is_evaluation(value))
}

/// The fold's verdict on a record. Its rule, run on the record's inputs in
/// Rarog's units, must take every decision the donor took, and form every
/// score within the rounding of its inputs: within a unit, plus as far as
/// the score moves when any input moves by a unit. A decision that differs
/// lies within a unit of its threshold when moving the inputs by a unit
/// reaches the donor's value or both sides of it, and is an exception, not
/// a mismatch.
fn fold_verdict(s: &Seeds, r: &Record<'_>) -> Option<(Verdict, Outs)> {
    let zero = comparable_outputs(s, r, &[])?;
    let decides = |o: &Outs| decisions(o).all(|out| out.rule == out.record);
    let near = |o: &Outs| {
        o.0.iter()
            .filter(|out| out.class == Class::Quantity)
            .all(|out| (out.rule - out.record).abs() <= 1)
    };
    if decides(&zero) && near(&zero) {
        return Some((Verdict::Match, zero));
    }
    let shape: Vec<Class> = zero.0.iter().map(|out| out.class).collect();
    // The outputs as each input moves by a unit.
    let all_moved: Vec<Outs> = jitters(&r.jitter_fields())
        .iter()
        .filter_map(|jitter| comparable_outputs(s, r, jitter))
        .collect();
    // Those shaped as the unmoved ones.
    let moved: Vec<&Outs> = all_moved
        .iter()
        .filter(|o| o.0.iter().map(|out| out.class).eq(shape.iter().copied()))
        .collect();
    let at = |o: &Outs, index: usize| o.0[index].rule;
    if decides(&zero) {
        // Every decision holds: each score within the unit moves' reach.
        let holding: Vec<&Outs> = moved.iter().copied().filter(|o| decides(o)).collect();
        let within = (0..shape.len())
            .filter(|&index| shape[index] == Class::Quantity)
            .all(|index| {
                let reach = holding
                    .iter()
                    .map(|o| (at(o, index) - at(&zero, index)).abs())
                    .max()
                    .unwrap_or(0);
                (zero.0[index].record - at(&zero, index)).abs() <= reach + 1
            });
        let verdict = if within {
            Verdict::Match
        } else {
            Verdict::Mismatch
        };
        return Some((verdict, zero));
    }
    if holds_distance(r) {
        return Some((Verdict::Distance, zero));
    }
    // A decision against the moved outputs that agree with the unmoved ones
    // up to it: a gate that stops firing drops the amounts it forms, and
    // still brackets its own decision.
    let bracketed = (0..shape.len())
        .filter(|&index| shape[index] == Class::Decision)
        .all(|index| {
            let record = zero.0[index].record;
            let aligned = all_moved.iter().filter(|o| {
                o.0.len() > index
                    && o.0[..=index]
                        .iter()
                        .map(|out| out.class)
                        .eq(shape[..=index].iter().copied())
            });
            let values =
                || std::iter::once(at(&zero, index)).chain(aligned.clone().map(|o| at(o, index)));
            values().min().is_some_and(|low| low <= record)
                && values().max().is_some_and(|high| high >= record)
        });
    let verdict = if bracketed {
        Verdict::Exception
    } else {
        Verdict::Mismatch
    };
    Some((verdict, zero))
}

/// Whether two sets of outputs from one record take the same decisions and
/// form the same scores.
fn same_outputs(a: &Outs, b: &Outs) -> bool {
    a.0.len() == b.0.len()
        && a.0.iter().zip(&b.0).all(|(x, y)| {
            x.class == y.class && (x.class == Class::Intermediate || x.rule == y.rule)
        })
}

/// One planted seed: the kind it must fail and the change.
struct Plant {
    kind: &'static str,
    change: fn(&mut Seeds),
}

const PLANTS: &[Plant] = &[
    Plant {
        kind: "wrapper",
        change: |s| s.wrap_raw_base += 512,
    },
    Plant {
        kind: "optimism",
        change: |s| s.optimism_scale += 8,
    },
    Plant {
        kind: "eval_bonus",
        change: |s| s.eval_bonus_scale += 512,
    },
    Plant {
        kind: "tt_cut",
        change: |s| s.tt_cut_typed_depth += 1,
    },
    Plant {
        kind: "hindsight",
        change: |s| s.hindsight_rise += 16,
    },
    Plant {
        kind: "razor",
        change: |s| s.razor_base_16 += 1_024,
    },
    Plant {
        kind: "rfp",
        change: |s| s.rfp_constant_16 += 64,
    },
    Plant {
        kind: "nmp_gate",
        change: |s| s.nmp_base_16 += 64,
    },
    Plant {
        kind: "nmp_r",
        change: |s| s.nmp_r_base += 256,
    },
    Plant {
        kind: "probcut_gate",
        change: |s| s.probcut_margin += 8,
    },
    Plant {
        kind: "probcut_depth",
        change: |s| s.probcut_depth_per += 32,
    },
    Plant {
        kind: "singular_gate",
        change: |s| s.singular_depth += 1,
    },
    Plant {
        kind: "singular_result",
        change: |s| s.negative_extension += 1,
    },
    Plant {
        kind: "ldse",
        change: |s| s.ldse_margin += 16,
    },
    Plant {
        kind: "lmp",
        change: |s| s.lmp_base += 512,
    },
    Plant {
        kind: "futility",
        change: |s| s.fp_constant_16 += 128,
    },
    Plant {
        kind: "bad_noisy",
        change: |s| s.bnfp_constant_16 += 128,
    },
    Plant {
        kind: "history_prune",
        change: |s| s.hp_slope += 64,
    },
    Plant {
        kind: "see_margin",
        change: |s| s.see_quiet_constant += 1,
    },
    Plant {
        kind: "lmr",
        change: |s| s.lmr_quiet += 512,
    },
    Plant {
        kind: "research",
        change: |s| s.research_deeper_16 += 128,
    },
    Plant {
        kind: "unreduced",
        change: |s| s.fds_quiet += 512,
    },
    Plant {
        kind: "updates",
        change: |s| s.quiet_malus_per += 1,
    },
    Plant {
        kind: "fail_low_reward",
        change: |s| s.reward_base += 1,
    },
    Plant {
        kind: "corr_update",
        change: |s| s.corr_slope += 8,
    },
    Plant {
        kind: "q_stand_pat",
        change: |s| s.q_stand_pat_t += 100,
    },
    Plant {
        kind: "q_lmp",
        change: |s| s.q_move_limit += 1,
    },
    Plant {
        kind: "q_see",
        change: |s| s.q_see_constant += 8,
    },
    Plant {
        kind: "q_tail",
        change: |s| s.q_tail_t += 100,
    },
    Plant {
        kind: "pick_noisy",
        change: |s| s.pick_victim += 1_024,
    },
    Plant {
        kind: "pick_quiet",
        change: |s| s.pick_quiet += 64,
    },
    Plant {
        kind: "window",
        change: |s| s.window_base_1024 += 4_096,
    },
    Plant {
        kind: "tt_cut_bonus",
        change: |s| s.tt_bonus_quiet_slope += 16,
    },
    Plant {
        kind: "tt_cut_return",
        change: |s| s.tt_cut_fifty -= 85,
    },
    Plant {
        kind: "eval_bonus_gate",
        change: |s| s.eval_bonus_depth += 1,
    },
    Plant {
        kind: "probcut_value",
        change: |s| s.probcut_t += 100,
    },
    Plant {
        kind: "malus_fade",
        change: |s| s.malus_fade += 1,
    },
    Plant {
        kind: "q_cutoff_bonus",
        change: |s| s.q_cutoff_bonus += 1,
    },
    Plant {
        kind: "score_held",
        change: |s| s.window_stable_16 += 64,
    },
    Plant {
        kind: "pick_threshold",
        change: |s| s.pick_see_offset += 1,
    },
    // Each converted by the wrong dimension: a margin divided by the units'
    // ratio, a coefficient multiplied by it.
    Plant {
        kind: "futility",
        change: |s| s.fp_constant_16 = 5_032,
    },
    Plant {
        kind: "razor",
        change: |s| s.razor_base_16 = 9_391,
    },
    Plant {
        kind: "lmr",
        change: |s| s.lmr_correction = 1_380,
    },
];

/// Seeds no record can detect, each with its reason.
const UNDETECTABLE: &[(&str, &str)] = &[
    (
        "rfp_min_16",
        "a margin floor below one unit, set against a whole number of units in \
         sixteenths: every floor from 1 to 16 sixteenths takes the same decisions",
    ),
    (
        "nmp_min_16",
        "a margin floor below one unit, set against a whole number of units in \
         sixteenths: every floor from 1 to 16 sixteenths takes the same decisions",
    ),
    (
        "probcut_see_scale",
        "ProbCut's picker bar, which no trace kind records: the picker's exchange \
         test is not a rule the trace samples",
    ),
    (
        "double_new_pv_16",
        "read only at a PV node whose singular search failed low, which the \
         trace samples sparsely (47 records in the 2026-11-17 capture, 11 at a new PV \
         node), none of them within a step of the margin",
    ),
    (
        "double_quiet_16",
        "a quarter moves the double bar by 26 sixteenths, and the closest of the 717 quiet \
         fail-low records in the 2026-11-17 capture stands 41 from it: whole-unit gaps \
         against the bar leave no record nearer",
    ),
    (
        "triple_pv_16",
        "read only at a PV node whose singular search failed low, which the \
         trace samples sparsely (47 records in the 2026-11-17 capture, 11 at a new PV \
         node), none of them within a step of the margin",
    ),
    (
        "triple_new_pv_16",
        "read only at a PV node whose singular search failed low, which the \
         trace samples sparsely (47 records in the 2026-11-17 capture, 11 at a new PV \
         node), none of them within a step of the margin",
    ),
    (
        "lmr_spread_thread",
        "a per-thread offset, times thread 0 in a one-thread trace",
    ),
    (
        "fds_tt_pv_deep",
        "the depth, not the reduction, is compared, and none of the 1,449 records with a deep \
         stored score at a TT-PV node in the 2026-11-17 capture lies within a quarter of \
         it of a depth bar",
    ),
    (
        "fds_spread_thread",
        "a per-thread offset, times thread 0 in a one-thread trace",
    ),
    (
        "corr_keyed_max",
        "the trace records the continuation correction tables' updates and none of the \
         keyed tables'",
    ),
    (
        "pick_escape[0]",
        "a pawn escapes nothing: the donor maps no threat to pawns",
    ),
    (
        "pick_escape[5]",
        "a king escapes nothing: the donor maps no threat to kings",
    ),
    (
        "pawn_history_max",
        "the pawn table's limit acts only through the table's entries, which the \
         trace records as inputs",
    ),
    (
        "exchange_value[0]",
        "the exchange values act through an exchange's result and the board's \
         material, which the trace records as inputs, never in a rule's arithmetic",
    ),
    (
        "exchange_value[1]",
        "the exchange values act through an exchange's result and the board's \
         material, which the trace records as inputs, never in a rule's arithmetic",
    ),
    (
        "exchange_value[2]",
        "the exchange values act through an exchange's result and the board's \
         material, which the trace records as inputs, never in a rule's arithmetic",
    ),
    (
        "exchange_value[3]",
        "the exchange values act through an exchange's result and the board's \
         material, which the trace records as inputs, never in a rule's arithmetic",
    ),
    (
        "exchange_value[4]",
        "the exchange values act through an exchange's result and the board's \
         material, which the trace records as inputs, never in a rule's arithmetic",
    ),
];

#[derive(Default)]
struct Tally {
    records: usize,
    positions: usize,
    exceptions: usize,
    distances: usize,
    mismatches: usize,
    /// The first few records the rule did not reproduce.
    examples: Vec<String>,
}

#[test]
#[ignore = "reads the donor's trace from ignored storage, named by RAROG_DONOR_TRACE"]
fn the_rules_reproduce_the_donors_decisions() {
    let dir = PathBuf::from(
        std::env::var_os("RAROG_DONOR_TRACE").expect("RAROG_DONOR_TRACE names the trace"),
    );
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("the trace directory is readable")
        .map(|entry| entry.expect("a readable entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("trace_") && name.ends_with(".txt"))
        })
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "no trace_*.txt files in {}",
        dir.display()
    );

    let base = Seeds::DONOR;
    let planted: Vec<Seeds> = PLANTS
        .iter()
        .map(|plant| {
            let mut seeds = Seeds::DONOR;
            (plant.change)(&mut seeds);
            assert_ne!(
                seeds,
                Seeds::DONOR,
                "the {} plant changes a seed",
                plant.kind
            );
            seeds
        })
        .collect();
    assert_eq!(
        std::mem::size_of::<Seeds>(),
        std::mem::size_of::<i32>() * SEED_FIELDS.len(),
        "the seed list names every seed scalar"
    );
    // Each seed changed by a quarter of its value, at least one, up and down.
    let perturbed: Vec<[Seeds; 2]> = SEED_FIELDS
        .iter()
        .map(|(name, field)| {
            [1, -1].map(|sign| {
                let mut seeds = Seeds::DONOR;
                let seed = field(&mut seeds);
                *seed += sign * (seed.abs() / 4).max(1);
                assert_ne!(seeds, Seeds::DONOR, "{name} changes");
                seeds
            })
        })
        .collect();
    // The seeds no record has detected yet.
    let mut undetected: Vec<usize> = (0..SEED_FIELDS.len()).collect();
    let mut tallies: BTreeMap<String, Tally> = BTreeMap::new();
    // Mismatches under each plant, by kind.
    let mut plant_tallies: Vec<BTreeMap<String, usize>> = vec![BTreeMap::new(); PLANTS.len()];
    let mut unparsed = 0usize;
    let mut uncovered: BTreeSet<String> = BTreeSet::new();

    for file in &files {
        let text = std::fs::read_to_string(file).expect("a readable trace file");
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for line in text.lines().filter(|line| !line.is_empty()) {
            let Some(record) = Record::parse(line) else {
                unparsed += 1;
                continue;
            };
            let Some((v, outs)) = fold_verdict(&base, &record) else {
                uncovered.insert(record.kind.to_string());
                continue;
            };
            let tally = tallies.entry(record.kind.to_string()).or_default();
            tally.records += 1;
            if seen.insert(record.kind) {
                tally.positions += 1;
            }
            match v {
                Verdict::Match => {}
                Verdict::Exception => tally.exceptions += 1,
                Verdict::Distance => tally.distances += 1,
                Verdict::Mismatch => {
                    tally.mismatches += 1;
                    if tally.examples.len() < 5 {
                        tally.examples.push(line.to_string());
                    }
                }
            }
            // A seed change that moves none of the record's outputs leaves
            // its verdict as it was.
            let moves = |seeds: &Seeds| {
                comparable_outputs(seeds, &record, &[]).is_none_or(|o| !same_outputs(&o, &outs))
            };
            for (seeds, counts) in planted.iter().zip(plant_tallies.iter_mut()) {
                if moves(seeds)
                    && fold_verdict(seeds, &record).is_some_and(|(v, _)| v == Verdict::Mismatch)
                {
                    *counts.entry(record.kind.to_string()).or_default() += 1;
                }
            }
            if v == Verdict::Match {
                undetected.retain(|&index| !perturbed[index].iter().any(&moves));
            }
        }
    }

    let mut report = String::new();
    let _ = writeln!(report, "{} trace files from {}", files.len(), dir.display());
    for (kind, tally) in &tallies {
        let _ = writeln!(
            report,
            "{kind:16} records {:>8} positions {:>4} exceptions {:>6} distances {:>5} mismatches {}",
            tally.records, tally.positions, tally.exceptions, tally.distances, tally.mismatches
        );
        for example in &tally.examples {
            let _ = writeln!(report, "    {example}");
        }
    }
    let _ = writeln!(report, "not covered: {uncovered:?}");
    for (plant, counts) in PLANTS.iter().zip(&plant_tallies) {
        let _ = writeln!(report, "plant {:16} fails {counts:?}", plant.kind);
    }
    let undetected: Vec<&str> = undetected
        .iter()
        .map(|&index| SEED_FIELDS[index].0)
        .collect();
    let _ = writeln!(
        report,
        "seeds no record detects ({} of {}): {undetected:?}",
        undetected.len(),
        SEED_FIELDS.len()
    );
    println!("{report}");

    assert_eq!(unparsed, 0, "every trace line parses\n{report}");
    for kind in COVERED {
        let tally = tallies
            .get(*kind)
            .unwrap_or_else(|| panic!("no {kind} records\n{report}"));
        let floor = if RARE.contains(kind) {
            MIN_RARE_RECORDS
        } else {
            MIN_RECORDS
        };
        assert!(
            tally.records >= floor && tally.positions >= MIN_POSITIONS,
            "{kind}: {} records from {} positions is below the floor\n{report}",
            tally.records,
            tally.positions
        );
        assert_eq!(tally.mismatches, 0, "{kind} mismatches the donor\n{report}");
        #[expect(
            clippy::cast_precision_loss,
            reason = "a record count is far below f64's exact range"
        )]
        let share = tally.exceptions as f64 / tally.records as f64;
        assert!(
            share < MAX_EXCEPTION_SHARE,
            "{kind}: {} exceptions in {} records\n{report}",
            tally.exceptions,
            tally.records
        );
    }
    assert_eq!(
        tallies.len(),
        COVERED.len(),
        "a rule covers a kind the list does not name\n{report}"
    );
    for (plant, counts) in PLANTS.iter().zip(&plant_tallies) {
        assert!(
            counts.get(plant.kind).is_some_and(|&n| n > 0),
            "the {} plant fails nothing\n{report}",
            plant.kind
        );
        assert_eq!(
            counts.len(),
            1,
            "the {} plant fails another kind too\n{report}",
            plant.kind
        );
    }
    let expected: Vec<&str> = UNDETECTABLE.iter().map(|&(name, _)| name).collect();
    assert_eq!(
        undetected, expected,
        "every seed but the named ones is detected\n{report}"
    );
}
