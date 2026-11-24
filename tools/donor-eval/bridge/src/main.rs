//! Rarog's search on a donor's classical evaluation (E.4.0.1). The engine's
//! own `src/main.rs`, with one Stockfish tag's classical evaluation
//! installed in place of Rarog's through that tag's bridge DLL
//! (`../README.md`). A diagnostic instrument, never shipped.
//!
//! Two commands of its own run before the engine starts:
//! `donor-parity <fens> <out.tsv>` checks the bridge against the tag's own
//! FEN setter, `donor-k <rows>` measures the scale the frozen rule maps
//! the donor's units with, and `donor-scale <fens>` the size ratios it
//! was chosen over.

#[cfg(not(any(feature = "sf11", feature = "sf16")))]
compile_error!("build with one donor: --features sf11 or --features sf16");
#[cfg(all(feature = "sf11", feature = "sf16"))]
compile_error!("build with one donor, not both");

use std::fs;
use std::io::Write as _;
use std::process;
use std::sync::{Arc, OnceLock};
use std::thread;

use rarog::board::{Board, Color, Piece};
use rarog::cpu_advice;
use rarog::crash_report;
use rarog::engine::Engine;
use rarog::engine_command::{EngineCommandQueue, EngineControl};
use rarog::eval::Evaluator;
use rarog::infra::{THREAD_STACK_SIZE, capitalize_first_letter};
use rarog::uci::{CommandOutcome, UciProtocol};

#[cfg(windows)]
#[global_allocator]
static ALLOCATOR: rarog::large_pages::PageAllocator = rarog::large_pages::PageAllocator;

/// The tag `rarog_donor_init` must report.
#[cfg(feature = "sf11")]
const DONOR_VERSION: i32 = 11;
#[cfg(feature = "sf16")]
const DONOR_VERSION: i32 = 16;

/// The donor's units in Rarog's, in 1/1024: the donor's K over Rarog's on
/// `hce-v4-tb`'s validation rows, positions not in check, by `donor-k`, so
/// that a score predicts the same result in both (the rule frozen in
/// RAR-O13 before any game).
#[cfg(feature = "sf11")]
const SCALE_1024: i32 = 684;
#[cfg(feature = "sf16")]
const SCALE_1024: i32 = 717;

const _: () = assert!(SCALE_1024 > 0, "the scale is frozen before any read");

/// Stockfish's `VALUE_INFINITE`. `sf_16` starts a search with its best
/// value at minus this, so its lazy cut never fires before the first
/// root window returns.
const VALUE_INFINITE: i32 = 32_001;

/// Rarog's layout of a position, as `rarog_bridge.cpp` reads it.
#[repr(C)]
struct DonorPosition {
    /// Stockfish piece codes by square, a1 = 0; 0 is empty.
    pieces: [u8; 64],
    side: u8,
    /// Rarog's castling bits, which are Stockfish's.
    castling: u8,
    /// The en-passant square, 64 for none.
    ep: u8,
    rule50: u8,
    /// The root's last score in the donor's units (`sf_16`'s lazy cut).
    root: i32,
}

/// A NUL-terminated FEN of at most 127 bytes, the only form
/// `rarog_donor_eval_fen` is handed.
#[repr(C)]
struct FenBuffer([u8; 128]);

impl FenBuffer {
    fn new(fen: &str) -> Option<Self> {
        let bytes = fen.as_bytes();
        if bytes.len() >= 128 || bytes.contains(&0) {
            return None;
        }
        let mut buffer = [0u8; 128];
        buffer[..bytes.len()].copy_from_slice(bytes);
        Some(Self(buffer))
    }
}

#[cfg_attr(feature = "sf11", link(name = "sf11_eval", kind = "raw-dylib"))]
#[cfg_attr(feature = "sf16", link(name = "sf16_eval", kind = "raw-dylib"))]
// SAFETY: each item is the DLL's `extern "C"` export of that name and
// signature (`rarog_bridge.cpp`). The position functions read the
// `DonorPosition` they are handed and nothing else of the caller's; the
// FEN function reads its buffer up to the NUL `FenBuffer` guarantees.
// None keeps a pointer past its return.
unsafe extern "C" {
    safe fn rarog_donor_init() -> i32;
    safe fn rarog_donor_eval(position: &DonorPosition) -> i32;
    safe fn rarog_donor_rebuild(position: &DonorPosition);
    safe fn rarog_donor_eval_fen(fen: &FenBuffer, root: i32) -> i32;
}

