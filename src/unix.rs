//! The C library calls the engine makes on Unix, declared once: the
//! physical-memory query and, on Linux, the huge-page advice for the page
//! allocations. One foreign block holds every such declaration; each caller
//! of an `unsafe` one holds the block that justifies its call, and a function
//! sound for every argument is declared `safe`.

#[cfg(target_os = "linux")]
use std::ffi::c_void;
use std::ffi::{c_int, c_long};

// SAFETY: the signatures are POSIX's. `sysconf` reads one configuration
// value by its number and returns -1 for a number it does not know, so every
// argument is sound and it is declared `safe`; it takes no pointer and keeps
// no state. `madvise` takes a range of the caller's memory and stays
// `unsafe`: some advice discards the range's contents.
unsafe extern "C" {
    pub(crate) safe fn sysconf(name: c_int) -> c_long;
    #[cfg(target_os = "linux")]
    pub(crate) fn madvise(addr: *mut c_void, length: usize, advice: c_int) -> c_int;
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) const PAGE_SIZE: c_int = 30;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) const PHYS_PAGES: c_int = 85;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) const PAGE_SIZE: c_int = 29;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) const PHYS_PAGES: c_int = 200;

/// Advice that the range may be backed by transparent huge pages. It changes
/// how the kernel may back the range, never what it holds. The value is the
/// kernel's generic one, which x86-64 and AArch64 use; alpha, MIPS, PA-RISC
/// and Xtensa number advice differently, and Rarog builds for none of them.
#[cfg(target_os = "linux")]
pub(crate) const MADV_HUGEPAGE: c_int = 14;
