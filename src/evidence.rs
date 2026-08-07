//! Phase 4.2 — typed search-result evidence and published consumer capabilities.
//!
//! # Why this module exists
//!
//! Before 4.2 the search decoded a transposition-table probe into five loose
//! locals (`tt_score`, `tt_depth`, `tt_bound`, `tt_pv`, `tt_raw_move`) and then
//! spelled out each consumer's admission rule inline, at thirteen separate read
//! sites. Two consequences were measured in RAR-S22:
//!
//! * Nothing recorded WHICH producer wrote an entry. 67% of sampled stores are
//!   depth-0 qsearch entries and 37% are bare stand pat, yet a depth-0 `Lower`
//!   bound is indistinguishable from a searched one once it is in the table. A
//!   third of sampled singular attempts read an entry at exactly ProbCut's
//!   `depth-3`/`Lower` signature; the coincidence was countable, the producer
//!   was not.
//! * Because each rule was written where it was used, two consumers that ought
//!   to share a rule had silently drifted apart — see
//!   [`NodeEvidence::refine_eval`] versus
//!   [`NodeEvidence::refine_eval_bound_only`].
//!
//! # What 4.2 changes, and what it deliberately does not
//!
//! This module is **behaviour-neutral by construction**. Every predicate below
//! reproduces its pre-4.2 condition exactly, including the drift just named, so
//! the bench fingerprint is unchanged. The point is to move the rules to one
//! place where 4.3 and 4.4 can each tighten ONE predicate under its own gate,
//! instead of performing scattered condition surgery and hoping the arms stay
//! separable.
//!
//! [`OutcomeKind`] is therefore recorded at the store sites and consumed by the
//! debug contract and the diagnostic producer map, but it is **not persisted**:
//! `TtEntry.flag_age` has no spare bits, and the one cheap slot moves the bench
//! fingerprint. `PLAN.md` §5 4.2 prices the alternatives. Consumers may not
//! branch on a producer class they cannot actually read back yet — where a rule
//! needs provenance, that is 4.3/4.4 work with a strength gate attached.

use crate::board::Move;
use crate::eval::{MATE_SCORE, VALUE_NONE};
use crate::tt::{Bound, TtEntry, score_from_tt};

/// Mate-distance reservation, mirroring `search::MAX_PLY` and `tt::MAX_PLY`.
const MAX_PLY: i32 = 128;

/// What actually produced a score.
///
/// Invariant 1 of `PLAN.md` §5: every result is typed. The variants are ordered
/// from most to least authoritative, but authority is not a total order — see
/// the predicates rather than comparing variants.
///
/// `VerifiedReduced`, `Null` and `Incomplete` are declared but not yet produced
/// by any store site: no current path writes a null-move or aborted result to
/// the table, and LMR re-searches store through the ordinary full-search path.
/// They are named here because 4.4 (null-move contract) and 4.7 (abort returns
/// last completed evidence) need them, and because leaving them out would make
/// the taxonomy look complete when it is not.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OutcomeKind {
    /// Full-width search of the node at its nominal depth.
    Full,
    /// A reduced search whose result was re-searched at full depth and held.
    VerifiedReduced,
    /// A qsearch move that was actually searched and failed high.
    QsearchMove,
    /// A completed qsearch node: capture/evasion horizon, no full-width claim.
    QsearchTail,
    /// Static evaluation returned without searching a single move.
    StandPat,
    /// Speculative ProbCut result. The stored score is margin-shifted off the
    /// speculative window, so it is not a plain search result at its depth.
    ProbCut,
    /// Null-move result. Declared for 4.4; not currently stored.
    Null,
    /// Tablebase WDL. Authoritative, but it is not a search and carries no move.
    Tablebase,
    /// Search was aborted before the node completed. Declared for 4.7.
    Incomplete,
}

impl OutcomeKind {
    /// A move loop ran at this node. False for stand pat and tablebase hits.
    #[inline(always)]
    pub const fn is_searched(self) -> bool {
        matches!(
            self,
            Self::Full
                | Self::VerifiedReduced
                | Self::QsearchMove
                | Self::QsearchTail
                | Self::ProbCut
                | Self::Null
        )
    }

    /// Produced at the qsearch horizon, i.e. stored at depth 0. These carry no
    /// claim about quiet continuations.
    #[inline(always)]
    pub const fn is_horizon(self) -> bool {
        matches!(self, Self::QsearchMove | Self::QsearchTail | Self::StandPat)
    }

