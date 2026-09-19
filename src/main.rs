use std::process;
use std::sync::Arc;
use std::thread;

use rarog::cpu_advice;
use rarog::crash_report;
use rarog::engine::Engine;
use rarog::engine_command::{EngineCommandQueue, EngineControl};
use rarog::infra::{THREAD_STACK_SIZE, capitalize_first_letter};
use rarog::uci_protocol::{CommandOutcome, UciProtocol};

// No startup CPU guard. A runtime check for a feature the build STATICALLY
// requires is `true` by construction: `std::is_x86_feature_detected!` expands
// to `cfg!(target_feature = "…") || runtime_detect(…)`, so in the PEXT tier
// (`-C target-feature=+bmi2`) it folds to `true` and the message is stripped.
// A guard that fires needs raw `CPUID` from a translation unit compiled at the
// baseline, which is a new FFI site against the frozen unsafe floor. Instead
// `README` states each asset's CPU requirement and `cargo xtask verify-isa`
// proves each asset matches it.
fn main() {
    // FIRST, before any thread exists. A panic on the engine thread
    // is otherwise reported only on stderr, which the tournament harness
    // drains asynchronously and loses to a fast abort -- the reason the
    // 2026-09-04 EngineCrash could not be diagnosed. See `crash_report`.
    crash_report::install_stdout_reporter();
    request_fine_grained_scheduling();

    println!(
        "{} {} by {}",
        capitalize_first_letter(env!("CARGO_PKG_NAME")),
        rarog::VERSION,
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    // Say so when this CPU would be better served by a different asset.
    // Silent when the choice is already right, which is the common case.
    if let Some(advice) = cpu_advice::startup_advice() {
        println!("{advice}");
    }

    // Before any input is read: `go` is timestamped as it is parsed, so a table
    // still being built when a search starts is charged to that search's clock.
    rarog::initialize_tables();

    let commands = EngineCommandQueue::default();
    let control = Arc::new(EngineControl::default());
    let engine_commands = commands.clone();
    let engine_control = Arc::clone(&control);
    let engine_thread = thread::Builder::new()
        .name("rarog-engine".to_string())
        .stack_size(THREAD_STACK_SIZE)
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

    // Arguments are commands, run through the very same dispatch stdin
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
/// This does not prevent time forfeits (the measured ~35 ms stalls are
/// scheduler starvation under multi-thread contention, which the SMP time
/// reserve in `search/time.rs` covers). What it does buy: the 1 ms
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
