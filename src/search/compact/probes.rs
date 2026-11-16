//! Rate probes: how often each of the compact kernel's rules is asked and
//! how often it fires, for the tree statistics that compare the kernel with
//! its donor. A build with the `probes` feature prints them before `bestmove`
//! as `info string rates <kind>=<fires>/<attempts> ...`, in the donor
//! twin's kinds and format; without the feature the probes are compiled out.

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
}
