//! The machine's physical memory: the most a `Hash` can be. A platform that
//! reserves address space lazily (macOS does) grants a table larger than the
//! machine, and filling it then never ends, so the engine bounds `Hash` by
//! this figure itself rather than by what the allocator accepts.

use std::sync::OnceLock;

/// Total physical memory in bytes, read once; `None` where the platform does
/// not report it.
pub fn total_bytes() -> Option<u64> {
    static TOTAL: OnceLock<Option<u64>> = OnceLock::new();
    *TOTAL.get_or_init(query)
}

#[cfg(windows)]
fn query() -> Option<u64> {
    let mut status = crate::win32::MemoryStatusEx {
        length: u32::try_from(size_of::<crate::win32::MemoryStatusEx>()).ok()?,
        ..crate::win32::MemoryStatusEx::default()
    };
    (crate::win32::GlobalMemoryStatusEx(&mut status) != 0).then_some(status.total_phys)
}

#[cfg(all(
    unix,
    any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    )
))]
fn query() -> Option<u64> {
    let pages = u64::try_from(unix::sysconf(unix::PHYS_PAGES)).ok()?;
    let page_size = u64::try_from(unix::sysconf(unix::PAGE_SIZE)).ok()?;
    pages.checked_mul(page_size).filter(|&bytes| bytes > 0)
}

#[cfg(not(any(
    windows,
    all(
        unix,
        any(
            target_os = "linux",
            target_os = "android",
            target_os = "macos",
            target_os = "ios"
        )
    )
)))]
fn query() -> Option<u64> {
    None
}

#[cfg(all(
    unix,
    any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    )
))]
mod unix {
    use std::ffi::{c_int, c_long};

    // SAFETY: `sysconf` reads one configuration value by its number and
    // returns -1 for a number it does not know, so every argument is sound
    // and it is declared `safe`; it takes no pointer and keeps no state.
    unsafe extern "C" {
        pub safe fn sysconf(name: c_int) -> c_long;
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub const PAGE_SIZE: c_int = 30;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub const PHYS_PAGES: c_int = 85;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub const PAGE_SIZE: c_int = 29;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub const PHYS_PAGES: c_int = 200;
}

#[cfg(test)]
mod tests {
    /// The supported platforms report a figure, and it is a plausible one.
    #[test]
    #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
    fn the_supported_platforms_report_their_physical_memory() {
        let bytes = super::total_bytes().expect("the platform reports its physical memory");
        assert!(bytes >= 256 << 20, "{bytes} bytes");
        assert!(bytes < 1 << 50, "{bytes} bytes");
    }
}
