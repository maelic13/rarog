//! Layer A of the parity instrument: each of the compact kernel's rules
//! against the donor's own decisions on the same inputs.
//!
//! The donor's tracing twin writes one line per decision it takes at a
//! sample of nodes, `<kind> fire=<0|1> <name>=<int> ...`, its inputs and
//! outputs as named integers. The test reads every trace file in the
//! directory `RAROG_DONOR_TRACE` names, calls the rule for each record of
//! its kind and requires every recorded output exactly. A trace is the
//! donor's behaviour as numbers, so it stays in ignored storage and this
//! test is a local check, not CI's.
//!
//! Every input a rule reads is recorded; the trace of 2026-11-16 lacked
//! three (`nmp_gate`'s capture refutation, `singular_result`'s searched
//! move, `q_stand_pat`'s node type), so this test needs the extended twin's
//! trace of 2026-11-17.
//!
//! Two outputs are the board's, not a rule's: whether a move passes its SEE
//! threshold (`see_margin`, `q_see`). Their thresholds are checked.
//!
//! Each plant changes one seed; it must fail the kind that reads it and no
//! other, which proves both that the kind's check reads its rule and that
//! no other kind's check sees a change it should not. Six kinds read no
//! seed (`estimate`, `nmp_result`, `q_tt_cut`, `gravity`, `null_bound`,
//! `draw_score`) and have none; `spread` reads only the per-thread offsets,
//! which a one-thread trace multiplies by zero.
//!
//! The seed-coverage pass changes every seed in turn by a quarter of its
//! value, at least one, up and then down, and lists each seed that no
//! record detects either way; the list must hold only the seeds named with
//! their reason.
//!
//! The picker's two kinds score a move from its recorded components; the
//! components themselves are table reads.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;

use crate::eval::VALUE_NONE;
use crate::tt::Bound;

use super::super::history::apply_bonus;
use super::rules::{self, SEED_FIELDS, Seeds};

/// A kind's records must come from at least this many positions, and
/// number at least this many, for its zero mismatches to count.
const MIN_POSITIONS: usize = 200;
const MIN_RECORDS: usize = 2_000;
/// The floor for a kind written only when a rare event happens, declared
/// before the extended trace was captured: a ProbCut cutoff, a null
/// search, a completed iteration.
const MIN_RARE_RECORDS: usize = 200;
const RARE: &[&str] = &["null_bound", "probcut_value", "score_held"];

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

/// How a record compared with its rule.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Verdict {
    Match,
    Mismatch,
}