    /// The score is margin-shifted or window-speculative rather than a plain
    /// negamax value at the stored depth. 4.3 forbids these from granting
    /// singular or exact-learning authority.
    #[inline(always)]
    pub const fn is_speculative(self) -> bool {
        matches!(self, Self::ProbCut | Self::Null)
    }

    /// Stable label for diagnostics and assertion messages.
    #[inline(always)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::VerifiedReduced => "verified_reduced",
            Self::QsearchMove => "qsearch_move",
            Self::QsearchTail => "qsearch_tail",
            Self::StandPat => "stand_pat",
            Self::ProbCut => "probcut",
            Self::Null => "null",
            Self::Tablebase => "tablebase",
            Self::Incomplete => "incomplete",
        }
    }
}

/// One node's decoded transposition-table evidence, plus the admission rules
/// its consumers are allowed to apply.
///
/// Built once per node immediately after the probe. Mate-distance conversion
/// and rule-50 clamping happen here exactly once (`score_from_tt`), so no
/// consumer can forget them or apply them twice — before 4.2 the main search
/// decoded the same entry twice, once for `tt_score` and again inside the
/// cutoff block.
///
/// Node-local context (`depth`, `alpha`, `beta`, `is_pv`) is passed to the
/// predicates rather than stored: IIR mutates `depth` after the cutoff test and
/// the move loop raises `alpha`, so a snapshot would go stale mid-node.
#[derive(Copy, Clone, Debug)]
pub struct NodeEvidence {
    /// Bound kind, or `None` for a probe miss.
    pub bound: Option<Bound>,
    /// Stored depth; `-1` on a miss, matching the pre-4.2 `tt_depth` default.
    pub depth: i32,
    /// Score with mate distance and rule-50 already resolved for this node;
    /// `VALUE_NONE` on a miss.
    pub score: i32,
    /// Raw (uncorrected) static eval as stored, or `VALUE_NONE`.
    pub raw_static_eval: i32,
    /// Stored best move, unvalidated — callers must still check legality.
    pub mv: Option<Move>,
    /// The entry's own PV bit. Combine with the node's `is_pv` via
    /// [`Self::pv_line`]; do not read this directly for pruning decisions.
    pub stored_pv: bool,
    /// Whether the probe hit at all.
    pub hit: bool,
}

impl NodeEvidence {
    /// Evidence for a probe that missed.
    pub const MISS: Self = Self {
        bound: None,
        depth: -1,
        score: VALUE_NONE,
        raw_static_eval: VALUE_NONE,
        mv: None,
        stored_pv: false,
        hit: false,
    };

    /// Decode a probe result. `halfmove_clock` is the node's, and is what makes
    /// a mate score rule-50-safe.
    #[inline(always)]
    pub fn from_probe(entry: Option<TtEntry>, ply: usize, halfmove_clock: u8) -> Self {
        match entry {
            None => Self::MISS,
            Some(entry) => Self {
                bound: entry.bound(),
                depth: i32::from(entry.depth),
                score: score_from_tt(i32::from(entry.score), ply, halfmove_clock),
                raw_static_eval: i32::from(entry.static_eval),
                mv: entry.best_move(),
                stored_pv: entry.is_pv_node(),
                hit: true,
            },
        }
    }

    /// Does this node sit on a PV line, either currently or per the stored bit?
    ///
    /// One inherited bit currently gates RFP, razor, NMP and ProbCut together;
    /// 4.4 replaces that with per-mechanism eligibility. Routing every reader
    /// through here is what makes those call sites findable.
    #[inline(always)]
    pub fn pv_line(&self, is_pv: bool) -> bool {
        is_pv || self.stored_pv
    }

    /// An exact score is stored. Consumed by the LMR reduction adjustment.
    #[inline(always)]
    pub fn is_exact(&self) -> bool {
        matches!(self.bound, Some(Bound::Exact))
    }

    /// CAPABILITY: cut this node off outright.
    ///
    /// Returns the score to return, or `None`. The caller still owns the node
    /// role guards (`!is_pv`, no excluded move) — this covers only the
    /// evidence-side rule: deep enough, and a bound that resolves the window.
    ///
    /// Note there is no provenance guard: a depth-0 stand-pat `Lower` cuts a
    /// qsearch node exactly as a searched one does. That is current behaviour,
    /// preserved deliberately; 4.3 owns the change.
    #[inline(always)]
    pub fn cutoff_score(&self, depth: i32, alpha: i32, beta: i32) -> Option<i32> {
        if self.depth < depth {
            return None;
        }
        match self.bound? {
            Bound::Exact => Some(self.score),
            Bound::Lower if self.score >= beta => Some(self.score),
            Bound::Upper if self.score <= alpha => Some(self.score),
            _ => None,
        }
    }

