/// Emits a UCI `info string` diagnostic.
///
/// The single choke point for protocol-layer notices (option-parse errors,
/// bench and WAC setup, diagnostics), so their destination can change without
/// touching the call sites. The search writes its notices through `InfoSink`.
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

// 64-bit only: the `u64 -> usize` hash-indexing conversions in the table and
// the evaluation caches are lossless only there (`infra::index`). The binary
// depends on the library, so this one guard covers both.
#[cfg(not(target_pointer_width = "64"))]
compile_error!("Rarog supports only 64-bit targets (u64 hash -> usize indexing relies on it).");

/// Build every process-wide lookup table. A table built on first use is built
/// inside the first search that needs it and charged to that search's clock,
/// so the engine builds them all before it reads a command.
pub fn initialize_tables() {
    std::sync::LazyLock::force(&board::ATTACKS);
    kpk::initialize();
}

/// The engine version as reported to the user: the package version, with
/// `+b2core` when the selectivity-core candidate is compiled in.
#[cfg(not(feature = "b2core"))]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The engine version as reported to the user: the package version, with
/// `+b2core` when the selectivity-core candidate is compiled in.
#[cfg(feature = "b2core")]
pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "+b2core");

pub mod bench;
pub mod board;
pub mod cpu_advice;
pub mod crash_report;
pub mod diag;
pub mod engine;
pub mod engine_command;
pub mod eval;
pub mod infra;
mod kpk;
pub mod search;
pub mod search_options;
pub mod syzygy;
pub mod tt;
pub mod uci_protocol;
pub mod wac;
