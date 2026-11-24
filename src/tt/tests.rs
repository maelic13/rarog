use super::{
    LocalCluster, SharedCluster, TtProbe, cluster_count, cluster_index, key16_of, score_from_tt,
    score_to_tt, size_of,
};
use crate::score::{MATE_BOUND, MATE_SCORE, MAX_PLY, TB_VALUE, TB_WIN_SCORE};

/// An entry whose every field derives from its key, so a load that pairs
/// one store's tag with another's payload shows as a mismatch.
fn keyed_entry(key16: u16) -> super::TtEntry {
    super::TtEntry {
        key16,
        score: i16::from_ne_bytes(key16.to_ne_bytes()),
        static_eval: i16::from_ne_bytes((key16 ^ 0xA5A5).to_ne_bytes()),
        mv: key16,
        depth: i8::try_from(key16 % 120).expect("below 120"),
        flag_age: 1,
    }
}

fn same_payload(a: super::TtEntry, b: super::TtEntry) -> bool {
    (a.score, a.static_eval, a.mv, a.depth, a.flag_age)
        == (b.score, b.static_eval, b.mv, b.depth, b.flag_age)
}

/// A slot holding one store's tag and another's payload, as a read racing
/// two writers can see it, reconstructs a key no probe holds when the
/// payloads' folds differ, and is rejected. Equal folds are the tag's
/// blind spot, the 2^-16 the tag's strength allows: that pair loads.
#[test]
fn a_torn_shared_slot_is_rejected_unless_the_folds_agree() {
    use std::sync::atomic::Ordering::Relaxed;
    let torn = |a: super::TtEntry, b: super::TtEntry| {
        let cluster = SharedCluster::default();
        cluster.store(0, a.key16, a);
        cluster.store(1, b.key16, b);
        cluster.data[0].store(cluster.data[1].load(Relaxed), Relaxed);
        cluster
    };
    let a = keyed_entry(0x1234);
    let b = keyed_entry(0x4321);
    assert_ne!(
        super::fold16(super::pack_entry(a)),
        super::fold16(super::pack_entry(b))
    );
    let cluster = torn(a, b);
    assert!(cluster.load(0, a.key16).is_none(), "torn pair accepted");
    assert!(
        cluster.load(0, b.key16).is_none(),
        "torn pair accepted as the other"
    );
    assert!(cluster.load(1, b.key16).is_some(), "the untorn slot loads");

    // Flipping the same bits in two 16-bit lanes keeps the fold.
    let mut same_fold = a;
    same_fold.score ^= 0x0F0F;
    same_fold.static_eval ^= 0x0F0F;
    same_fold.key16 = 0x4321;
    assert_eq!(
        super::fold16(super::pack_entry(a)),
        super::fold16(super::pack_entry(same_fold))
    );
    let cluster = torn(a, same_fold);
    let loaded = cluster.load(0, a.key16).expect("equal folds hide the tear");
    assert!(
        same_payload(loaded, same_fold),
        "the payload is the other store's"
    );
}

/// Four threads store into and probe one slot at once, as hard as two
/// writers can race. Every probe that verifies must return the payload of
/// the key it asked for: a torn pair reaches it only through the tag's
/// 2^-16 blind spot. A tag without the payload's fold returns about 30
/// wrong payloads a run in a debug build, whose slower loop tears far
/// more often than release's, so the debug suite is this test's teeth.
#[test]
fn four_threads_on_one_shared_cluster_never_return_a_torn_entry() {
    const THREADS: u16 = 4;
    const ROUNDS: u32 = 1_000_000;
    let cluster = SharedCluster::default();
    let start = std::sync::Barrier::new(usize::from(THREADS));
    let (verified, wrong) = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..THREADS)
            .map(|thread| {
                let (cluster, start) = (&cluster, &start);
                scope.spawn(move || {
                    let (mut verified, mut wrong) = (0_u32, 0_u32);
                    start.wait();
                    for round in 0..ROUNDS {
                        let key16 =
                            (thread << 14) | u16::try_from(round & 0x3FFF).expect("fourteen bits");
                        cluster.store(0, key16, keyed_entry(key16));
                        // Widen the window in which another writer can
                        // land between this store and its probe, and keep
                        // the optimiser from handing the probe the value
                        // this thread just stored.
                        for _ in 0..16 {
                            std::hint::spin_loop();
                        }
                        if let Some(entry) = std::hint::black_box(cluster).load(0, key16) {
                            verified += 1;
                            wrong += u32::from(!same_payload(entry, keyed_entry(key16)));
                        }
                    }
                    (verified, wrong)
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("worker thread"))
            .fold((0, 0), |(v, w), (dv, dw)| (v + dv, w + dw))
    });
    assert!(verified > 0, "no probe verified");
    assert!(
        wrong <= 3,
        "{wrong} of {verified} verified probes returned another store's payload"
    );
}

