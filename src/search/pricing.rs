//! Pricing what pruning and reductions hide, in the `pricing` instrument
//! build only (`analysis/pruning_pricing_card_2026-11-04.md`).
//!
//! A sampled decision is reversed in a probe: the move a rule pruned, or the
//! reduced move that failed low, is searched at the depth the rule denied it;
//! a node a rule cut or razored is searched again with that rule off at its
//! ply. The probe must leave the measured search exactly as it found it, so
//! while it runs the main table is read and never written (a scratch table
//! takes its stores), the histories and corrections are frozen, and the stack,
//! PV and node count are restored afterwards. With sampling on, the main
//! search's node count therefore stays its fingerprint.
//!
//! A plant (a seed set by `RAROG_PRICING_SEED` to make one rule more
//! aggressive) validates a label. With `RAROG_PRICING_CONFINE=1` each probe
//! searches with the kernel's own seeds, so the plant acts only in the
//! measured search and not inside the probe that prices its decisions.
//!
//! Two checks of the probes themselves. With `RAROG_PRICING_IDENTITY=1`
//! each node-level decision is probed a second time with nothing switched
//! off, reading the main table alone and storing nothing: it must reproduce
//! the decision. With `RAROG_PRICING_COLD=1` probes read and write the
//! scratch table alone, so no result of the measured search reaches them.

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::OnceLock;

use crate::board::{Move, Piece};
use crate::tt::{TranspositionTable, TtEntry};

use super::stack::{PlyArray, StackEntry};
use super::{MAX_PLY, Searcher};

/// The scratch table's size: probes are shallow beside the main search.
const SCRATCH_MB: usize = 16;

/// A decision the instrument can price.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum Rule {
    Razor,
    ReverseFutility,
    NullMove,
    ProbCut,
    LateMovePruning,
    QuietFutility,
    BadNoisyFutility,
    HistoryPruning,
    QuietSee,
    NoisySee,
    LmrFailLow,
    QCount,
    QSee,
    /// The compact kernel's singular multi-cut: a node cut by the exclusion
    /// search's score.
    MultiCut,
    /// The compact kernel's full-depth search, reduced, that failed low.
    FdsFailLow,
}

impl Rule {
    const fn name(self) -> &'static str {
        match self {
            Self::Razor => "razor",
            Self::ReverseFutility => "rfp",
            Self::NullMove => "nmp",
            Self::ProbCut => "probcut",
            Self::LateMovePruning => "lmp",
            Self::QuietFutility => "futility",
            Self::BadNoisyFutility => "bnfp",
            Self::HistoryPruning => "history",
            Self::QuietSee => "see_quiet",
            Self::NoisySee => "see_noisy",
            Self::LmrFailLow => "lmr",
            Self::QCount => "q_count",
            Self::QSee => "q_see",
            Self::MultiCut => "multicut",
            Self::FdsFailLow => "fds",
        }
    }
}

/// The node's type as the search assigned it.
#[derive(Copy, Clone)]
pub(super) enum NodeKind {
    Pv,
    Cut,
    All,
}

impl NodeKind {
    pub(super) const fn of(pv: bool, cut_node: bool) -> Self {
        if pv {
            Self::Pv
        } else if cut_node {
            Self::Cut
        } else {
            Self::All
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Pv => "pv",
            Self::Cut => "cut",
            Self::All => "all",
        }
    }
}

/// For a hidden move that raised alpha: whether its node then failed low,
/// so the move would have changed the node's result.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(in crate::search) enum Changed {
    /// Not a raising move-level label.
    NotRead,
    /// Its node has not exited yet.
    Pending,
    Yes,
    No,
}

/// One decision as the search took it.
pub(super) struct Decision {
    pub(super) rule: Rule,
    pub(super) depth: i32,
    pub(super) ply: usize,
    pub(super) node: NodeKind,
    pub(super) move_count: i32,
    /// The moves the decision also removed behind the priced one.
    pub(super) tail: usize,
    /// Whether the priced move is quiet; none for a node-level rule.
    pub(super) quiet: Option<bool>,
}

/// The null-window search a hidden move would have had.
pub(super) struct ReSearch {
    pub(super) new_depth: i32,
    pub(super) alpha: i32,
    pub(super) cut_node: bool,
    pub(super) last_critical_ply: usize,
}

/// What a node-level rule returned, and the bound it is judged against.
#[derive(Copy, Clone)]
pub(super) enum Verdict {
    /// A cut at or above beta: wrong when the probe scores below `beta`.
    Cut { beta: i32 },
    /// A score read from quiescence instead of a search: wrong when the
    /// probe and the returned score fall on different sides of `alpha`.
    Shortcut { returned: i32, alpha: i32 },
}

