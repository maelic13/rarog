use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const RUSTFLAGS_SEPARATOR: &str = "\x1f";
const PGO_TRAINING_TIMEOUT: Duration = Duration::from_secs(20 * 60);

type Result<T> = std::result::Result<T, String>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Arch {
    Base,
    Avx2,
    Pext,
    Arm64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CommandKind {
    Build,
    VerifyIsa,
}

#[derive(Debug)]
struct Config {
    command: CommandKind,
    /// Artifact to verify, when `command` is `VerifyIsa`. Defaults to the
    /// dist asset for the requested tier, either PGO flavour.
    exe: Option<PathBuf>,
    /// The artifact was built with the TARGET's default `target-cpu` rather
    /// than the tier's — i.e. a plain `cargo build --release`. See
    /// [`tier_features`] for why this cannot be inferred.
    default_cpu: bool,
    arch: Arch,
    /// Tune codegen for the exact host CPU instead of the arch's portable
    /// baseline. ORTHOGONAL to `arch`: it swaps `-C target-cpu=<baseline>` for
    /// `-C target-cpu=native` and changes nothing else, so `--arch base
    /// --native` is a valid, correct build for a CPU with no BMI2 or AVX2.
    ///
    /// Before 2.3.0 this was an *arch* (`--arch native`) that hardcoded the
    /// PEXT code path, which meant a non-BMI2 x86_64 host could not get a
    /// native build at all — asking for one produced a binary emitting
    /// `_pext_u64` against a `target-cpu` that does not enable the feature,
    /// i.e. an illegal instruction at runtime.
    native: bool,
    target: String,
    pgo: bool,
    bench_depth: u16,
    /// Cargo features, comma-separated. Needed so a paired-ablation arm
    /// can be PGO-built: a non-PGO arm runs shallower, and a
    /// depth-dependent mechanism's ablation delta would be understated
    /// at that shallower operating point.
    features: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config = parse_args()?;
    ensure_arch_target_pair(config.arch, &config.target)?;
    if config.command == CommandKind::VerifyIsa {
        return verify_isa(&config);
    }
    ensure_rust_target(&config.target)?;

    if config.pgo {
        build_with_pgo(&config)
    } else {
        let target_dir = target_dir("release", config.arch, config.native, &config.target);
        cargo_build(
            &config.target,
            config.arch,
            config.native,
            &target_dir,
            &[],
            &config.features,
        )?;
        copy_dist_binary(
            &binary_path(&target_dir, &config.target),
            config.arch,
            config.native,
            &config.target,
            false,
        )
    }
}

fn parse_args() -> Result<Config> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "build".to_string());
    if command == "help" || command == "--help" || command == "-h" {
        print_usage();
        std::process::exit(0);
    }
    let command = match command.as_str() {
        "build" => CommandKind::Build,
        "verify-isa" => CommandKind::VerifyIsa,
        other => {
            return Err(format!(
                "unknown command `{other}`; expected `build` or `verify-isa`. \
                 Run `cargo xtask help`."
            ));
        }
    };

    let mut arch: Option<Arch> = None;
    let mut target: Option<String> = None;
    let mut exe: Option<PathBuf> = None;
    let mut pgo = false;
    let mut native = false;
    let mut default_cpu = false;
    let mut bench_depth = 13u16;
    let mut features = String::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--features" => {
                features = args
                    .next()
                    .ok_or_else(|| "`--features` requires a value".to_string())?;
            }
            "--arch" | "-a" => {
                let value = args
                    .next()
                    .ok_or_else(|| "`--arch` requires a value".to_string())?;
                if value.eq_ignore_ascii_case("native") {
                    // Legacy spelling: `--arch native` meant "PEXT + host
                    // tuning". Kept working, but it is now expressed as two
                    // independent choices.
                    eprintln!("note: `--arch native` is deprecated; use `--arch pext --native`");
                    arch = Some(Arch::Pext);
                    native = true;
                } else {
                    arch = Some(parse_arch(&value)?);
                }
            }
            "--target" | "-t" => {
                target = Some(
                    args.next()
                        .ok_or_else(|| "`--target` requires a value".to_string())?,
                );
            }
            "--exe" => {
                exe = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "`--exe` requires a value".to_string())?,
                ));
            }
            "--default-cpu" => default_cpu = true,
            "--pgo" => pgo = true,
            "--native" => native = true,
            "--bench-depth" => {
                let value = args
                    .next()
                    .ok_or_else(|| "`--bench-depth` requires a value".to_string())?;
                bench_depth = value
                    .parse::<u16>()
                    .map_err(|_| format!("invalid bench depth `{value}`"))?;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    let arch = arch.unwrap_or_else(default_arch);
    let target = target.unwrap_or_else(|| default_target(arch));
    ensure_native_is_buildable(arch, native, &target)?;
    Ok(Config {
        command,
        exe,
        default_cpu,
        arch,
        native,
        target,
        pgo,
        bench_depth,
        features,
    })
}

fn print_usage() {
    println!(
        "Usage:
  cargo xtask build [--arch base|x86-64|avx2|pext|arm64] [--native] [--target <triple>] [--pgo] [--bench-depth <n>]
  cargo xtask verify-isa [--arch <same>] [--target <triple>] [--exe <path>] [--pgo] [--native] [--default-cpu]

`--arch` picks the ISA contract: which source path compiles (PEXT vs portable
magic bitboards) and which CPU features are required.
`verify-isa` disassembles a finished artifact and holds it to that contract:
no instruction the tier's `target-cpu` does not enable, and every instruction
class the tier exists to emit. Needs `rustup component add llvm-tools`.
`--native` is INDEPENDENT of it: it swaps the portable `target-cpu` baseline
for this exact host CPU. LOCAL ONLY - such a binary is not guaranteed to run
anywhere else, and is marked `-native` in its filename.

Examples:
  cargo xtask build                              # portable x86-64
  cargo xtask build --arch avx2
  cargo xtask build --arch pext --pgo            # the shipped pext asset
  cargo xtask build --arch pext --native --pgo   # fastest build for this box
  cargo xtask build --arch base --native         # native on a pre-BMI2 CPU
  cargo xtask build --arch arm64 --target aarch64-apple-darwin
  cargo xtask verify-isa --arch base             # prove the baseline asset is baseline"
    );
}

fn parse_arch(value: &str) -> Result<Arch> {
    match value.to_ascii_lowercase().as_str() {
        "base" | "x86-64" | "x86_64" | "x64" => Ok(Arch::Base),
        "avx2" => Ok(Arch::Avx2),
        "pext" | "bmi2" => Ok(Arch::Pext),
        "arm64" | "aarch64" => Ok(Arch::Arm64),
        _ => Err(format!(
            "unknown arch `{value}`; expected base, avx2, pext, or arm64"
        )),
    }
}

/// Refuse a combination that cannot produce a working binary on this machine.
///
/// `--arch pext` compiles `_pext_u64` and demands BMI2. Asking for it *and*
/// `--native` says "build this for the machine I am on", so if that machine has
/// no BMI2 the result would be an illegal-instruction binary. Only checked when
/// building for the host, since cross-builds legitimately target other CPUs.
fn ensure_native_is_buildable(arch: Arch, native: bool, target: &str) -> Result<()> {
    if !native || arch != Arch::Pext {
        return Ok(());
    }
    let host = host_triple().unwrap_or_default();
    if target != host {
        return Ok(());
    }
    #[cfg(target_arch = "x86_64")]
    if !std::arch::is_x86_feature_detected!("bmi2") {
        return Err(
            "`--arch pext --native` needs BMI2, which this CPU does not report.              Use `--arch avx2 --native` or `--arch base --native` for a native              build here, or drop `--native` to cross-build a portable pext asset."
                .to_string(),
        );
    }
    Ok(())
}

