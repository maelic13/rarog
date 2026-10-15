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
        options.set_mode("DragonNNUE").unwrap();
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
    assert!(pending.set_mode("DragonNNUE").is_err());
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
    settings.set_mode("DragonNNUE").unwrap();
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

fn pieces(board: &Board) -> [u64; 12] {
    std::array::from_fn(|i| {
        board
            .pieces(
                if i < 6 { Color::White } else { Color::Black },
                [
                    rarog::board::Piece::Pawn,
                    rarog::board::Piece::Knight,
                    rarog::board::Piece::Bishop,
                    rarog::board::Piece::Rook,
                    rarog::board::Piece::Queen,
                    rarog::board::Piece::King,
                ][i % 6],
            )
            .0
    })
}

fn reference_model() -> &'static dragon_nnue::Network {
    static MODEL: OnceLock<dragon_nnue::Network> = OnceLock::new();
    MODEL.get_or_init(|| {
        dragon_nnue::Network::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/nnue/dragon-network.nnue"
        ))
        .unwrap()
    })
}

#[test]
fn incremental_kernels_match_scalar_and_native_on_arbitrary_jumps() {
    use dragon_nnue::{AccumulatorCache, Kernel, Position};
    let model = reference_model();
    for kernel in [Kernel::Portable, Kernel::Avx2, Kernel::AvxVnni] {
        let Some(mut cache) = AccumulatorCache::with_kernel(kernel) else {
            continue;
        };
        let mut rows = 0;
        let corpus: Vec<_> = [
            include_str!("fixtures/dragon-nnue-reference.tsv"),
            include_str!("fixtures/dragon-nnue-expanded.tsv"),
        ]
        .into_iter()
        .flat_map(str::lines)
        .filter(|r| !r.starts_with('#'))
        .collect();
        for row in corpus.into_iter().rev() {
            let f: Vec<_> = row.split('\t').collect();
            let board = Board::from_fen(f[3]).unwrap();
            let scale = f[1].parse().unwrap();
            let expected = model
                .evaluate_scaled(&Position::from_fen(f[3]).unwrap(), scale)
                .unwrap();
            let actual = cache
                .evaluate_scaled(
                    model,
                    &pieces(&board),
                    board.side_to_move() == Color::Black,
                    scale,
                )
                .unwrap();
            assert_eq!(actual, expected, "{kernel:?} {}", f[0]);
            assert_eq!(
                if board.side_to_move() == Color::Black {
                    -actual
                } else {
                    actual
                },
                f[2].parse::<i32>().unwrap()
            );
            if rows % 97 == 0 {
                cache.clear();
            }
            rows += 1;
        }
        assert_eq!(rows, 2738);
    }
}

#[test]
fn incremental_kernels_follow_legal_walks_unmake_null_and_special_moves() {
    use dragon_nnue::{AccumulatorCache, Kernel, Position};
    let model = reference_model();
    let mut caches: Vec<_> = [Kernel::Portable, Kernel::Avx2, Kernel::AvxVnni]
        .into_iter()
        .filter_map(AccumulatorCache::with_kernel)
        .collect();
    let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
    let mut check = |board: &Board| {
        let p = Position::from_fen(&board.to_fen()).unwrap();
        let expected = model.evaluate_scaled(&p, 72).unwrap();
        for cache in &mut caches {
            assert_eq!(
                cache
                    .evaluate_scaled(
                        model,
                        &pieces(board),
                        board.side_to_move() == Color::Black,
                        72
                    )
                    .unwrap(),
                expected,
                "{}",
                cache.kernel().name()
            );
        }
    };
    for fen in [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "8/P7/7k/8/8/8/7p/4K3 w - - 0 1",
    ] {
        let mut board = Board::from_fen(fen).unwrap();
        check(&board);
        for mv in board.generate_legal_moves() {
            board.make_move(mv);
            check(&board);
            board.unmake_move(mv);
            check(&board);
        }
        let mut moves = Vec::new();
        for _ in 0..160 {
            let legal = board.generate_legal_moves();
            if legal.is_empty() {
                break;
            }
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let index = usize::try_from(seed % u64::try_from(legal.len()).unwrap()).unwrap();
            let mv = legal[index];
            board.make_move(mv);
            moves.push(mv);
            check(&board);
            if !board.is_in_check() {
                board.make_null_move();
                check(&board);
                board.unmake_null_move();
                check(&board);
            }
        }
        while let Some(mv) = moves.pop() {
            board.unmake_move(mv);
            check(&board);
        }
    }
}

#[test]
fn warm_incremental_evaluation_allocates_nothing() {
    use dragon_nnue::AccumulatorCache;
    let model = reference_model();
    let mut cache = AccumulatorCache::new();
    let mut board = Board::default();
    let before = pieces(&board);
    assert!(board.play_uci("e2e4"));
    let after = pieces(&board);
    ALLOCATIONS.with(|n| n.set(0));
    COUNTING.with(|n| n.set(true));
    for _ in 0..8 {
        std::hint::black_box(cache.evaluate_scaled(model, &before, false, 72).unwrap());
        std::hint::black_box(cache.evaluate_scaled(model, &after, true, 72).unwrap());
    }
    COUNTING.with(|n| n.set(false));
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
}
