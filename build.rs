#[path = "src/build_name.rs"]
mod build_name;
#[path = "src/eval/endgame/kpk_generate.rs"]
mod kpk_generate;

use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(rarog_pext)");
    write_kpk_bitbase();

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
    name_the_build(&rustflags, pgo);
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

    // The C compiler's default instruction set is the toolchain's, not the
    // tier's, and it moves with the toolchain: built on the `ubuntu-26.04`
    // runner, the baseline asset carried 39 POPCNT, 101 BMI1, 281 BMI2 and
    // 295 AVX instructions, which rustc's baseline target cannot emit, where
    // the `ubuntu-24.04` build had none. So GNU and Clang get the tier's
    // level by name, read from the same target features as the Rust. MSVC's
    // x64 default is SSE2, the baseline itself.
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "x86_64" && target_env != "msvc" {
        if target_has_feature("avx2") && target_has_feature("bmi2") {
            build.flag_if_supported("-march=x86-64-v3");
        } else if target_has_feature("popcnt") && target_has_feature("sse4.2") {
            build.flag_if_supported("-march=x86-64-v2");
        } else {
            build.flag("-march=x86-64");
        }
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

/// The KPK bitbase, for `kpk.rs` to include: generated here, the engine
/// neither builds it at run time nor carries the generator.
fn write_kpk_bitbase() {
    let out = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR");
    std::fs::write(Path::new(&out).join("kpk.bin"), kpk_generate::bitbase())
        .expect("write the KPK bitbase");
}

/// Bake the build's name into the engine: `RAROG_VERSION`, which `id name`
/// reports, and `RAROG_BUILD_LABEL`, the tier and kinds the start-up line
/// names. `xtask` names the file from the same rules and checks the built
/// engine reports what the file name says.
fn name_the_build(rustflags: &str, pgo: bool) {
    let package = std::env::var("CARGO_PKG_VERSION").expect("cargo sets CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed={}", build_name::DEV_ID_VARIABLE);
    if build_name::is_development(&package) {
        watch_development_inputs();
    }
    let version =
        build_name::display_version(&package, Path::new(".")).unwrap_or_else(|err| panic!("{err}"));
    println!("cargo:rustc-env=RAROG_VERSION={version}");

    // Every enabled feature, as cargo names it here (`CARGO_FEATURE_<NAME>`).
    let features: Vec<String> = std::env::vars_os()
        .filter_map(|(key, _)| {
            key.to_str()?
                .strip_prefix("CARGO_FEATURE_")
                .map(str::to_lowercase)
        })
        .collect();
    let features: Vec<&str> = features.iter().map(String::as_str).collect();
    let debug = std::env::var("PROFILE").is_ok_and(|profile| profile == "debug");
    let kinds = build_name::kinds(pgo, debug, &features);
    let label = build_name::label(build_tier(rustflags), &kinds);
    println!("cargo:rustc-env=RAROG_BUILD_LABEL={label}");
}

/// The tier these codegen flags build, by `xtask`'s names: `native` in its
/// place for a host-tuned build, and `None` for a portable build on a target
/// without tiers (arm64).
fn build_tier(rustflags: &str) -> Option<&'static str> {
    if rustflags.contains("target-cpu=native") {
        return Some("native");
    }
    if std::env::var("CARGO_CFG_TARGET_ARCH").ok().as_deref() != Some("x86_64") {
        return None;
    }
    // `rarog_pext` first: the PEXT build enables every `avx2` feature too.
    Some(if std::env::var_os("CARGO_CFG_RAROG_PEXT").is_some() {
        "pext"
    } else if target_has_feature("avx2") {
        "avx2"
    } else if target_has_feature("sse4.2") && target_has_feature("popcnt") {
        "sse42"
    } else {
        "base"
    })
}

/// A development build names its commit and whether the tree differs from
/// it, so the build script reruns when either can change: on a new commit or
/// checkout (the HEAD file and its reflog) and on an edit to anything the
/// engine is built from. Naming any path replaces cargo's default of
/// rerunning on every file in the package, so the inputs are all named.
fn watch_development_inputs() {
    for input in ["src", "build.rs", "Cargo.toml", "Cargo.lock", "vendor"] {
        println!("cargo:rerun-if-changed={input}");
    }
    for git_file in ["HEAD", "logs/HEAD"] {
        let Ok(output) = Command::new("git")
            .args(["rev-parse", "--git-path", git_file])
            .output()
        else {
            return;
        };
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if output.status.success() && Path::new(&path).is_file() {
            println!("cargo:rerun-if-changed={path}");
        }
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
