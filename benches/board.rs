//! Board-operation throughput benchmark (`cross-engine-board-v1`).
//!
//! The timed region contains board work and nothing else: no heap allocation,
//! no container copy, no dead code, no evaluator. Work quanta are frozen and
//! verified by a preflight pass, so "same benchmark" is enforced rather than
//! assumed.
//!
//! Every generating workload writes into a caller-owned `MoveList` built
//! before timing starts, which is what the peer harnesses do and what the
//! search itself does. The by-value form returns a 520-byte aggregate that the
//! fat-LTO build copies on the normal return path, and RAR-M44 measured that
//! copy at +11.2% on legal generation and +40.5% on captures -- it was inside
//! the timed region and reported as move generation. `see_captures` and
//! `perft` deliberately keep their own delivery: they are the untouched
//! controls for the 4.11b.19 sessions.
//!
//! The profile is shared with Basilisk (`tests/board_performance.cpp`) and
//! Manta (`tools/board_bench.zig`); the contract they all implement is written
//! down in Manta's `docs/BOARD_BENCHMARK.md`. Corpus, order, work quanta,
//! estimator (150 ms warm-up, 11 x 150 ms samples, median +- MAD) and batch
//! calibration match those two implementations.
//!
//! Threshold SEE uses the frozen P/N/B/R/Q/K = 100/300/300/500/900/20000
//! comparison vector through Board's injected-value interface. Production SEE
//! remains 100/320/330/500/900/20000. Preflight prints the exact move/verdict
//! set, so the comparison runner checks answers as well as call count.

use std::hint::black_box;
use std::time::{Duration, Instant};

use rarog::board::{
    Board, CROSS_ENGINE_SEE_VALUES, MoveList, SeeValues, generate_captures, generate_captures_into,
    generate_legal_into, perft,
};

const WARMUP: Duration = Duration::from_millis(150);
// 9.7: N shorter samples instead of one 750 ms shot. A single sample on a
// desktop is hostage to whatever the OS scheduler did during those 750 ms —
// we have measured several-percent swings on IDENTICAL binaries — so one
// number cannot distinguish a real 2% change from noise. The median resists
// scheduler outliers; the MAD is printed beside it so the output itself says
// whether a difference between two runs is resolvable or inside the noise.
//
// Eleven samples, not nine: `cross-engine-board-v1` fixes the estimator so
// that runs from different engines are comparable at the same resolution.
const SAMPLES: usize = 11;
const SAMPLE_TIME: Duration = Duration::from_millis(150);

