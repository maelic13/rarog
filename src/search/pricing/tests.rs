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