/// `RAROG_HCE_BRIDGE_REPEAT`: the bridge rebuilds each position this many
/// times, the evaluation once, so its price can be read
/// (`tools/oracle-hybrids/bridge_price.py`).
static REPEAT: OnceLock<u32> = OnceLock::new();

fn piece_code(color: Color, piece: Piece) -> u8 {
    let kind = match piece {
        Piece::Pawn => 1,
        Piece::Knight => 2,
        Piece::Bishop => 3,
        Piece::Rook => 4,
        Piece::Queen => 5,
        Piece::King => 6,
    };
    match color {
        Color::White => kind,
        Color::Black => kind + 8,
    }
}

const PIECES: [Piece; 6] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

fn donor_position(board: &Board, root: i32) -> DonorPosition {
    let mut pieces = [0u8; 64];
    for color in [Color::White, Color::Black] {
        for piece in PIECES {
            for square in board.pieces(color, piece) {
                pieces[square.index()] = piece_code(color, piece);
            }
        }
    }
    DonorPosition {
        pieces,
        side: board.side_to_move() as u8,
        castling: board.castling().0,
        ep: board.ep_square().map_or(64, |square| square.0),
        rule50: board.halfmove_clock(),
        root,
    }
}

/// Rarog's root score in the donor's units, `None` as the donor's own
/// search starts.
fn donor_root(root: Option<i32>) -> i32 {
    root.map_or(-VALUE_INFINITE, |score| {
        let scaled = i64::from(score) * 1024 / i64::from(SCALE_1024);
        i32::try_from(scaled.clamp(-i64::from(VALUE_INFINITE), i64::from(VALUE_INFINITE)))
            .unwrap_or(0)
    })
}

/// The installed evaluation: the donor's value in Rarog's units.
fn donor_eval(board: &Board, root: Option<i32>) -> i32 {
    let root = donor_root(root);
    for _ in 1..*REPEAT.get().unwrap_or(&1) {
        rarog_donor_rebuild(&donor_position(std::hint::black_box(board), root));
    }
    let value = rarog_donor_eval(&donor_position(board, root));
    value * SCALE_1024 / 1024
}

fn read_fens(path: &str) -> Vec<Board> {
    let text = fs::read_to_string(path).unwrap_or_else(|error| {
        eprintln!("cannot read {path}: {error}");
        process::exit(2)
    });
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let fen = line.split('\t').next().unwrap_or(line);
            Board::from_fen(fen).unwrap_or_else(|error| {
                eprintln!("bad FEN `{fen}`: {error}");
                process::exit(2)
            })
        })
        .collect()
}

/// The bridge's placement against the tag's own FEN setter, at the start
/// value and at two root scores, on every position not in check. Writes
/// `fen<TAB>value` at the tag's `eval` command's root (0) for the
/// comparison with the release binary. Exit 1 on any mismatch.
fn parity(fens: &str, out: &str) -> i32 {
    let boards = read_fens(fens);
    let mut file = fs::File::create(out).unwrap_or_else(|error| {
        eprintln!("cannot write {out}: {error}");
        process::exit(2)
    });
    let (mut checked, mut mismatches, mut root_moved) = (0usize, 0usize, 0usize);
    for board in boards.iter().filter(|board| !board.is_in_check()) {
        let fen = board.to_fen();
        let Some(buffer) = FenBuffer::new(&fen) else {
            eprintln!("FEN too long: {fen}");
            return 2;
        };
        let at_start = rarog_donor_eval(&donor_position(board, -VALUE_INFINITE));
        if rarog_donor_eval(&donor_position(board, 0)) != at_start {
            root_moved += 1;
        }
        for root in [0, -VALUE_INFINITE, 150, -900] {
            let placed = rarog_donor_eval(&donor_position(board, root));
            let parsed = rarog_donor_eval_fen(&buffer, root);
            if placed != parsed {
                mismatches += 1;
                println!("mismatch root {root}: placement {placed} FEN {parsed} {fen}");
            }
            if root == 0 {
                writeln!(file, "{fen}\t{placed}").expect("write the parity table");
            }
        }
        checked += 1;
    }
    println!(
        "parity: {checked} positions, 4 roots each, {mismatches} mismatches; {root_moved} positions read differently at root 0 and at the search's start"
    );
    i32::from(mismatches > 0)
}