const BENCHMARK_FENS: &[(&str, &str)] = &[
    (
        "startpos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    (
        "kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ),
    (
        "midgame",
        "rnbq1k1r/pppp1ppp/4pn2/8/1b1PP3/2N2N2/PPP2PPP/R1BQKB1R w KQ - 2 5",
    ),
    ("endgame", "8/2p5/3p4/KP5r/8/8/8/7k w - - 0 1"),
    (
        "in-check",
        "rnbqkb1r/pppp1ppp/5n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 3 3",
    ),
];

/// Frozen work quanta for the corpus above, in results order.
const EXPECTED_OPS: [u64; 6] = [128, 10, 128, 10, 197_281, 4_597];

struct BenchResult {
    label: &'static str,
    unit: &'static str,
    /// Per-sample throughput (ops/s), one entry per sample, unsorted.
    samples: Vec<f64>,
    ops_per_iter: u64,
    iterations: u64,
}

impl BenchResult {
    /// Median throughput across samples — the robust point estimate.
    fn median(&self) -> f64 {
        let mut sorted = self.samples.clone();
        sorted.sort_by(f64::total_cmp);
        sorted[sorted.len() / 2]
    }

    /// Median absolute deviation, same units as the median. A rough 1-sigma
    /// analogue that ignores scheduler outliers entirely.
    fn mad(&self) -> f64 {
        let med = self.median();
        let mut dev: Vec<f64> = self.samples.iter().map(|s| (s - med).abs()).collect();
        dev.sort_by(f64::total_cmp);
        dev[dev.len() / 2]
    }

    /// MAD as a percentage of the median — the at-a-glance noise figure.
    fn spread_pct(&self) -> f64 {
        100.0 * self.mad() / self.median()
    }
}

fn main() {
    let (preflight_only, see_values) = arguments();
    let boards: Vec<Board> = BENCHMARK_FENS
        .iter()
        .map(|(_, fen)| Board::from_fen(fen).unwrap())
        .collect();

    let mut capture_boards = boards.clone();
    let mut mutable_boards = boards.clone();
    let mut see_boards = boards.clone();
    let mut simulation_boards = boards.clone();
    // Built here, not inside the workload: the contract requires the working
    // set to exist before timing starts and never be constructed in the timed
    // region. `perft` restores it exactly through make/unmake, so one board
    // serves every sample — which is also what the peer implementations do.
    let mut perft_board = Board::starting_position();
    // Caller-owned lists, built here for the same reason the boards are: a
    // container the workload constructs (or receives by value) is charged to
    // board throughput and is not board work.
    let mut scratch = MoveList::new();
    let mut outer = MoveList::new();
    let mut inner = MoveList::new();

    // Preflight: prove every work quantum before timing anything. A workload
    // that generates a different number of moves than its peers is not the
    // same benchmark, however similar the label looks.
    {
        let measured = [
            legal_movegen(&boards, &mut scratch),
            capture_gen(&mut capture_boards, &mut scratch),
            make_unmake(&mut mutable_boards, &mut scratch),
            see_captures(&mut see_boards, see_values),
            perft(&mut perft_board, 4),
            game_simulation(&mut simulation_boards, &mut outer, &mut inner),
        ];
        let mut ok = true;
        for (i, (&got, &want)) in measured.iter().zip(EXPECTED_OPS.iter()).enumerate() {
            if got != want {
                eprintln!("work mismatch for workload {i}: expected {want}, received {got}");
                ok = false;
            }
        }
        assert!(
            ok,
            "preflight failed: work quanta do not match the contract"
        );
    }

    println!("see-values: {}", format_values(see_values));
    println!("see-verdicts: {}", see_verdicts(&boards, see_values));
    println!("see-probe: {}", see_probe(see_values));
    if preflight_only {
        println!("preflight: PASS");
        return;
    }

    let results = [
        measure("legal moves", "moves", EXPECTED_OPS[0], || {
            legal_movegen(&boards, &mut scratch)
        }),
        measure("legal captures", "moves", EXPECTED_OPS[1], || {
            capture_gen(&mut capture_boards, &mut scratch)
        }),
        measure("make/unmake", "moves", EXPECTED_OPS[2], || {
            make_unmake(&mut mutable_boards, &mut scratch)
        }),
        measure("threshold SEE", "captures", EXPECTED_OPS[3], || {
            see_captures(&mut see_boards, see_values)
        }),
        measure("perft(4) startpos", "nodes", EXPECTED_OPS[4], || {
            perft(&mut perft_board, 4)
        }),
        measure("two-ply simulation", "moves", EXPECTED_OPS[5], || {
            game_simulation(&mut simulation_boards, &mut outer, &mut inner)
        }),
    ];

    println!();
    println!("Rarog board benchmark");
    println!("profile: cross-engine-board-v1");
    println!("positions: {}", BENCHMARK_FENS.len());
    println!(
        "samples: {SAMPLES} x {} ms after a {} ms warm-up (median +/- MAD)",
        SAMPLE_TIME.as_millis(),
        WARMUP.as_millis()
    );
    println!("preflight: PASS");
    println!();
    println!(
        "{:<22} {:>15} {:>15} {:>10} {:>12} {:>12}",
        "workload", "estimate ops/s", "MAD ops/s", "MAD %", "ops/iter", "total iters"
    );

    for result in &results {
        println!(
            "{:<22} {:>15.0} {:>15.0} {:>9.2}% {:>12} {:>12} {}",
            result.label,
            result.median(),
            result.mad(),
            result.spread_pct(),
            result.ops_per_iter,
            result.iterations,
            result.unit
        );
    }
}

fn arguments() -> (bool, SeeValues) {
    let mut preflight_only = false;
    let mut values = CROSS_ENGINE_SEE_VALUES;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bench" => {}
            "--preflight-only" => preflight_only = true,
            "--see-values" => {
                let raw = args.next().expect("--see-values requires P,N,B,R,Q,K");
                let parsed: Vec<i32> = raw
                    .split(',')
                    .map(|item| item.parse().expect("SEE values must be integers"))
                    .collect();
                assert_eq!(
                    parsed.len(),
                    6,
                    "--see-values requires exactly six integers"
                );
                values = SeeValues::new(
                    parsed[0], parsed[1], parsed[2], parsed[3], parsed[4], parsed[5],
                );
            }
            _ => panic!("usage: board [--preflight-only] [--see-values P,N,B,R,Q,K]"),
        }
    }
    (preflight_only, values)
}

fn format_values(values: SeeValues) -> String {
    values.as_array().map(|value| value.to_string()).join("/")
}

fn see_verdicts(boards: &[Board], values: SeeValues) -> String {
    let mut verdicts = Vec::new();
    for (index, original) in boards.iter().enumerate() {
        let mut board = original.clone();
        for &mv in &generate_captures(&mut board) {
            verdicts.push(format!(
                "{index}:{mv}={}",
                u8::from(board.see_ge_with_values(mv, 0, values))
            ));
        }
    }
    verdicts.sort();
    verdicts.join(",")
}

fn see_probe(values: SeeValues) -> bool {
    let board = Board::from_fen("4k3/8/2p5/3p4/8/8/3R4/4K3 w - - 0 1").unwrap();
    let mv = board.parse_move("d2d5").unwrap();
    board.see_ge_with_values(mv, 0, values)
}