fn default_arch() -> Arch {
    let host = host_triple().unwrap_or_default();
    if host.starts_with("aarch64-") {
        Arch::Arm64
    } else {
        Arch::Base
    }
}

fn default_target(arch: Arch) -> String {
    let host = host_triple().unwrap_or_default();
    let os = if host.contains("windows") {
        "windows"
    } else if host.contains("apple-darwin") {
        "macos"
    } else {
        "linux"
    };

    match (arch, os) {
        (Arch::Arm64, "windows") => "aarch64-pc-windows-msvc",
        (Arch::Arm64, "macos") => "aarch64-apple-darwin",
        (Arch::Arm64, _) => "aarch64-unknown-linux-gnu",
        (_, "windows") => "x86_64-pc-windows-msvc",
        (_, "macos") => "x86_64-apple-darwin",
        (_, _) => "x86_64-unknown-linux-gnu",
    }
    .to_string()
}

fn ensure_arch_target_pair(arch: Arch, target: &str) -> Result<()> {
    match arch {
        Arch::Base | Arch::Avx2 | Arch::Pext if !target.starts_with("x86_64-") => Err(format!(
            "`--arch {}` requires an x86_64 target, got `{target}`",
            arch_arg_name(arch)
        )),
        Arch::Arm64 if !target.starts_with("aarch64-") => Err(format!(
            "`--arch arm64` requires an aarch64 target, got `{target}`"
        )),
        _ => Ok(()),
    }
}

fn arch_arg_name(arch: Arch) -> &'static str {
    match arch {
        Arch::Base => "base",
        Arch::Avx2 => "avx2",
        Arch::Pext => "pext",
        Arch::Arm64 => "arm64",
    }
}

/// Directory/filename tag for an (arch, native) pair.
///
/// Native and portable builds of the same arch are DIFFERENT binaries, so they
/// must not share a PGO profile directory or an intermediate target dir —
/// otherwise switching flavours silently reuses the other's layout. Cargo
/// fingerprints RUSTFLAGS so it would rebuild rather than emit a stale binary,
/// but keying the paths keeps the artifact tree unambiguous and stops the two
/// flavours thrashing each other's incremental state.
fn flavour_name(arch: Arch, native: bool) -> String {
    if native {
        format!("{}-native", arch_arg_name(arch))
    } else {
        arch_arg_name(arch).to_string()
    }
}

fn asset_arch_name(arch: Arch) -> &'static str {
    match arch {
        Arch::Base => "x86-64",
        Arch::Avx2 => "avx2",
        Arch::Pext => "pext",
        Arch::Arm64 => "arm64",
    }
}

/// Codegen flags for an (arch, native) pair.
///
/// `arch` selects the ISA contract — which source path is compiled
/// (`--cfg rarog_pext`) and which features are required. `native` only swaps
/// the portable `target-cpu` baseline for the host's own CPU. They are
/// independent, so every combination is meaningful: `--arch base --native`
/// tunes for this machine while emitting nothing above the x86-64 baseline,
/// and `--arch pext` stays portable across every BMI2-capable CPU.
///
/// `target-cpu=native` is LOCAL ONLY — the resulting binary is not guaranteed
/// to run on any other machine, so distributed assets never set it.
fn rustflags(arch: Arch, native: bool) -> Vec<String> {
    let cpu = |portable: &str| -> Vec<String> {
        vec![
            "-C".into(),
            if native {
                "target-cpu=native".to_string()
            } else {
                format!("target-cpu={portable}")
            },
        ]
    };
    match arch {
        Arch::Base => cpu("x86-64"),
        Arch::Avx2 => cpu("x86-64-v3"),
        Arch::Pext => {
            let mut flags = vec!["--cfg".into(), "rarog_pext".into()];
            flags.extend(cpu("x86-64-v3"));
            // Required even under `native`: the PEXT source path calls
            // `_pext_u64`, so the feature must be on regardless of how the
            // baseline was chosen.
            flags.extend(["-C".into(), "target-feature=+bmi2".into()]);
            flags
        }
        Arch::Arm64 => cpu("generic"),
    }
}

// ─── 4.8a: the ISA contract, as something that EXECUTES ─────────────────────
//
// A tier is a promise about which instructions an asset may contain, and until
// now that promise lived only in `rustflags` above and in prose. Node agreement
// across CI cells does not test it: a binary that emits POPCNT on the baseline
// tier computes exactly the right node count on every machine that can run it
// at all, and crashes with `#UD` on the machines the tier exists for.
//
// It was not hypothetical. The 2.3.0/2.3.1 baseline assets shipped **15 `popcntq`**,
// every one of them from `vendor/fathom`, because `-C target-cpu` is a rustc
// flag that `cc` never sees and Fathom picks its popcount from the compiler
// rather than from the target (fixed in `build.rs`). This command is what keeps
// that class of drift from coming back silently.
//
// SCOPE, stated honestly: this proves the CLASSES named below and nothing
// wider. It is a lower bound on conformance, not a proof of it.

/// One named instruction class, and the rustc `target_feature` that permits it.
///
/// The feature name is the load-bearing field. What a tier may emit is decided
/// by asking RUSTC what its `target-cpu` enables, never by a list maintained
/// here from memory — that list would be folklore, and folklore is exactly what
/// this command exists to replace. Writing one down was tried first and was
/// wrong within the hour: `movddup` was assumed to be outside the baseline, and
/// the pinned rustc's `x86-64` model turns out to enable `sse3`.
struct InstructionClass {
    name: &'static str,
    feature: &'static str,
    /// Bare mnemonics, matched with or without an operand-size suffix.
    mnemonics: &'static [&'static str],
    /// Matched as a prefix instead, for families too large to list (`v...`).
    prefixes: &'static [&'static str],
}