/// The table places the mate band by the search's ply cap, so its copy
/// must be the search's.
#[test]
fn decisive_scores_round_trip_through_the_table_at_any_ply() {
    for ply in [0, 1, MAX_PLY - 1] {
        let p = i32::try_from(ply).expect("small ply");
        for score in [
            TB_VALUE - p,
            -(TB_VALUE - p),
            MATE_SCORE - p - 3,
            -(MATE_SCORE - p - 3),
            TB_WIN_SCORE - 1,
            -(TB_WIN_SCORE - 1),
            0,
            731,
        ] {
            assert_eq!(
                score_from_tt(score_to_tt(score, ply), ply, 0),
                score,
                "{score} at {ply}"
            );
        }
    }
    // Every tablebase value at every ply is decisive and below the mates.
    for ply in 0..MAX_PLY {
        let win = TB_VALUE - crate::infra::to_i32(ply);
        assert!((TB_WIN_SCORE..MATE_BOUND).contains(&win), "{win}");
    }
}

#[test]
fn a_mate_beyond_the_rule50_horizon_reads_back_as_not_decisive() {
    // A mate in 10 plies stored at the node: on the horizon at clock 90 and
    // kept; past it at 91 and below the decisive band.
    let (stored, ply) = (MATE_SCORE - 10, 3usize);
    let p = i32::try_from(ply).expect("small ply");
    assert_eq!(score_from_tt(stored, ply, 90), stored - p);
    assert_eq!(score_from_tt(stored, ply, 91), TB_WIN_SCORE - 1);
    assert_eq!(score_from_tt(-stored, ply, 90), -stored + p);
    assert_eq!(score_from_tt(-stored, ply, 91), -TB_WIN_SCORE + 1);
    // A tablebase-band value passes through unchanged at any clock.
    assert_eq!(score_from_tt(TB_VALUE - 10, ply, 99), TB_VALUE - 10);
}

#[test]
fn a_miss_grants_no_capability() {
    let miss = TtProbe::MISS;
    assert_eq!(miss.depth, -1);
    assert!(miss.bound.is_none());
    assert!(miss.mv.is_none());
}

#[test]
fn pv_line_is_the_union_of_node_and_stored_bits() {
    let stored = TtProbe {
        stored_pv: true,
        ..TtProbe::MISS
    };
    assert!(stored.pv_line(false));
    assert!(TtProbe::MISS.pv_line(true));
    assert!(!TtProbe::MISS.pv_line(false));
}

/// Every key lands in the table at any count, the extremes included, and
/// the keys cover it evenly at a count that is no power of two.
#[test]
fn cluster_index_covers_any_table_evenly() {
    for clusters in [1usize, 3, 1000, 32_000_000, 1 << 40] {
        assert_eq!(cluster_index(0, clusters), 0);
        assert_eq!(cluster_index(u64::MAX, clusters), clusters - 1);
    }
    const CLUSTERS: usize = 1000;
    let mut hits = [0u32; CLUSTERS];
    let mut key = 0x9E37_79B9_7F4A_7C15_u64;
    for _ in 0..1_000_000 {
        key = key.wrapping_mul(0x5851_F42D_4C95_7F2D).wrapping_add(1);
        hits[cluster_index(key, CLUSTERS)] += 1;
    }
    let (least, most) = (hits.iter().min(), hits.iter().max());
    assert!(
        least >= Some(&800) && most <= Some(&1200),
        "{least:?} {most:?}"
    );
}

