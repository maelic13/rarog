//! The process allocator's page allocations (`src/large_pages.rs`): a region
//! at the threshold comes back zeroed, aligned to at least 64 KB and writable
//! to its last byte, and is released; a small layout is served by the system
//! allocator; a reallocation across the threshold keeps its bytes in both
//! directions. The tests pass with or without the "Lock pages in memory"
//! right, since the allocator falls back to ordinary pages, and they report
//! which backing they saw.

#![cfg(windows)]

use std::alloc::{GlobalAlloc, Layout};

use rarog::large_pages::{LargePages, PageAllocator, THRESHOLD, large_pages};

#[test]
fn a_page_allocation_is_zeroed_aligned_writable_and_released() {
    let layout = Layout::from_size_align(THRESHOLD + 4_096, 64).expect("valid layout");
    // SAFETY: a fresh allocation of `layout`, read and written only within
    // its size, and released once with the same layout.
    unsafe {
        let region = PageAllocator.alloc(layout);
        assert!(!region.is_null(), "a page allocation is served");
        assert_eq!(
            region as usize % 65_536,
            0,
            "a region starts on a 64 KB boundary"
        );
        let bytes = std::slice::from_raw_parts_mut(region, layout.size());
        assert!(bytes.iter().all(|&b| b == 0), "a fresh region is zeroed");
        bytes[0] = 1;
        bytes[layout.size() - 1] = 2;
        assert_eq!(bytes[0] + bytes[layout.size() - 1], 3);
        PageAllocator.dealloc(region, layout);
    }
    let backing = large_pages();
    assert_ne!(
        backing,
        LargePages::NotUsed,
        "the first page allocation decides the right"
    );
    println!("large pages: {backing}");
}

#[test]
fn a_small_allocation_is_served_and_released() {
    let layout = Layout::from_size_align(64, 8).expect("valid layout");
    // SAFETY: a fresh allocation of `layout`, written within its size and
    // released once with the same layout.
    unsafe {
        let block = PageAllocator.alloc(layout);
        assert!(!block.is_null());
        block.write_bytes(0xA5, layout.size());
        assert_eq!(*block.add(layout.size() - 1), 0xA5);
        PageAllocator.dealloc(block, layout);
    }
}

#[test]
fn a_reallocation_across_the_threshold_keeps_its_bytes() {
    let small = Layout::from_size_align(THRESHOLD / 2, 8).expect("valid layout");
    // SAFETY: each pointer is used only within the size it was allocated or
    // reallocated to, and the final block is released with its own layout.
    unsafe {
        let block = PageAllocator.alloc(small);
        assert!(!block.is_null());
        for offset in 0..small.size() {
            *block.add(offset) = u8::try_from(offset % 251).expect("below 251");
        }
        let grown = PageAllocator.realloc(block, small, THRESHOLD + 1);
        assert!(!grown.is_null(), "growing across the threshold is served");
        for offset in (0..small.size()).step_by(997) {
            assert_eq!(
                *grown.add(offset),
                u8::try_from(offset % 251).expect("below 251"),
                "byte {offset} survives growing"
            );
        }
        let large = Layout::from_size_align(THRESHOLD + 1, 8).expect("valid layout");
        let shrunk = PageAllocator.realloc(grown, large, small.size());
        assert!(
            !shrunk.is_null(),
            "shrinking across the threshold is served"
        );
        for offset in (0..small.size()).step_by(997) {
            assert_eq!(
                *shrunk.add(offset),
                u8::try_from(offset % 251).expect("below 251"),
                "byte {offset} survives shrinking"
            );
        }
        PageAllocator.dealloc(shrunk, small);
    }
}
