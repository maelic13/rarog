use std::process;
use std::sync::Arc;
use std::thread;

use rarog::cpu_advice;
use rarog::crash_report;
use rarog::engine::Engine;
use rarog::engine_command::{EngineCommandQueue, EngineControl};
use rarog::infra::capitalize_first_letter;
use rarog::uci_protocol::{CommandOutcome, UciProtocol};

// 9.0b: Rarog supports 64-bit targets only (user decision 2026-07-19 — the
// shipped arches are x86-64/x86-64-v3, and 64-bit is what makes the
// `u64 → usize` hash-indexing conversions in the TT/eval caches lossless; see
// `infra::index`). Fail at COMPILE time on anything else instead of shipping
// silently-wrong index math.
#[cfg(not(target_pointer_width = "64"))]
compile_error!("Rarog supports only 64-bit targets (u64 hash -> usize indexing relies on it).");

const ENGINE_THREAD_STACK_SIZE: usize = 16 * 1024 * 1024;

// 4.8a — THE STARTUP CPU GUARD IS GONE, because it never worked.
//
// Until now `main` opened with a BMI2 check meant to turn "you downloaded the
// wrong asset" into a sentence instead of `STATUS_ILLEGAL_INSTRUCTION`. It
// could not: `std::is_x86_feature_detected!` expands to
// `cfg!(target_feature = "…") || runtime_detect(…)`, and the PEXT tier sets
// `-C target-feature=+bmi2`, so the macro folded to a compile-time `true`, the
// branch folded to dead code, and the message was stripped. **A runtime check
// for a feature the build STATICALLY requires is `true` by construction.**
//
// Measured, not deduced: the released `rarog-v2.3.0-windows-pext-pgo.exe` and
// `rarog-v2.3.1-windows-pext-pgo.exe` contain no trace of the message string.
// The promise has never been able to fire in a shipped asset.
//
// Rewriting it to cover AVX2 as well reproduced the same dead code for the same
// reason, which is what exposed the mechanism. A guard that actually fires needs
// raw `CPUID` from a translation unit compiled at the BASELINE — the tier flags
// otherwise license the compiler to emit tier instructions inside the guard
// itself, and ahead of it in `main`. That is one new FFI site against a frozen
// unsafe floor (PLAN principle #8), so it is a decision to take deliberately
// rather than a detail to slip in here; PLAN 4.8a records it as the open option.
//
// What replaces it is the honest half of PLAN 4.8's own instruction: state the
// requirement exactly. `README` now lists the measured CPU requirement per
// asset, and `cargo xtask verify-isa` proves each asset matches it.
fn main() {
    // 4.11.11: FIRST, before any thread exists. A panic on the engine thread
    // is otherwise reported only on stderr, which the tournament harness
    // drains asynchronously and loses to a fast abort -- the reason the
    // 2026-09-04 EngineCrash could not be diagnosed. See `crash_report`.
    crash_report::install_stdout_reporter();
    request_fine_grained_scheduling();

    println!(
        "{} {} by {}",
        capitalize_first_letter(env!("CARGO_PKG_NAME")),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    // A.4.2: say so when this CPU would be better served by a different asset.
    // Silent when the choice is already right, which is the common case.
    if let Some(advice) = cpu_advice::startup_advice() {
        println!("{advice}");
    }

    let commands = EngineCommandQueue::default();
    let control = Arc::new(EngineControl::default());
    let engine_commands = commands.clone();
    let engine_control = Arc::clone(&control);
    let engine_thread = thread::Builder::new()
        .name("rarog-engine".to_string())
        .stack_size(ENGINE_THREAD_STACK_SIZE)
        // Construct the Engine (which owns the large inline-array Searcher)
        // *inside* this 16 MB thread, not on the caller's stack. In debug builds
        // the default 1 MB Windows main-thread stack overflows while building the
        // Searcher (no copy elision); doing it here keeps the big frames on the
        // large stack the search already runs on. Zero search impact.
        .spawn(move || {
            let mut engine = Engine::new(engine_commands, engine_control);
            engine.start();
        })
        .expect("Engine thread failed to start.");

    // A.4.5: arguments are commands, run through the very same dispatch stdin
    // uses. Before this, `main` ignored `std::env::args()` entirely, so
    // `rarog.exe bench 13` printed the banner, hit EOF, benched nothing and
    // exited 0 — a silent no-op with a success code, which is the one outcome
    // a command-line tool must never produce.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut protocol = UciProtocol::new(commands, control);
    let outcome = if args.is_empty() {
        // No arguments: a GUI's path, byte-identical to before.
        protocol.uci_loop();
        CommandOutcome::Handled
    } else {
        protocol.run_once(&args.join(" "))
    };
    engine_thread.join().expect("Engine thread failed.");

    if outcome == CommandOutcome::Unknown {
        // Non-zero so a script can tell a typo from a result. The message came
        // from `unknown_command`, which has already printed and flushed.
        process::exit(2);
    }
}

/// Ask Windows for 1 ms scheduling granularity.
///
/// Written during the 8.13(e) time-forfeit hunt, and kept with an honest
/// scope note: this did NOT fix the forfeits (measured — the ~35 ms stalls
/// are scheduler starvation under multi-thread contention, addressed by the
/// SMP time reserve in `time_manager.rs`). What it does buy: the 1 ms
/// `thread::sleep` in the ponder/infinite wait loop actually sleeps ~1 ms
/// instead of a 15.6 ms tick, so `ponderhit`/`stop` are picked up promptly,
/// and short timed waits across the engine stop being tick-quantised.
///
/// `timeBeginPeriod(1)` lasts for the lifetime of the process and Windows
/// reverts it at exit, so no paired `timeEndPeriod` is needed.
#[cfg(windows)]
fn request_fine_grained_scheduling() {
    #[link(name = "winmm")]
    unsafe extern "system" {
        fn timeBeginPeriod(u_period: u32) -> u32;
    }
    // SAFETY: plain FFI call with no pointers or state; the only effect is a
    // process-scoped timer-resolution request the OS reverts at exit.
    unsafe {
        timeBeginPeriod(1);
    }
}

#[cfg(not(windows))]
fn request_fine_grained_scheduling() {}