/// The tag and the cluster read opposite ends of the key, so keys that
/// share a cluster still differ in the tag.
#[test]
fn the_tag_is_independent_of_the_cluster() {
    let key = 0x0123_4567_89AB_CDEF_u64;
    assert_eq!(key16_of(key), 0xCDEF);
    assert_eq!(key16_of(key ^ 0xFFFF_0000_0000_0000), 0xCDEF);
    assert_eq!(
        cluster_index(key, 1 << 20),
        cluster_index(key ^ 0xFFFF, 1 << 20)
    );
}

/// `Hash` is the size asked for, not the power of two below it: 1000 MiB
/// is not floored to 512.
#[test]
fn a_table_takes_the_whole_hash_at_any_size() {
    for mb in [1000usize, 2000, 3000, 32_000] {
        let bytes = mb << 20;
        assert_eq!(
            cluster_count::<LocalCluster>(mb) * size_of::<LocalCluster>(),
            bytes
        );
        assert_eq!(
            cluster_count::<SharedCluster>(mb) * size_of::<SharedCluster>(),
            bytes
        );
    }
}

mod core {
    use super::super::{
        AGE_MASK, AGE_QUALITY_DIVISOR, AGE_STRIDE, Bound, EVAL_ONLY_DEPTH, LocalTable, PV_BIT,
        QUIESCENCE_DEPTH, TranspositionTable, TtEntry, TtProbe, TtStorage, TtStore, entry_quality,
    };
    use crate::board::Move;
    use crate::score::VALUE_NONE;

    const KEY: u64 = 0xA5A5_1234_5678_9ABC;

    fn stored(depth: i32, score: i32, bound: Bound, mv: Move, is_pv: bool) -> TtStore {
        TtStore {
            key: KEY,
            depth,
            score,
            bound,
            mv,
            ply: 0,
            static_eval: 17,
            is_pv,
        }
    }

    fn both_backends() -> [TranspositionTable; 2] {
        let local = TranspositionTable::new(1);
        let mut shared = TranspositionTable::new(1);
        shared.make_shared(1);
        [local, shared]
    }

    #[test]
    fn five_bit_age_costs_four_plies_a_generation_and_wraps_after_32() {
        let entry = TtEntry {
            depth: 40,
            flag_age: Bound::Exact as u8 | PV_BIT,
            ..TtEntry::default()
        };
        for generation in 0_u8..32 {
            let age = generation.wrapping_mul(AGE_STRIDE) & AGE_MASK;
            assert_eq!(
                entry_quality(entry, age),
                40 - i32::from(generation) * 4,
                "generation {generation}"
            );
        }
        assert_eq!(AGE_QUALITY_DIVISOR, 2);
        let mut tt = TranspositionTable::new(1);
        for _ in 0..32 {
            tt.new_search();
        }
        let TtStorage::Local(LocalTable { age, .. }) = &tt.storage else {
            panic!("new table must use local storage");
        };
        assert_eq!(*age, 0, "32 searches wrap the five-bit age");
    }

    #[test]
    fn an_eval_only_entry_carries_the_eval_and_nothing_else() {
        for mut tt in both_backends() {
            tt.store_eval(KEY, -123, true);
            let entry = tt.probe(KEY).expect("eval-only entry is found");
            let probe = TtProbe::from_entry(Some(entry), 0, 0);
            assert_eq!(probe.raw_static_eval, -123);
            assert_eq!(probe.bound, None);
            assert_eq!(probe.score, VALUE_NONE);
            assert_eq!(probe.depth, EVAL_ONLY_DEPTH);
            assert_eq!(probe.mv, None);
            assert!(probe.pv_line(false), "the PV bit is stored");
        }
        // Eval-only entries occupy their slots: a table holding nothing
        // else reports them in `hashfull`.
        for mut tt in both_backends() {
            assert_eq!(tt.hashfull(), 0);
            for i in 0..20_000_u64 {
                tt.store_eval(i.wrapping_mul(0x9E37_79B9_7F4A_7C15), -123, true);
            }
            assert!(tt.hashfull() > 0, "eval-only entries count as used");
        }
    }