/// A capture quiescence skipped, with the bounds it would have had.
pub(super) struct QCapture {
    pub(super) mv: Move,
    pub(super) piece: Piece,
    pub(super) alpha: i32,
    pub(super) beta: i32,
}

struct Record {
    rule: Rule,
    depth: i32,
    ply: usize,
    node: NodeKind,
    move_count: i32,
    tail: usize,
    quiet: Option<bool>,
    nodes: u64,
    label: bool,
    changed: Changed,
    kind: ProbeKind,
}

/// What a probe is for.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(in crate::search) enum ProbeKind {
    /// The decision reversed: the rule off at its ply, or the hidden move
    /// searched.
    Priced,
    /// The node searched again with nothing off: it must reproduce the
    /// decision.
    Identity,
}

/// The tables a probe reads; it stores into the scratch table unless it
/// reads the main one alone.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum ProbeTables {
    /// The deeper of the main and scratch entries, the main one on a tie.
    Warm,
    /// The scratch table alone: no result of the measured search.
    Cold,
    /// The main table alone, nothing stored: what the decision read.
    Identity,
}

/// The thread state a probe saves and restores, beyond its buffers.
pub(in crate::search) struct Saved {
    nodes: u64,
    seldepth: usize,
    nmp_min_ply: i32,
    #[cfg(test)]
    moves_made: u64,
    /// The compact kernel's rule counts, which a probe's nodes would add to.
    #[cfg(feature = "probes")]
    rates: super::compact::probes::Rates,
}

/// One thread's instrument: the switches a probe reads, its scratch table,
/// the save buffers and the records of the current search.
pub(in crate::search) struct Pricing {
    /// A probe is running: no table write reaches the main table, no
    /// history moves, and no further decision is sampled.
    pub(super) probing: bool,
    /// The rule a node-level probe switched off, and the ply it is off at.
    pub(super) off: Option<(usize, Rule)>,
    /// Probes search with the kernel's own seeds, so a planted seed acts
    /// only in the measured search.
    confine: bool,
    /// Probes read the scratch table alone.
    cold: bool,
    /// Node-level decisions get an identity probe too.
    identity: bool,
    /// The running probe's tables.
    tables: ProbeTables,
    scratch: Option<TranspositionTable>,
    records: Vec<Record>,
    /// Raising move-level records per ply still waiting for their node.
    pending: PlyArray<u16>,
    stack: Option<Box<PlyArray<StackEntry>>>,
    pv_table: Option<Box<PlyArray<[Move; MAX_PLY]>>>,
    pv_len: Option<Box<PlyArray<usize>>>,
    probe_nodes: u64,
}

impl Pricing {
    /// Whether a probe is running with the compiled-in parameters, so a
    /// plant acts only in the measured search.
    pub(in crate::search) fn confined_probe(&self) -> bool {
        self.probing && self.confine
    }

    /// Whether node-level decisions also get an identity probe.
    pub(in crate::search) fn identity(&self) -> bool {
        self.identity
    }
}

impl Default for Pricing {
    fn default() -> Self {
        Self {
            probing: false,
            off: None,
            confine: confine_from_env(),
            cold: cold_from_env(),
            identity: identity_from_env(),
            tables: ProbeTables::Warm,
            scratch: None,
            records: Vec::new(),
            pending: PlyArray::new(0),
            stack: None,
            pv_table: None,
            pv_len: None,
            probe_nodes: 0,
        }
    }
}

/// The extra stride shift from `RAROG_PRICING`: unset is off, `0` the
/// card's rates, `1` each halved.
fn extra_shift() -> Option<u32> {
    static SHIFT: OnceLock<Option<u32>> = OnceLock::new();
    *SHIFT.get_or_init(|| {
        std::env::var("RAROG_PRICING")
            .ok()
            .and_then(|raw| raw.trim().parse::<u32>().ok())
            .filter(|shift| *shift < 16)
    })
}

/// Whether probes search with the compiled-in parameters, from
/// `RAROG_PRICING_CONFINE` (`1` confines a planted option to the measured
/// search; unset, probes search with the options set).
fn confine_from_env() -> bool {
    static CONFINE: OnceLock<bool> = OnceLock::new();
    env_switch(&CONFINE, "RAROG_PRICING_CONFINE")
}

