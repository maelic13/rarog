use rarog::board::Move;
use rarog::evidence::OutcomeKind;
use rarog::tt::{Bound, TranspositionTable, TtStore};

fn assert_provenance_round_trip(shared: bool) {
    let mut tt = TranspositionTable::new(1);
    if shared {
        tt.make_shared(1);
    }

    let full_key = 0x1234_0000_0000_0011;
    let speculative_key = 0x5678_0000_0000_0022;
    let mv = Move::from_uci("e2e4").expect("valid move");

    tt.store(TtStore {
        key: full_key,
        depth: 5,
        score: 40,
        bound: Bound::Exact,
        mv,
        ply: 0,
        static_eval: 12,
        is_pv: true,
        kind: OutcomeKind::Full,
    });
    tt.store(TtStore {
        key: speculative_key,
        depth: 5,
        score: 180,
        bound: Bound::Lower,
        mv,
        ply: 0,
        static_eval: 20,
        is_pv: false,
        kind: OutcomeKind::ProbCut,
    });

    let full = tt.probe(full_key).expect("full entry must round-trip");
    let speculative = tt
        .probe(speculative_key)
        .expect("speculative entry must round-trip");
    assert!(!full.is_speculative());
    assert!(full.is_pv_node());
    assert!(speculative.is_speculative());
    assert!(!speculative.is_pv_node());
    assert_eq!(speculative.bound(), Some(Bound::Lower));
    assert_eq!(speculative.score, 180);
}

#[test]
fn local_table_round_trips_speculative_provenance() {
    assert_provenance_round_trip(false);
}

#[test]
fn shared_table_round_trips_speculative_provenance() {
    assert_provenance_round_trip(true);
}
