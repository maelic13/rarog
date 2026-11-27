//! The process allocator's page allocations (`src/large_pages.rs`): a region
//! at the threshold is aligned (to at least 64 KB on Windows, to a 2 MB huge
//! page on Linux) and writable to its last byte, and is released; on Windows
//! it comes back zeroed, and on Linux the kernel lists it as advised for
//! transparent huge pages. A small layout is served by the system allocator;
//! a reallocation across the threshold keeps its bytes in both directions.
//! The tests pass with or without large pages (the "Lock pages in memory"
//! right, the kernel's mode), since the allocator falls back to ordinary
//! pages, and they report which backing they saw.

#![cfg(any(windows, target_os = "linux"))]

use std::alloc::{GlobalAlloc, Layout};

use rarog::large_pages::{LargePages, PageAllocator, THRESHOLD, large_pages};

/// The boundary a page allocation's region starts on.
#[cfg(windows)]
const REGION_ALIGN: usize = 65_536;
#[cfg(target_os = "linux")]
const REGION_ALIGN: usize = 2 << 20;

#[test]
fn a_page_allocation_is_aligned_writable_and_released() {
    let layout = Layout::from_size_align(THRESHOLD + 4_096, 64).expect("valid layout");
    // SAFETY: a fresh allocation of `layout`, read and written only within
    // its size, and released once with the same layout.
    unsafe {
        let region = PageAllocator.alloc(layout);
        assert!(!region.is_null(), "a page allocation is served");
        assert_eq!(
            region as usize % REGION_ALIGN,
            0,
            "a region starts on its boundary"
        );
        let bytes = std::slice::from_raw_parts_mut(region, layout.size());
        #[cfg(windows)]
        assert!(
            bytes.iter().all(|&b| b == 0),
            "a fresh region is zeroed by the system"
        );
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

/// The advice reaches the kernel: the region's mapping carries the `hg`
/// flag, which `madvise(MADV_HUGEPAGE)` sets whatever the mode. A kernel
/// built without transparent huge pages has no mode to report and refuses
/// the advice, and there the allocation still works in ordinary pages.
#[cfg(target_os = "linux")]
#[test]
fn a_page_allocation_is_advised_for_huge_pages() {
    let layout = Layout::from_size_align(THRESHOLD, 64).expect("valid layout");
    let kernel_has_them =
        std::path::Path::new("/sys/kernel/mm/transparent_hugepage/enabled").exists();
    // SAFETY: a fresh allocation of `layout`, read only within its size, and
    // released once with the same layout.
    unsafe {
        let region = PageAllocator.alloc_zeroed(layout);
        assert!(!region.is_null(), "a page allocation is served");
        let bytes = std::slice::from_raw_parts(region, layout.size());
        assert!(bytes.iter().all(|&b| b == 0), "a zeroed region is zeroed");
        let flags = vm_flags(region as usize).expect("the region is mapped");
        assert_eq!(
            flags.split_whitespace().any(|flag| flag == "hg"),
            kernel_has_them,
            "flags {flags}"
        );
        PageAllocator.dealloc(region, layout);
    }
    println!("large pages: {}", large_pages());
}

/// The `VmFlags` of the mapping that holds `address`, from this process's
/// `smaps`.
#[cfg(target_os = "linux")]
fn vm_flags(address: usize) -> Option<String> {
    let smaps = std::fs::read_to_string("/proc/self/smaps").ok()?;
    let mut inside = false;
    for line in smaps.lines() {
        // A mapping's first line starts with its range, `start-end`, in hex.
        let range = line
            .split_once(' ')
            .and_then(|(range, _)| range.split_once('-'))
            .and_then(|(start, end)| {
                Some(usize::from_str_radix(start, 16).ok()?..usize::from_str_radix(end, 16).ok()?)
            });
        if let Some(range) = range {
            inside = range.contains(&address);
        } else if inside && let Some(flags) = line.strip_prefix("VmFlags:") {
            return Some(flags.trim().to_owned());
        }
    }
    None
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