impl From<bool> for Verdict {
    fn from(matched: bool) -> Self {
        if matched { Self::Match } else { Self::Mismatch }
    }
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

/// How the rule compares with the record; `None` for a kind no rule covers.
fn verdict(s: &Seeds, r: &Record<'_>) -> Option<Verdict> {
    let fire = |fired: bool| fired == r.flag("fire");
    let v = match r.kind {
        "wrapper" => {
            let input = rules::WrapperInput {
                raw: r.int("raw"),
                material: r.int("material"),
                optimism: r.int("optimism"),
                fifty: r.int("fifty"),
                correction: r.int("correction"),
            };
            Verdict::from(rules::wrapped_eval(s, &input) == r.int("eval"))
        }
        "optimism" => Verdict::from(rules::optimism(s, r.int("best_avg")) == r.int("optimism")),
        "estimate" => {
            let input = rules::EstimateInput {
                in_check: r.flag("in_check"),
                excluded: r.flag("excluded"),
                tt_score: r.int("tt_score"),
                bound: r.bound("bound"),
                eval: r.int("eval"),
            };
            Verdict::from(rules::estimated_score(&input) == r.int("estimate"))
        }
        "eval_bonus" => Verdict::from(
            rules::eval_difference_bonus(s, r.int("eval"), r.int("parent_eval")) == r.int("bonus"),
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
            Verdict::from(fire(rules::tt_cuts(s, &input)))
        }
        "hindsight" => Verdict::from(
            rules::hindsight_depth(
                s,
                r.int("depth_in"),
                r.int("reduction"),
                r.int("eval_delta"),
                r.flag("tt_pv"),
            ) == r.int("depth"),
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
            Verdict::from(fire(rules::razors(s, &input)))
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
            Verdict::from(
                rules::rfp_margin(s, &input) == r.int("margin")
                    && fire(rules::rfp_cuts(s, &input))
                    && rules::rfp_value(s, input.estimate, input.beta) == r.int("value"),
            )
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
            Verdict::from(
                rules::nmp_margin(s, &input) == r.int("margin") && fire(rules::nmp_gate(s, &input)),
            )
        }
        "nmp_r" => Verdict::from(
            rules::nmp_reduction(
                s,
                r.flag("improving"),
                r.int("depth"),
                r.int("estimate"),
                r.int("beta"),
            ) == r.int("r"),
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
            let recorded = match r.int("outcome") {
                0 => rules::NullOutcome::Fail,
                1 => rules::NullOutcome::Cut,
                2 => rules::NullOutcome::Verify(r.int("verify_depth")),
                outcome => panic!("null outcome {outcome}"),
            };
            Verdict::from(outcome == recorded && fire(outcome != rules::NullOutcome::Fail))
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
            Verdict::from(
                probcut_beta == r.int("probcut_beta") && fire(rules::probcut_gate(&input)),
            )
        }
        "probcut_depth" => {
            let score = r.int("score");
            let probcut_beta = r.int("probcut_beta");
            let d =
                rules::probcut_depth(s, r.int("depth"), r.flag("improving"), score, probcut_beta);
            Verdict::from(
                d.base == r.int("base_depth")
                    && d.depth == r.int("probcut_depth")
                    && d.adjusted_beta == r.int("adjusted_beta")
                    && fire(score >= probcut_beta && d.depth > 0),
            )
        }
        "singular_gate" => Verdict::from(fire(rules::singular_candidate(
            s,
            r.int("depth"),
            r.flag("tt_pv"),
            r.int("tt_depth"),
            r.bound("bound"),
            r.int("tt_score"),
        ))),
        "singular_result" => singular_verdict(s, r),
        "ldse" => Verdict::from(fire(rules::ldse_extends(
            s,
            r.flag("singular_searched"),
            r.int("depth"),
            r.flag("in_check"),
            r.flag("cut_node"),
            r.int("estimate"),
            r.int("alpha"),
        ))),
        "lmp" => {
            let input = prune_input(r);
            Verdict::from(
                rules::lmp_limit(s, &input) == r.int("limit") && fire(rules::lmp_prunes(s, &input)),
            )
        }
        "futility" => {
            let input = prune_input(r);
            Verdict::from(
                rules::futility_value(s, &input) == r.int("value")
                    && fire(rules::futility_prunes(s, &input)),
            )
        }
        "bad_noisy" => {
            let input = prune_input(r);
            Verdict::from(
                rules::bad_noisy_value(s, &input) == r.int("value")
                    && fire(rules::bad_noisy_prunes(s, &input)),
            )
        }
        "history_prune" => Verdict::from(fire(rules::history_prunes(s, &prune_input(r)))),
        "see_margin" => Verdict::from(
            rules::see_threshold(s, r.flag("quiet"), r.int("depth"), r.int("history"))
                == r.int("threshold"),
        ),
        "lmr" => {
            let input = reduction_input(r, true);
            let reduction = rules::late_move_reduction(s, &input);
            Verdict::from(
                reduction == r.int("reduction")
                    && rules::lmr_depth(r.int("new_depth"), reduction, input.pv)
                        == r.int("reduced_depth"),
            )
        }
        "research" => {
            let new_depth = rules::research_depth(
                s,
                r.flag("root"),
                r.int("score"),
                r.int("best_score"),
                r.int("new_depth_in"),
            );
            Verdict::from(
                new_depth == r.int("new_depth") && fire(new_depth > r.int("reduced_depth")),
            )
        }
        "unreduced" => {
            let input = reduction_input(r, false);
            let reduction = rules::unreduced_reduction(s, &input);
            let new_depth = r.int("new_depth");
            let reduced = rules::unreduced_depth(s, new_depth, reduction);
            Verdict::from(
                reduction == r.int("reduction")
                    && reduced == r.int("reduced_depth")
                    && fire(reduced < new_depth),
            )
        }
        "updates" => {
            let a = rules::update_amounts(
                s,
                r.int("depth"),
                r.flag("cut_node"),
                r.int("noisy"),
                r.int("quiet"),
            );
            Verdict::from(
                a.noisy_bonus == r.int("noisy_bonus")
                    && a.noisy_malus == r.int("noisy_malus")
                    && a.quiet_bonus == r.int("quiet_bonus")
                    && a.quiet_malus == r.int("quiet_malus")
                    && a.cont_bonus == r.int("cont_bonus")
                    && a.cont_malus == r.int("cont_malus")
                    && a.parent_malus == r.int("parent_malus")
                    && a.research_bonus == r.int("research_bonus"),
            )
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
                Verdict::from(
                    reward.factor == r.int("factor")
                        && reward.bonus == r.int("bonus")
                        && reward.grandparent == r.int("grandparent"),
                )
            } else {
                Verdict::from(rules::noisy_fail_low_reward(s, r.int("depth")) == r.int("bonus"))
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
            Verdict::from(fire(update.is_some()) && update.is_none_or(|b| b == r.int("bonus")))
        }
        "gravity" => {
            // The history tables' own update, which every kernel shares.
            let mut entry = i16::try_from(r.int("entry")).expect("a table entry");
            apply_bonus(&mut entry, r.int("bonus"), r.int("max"));
            Verdict::from(i32::from(entry) == r.int("result"))
        }
        "q_tt_cut" => Verdict::from(fire(rules::q_tt_cuts(
            r.flag("pv"),
            r.int("tt_score"),
            r.bound("bound"),
            r.int("alpha"),
            r.int("beta"),
        ))),
        "q_stand_pat" => {
            let best_score = r.int("best_score");
            let beta = r.int("beta");
            Verdict::from(
                rules::q_stand_pat_value(s, best_score, beta) == r.int("value")
                    && fire(best_score >= beta)
                    && rules::stand_pat(
                        r.flag("in_check"),
                        r.flag("pv"),
                        r.int("eval"),
                        r.int("tt_score"),
                        r.bound("bound"),
                    ) == best_score,
            )
        }
        "q_lmp" => Verdict::from(fire(rules::q_stops(
            s,
            r.int("move_count"),
            r.flag("direct_check"),
        ))),
        "q_see" => Verdict::from(
            rules::q_see_threshold(
                s,
                r.int("alpha"),
                r.int("eval"),
                r.int("correction"),
                r.int("history"),
            ) == r.int("threshold"),
        ),
        "q_tail" => {
            let best_score = r.int("best_score");
            let beta = r.int("beta");
            Verdict::from(
                rules::q_tail_value(s, best_score, beta) == r.int("value")
                    && fire(best_score >= beta),
            )
        }
        "window" => window_verdict(s, r),
        "pick_noisy" => Verdict::from(
            rules::noisy_order(
                s,
                &rules::NoisyOrderInput {
                    victim: r.int("victim"),
                    history: r.int("history"),
                    queen_promotion: r.flag("queen_promotion"),
                    in_check: r.flag("in_check"),
                    piece: r.int("piece"),
                },
            ) == r.int("score"),
        ),
        "pick_quiet" => Verdict::from(
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
            ) == r.int("score"),
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
            Verdict::from(
                fire(rewards.is_some())
                    && rewards.is_none_or(|(quiet, cont)| {
                        quiet == r.int("quiet_bonus") && cont == r.int("cont_bonus")
                    }),
            )
        }
        "tt_cut_return" => Verdict::from(fire(rules::tt_cut_returns(s, r.int("fifty")))),
        "eval_bonus_gate" => Verdict::from(fire(rules::eval_bonus_applies(
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
        ))),
        "null_bound" => {
            let bound = rules::null_bound(
                r.int("depth"),
                r.int("beta"),
                r.int("tt_score"),
                r.bound("bound"),
                r.int("tt_depth"),
            );
            Verdict::from(bound == r.int("null_beta") && fire(bound != r.int("beta")))
        }
        "probcut_value" => {
            Verdict::from(rules::probcut_value(s, r.int("score"), r.int("beta")) == r.int("value"))
        }
        "spread" => {
            let term = match r.int("term") {
                0 => rules::SpreadTerm::Lmr,
                1 => rules::SpreadTerm::Unreduced,
                term => panic!("spread term {term}"),
            };
            Verdict::from(
                rules::spread(s, term, r.count("nodes"), r.count("thread")) == r.int("spread"),
            )
        }
        "malus_fade" => Verdict::from(rules::malus_fade(s, r.int("index")) == r.int("scale")),
        "q_cutoff_bonus" => {
            let reward =
                rules::q_cutoff_reward(s, r.int("best_score"), r.int("beta"), r.flag("noisy_best"));
            Verdict::from(
                fire(reward.is_some()) && reward.is_none_or(|bonus| bonus == r.int("bonus")),
            )
        }
        "score_held" => Verdict::from(fire(rules::score_held(s, r.int("score"), r.int("average")))),
        "pick_threshold" => Verdict::from(
            rules::good_noisy_threshold(
                s,
                r.int("score"),
                r.flag("tt_quiet"),
                r.int("good_so_far"),
            ) == r.int("threshold"),
        ),
        "draw_score" => Verdict::from(rules::draw_score(r.count("nodes")) == r.int("value")),
        _ => return None,
    };
    Some(v)
}

