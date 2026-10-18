//! Large pages for the engine's big tables, where Windows grants them.
//!
//! A search reads its transposition table, correction tables and
//! continuation histories at random: about 100 MB at the default `Hash`,
//! against a second-level TLB that maps 8 MB of 4 KB pages, so nearly every
//! such read also waits for a page walk. [`PageAllocator`] backs every
//! allocation of at least [`THRESHOLD`] bytes with 2 MB pages when the user
//! holds the "Lock pages in memory" right, and with ordinary pages otherwise;
//! smaller allocations go to the system allocator. The tables, their contents
//! and the search are unchanged.
//!
//! The trait's default `alloc_zeroed` and `realloc` are kept: a page
//! allocation comes back zeroed from the system already, so the default's
//! clearing is wasted work at start-up only, and overriding it would be one
//! more `unsafe` site for no speed in play.

use std::fmt;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

/// Allocations of at least this many bytes are page allocations: the
/// transposition table, the correction and continuation tables, and with the
/// threshold below one large page also the pawn history (786 KB) and the pawn
/// cache (1.3 MB), two more tables a search reads at random. A page
/// allocation is rounded up to whole large pages, so each of those two costs
/// at most 2 MB of locked memory per search thread.
pub const THRESHOLD: usize = 512 * 1024;

/// What the process allocator has done with its page allocations so far.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LargePages {
    /// No page allocation has been made, or the allocator is not installed.
    NotUsed,
    /// The process may not use large pages: ordinary pages back the tables.
    Off,
    /// Every page allocation so far is in large pages.
    On,
    /// The right is held, but some page allocation fell back to ordinary
    /// pages because the system found no contiguous memory for it.
    Partial,
}

impl fmt::Display for LargePages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotUsed => "not used",
            Self::Off => "off",
            Self::On => "on",
            Self::Partial => "partial",
        })
    }
}

const UNASKED: u8 = 0;
const DENIED: u8 = 1;
const GRANTED: u8 = 2;

/// Whether the process may use large pages, decided at the first page
/// allocation.
static RIGHT: AtomicU8 = AtomicU8::new(UNASKED);
/// Page allocations that asked for large pages and got ordinary ones.
static FALLBACKS: AtomicUsize = AtomicUsize::new(0);

/// How the big tables are backed so far.
pub fn large_pages() -> LargePages {
    match RIGHT.load(Ordering::Acquire) {
        UNASKED => LargePages::NotUsed,
        DENIED => LargePages::Off,
        _ if FALLBACKS.load(Ordering::Relaxed) == 0 => LargePages::On,
        _ => LargePages::Partial,
    }
}

/// The process allocator on Windows: page allocations for the big tables,
/// the system allocator for everything else (see the module comment).
pub struct PageAllocator;

#[cfg(windows)]
mod win {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{DENIED, FALLBACKS, GRANTED, PageAllocator, RIGHT, THRESHOLD, UNASKED};
    use crate::win32::{
        AdjustTokenPrivileges, CloseHandle, GetCurrentProcess, GetLargePageMinimum, GetLastError,
        Handle, LookupPrivilegeValueW, Luid, MEM_COMMIT, MEM_LARGE_PAGES, MEM_RELEASE, MEM_RESERVE,
        OpenProcessToken, PAGE_READWRITE, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
        TOKEN_QUERY, TokenPrivileges, VirtualAlloc, VirtualFree,
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
    fn large_page_size() -> Option<usize> {
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
                &mut token,
            ) != 0
            {
                let mut luid = Luid { low: 0, high: 0 };
                if LookupPrivilegeValueW(std::ptr::null(), LOCK_MEMORY.as_ptr(), &mut luid) != 0 {
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
                        &privileges,
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

    /// Whether `layout` is a page allocation. A region starts on a 64 KB
    /// boundary, which covers every alignment the engine asks for; a larger
    /// one stays with the system allocator.
    fn is_page_allocation(layout: Layout) -> bool {
        layout.size() >= THRESHOLD && layout.align() <= 65_536
    }

    const _: () = assert!(UNASKED == 0, "a fresh static reads as unasked");

    // SAFETY: a page allocation is a fresh `VirtualAlloc` region, aligned to
    // at least 64 KB, and only a page allocation is released with
    // `VirtualFree`: both directions decide by the same layout test, and every
    // other layout goes to the system allocator unchanged. The trait's
    // default `alloc_zeroed` and `realloc` are built from these two, and a
    // `realloc` across the threshold allocates by the new layout, copies and
    // releases by the old one, so each side is routed by its own layout.
    unsafe impl GlobalAlloc for PageAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            // SAFETY: a small layout is forwarded unchanged; a page
            // allocation asks for a new region at no particular address, and
            // each call returns null on failure. The large-page size is a
            // positive multiple of the ordinary page, so the rounded size is
            // at least the layout's.
            unsafe {
                if !is_page_allocation(layout) {
                    return System.alloc(layout);
                }
                let size = layout.size();
                if let Some(page) = large_page_size() {
                    let large = VirtualAlloc(
                        std::ptr::null_mut(),
                        size.div_ceil(page) * page,
                        MEM_RESERVE | MEM_COMMIT | MEM_LARGE_PAGES,
                        PAGE_READWRITE,
                    );
                    if !large.is_null() {
                        return large.cast();
                    }
                    // No contiguous memory for large pages right now: the
                    // table still works in ordinary pages, and `large_pages`
                    // reports the mix.
                    FALLBACKS.fetch_add(1, Ordering::Relaxed);
                }
                VirtualAlloc(
                    std::ptr::null_mut(),
                    size,
                    MEM_RESERVE | MEM_COMMIT,
                    PAGE_READWRITE,
                )
                .cast()
            }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            // SAFETY: `ptr` came from `alloc` with this layout, so the same
            // test names the allocator that made it; a page allocation's
            // pointer is its region's base, which `MEM_RELEASE` requires.
            unsafe {
                if is_page_allocation(layout) {
                    VirtualFree(ptr.cast(), 0, MEM_RELEASE);
                } else {
                    System.dealloc(ptr, layout);
                }
            }
        }
    }
}
