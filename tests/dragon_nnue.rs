#![cfg(not(feature = "texel"))]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::OnceLock;

use rarog::board::{Board, Color};
use rarog::eval::{EvaluationOptions, Evaluator};
use rarog::search::{SearchEvent, Searcher};
use rarog::search_options::SearchOptions;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
struct Allocator;
// SAFETY: All memory operations are forwarded unchanged to System. Counters
// are thread local and do not allocate inside the allocator.
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
        }
        // SAFETY: The caller supplied the layout.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: The live pointer and layout are passed through unchanged.
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNTING.try_with(Cell::get).unwrap_or(false) {
            let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
        }
        // SAFETY: The caller supplied the original allocation and new size.
        unsafe { System.realloc(ptr, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

fn options() -> &'static EvaluationOptions {
    static OPTIONS: OnceLock<EvaluationOptions> = OnceLock::new();
    OPTIONS.get_or_init(|| {
        let mut options = EvaluationOptions::default();
        if let Ok(file) = std::env::var("RAROG_DRAGON_NNUE") {
            options.set_file(&file).unwrap();
        }
        options.set_mode("Dragon NNUE").unwrap();
        options
    })
}

#[test]
fn dragon_matches_native_fixtures() {
    let mut evaluation = Evaluator::default();
    let mut settings = options().clone();
    let mut accepted = Vec::new();
    let mut rejected = 0;
    // Eligibility depends only on the host FEN contract, never on scores.
    for text in [
        include_str!("fixtures/dragon-nnue-reference.tsv"),
        include_str!("fixtures/dragon-nnue-expanded.tsv"),
    ] {
        for row in text.lines().filter(|s| !s.starts_with('#')) {
            let fields: Vec<_> = row.split('\t').collect();
            match Board::from_fen(fields[3]) {
                Ok(_) => accepted.push(row),
                Err(_) => rejected += 1,
            }
        }
    }
    println!(
        "host parser accepted {} fixture rows; rejected {rejected}",
        accepted.len()
    );
    assert!(accepted.len() >= 500);
    // Export the preselected corpus for fresh original-process comparisons.
    if let Ok(path) = std::env::var("RAROG_DRAGON_ACCEPTED_FIXTURES") {
        std::fs::write(path, accepted.join("\n") + "\n").unwrap();
    }
    for row in &accepted {
        let fields: Vec<_> = row.split('\t').collect();
        let board = Board::from_fen(fields[3]).unwrap();
        settings.set_scale(fields[1].parse().unwrap()).unwrap();
        evaluation.configure_evaluation(&settings);
        let score = evaluation.evaluate(&board);
        let white = if board.side_to_move() == Color::Black {
            -score
        } else {
            score
        };
        assert_eq!(
            white,
            fields[2].parse::<i32>().unwrap(),
            "{} scale {}",
            fields[0],
            fields[1]
        );
    }
}

#[test]
fn dragon_switches_caches_and_restores_moves_and_nulls() {
    let mut eval = Evaluator::default();
    let mut board = Board::default();
    let hce = eval.evaluate(&board);
    let mut pending = EvaluationOptions::default();
    pending
        .set_file("missing-network-before-mode.nnue")
        .unwrap();
    assert!(pending.set_mode("Dragon NNUE").is_err());
    pending.set_file(options().file()).unwrap();
    assert_eq!(pending.mode(), rarog::eval::EvaluationMode::DragonNnue);
    let mut settings = options().clone();
    assert!(eval.configure_evaluation(&settings));
    assert_eq!(eval.evaluate(&board), 52);
    assert!(!eval.configure_evaluation(&settings));
    settings.set_scale(100).unwrap();
    assert!(eval.configure_evaluation(&settings));
    assert_eq!(eval.evaluate(&board), 73);
    board.make_null_move();
    assert_eq!(eval.evaluate(&board), 73);
    board.unmake_null_move();
    assert_eq!(eval.evaluate(&board), 73);
    for seed in [
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "8/P7/7k/8/8/8/7p/4K3 w - - 0 1",
    ] {
        let mut b = Board::from_fen(seed).unwrap();
        let before = eval.evaluate(&b);
        for mv in b.generate_legal_moves() {
            b.make_move(mv);
            let mut cold = Evaluator::default();
            cold.configure_evaluation(&settings);
            assert_eq!(eval.evaluate(&b), cold.evaluate(&b));
            b.unmake_move(mv);
            assert_eq!(eval.evaluate(&b), before);
        }
    }
    settings.set_mode("Classical").unwrap();
    assert!(eval.configure_evaluation(&settings));
    assert_eq!(eval.evaluate(&board), hce);
    settings.set_mode("Dragon NNUE").unwrap();
    assert!(settings.set_file("missing-dragon-network.nnue").is_err());
    assert_eq!(settings.mode(), rarog::eval::EvaluationMode::DragonNnue);
    eval.configure_evaluation(&settings);
    assert_eq!(eval.evaluate(&board), 73);
}

#[test]
fn dragon_adapter_evaluation_allocates_nothing() {
    let mut eval = Evaluator::default();
    eval.configure_evaluation(options());
    let board = Board::default();
    ALLOCATIONS.with(|n| n.set(0));
    COUNTING.with(|on| on.set(true));
    let score = std::hint::black_box(eval.evaluate(std::hint::black_box(&board)));
    COUNTING.with(|on| on.set(false));
    assert_eq!(score, 52);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
}

#[test]
fn dragon_can_play_with_one_and_two_workers() {
    for threads in [1, 2] {
        let mut options = SearchOptions::default();
        options.engine.evaluation = crate::options().clone();
        options.engine.threads = threads;
        options.engine.hash_mb = 16;
        options.limits.nodes = 2000;
        let mut searcher = Searcher::default();
        searcher.configure(&options.engine);
        for _ in 0..8 {
            let legal = options.board.generate_legal_moves();
            if legal.is_empty() {
                break;
            }
            let result =
                searcher.search(options.board.clone(), &options, false, || SearchEvent::None);
            assert!(legal.contains(&result.bestmove));
            options.board.make_move(result.bestmove);
        }
    }
}