fn singular_verdict(s: &Seeds, r: &Record<'_>) -> Verdict {
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
        return Verdict::from(
            rules::singular_outcome(s, &input) == rules::SingularOutcome::MultiCut(r.int("value")),
        );
    }
    let depth = r.int("depth");
    let pv = r.flag("pv");
    let tt_pv = r.flag("tt_pv");
    let margin = rules::singular_margin(s, depth, r.flag("exact"), tt_pv && !pv);
    if margin != r.int("margin")
        || r.int("tt_score") - margin != singular_beta
        || (depth - 1) / 2 != r.int("singular_depth")
    {
        return Verdict::Mismatch;
    }
    let input = rules::SingularInput {
        singular_score,
        singular_beta,
        beta,
        tt_score: r.int("tt_score"),
        pv,
        tt_was_pv: r.flag("tt_was_pv"),
        tt_quiet: r.flag("tt_quiet"),
        correction: r.int("correction"),
        cut_node: r.flag("cut_node"),
        searched_a_move: r.flag("searched"),
    };
    let (extension, cleared) = match rules::singular_outcome(s, &input) {
        rules::SingularOutcome::None => (0, false),
        rules::SingularOutcome::Extend(plies) => (plies, false),
        rules::SingularOutcome::Demote => (0, true),
        rules::SingularOutcome::MultiCut(_) => return Verdict::Mismatch,
    };
    Verdict::from(
        extension == r.int("extension")
            && cleared == r.flag("tt_cleared")
            && (extension != 0) == r.flag("fire"),
    )
}