/// Whether probes read the scratch table alone, from `RAROG_PRICING_COLD`.
fn cold_from_env() -> bool {
    static COLD: OnceLock<bool> = OnceLock::new();
    env_switch(&COLD, "RAROG_PRICING_COLD")
}

/// Whether node-level decisions get an identity probe, from
/// `RAROG_PRICING_IDENTITY`.
fn identity_from_env() -> bool {
    static IDENTITY: OnceLock<bool> = OnceLock::new();
    env_switch(&IDENTITY, "RAROG_PRICING_IDENTITY")
}

/// A switch read once from the environment: `1` is on, anything else off.
fn env_switch(cell: &'static OnceLock<bool>, var: &str) -> bool {
    *cell.get_or_init(|| std::env::var(var).is_ok_and(|raw| raw.trim() == "1"))
}

/// The file the records are appended to, from `RAROG_PRICING_OUT`.
fn out_path() -> Option<&'static str> {
    static PATH: OnceLock<Option<String>> = OnceLock::new();
    PATH.get_or_init(|| std::env::var("RAROG_PRICING_OUT").ok())
        .as_deref()
}

fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

impl Searcher {
    /// The sample's draw at a stride of `2^shift` decisions, before the
    /// extra shift `RAROG_PRICING` adds.
    pub(in crate::search) fn pricing_draw(
        &self,
        rule: Rule,
        shift: u32,
        key: u64,
        mv: Move,
        depth: i32,
    ) -> bool {
        let Some(extra) = extra_shift() else {
            return false;
        };
        if self.td.pricing.probing || self.shared.threads != 1 || self.td.thread_id != 0 {
            return false;
        }
        let depth_bits = u64::from(depth.clamp(0, 255).cast_unsigned());
        let drawn = mix(key ^ (u64::from(mv.0) << 16) ^ ((rule as u64) << 32) ^ (depth_bits << 40));
        let stride = (1u64 << (shift + extra)) - 1;
        drawn & stride == 0
    }

    /// The table entry a probe sees, by its tables.
    pub(super) fn pricing_tt_probe(&self, key: u64) -> Option<TtEntry> {
        let scratch = || {
            self.td
                .pricing
                .scratch
                .as_ref()
                .and_then(|table| table.probe(key))
        };
        match self.td.pricing.tables {
            ProbeTables::Warm => match (self.shared.tt.probe(key), scratch()) {
                (Some(main), Some(scratch)) if scratch.depth > main.depth => Some(scratch),
                (main, scratch) => main.or(scratch),
            },
            ProbeTables::Cold => scratch(),
            ProbeTables::Identity => self.shared.tt.probe(key),
        }
    }

    /// The table a probe writes; none for an identity probe.
    pub(super) fn pricing_scratch(&mut self) -> Option<&mut TranspositionTable> {
        let pricing = &mut self.td.pricing;
        if pricing.tables == ProbeTables::Identity {
            return None;
        }
        Some(
            pricing
                .scratch
                .get_or_insert_with(|| TranspositionTable::new(SCRATCH_MB)),
        )
    }

    pub(in crate::search) fn probe_begin(&mut self) -> Saved {
        let td = &mut self.td;
        let pricing = &mut td.pricing;
        match &mut pricing.stack {
            Some(saved) => saved.as_mut().clone_from(&td.stack),
            None => pricing.stack = Some(Box::new(td.stack.clone())),
        }
        match &mut pricing.pv_table {
            Some(saved) => saved.as_mut().clone_from(&td.pv_table),
            None => pricing.pv_table = Some(Box::new(td.pv_table.clone())),
        }
        match &mut pricing.pv_len {
            Some(saved) => saved.as_mut().clone_from(&td.pv_len),
            None => pricing.pv_len = Some(Box::new(td.pv_len.clone())),
        }
        if pricing.scratch.is_none() {
            pricing.scratch = Some(TranspositionTable::new(SCRATCH_MB));
        }
        pricing.probing = true;
        pricing.tables = if pricing.cold {
            ProbeTables::Cold
        } else {
            ProbeTables::Warm
        };
        td.hist.frozen = true;
        Saved {
            nodes: td.nodes,
            seldepth: td.seldepth,
            nmp_min_ply: td.nmp_min_ply,
            #[cfg(test)]
            moves_made: td.moves_made,
            #[cfg(feature = "probes")]
            rates: td.rates.clone(),
        }
    }