    /// CAPABILITY: stand in for the static eval when forward-pruning.
    ///
    /// The main-search form: requires a real score and `min_depth`
    /// (`EvalPruneTtMinDepth`, seeded 0) plies of stored depth. At the seed this
    /// admits depth-0 qsearch entries at any node depth, which RAR-S22 measured
    /// as 67% of stores. Raising the seed is a tuning decision inside 4.10;
    /// typing the producer is 4.3.
    #[inline(always)]
    pub fn refine_eval(&self, static_eval: i32, min_depth: i32) -> i32 {
        if self.score == VALUE_NONE || self.depth < min_depth {
            return static_eval;
        }
        self.refine_eval_bound_only(static_eval)
    }

    /// CAPABILITY: the same refinement with NO depth or `VALUE_NONE` guard.
    ///
    /// ⚠ This is the qsearch stand-pat path (RAR-S02, accepted at about +6.5
    /// Elo), and the missing guards are a real asymmetry against
    /// [`Self::refine_eval`], not an oversight in this refactor. It is exposed
    /// as its own named capability so the divergence is visible at both call
    /// sites; unifying the two is 4.3 work and needs its own gate, because
    /// RAR-S15 shows a cleaner primitive can de-tune the consumers fitted
    /// around the looser one.
    #[inline(always)]
    pub fn refine_eval_bound_only(&self, base: i32) -> i32 {
        match self.bound {
            Some(Bound::Exact) => self.score,
            Some(Bound::Lower) if self.score > base => self.score,
            Some(Bound::Upper) if self.score < base => self.score,
            _ => base,
        }
    }

    /// CAPABILITY: seed a singular-extension verification window.
    ///
    /// Requires a lower-or-exact bound within `depth_margin` plies of the node
    /// depth and a non-mate score. It does NOT require the score to come from a
    /// full search, so at the default margin of 3 ProbCut's margin-shifted
    /// `depth-3` `Lower` qualifies — 32 of 101 sampled attempts sat on that
    /// signature. Tightening the margin is 4.3 arm B and needs its own gate.
    #[inline(always)]
    pub fn allows_singular(&self, depth: i32, depth_margin: i32) -> bool {
        self.depth >= depth - depth_margin
            && matches!(self.bound, Some(Bound::Lower | Bound::Exact))
            && self.score.abs() < MATE_SCORE - MAX_PLY
    }

    /// CAPABILITY: is the stored depth too shallow to guide move ordering?
    ///
    /// The evidence half of the IIR predicate; the caller owns the node-role
    /// half (no TT move, or non-PV). 4.4 restricts IIR by role and TT quality.
    #[inline(always)]
    pub fn too_shallow_to_order(&self, depth: i32) -> bool {
        self.depth < depth - 3
    }

    /// An inexact bound that points the wrong way for the current window: a
    /// `Lower` at or below `alpha`, or an `Upper` at or above `beta`. Such an
    /// entry is admissible but told us nothing, and 4.2's registered shadow test
    /// is whether it should carry a confidence or depth penalty.
    ///
    /// Diagnostic only — no consumer branches on it.
    #[inline(always)]
    pub fn contradicts_window(&self, alpha: i32, beta: i32) -> bool {
        matches!(self.bound, Some(Bound::Lower)) && self.score <= alpha
            || matches!(self.bound, Some(Bound::Upper)) && self.score >= beta
    }
}

/// Move-ordering stage a move was picked from.
///
/// Replaces a bare `0..3` integer in the move loop. The classes are ordered as
/// the picker yields them.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MoveClass {
    /// The transposition-table move.
    TtMove,
    /// Capture with non-negative SEE at pick time.
    GoodCapture,
    /// Quiet move, per `Board::is_quiet_move`.
    Quiet,
    /// Capture with negative SEE, and anything not covered above.
    BadCapture,
}

/// Pre-move evidence snapshot, taken when the picker yields a move.
///
/// 4.2 populates only what it can populate correctly: the ordering class and
/// the picker's own scores. 4.6 extends this with check/evasion taxonomy, node
/// and TT evidence, correction confidence and extension/IIR debt, and derives
/// LMP, futility, SEE pruning and LMR from one prospective depth built here.
///
/// `see` is the value AT PICK TIME. The move loop's own `see` local is refined
/// later for some moves; this snapshot deliberately does not track that, so
/// classification cannot drift depending on where it is read.
#[derive(Copy, Clone, Debug)]
pub struct MoveEvidence {
    /// Stage the move came from.
    pub class: MoveClass,
    /// SEE at pick time; 0 for non-captures.
    pub see: i32,
    /// Quiet history score at pick time; 0 for non-quiet moves.
    pub quiet_history: i32,
}