fn window_verdict(s: &Seeds, r: &Record<'_>) -> Verdict {
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
                if after.reduction != r.int("reduction") {
                    return Verdict::Mismatch;
                }
                after
            }
        }
    };
    Verdict::from(
        window.alpha == r.int("alpha")
            && window.beta == r.int("beta")
            && window.delta == r.int("delta"),
    )
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
        change: |s| s.eval_bonus_scale += 64,
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
        change: |s| s.razor_base += 64,
    },
    Plant {
        kind: "rfp",
        change: |s| s.rfp_constant += 1,
    },
    Plant {
        kind: "nmp_gate",
        change: |s| s.nmp_base += 1,
    },
    Plant {
        kind: "nmp_r",
        change: |s| s.nmp_r_base += 256,
    },
    Plant {
        kind: "probcut_gate",
        change: |s| s.probcut_margin += 1,
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
        change: |s| s.fp_constant += 1,
    },
    Plant {
        kind: "bad_noisy",
        change: |s| s.bnfp_constant += 1,
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
        change: |s| s.lmr_quiet += 1,
    },
    Plant {
        kind: "research",
        change: |s| s.research_deeper += 8,
    },
    Plant {
        kind: "unreduced",
        change: |s| s.fds_quiet += 1,
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
        change: |s| s.q_see_constant += 1,
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
        change: |s| s.window_base += 1,
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
        change: |s| s.window_stable += 20,
    },
    Plant {
        kind: "pick_threshold",
        change: |s| s.pick_see_offset += 1,
    },
];

