//! Isolated board-operation benchmark (`rarog-board-v2`).
//!
//! This is deliberately separate from the frozen `cross-engine-board-v1`
//! benchmark.  Inputs are precomputed before timing, so generation, mutation,
//! staged generation and threshold SEE stay distinguishable.  `black_box` is
//! Rust's portable optimizer barrier: each result is consumed inside the timed
//! loop and the final checksum is printed, keeping its data dependency live.

use std::hint::black_box;
use std::time::{Duration, Instant};

use rarog::board::{Board, Move};

const PROFILE: &str = include_str!("../tests/data/board-v2.tsv");
const ORACLE: &str = include_str!("../tests/data/board-v2-oracle.tsv");
const WARMUP: Duration = Duration::from_millis(25);
const SAMPLE_TIME: Duration = Duration::from_millis(25);
const SAMPLES: usize = 7;

struct ResultRow {
    label: &'static str,
    unit: &'static str,
    work: u64,
    samples: Vec<f64>,
    checksum: u64,
}

fn main() {
    let boards = profile_boards();
    preflight(&boards);

    let mut capture_boards = boards.clone();
    let mut staged_boards = boards.clone();
    let mut mutations = Vec::new();
    let mut see_inputs = Vec::new();
    for board in &boards {
        for &mv in &board.generate_legal_movelist() {
            mutations.push((board.clone(), mv));
        }
        let mut captures = board.clone();
        for &mv in &captures.generate_legal_captures() {
            see_inputs.push((board.clone(), mv));
        }
    }

    let legal_work = legal_moves(&boards);
    let capture_work = captures(&mut capture_boards);
    let staged_work = staged_moves(&mut staged_boards);
    let mutation_work = mutate(&mut mutations);
    let see_work = see(&see_inputs);

    println!("Rarog board benchmark");
    println!("profile: rarog-board-v2 (isolated primitives)");
    println!("preflight: PASS (external legal/capture identities)");
    println!("manifest.arch: {}", std::env::consts::ARCH);
    println!("manifest.os: {}", std::env::consts::OS);
    println!("manifest.backend: {}", backend());
    println!("manifest.texel: {}", cfg!(feature = "texel"));
    println!(
        "samples: {SAMPLES} x {} ms after {} ms warm-up",
        SAMPLE_TIME.as_millis(),
        WARMUP.as_millis()
    );

    let results = [
        measure("legal generation", "moves", legal_work, || {
            legal_moves(&boards)
        }),
        measure("capture generation", "moves", capture_work, || {
            captures(&mut capture_boards)
        }),
        measure("staged generation", "moves", staged_work, || {
            staged_moves(&mut staged_boards)
        }),
        measure("make/unmake only", "moves", mutation_work, || {
            mutate(&mut mutations)
        }),
        measure("threshold SEE only", "captures", see_work, || {
            see(&see_inputs)
        }),
    ];

    let mut final_checksum = 0u64;
    for row in &results {
        let median = median(&row.samples);
        final_checksum ^= row.checksum;
        println!(
            "summary|{}|{}|work={}|median_ops_s={median:.0}|checksum={}",
            row.label, row.unit, row.work, row.checksum
        );
        for (index, sample) in row.samples.iter().enumerate() {
            println!("sample|{}|{index}|ops_s={sample:.3}", row.label);
        }
    }
    println!("live-output-checksum: {final_checksum}");
}

fn backend() -> &'static str {
    #[cfg(rarog_pext)]
    {
        "pext"
    }
    #[cfg(not(rarog_pext))]
    {
        "magic"
    }
}

fn profile_boards() -> Vec<Board> {
    PROFILE
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<_> = line.split('|').collect();
            assert_eq!(fields.len(), 4, "bad board-v2 profile row: {line}");
            Board::from_fen(fields[2]).unwrap_or_else(|error| panic!("bad {}: {error}", fields[0]))
        })
        .collect()
}

fn preflight(boards: &[Board]) {
    let oracle: Vec<_> = ORACLE
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect();
    assert_eq!(
        boards.len(),
        oracle.len(),
        "profile and oracle length differ"
    );
    for (board, line) in boards.iter().zip(oracle) {
        let fields: Vec<_> = line.split('|').collect();
        assert_eq!(fields.len(), 8, "bad board-v2 oracle row: {line}");
        let legal = board.generate_legal_movelist().len();
        let mut capture_board = board.clone();
        let captures = capture_board.generate_legal_captures().len();
        assert_eq!(
            legal,
            fields[4].split_whitespace().count(),
            "{} legal moves",
            fields[0]
        );
        assert_eq!(
            captures,
            fields[5].split_whitespace().count(),
            "{} captures",
            fields[0]
        );
    }
}

fn measure<F>(label: &'static str, unit: &'static str, work: u64, mut run: F) -> ResultRow
where
    F: FnMut() -> u64,
{
    let warmup = Instant::now();
    while warmup.elapsed() < WARMUP {
        black_box(run());
    }
    let mut samples = Vec::with_capacity(SAMPLES);
    let mut checksum = 0u64;
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let mut operations = 0u64;
        let mut iterations = 0u64;
        while start.elapsed() < SAMPLE_TIME {
            let current = black_box(run());
            assert_eq!(current, work, "{label} work changed during timing");
            operations += current;
            checksum ^= current.rotate_left((iterations % 63) as u32);
            iterations += 1;
        }
        samples.push(operations as f64 / start.elapsed().as_secs_f64());
    }
    ResultRow {
        label,
        unit,
        work,
        samples,
        checksum,
    }
}

fn median(samples: &[f64]) -> f64 {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[sorted.len() / 2]
}

fn legal_moves(boards: &[Board]) -> u64 {
    boards
        .iter()
        .map(|board| {
            let moves = black_box(board.generate_legal_movelist());
            black_box(&moves);
            moves.len() as u64
        })
        .sum()
}

fn captures(boards: &mut [Board]) -> u64 {
    boards
        .iter_mut()
        .map(|board| {
            let moves = black_box(board.generate_legal_captures());
            black_box(&moves);
            moves.len() as u64
        })
        .sum()
}

fn staged_moves(boards: &mut [Board]) -> u64 {
    boards
        .iter_mut()
        .map(|board| {
            let (captures, pinned) = board.generate_legal_captures_pinned();
            let quiets = board.generate_legal_quiets_pinned(pinned);
            black_box((&captures, &quiets));
            (captures.len() + quiets.len()) as u64
        })
        .sum()
}

fn mutate(inputs: &mut [(Board, Move)]) -> u64 {
    for (board, mv) in &mut *inputs {
        board.make_move_unchecked(*mv);
        black_box(&board);
        board.unmake_move(*mv);
    }
    inputs.len() as u64
}

fn see(inputs: &[(Board, Move)]) -> u64 {
    for (board, mv) in inputs {
        black_box(board.see_ge(*mv, 0));
    }
    inputs.len() as u64
}
