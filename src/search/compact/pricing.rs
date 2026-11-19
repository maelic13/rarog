//! The pricing instrument on the compact kernel, in the `pricing` build
//! only (`analysis/pruning_pricing_card_2026-11-04.md`).
//!
//! A sampled decision is reversed by searching what its rule denied, and
//! the probe leaves the measured search as it found it
//! (`super::super::pricing` saves and restores the thread's state, freezes
//! the histories and the corrections, and sends every table store to its
//! scratch table).
//!
//! A plant changes one seed: `RAROG_PRICING_SEED=<name>=<value>[,...]`
//! sets the measured search's table, and with `RAROG_PRICING_CONFINE=1` a
//! probe reads the kernel's own table, so the plant acts only in the decisions
//! it prices.

use std::sync::OnceLock;

use crate::board::{Board, Move, Piece};

use super::super::node::{NodeType, NonPv, Pv};
use super::super::pricing::{
    Changed, Decision, NodeKind, ProbeKind, QCapture, ReSearch, Rule, Verdict,
};
use super::super::{SearchEvent, Searcher};
use super::is_quiet_move;
use super::rules::{SEED_FIELDS, SEEDS, Seeds};

/// A compact node as it was entered, for a probe that searches it again.
pub(super) struct CompactEntry {
    pub(super) depth: i32,
    pub(super) alpha: i32,
    pub(super) beta: i32,
    pub(super) cut_node: bool,
    pub(super) last_critical_ply: usize,
    /// The child ply's cutoff count as the node found it: the searches
    /// before the decision (the null move's, ProbCut's) add to it, and the
    /// node reads it.
    pub(super) child_cutoffs: i32,
    /// The node count at entry: the draw score and the reductions' spread
    /// read its phase, so a re-entry starts from it.
    pub(super) nodes: u64,
}

impl Rule {
    /// The sampling stride's log2 on the compact kernel, set from each
    /// rule's decision count so a compact `bench 13` probes about 10,000
    /// decisions of each.
    pub(super) const fn compact_shift(self) -> u32 {
        match self {
            Self::LmrFailLow | Self::ReverseFutility => 7,
            Self::QuietSee => 6,
            Self::NoisySee
            | Self::QuietFutility
            | Self::HistoryPruning
            | Self::QSee
            | Self::FdsFailLow => 5,
            Self::LateMovePruning | Self::BadNoisyFutility | Self::QCount | Self::ProbCut => 4,
            Self::Razor | Self::NullMove | Self::MultiCut => 3,
        }
    }
}

/// The measured search's seed table: the kernel's with the planted values,
/// read once from `RAROG_PRICING_SEED`.
fn planted() -> &'static Seeds {
    static PLANTED: OnceLock<Seeds> = OnceLock::new();
    PLANTED.get_or_init(|| {
        let mut seeds = Seeds::FITTED;
        let Ok(raw) = std::env::var("RAROG_PRICING_SEED") else {
            return seeds;
        };
        for pair in raw.split(',').filter(|pair| !pair.trim().is_empty()) {
            let (name, value) = pair
                .split_once('=')
                .unwrap_or_else(|| panic!("RAROG_PRICING_SEED: {pair} is not name=value"));
            let value: i32 = value
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("RAROG_PRICING_SEED: {pair} has no integer"));
            let (_, field) = SEED_FIELDS
                .iter()
                .find(|(field, _)| *field == name.trim())
                .unwrap_or_else(|| panic!("RAROG_PRICING_SEED: no seed {name}"));
            *field(&mut seeds) = value;
        }
        seeds
    })
}