/// Seeds no record can detect, each with its reason.
const UNDETECTABLE: &[(&str, &str)] = &[
    (
        "double_new_pv",
        "read only at a PV node whose singular search failed low, which the \
         trace samples sparsely (47 records in the 2026-11-17 capture, 11 at a new PV \
         node), none of them within a step of the margin",
    ),
    (
        "triple_pv",
        "read only at a PV node whose singular search failed low, which the \
         trace samples sparsely (47 records in the 2026-11-17 capture, 11 at a new PV \
         node), none of them within a step of the margin",
    ),
    (
        "triple_new_pv",
        "read only at a PV node whose singular search failed low, which the \
         trace samples sparsely (47 records in the 2026-11-17 capture, 11 at a new PV \
         node), none of them within a step of the margin",
    ),
    (
        "lmr_spread_thread",
        "a per-thread offset, times thread 0 in a one-thread trace",
    ),
    (
        "fds_singular",
        "the full-depth search reads it only with the TT move's score and a singular score, \
         but a singular search needs depth 5 and from depth 2 every move after the first is \
         reduced, so the full-depth search runs only for the first move, before its score exists",
    ),
    (
        "fds_singular_offset",
        "read only inside the full-depth search's singular term, which never runs (fds_singular)",
    ),
    (
        "fds_singular_cap",
        "read only inside the full-depth search's singular term, which never runs (fds_singular)",
    ),
    (
        "fds_spread_thread",
        "a per-thread offset, times thread 0 in a one-thread trace",
    ),
    (
        "pick_escape[0]",
        "a pawn escapes nothing: the donor maps no threat to pawns",
    ),
    (
        "pick_escape[5]",
        "a king escapes nothing: the donor maps no threat to kings",
    ),
];

#[derive(Default)]
struct Tally {
    records: usize,
    positions: usize,
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
            let Some(v) = verdict(&base, &record) else {
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
                Verdict::Mismatch => {
                    tally.mismatches += 1;
                    if tally.examples.len() < 5 {
                        tally.examples.push(line.to_string());
                    }
                }
            }
            for (seeds, counts) in planted.iter().zip(plant_tallies.iter_mut()) {
                if verdict(seeds, &record) == Some(Verdict::Mismatch) {
                    *counts.entry(record.kind.to_string()).or_default() += 1;
                }
            }
            if v == Verdict::Match {
                undetected.retain(|&index| {
                    perturbed[index]
                        .iter()
                        .all(|seeds| verdict(seeds, &record) == Some(v))
                });
            }
        }
    }

    let mut report = String::new();
    let _ = writeln!(report, "{} trace files from {}", files.len(), dir.display());
    for (kind, tally) in &tallies {
        let _ = writeln!(
            report,
            "{kind:16} records {:>8} positions {:>4} mismatches {}",
            tally.records, tally.positions, tally.mismatches
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
