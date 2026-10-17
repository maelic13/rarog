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

use std::sync::atomic::{AtomicUsize, Ordering};

/// Allocations of at least this many bytes are page allocations.
pub const THRESHOLD: usize = 2 * 1024 * 1024;

/// The large-page size once the first page allocation has asked: 0 before,
/// 1 when the process may not use large pages.
static LARGE_PAGE: AtomicUsize = AtomicUsize::new(0);

/// Whether the big tables are in large pages; `None` before the first page
/// allocation and wherever the allocator is not installed.
pub fn large_pages_in_use() -> Option<bool> {
    match LARGE_PAGE.load(Ordering::Relaxed) {
        0 => None,
        1 => Some(false),
        _ => Some(true),
    }
}

/// The process allocator on Windows: page allocations for the big tables,
/// the system allocator for everything else (see the module comment).
pub struct PageAllocator;

#[cfg(windows)]
mod win {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::ffi::c_void;
    use std::sync::atomic::Ordering;

    use super::{LARGE_PAGE, PageAllocator, THRESHOLD};

    type Handle = *mut c_void;

    #[repr(C)]
    struct Luid {
        low: u32,
        high: i32,
    }

    /// `TOKEN_PRIVILEGES` with its one `LUID_AND_ATTRIBUTES` inline.
    #[repr(C)]
    struct TokenPrivileges {
        count: u32,
        luid: Luid,
        attributes: u32,
    }

    const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
    const TOKEN_QUERY: u32 = 0x0008;
    const SE_PRIVILEGE_ENABLED: u32 = 0x0002;
    const MEM_COMMIT: u32 = 0x1000;
    const MEM_RESERVE: u32 = 0x2000;
    const MEM_RELEASE: u32 = 0x8000;
    const MEM_LARGE_PAGES: u32 = 0x2000_0000;
    const PAGE_READWRITE: u32 = 0x04;

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

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> Handle;
        fn CloseHandle(handle: Handle) -> i32;
        fn GetLastError() -> u32;
        fn GetLargePageMinimum() -> usize;
        fn VirtualAlloc(address: *mut c_void, size: usize, kind: u32, protect: u32) -> *mut c_void;
        fn VirtualFree(address: *mut c_void, size: usize, kind: u32) -> i32;
    }

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn OpenProcessToken(process: Handle, access: u32, token: *mut Handle) -> i32;
        fn LookupPrivilegeValueW(system: *const u16, name: *const u16, luid: *mut Luid) -> i32;
        fn AdjustTokenPrivileges(
            token: Handle,
            disable_all: i32,
            new_state: *const TokenPrivileges,
            length: u32,
            previous: *mut TokenPrivileges,
            return_length: *mut u32,
        ) -> i32;
    }

    /// The large-page size when the process may use large pages. The first
    /// call enables the right in the process token; a user without it gets
    /// `None` from then on.
    fn large_page_size() -> Option<usize> {
        let mut page = LARGE_PAGE.load(Ordering::Relaxed);
        if page == 0 {
            // SAFETY: Win32 calls with pointers to locals that outlive them;
            // the token handle is closed on the path that opened it.
            page = unsafe {
                let mut token: Handle = std::ptr::null_mut();
                let mut granted = false;
                if OpenProcessToken(
                    GetCurrentProcess(),
                    TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                    &mut token,
                ) != 0
                {
                    let mut luid = Luid { low: 0, high: 0 };
                    if LookupPrivilegeValueW(std::ptr::null(), LOCK_MEMORY.as_ptr(), &mut luid) != 0
                    {
                        let privileges = TokenPrivileges {
                            count: 1,
                            luid,
                            attributes: SE_PRIVILEGE_ENABLED,
                        };
                        // The call succeeds without the right, and then says
                        // so in the last error.
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
                if granted {
                    GetLargePageMinimum().max(1)
                } else {
                    1
                }
            };
            LARGE_PAGE.store(page, Ordering::Relaxed);
        }
        (page > 1).then_some(page)
    }

    /// Whether `layout` is a page allocation. A region starts on a 64 KB
    /// boundary, which covers every alignment the engine asks for; a larger
    /// one stays with the system allocator.
    fn is_page_allocation(layout: Layout) -> bool {
        layout.size() >= THRESHOLD && layout.align() <= 65_536
    }

    // SAFETY: a page allocation is a fresh `VirtualAlloc` region, aligned to
    // at least 64 KB, and only a page allocation is released with
    // `VirtualFree`: both directions decide by the same layout test, and every
    // other layout goes to the system allocator unchanged. The trait's
    // default `alloc_zeroed` and `realloc` are built from these two.
    unsafe impl GlobalAlloc for PageAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            // SAFETY: a small layout is forwarded unchanged; a page
            // allocation asks for a new region at no particular address, and
            // each call returns null on failure.
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
            // test names the allocator that made it.
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
