//! Large pages for the engine's big tables, on Windows and Linux.
//!
//! A search reads its transposition table, correction tables and
//! continuation histories at random: about 100 MB at the default `Hash`,
//! against a second-level TLB that maps 8 MB of 4 KB pages, so nearly every
//! such read also waits for a page walk. [`PageAllocator`] serves every
//! allocation of at least [`THRESHOLD`] bytes as a page allocation and
//! smaller ones from the system allocator. On Windows a page allocation is
//! backed by 2 MB pages when the user holds the "Lock pages in memory" right,
//! and by ordinary pages otherwise. On Linux it is 2 MB aligned and advised
//! for transparent huge pages, which lock nothing and need no privilege: the
//! kernel backs it in 2 MB pages as its mode allows and as it finds the
//! memory. The tables, their contents and the search are unchanged. The
//! `LargePages` option turns the large pages off, for a system where they
//! hurt more than page walks do.
//!
//! The trait's default `alloc_zeroed` and `realloc` are kept: the default
//! clears a page allocation the system may already have zeroed, wasted work
//! at start-up only, and overriding it would be one more `unsafe` site for
//! no speed in play.

use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

/// Allocations of at least this many bytes are page allocations: the
/// transposition table, the correction and continuation tables, and with the
/// threshold below one large page also the pawn history (786 KB) and the pawn
/// cache (1.3 MB), two more tables a search reads at random. A page
/// allocation is rounded up to whole large pages, so each of those two costs
/// at most 2 MB per search thread.
pub const THRESHOLD: usize = 512 * 1024;

/// What the process allocator has done with its page allocations so far.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LargePages {
    /// No page allocation has been made, or the allocator is not installed.
    NotUsed,
    /// The `LargePages` option is off: the tables are not given large pages.
    Disabled,
    /// The process may not use large pages: on Windows the right is not
    /// held, on Linux the kernel's transparent huge pages are `never` or
    /// absent. Ordinary pages back the tables.
    Off,
    /// Every page allocation so far is in large pages. On Linux: advised for
    /// them under a mode that allows them, which the kernel serves as it
    /// finds the memory.
    On,
    /// Large pages are allowed, but some page allocation fell back to
    /// ordinary pages: on Windows the system found no contiguous memory for
    /// it, on Linux the kernel refused its advice.
    Partial,
}

impl fmt::Display for LargePages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotUsed => "not used",
            Self::Disabled => "disabled",
            Self::Off => "off",
            Self::On => "on",
            Self::Partial => "partial",
        })
    }
}

const UNASKED: u8 = 0;
const DENIED: u8 = 1;
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    expect(dead_code, reason = "only Windows and Linux ask for large pages")
)]
const GRANTED: u8 = 2;

/// Whether the process may use large pages, decided at the first page
/// allocation: on Windows by the right, on Linux by whether the kernel takes
/// the advice.
static RIGHT: AtomicU8 = AtomicU8::new(UNASKED);
/// Page allocations that asked for large pages and got ordinary ones.
static FALLBACKS: AtomicUsize = AtomicUsize::new(0);
/// Whether page allocations ask for large pages: the `LargePages` option,
/// for the whole process. The engine thread changes it only between searches
/// and then allocates every table it governs again.
static ENABLED: AtomicBool = AtomicBool::new(true);

/// Turn large pages on or off for the page allocations that follow; the
/// tables already allocated keep their backing until they are allocated
/// again.
pub(crate) fn set_enabled(enabled: bool) {
    if ENABLED.swap(enabled, Ordering::Relaxed) != enabled {
        // What `large_pages` reports describes the tables allocated after
        // the change, not a mix with the ones the change replaces.
        FALLBACKS.store(0, Ordering::Relaxed);
    }
}

/// How the big tables are backed so far. On Linux the kernel's mode is read
/// at each call, so a mode set to `never` since reads as off.
pub fn large_pages() -> LargePages {
    if !ENABLED.load(Ordering::Relaxed) {
        return LargePages::Disabled;
    }
    match RIGHT.load(Ordering::Acquire) {
        UNASKED => LargePages::NotUsed,
        DENIED => LargePages::Off,
        #[cfg(target_os = "linux")]
        _ if !linux::mode_allows() => LargePages::Off,
        _ if FALLBACKS.load(Ordering::Relaxed) == 0 => LargePages::On,
        _ => LargePages::Partial,
    }
}

/// Whether the transparent-huge-page mode the kernel reports (the bracketed
/// word of `always [madvise] never`) lets an advised region have huge pages.
#[cfg(any(target_os = "linux", test))]
fn mode_allows_advised(report: &str) -> bool {
    let selected = report
        .split_whitespace()
        .find_map(|word| word.strip_prefix('[')?.strip_suffix(']'));
    matches!(selected, Some("always" | "madvise"))
}

