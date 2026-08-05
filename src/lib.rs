/// Emits a UCI `info string` diagnostic.
///
/// 9.0a: the single choke point for engine-side diagnostics (option-parse
/// errors, tablebase status, search notices). Previously ~15 bare `println!`
/// calls were scattered through search and option parsing, which made engine
/// output untestable and uncontrollable — a GUI received `info string` lines
/// emitted from deep inside a parser. Routing them through one macro means the
/// destination can change (suppressed under test, mirrored to a log, gated by
/// a verbosity level) without touching the call sites.
///
/// UCI *protocol* output (`bestmove`, `info depth …`) deliberately stays in
/// the protocol layer, and the `bench`/`wac` console reports stay plain
/// `println!` — those are human-facing CLI output, not engine diagnostics.
#[macro_export]
macro_rules! info_string {
    ($($arg:tt)*) => {
        println!("info string {}", format_args!($($arg)*))
    };
}

// 9.0b: 64-bit only — see the matching guard in main.rs.
#[cfg(not(target_pointer_width = "64"))]
compile_error!("Rarog supports only 64-bit targets (u64 hash -> usize indexing relies on it).");

pub mod bench;
pub mod board;
pub mod diag;
pub mod engine;
pub mod engine_command;
pub mod eval;
pub mod evidence;
pub mod infra;
mod kpk;
mod move_ordering;
pub mod params;
pub mod search;
pub mod search_options;
mod search_threads;
pub mod syzygy;
mod time_manager;
pub mod tt;
pub mod uci_protocol;
pub mod wac;
