//! Route a panic onto the channel a tournament harness actually keeps.
//!
//! Written for PLAN 4.11.11, after Rarog 2.3.2 lost one game in ~5,200 to
//! `EngineCrash` in the 2026-09-04 rating tournament and **the cause could not
//! be established from anything that was retained**. The incident report
//! (`20260904-230039-002`) has the full UCI transcript, the clocks and the
//! position; it has no exit status and no stderr. Every other incident in that
//! run is equally mute, including five for a different engine, so the silence
//! is a property of the pipeline rather than evidence about the death.
//!
//! Rarog's own half of that gap is this module. Release builds set
//! `panic = "abort"`, and the default hook writes the panic to **stderr** —
//! which the harness drains on a background task that loses the race against a
//! fast abort. The UCI transcript, in contrast, is recorded synchronously by
//! the reader that then observes EOF, so anything Rarog puts on **stdout**
//! before dying is retained by construction.
//!
//! So the hook mirrors the panic to stdout as an `info string`, then chains to
//! the default hook so stderr and `RUST_BACKTRACE` keep working unchanged. A
//! GUI must ignore `info string` lines it does not understand, so this is
//! protocol-legal in the middle of a search.
//!
//! **Scope, stated honestly.** This makes a *panic* diagnosable. It cannot see
//! a death that never reaches the Rust runtime — an access violation, a
//! `STATUS_ILLEGAL_INSTRUCTION`, or a kill from outside. Distinguishing those
//! needs a structured-exception handler, which is a new FFI site against the
//! frozen unsafe floor (PLAN principle #8) and therefore a deliberate decision
//! rather than a detail; PLAN 4.11.11 records it as the open option. What this
//! buys is the ability to tell the two classes apart the next time: a report on
//! stdout means Rust panicked and names the line, and no report narrows the
//! next search to the ways a process dies without the runtime noticing.
//!
//! Two constraints shape the writer, and both cost real work elsewhere in this
//! repo when they were violated:
//!
//! * **No `println!`, no `info_string!`.** Both panic if the write fails, and a
//!   panic inside the panic hook aborts immediately with nothing printed —
//!   turning the instrument into the thing that hides the evidence. Every write
//!   here is a discarded `Result`.
//! * **One line, and a leading newline.** A panic can land halfway through an
//!   `info depth …` line, and the stdout lock is reentrant, so the report can
//!   interleave. The leading newline terminates whatever was in flight; the
//!   message has its own newlines folded so the report cannot be split into
//!   fragments a parser would read as separate UCI commands.

use std::io::{self, Write};
use std::panic::{self, PanicHookInfo};

/// Where a formatted report goes. A plain `fn` pointer, not a closure: the
/// hook must not own captured state that a panicking thread might already be
/// borrowing.
pub type ReportSink = fn(&str);

/// The `info string` a panic is reported as.
///
/// Pure, so it is tested directly rather than by inspecting process output.
/// Newlines and carriage returns in `message` fold to ` | ` — a payload
/// formatted over several lines (an `assert_eq!`, typically) would otherwise
/// emit fragments that a UCI parser reads as separate commands.
pub fn panic_line(thread: &str, message: &str, location: &str) -> String {
    format!(
        "info string PANIC thread={} at {}: {}",
        one_line(thread),
        one_line(location),
        one_line(message)
    )
}

fn one_line(text: &str) -> String {
    text.replace(['\n', '\r'], " | ")
}

/// Pull the reportable parts out of a hook's `PanicHookInfo`.
///
/// `payload` carries `&str` for a literal `panic!` and `String` for a
/// formatted one; anything else is a `panic_any` this crate never issues, and
/// is named rather than dropped so the report still says a panic happened.
fn describe(info: &PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    let message = payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "<non-string panic payload>".to_string());
    let location = info
        .location()
        .map_or_else(|| "<unknown location>".to_string(), ToString::to_string);
    let thread = std::thread::current()
        .name()
        .unwrap_or("<unnamed>")
        .to_string();
    panic_line(&thread, &message, &location)
}

/// Write one report to stdout, terminating any line already in flight.
fn stdout_sink(line: &str) {
    let mut out = io::stdout().lock();
    // Errors are dropped deliberately: the GUI having closed the pipe is a
    // normal way for a session to end, and this is the one place where a
    // failed write must never become a second panic.
    let _ = out.write_all(b"\n");
    let _ = out.write_all(line.as_bytes());
    let _ = out.write_all(b"\n");
    let _ = out.flush();
}

/// Install the reporter, mirroring every panic to `sink` before the hook that
/// was previously in place runs.
///
/// Public so the wire itself can be proved live in a test, per the standing
/// rule that a harness wire is not trusted until it has been shown to fire;
/// the binary calls [`install_stdout_reporter`].
pub fn install_with(sink: ReportSink) {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        sink(&describe(info));
        previous(info);
    }));
}

/// Install the reporter used by the shipped binary.
pub fn install_stdout_reporter() {
    install_with(stdout_sink);
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Mutex, MutexGuard, OnceLock};

    /// `set_hook` is process-global, so the two tests that install one must not
    /// overlap. They also leave a hook behind, which is why each restores the
    /// default before releasing the lock.
    fn hook_guard() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn captured() -> &'static Mutex<Vec<String>> {
        static CAPTURED: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
        CAPTURED.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn capture_sink(line: &str) {
        captured()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(line.to_string());
    }

    #[test]
    fn a_report_is_one_line_and_names_thread_location_and_message() {
        let line = panic_line(
            "rarog-engine",
            "assertion failed\n  left: 1\n right: 2",
            "src/search.rs:4242:9",
        );

        assert!(!line.contains('\n'), "report must be one line: {line}");
        assert!(line.starts_with("info string PANIC "), "{line}");
        assert!(line.contains("thread=rarog-engine"), "{line}");
        assert!(line.contains("at src/search.rs:4242:9"), "{line}");
        assert!(
            line.contains("assertion failed |   left: 1 |  right: 2"),
            "{line}"
        );
    }

    /// The wire, not the formatter: install the hook, panic for real, and
    /// require the report to arrive. Without this the module could be dead in
    /// exactly the way `--rset` was dead for two screens.
    #[test]
    fn the_installed_hook_fires_on_a_real_panic() {
        let _guard = hook_guard();
        captured()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();

        install_with(capture_sink);
        let result = panic::catch_unwind(|| panic!("deliberate probe {}", 7));
        let _ = panic::take_hook();

        assert!(result.is_err(), "the probe must actually panic");
        let lines = captured()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        assert_eq!(lines.len(), 1, "exactly one report per panic: {lines:?}");
        assert!(lines[0].contains("deliberate probe 7"), "{}", lines[0]);
        // Not `src/crash_report.rs` — `Location::file()` uses the platform's
        // separator, so the path is backslashed on Windows and the anchored
        // form passes only on the CI matrix's other half.
        assert!(lines[0].contains("crash_report.rs:"), "{}", lines[0]);
    }

    /// Chaining, not replacing: the default hook still runs, so stderr and
    /// `RUST_BACKTRACE` keep working. Proved by installing twice and requiring
    /// both sinks to see the same panic.
    #[test]
    fn the_previous_hook_still_runs_after_ours() {
        let _guard = hook_guard();
        captured()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();

        install_with(capture_sink);
        install_with(capture_sink);
        let result = panic::catch_unwind(|| panic!("chained probe"));
        let _ = panic::take_hook();
        let _ = panic::take_hook();

        assert!(result.is_err());
        let lines = captured()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        assert_eq!(lines.len(), 2, "both hooks must fire: {lines:?}");
    }
}