/// Pick an inner batch size so the deadline clock is read about once per
/// millisecond. `Instant::now()` costs tens of nanoseconds; the 10-op
/// workloads run an iteration in ~100 ns, so checking the deadline once per
/// iteration would leave a double-digit percentage of clock overhead inside
/// the timed region and report it as board throughput.
fn calibrate_batch<F>(workload: &mut F) -> u64
where
    F: FnMut() -> u64,
{
    const TARGET: Duration = Duration::from_millis(1);
    const MAX_BATCH: u64 = 1_000_000;
    const PROBES: u32 = 32;

    let start = Instant::now();
    for _ in 0..PROBES {
        black_box(workload());
    }
    let per_iter = start.elapsed() / PROBES;
    if per_iter.is_zero() {
        return MAX_BATCH;
    }
    let batch = TARGET.as_nanos() / per_iter.as_nanos();
    u64::try_from(batch)
        .unwrap_or(MAX_BATCH)
        .clamp(1, MAX_BATCH)
}

fn measure<F>(
    label: &'static str,
    unit: &'static str,
    expected: u64,
    mut workload: F,
) -> BenchResult
where
    F: FnMut() -> u64,
{
    let warmup_start = Instant::now();
    while warmup_start.elapsed() < WARMUP {
        black_box(workload());
    }

    let batch = calibrate_batch(&mut workload);

    let mut samples = Vec::with_capacity(SAMPLES);
    let mut iterations = 0u64;
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let mut ops = 0u64;
        let mut iters = 0u64;
        while start.elapsed() < SAMPLE_TIME {
            for _ in 0..batch {
                ops += black_box(workload());
            }
            iters += batch;
        }
        // Read the clock BEFORE checking the quantum, so the check itself is
        // outside the timed region rather than charged to board throughput.
        let elapsed = start.elapsed().as_secs_f64();
        assert_eq!(
            ops,
            expected * iters,
            "{label} drifted off its frozen work quantum mid-measurement"
        );
        samples.push(ops as f64 / elapsed);
        iterations += iters;
    }

    BenchResult {
        label,
        unit,
        samples,
        ops_per_iter: expected,
        iterations,
    }
}

// `generate_legal_into` writes into a fixed-capacity `MoveList` the caller
// owns. The convenience wrapper `generate_legal_moves` returns a `Vec<Move>`
// and so puts a `Vec::with_capacity(48)` malloc/free pair inside the timed
// region — that allocation was worth 17-43% on the four workloads that used
// it, and it is allocator throughput, not board throughput. The by-value
// `generate_legal_movelist` has the same defect in smaller print: a 520-byte
// return copy. Search itself uses the out-parameter form, so this is also the
// more representative call.
fn legal_movegen(boards: &[Board], moves: &mut MoveList) -> u64 {
    let mut total = 0u64;
    for board in boards {
        generate_legal_into(black_box(board), moves);
        total += moves.len() as u64;
        black_box(&*moves);
    }
    total
}

fn capture_gen(boards: &mut [Board], moves: &mut MoveList) -> u64 {
    let mut total = 0u64;
    for board in boards {
        generate_captures_into(black_box(board), moves);
        total += moves.len() as u64;
        black_box(&*moves);
    }
    total
}

fn make_unmake(boards: &mut [Board], moves: &mut MoveList) -> u64 {
    let mut ops = 0u64;
    for board in boards {
        generate_legal_into(board, moves);
        for &mv in moves.as_slice() {
            board.make_move(mv);
            black_box(&board);
            board.unmake_move(mv);
            ops += 1;
        }
    }
    ops
}

// Measures `see_ge(mv, 0)`, the threshold form, not the full-value `see()`.
// Two independent reasons, and the cross-engine one is the weaker of them:
//
//   OWN HOT PATH — search.rs calls `see_ge` in twelve places (capture pruning,
//   the qsearch bad-capture floor, LMR/ordering gates) and full `see()` in
//   exactly one (move-ordering score at 3657). The threshold form is what
//   actually dominates the search loop, so it is what a board benchmark should
//   report. The old workload measured the rarer call.
//
//   COMPARABILITY — the peer engines measure `see_ge(m, 0)` over the same five
//   FENs. Measuring the same operation makes the numbers a comparison rather
//   than a coincidence of labels.
//
// The two are genuinely different operations, not two spellings of one: the
// threshold form early-exits as soon as the running balance settles the
// question, so it is expected to be the faster of the two. Both are pin-aware.
fn see_captures(boards: &mut [Board], values: SeeValues) -> u64 {
    let mut ops = 0u64;
    for board in boards {
        let captures = generate_captures(board);
        for &mv in &captures {
            black_box(board.see_ge_with_values(mv, 0, values));
            ops += 1;
        }
    }
    ops
}

fn game_simulation(boards: &mut [Board], outer: &mut MoveList, inner: &mut MoveList) -> u64 {
    let mut ops = 0u64;
    for board in boards {
        generate_legal_into(board, outer);
        for &mv in outer.as_slice() {
            board.make_move(mv);
            generate_legal_into(board, inner);
            ops += inner.len() as u64;
            black_box(&*inner);
            board.unmake_move(mv);
        }
    }
    ops
}