    /// Restore what the probe changed; returns the probe's nodes.
    pub(in crate::search) fn probe_end(&mut self, saved: &Saved) -> u64 {
        let td = &mut self.td;
        let pricing = &mut td.pricing;
        let restore = "probe_begin filled the buffers";
        td.stack
            .clone_from(pricing.stack.as_deref().expect(restore));
        td.pv_table
            .clone_from(pricing.pv_table.as_deref().expect(restore));
        td.pv_len
            .clone_from(pricing.pv_len.as_deref().expect(restore));
        pricing.probing = false;
        pricing.off = None;
        td.hist.frozen = false;
        let nodes = td.nodes - saved.nodes;
        td.nodes = saved.nodes;
        td.seldepth = saved.seldepth;
        td.nmp_min_ply = saved.nmp_min_ply;
        #[cfg(test)]
        {
            td.moves_made = saved.moves_made;
        }
        #[cfg(feature = "probes")]
        td.rates.clone_from(&saved.rates);
        pricing.probe_nodes += nodes;
        nodes
    }

    pub(in crate::search) fn push_record(
        &mut self,
        decision: &Decision,
        nodes: u64,
        label: bool,
        changed: Changed,
        kind: ProbeKind,
    ) {
        let pricing = &mut self.td.pricing;
        if pricing.records.capacity() == 0 {
            pricing.records.reserve(1 << 16);
        }
        if changed == Changed::Pending {
            pricing.pending[decision.ply] += 1;
        }
        pricing.records.push(Record {
            rule: decision.rule,
            depth: decision.depth,
            ply: decision.ply,
            node: decision.node,
            move_count: decision.move_count,
            tail: decision.tail,
            quiet: decision.quiet,
            nodes,
            label,
            changed,
            kind,
        });
    }

    pub(in crate::search) fn record_raise(
        &mut self,
        decision: &Decision,
        nodes: u64,
        raises: bool,
    ) {
        let changed = if raises {
            Changed::Pending
        } else {
            Changed::NotRead
        };
        self.push_record(decision, nodes, raises, changed, ProbeKind::Priced);
    }

    /// A priced probe switches the rule off at its ply; an identity probe
    /// switches nothing and reads the main table alone.
    pub(in crate::search) fn probe_switches(&mut self, decision: &Decision, kind: ProbeKind) {
        match kind {
            ProbeKind::Priced => self.td.pricing.off = Some((decision.ply, decision.rule)),
            ProbeKind::Identity => self.td.pricing.tables = ProbeTables::Identity,
        }
    }

    /// Close the raising records of the node at `ply` as it exits: whether
    /// it failed low, so a hidden move that raised would have changed it.
    pub(super) fn pricing_close(&mut self, ply: usize, failed_low: bool) {
        let pricing = &mut self.td.pricing;
        let mut left = pricing.pending[ply];
        // A probe's own nodes close nothing: no probe nests, so every
        // pending record belongs to the main search.
        if left == 0 || pricing.probing {
            return;
        }
        for record in pricing.records.iter_mut().rev() {
            if record.ply == ply && record.changed == Changed::Pending {
                record.changed = if failed_low {
                    Changed::Yes
                } else {
                    Changed::No
                };
                left -= 1;
                if left == 0 {
                    break;
                }
            }
        }
        pricing.pending[ply] = 0;
    }

    /// Append this search's records to `RAROG_PRICING_OUT`, with one summary
    /// line, and start the next search empty.
    pub(super) fn pricing_dump(&mut self) {
        let main_nodes = self.td.nodes;
        let pricing = &mut self.td.pricing;
        if let Some(path) = out_path()
            && extra_shift().is_some()
            && let Err(err) = write_records(path, main_nodes, pricing)
        {
            eprintln!("pricing: cannot write {path}: {err}");
        }
        pricing.records.clear();
        pricing.probe_nodes = 0;
        pricing.pending = PlyArray::new(0);
    }
}

impl Verdict {
    pub(in crate::search) const fn wrong(self, probe: i32) -> bool {
        match self {
            Self::Cut { beta } => probe < beta,
            Self::Shortcut { returned, alpha } => (probe > alpha) != (returned > alpha),
        }
    }
}

