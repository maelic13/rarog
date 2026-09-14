//! Behaviour of the SHARED (multi-threaded) transposition table.
//!
//! The 1-thread bench fingerprint — the project's usual safety net — cannot
//! see this backend at all: it is only ever constructed when `Threads > 1`.
//! So the shared store/probe path had no direct coverage while it was being
//! repacked from 16 B slots (`key ^ data`, `data`) down to 10 B (payload plus
//! a `key16 ^ fold16(data)` tag). These tests pin the properties that repack
//! had to preserve: what goes in comes back out bit-exact, and what was never
//! stored is essentially never returned.

use rarog::board::Move;
use rarog::tt::{Bound, TranspositionTable, TtStore};

const ENTRIES: u32 = 5_000;

fn shared_table(mb: usize) -> TranspositionTable {
    let mut tt = TranspositionTable::new(mb);
    tt.make_shared(mb);
    tt
}

/// Distinct, well-spread keys — the cluster index takes the low bits and the
/// verification tag the top 16, so a bare counter would leave the tag constant
/// and never exercise it.
fn key_for(i: u32) -> u64 {
    u64::from(i)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .rotate_left(17)
}

fn depth_for(i: u32) -> i32 {
    i32::try_from(i % 60).expect("in range") - 1 // includes the -1 qsearch depth
}

fn score_for(i: u32) -> i32 {
    i32::try_from(i % 2_000).expect("in range") - 1_000 // spans negative scores
}

fn eval_for(i: u32) -> i32 {
    500 - i32::try_from(i % 1_000).expect("in range")
}

fn move_for(i: u32) -> Move {
    Move(u16::try_from(i % 30_000).expect("in range") | 1) // never null
}

#[test]
fn shared_store_then_probe_round_trips_every_field() {
    let mut tt = shared_table(16);

    for i in 0..ENTRIES {
        tt.store(TtStore {
            key: key_for(i),
            depth: depth_for(i),
            score: score_for(i),
            bound: match i % 3 {
                0 => Bound::Exact,
                1 => Bound::Lower,
                _ => Bound::Upper,
            },
            mv: move_for(i),
            ply: 0,
            static_eval: eval_for(i),
            is_pv: i % 2 == 0,
        });
    }

    // 16 MiB holds ~1.6M slots, so 5k entries cannot have been evicted.
    for i in 0..ENTRIES {
        let entry = tt
            .probe(key_for(i))
            .unwrap_or_else(|| panic!("entry {i} vanished from the shared table"));
        assert_eq!(i32::from(entry.depth), depth_for(i), "depth for {i}");
        assert_eq!(i32::from(entry.score), score_for(i), "score for {i}");
        assert_eq!(
            i32::from(entry.static_eval),
            eval_for(i),
            "static_eval for {i}"
        );
        assert_eq!(entry.best_move(), Some(move_for(i)), "move for {i}");
        assert_eq!(entry.is_pv_node(), i % 2 == 0, "pv bit for {i}");
        assert!(entry.bound().is_some(), "bound for {i}");
    }
}

#[test]
fn shared_probe_almost_never_matches_keys_never_stored() {
    let mut tt = shared_table(16);

    for i in 0..ENTRIES {
        tt.store(TtStore {
            key: key_for(i),
            depth: 10,
            score: 42,
            bound: Bound::Exact,
            mv: Move(0x1234),
            ply: 0,
            static_eval: 7,
            is_pv: false,
        });
    }

    // Disjoint key space. Verification is 16-bit — the same strength the
    // single-threaded table has always had — so a small collision rate is by
    // design; anything approaching 1% would mean the tag is not being checked.
    let probes: usize = 20_000;
    let false_hits = (0..probes)
        .filter(|i| {
            let far = u32::try_from(i + 1_000_000).expect("in range");
            tt.probe(key_for(far)).is_some()
        })
        .count();
    assert!(
        false_hits * 100 < probes,
        "{false_hits}/{probes} probes matched entries that were never stored"
    );
}

#[test]
fn shared_table_starts_empty() {
    let tt = shared_table(4);
    for i in 0..1_000 {
        assert!(
            tt.probe(key_for(i)).is_none(),
            "fresh shared table returned an entry for key {i}"
        );
    }
    assert_eq!(tt.hashfull(), 0, "fresh shared table reports non-zero fill");
}
