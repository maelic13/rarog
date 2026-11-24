//! Rate probes: how often each of the compact kernel's rules is asked and
//! how often it fires, for the tree statistics that compare the kernel with
//! its donor. A build with the `probes` feature prints them before `bestmove`
//! as `info string rates <kind>=<fires>/<attempts> ...`, in the donor
//! twin's kinds and format; without the feature the probes are compiled out.
#![cfg_attr(
    not(feature = "probes"),
    expect(
        dead_code,
        reason = "without the feature only this module's own test builds it"
    )
)]

use std::cell::Cell;
use std::fmt::Write as _;

/// The rules counted, by the donor twin's names.
#[derive(Copy, Clone, Debug)]
pub(in crate::search) enum Kind {
    Wrapper,
    TtCut,
    Estimate,
    EvalBonus,
    Hindsight,
    Razor,
    Rfp,
    NmpGate,
    NmpR,
    NmpResult,
    ProbcutGate,
    ProbcutDepth,
    SingularGate,
    SingularResult,
    Ldse,
    Lmp,
    Futility,
    BadNoisy,
    HistoryPrune,
    SeeMargin,
    Lmr,
    Research,
    Unreduced,
    Updates,
    FailLowReward,
    CorrUpdate,
    QTtCut,
    QStandPat,
    QSee,
    QLmp,
    QTail,
    Window,
    Optimism,
    PickNoisy,
    PickQuiet,
    Gravity,
}

const KINDS: usize = Kind::Gravity as usize + 1;

const NAMES: [&str; KINDS] = [
    "wrapper",
    "tt_cut",
    "estimate",
    "eval_bonus",
    "hindsight",
    "razor",
    "rfp",
    "nmp_gate",
    "nmp_r",
    "nmp_result",
    "probcut_gate",
    "probcut_depth",
    "singular_gate",
    "singular_result",
    "ldse",
    "lmp",
    "futility",
    "bad_noisy",
    "history_prune",
    "see_margin",
    "lmr",
    "research",
    "unreduced",
    "updates",
    "fail_low_reward",
    "corr_update",
    "q_tt_cut",
    "q_stand_pat",
    "q_see",
    "q_lmp",
    "q_tail",
    "window",
    "optimism",
    "pick_noisy",
    "pick_quiet",
    "gravity",
];

/// One search's counts. The cells let a rule asked through a shared borrow
/// (the picker's scoring) be counted too; a thread owns its counts.
#[derive(Clone)]
pub(in crate::search) struct Rates {
    fires: [Cell<u64>; KINDS],
    attempts: [Cell<u64>; KINDS],
}

impl Default for Rates {
    fn default() -> Self {
        Self {
            fires: [const { Cell::new(0) }; KINDS],
            attempts: [const { Cell::new(0) }; KINDS],
        }
    }
}

impl Rates {
    /// One asking of `kind`, and whether it fired.
    #[inline(always)]
    pub(in crate::search) fn record(&self, kind: Kind, fired: bool) {
        let index = kind as usize;
        self.attempts[index].set(self.attempts[index].get() + 1);
        self.fires[index].set(self.fires[index].get() + u64::from(fired));
    }

    /// `count` askings of `kind` that all fired.
    pub(in crate::search) fn add_fired(&self, kind: Kind, count: u64) {
        let index = kind as usize;
        self.attempts[index].set(self.attempts[index].get() + count);
        self.fires[index].set(self.fires[index].get() + count);
    }

    /// Forget the counts, at a search's start.
    pub(in crate::search) fn clear(&self) {
        for cell in self.fires.iter().chain(&self.attempts) {
            cell.set(0);
        }
    }

    /// The line printed before `bestmove`.
    pub(in crate::search) fn line(&self) -> String {
        let mut line = String::from("rates");
        for (index, name) in NAMES.iter().enumerate() {
            let _ = write!(
                line,
                " {name}={}/{}",
                self.fires[index].get(),
                self.attempts[index].get()
            );
        }
        line
    }
}