/// The process allocator on Windows and Linux: page allocations for the big
/// tables, the system allocator for everything else (see the module comment).
pub struct PageAllocator;

#[cfg(windows)]
use win as pages;

#[cfg(target_os = "linux")]
use linux as pages;

#[cfg(windows)]
mod win {
    use std::alloc::Layout;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{DENIED, GRANTED, RIGHT, THRESHOLD};
    use crate::win32::{
        AdjustTokenPrivileges, CloseHandle, GetCurrentProcess, GetLargePageMinimum, GetLastError,
        Handle, LookupPrivilegeValueW, Luid, OpenProcessToken, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, TokenPrivileges,
    };

    /// "SeLockMemoryPrivilege", NUL-terminated UTF-16.
    const LOCK_MEMORY: [u16; 22] = {
        let name = b"SeLockMemoryPrivilege";
        let mut wide = [0u16; 22];
        let mut i = 0;
        while i < name.len() {
            wide[i] = name[i] as u16;
            i += 1;
        }
        wide
    };

    /// The large-page size, valid once `RIGHT` is `GRANTED`.
    static PAGE: AtomicUsize = AtomicUsize::new(0);

    /// The large-page size when the process may use large pages. The first
    /// call enables the right in the process token and remembers the answer;
    /// two first callers racing both ask, and agree.
    pub(super) fn large_page_size() -> Option<usize> {
        match RIGHT.load(Ordering::Acquire) {
            GRANTED => return Some(PAGE.load(Ordering::Relaxed)),
            DENIED => return None,
            _ => {}
        }
        // SAFETY: Win32 calls with pointers to locals that outlive them; the
        // token handle is closed on the path that opened it; the privilege
        // name is NUL-terminated.
        let page = unsafe {
            let mut token: Handle = std::ptr::null_mut();
            let mut granted = false;
            if OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &raw mut token,
            ) != 0
            {
                let mut luid = Luid { low: 0, high: 0 };
                if LookupPrivilegeValueW(std::ptr::null(), LOCK_MEMORY.as_ptr(), &raw mut luid) != 0
                {
                    let privileges = TokenPrivileges {
                        count: 1,
                        luid,
                        attributes: SE_PRIVILEGE_ENABLED,
                    };
                    // The call succeeds without the right and then reports
                    // ERROR_NOT_ALL_ASSIGNED in the last error; only
                    // ERROR_SUCCESS means the right is enabled.
                    granted = AdjustTokenPrivileges(
                        token,
                        0,
                        &raw const privileges,
                        0,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    ) != 0
                        && GetLastError() == 0;
                }
                CloseHandle(token);
            }
            // A system without large pages reports a minimum of zero.
            if granted { GetLargePageMinimum() } else { 0 }
        };
        if page == 0 {
            RIGHT.store(DENIED, Ordering::Release);
            None
        } else {
            PAGE.store(page, Ordering::Relaxed);
            RIGHT.store(GRANTED, Ordering::Release);
            Some(page)
        }
    }

    /// The region a page allocation of `layout` takes, `None` for a layout
    /// the system allocator serves. A region starts on a 64 KB boundary,
    /// which covers every alignment the engine asks for; a larger one stays
    /// with the system allocator. `VirtualAlloc` rounds the size itself.
    pub(super) fn region(layout: Layout) -> Option<Layout> {
        (layout.size() >= THRESHOLD && layout.align() <= 65_536).then_some(layout)
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::alloc::Layout;
    use std::sync::atomic::Ordering;

    use super::{DENIED, FALLBACKS, GRANTED, RIGHT, THRESHOLD, UNASKED};

    /// The transparent huge page of a kernel on 4 KB base pages. On a larger
    /// base page the alignment is still sound, and the kernel's own huge page
    /// size decides what the advice gets.
    const HUGE_PAGE: usize = 2 << 20;

    /// Where the kernel reports its transparent-huge-page mode.
    const MODE: &str = "/sys/kernel/mm/transparent_hugepage/enabled";

    /// The region a page allocation of `layout` takes, `None` for a layout
    /// the system allocator serves: whole huge pages, aligned to one, so
    /// that the kernel can back every part of the table with them.
    pub(super) fn region(layout: Layout) -> Option<Layout> {
        if layout.size() < THRESHOLD || layout.align() > HUGE_PAGE {
            return None;
        }
        let size = layout.size().checked_next_multiple_of(HUGE_PAGE)?;
        Layout::from_size_align(size, HUGE_PAGE).ok()
    }

    /// Record whether the kernel took the advice for one page allocation:
    /// the first decides whether the process may use large pages, and a
    /// later refusal is a fallback.
    pub(super) fn record_advice(taken: bool) {
        let decided = if taken { GRANTED } else { DENIED };
        if RIGHT
            .compare_exchange(UNASKED, decided, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
            && !taken
        {
            FALLBACKS.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Whether the kernel's mode lets an advised region have huge pages; an
    /// unreadable report reads as no.
    pub(super) fn mode_allows() -> bool {
        std::fs::read_to_string(MODE).is_ok_and(|report| super::mode_allows_advised(&report))
    }
}

#[cfg(any(windows, target_os = "linux"))]
mod allocator {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::Ordering;

    #[cfg(windows)]
    use super::FALLBACKS;
    use super::{ENABLED, PageAllocator, pages};
    #[cfg(target_os = "linux")]
    use crate::unix::{MADV_HUGEPAGE, madvise};
    #[cfg(windows)]
    use crate::win32::{
        MEM_COMMIT, MEM_LARGE_PAGES, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAlloc,
        VirtualFree,
    };

    const _: () = assert!(super::UNASKED == 0, "a fresh static reads as unasked");

    // SAFETY: a page allocation is a fresh region from the system, and only
    // a page allocation is released as one: both directions decide by the
    // same test, `pages::region`, on the layout, and every other layout goes
    // to the system allocator unchanged. The trait's default `alloc_zeroed`
    // and `realloc` are built from these two, and a `realloc` across the
    // threshold allocates by the new layout, copies and releases by the old
    // one, so each side is routed by its own layout.
    unsafe impl GlobalAlloc for PageAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            // SAFETY: a small layout is forwarded unchanged, and each call
            // returns null on failure. On Windows a page allocation asks for
            // a new region at no particular address; the large-page size is a
            // positive multiple of the ordinary page, so the rounded size is
            // at least the layout's. On Linux the region is a valid layout at
            // least as large and as aligned as the one asked for, and the
            // advice covers exactly that fresh, page-aligned allocation and
            // changes only how the kernel backs it.
            unsafe {
                let Some(region) = pages::region(layout) else {
                    return System.alloc(layout);
                };
                #[cfg(windows)]
                let base = {
                    if ENABLED.load(Ordering::Relaxed)
                        && let Some(page) = pages::large_page_size()
                    {
                        let large = VirtualAlloc(
                            std::ptr::null_mut(),
                            region.size().div_ceil(page) * page,
                            MEM_RESERVE | MEM_COMMIT | MEM_LARGE_PAGES,
                            PAGE_READWRITE,
                        );
                        if !large.is_null() {
                            return large.cast();
                        }
                        // No contiguous memory for large pages right now:
                        // the table still works in ordinary pages, and
                        // `large_pages` reports the mix.
                        FALLBACKS.fetch_add(1, Ordering::Relaxed);
                    }
                    VirtualAlloc(
                        std::ptr::null_mut(),
                        region.size(),
                        MEM_RESERVE | MEM_COMMIT,
                        PAGE_READWRITE,
                    )
                    .cast()
                };
                #[cfg(target_os = "linux")]
                let base = System.alloc(region);
                #[cfg(target_os = "linux")]
                if !base.is_null() && ENABLED.load(Ordering::Relaxed) {
                    let taken = madvise(base.cast(), region.size(), MADV_HUGEPAGE) == 0;
                    pages::record_advice(taken);
                }
                base
            }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            // SAFETY: `ptr` came from `alloc` with this layout, so the same
            // test names the allocator that made it and the region it took.
            // On Windows a page allocation's pointer is its region's base,
            // which `MEM_RELEASE` requires; on Linux the region is the layout
            // it was allocated with.
            unsafe {
                match pages::region(layout) {
                    None => System.dealloc(ptr, layout),
                    #[cfg(windows)]
                    Some(_) => {
                        VirtualFree(ptr.cast(), 0, MEM_RELEASE);
                    }
                    #[cfg(target_os = "linux")]
                    Some(region) => System.dealloc(ptr, region),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mode_allows_advised;

    /// The selected mode is the bracketed word; `never`, a report with no
    /// selection and an empty one read as off.
    #[test]
    fn the_kernel_s_mode_is_its_bracketed_word() {
        assert!(mode_allows_advised("always [madvise] never\n"));
        assert!(mode_allows_advised("[always] madvise never\n"));
        assert!(!mode_allows_advised("always madvise [never]\n"));
        assert!(!mode_allows_advised("always madvise never\n"));
        assert!(!mode_allows_advised(""));
    }
}