const X86_CLASSES: &[InstructionClass] = &[
    InstructionClass {
        name: "popcnt",
        feature: "popcnt",
        mnemonics: &["popcnt"],
        prefixes: &[],
    },
    // `tzcnt` is deliberately NOT here — see `TZCNT_IS_BASELINE_SAFE`. `lzcnt`
    // is, and gets its own class because rustc reports it as its own feature.
    InstructionClass {
        name: "lzcnt",
        feature: "lzcnt",
        mnemonics: &["lzcnt"],
        prefixes: &[],
    },
    InstructionClass {
        name: "bmi1",
        feature: "bmi1",
        mnemonics: &["andn", "blsi", "blsmsk", "blsr", "bextr"],
        prefixes: &[],
    },
    // `pext`/`pdep` are split out from the rest of BMI2 so the avx2 tier can
    // forbid the PEXT SOURCE PATH while still permitting the BMI2 instructions
    // `x86-64-v3` grants it. That distinction is what separates the two assets.
    InstructionClass {
        name: "bmi2",
        feature: "bmi2",
        mnemonics: &["shlx", "shrx", "sarx", "bzhi", "mulx", "rorx"],
        prefixes: &[],
    },
    InstructionClass {
        name: "pext",
        feature: "bmi2",
        mnemonics: &["pext", "pdep"],
        prefixes: &[],
    },
    InstructionClass {
        name: "avx",
        feature: "avx",
        mnemonics: &[],
        prefixes: &["v"],
    },
    InstructionClass {
        name: "sse3",
        feature: "sse3",
        mnemonics: &[
            "movddup", "lddqu", "haddpd", "haddps", "hsubpd", "hsubps", "movshdup",
        ],
        prefixes: &[],
    },
    InstructionClass {
        name: "ssse3",
        feature: "ssse3",
        mnemonics: &[
            "pshufb",
            "palignr",
            "phaddd",
            "phaddw",
            "pabsd",
            "pabsb",
            "pabsw",
            "pmaddubsw",
            "psignb",
            "psignd",
        ],
        prefixes: &[],
    },
    InstructionClass {
        name: "sse4.1",
        feature: "sse4.1",
        mnemonics: &[
            "ptest",
            "pblendvb",
            "blendvps",
            "blendvpd",
            "roundss",
            "roundsd",
            "roundps",
            "roundpd",
            "pmulld",
            "pminsd",
            "pmaxsd",
            "pminud",
            "pmaxud",
            "packusdw",
            "extractps",
            "insertps",
            "phminposuw",
            "mpsadbw",
            "dpps",
            "dppd",
            "pcmpeqq",
        ],
        prefixes: &["pmovzx", "pmovsx"],
    },
    InstructionClass {
        name: "sse4.2",
        feature: "sse4.2",
        mnemonics: &[
            "crc32",
            "pcmpgtq",
            "pcmpistri",
            "pcmpestri",
            "pcmpistrm",
            "pcmpestrm",
        ],
        prefixes: &[],
    },
];

/// AArch64 classes above the `generic` baseline.
///
/// SVE is matched only on mnemonics that cannot be anything else — `st1`/`ld1r`
/// look like SVE but are ordinary NEON, and a class that fires on every NEON
/// store would be worse than no class at all.
const ARM_CLASSES: &[InstructionClass] = &[
    InstructionClass {
        name: "sve",
        feature: "sve",
        mnemonics: &[
            "ptrue", "whilelo", "rdvl", "setffr", "addvl", "cntb", "cntd",
        ],
        prefixes: &[],
    },
    InstructionClass {
        name: "aes",
        feature: "aes",
        mnemonics: &["aese", "aesd", "aesmc", "aesimc"],
        prefixes: &[],
    },
    InstructionClass {
        name: "sha2",
        feature: "sha2",
        mnemonics: &[
            "sha1c",
            "sha1h",
            "sha1m",
            "sha1p",
            "sha256h",
            "sha256h2",
            "sha256su0",
        ],
        prefixes: &[],
    },
    InstructionClass {
        name: "dotprod",
        feature: "dotprod",
        mnemonics: &["sdot", "udot"],
        prefixes: &[],
    },
    // 4.8b — the TT prefetch. `prfm` is ARMv8 BASELINE, so it can never be a
    // forbidden class; it is listed so the arm64 tier can REQUIRE it. Until
    // 4.8b the ARM64 assets shipped with `prefetch_ptr` compiled to nothing,
    // which no test, fingerprint or node count could see — the engine plays
    // identically with and without a cache hint, it just plays slower. A
    // required class is the only instrument that catches that class of silent
    // loss, and it is why `neon` is the feature named here: `prfm` needs no
    // feature at all, and `neon` is what `target-cpu=generic` guarantees.
    InstructionClass {
        name: "prefetch",
        feature: "neon",
        mnemonics: &["prfm"],
        prefixes: &[],
    },
];

/// Why `tzcnt` is permitted on a tier that forbids the rest of BMI1.
///
/// Its encoding is `F3 0F BC` — `rep bsf` — and the `rep` prefix is ignored by
/// a CPU without BMI1, so the instruction decodes and runs as `bsf` there. LLVM
/// emits it at `target-cpu=x86-64` deliberately for that reason. The two differ
/// only for a ZERO operand, where `bsf` leaves the destination unchanged while
/// `tzcnt` writes the operand width, and LLVM covers exactly that: measured over
/// the 212 baseline sites, each is either guarded by a `test`/`je` that makes
/// zero unreachable or preceded by a `mov $0x40, dst` whose preserved value IS
/// the answer `tzcnt` would have written.
const TZCNT_IS_BASELINE_SAFE: &str =
    "tzcnt is encoded as `rep bsf` and runs correctly on a pre-BMI1 CPU";

/// What each tier promises, beyond what its `target-cpu` already decides.
struct IsaContract {
    classes: &'static [InstructionClass],
    /// Forbidden even though the tier's `target-cpu` permits it. Only one entry
    /// exists: the avx2 asset must not contain the PEXT source path, which is a
    /// choice about which asset is which, not about what the CPU can execute.
    also_forbidden: &'static [&'static str],
    /// Classes that must be PRESENT. An optimized tier that quietly lost its
    /// optimized path is as broken as a baseline that gained one — and far
    /// harder to notice, because it still produces the right answer.
    required: &'static [&'static str],
}

fn isa_contract(arch: Arch) -> IsaContract {
    match arch {
        Arch::Base => IsaContract {
            classes: X86_CLASSES,
            also_forbidden: &[],
            required: &[],
        },
        Arch::Avx2 => IsaContract {
            classes: X86_CLASSES,
            also_forbidden: &["pext"],
            required: &["avx", "popcnt", "bmi2"],
        },
        Arch::Pext => IsaContract {
            classes: X86_CLASSES,
            also_forbidden: &[],
            required: &["avx", "popcnt", "bmi2", "pext"],
        },
        Arch::Arm64 => IsaContract {
            classes: ARM_CLASSES,
            also_forbidden: &[],
            required: &["prefetch"],
        },
    }
}

/// The target features this tier's codegen flags actually enable, straight from
/// the compiler that will emit the code.
///
/// This is the single source of truth for the whole command. PLAN 4.8 requires
/// the `target-cpu`/`target-feature` contract to be "inspected in generated
/// artifacts" rather than assumed, and asking rustc is how the inspection stays
/// correct across a pinned-toolchain bump instead of decaying into folklore.
fn tier_features(arch: Arch, target: &str, default_cpu: bool) -> Result<Vec<String>> {
    let mut args: Vec<String> = vec![
        "--print".into(),
        "cfg".into(),
        "--target".into(),
        target.into(),
    ];
    // `default_cpu` describes the ARTIFACT's provenance, and getting it wrong
    // makes the whole check meaningless in one direction or the other.
    //
    // A tier asset is built by `cargo xtask build --arch T`, so its permitted
    // set is T's flags. A plain `cargo build --release` binary — which is what
    // the CI bench cells produce — is built with the TARGET's own default
    // `target-cpu`, and on `aarch64-apple-darwin` that is emphatically not
    // `generic`: it enables aes, sha2, dotprod and a dozen more. Holding such a
    // binary to the `generic` set would forbid instructions it is entitled to
    // emit, so the checker would be reporting a defect that is really a
    // mismatch between what was built and what was asserted. Every other
    // shipped triple happens to agree (both x86-64 defaults equal the base
    // tier; both non-Apple aarch64 defaults equal `generic`), which is exactly
    // why this needed to be found by inspection rather than by a green run.
    if !default_cpu {
        // Reuse the very flags the build uses, minus `--cfg`, which selects a
        // source path rather than an instruction set.
        let flags = rustflags(arch, false);
        let mut index = 0;
        while index < flags.len() {
            if flags[index] == "--cfg" {
                index += 2;
                continue;
            }
            args.push(flags[index].clone());
            index += 1;
        }
    }
    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    let cfg = command_output("rustc", &borrowed)?;
    Ok(cfg
        .lines()
        .filter_map(|line| {
            line.strip_prefix("target_feature=\"")
                .and_then(|rest| rest.strip_suffix('"'))
                .map(str::to_string)
        })
        .collect())
}