impl Searcher {
    /// The seed table this search reads: the planted one, or inside a
    /// confined probe the kernel's own.
    pub(super) fn pricing_seeds(&self) -> &'static Seeds {
        if self.td.pricing.confined_probe() {
            SEEDS
        } else {
            planted()
        }
    }

    /// Whether the compact kernel's decision is in the sample.
    pub(super) fn compact_sampled(&self, rule: Rule, key: u64, mv: Move, depth: i32) -> bool {
        self.pricing_draw(rule, rule.compact_shift(), key, mv, depth)
    }

    /// A node-level decision, priced when sampled: off the root and outside
    /// an exclusion search.
    pub(super) fn compact_price_cut<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        rule: Rule,
        key: u64,
        depth: i32,
        ply: usize,
        excluded: bool,
        entry: &CompactEntry,
        verdict: Verdict,
        poll: &mut P,
    ) {
        if NODE::ROOT || excluded || !self.compact_sampled(rule, key, Move::NULL, depth) {
            return;
        }
        let decision = Decision {
            rule,
            depth,
            ply,
            node: NodeKind::of(NODE::PV, entry.cut_node),
            move_count: 0,
            tail: 0,
            quiet: None,
        };
        self.compact_price_node::<NODE, _>(board, &decision, entry, verdict, poll);
    }

    /// A move a rule pruned, priced when sampled outside an exclusion search
    /// by its full-depth search: a ply less, the child's node type flipped.
    pub(super) fn compact_price_prune<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        rule: Rule,
        key: u64,
        mv: Move,
        piece: Piece,
        depth: i32,
        ply: usize,
        cut_node: bool,
        excluded: bool,
        move_count: i32,
        tail: usize,
        alpha: i32,
        last_critical_ply: usize,
        poll: &mut P,
    ) {
        if excluded || !self.compact_sampled(rule, key, mv, depth) {
            return;
        }
        let decision = Decision {
            rule,
            depth,
            ply,
            node: NodeKind::of(NODE::PV, cut_node),
            move_count,
            tail,
            quiet: Some(is_quiet_move(mv)),
        };
        let search = ReSearch {
            new_depth: depth - 1,
            alpha,
            cut_node: !cut_node,
            last_critical_ply,
        };
        self.compact_price_pruned_move(board, &decision, mv, piece, &search, poll);
    }

    /// A reduced move that failed low, made, priced when sampled outside an
    /// exclusion search by `search`, the search its reduction denied.
    pub(super) fn compact_price_reduction<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        rule: Rule,
        key: u64,
        mv: Move,
        depth: i32,
        ply: usize,
        cut_node: bool,
        excluded: bool,
        move_count: i32,
        search: &ReSearch,
        poll: &mut P,
    ) {
        if excluded || !self.compact_sampled(rule, key, mv, depth) {
            return;
        }
        let decision = Decision {
            rule,
            depth,
            ply,
            node: NodeKind::of(NODE::PV, cut_node),
            move_count,
            tail: 0,
            quiet: Some(is_quiet_move(mv)),
        };
        self.compact_price_reduced_move(board, &decision, search, poll);
    }

    /// A move quiescence skipped, priced when sampled. Quiescence has no
    /// depth; its decisions are drawn and recorded at depth 0.
    pub(super) fn compact_price_q<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        rule: Rule,
        key: u64,
        mv: Move,
        ply: usize,
        move_count: i32,
        alpha: i32,
        beta: i32,
        poll: &mut P,
    ) {
        if !self.compact_sampled(rule, key, mv, 0) {
            return;
        }
        let decision = Decision {
            rule,
            depth: 0,
            ply,
            node: NodeKind::of(NODE::PV, false),
            move_count,
            tail: 0,
            quiet: Some(is_quiet_move(mv)),
        };
        let capture = QCapture {
            mv,
            piece: board.moving_piece(mv),
            alpha,
            beta,
        };
        self.compact_price_q_move::<NODE, _>(board, &decision, &capture, poll);
    }

    /// A node a rule cut or razored, searched again at its entry depth and
    /// window with the rule off at its ply; with `RAROG_PRICING_IDENTITY=1`
    /// a second time with nothing off, which must reproduce the decision.
    pub(super) fn compact_price_node<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        decision: &Decision,
        entry: &CompactEntry,
        verdict: Verdict,
        poll: &mut P,
    ) {
        self.compact_node_probe::<NODE, _>(
            board,
            decision,
            entry,
            verdict,
            ProbeKind::Priced,
            poll,
        );
        if self.td.pricing.identity() {
            self.compact_node_probe::<NODE, _>(
                board,
                decision,
                entry,
                verdict,
                ProbeKind::Identity,
                poll,
            );
        }
    }

    fn compact_node_probe<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        decision: &Decision,
        entry: &CompactEntry,
        verdict: Verdict,
        kind: ProbeKind,
        poll: &mut P,
    ) {
        let saved = self.probe_begin();
        self.probe_switches(decision, kind);
        self.td.stack[decision.ply + 1].cutoff_count = entry.child_cutoffs;
        let resume = self.td.nodes;
        self.td.nodes = entry.nodes;
        let score = self.compact_negamax::<NODE, _>(
            board,
            entry.alpha,
            entry.beta,
            entry.depth,
            entry.cut_node,
            decision.ply,
            entry.last_critical_ply,
            Move::NULL,
            poll,
        );
        // The probe's own nodes, counted on from where the search stood.
        self.td.nodes = resume + (self.td.nodes - entry.nodes);
        let nodes = self.probe_end(&saved);
        self.push_record(
            decision,
            nodes,
            verdict.wrong(score),
            Changed::NotRead,
            kind,
        );
    }

    /// A move a rule pruned, searched as the move loop would have searched
    /// it at full depth: pushed, made, searched with the null window above
    /// alpha, unmade.
    pub(super) fn compact_price_pruned_move<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        decision: &Decision,
        mv: Move,
        piece: Piece,
        search: &ReSearch,
        poll: &mut P,
    ) {
        let saved = self.probe_begin();
        let ply = decision.ply;
        self.compact_push_move(board, ply, mv, piece, decision.move_count);
        board.make_move(mv);
        let score = self.compact_re_search(board, ply, search, poll);
        board.unmake_move(mv);
        let nodes = self.probe_end(&saved);
        self.record_raise(decision, nodes, score > search.alpha);
    }

    /// A reduced move that failed low, still made, searched again at the
    /// depth its reduction denied.
    pub(super) fn compact_price_reduced_move<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        decision: &Decision,
        search: &ReSearch,
        poll: &mut P,
    ) {
        let saved = self.probe_begin();
        let score = self.compact_re_search(board, decision.ply, search, poll);
        let nodes = self.probe_end(&saved);
        self.record_raise(decision, nodes, score > search.alpha);
    }

    fn compact_re_search<P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        ply: usize,
        search: &ReSearch,
        poll: &mut P,
    ) -> i32 {
        -self.compact_negamax::<NonPv, _>(
            board,
            -search.alpha - 1,
            -search.alpha,
            search.new_depth,
            search.cut_node,
            ply + 1,
            search.last_critical_ply,
            Move::NULL,
            poll,
        )
    }

    /// A move quiescence skipped, searched as quiescence searches its moves.
    pub(super) fn compact_price_q_move<NODE: NodeType, P: FnMut() -> SearchEvent + ?Sized>(
        &mut self,
        board: &mut Board,
        decision: &Decision,
        capture: &QCapture,
        poll: &mut P,
    ) {
        let saved = self.probe_begin();
        let ply = decision.ply;
        self.compact_push_move(board, ply, capture.mv, capture.piece, decision.move_count);
        board.make_move(capture.mv);
        let score = if NODE::PV {
            -self.compact_quiescence::<Pv, _>(board, -capture.beta, -capture.alpha, ply + 1, poll)
        } else {
            -self.compact_quiescence::<NonPv, _>(
                board,
                -capture.beta,
                -capture.alpha,
                ply + 1,
                poll,
            )
        };
        board.unmake_move(capture.mv);
        let nodes = self.probe_end(&saved);
        self.push_record(
            decision,
            nodes,
            score > capture.alpha,
            Changed::NotRead,
            ProbeKind::Priced,
        );
    }
}
