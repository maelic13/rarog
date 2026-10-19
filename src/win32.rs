//! The Windows API the engine calls, declared once: the timer-resolution
//! request and the large-page allocation path. One `unsafe extern` block holds
//! every foreign declaration the crate makes on Windows; each caller holds the
//! `unsafe` block that justifies its call.

use std::ffi::c_void;

pub type Handle = *mut c_void;

#[repr(C)]
pub struct Luid {
    pub low: u32,
    pub high: i32,
}

/// `TOKEN_PRIVILEGES` with its one `LUID_AND_ATTRIBUTES` inline.
#[repr(C)]
pub struct TokenPrivileges {
    pub count: u32,
    pub luid: Luid,
    pub attributes: u32,
}

pub const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
pub const TOKEN_QUERY: u32 = 0x0008;
pub const SE_PRIVILEGE_ENABLED: u32 = 0x0002;
pub const MEM_COMMIT: u32 = 0x1000;
pub const MEM_RESERVE: u32 = 0x2000;
pub const MEM_RELEASE: u32 = 0x8000;
pub const MEM_LARGE_PAGES: u32 = 0x2000_0000;
pub const PAGE_READWRITE: u32 = 0x04;

#[link(name = "kernel32")]
#[link(name = "advapi32")]
#[link(name = "winmm")]
unsafe extern "system" {
    pub fn timeBeginPeriod(period: u32) -> u32;
    pub fn GetCurrentProcess() -> Handle;
    pub fn CloseHandle(handle: Handle) -> i32;
    pub fn GetLastError() -> u32;
    pub fn GetLargePageMinimum() -> usize;
    pub fn VirtualAlloc(address: *mut c_void, size: usize, kind: u32, protect: u32) -> *mut c_void;
    pub fn VirtualFree(address: *mut c_void, size: usize, kind: u32) -> i32;
    pub fn OpenProcessToken(process: Handle, access: u32, token: *mut Handle) -> i32;
    pub fn LookupPrivilegeValueW(system: *const u16, name: *const u16, luid: *mut Luid) -> i32;
    pub fn AdjustTokenPrivileges(
        token: Handle,
        disable_all: i32,
        new_state: *const TokenPrivileges,
        length: u32,
        previous: *mut TokenPrivileges,
        return_length: *mut u32,
    ) -> i32;
}
