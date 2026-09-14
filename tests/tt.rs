//! Transposition table: store, probe, replacement, clearing, mate-score conversion and `hashfull`.

use rarog::board::Move;
use rarog::eval::MATE_SCORE;
use rarog::tt::{Bound, TranspositionTable, TtStore, score_from_tt, score_to_tt};

#[test]
fn transposition_table_store_probe_replace_clear_and_mate_scores() {
    let mut table = TranspositionTable::new(1);
    let key = 0x1234_0000_0000_0000;
    let best = Move::from_uci("e2e4").expect("valid UCI move");

    table.store(TtStore {
        key,
        depth: 5,
        score: 123,
        bound: Bound::Exact,
        mv: best,
        ply: 0,
        static_eval: 42,
        is_pv: false,
    });
    let entry = table.probe(key).expect("entry must be stored");
    assert_eq!(entry.score, 123);
    assert_eq!(entry.static_eval, 42);
    assert_eq!(entry.depth, 5);
    assert_eq!(entry.bound(), Some(Bound::Exact));
    assert_eq!(entry.best_move(), Some(best));

    table.store(TtStore {
        key: 0x5678_0000_0000_0001,
        depth: 1,
        score: 1,
        bound: Bound::Exact,
        mv: best,
        ply: 0,
        static_eval: 0,
        is_pv: false,
    });
    assert!(table.hashfull() > 0);

    assert!(!table.resize(usize::MAX));
    assert!(
        table.probe(key).is_some(),
        "failed resize must keep the current table"
    );

    table.make_shared(1);
    assert!(
        table.probe(key).is_none(),
        "shared TT should not import local key16-only entries"
    );
    table.store(TtStore {
        key,
        depth: 5,
        score: 123,
        bound: Bound::Exact,
        mv: best,
        ply: 0,
        static_eval: 42,
        is_pv: false,
    });
    let shared_entry = table
        .probe(key)
        .expect("entry must be stored in shared table");
    assert_eq!(shared_entry.score, 123);
    assert_eq!(shared_entry.bound(), Some(Bound::Exact));
    assert_eq!(shared_entry.best_move(), Some(best));
    // Verification is the top-16 tag, NOT the full 64-bit key. The shared
    // table used to store `key ^ data` alongside `data` — 16 B per slot — to
    // verify all 64 bits. That bought a guarantee the single-threaded table
    // has never had (it compares a plain `key16`) at the price of 60% more
    // memory per entry, which halved the positions a multi-threaded search
    // could remember. The slot is now 10 B and verifies 16 bits, matching the
    // local backend exactly; a differing tag must still be rejected.
    assert!(
        table.probe(key ^ 0x8000_0000_0000_0000).is_none(),
        "shared TT must reject a key whose verification tag differs"
    );

    table.store(TtStore {
        key,
        depth: 4,
        score: 90,
        bound: Bound::Upper,
        mv: Move::NULL,
        ply: 0,
        static_eval: 11,
        is_pv: false,
    });
    let replaced = table.probe(key).expect("entry must remain present");
    assert_eq!(replaced.bound(), Some(Bound::Upper));
    assert_eq!(replaced.best_move(), Some(best));

    let mate_in_three = MATE_SCORE - 3;
    let tt_score = score_to_tt(mate_in_three, 7);
    assert_eq!(score_from_tt(tt_score, 7, 0), mate_in_three);
    let mated_in_three = -MATE_SCORE + 3;
    let tt_score = score_to_tt(mated_in_three, 7);
    assert_eq!(score_from_tt(tt_score, 7, 0), mated_in_three);
    assert!(
        score_from_tt(score_to_tt(MATE_SCORE - 12, 0), 0, 95) < MATE_SCORE - 128,
        "mate scores past the 50-move horizon must not be reused as forced mates"
    );

    table.clear();
    assert!(table.probe(key).is_none());
    assert_eq!(table.hashfull(), 0);
}

#[test]
fn transposition_table_hashfull_counts_only_current_generation_entries() {
    let best = Move::from_uci("e2e4").expect("valid UCI move");
    let key = 0xCAFE_0000_0000_0001;
    let second_key = 0xCAFE_0000_0000_0002;
    let fresh_key = 0xBABE_0000_0000_0003;
    let second_fresh_key = 0xBABE_0000_0000_0004;

    let mut table = TranspositionTable::new(1);
    table.store(TtStore {
        key,
        depth: 6,
        score: 12,
        bound: Bound::Exact,
        mv: best,
        ply: 0,
        static_eval: 34,
        is_pv: false,
    });
    table.store(TtStore {
        key: second_key,
        depth: 5,
        score: 20,
        bound: Bound::Upper,
        mv: best,
        ply: 0,
        static_eval: 10,
        is_pv: false,
    });
    table.prefetch(key);
    assert!(table.hashfull() > 0);

    table.new_search();
    assert_eq!(
        table.hashfull(),
        0,
        "hashfull should ignore entries from older TT generations"
    );
    assert!(
        table.probe(key).is_some(),
        "stale hashfull accounting must not make entries unprobeable"
    );

    table.store(TtStore {
        key: fresh_key,
        depth: 4,
        score: -8,
        bound: Bound::Lower,
        mv: best,
        ply: 0,
        static_eval: -10,
        is_pv: false,
    });
    table.store(TtStore {
        key: second_fresh_key,
        depth: 3,
        score: -12,
        bound: Bound::Exact,
        mv: best,
        ply: 0,
        static_eval: -3,
        is_pv: false,
    });
    assert!(table.hashfull() > 0);

    let mut shared = TranspositionTable::new(1);
    shared.make_shared(1);
    shared.store(TtStore {
        key,
        depth: 5,
        score: 99,
        bound: Bound::Exact,
        mv: best,
        ply: 0,
        static_eval: 11,
        is_pv: false,
    });
    shared.store(TtStore {
        key: second_key,
        depth: 4,
        score: 88,
        bound: Bound::Lower,
        mv: best,
        ply: 0,
        static_eval: 22,
        is_pv: false,
    });
    // `hashfull` reports per-mille of SLOTS over the sampled clusters, so two
    // entries in a 1 MiB shared table legitimately round to zero — it only
    // read non-zero before because the slot count per cluster was small enough
    // to flatter the integer division. Fill enough of the sampled region (the
    // cluster index is the key's low bits) to give the generation logic under
    // test a real reading to move off.
    for filler in 3..64u64 {
        shared.store(TtStore {
            key: 0xCAFE_0000_0000_0000 | filler,
            depth: 3,
            score: 5,
            bound: Bound::Exact,
            mv: best,
            ply: 0,
            static_eval: 5,
            is_pv: false,
        });
    }
    shared.prefetch(key);
    assert!(shared.hashfull() > 0);
    shared.new_search();
    assert_eq!(shared.hashfull(), 0);
    assert!(shared.probe(key).is_some());
}