impl MoveEvidence {
    /// Classify a picked move. `see` and `quiet_history` are the picker's, and
    /// `is_capture`/`is_quiet` the board's — all as computed at pick time.
    #[inline(always)]
    pub fn new(
        is_tt_move: bool,
        is_capture: bool,
        is_quiet: bool,
        see: i32,
        quiet_history: i32,
    ) -> Self {
        let class = if is_tt_move {
            MoveClass::TtMove
        } else if is_capture && see >= 0 {
            MoveClass::GoodCapture
        } else if is_quiet {
            MoveClass::Quiet
        } else {
            MoveClass::BadCapture
        };
        Self {
            class,
            see,
            quiet_history,
        }
    }
}

/// Debug-only producer contract: does a store's declared kind agree with the
/// shape of what it is storing?
///
/// Compiled out of release builds. It exists because the taxonomy is only worth
/// having if a mislabelled store is caught at the store site rather than
/// inferred later from a depth coincidence — which is precisely the failure
/// RAR-S22 documented for ProbCut and singular.
#[inline(always)]
pub fn debug_assert_outcome(kind: OutcomeKind, depth: i32, bound: Bound, mv: Move) {
    match kind {
        OutcomeKind::StandPat => {
            debug_assert_eq!(depth, 0, "stand pat is a depth-0 estimate");
            debug_assert_eq!(bound, Bound::Lower, "stand pat only ever fails high");
            debug_assert!(mv.is_null(), "stand pat searched no move");
        }
        OutcomeKind::QsearchMove => {
            debug_assert_eq!(depth, 0, "qsearch stores at the horizon");
            debug_assert_eq!(bound, Bound::Lower, "a qsearch move store is a fail-high");
            debug_assert!(!mv.is_null(), "a qsearch move store names its move");
        }
        OutcomeKind::QsearchTail => {
            debug_assert_eq!(depth, 0, "qsearch stores at the horizon");
            debug_assert_ne!(
                bound,
                Bound::Lower,
                "a completed qsearch node is exact or an upper bound"
            );
        }
        OutcomeKind::ProbCut => {
            debug_assert!(
                depth >= 1,
                "ProbCut runs only at depth >= 4, storing depth-3"
            );
            debug_assert_eq!(bound, Bound::Lower, "ProbCut is a speculative fail-high");
        }
        OutcomeKind::Tablebase => {
            debug_assert_eq!(bound, Bound::Exact, "a WDL hit is exact");
            debug_assert!(mv.is_null(), "a WDL hit carries no move");
        }
        OutcomeKind::Full | OutcomeKind::VerifiedReduced => {
            // No depth row here on purpose. In the engine an interior store is
            // always at depth >= 1 (`depth <= 0` routes to qsearch before any
            // store), but that is a SEARCH-path property, and this contract runs
            // in `TranspositionTable::store`, which the layout tests also drive
            // across the full i8 depth range to prove the field round-trips.
            // Asserting it here would make an encoding test fail for a reason
            // that has nothing to do with encoding. The depth-versus-authority
            // relation is what 4.3's typed consumers enforce, on the read side
            // where it actually decides something.
        }
        OutcomeKind::Null | OutcomeKind::Incomplete => {
            debug_assert!(false, "{} results are not stored yet", kind.label());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MoveClass, MoveEvidence, NodeEvidence, OutcomeKind};
    use crate::eval::{MATE_SCORE, VALUE_NONE};
    use crate::tt::Bound;

    /// Build evidence directly, bypassing `from_probe`, so a case can be stated
    /// without constructing a table and a matching key.
    fn evidence(bound: Bound, depth: i32, score: i32) -> NodeEvidence {
        NodeEvidence {
            bound: Some(bound),
            depth,
            score,
            ..NodeEvidence::MISS
        }
    }

    #[test]
    fn a_miss_grants_no_capability() {
        let miss = NodeEvidence::MISS;
        assert_eq!(miss.cutoff_score(0, -100, 100), None);
        assert_eq!(miss.refine_eval(42, 0), 42);
        assert_eq!(miss.refine_eval_bound_only(42), 42);
        assert!(!miss.allows_singular(4, 3));
        assert!(!miss.is_exact());
        // -1 is the pre-4.2 `tt_depth` default and must keep behaving as one.
        assert_eq!(miss.depth, -1);
    }

    #[test]
    fn cutoff_requires_depth_and_a_resolving_bound() {
        // Exact resolves any window.
        assert_eq!(
            evidence(Bound::Exact, 8, 50).cutoff_score(8, 0, 100),
            Some(50)
        );
        // Deeper than asked is fine; shallower is not.
        assert_eq!(
            evidence(Bound::Exact, 9, 50).cutoff_score(8, 0, 100),
            Some(50)
        );
        assert_eq!(evidence(Bound::Exact, 7, 50).cutoff_score(8, 0, 100), None);
        // A lower bound needs to reach beta, an upper bound to fall to alpha.
        assert_eq!(
            evidence(Bound::Lower, 8, 150).cutoff_score(8, 0, 100),
            Some(150)
        );
        assert_eq!(evidence(Bound::Lower, 8, 50).cutoff_score(8, 0, 100), None);
        assert_eq!(
            evidence(Bound::Upper, 8, -50).cutoff_score(8, 0, 100),
            Some(-50)
        );
        assert_eq!(evidence(Bound::Upper, 8, 50).cutoff_score(8, 0, 100), None);
    }

    #[test]
    fn eval_refinement_only_moves_in_the_bound_direction() {
        // A lower bound may raise the estimate, never lower it.
        assert_eq!(evidence(Bound::Lower, 4, 80).refine_eval(30, 0), 80);
        assert_eq!(evidence(Bound::Lower, 4, 10).refine_eval(30, 0), 30);
        // An upper bound may lower it, never raise it.
        assert_eq!(evidence(Bound::Upper, 4, 10).refine_eval(30, 0), 10);
        assert_eq!(evidence(Bound::Upper, 4, 80).refine_eval(30, 0), 30);
    }

    #[test]
    fn only_the_main_search_form_enforces_a_depth_floor() {
        // This test pins the audited asymmetry between the two capabilities. If
        // 4.3 unifies them, it must change this test deliberately, under a gate.
        let shallow = evidence(Bound::Lower, 0, 80);
        assert_eq!(shallow.refine_eval(30, 4), 30, "depth floor rejects it");
        assert_eq!(
            shallow.refine_eval_bound_only(30),
            80,
            "the qsearch form has no floor"
        );
        // The seeded EvalPruneTtMinDepth of 0 admits a depth-0 entry.
        assert_eq!(shallow.refine_eval(30, 0), 80);

        let none = NodeEvidence {
            score: VALUE_NONE,
            ..evidence(Bound::Lower, 8, 0)
        };
        assert_eq!(none.refine_eval(30, 0), 30, "VALUE_NONE is rejected");
    }

    #[test]
    fn singular_depth_margin_cannot_identify_a_probcut_producer() {
        // A same-node ProbCut writes a Lower bound at depth-3, and singular at
        // margin 3 accepts that shape. Shape is not provenance: a full search
        // can look identical, and a ProbCut from a deeper search may later be
        // consumed at a shallower node.
        let probcut_shaped = evidence(Bound::Lower, 5, 40);
        assert!(probcut_shaped.allows_singular(8, 3));
        // Arm B excludes the whole same-depth band, not just ProbCut.
        assert!(!probcut_shaped.allows_singular(8, 2));
        // Shallower than the margin is refused either way.
        assert!(!evidence(Bound::Lower, 4, 40).allows_singular(8, 3));
        // Upper bounds never qualify.
        assert!(!evidence(Bound::Upper, 8, 40).allows_singular(8, 3));
        // Mate scores never qualify.
        assert!(!evidence(Bound::Lower, 8, MATE_SCORE - 10).allows_singular(8, 3));
        assert!(!evidence(Bound::Lower, 8, -MATE_SCORE + 10).allows_singular(8, 3));
    }

    #[test]
    fn guarded_refinement_at_depth_zero_equals_the_unguarded_form() {
        // 4.3 arm C lands inert, and this is the claim that makes it inert: for
        // every state the engine can actually store, `refine_eval(_, 0)` admits
        // exactly what `refine_eval_bound_only` admits. Two facts carry it —
        // every stored depth is >= 0 (qsearch writes 0, every other producer
        // writes >= 1), and a post-conversion `ev.score` can never be
        // VALUE_NONE (mate scores come back clamped to +/-MATE_SCORE). If a
        // future producer breaks either, this test fails and the arm stops
        // being inert, which is the point.
        for bound in [Bound::Exact, Bound::Lower, Bound::Upper] {
            for depth in 0..=12 {
                for score in [-MATE_SCORE, -300, -1, 0, 1, 300, MATE_SCORE] {
                    for base in [-500, -1, 0, 1, 500] {
                        let ev = evidence(bound, depth, score);
                        assert_eq!(
                            ev.refine_eval(base, 0),
                            ev.refine_eval_bound_only(base),
                            "bound {bound:?} depth {depth} score {score} base {base}"
                        );
                    }
                }
            }
        }
        // A miss agrees too, because its bound is None.
        assert_eq!(
            NodeEvidence::MISS.refine_eval(42, 0),
            NodeEvidence::MISS.refine_eval_bound_only(42)
        );
    }

    #[test]
    fn window_contradiction_is_a_one_sided_test() {
        assert!(evidence(Bound::Lower, 8, -10).contradicts_window(0, 100));
        assert!(!evidence(Bound::Lower, 8, 150).contradicts_window(0, 100));
        assert!(evidence(Bound::Upper, 8, 150).contradicts_window(0, 100));
        assert!(!evidence(Bound::Upper, 8, -10).contradicts_window(0, 100));
        assert!(!evidence(Bound::Exact, 8, 50).contradicts_window(0, 100));
    }

    #[test]
    fn a_contradicting_bound_can_never_produce_a_cutoff() {
        // 4.2b relies on this: the shadow test measures only the NON-cutoff
        // consumers, and that is sound only because a contradicting entry is
        // structurally incapable of cutting off. A `Lower` at or below alpha
        // cannot reach beta, and an `Upper` at or above beta cannot fall to
        // alpha, for any window with alpha < beta. Swept rather than argued.
        for alpha in -300..=300 {
            for beta in (alpha + 1)..=300 {
                for score in [alpha - 1, alpha, beta, beta + 1] {
                    for bound in [Bound::Lower, Bound::Upper] {
                        // Depth is generous so only the bound direction decides.
                        let ev = evidence(bound, 99, score);
                        if ev.contradicts_window(alpha, beta) {
                            assert_eq!(
                                ev.cutoff_score(0, alpha, beta),
                                None,
                                "bound {bound:?} score {score} cut off window [{alpha},{beta}]"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn pv_line_is_the_union_of_node_and_stored_bits() {
        let stored = NodeEvidence {
            stored_pv: true,
            ..NodeEvidence::MISS
        };
        assert!(stored.pv_line(false), "the stored bit alone is enough");
        assert!(
            NodeEvidence::MISS.pv_line(true),
            "the node bit alone is enough"
        );
        assert!(!NodeEvidence::MISS.pv_line(false));
    }

    #[test]
    fn outcome_kinds_partition_authority() {
        assert!(OutcomeKind::Full.is_searched());
        assert!(!OutcomeKind::StandPat.is_searched());
        assert!(!OutcomeKind::Tablebase.is_searched());

        assert!(OutcomeKind::StandPat.is_horizon());
        assert!(OutcomeKind::QsearchTail.is_horizon());
        assert!(!OutcomeKind::ProbCut.is_horizon());
        assert!(!OutcomeKind::Full.is_horizon());

        assert!(OutcomeKind::ProbCut.is_speculative());
        assert!(!OutcomeKind::Full.is_speculative());
        assert!(!OutcomeKind::QsearchMove.is_speculative());
    }

    #[test]
    fn move_class_follows_the_picker_order() {
        // The TT move wins regardless of its own shape.
        let tt = MoveEvidence::new(true, true, false, -500, 0);
        assert_eq!(tt.class, MoveClass::TtMove);
        assert_eq!(
            MoveEvidence::new(false, true, false, 0, 0).class,
            MoveClass::GoodCapture
        );
        assert_eq!(
            MoveEvidence::new(false, true, false, -1, 0).class,
            MoveClass::BadCapture
        );
        assert_eq!(
            MoveEvidence::new(false, false, true, 0, 77).class,
            MoveClass::Quiet
        );
        // Neither capture nor quiet (e.g. a quiet promotion) lands in the tail.
        assert_eq!(
            MoveEvidence::new(false, false, false, 0, 0).class,
            MoveClass::BadCapture
        );
        assert_eq!(
            MoveEvidence::new(false, false, true, 0, 77).quiet_history,
            77
        );
    }
}