/// Mean |Rarog's evaluation| and mean |the donor's| over the positions not
/// in check, both from the side to move, the donor at the tag's `eval`
/// command's root (0), and the scale in 1/1024 the frozen rule takes.
fn scale(fens: &str) -> i32 {
    let boards = read_fens(fens);
    let mut evaluator = Evaluator::default();
    let (mut rarog_sum, mut donor_sum, mut count) = (0f64, 0f64, 0usize);
    let mut ratios = Vec::new();
    for board in boards.iter().filter(|board| !board.is_in_check()) {
        let rarog = f64::from(evaluator.evaluate(board));
        let donor = f64::from(rarog_donor_eval(&donor_position(board, 0)));
        rarog_sum += rarog.abs();
        donor_sum += donor.abs();
        if donor != 0.0 {
            ratios.push(rarog / donor);
        }
        count += 1;
    }
    ratios.sort_by(f64::total_cmp);
    let median = ratios.get(ratios.len() / 2).copied().unwrap_or(f64::NAN);
    let (rarog_mean, donor_mean) = (rarog_sum / count as f64, donor_sum / count as f64);
    println!(
        "scale: sf{DONOR_VERSION} {count} positions not in check; mean |rarog| {rarog_mean:.2}, \
         mean |donor| {donor_mean:.2}, ratio {:.4}, SCALE_1024 {}; per-position median ratio {median:.4}",
        rarog_mean / donor_mean,
        (1024.0 * rarog_mean / donor_mean).round()
    );
    0
}

/// The K of `1 / (1 + exp(-K s / 400))` that fits `rows`' (score from
/// white, white's result) best in squared error, as the tuner fits its K.
fn fit_k(rows: &[(f64, f64)]) -> f64 {
    let loss = |k: f64| {
        rows.iter()
            .map(|&(score, result)| {
                let predicted = 1.0 / (1.0 + (-k * score / 400.0).exp());
                (result - predicted).powi(2)
            })
            .sum::<f64>()
    };
    let (mut lo, mut hi) = (0.01f64, 10.0f64);
    for _ in 0..200 {
        let m1 = lo + (hi - lo) / 3.0;
        let m2 = hi - (hi - lo) / 3.0;
        if loss(m1) < loss(m2) {
            hi = m2;
        } else {
            lo = m1;
        }
    }
    let k = (lo + hi) / 2.0;
    assert!(k > 0.011 && k < 9.99, "K at a bound: {k}");
    k
}

/// Each evaluation's K on game results (`fen;result`, the result from
/// white), positions not in check, the donor at root 0: equal K means a
/// score predicts the same result in both. The scale the frozen rule takes
/// is K_donor / K_rarog in 1/1024.
fn k_scale(rows_path: &str) -> i32 {
    let text = fs::read_to_string(rows_path).unwrap_or_else(|error| {
        eprintln!("cannot read {rows_path}: {error}");
        process::exit(2)
    });
    let mut evaluator = Evaluator::default();
    let (mut rarog, mut donor) = (Vec::new(), Vec::new());
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let Some((fen, result)) = line.rsplit_once(';') else {
            eprintln!("bad row `{line}`");
            return 2;
        };
        let result: f64 = result.trim().parse().unwrap_or_else(|_| {
            eprintln!("bad result `{line}`");
            process::exit(2)
        });
        let board = Board::from_fen(fen).unwrap_or_else(|error| {
            eprintln!("bad FEN `{fen}`: {error}");
            process::exit(2)
        });
        if board.is_in_check() {
            continue;
        }
        let white = if board.side_to_move() == Color::White {
            1.0
        } else {
            -1.0
        };
        rarog.push((white * f64::from(evaluator.evaluate(&board)), result));
        donor.push((
            white * f64::from(rarog_donor_eval(&donor_position(&board, 0))),
            result,
        ));
    }
    let (k_rarog, k_donor) = (fit_k(&rarog), fit_k(&donor));
    println!(
        "k-scale: sf{DONOR_VERSION} {} rows not in check; K rarog {k_rarog:.5}, K donor {k_donor:.5}, \
         ratio {:.4}, SCALE_1024 {}",
        rarog.len(),
        k_donor / k_rarog,
        (1024.0 * k_donor / k_rarog).round()
    );
    0
}