/// The eight seeds no `bench` decision reaches (D.7.11), counted in play:
/// how often each one's term is asked and how often it binds, that is,
/// changes its rule's result.
#[derive(Copy, Clone, Debug)]
pub(in crate::search) enum Seed {
    /// The table cutoff refused this close to the fifty-move limit.
    TtCutFifty,
    /// Reverse futility's margin at its floor.
    RfpMin,
    /// The null-move gate's margin at its floor.
    NmpMin,
    /// The reduction's singular term non-zero.
    FdsSingular,
    /// The singular term at its cap.
    FdsSingularCap,
    /// The singular term other than it would be with no offset.
    FdsSingularOffset,
    /// The window's held count at its cap.
    WindowStabilityCap,
    /// An in-check noisy move's good or bad tier other than it would be
    /// without the check base, the only order the base can change.
    PickCheckBase,
}

const SEEDS: usize = Seed::PickCheckBase as usize + 1;

const SEED_NAMES: [&str; SEEDS] = [
    "tt_cut_fifty",
    "rfp_min_16",
    "nmp_min_16",
    "fds_singular",
    "fds_singular_cap",
    "fds_singular_offset",
    "window_stability_cap",
    "pick_check_base",
];

thread_local! {
    static SEED_ASKS: [Cell<u64>; SEEDS] = const { [const { Cell::new(0) }; SEEDS] };
    static SEED_BINDS: [Cell<u64>; SEEDS] = const { [const { Cell::new(0) }; SEEDS] };
}

/// One asking of `seed`'s term on this thread, and whether it bound.
#[inline(always)]
pub(in crate::search) fn seed(seed: Seed, bound: bool) {
    let index = seed as usize;
    SEED_ASKS.with(|asks| asks[index].set(asks[index].get() + 1));
    SEED_BINDS.with(|binds| binds[index].set(binds[index].get() + u64::from(bound)));
}

/// Forget this thread's seed counts, at a search's start.
pub(in crate::search) fn seed_clear() {
    SEED_ASKS.with(|asks| asks.iter().for_each(|cell| cell.set(0)));
    SEED_BINDS.with(|binds| binds.iter().for_each(|cell| cell.set(0)));
}

/// The line printed before `bestmove`, beside the rates.
pub(in crate::search) fn seed_line() -> String {
    let mut line = String::from("seeds");
    SEED_ASKS.with(|asks| {
        SEED_BINDS.with(|binds| {
            for (index, name) in SEED_NAMES.iter().enumerate() {
                let _ = write!(line, " {name}={}/{}", binds[index].get(), asks[index].get());
            }
        });
    });
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_names_every_kind_once_in_the_twins_format() {
        let rates = Rates::default();
        rates.record(Kind::Rfp, true);
        rates.record(Kind::Rfp, false);
        rates.record(Kind::Gravity, true);
        let line = rates.line();
        assert!(line.starts_with("rates wrapper=0/0 tt_cut=0/0"));
        assert!(line.contains(" rfp=1/2 "));
        assert!(line.ends_with(" gravity=1/1"));
        assert_eq!(line.split(' ').count(), KINDS + 1);
        rates.clear();
        assert!(rates.line().contains(" rfp=0/0 "));
    }

    #[test]
    fn the_seed_line_counts_asks_and_binds_per_seed() {
        seed_clear();
        seed(Seed::RfpMin, true);
        seed(Seed::RfpMin, false);
        seed(Seed::PickCheckBase, false);
        let line = seed_line();
        assert!(
            line.starts_with("seeds tt_cut_fifty=0/0 rfp_min_16=1/2 "),
            "{line}"
        );
        assert!(line.ends_with(" pick_check_base=0/1"), "{line}");
        assert_eq!(line.split(' ').count(), SEEDS + 1);
        seed_clear();
        assert!(seed_line().contains(" rfp_min_16=0/0 "));
    }
}
