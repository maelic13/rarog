//! Per-node search code never allocates (AGENTS, *Research and implementation*).
//!
//! A counting global allocator watches whole searches of the same positions
//! at two depths, at Threads 1 and 4. Search setup and each iteration may
//! allocate a little: the root node's move lists, the reported line. Nothing
//! per node may, so going deeper may add only a small fixed number of
//! allocations per extra iteration while the node count grows by more than an
//! order of magnitude. An allocation at every node, or at one node in a
//! hundred, breaks that bound.
//!
//! This file holds a single test on purpose: the counter is process-wide, so a
//! second test running in parallel would count into the same window.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

use rarog::bench::BENCH_FENS;
use rarog::board::Board;
use rarog::search::{SearchEvent, Searcher};
use rarog::search_options::SearchOptions;

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);

struct CountingAllocator;

// SAFETY: every method forwards to `System` with the caller's own arguments,
// so the `GlobalAlloc` contract is `System`'s; counting touches no memory the
// allocator hands out.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the caller upholds `alloc`'s contract for `layout`.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the caller upholds `alloc_zeroed`'s contract for `layout`.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: `ptr` came from this allocator, which is `System`, with
        // `layout`; the caller upholds the rest of `realloc`'s contract.
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from this allocator, which is `System`, with
        // `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Plies between the two searches; each extra ply is one more iteration.
const SHALLOW_DEPTH: u32 = 5;
/// The contract is the release binary's, so release searches the full depth.
/// Debug stops two plies short, in about a third of the time: that tree
/// still grows more than twentyfold and fires every per-node diag counter the
/// full one does, except a depth band of the null-move cut and an endgame
/// census entry with no code path of its own.
const DEEP_DEPTH: u32 = if cfg!(debug_assertions) { 11 } else { 13 };

/// Allocations one iteration may add, per searching thread. The root node
/// builds its move lists on every visit and each iteration reports a line;
/// measured at most 25 per iteration per thread over these positions (the
/// endgame, which re-searches its aspiration window most). The bound leaves
/// room for that and still catches anything that allocates at more than about
/// one node in a hundred.
const ALLOCATIONS_PER_ITERATION_PER_THREAD: u64 = 64;

/// The deep search must be much larger than the shallow one, or a per-node
/// allocation could hide inside the budget.
const MIN_NODE_GROWTH: u64 = 10;

/// Bench positions of different kinds. Each must keep growing between the two
/// depths on every search this crate builds; a position the search solves
/// early (a forced mate) stops growing and cannot expose a per-node allocation.
const POSITIONS: [usize; 4] = [0, 12, 25, 33];

/// Allocations and nodes of one search from a cleared state, so the hash table
/// of an earlier search cannot shrink the tree. The board is built and the
/// state cleared before the window opens.
fn measured_search(searcher: &mut Searcher, options: &SearchOptions, depth: u32) -> (u64, u64) {
    let mut options = options.clone();
    options.limits.depth = Some(depth);
    let root = options.board.clone();
    searcher.new_game();
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let result = searcher.search(root, &options, false, || SearchEvent::None);
    let allocations = ALLOCATIONS.load(Ordering::Relaxed) - before;
    (allocations, result.nodes)
}

#[test]
fn allocations_grow_per_iteration_never_per_node() {
    for index in POSITIONS {
        // The single-thread shallow tree is the honest size of a depth-5
        // search of this position. The threaded searches are sized against it
        // below, because a threaded node count is the scheduler's as much as
        // the search's: on an oversubscribed host (CI's three-core macOS
        // runner at Threads 4) the main thread is starved while the helpers
        // run free, and a depth-5 search was once counted at 219,395 nodes.
        let mut single_thread_shallow_nodes = 0;
        for threads in [1, 4] {
            let fen = BENCH_FENS[index];
            let mut options = SearchOptions {
                board: Board::from_fen(fen).expect("bench FEN parses"),
                ..SearchOptions::default()
            };
            options.engine.threads = threads;

            let mut searcher = Searcher::default();
            searcher.configure(&options.engine);
            // The first search pays one-time set-up (lazy tables, the helper
            // pool's first jobs); it is not what this test is about.
            measured_search(&mut searcher, &options, SHALLOW_DEPTH);

            let (shallow_allocations, shallow_nodes) =
                measured_search(&mut searcher, &options, SHALLOW_DEPTH);
            let (deep_allocations, deep_nodes) =
                measured_search(&mut searcher, &options, DEEP_DEPTH);
            let context = format!(
                "Threads {threads}, bench position {index}: depth {SHALLOW_DEPTH} made \
                 {shallow_allocations} allocations in {shallow_nodes} nodes, depth \
                 {DEEP_DEPTH} made {deep_allocations} in {deep_nodes}"
            );
            if threads == 1 {
                single_thread_shallow_nodes = shallow_nodes;
            }
            assert!(
                deep_nodes >= MIN_NODE_GROWTH * single_thread_shallow_nodes,
                "the deep search is too small to expose a per-node allocation; {context}"
            );
            let extra_iterations = u64::from(DEEP_DEPTH - SHALLOW_DEPTH);
            let budget = ALLOCATIONS_PER_ITERATION_PER_THREAD
                * extra_iterations
                * u64::try_from(threads).expect("a thread count fits in u64");
            assert!(
                deep_allocations.saturating_sub(shallow_allocations) <= budget,
                "allocations grew faster than {ALLOCATIONS_PER_ITERATION_PER_THREAD} per \
                 iteration per thread (budget {budget}): per-node code allocates; {context}"
            );
        }
    }
}
