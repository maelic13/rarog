//! 9.4 regression guard: `Hash` is a contract.
//!
//! Before 9.4, `make_shared` inherited the LOCAL cluster count. A
//! `SharedCluster` is 64 B against `LocalCluster`'s 32 B, so the first
//! multi-threaded search silently allocated twice the `Hash` the user set —
//! and, because the local table was still alive during the conversion, peaked
//! at three times it, at `go` time, mid-game. Nothing reported this; it would
//! have surfaced as swapping and time losses in a tournament and been
//! diagnosed as engine weakness.

use rarog::tt::TranspositionTable;

const MIB: usize = 1024 * 1024;

#[test]
fn local_table_fits_the_hash_budget() {
    for mb in [1usize, 16, 64, 256] {
        let tt = TranspositionTable::new(mb);
        assert!(
            tt.allocated_bytes() <= mb * MIB,
            "local table at Hash={mb} MiB allocated {} bytes, budget {}",
            tt.allocated_bytes(),
            mb * MIB
        );
    }
}

#[test]
fn shared_table_fits_the_same_budget_as_local() {
    for mb in [1usize, 16, 64, 256] {
        let mut tt = TranspositionTable::new(mb);
        let local = tt.allocated_bytes();
        tt.make_shared(mb);
        let shared = tt.allocated_bytes();
        assert!(
            shared <= mb * MIB,
            "shared table at Hash={mb} MiB allocated {shared} bytes, budget {}",
            mb * MIB
        );
        // The pre-9.4 bug in one line: shared must not exceed local.
        assert!(
            shared <= local,
            "going multi-threaded grew the table from {local} to {shared} bytes \
             at Hash={mb} MiB"
        );
    }
}

/// Bytes are the `Hash` contract; ENTRIES are what the search spends.
///
/// The shared table honoured the byte budget while storing far fewer
/// positions: an `AtomicTtEntry` is 16 B against `TtEntry`'s 10 B, and the
/// 64 B cluster held only 3 of them (48 B used, 16 B padding), so going
/// multi-threaded silently HALVED the number of positions the engine could
/// remember. Nothing reported it — it surfaced only as `hashfull` hitting
/// ~970/1000 within a 3-second Threads=4 search.
#[test]
fn shared_table_does_not_shrink_entry_capacity() {
    for mb in [1usize, 16, 64, 256] {
        let mut tt = TranspositionTable::new(mb);
        let local = tt.capacity_entries();
        tt.make_shared(mb);
        let shared = tt.capacity_entries();
        assert!(
            shared * 2 >= local,
            "going multi-threaded at Hash={mb} MiB cut capacity from {local} to \
             {shared} entries (more than a 2x loss)"
        );
    }
}

/// `Hash` is the size asked for, in both forms: these were floored to 64 and
/// 256 when the table's size had to be a power of two.
#[test]
fn hash_need_not_be_a_power_of_two() {
    for mb in [100, 300] {
        let mut tt = TranspositionTable::new(mb);
        assert_eq!(tt.allocated_bytes(), mb << 20, "local {mb}");
        let local_entries = tt.capacity_entries();
        tt.make_shared(mb);
        assert_eq!(tt.allocated_bytes(), mb << 20, "shared {mb}");
        assert_eq!(tt.capacity_entries(), local_entries, "{mb}");
    }
}

#[test]
fn make_shared_is_idempotent() {
    let mut tt = TranspositionTable::new(64);
    tt.make_shared(64);
    let once = tt.allocated_bytes();
    tt.make_shared(64);
    assert_eq!(tt.allocated_bytes(), once, "second make_shared reallocated");
}
