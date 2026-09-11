fn main() {
    println!("cargo:rustc-check-cfg=cfg(rarog_pext)");

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // A.4.1 / RAR-P21 — the macOS PGO warning below is EXPECTED. Do not "fix"
    // it by silencing warnings.
    //
    // A PGO build on macOS prints:
    //
    //     warning: rarog@x.y.z: Inherited flag "-fprofile-use=<...>.profdata"
    //              is not supported by the currently used CC
    //
    // What it means: `cc` inherits a subset of rustc codegen flags, and PGO
    // flags are inherited for CLANG ONLY, by design — GCC and clang use
    // incompatible profile formats, and rustc is LLVM like clang. So on macOS
    // `cc` translates `-C profile-use` into `-fprofile-use=`, probes whether
    // the compiler accepts it, and Apple clang refuses: its LLVM is older than
    // the one whose `llvm-profdata` wrote the file. `cc` then drops the flag
    // and says so.
    //
    // What it does NOT mean: that macOS is the odd one out. On MSVC the PGO
    // flags are never inherited at all — verified by capturing the invocation,
    // which is plain `cl.exe -nologo -MD -O2 -Brepro -I vendor/fathom/src -W0
    // /TP /std:c++17 -DTB_NO_HELPER_API ...` with no profile flag of any form.
    // **No platform compiles this file with PGO.** macOS is only the one that
    // mentions it. Reading Windows' silence as "it works there" is precisely
    // the mistake RAR-P21 corrects.
    //
    // Why it is left alone: tablebases work normally — this is about how
    // `tbprobe.c` is optimised, never about whether probing runs or is
    // correct. `SyzygyPath` is empty by default, no gate harness sets it, and
    // datagen hands its tablebase flags to fastchess rather than to the
    // engine, so no measurement can be affected. The exposure is a little
    // probe speed for users who configure tablebases.
    //
    // If the noise ever needs to go, `.inherit_rustflags(false)` is the switch,
    // and it is ISA-safe: `cc` never inherits `-C target-cpu`/`target-feature`
    // and emits no `-march`/`-mcpu`/`-mtune`, so it cannot move this file's
    // instruction set. It is blunt, though — it also stops inheriting
    // `stack-protector`, `dwarf-version` and `control-flow-guard`, none of
    // which we set today but any of which a future flag might rely on.
    let mut build = cc::Build::new();
    build
        .file("vendor/fathom/src/tbprobe.c")
        .include("vendor/fathom/src")
        .define("TB_NO_HELPER_API", None)
        .warnings(false);

    if !target_has_feature("popcnt") {
        // 4.8a — the ISA tier must reach the VENDORED C, not only the Rust.
        //
        // `-C target-cpu=...` is a rustc flag; `cc` never sees it and compiles
        // `tbprobe.c` with the toolchain's own defaults. Fathom then picks its
        // popcount from the COMPILER, not from the target: on MSVC x64 the
        // `_MSC_VER && _M_AMD64` branch takes `_mm_popcnt_u64` with no feature
        // test at all. Measured on the shipped 2.4.0 baseline asset: **15
        // `popcntq` instructions, every one of them from this object**, in the
        // tier whose whole promise is that it runs on a plain x86-64 CPU.
        // POPCNT is SSE4.2-era (Nehalem/Barcelona, 2008), so on an older CPU
        // that is `#UD` — an illegal-instruction crash inside Syzygy probing,
        // not the graceful rejection the engine advertises.
        //
        // Fathom provides exactly the escape hatch needed, so the fix is to
        // USE it whenever the target does not promise the instruction. The
        // software fallback is a five-operation SWAR popcount and is reached
        // only during tablebase probing, so the compatibility tier pays a
        // little probe speed to actually be compatible. Tiers that require
        // `x86-64-v3` (avx2, pext) keep the hardware instruction: v3 includes
        // POPCNT, so for them it is inside the contract.
        build.define("TB_NO_HW_POP_COUNT", None);
    }

    if target_env == "msvc" {
        build
            .cpp(true)
            .flag_if_supported("/TP")
            .flag_if_supported("/std:c++17");
    } else {
        build.flag_if_supported("-std=c11");
    }

    build.compile("fathom");

    if std::env::var("CARGO_CFG_UNIX").is_ok() {
        println!("cargo:rustc-link-lib=pthread");
    }
}

/// Is `feature` enabled for the target this build is compiling for?
///
/// Cargo computes `CARGO_CFG_TARGET_FEATURE` from the fully resolved codegen
/// flags, so it reflects `-C target-cpu=x86-64-v3` and `-C target-feature=+bmi2`
/// exactly as the Rust side sees them. Reading it is what keeps the C and Rust
/// halves of one artifact on the same ISA contract instead of on two.
fn target_has_feature(feature: &str) -> bool {
    std::env::var("CARGO_CFG_TARGET_FEATURE")
        .unwrap_or_default()
        .split(',')
        .any(|enabled| enabled == feature)
}