    /// The keep rule reads a quiescence store as the donor's table does:
    /// it keeps a same-position entry three plies deep (five for a PV
    /// store) and replaces a shallower one.
    #[test]
    fn a_quiescence_store_keeps_an_entry_three_plies_deep_and_replaces_a_shallower_one() {
        let mv = Move::from_uci("e2e4").expect("valid move");
        for (is_pv, kept) in [(false, 3), (true, 5)] {
            for mut tt in both_backends() {
                tt.store(stored(kept, 50, Bound::Lower, mv, false));
                tt.store(stored(
                    QUIESCENCE_DEPTH,
                    10,
                    Bound::Exact,
                    Move::NULL,
                    is_pv,
                ));
                let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
                assert_eq!((probe.depth, probe.score), (kept, 50), "pv {is_pv}");
            }
            for mut tt in both_backends() {
                tt.store(stored(kept - 1, 50, Bound::Lower, mv, false));
                tt.store(stored(
                    QUIESCENCE_DEPTH,
                    10,
                    Bound::Exact,
                    Move::NULL,
                    is_pv,
                ));
                let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
                assert_eq!(
                    (probe.depth, probe.score),
                    (QUIESCENCE_DEPTH, 10),
                    "pv {is_pv}"
                );
                assert_eq!(probe.mv, Some(mv), "a store without a move keeps the move");
            }
        }
    }

    /// A quiescence result and an evaluation-only entry rank level as
    /// victims, below every searched depth, as in the donor's table.
    #[test]
    fn quiescence_and_evaluation_only_entries_rank_level_as_victims() {
        let at = |depth: i32, bound: u8| TtEntry {
            depth: i8::try_from(depth).expect("a small depth"),
            flag_age: bound,
            ..TtEntry::default()
        };
        let quiescence = entry_quality(at(QUIESCENCE_DEPTH, Bound::Exact as u8), 0);
        let eval_only = entry_quality(at(EVAL_ONLY_DEPTH, 0), 0);
        assert_eq!(quiescence, eval_only);
        assert!(entry_quality(at(1, Bound::Lower as u8), 0) > quiescence);
        assert!(at(EVAL_ONLY_DEPTH, 0).is_occupied() && !TtEntry::default().is_occupied());
    }

    #[test]
    fn a_searched_result_replaces_an_eval_only_entry() {
        let mv = Move::from_uci("e2e4").expect("valid move");
        for mut tt in both_backends() {
            tt.store_eval(KEY, 40, false);
            tt.store(stored(0, 25, Bound::Lower, mv, false));
            let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
            assert_eq!(probe.bound, Some(Bound::Lower));
            assert_eq!(probe.depth, 0);
            assert_eq!(probe.score, 25);
            assert_eq!(probe.mv, Some(mv));
        }
    }

    #[test]
    fn a_much_deeper_entry_keeps_its_result_but_takes_a_new_move() {
        let old = Move::from_uci("e2e4").expect("valid move");
        let new = Move::from_uci("d2d4").expect("valid move");
        for mut tt in both_backends() {
            tt.store(stored(12, 90, Bound::Exact, old, false));
            // 8 + 4 <= 12: refused, move refreshed.
            tt.store(stored(8, -40, Bound::Upper, new, false));
            let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
            assert_eq!(
                (probe.depth, probe.score, probe.bound),
                (12, 90, Some(Bound::Exact))
            );
            assert_eq!(probe.mv, Some(new));
            // On a PV line the margin is six: 7 + 4 + 2 > 12 replaces.
            tt.store(stored(7, -30, Bound::Upper, Move::NULL, true));
            let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
            assert_eq!((probe.depth, probe.score), (7, -30));
            assert_eq!(probe.mv, Some(new), "a store without a move keeps the move");
            // A new generation lifts the refusal.
            tt.store(stored(20, 5, Bound::Exact, old, false));
            tt.new_search();
            tt.store(stored(3, 6, Bound::Lower, Move::NULL, false));
            let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
            assert_eq!((probe.depth, probe.score), (3, 6));
        }
    }
}