/// One line per row of `rows_path` (`fen;...`, anything after the FEN is
/// ignored), in order: Rarog's evaluation and the donor's raw value at root
/// 0, both from white and tab-separated, or `check` for a position in check,
/// which neither evaluation scores. The static reads join it to the rows by
/// line number.
fn dump(rows_path: &str, out: &str) -> i32 {
    let text = fs::read_to_string(rows_path).unwrap_or_else(|error| {
        eprintln!("cannot read {rows_path}: {error}");
        process::exit(2)
    });
    let mut file = std::io::BufWriter::new(fs::File::create(out).unwrap_or_else(|error| {
        eprintln!("cannot write {out}: {error}");
        process::exit(2)
    }));
    let mut evaluator = Evaluator::default();
    let (mut scored, mut in_check) = (0usize, 0usize);
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let fen = line.split(';').next().unwrap_or(line);
        let board = Board::from_fen(fen).unwrap_or_else(|error| {
            eprintln!("bad FEN `{fen}`: {error}");
            process::exit(2)
        });
        if board.is_in_check() {
            writeln!(file, "check").expect("write the dump");
            in_check += 1;
            continue;
        }
        let white = if board.side_to_move() == Color::White {
            1
        } else {
            -1
        };
        let rarog = white * evaluator.evaluate(&board);
        let donor = white * rarog_donor_eval(&donor_position(&board, 0));
        writeln!(file, "{rarog}\t{donor}").expect("write the dump");
        scored += 1;
    }
    file.flush().expect("write the dump");
    println!("dump: sf{DONOR_VERSION} {scored} rows scored, {in_check} in check");
    0
}

fn main() {
    let version = rarog_donor_init();
    assert_eq!(
        version, DONOR_VERSION,
        "the DLL beside the executable is another tag's"
    );
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("donor-parity") if args.len() == 3 => process::exit(parity(&args[1], &args[2])),
        Some("donor-scale") if args.len() == 2 => process::exit(scale(&args[1])),
        Some("donor-k") if args.len() == 2 => process::exit(k_scale(&args[1])),
        Some("donor-dump") if args.len() == 3 => process::exit(dump(&args[1], &args[2])),
        _ => {}
    }
    let repeat = std::env::var("RAROG_HCE_BRIDGE_REPEAT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1u32)
        .max(1);
    REPEAT.set(repeat).expect("set once");
    assert!(rarog::eval::install_donor(donor_eval), "installed once");
    engine_main(args);
}

/// `src/main.rs`'s `main` from here on, unchanged but for its arguments.
fn engine_main(args: Vec<String>) {
    crash_report::install_stdout_reporter();
    request_fine_grained_scheduling();

    let label = if rarog::BUILD_LABEL.is_empty() {
        String::new()
    } else {
        format!(" ({})", rarog::BUILD_LABEL)
    };
    rarog::uci_println!(
        "{} {} by {}{label}",
        capitalize_first_letter(env!("CARGO_PKG_NAME")),
        rarog::VERSION,
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    if let Some(advice) = cpu_advice::startup_advice() {
        rarog::uci_println!("{advice}");
    }

    let commands = EngineCommandQueue::default();
    let control = Arc::new(EngineControl::default());
    let engine_commands = commands.clone();
    let engine_control = Arc::clone(&control);
    let engine_thread = thread::Builder::new()
        .name("rarog-engine".to_string())
        .stack_size(THREAD_STACK_SIZE)
        .spawn(move || {
            let mut engine = Engine::new(engine_commands, engine_control);
            engine.start();
        })
        .expect("Engine thread failed to start.");

    let mut protocol = UciProtocol::new(commands, control);
    let outcome = if args.is_empty() {
        protocol.uci_loop()
    } else {
        protocol.run_once(&args.join(" "))
    };
    engine_thread.join().expect("Engine thread failed.");

    match outcome {
        CommandOutcome::Unknown => process::exit(2),
        CommandOutcome::Fatal => process::exit(1),
        CommandOutcome::Handled | CommandOutcome::Quit => {}
    }
}

#[cfg(windows)]
fn request_fine_grained_scheduling() {
    rarog::win32::timeBeginPeriod(1);
}

#[cfg(not(windows))]
fn request_fine_grained_scheduling() {}
