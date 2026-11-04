//! The universal x86-64 binary's entry point.
//!
//! The binary holds four complete engines, one per tier, each the ordinary
//! PGO build of that tier with the few global symbols it defines renamed
//! after the profile was applied (`cargo xtask build --universal`). This is
//! the only code outside them, and it runs first: compiled for the x86-64
//! baseline, it reads the CPU, picks the fastest tier the CPU runs (or the
//! one `RAROG_ISA` forces) and hands the process to that tier's own `main`,
//! which starts the engine exactly as the single-tier build does. The
//! engine's start-up line names the tier; `id name` does not change.
//!
//! It is not part of the engine crate: an ordinary build of a crate that
//! calls the tiers' entry points cannot link without them.

#[path = "../src/cpu_tier.rs"]
mod cpu_tier;

use std::ffi::{CString, c_char, c_int};
use std::process::ExitCode;

use cpu_tier::Tier;

/// rustc's C entry point of one tier, renamed from that tier object's `main`.
type TierMain = unsafe extern "C" fn(c_int, *const *const c_char) -> c_int;

unsafe extern "C" {
    fn rarog_base_main(argc: c_int, argv: *const *const c_char) -> c_int;
    fn rarog_sse42_main(argc: c_int, argv: *const *const c_char) -> c_int;
    fn rarog_avx2_main(argc: c_int, argv: *const *const c_char) -> c_int;
    fn rarog_pext_main(argc: c_int, argv: *const *const c_char) -> c_int;
}

fn main() -> ExitCode {
    let forced = std::env::var(cpu_tier::FORCE_VARIABLE).ok();
    let tier = match cpu_tier::universal_tier(
        cpu_tier::host_level(),
        cpu_tier::host_has_slow_pext(),
        forced.as_deref(),
    ) {
        Ok(tier) => tier,
        Err(refusal) => {
            // On stdout too, where a GUI shows engine output.
            println!("info string {refusal}");
            eprintln!("{refusal}");
            return ExitCode::FAILURE;
        }
    };
    let entry: TierMain = match tier {
        Tier::Base => rarog_base_main,
        Tier::Sse42 => rarog_sse42_main,
        Tier::Avx2 => rarog_avx2_main,
        Tier::Pext => rarog_pext_main,
    };

    // The C `argc`/`argv` a tier's entry expects: Unix's runtime reads them,
    // Windows' reads the command line itself.
    let args: Vec<CString> = std::env::args_os()
        .filter_map(|arg| CString::new(arg.as_encoded_bytes().to_vec()).ok())
        .collect();
    let mut argv: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
    argv.push(std::ptr::null());
    let argc = c_int::try_from(args.len()).unwrap_or(c_int::MAX);
    // SAFETY: `entry` is a tier's `main`, which a C runtime calls with exactly
    // this: `argc` NUL-terminated strings in `argv`, then a null pointer. The
    // strings and the array outlive the call, which runs the whole engine.
    let status = unsafe { entry(argc, argv.as_ptr()) };
    ExitCode::from(u8::try_from(status).unwrap_or(1))
}
