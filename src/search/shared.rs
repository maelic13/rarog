//! State every search thread shares: stop requests, pooled node and
//! tablebase counters, root-move scores and soft-stop votes.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, AtomicU64, AtomicUsize, Ordering};

pub(super) const STOP_NONE: u8 = 0;
pub(super) const STOP_SEARCH: u8 = 1;
pub(super) const STOP_QUIT: u8 = 2;

pub(super) struct SharedContext {
    pub(super) stop_state: AtomicU8,
    pub ponderhit: AtomicBool,
    pub nodes: AtomicU64,
    pub tb_hits: AtomicU64,
    /// Pooled per-root-move knowledge, indexed by the move's position
    /// in the root list. Packed `(bound << 56) | (depth << 32) | score_bits`
    /// so a single relaxed load/CAS carries a consistent triple;
    /// `NO_ROOT_SCORE` = unset.
    ///
    /// Threads publish the (depth, score, bound) they have proven for each
    /// searched root move, and every thread orders its root list from the
    /// POOL's view instead of only its own. The shared TT already propagates
    /// most of this implicitly; the explicit channel exists because TT
    /// entries for root moves get overwritten under pressure while these do
    /// not.
    root_scores: Vec<AtomicI64>,
    /// Symmetric soft-stop votes. Each thread whose own soft target
    /// expires casts one vote and keeps searching; the pool stops once a
    /// strict majority agrees, so the decision uses N clamped opinions
    /// rather than the main thread's single noisy estimate. Threshold and
    /// its measured justification: [`SharedContext::votes_needed`].
    stop_votes: AtomicUsize,
    thread_count: usize,
}

/// Sentinel for an unpublished root score.
const NO_ROOT_SCORE: i64 = i64::MIN;
/// Keeps the packed score non-negative in its 32-bit field.
const SCORE_BIAS: i64 = 1 << 31;
const LOW32: i64 = 0xFFFF_FFFF;
/// Depth field narrowed to 24 bits (MAX_PLY fits with room to spare) to free
/// the top byte for the bound tag.
const DEPTH_MASK: i64 = 0xFF_FFFF;
const DEPTH_SHIFT: u32 = 32;
const BOUND_SHIFT: u32 = 56;

/// What a published root score means. Declaration order is the
/// replacement rank at equal depth — an Exact score beats a Lower bound beats
/// an Upper bound (a fail-low only proves `true <= score`, the weakest fact).
/// Every searched root move is published with its real bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum RootBound {
    Upper = 0,
    Lower = 1,
    Exact = 2,
}

impl RootBound {
    fn from_bits(bits: i64) -> RootBound {
        match bits {
            1 => RootBound::Lower,
            2 => RootBound::Exact,
            _ => RootBound::Upper,
        }
    }
}

fn pack_root_score(depth: i32, score: i32, bound: RootBound) -> i64 {
    ((bound as i64) << BOUND_SHIFT)
        | ((i64::from(depth) & DEPTH_MASK) << DEPTH_SHIFT)
        | ((i64::from(score) + SCORE_BIAS) & LOW32)
}

fn unpack_root_score(packed: i64) -> (i32, i32, RootBound) {
    // `expect`, not `unwrap_or(0)`. Both conversions are infallible
    // by construction and provably so — `pack_root_score` masks depth with
    // `DEPTH_MASK` (24 bits, so 0..=16_777_215) and the score field with
    // `LOW32`, which after subtracting `SCORE_BIAS` (2^31) spans exactly
    // −2^31..=2^31−1, i.e. the whole of `i32`. The previous fallback could
    // therefore never fire, but if the packing were ever changed it would
    // substitute a silent `0` — a WRONG root score fed straight into root
    // ordering and aspiration seeding. A loud failure is strictly better than
    // a plausible wrong number in a search heuristic.
    let depth = i32::try_from((packed >> DEPTH_SHIFT) & DEPTH_MASK)
        .expect("pack_root_score masks depth to 24 bits");
    let score = i32::try_from((packed & LOW32) - SCORE_BIAS)
        .expect("pack_root_score's bias maps the score field onto exactly i32");
    let bound = RootBound::from_bits(packed >> BOUND_SHIFT);
    (depth, score, bound)
}

impl SharedContext {
    pub(crate) fn new(initial_tb_hits: u64, root_move_count: usize, thread_count: usize) -> Self {
        Self {
            stop_state: AtomicU8::new(STOP_NONE),
            ponderhit: AtomicBool::new(false),
            nodes: AtomicU64::new(0),
            tb_hits: AtomicU64::new(initial_tb_hits),
            root_scores: (0..root_move_count)
                .map(|_| AtomicI64::new(NO_ROOT_SCORE))
                .collect(),
            stop_votes: AtomicUsize::new(0),
            thread_count,
        }
    }

