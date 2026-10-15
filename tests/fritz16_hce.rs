#![cfg(not(feature = "texel"))]

use rarog::board::{Board, Color, Piece};
use rarog::eval::{EvaluationMode, EvaluationOptions, Evaluator};
use rarog::search::{SearchEvent, Searcher};
use rarog::search_options::SearchOptions;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
struct Allocator;
// SAFETY: Every allocation operation is forwarded unchanged to System; the
// thread-local counters do not allocate and do not affect allocator ownership.
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
        }
        // SAFETY: The original caller's layout is forwarded unchanged.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: The caller's live allocation and original layout are unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNTING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
        }
        // SAFETY: The live allocation and requested size are forwarded unchanged.
        unsafe { System.realloc(ptr, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

fn options() -> EvaluationOptions {
    let mut options = EvaluationOptions::default();
    options.set_mode("Fritz16HCE").unwrap();
    options
}

fn native_input(board: &Board) -> fritz16_hce::Position {
    fritz16_hce::Position {
        pieces: std::array::from_fn(|i| {
            board
                .pieces(
                    if i < 6 { Color::White } else { Color::Black },
                    [
                        Piece::Pawn,
                        Piece::Knight,
                        Piece::Bishop,
                        Piece::Rook,
                        Piece::Queen,
                        Piece::King,
                    ][i % 6],
                )
                .0
        }),
        black_to_move: board.side_to_move() == Color::Black,
        castling: board.castling().0,
    }
}

#[test]
fn all_three_evaluators_remain_selectable_and_caches_follow_configuration() {
    let board = Board::default();
    let mut eval = Evaluator::default();
    let classical = eval.evaluate(&board);
    let mut settings = options();
    assert!(eval.configure_evaluation(&settings));
    assert_eq!(settings.mode(), EvaluationMode::Fritz16Hce);
    assert_eq!(eval.evaluate(&board), 2);
    assert!(!eval.configure_evaluation(&settings));
    settings.set_mode("DragonNNUE").unwrap();
    assert!(eval.configure_evaluation(&settings));
    assert_eq!(eval.evaluate(&board), 52);
    settings.set_mode("Fritz16HCE").unwrap();
    assert!(eval.configure_evaluation(&settings));
    assert_eq!(eval.evaluate(&board), 2);
    settings.set_mode("Classical").unwrap();
    assert!(eval.configure_evaluation(&settings));
    assert_eq!(eval.evaluate(&board), classical);
    assert!(
        SearchOptions::get_uci_options()
            .iter()
            .any(|s| s.contains("var Classical var DragonNNUE var Fritz16HCE"))
    );
}

#[test]
fn adapter_matches_standalone_through_moves_undo_and_null_moves() {
    let mut eval = Evaluator::default();
    eval.configure_evaluation(&options());
    let mut reference = fritz16_hce::Evaluator::default();
    let mut seed = 0x5782b8a9546c31efu64;
    for fen in [
        rarog::board::STARTING_FEN,
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "8/P7/7k/8/8/8/7p/4K3 w - - 0 1",
    ] {
        let mut board = Board::from_fen(fen).unwrap();
        let mut played = Vec::new();
        for _ in 0..100 {
            assert_eq!(
                eval.evaluate(&board),
                reference.evaluate(&native_input(&board))
            );
            if !board.is_in_check() {
                board.make_null_move();
                assert_eq!(
                    eval.evaluate(&board),
                    reference.evaluate(&native_input(&board))
                );
                board.unmake_null_move();
            }
            let legal = board.generate_legal_moves();
            if legal.is_empty() {
                break;
            }
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let mv = legal[usize::try_from(seed % u64::try_from(legal.len()).unwrap()).unwrap()];
            board.make_move(mv);
            played.push(mv);
        }
        while let Some(mv) = played.pop() {
            board.unmake_move(mv);
            assert_eq!(
                eval.evaluate(&board),
                reference.evaluate(&native_input(&board))
            );
        }
    }
}

#[test]
fn fritz_can_play_with_one_and_two_workers() {
    for threads in [1, 2] {
        let mut settings = SearchOptions::default();
        settings.engine.evaluation = options();
        settings.engine.threads = threads;
        settings.engine.hash_mb = 16;
        settings.limits.nodes = 2_000;
        let mut searcher = Searcher::default();
        searcher.configure(&settings.engine);
        for _ in 0..4 {
            let legal = settings.board.generate_legal_moves();
            let result = searcher.search(settings.board.clone(), &settings, false, || {
                SearchEvent::None
            });
            assert!(legal.contains(&result.bestmove));
            settings.board.make_move(result.bestmove);
        }
    }
}

#[test]
fn fritz_inference_allocates_nothing_on_cold_or_warm_caches() {
    let mut eval = Evaluator::default();
    eval.configure_evaluation(&options());
    let boards = [
        Board::default(),
        Board::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap(),
        Board::from_fen("8/P7/7k/8/8/8/7p/4K3 w - - 0 1").unwrap(),
    ];
    ALLOCATIONS.with(|n| n.set(0));
    COUNTING.with(|on| on.set(true));
    for _ in 0..4 {
        for board in &boards {
            std::hint::black_box(eval.evaluate(board));
        }
    }
    COUNTING.with(|on| on.set(false));
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
}
