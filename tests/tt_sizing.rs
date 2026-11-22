//! `Hash` is a contract: the table's bytes stay within it in both forms.
//!
//! A `SharedCluster` is 64 B against `LocalCluster`'s 32 B, so `make_shared`
//! must size the shared table from the byte budget, not inherit the local
//! cluster count: that would silently allocate twice the `Hash` the user set,
//! and peak at three times it while the local table is still alive during the
//! conversion, which a tournament sees as swapping and time losses.

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
        // Shared must not exceed local.
        assert!(
            shared <= local,
            "going multi-threaded grew the table from {local} to {shared} bytes \
             at Hash={mb} MiB"
        );
    }
}

/// Bytes are the `Hash` contract; ENTRIES are what the search spends.
///
/// Two backends can honour the same byte budget while one stores far fewer
/// positions. The shared table must hold at least half the local table's
/// entries, so going multi-threaded does not silently shrink what the engine
/// can remember.
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

/// `Hash` is the size asked for, in both forms, not the power of two below it.
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