    /// Publish `(depth, score, bound)` for root move `index` if it improves on
    /// what the pool already knows.
    ///
    /// Packed into one atomic so a reader always sees a consistent triple:
    /// bound tag in the top byte, depth below it, score offset-encoded into
    /// the low 32 bits so it stays non-negative. Replacement rule: deeper
    /// always wins; at equal depth a stronger bound (Exact > Lower > Upper)
    /// wins, and an equal bound refreshes with the newer score.
    pub(super) fn publish_root_score(
        &self,
        index: usize,
        depth: i32,
        score: i32,
        bound: RootBound,
    ) {
        let Some(slot) = self.root_scores.get(index) else {
            return;
        };
        let packed = pack_root_score(depth, score, bound);
        let _ = slot.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            if current == NO_ROOT_SCORE {
                return Some(packed);
            }
            let (cur_depth, _, cur_bound) = unpack_root_score(current);
            if depth > cur_depth || (depth == cur_depth && bound >= cur_bound) {
                Some(packed)
            } else {
                None
            }
        });
    }

    /// Pooled `(depth, score, bound)` for root move `index`, if any thread has
    /// one.
    pub(super) fn root_score(&self, index: usize) -> Option<(i32, i32, RootBound)> {
        let packed = self.root_scores.get(index)?.load(Ordering::Relaxed);
        if packed == NO_ROOT_SCORE {
            return None;
        }
        Some(unpack_root_score(packed))
    }

    /// The pool's best Exact root score at its deepest published depth:
    /// the pool-wide PV estimate a thread can center its aspiration
    /// window on when the pool has searched deeper than the thread itself.
    /// Root lists are tiny, so the linear scan (once per iteration per thread)
    /// is free.
    pub(super) fn pool_best_exact(&self) -> Option<(i32, i32)> {
        let mut best: Option<(i32, i32)> = None;
        for slot in &self.root_scores {
            let packed = slot.load(Ordering::Relaxed);
            if packed == NO_ROOT_SCORE {
                continue;
            }
            let (depth, score, bound) = unpack_root_score(packed);
            if bound != RootBound::Exact {
                continue;
            }
            if best.is_none_or(|(bd, bs)| depth > bd || (depth == bd && score > bs)) {
                best = Some((depth, score));
            }
        }
        best
    }

    /// Votes needed to end the search: a strict majority, `floor(N/2)+1`.
    ///
    /// The vote is an order statistic over the threads' independent soft
    /// targets, so this stops the pool at the **median** expiry: N clamped
    /// opinions rather than one noisy estimate.
    ///
    /// **`N = 2` means unanimity, and that is measured.** A strict majority of
    /// two is two, so a 2-thread pool waits for the *later* thread. Stopping on
    /// the first vote instead measured **−15.85 ± 6.12 Elo** (5,222 games at
    /// Threads=2).
    ///
    /// Why: stopping on the first vote takes `min` of the
    /// two expiry times, and the minimum of two draws is a **downward-biased**
    /// estimator — it is the opposite wrong tail, not a correction. Two
    /// opinions have no median, and the honest summary of two numbers is
    /// their mean, which a vote cannot express; of the two representable
    /// choices, `max` is the one that wins, because the soft target is a
    /// heuristic *estimate* rather than a budget and `maximum_ms` is the
    /// actual constraint (0 forfeits across 7,880 games in the 2T null). More
    /// search time is worth more than staying near the soft estimate.
    ///
    /// Do not "simplify" this back to a special case without re-gating it.
    const fn votes_needed(thread_count: usize) -> usize {
        thread_count / 2 + 1
    }

    /// Register one thread's "I would stop now" vote; returns true once
    /// enough of the pool agrees, at which point the caller stops the whole
    /// search.
    pub(super) fn vote_to_stop(&self) -> bool {
        let votes = self.stop_votes.fetch_add(1, Ordering::Relaxed) + 1;
        votes >= Self::votes_needed(self.thread_count)
    }

    pub(crate) fn request_stop(&self) {
        let _ = self
            .stop_state
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |state| {
                (state != STOP_QUIT).then_some(STOP_SEARCH)
            });
    }

    pub(crate) fn request_quit(&self) {
        self.stop_state.store(STOP_QUIT, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
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
}