fn write_records(path: &str, main_nodes: u64, pricing: &Pricing) -> std::io::Result<()> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut out = BufWriter::new(file);
    writeln!(
        out,
        "# search nodes {main_nodes} probes {} probe_nodes {}",
        pricing.records.len(),
        pricing.probe_nodes
    )?;
    for r in &pricing.records {
        let changed = match r.changed {
            Changed::NotRead => "-",
            Changed::Pending => "?",
            Changed::Yes => "1",
            Changed::No => "0",
        };
        let quiet = match r.quiet {
            None => "-",
            Some(true) => "1",
            Some(false) => "0",
        };
        writeln!(
            out,
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{changed}\t{quiet}",
            r.rule.name(),
            match r.kind {
                ProbeKind::Priced => "",
                ProbeKind::Identity => "_identity",
            },
            r.depth,
            r.ply,
            r.node.name(),
            r.move_count,
            r.tail,
            r.nodes,
            u8::from(r.label),
        )?;
    }
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tt::{Bound, TtStore};

    /// A confined probe searches with the kernel's own seeds and hands the
    /// planted table back when it closes: the switch the seed table reads.
    #[test]
    fn a_confined_probe_searches_with_the_compiled_in_parameters() {
        let mut searcher = Searcher::default();
        searcher.td.pricing.confine = true;
        for _ in 0..2 {
            assert!(!searcher.td.pricing.confined_probe(), "the measured search");
            let saved = searcher.probe_begin();
            assert!(searcher.td.pricing.confined_probe(), "the probe");
            searcher.probe_end(&saved);
        }
        assert!(!searcher.td.pricing.confined_probe());
    }

    fn entry_at(depth: i32, key: u64) -> TtStore {
        TtStore {
            key,
            depth,
            score: 10 * depth,
            bound: Bound::Exact,
            mv: Move::NULL,
            ply: 0,
            static_eval: 0,
            is_pv: false,
        }
    }

    /// A warm probe reads the deeper entry, a cold one the scratch table's
    /// alone, an identity probe the main table's alone and stores nothing.
    #[test]
    fn each_probe_reads_its_own_tables() {
        let key = 0x1234_5678_9abc_def0;
        let mut searcher = Searcher::default();
        searcher.td.pricing.cold = false;
        searcher.shared.tt.store(entry_at(3, key));
        let saved = searcher.probe_begin();
        assert_eq!(searcher.td.pricing.tables, ProbeTables::Warm);
        searcher
            .pricing_scratch()
            .expect("a warm probe stores")
            .store(entry_at(5, key));
        // The stored score names the entry: ten times its depth.
        let which = |entry: Option<TtEntry>| entry.map(|e| i32::from(e.score) / 10);
        assert_eq!(which(searcher.pricing_tt_probe(key)), Some(5));
        searcher.td.pricing.tables = ProbeTables::Cold;
        assert_eq!(which(searcher.pricing_tt_probe(key)), Some(5));
        searcher.td.pricing.tables = ProbeTables::Identity;
        assert_eq!(which(searcher.pricing_tt_probe(key)), Some(3));
        assert!(searcher.pricing_scratch().is_none());
        searcher.probe_end(&saved);

        let other = key ^ 0xffff_0000_0000_0000;
        searcher.shared.tt.store(entry_at(4, other));
        searcher.td.pricing.cold = true;
        let saved = searcher.probe_begin();
        assert_eq!(searcher.td.pricing.tables, ProbeTables::Cold);
        assert!(searcher.pricing_tt_probe(other).is_none());
        searcher.probe_end(&saved);
    }

    /// A priced probe switches its rule off at its ply; an identity probe
    /// switches nothing and reads the main table alone.
    #[test]
    fn an_identity_probe_switches_nothing_off() {
        let mut searcher = Searcher::default();
        let decision = Decision {
            rule: Rule::Razor,
            depth: 1,
            ply: 4,
            node: NodeKind::of(false, true),
            move_count: 0,
            tail: 0,
            quiet: None,
        };
        let saved = searcher.probe_begin();
        searcher.probe_switches(&decision, ProbeKind::Identity);
        assert_eq!(searcher.td.pricing.off, None);
        assert_eq!(searcher.td.pricing.tables, ProbeTables::Identity);
        searcher.probe_end(&saved);
        let saved = searcher.probe_begin();
        searcher.probe_switches(&decision, ProbeKind::Priced);
        assert_eq!(searcher.td.pricing.off, Some((4, Rule::Razor)));
        assert_ne!(searcher.td.pricing.tables, ProbeTables::Identity);
        searcher.probe_end(&saved);
        assert_eq!(searcher.td.pricing.off, None);
    }

    /// Unconfined, a probe searches with the planted seeds.
    #[test]
    fn an_unconfined_probe_keeps_the_set_parameters() {
        let mut searcher = Searcher::default();
        searcher.td.pricing.confine = false;
        let saved = searcher.probe_begin();
        assert!(!searcher.td.pricing.confined_probe());
        searcher.probe_end(&saved);
    }
}
