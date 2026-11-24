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
/// require the report to arrive, so the module cannot be dead unnoticed.
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
    let lines = probe_reports("deliberate probe");
    assert_eq!(lines.len(), 1, "exactly one report per panic: {lines:?}");
    assert!(lines[0].contains("deliberate probe 7"), "{}", lines[0]);
    // The panic's own file as `Location::file()` writes it, the platform's
    // separator included, so the check holds wherever the test lives.
    let here = format!("{}:", file!());
    assert!(lines[0].contains(&here), "{}", lines[0]);
}

/// A helper thread's panic reaches the same report: the hook is the
/// process's, not the installing thread's, and the line names the helper.
#[test]
fn a_panic_on_a_helper_thread_is_reported_with_its_name() {
    let _guard = hook_guard();
    captured()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();

    install_with(capture_sink);
    let joined = std::thread::Builder::new()
        .name("rarog-search-1".to_string())
        .spawn(|| panic!("helper probe"))
        .expect("the probe thread starts")
        .join();
    let _ = panic::take_hook();

    assert!(joined.is_err(), "the probe thread must actually panic");
    let lines = probe_reports("helper probe");
    assert_eq!(lines.len(), 1, "exactly one report per panic: {lines:?}");
    assert!(lines[0].contains("thread=rarog-search-1"), "{}", lines[0]);
    assert!(lines[0].contains("helper probe"), "{}", lines[0]);
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
    let lines = probe_reports("chained probe");
    assert_eq!(lines.len(), 2, "both hooks must fire: {lines:?}");
}

/// The captured reports of one probe's panic. The hook is the process's,
/// so a test that panics on purpose on another thread meanwhile is
/// reported too, and counting every line would count it.
fn probe_reports(probe: &str) -> Vec<String> {
    captured()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .filter(|line| line.contains(probe))
        .cloned()
        .collect()
}
