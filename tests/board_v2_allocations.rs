//! Allocation guard for the isolated board-v2 primitives.
//!
//! Inputs are built before the counter is armed.  If a hot board primitive
//! regresses to heap allocation, this test fails instead of letting a timing
//! benchmark mistake allocator throughput for board throughput.

use std::hint::black_box;

use rarog::board::{Board, MoveList};

#[path = "common/counting.rs"]
mod counting;

#[global_allocator]
static ALLOCATOR: counting::CountingAllocator = counting::CountingAllocator;

fn allocations_during(work: impl FnOnce()) -> u64 {
    let before = counting::allocations();
    work();
    counting::allocations() - before
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
        let mut captures = MoveList::new();
        let pinned = board.generate_legal_captures_pinned_into(&mut captures);
        let mut quiets = MoveList::new();
        board.generate_legal_quiets_pinned_into(pinned, &mut quiets);
        black_box((captures, quiets));
    }
    for (board, mv) in &mut mutations {
        board.make_move(*mv);
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
                let mut captures = MoveList::new();
                let pinned = board.generate_legal_captures_pinned_into(&mut captures);
                let mut quiets = MoveList::new();
                board.generate_legal_quiets_pinned_into(pinned, &mut quiets);
                black_box((captures, quiets));
            }
            for (board, mv) in &mut mutations {
                board.make_move(*mv);
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
