use super::*;

#[test]
fn shared_stop_request_sets_search_stop() {
    let state = SharedContext::new(0, 0, 1);

    state.request_stop();

    assert_eq!(state.stop_state.load(Ordering::Relaxed), STOP_SEARCH);
}

/// The threshold is a strict majority at EVERY pool size, including the
/// two-thread case where that means unanimity. Exempting N=2 measured
/// −15.85 ± 6.12 Elo — see [`SharedContext::votes_needed`].
#[test]
fn stop_vote_threshold_is_a_strict_majority_at_every_pool_size() {
    assert_eq!(
        SharedContext::votes_needed(2),
        2,
        "N=2 needs BOTH votes: exempting it measured -15.85 Elo"
    );
    for n in [1usize, 2, 3, 4, 5, 6, 8, 16] {
        assert_eq!(
            SharedContext::votes_needed(n),
            n / 2 + 1,
            "N={n} must keep the strict-majority threshold"
        );
        assert!(
            SharedContext::votes_needed(n) * 2 > n,
            "N={n} threshold must still be a strict majority"
        );
    }
}

/// One thread alone can never end a multi-thread search — the latch in
/// `search_root` plus this threshold are what stop a single noisy
/// estimate from deciding for the pool.
#[test]
fn stop_vote_fires_at_the_threshold_not_before() {
    let two = SharedContext::new(0, 0, 2);
    assert!(!two.vote_to_stop(), "1 of 2 must not stop");
    assert!(two.vote_to_stop(), "2 of 2 is unanimity and must stop");

    let four = SharedContext::new(0, 0, 4);
    assert!(!four.vote_to_stop(), "1 of 4 must not stop");
    assert!(!four.vote_to_stop(), "2 of 4 must not stop");
    assert!(four.vote_to_stop(), "3 of 4 is the majority and must stop");
}

#[test]
fn shared_stop_request_does_not_overwrite_quit() {
    let state = SharedContext::new(0, 0, 1);

    state.request_quit();
    state.request_stop();

    assert_eq!(state.stop_state.load(Ordering::Relaxed), STOP_QUIT);
}

#[test]
fn root_score_packing_roundtrips_and_ranks_bounds() {
    let state = SharedContext::new(0, 2, 1);

    // Negative scores and bound tags survive the round trip.
    state.publish_root_score(0, 12, -481, RootBound::Upper);
    assert_eq!(state.root_score(0), Some((12, -481, RootBound::Upper)));

    // Equal depth: a stronger bound replaces a weaker one...
    state.publish_root_score(0, 12, 37, RootBound::Exact);
    assert_eq!(state.root_score(0), Some((12, 37, RootBound::Exact)));
    // ...and a weaker bound at equal depth is refused.
    state.publish_root_score(0, 12, 999, RootBound::Upper);
    assert_eq!(state.root_score(0), Some((12, 37, RootBound::Exact)));

    // Deeper always wins, whatever the bound.
    state.publish_root_score(0, 13, -5, RootBound::Upper);
    assert_eq!(state.root_score(0), Some((13, -5, RootBound::Upper)));

    // pool_best_exact sees only Exact entries.
    assert_eq!(state.pool_best_exact(), None);
    state.publish_root_score(1, 9, 120, RootBound::Exact);
    assert_eq!(state.pool_best_exact(), Some((9, 120)));
}

/// The extremes the `expect`s in `unpack_root_score` rely on.
/// `i32::MIN`/`i32::MAX` sit exactly at the ends of the biased score field
/// and the depth mask is 24 bits, so if either invariant is ever broken by
/// a repacking these round-trips fail here rather than silently degrading a
/// root score in a live search.
#[test]
fn root_score_packing_survives_the_field_extremes() {
    let state = SharedContext::new(0, 1, 1);

    for score in [i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX - 1, i32::MAX] {
        state.publish_root_score(0, 1, score, RootBound::Exact);
        assert_eq!(
            state.root_score(0),
            Some((1, score, RootBound::Exact)),
            "score extreme {score} did not round-trip"
        );
    }

    // Deepest value the 24-bit depth field can hold.
    let deepest = 0x00FF_FFFF;
    state.publish_root_score(0, deepest, 7, RootBound::Exact);
    assert_eq!(state.root_score(0), Some((deepest, 7, RootBound::Exact)));
}