/// Find the tier's artifact in `target/dist`, accepting either PGO flavour.
///
/// `--pgo` is a BUILD flag; it says nothing about which instructions an
/// artifact contains, so requiring the caller to repeat it here bought nothing
/// and cost a real failure: `cargo xtask build --arch arm64 --pgo` followed by
/// the obvious `cargo xtask verify-isa --arch arm64` reported "artifact not
/// found" while the artifact sat next to the name it looked for. Try the
/// requested flavour, then the other, and only then fail — listing what IS
/// there, because "not found" without the directory contents is the least
/// useful thing a tool can say.
fn resolve_dist_artifact(config: &Config) -> Result<PathBuf> {
    let dist = PathBuf::from("target").join("dist");
    for pgo in [config.pgo, !config.pgo] {
        let candidate = dist.join(asset_name(config.arch, config.native, &config.target, pgo)?);
        if candidate.is_file() {
            if pgo != config.pgo {
                println_flush(format_args!(
                    "  note: using the {} artifact ({})",
                    if pgo { "PGO" } else { "non-PGO" },
                    candidate.display()
                ));
            }
            return Ok(candidate);
        }
    }
    let mut found: Vec<String> = fs::read_dir(&dist)
        .map(|entries| {
            entries
                .filter_map(|entry| Some(entry.ok()?.file_name().to_string_lossy().into_owned()))
                .collect()
        })
        .unwrap_or_default();
    found.sort();
    let listing = if found.is_empty() {
        "  (nothing in target/dist)".to_string()
    } else {
        found
            .iter()
            .map(|name| format!("  {name}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    Err(format!(
        "no `{}` artifact in {}. Build it first, e.g. `cargo xtask build --arch {}`.\n\
         Present:\n{listing}",
        arch_arg_name(config.arch),
        dist.display(),
        arch_arg_name(config.arch)
    ))
}

/// Disassemble one artifact and hold it to its tier's contract.
fn verify_isa(config: &Config) -> Result<()> {
    let exe = match &config.exe {
        Some(path) => {
            if !path.is_file() {
                return Err(format!("artifact not found: {}", path.display()));
            }
            path.clone()
        }
        None => resolve_dist_artifact(config)?,
    };
    let objdump = find_llvm_tool("llvm-objdump").ok_or_else(|| {
        "llvm-objdump not found. Install it with `rustup component add llvm-tools`.".to_string()
    })?;

    let features = tier_features(config.arch, &config.target, config.default_cpu)?;
    println_flush(format_args!(
        "Verifying {} against the `{}` ISA contract",
        exe.display(),
        arch_arg_name(config.arch)
    ));
    println_flush(format_args!(
        "  rustc enables ({}): {}",
        if config.default_cpu {
            "target default cpu"
        } else {
            "tier baseline"
        },
        features.join(" ")
    ));
    let disassembly = command_output(
        objdump.to_str().ok_or("llvm-objdump path is not UTF-8")?,
        &[
            "-d",
            "--no-show-raw-insn",
            exe.to_str().ok_or("artifact path is not UTF-8")?,
        ],
    )?;

    let contract = isa_contract(config.arch);
    let mut counts: Vec<(&str, u64)> = contract
        .classes
        .iter()
        .map(|class| (class.name, 0u64))
        .collect();
    let mut instructions = 0u64;
    let mut tzcnt = 0u64;
    for line in disassembly.lines() {
        let Some(mnemonic) = disassembled_mnemonic(line) else {
            continue;
        };
        instructions += 1;
        if strip_operand_suffix(mnemonic) == "tzcnt" {
            tzcnt += 1;
            continue;
        }
        for (index, class) in contract.classes.iter().enumerate() {
            if class.matches(mnemonic) {
                counts[index].1 += 1;
            }
        }
    }
    if instructions == 0 {
        return Err(format!(
            "llvm-objdump produced no instructions for {} — it is probably not an \
             object file for this host's disassembler",
            exe.display()
        ));
    }

    let count_of = |name: &str| -> u64 {
        counts
            .iter()
            .find_map(|(class, count)| (*class == name).then_some(*count))
            .unwrap_or(0)
    };
    println_flush(format_args!("  {instructions} instructions disassembled"));
    for (class, count) in &counts {
        println_flush(format_args!("    {class:<10} {count}"));
    }
    if tzcnt > 0 {
        println_flush(format_args!(
            "    {:<10} {} (permitted: {})",
            "tzcnt", tzcnt, TZCNT_IS_BASELINE_SAFE
        ));
    }

    let mut failures = Vec::new();
    for class in contract.classes {
        let count = count_of(class.name);
        if count == 0 {
            continue;
        }
        let permitted_by_cpu = features.iter().any(|feature| feature == class.feature);
        let banned_by_tier = contract.also_forbidden.contains(&class.name);
        if !permitted_by_cpu {
            failures.push(format!(
                "FORBIDDEN `{}` appears {count} times, but the `{}` tier does not enable \
                 `{}` — this artifact cannot run on every CPU the tier promises",
                class.name,
                arch_arg_name(config.arch),
                class.feature
            ));
        } else if banned_by_tier {
            failures.push(format!(
                "FORBIDDEN `{}` appears {count} times — the `{}` asset must not carry \
                 that source path, or the two assets are the same binary",
                class.name,
                arch_arg_name(config.arch)
            ));
        }
    }
    for class in contract.required {
        if count_of(class) == 0 {
            failures.push(format!(
                "REQUIRED `{class}` never appears — the `{}` tier is not actually \
                 emitting the path it is built for",
                arch_arg_name(config.arch)
            ));
        }
    }
    if failures.is_empty() {
        println_flush(format_args!("  contract holds"));
        Ok(())
    } else {
        Err(failures.join("\n       "))
    }
}

impl InstructionClass {
    fn matches(&self, mnemonic: &str) -> bool {
        let bare = strip_operand_suffix(mnemonic);
        self.mnemonics.contains(&bare)
            || self.mnemonics.contains(&mnemonic)
            || self
                .prefixes
                .iter()
                .any(|prefix| mnemonic.starts_with(prefix))
    }
}

/// AT&T syntax carries an operand-size suffix (`popcntq`), Intel and AArch64 do
/// not. Strip one so a class list can name the bare instruction.
fn strip_operand_suffix(mnemonic: &str) -> &str {
    let candidate = mnemonic
        .strip_suffix('q')
        .or_else(|| mnemonic.strip_suffix('l'))
        .or_else(|| mnemonic.strip_suffix('w'))
        .or_else(|| mnemonic.strip_suffix('b'));
    // Only strip when something is left that could be an instruction name.
    candidate.filter(|bare| bare.len() >= 3).unwrap_or(mnemonic)
}

/// Pull the mnemonic out of one `llvm-objdump -d --no-show-raw-insn` line.
///
/// The shape is `<hex address>:\t<mnemonic>\t<operands>`, which holds for both
/// the x86 (AT&T) and AArch64 printers. Anything else — headers, section
/// banners, symbol lines, blank lines — is not an instruction.
fn disassembled_mnemonic(line: &str) -> Option<&str> {
    let (address, rest) = line.trim_start().split_once(':')?;
    if address.is_empty() || !address.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let mnemonic = rest
        .split(|c: char| c.is_ascii_whitespace())
        .find(|token| !token.is_empty())?;
    mnemonic
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.')
        .then_some(mnemonic)
}

fn find_llvm_tool(name: &str) -> Option<PathBuf> {
    find_on_path(name).or_else(|| find_rustup_llvm_tool(name))
}

fn find_rustup_llvm_tool(name: &str) -> Option<PathBuf> {
    let sysroot = command_output("rustc", &["--print", "sysroot"]).ok()?;
    let host = host_triple().ok()?;
    let candidate = PathBuf::from(sysroot)
        .join("lib")
        .join("rustlib")
        .join(host)
        .join("bin")
        .join(tool_name(name));
    candidate.exists().then_some(candidate)
}

/// Linker overrides a target needs on top of its codegen flags.
///
/// WORKAROUND — REMOVE WHEN FIXED UPSTREAM (rust-lang/rust#156675, open as of
/// rustc 1.97.1 / LLVM 22.1.6).
///
/// `aarch64-pc-windows-msvc` only. Linked with MSVC's `link.exe`, the
/// instrumented binary runs correctly but writes a `.profraw` whose
/// symbol-name table is empty, so `llvm-profdata merge` rejects every input
/// with "malformed instrumentation profile data: symbol name is empty" and
/// then "no profile can be merged" — PGO cannot complete at all. Linking the
/// same objects with LLD fixes it outright: the merged profile carries real
/// mangled Rust symbols and `-C profile-use` consumes it.
///
/// `/OPT:NOREF /OPT:NOICF` does NOT help, so this is not link.exe dead-stripping
/// `__llvm_prf_names` — the names section is present and non-empty in the
/// failing binary. The two linkers disagree about something else in the
/// `__llvm_prf` layout; the precise mechanism is unknown here.
///
/// On every toolchain bump: delete this function and run
/// `cargo xtask build --arch arm64 --pgo` on Windows ARM64. If the merge
/// succeeds, link.exe is fixed and the override is dead weight.
///
/// Applied to EVERY build of the target, not just `--pgo` ones, so the PGO and
/// non-PGO binaries differ in exactly one variable — the profile — and a
/// performance bisect never has to ask "which linker was that one?".
fn linker_flags(target: &str) -> Result<Vec<String>> {
    if target != "aarch64-pc-windows-msvc" {
        return Ok(Vec::new());
    }
    let lld = find_rust_lld().ok_or_else(|| {
        "rust-lld was not found in the toolchain sysroot. PGO on \
         `aarch64-pc-windows-msvc` requires it: MSVC link.exe produces \
         profiles that llvm-profdata cannot merge (rust-lang/rust#156675). \
         Reinstall the pinned toolchain and retry."
            .to_string()
    })?;
    Ok(vec![
        "-C".into(),
        format!("linker={}", lld.display()),
        "-C".into(),
        "linker-flavor=lld-link".into(),
    ])
}

/// `rust-lld` ships inside the sysroot, so the workaround above adds no
/// external dependency — in particular it does NOT require a separately
/// installed LLVM on the CI runner, which `windows-11-arm` does not guarantee.
/// It is a host executable, so it lives under the HOST triple like the other
/// bundled LLVM tools, even when linking for a different target.
fn find_rust_lld() -> Option<PathBuf> {
    let sysroot = command_output("rustc", &["--print", "sysroot"]).ok()?;
    let host = host_triple().ok()?;
    let candidate = PathBuf::from(sysroot)
        .join("lib")
        .join("rustlib")
        .join(host)
        .join("bin")
        .join(tool_name("rust-lld"));
    candidate.exists().then_some(candidate)
}

fn build_with_pgo(config: &Config) -> Result<()> {
    let host = host_triple()?;
    if config.target != host {
        return Err(format!(
            "PGO training must run the instrumented binary locally, so target `{}` must match host `{host}`",
            config.target
        ));
    }

    let llvm_profdata = ensure_llvm_profdata()?;
    let pgo_dir = PathBuf::from("target")
        .join("pgo")
        .join(sanitize(&config.target))
        .join(flavour_name(config.arch, config.native));
    let raw_dir = pgo_dir.join("raw");
    if raw_dir.exists() {
        fs::remove_dir_all(&raw_dir)
            .map_err(|err| format!("failed to remove `{}`: {err}", raw_dir.display()))?;
    }
    fs::create_dir_all(&raw_dir)
        .map_err(|err| format!("failed to create `{}`: {err}", raw_dir.display()))?;

    let mut generate_flags = vec![
        "-C".to_string(),
        format!("profile-generate={}", raw_dir.display()),
    ];
    generate_flags.extend(rustflags(config.arch, config.native));

    let gen_target_dir = target_dir("pgo-gen", config.arch, config.native, &config.target);
    cargo_build(
        &config.target,
        config.arch,
        config.native,
        &gen_target_dir,
        &generate_flags,
        &config.features,
    )?;

    let instrumented = binary_path(&gen_target_dir, &config.target);
    run_training_bench(&instrumented, &raw_dir, config.bench_depth)?;

    let profdata = pgo_dir.join("rarog.profdata");
    merge_profiles(&llvm_profdata, &raw_dir, &profdata, &config.target)?;

    let mut use_flags = vec![
        "-C".to_string(),
        format!("profile-use={}", profdata.display()),
    ];
    use_flags.extend(rustflags(config.arch, config.native));

    let use_target_dir = target_dir("pgo-use", config.arch, config.native, &config.target);
    cargo_build(
        &config.target,
        config.arch,
        config.native,
        &use_target_dir,
        &use_flags,
        &config.features,
    )?;
    copy_dist_binary(
        &binary_path(&use_target_dir, &config.target),
        config.arch,
        config.native,
        &config.target,
        true,
    )
}

fn cargo_build(
    target: &str,
    arch: Arch,
    native: bool,
    target_dir: &Path,
    override_flags: &[String],
    features: &str,
) -> Result<()> {
    let mut flags = if override_flags.is_empty() {
        rustflags(arch, native)
    } else {
        override_flags.to_vec()
    };
    // Appended here rather than folded into `rustflags` so it reaches the
    // `override_flags` path too — the PGO generate and use builds both go
    // through that branch, and the generate build is precisely the one that
    // must be LLD-linked for the profile to be mergeable.
    flags.extend(linker_flags(target)?);

    println_flush(format_args!(
        "Building Rarog {} for {}{}",
        asset_arch_name(arch),
        target,
        if override_flags.is_empty() {
            ""
        } else {
            " with PGO flags"
        }
    ));

    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg(target)
        .arg("--target-dir")
        .arg(target_dir);
    if !features.is_empty() {
        command.arg("--features").arg(features);
    }
    let status = command
        .env("CARGO_ENCODED_RUSTFLAGS", flags.join(RUSTFLAGS_SEPARATOR))
        .env_remove("RUSTFLAGS")
        .status()
        .map_err(|err| format!("failed to run cargo build: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo build failed with status {status}"))
    }
}

