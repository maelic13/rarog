//! Allocation guard for the isolated board-v2 primitives.
//!
//! Inputs are built before the counter is armed.  If a hot board primitive
//! regresses to heap allocation, this test fails instead of letting a timing
//! benchmark mistake allocator throughput for board throughput.

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rarog::board::Board;

struct CountingAllocator;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

// SAFETY: this implementation delegates allocation semantics to `System` and
// only performs lock-free accounting outside the system allocator call.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ACTIVE.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: `layout` is supplied by Rust's allocator ABI and is passed
        // unchanged to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` and `layout` were supplied by Rust's allocator ABI and
        // are passed unchanged to the system allocator.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if ACTIVE.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: `ptr`, `layout`, and `size` are supplied by Rust's allocator
        // ABI and are passed unchanged to the system allocator.
        unsafe { System.realloc(ptr, layout, size) }
    }
}

fn allocations_during(work: impl FnOnce()) -> usize {
    ALLOCATIONS.store(0, Ordering::SeqCst);
    ACTIVE.store(true, Ordering::SeqCst);
    work();
    ACTIVE.store(false, Ordering::SeqCst);
    ALLOCATIONS.load(Ordering::SeqCst)
}

fn corpus() -> Vec<Board> {
    include_str!("data/board-v2.tsv")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<_> = line.split('|').collect();
            Board::from_fen(fields[2]).expect("board-v2 profile must contain valid FENs")
        })
        .collect()
}

#[test]
fn board_v2_isolated_primitives_do_not_allocate_after_warmup() {
    let boards = corpus();
    let mut capture_boards = boards.clone();
    let mut staged_boards = boards.clone();
    let mut mutations = Vec::new();
    let mut see_inputs = Vec::new();
    for board in &boards {
        for &mv in &board.generate_legal_movelist() {
            mutations.push((board.clone(), mv));
        }
        let mut capture_board = board.clone();
        for &mv in &capture_board.generate_legal_captures() {
            see_inputs.push((board.clone(), mv));
        }
    }

    // Warm lazy attack tables and every call path before arming the allocator.
    for board in &boards {
        black_box(board.generate_legal_movelist());
    }
    for board in &mut capture_boards {
        black_box(board.generate_legal_captures());
    }
    for board in &mut staged_boards {
        let (captures, pinned) = board.generate_legal_captures_pinned();
        black_box((captures, board.generate_legal_quiets_pinned(pinned)));
    }
    for (board, mv) in &mut mutations {
        board.make_move_unchecked(*mv);
        board.unmake_move(*mv);
    }
    for (board, mv) in &see_inputs {
        black_box(board.see_ge(*mv, 0));
    }

    assert_eq!(
        allocations_during(|| {
            for board in &boards {
                black_box(board.generate_legal_movelist());
            }
            for board in &mut capture_boards {
                black_box(board.generate_legal_captures());
            }
            for board in &mut staged_boards {
                let (captures, pinned) = board.generate_legal_captures_pinned();
                black_box((captures, board.generate_legal_quiets_pinned(pinned)));
            }
            for (board, mv) in &mut mutations {
                board.make_move_unchecked(*mv);
                black_box(&board);
                board.unmake_move(*mv);
            }
            for (board, mv) in &see_inputs {
                black_box(board.see_ge(*mv, 0));
            }
        }),
        0,
        "a board-v2 hot primitive allocated after warmup"
    );
}
