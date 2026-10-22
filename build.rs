fn main() {
    println!("cargo:rustc-check-cfg=cfg(rarog_pext)");

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // Fathom's C is compiled without PGO on every platform. `cc` inherits
    // rustc's codegen flags and, for clang, turns `-C profile-generate` and
    // `-C profile-use` into `-fprofile-*`. Apple clang accepts the generate
    // flag and instruments `tbprobe.c` in its own LLVM's raw-profile format,
    // which the Rust profiling runtime refuses once the two LLVMs differ
    // (rustc 1.99, LLVM 23: "Runtime and instrumentation version mismatch:
    // expected 11, but get 10"), so training writes no profile and the macOS
    // PGO build fails. MSVC and GCC never inherit the PGO flags, so their C
    // object is unchanged. Inheritance is off only under PGO; `cc` never
    // inherits `-C target-cpu`/`target-feature`, so this cannot move the
    // file's instruction set, which the ISA tier sets below.
    let rustflags = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    let pgo = rustflags.contains("profile-generate") || rustflags.contains("profile-use");
    let mut build = cc::Build::new();
    build
        .file("vendor/fathom/src/tbprobe.c")
        .include("vendor/fathom/src")
        .define("TB_NO_HELPER_API", None)
        .warnings(false)
        .inherit_rustflags(!pgo);

    if !target_has_feature("popcnt") {
        // The ISA tier must reach the VENDORED C, not only the Rust.
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