fn run_training_bench(binary: &Path, raw_dir: &Path, depth: u16) -> Result<()> {
    println_flush(format_args!(
        "Training PGO profile with internal bench depth {depth}"
    ));
    let profile_pattern = raw_dir.join("rarog-%p-%m.profraw");
    let mut child = Command::new(binary)
        .env("LLVM_PROFILE_FILE", &profile_pattern)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| format!("failed to run `{}`: {err}", binary.display()))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "failed to open engine stdin".to_string())?;
        writeln!(stdin, "bench {depth}").map_err(|err| format!("failed to start bench: {err}"))?;
        stdin
            .flush()
            .map_err(|err| format!("failed to flush stdin: {err}"))?;
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to open engine stdout".to_string())?;
    let (line_tx, line_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if line_tx.send(line).is_err() {
                break;
            }
        }
    });

    let deadline = Instant::now() + PGO_TRAINING_TIMEOUT;
    let mut saw_summary = false;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            kill_child(&mut child);
            return Err(format!(
                "training bench timed out after {} seconds",
                PGO_TRAINING_TIMEOUT.as_secs()
            ));
        }

        match line_rx.recv_timeout(remaining.min(Duration::from_millis(250))) {
            Ok(Ok(line)) => {
                println_flush(format_args!("{line}"));
                // Mirror Basilisk's PGO guard: a corrupt/illegal bench position
                // must never silently train the profile. `bench` emits
                // "failed to parse" for any position `from_fen` rejects (bad pawn
                // count, back-rank pawns, etc.) and then aborts without a summary
                // — fail fast here rather than hanging until the timeout.
                const ILLEGAL_MARKERS: [&str; 3] = [
                    "failed to parse",
                    "more than 8 pawns",
                    "not legal on the first or eighth rank",
                ];
                if ILLEGAL_MARKERS.iter().any(|marker| line.contains(marker)) {
                    kill_child(&mut child);
                    return Err(format!(
                        "PGO training hit an illegal bench position: {line}"
                    ));
                }
                if line.starts_with("Nodes/second") {
                    saw_summary = true;
                    break;
                }
            }
            Ok(Err(err)) => {
                kill_child(&mut child);
                return Err(format!("failed reading engine output: {err}"));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(status) = child
                    .try_wait()
                    .map_err(|err| format!("failed checking engine status: {err}"))?
                {
                    if status.success() {
                        break;
                    }
                    return Err(format!("training bench exited with status {status}"));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    if let Some(stdin) = child.stdin.as_mut() {
        let _ = writeln!(stdin, "quit");
    }

    let status = wait_child_with_timeout(&mut child, Duration::from_secs(10))?;
    if !status.success() {
        return Err(format!("training bench exited with status {status}"));
    }
    if !saw_summary {
        return Err("training bench did not produce a bench summary".to_string());
    }
    Ok(())
}

fn wait_child_with_timeout(
    child: &mut Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|err| format!("failed checking engine status: {err}"))?
        {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            kill_child(child);
            return child
                .wait()
                .map_err(|err| format!("failed waiting for killed engine: {err}"));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn kill_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn merge_profiles(
    llvm_profdata: &Path,
    raw_dir: &Path,
    profdata: &Path,
    target: &str,
) -> Result<()> {
    let mut inputs = Vec::new();
    for entry in fs::read_dir(raw_dir)
        .map_err(|err| format!("failed to read `{}`: {err}", raw_dir.display()))?
    {
        let path = entry
            .map_err(|err| format!("failed to read profile entry: {err}"))?
            .path();
        if path.extension() == Some(OsStr::new("profraw")) {
            inputs.push(path);
        }
    }
    if inputs.is_empty() {
        return Err(format!(
            "no .profraw files found in `{}`",
            raw_dir.display()
        ));
    }

    println_flush(format_args!("Merging {} profile file(s)", inputs.len()));
    let status = Command::new(llvm_profdata)
        .arg("merge")
        .arg("-output")
        .arg(profdata)
        .args(&inputs)
        .status()
        .map_err(|err| format!("failed to run `{}`: {err}", llvm_profdata.display()))?;

    if status.success() {
        return Ok(());
    }
    let mut message = format!("llvm-profdata merge failed with status {status}");
    message.push_str(&pgo_merge_hint(target));
    Err(message)
}

/// Targets with a known merge failure mode get an actionable hint instead of a
/// bare exit code.
///
/// On `aarch64-pc-windows-msvc` a merge failure now means the LLD override in
/// [`linker_flags`] did not take effect — with it, the merge succeeds; without
/// it, every `.profraw` has an empty symbol-name table. So point at the
/// override rather than declaring PGO impossible.
fn pgo_merge_hint(target: &str) -> String {
    if target.starts_with("aarch64-pc-windows") {
        format!(
            "\n\nnote: PGO on `{target}` works only when the instrumented \
             binary is linked with LLD — MSVC link.exe emits .profraw files \
             with an empty symbol-name table (rust-lang/rust#156675). This \
             build should have passed `-C linker-flavor=lld-link` via \
             `linker_flags`; verify rust-lld exists in the toolchain sysroot."
        )
    } else {
        String::new()
    }
}

fn ensure_rust_target(target: &str) -> Result<()> {
    if find_on_path("rustup").is_none() {
        eprintln!(
            "rustup not found; if `{target}` is not installed, run `rustup target add {target}`."
        );
        return Ok(());
    }

    let status = Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg(target)
        .status()
        .map_err(|err| format!("failed to run rustup target add: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to install target `{target}`; run `rustup target add {target}` manually"
        ))
    }
}

fn ensure_llvm_profdata() -> Result<PathBuf> {
    if let Some(path) = find_llvm_profdata() {
        return Ok(path);
    }

    if find_on_path("rustup").is_none() {
        return Err(
            "llvm-profdata was not found. Install it with `rustup component add llvm-tools-preview` or add LLVM's bin directory to PATH."
                .to_string(),
        );
    }

    println_flush(format_args!(
        "Installing llvm-tools-preview for PGO support"
    ));
    let status = Command::new("rustup")
        .arg("component")
        .arg("add")
        .arg("llvm-tools-preview")
        .status()
        .map_err(|err| format!("failed to run rustup component add: {err}"))?;
    if !status.success() {
        return Err(
            "failed to install llvm-tools-preview; run `rustup component add llvm-tools-preview` manually"
                .to_string(),
        );
    }

    find_llvm_profdata().ok_or_else(|| {
        "llvm-profdata was still not found after installing llvm-tools-preview".to_string()
    })
}

fn find_llvm_profdata() -> Option<PathBuf> {
    find_llvm_tool("llvm-profdata")
}

fn find_on_path(program: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    for dir in env::split_paths(&paths) {
        let candidate = dir.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
        if cfg!(windows) && !program.ends_with(".exe") {
            let candidate = dir.join(format!("{program}.exe"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn tool_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn host_triple() -> Result<String> {
    let output = command_output("rustc", &["-vV"])?;
    output
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_string))
        .ok_or_else(|| "failed to parse rustc host triple".to_string())
}

fn command_output(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run `{program}`: {err}"))?;
    if !output.status.success() {
        return Err(format!("`{program}` failed with status {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn target_dir(kind: &str, arch: Arch, native: bool, target: &str) -> PathBuf {
    PathBuf::from("target").join("xtask").join(format!(
        "{kind}-{}-{}",
        sanitize(target),
        flavour_name(arch, native)
    ))
}

fn binary_path(target_dir: &Path, target: &str) -> PathBuf {
    target_dir
        .join(target)
        .join("release")
        .join(format!("rarog{}", exe_suffix(target)))
}

fn copy_dist_binary(
    binary: &Path,
    arch: Arch,
    native: bool,
    target: &str,
    pgo: bool,
) -> Result<()> {
    if !binary.exists() {
        return Err(format!(
            "expected binary `{}` does not exist",
            binary.display()
        ));
    }
    let dist = PathBuf::from("target").join("dist");
    fs::create_dir_all(&dist)
        .map_err(|err| format!("failed to create `{}`: {err}", dist.display()))?;
    let asset = dist.join(asset_name(arch, native, target, pgo)?);
    fs::copy(binary, &asset).map_err(|err| {
        format!(
            "failed to copy `{}` to `{}`: {err}",
            binary.display(),
            asset.display()
        )
    })?;
    println_flush(format_args!("Built {}", asset.display()));
    Ok(())
}

fn println_flush(args: std::fmt::Arguments<'_>) {
    println!("{args}");
    io::stdout().flush().expect("stdout flush failed");
}

fn asset_name(arch: Arch, native: bool, target: &str, pgo: bool) -> Result<String> {
    let pgo_suffix = if pgo { "-pgo" } else { "" };
    Ok(format!(
        "rarog-v{}-{}-{}{}{}{}",
        package_version()?,
        os_name(target),
        asset_arch_name(arch),
        // A native build is host-specific and must never be mistaken for a
        // distributable asset, so it is marked in the filename.
        if native { "-native" } else { "" },
        pgo_suffix,
        exe_suffix(target)
    ))
}

fn package_version() -> Result<String> {
    let manifest = fs::read_to_string("Cargo.toml")
        .map_err(|err| format!("failed to read Cargo.toml: {err}"))?;
    for line in manifest.lines() {
        let line = line.trim();
        if let Some(version) = line.strip_prefix("version = ") {
            return Ok(version.trim_matches('"').to_string());
        }
    }
    Err("failed to find package version in Cargo.toml".to_string())
}

fn os_name(target: &str) -> &'static str {
    if target.contains("windows") {
        "windows"
    } else if target.contains("apple-darwin") {
        "macos"
    } else {
        "linux"
    }
}

fn exe_suffix(target: &str) -> &'static str {
    if target.contains("windows") {
        ".exe"
    } else {
        ""
    }
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(arch: Arch, native: bool) -> String {
        rustflags(arch, native).join(" ")
    }

    /// The parser has to pick out mnemonics from real `llvm-objdump` output and
    /// reject everything else. A parser that quietly matched nothing would make
    /// `verify-isa` pass every artifact — the failure mode that looks exactly
    /// like success.
    #[test]
    fn disassembly_parser_finds_mnemonics_and_ignores_everything_else() {
        assert_eq!(
            disassembled_mnemonic("140001024:     \tcmpl\t$0x11, 0x10(%rdi)"),
            Some("cmpl")
        );
        assert_eq!(
            disassembled_mnemonic("  100000: \tstp\tx29, x30, [sp, #-16]!"),
            Some("stp")
        );
        assert_eq!(
            disassembled_mnemonic("140037d0d:     \tmovddup\t%xmm1, %xmm1"),
            Some("movddup")
        );
        assert_eq!(disassembled_mnemonic("Disassembly of section .text:"), None);
        assert_eq!(disassembled_mnemonic("0000000140001000 <.text>:"), None);
        assert_eq!(disassembled_mnemonic(""), None);
    }

    /// AT&T prints `popcntq`, AArch64 prints `prfm`. One class list has to name
    /// both without listing every suffix.
    #[test]
    fn instruction_classes_match_with_and_without_operand_suffixes() {
        let popcnt = &X86_CLASSES[0];
        assert_eq!(popcnt.name, "popcnt");
        assert!(popcnt.matches("popcntq"));
        assert!(popcnt.matches("popcnt"));
        assert!(!popcnt.matches("popa"));

        // Named with its trailing `b`, so stripping must not lose it.
        let ssse3 = X86_CLASSES
            .iter()
            .find(|class| class.name == "ssse3")
            .expect("ssse3 class");
        assert!(ssse3.matches("pshufb"));

        let avx = X86_CLASSES
            .iter()
            .find(|class| class.name == "avx")
            .expect("avx class");
        assert!(avx.matches("vmovdqu"));
        assert!(!avx.matches("movdqu"));
    }

    /// `tzcnt` must never be classified as BMI1 — it is the one instruction in
    /// that family a baseline artifact may legitimately contain.
    #[test]
    fn tzcnt_is_not_classified_as_bmi1() {
        let bmi1 = X86_CLASSES
            .iter()
            .find(|class| class.name == "bmi1")
            .expect("bmi1 class");
        assert!(!bmi1.matches("tzcntq"));
        assert!(bmi1.matches("blsrq"));
        assert!(!TZCNT_IS_BASELINE_SAFE.is_empty());
    }

    /// Every class must name a feature rustc actually reports, or the contract
    /// silently permits it: an unknown feature name is never in the enabled
    /// list, which would make the class permanently forbidden rather than
    /// permanently allowed — still wrong, and far harder to notice.
    #[test]
    fn every_class_feature_is_reported_by_rustc_for_the_richest_tier() {
        let features = tier_features(Arch::Pext, "x86_64-unknown-linux-gnu", false)
            .expect("rustc --print cfg");
        for class in X86_CLASSES {
            assert!(
                features.iter().any(|enabled| enabled == class.feature),
                "class `{}` names feature `{}`, which `x86-64-v3 +bmi2` does not enable — \
                 either the name is a typo or the class does not belong to this tier",
                class.name,
                class.feature
            );
        }
    }

    /// 4.8b: the ARM64 assets shipped for three releases with the TT prefetch
    /// compiled to nothing, and nothing could see it — the engine plays
    /// identically without a cache hint, just slower, so no node count, test or
    /// fingerprint moves. Requiring the instruction is the only instrument that
    /// catches a silent loss of that shape, so pin that it IS required.
    #[test]
    fn the_arm64_tier_requires_the_tt_prefetch() {
        assert!(isa_contract(Arch::Arm64).required.contains(&"prefetch"));

        let prefetch = ARM_CLASSES
            .iter()
            .find(|class| class.name == "prefetch")
            .expect("prefetch class");
        assert!(prefetch.matches("prfm"));
        // `prfm` is ARMv8 baseline, so the feature it names must be one the
        // generic tier enables — otherwise the class would be forbidden AND
        // required at once, and the tier could never pass.
        let features = tier_features(Arch::Arm64, "aarch64-unknown-linux-gnu", false)
            .expect("rustc --print cfg");
        assert!(features.iter().any(|f| f == prefetch.feature));
    }

    /// The one tier distinction that is a packaging choice rather than a CPU
    /// capability: avx2 and pext require the same features, so only an explicit
    /// rule keeps the PEXT source path out of the avx2 asset.
    #[test]
    fn only_the_avx2_tier_bans_a_capability_its_cpu_allows() {
        assert_eq!(isa_contract(Arch::Avx2).also_forbidden, &["pext"]);
        assert!(isa_contract(Arch::Base).also_forbidden.is_empty());
        assert!(isa_contract(Arch::Pext).also_forbidden.is_empty());
        assert!(isa_contract(Arch::Pext).required.contains(&"pext"));
    }

    /// The property that motivated the 2.3.0 rework: `--arch` and `--native`
    /// are INDEPENDENT. Before it, `native` was an arch that hardcoded the PEXT
    /// path, so a pre-BMI2 x86_64 host could not get a native build at all —
    /// it got `_pext_u64` against a `target-cpu` that did not enable BMI2.
    #[test]
    fn native_is_orthogonal_to_arch() {
        // Non-PEXT archs must NEVER pull in the PEXT path or BMI2, native or not.
        for arch in [Arch::Base, Arch::Avx2, Arch::Arm64] {
            for native in [false, true] {
                let f = flags(arch, native);
                assert!(
                    !f.contains("rarog_pext"),
                    "{arch:?} native={native} must not enable the PEXT source path: {f}"
                );
                assert!(
                    !f.contains("bmi2"),
                    "{arch:?} native={native} must not require BMI2: {f}"
                );
            }
        }
        // ...and every arch must honour `--native` by targeting the host CPU.
        for arch in [Arch::Base, Arch::Avx2, Arch::Pext, Arch::Arm64] {
            assert!(
                flags(arch, true).contains("target-cpu=native"),
                "{arch:?} --native must target the host CPU"
            );
            assert!(
                !flags(arch, false).contains("target-cpu=native"),
                "{arch:?} without --native must stay portable"
            );
        }
    }

    /// Exact portable baselines — these are the contract the shipped assets are
    /// built against, so a change here changes what users run.
    #[test]
    fn portable_baselines_are_pinned() {
        assert_eq!(flags(Arch::Base, false), "-C target-cpu=x86-64");
        assert_eq!(flags(Arch::Avx2, false), "-C target-cpu=x86-64-v3");
        assert_eq!(
            flags(Arch::Pext, false),
            "--cfg rarog_pext -C target-cpu=x86-64-v3 -C target-feature=+bmi2"
        );
        assert_eq!(flags(Arch::Arm64, false), "-C target-cpu=generic");
    }

    /// PEXT compiles `_pext_u64`, so BMI2 must be required even under `--native`
    /// — `target-cpu=native` alone would not guarantee the feature is on.
    #[test]
    fn pext_requires_bmi2_even_when_native() {
        let f = flags(Arch::Pext, true);
        assert!(f.contains("--cfg rarog_pext"), "{f}");
        assert!(f.contains("target-feature=+bmi2"), "{f}");
        assert!(f.contains("target-cpu=native"), "{f}");
    }

    /// A bare `llvm-profdata` exit code is unactionable on the one target that
    /// requires a linker workaround, so that target — and only that target —
    /// must explain how to verify the LLD override.
    #[test]
    fn pgo_merge_hint_fires_only_for_windows_arm64() {
        let hint = pgo_merge_hint("aarch64-pc-windows-msvc");
        assert!(
            hint.contains("linked with LLD")
                && hint.contains("linker-flavor=lld-link")
                && hint.contains("rust-lld"),
            "windows-arm64 merge failure must explain itself: {hint}"
        );
        for target in [
            "aarch64-apple-darwin",
            "aarch64-unknown-linux-gnu",
            "x86_64-pc-windows-msvc",
            "x86_64-unknown-linux-gnu",
        ] {
            assert!(
                pgo_merge_hint(target).is_empty(),
                "{target} has working PGO and must not claim otherwise"
            );
        }
    }

    /// PGO is a third independent axis: it PREPENDS its own flag and must leave
    /// the arch/native flags untouched, in both phases.
    #[test]
    fn pgo_flags_compose_with_every_flavour() {
        for arch in [Arch::Base, Arch::Avx2, Arch::Pext, Arch::Arm64] {
            for native in [false, true] {
                for phase in ["profile-generate=/tmp/raw", "profile-use=/tmp/x.profdata"] {
                    let mut composed = vec!["-C".to_string(), phase.to_string()];
                    composed.extend(rustflags(arch, native));
                    let joined = composed.join(" ");
                    assert!(joined.contains(phase), "{joined}");
                    for flag in rustflags(arch, native) {
                        assert!(
                            joined.contains(&flag),
                            "PGO composition dropped `{flag}` for {arch:?} native={native}"
                        );
                    }
                }
            }
        }
    }

    /// A host-tuned binary must never be mistaken for a distributable asset,
    /// and native/portable builds must not share PGO or intermediate dirs.
    #[test]
    fn native_builds_are_tagged_everywhere() {
        let portable = asset_name(Arch::Pext, false, "x86_64-pc-windows-msvc", true).unwrap();
        let native = asset_name(Arch::Pext, true, "x86_64-pc-windows-msvc", true).unwrap();
        assert!(!portable.contains("native"), "{portable}");
        assert!(native.contains("-native"), "{native}");
        assert!(portable.ends_with("-pgo.exe") && native.ends_with("-pgo.exe"));
        assert_ne!(portable, native);

        assert_eq!(flavour_name(Arch::Pext, false), "pext");
        assert_eq!(flavour_name(Arch::Pext, true), "pext-native");
        assert_ne!(
            target_dir("pgo-gen", Arch::Pext, false, "x86_64-pc-windows-msvc"),
            target_dir("pgo-gen", Arch::Pext, true, "x86_64-pc-windows-msvc"),
            "native and portable PGO builds must not share an intermediate dir"
        );
    }

    /// `native` is no longer an arch; it is a flag. `parse_args` still accepts
    /// the old spelling, but `parse_arch` itself must not.
    #[test]
    fn native_is_not_an_arch() {
        assert!(parse_arch("native").is_err());
        assert_eq!(parse_arch("pext").unwrap(), Arch::Pext);
        assert_eq!(parse_arch("bmi2").unwrap(), Arch::Pext);
        assert_eq!(parse_arch("x86-64").unwrap(), Arch::Base);
        assert_eq!(parse_arch("aarch64").unwrap(), Arch::Arm64);
    }

    #[test]
    fn arch_and_target_must_agree() {
        assert!(ensure_arch_target_pair(Arch::Pext, "x86_64-pc-windows-msvc").is_ok());
        assert!(ensure_arch_target_pair(Arch::Pext, "aarch64-apple-darwin").is_err());
        assert!(ensure_arch_target_pair(Arch::Arm64, "aarch64-apple-darwin").is_ok());
        assert!(ensure_arch_target_pair(Arch::Arm64, "x86_64-unknown-linux-gnu").is_err());
    }

    /// The BMI2 pre-flight only applies to a native PEXT build FOR THIS HOST;
    /// every other combination must pass regardless of what CPU runs the tests.
    #[test]
    fn bmi2_guard_only_fires_for_native_pext_on_host() {
        assert!(ensure_native_is_buildable(Arch::Pext, false, "x86_64-pc-windows-msvc").is_ok());
        assert!(ensure_native_is_buildable(Arch::Base, true, "x86_64-pc-windows-msvc").is_ok());
        assert!(ensure_native_is_buildable(Arch::Avx2, true, "x86_64-pc-windows-msvc").is_ok());
        // Cross-build: the host's features say nothing about the target's.
        assert!(ensure_native_is_buildable(Arch::Pext, true, "some-other-triple").is_ok());
    }
}
