//! A global allocator that counts every allocation and reallocation and
//! forwards each call to `System`. A test crate installs it with its own
//! `#[global_allocator]` and reads `allocations()` before and after the work
//! it watches; the count is process-wide, so such a crate runs one watching
//! test at a time.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);

/// Allocations and reallocations since the process started.
pub fn allocations() -> u64 {
    ALLOCATIONS.load(Ordering::SeqCst)
}

pub struct CountingAllocator;

// SAFETY: every method forwards to `System` with the caller's own arguments,
// so the `GlobalAlloc` contract is `System`'s; counting touches no memory the
// allocator hands out.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the caller upholds `alloc`'s contract for `layout`.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the caller upholds `alloc_zeroed`'s contract for `layout`.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: `ptr` came from this allocator, which is `System`, with
        // `layout`; the caller upholds the rest of `realloc`'s contract.
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from this allocator, which is `System`, with
        // `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }
}
