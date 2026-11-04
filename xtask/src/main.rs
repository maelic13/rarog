mod release;
mod universal;

// The naming rules and the tier ladder are the engine's, compiled here too so
// the file name and the CPU detection cannot drift from what the engine says.
#[path = "../../src/build_name.rs"]
mod build_name;
#[path = "../../src/cpu_tier.rs"]
mod cpu_tier;

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

/// What a build targets: one x86-64 tier, the universal x86-64 binary that
/// holds every tier, or arm64, which has none.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Arch {
    Base,
    Sse42,
    Avx2,
    Pext,
    Universal,
    Arm64,
}

impl Arch {
    const fn from_tier(tier: cpu_tier::Tier) -> Arch {
        match tier {
            cpu_tier::Tier::Base => Arch::Base,
            cpu_tier::Tier::Sse42 => Arch::Sse42,
            cpu_tier::Tier::Avx2 => Arch::Avx2,
            cpu_tier::Tier::Pext => Arch::Pext,
        }
    }

    const fn tier(self) -> Option<cpu_tier::Tier> {
        match self {
            Arch::Base => Some(cpu_tier::Tier::Base),
            Arch::Sse42 => Some(cpu_tier::Tier::Sse42),
            Arch::Avx2 => Some(cpu_tier::Tier::Avx2),
            Arch::Pext => Some(cpu_tier::Tier::Pext),
            Arch::Universal | Arch::Arm64 => None,
        }
    }
}

/// What a PGO build is for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Purpose {
    /// The binary is the product.
    Binary,
    /// One tier of the universal binary: every function aligned to 64 bytes,
    /// and the post-LTO object and the link line kept for the universal link.
    UniversalTier,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CommandKind {
    Build,
    VerifyIsa,
}

#[derive(Debug, Clone)]
struct Config {
    command: CommandKind,
    /// Artifact to verify, when `command` is `VerifyIsa`. Defaults to the
    /// dist file for the requested build, PGO or not.
    exe: Option<PathBuf>,
    /// The artifact was built with the TARGET's default `target-cpu` rather
    /// than the tier's — i.e. a plain `cargo build --release`. See
    /// [`tier_features`] for why this cannot be inferred.
    default_cpu: bool,
    arch: Arch,
    /// Tune codegen for the exact host CPU instead of the tier's portable
    /// baseline. It keeps the host's best tier's source path (PEXT or not)
    /// and swaps `-C target-cpu=<baseline>` for `-C target-cpu=native`; the
    /// file names `native` in the tier's place.
    native: bool,
    target: String,
    /// Profile-guided: the default, and what every published build is.
    pgo: bool,
    bench_depth: u16,
    /// Cargo features, each a build kind the file and start-up line name
    /// (`build_name::kinds`). A candidate arm behind a feature is PGO-built
    /// like any other, since a non-PGO arm runs shallower.
    features: Vec<String>,
}

impl Config {
    fn feature_list(&self) -> Vec<&str> {
        self.features.iter().map(String::as_str).collect()
    }

    /// The tier word of the file name and start-up line: the tier, `native`
    /// in its place, or nothing for a portable arm64 build.
    fn tier_word(&self) -> Option<&'static str> {
        if self.native {
            Some("native")
        } else {
            self.arch.tier().map(cpu_tier::Tier::name)
        }
    }

    /// The kinds the build carries: `nopgo`, then its features.
    fn kinds(&self) -> Vec<String> {
        build_name::kinds(self.pgo, false, &self.feature_list())
    }
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
    let version = build_name::display_version(&package_version()?, Path::new("."))?;

    let binary = if config.arch == Arch::Universal {
        universal::build(&config, &version)?
    } else if config.pgo {
        build_with_pgo(&config, Purpose::Binary)?
    } else {
        let target_dir = target_dir("release", config.arch, config.native, &config.target);
        cargo_build(&config, &target_dir, &[], None)?;
        binary_path(&target_dir, &config.target)
    };
    let asset = copy_dist_binary(&binary, &config, &version)?;
    if config.arch == Arch::Universal {
        if let Err(err) = universal::check(&asset, &version) {
            let _ = fs::remove_file(&asset);
            return Err(err);
        }
    } else if host_runs(&config)? {
        let label = build_name::label(config.tier_word(), &config.kinds());
        if let Err(err) = check_reported_name(&asset, &version, &label, None) {
            // A file whose name the engine contradicts must not be picked up.
            let _ = fs::remove_file(&asset);
            return Err(err);
        }
    } else {
        println_flush(format_args!(
            "  note: this host cannot run the build, so the name it reports is unchecked"
        ));
    }
    Ok(())
}

fn parse_args() -> Result<Config> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "build".to_string());
    if command == "help" || command == "--help" || command == "-h" {
        print_usage();
        std::process::exit(0);
    }
    if command == "release-check" {
        let check = parse_release_check(args)?;
        release::run(&check)?;
        std::process::exit(0);
    }
    if command == "declared-fingerprint" {
        if let Some(extra) = args.next() {
            return Err(format!(
                "`declared-fingerprint` takes no arguments, got `{extra}`"
            ));
        }
        let guide = std::fs::read_to_string("GUIDE.md").map_err(|e| format!("GUIDE.md: {e}"))?;
        println!("{}", release::declared_fingerprint(&guide)?);
        std::process::exit(0);
    }
    if command == "version" {
        if let Some(extra) = args.next() {
            return Err(format!("`version` takes no arguments, got `{extra}`"));
        }
        println!(
            "{}",
            build_name::display_version(&package_version()?, Path::new("."))?
        );
        std::process::exit(0);
    }
    let command = match command.as_str() {
        "build" => CommandKind::Build,
        "verify-isa" => CommandKind::VerifyIsa,
        other => {
            return Err(format!(
                "unknown command `{other}`; expected `build`, `verify-isa`, `version`, \
                 `release-check` or `declared-fingerprint`. Run `cargo xtask help`."
            ));
        }
    };

    let mut request: Option<Request> = None;
    let mut target: Option<String> = None;
    let mut exe: Option<PathBuf> = None;
    let mut pgo = true;
    let mut default_cpu = false;
    let mut bench_depth = 13u16;
    let mut features = Vec::new();

    while let Some(arg) = args.next() {
        let mut value = |name: &str| -> Result<String> {
            args.next()
                .ok_or_else(|| format!("`{name}` requires a value"))
        };
        let mut ask = |wanted: Request| -> Result<()> {
            if request.replace(wanted).is_some() {
                return Err(
                    "`--tier`, `--native` and `--universal` exclude one another".to_string()
                );
            }
            Ok(())
        };
        match arg.as_str() {
            // The default; spelled out for symmetry with `--universal`.
            "--release" => {}
            "--tier" => ask(Request::Tier(parse_tier(&value("--tier")?)?))?,
            "--universal" => ask(Request::Universal)?,
            "--native" => ask(Request::Native)?,
            "--features" => features = parse_features(&value("--features")?)?,
            "--target" | "-t" => target = Some(value("--target")?),
            "--exe" => exe = Some(PathBuf::from(value("--exe")?)),
            "--default-cpu" => default_cpu = true,
            "--nopgo" => pgo = false,
            "--bench-depth" => {
                let depth = value("--bench-depth")?;
                bench_depth = depth
                    .parse::<u16>()
                    .map_err(|_| format!("invalid bench depth `{depth}`"))?;
            }
            "--arch" | "-a" => {
                return Err(
                    "`--arch` is gone: `--tier base|sse42|avx2|pext` picks an x86-64 tier, \
                     the default is this host's best, and an arm64 build follows its `--target`"
                        .to_string(),
                );
            }
            "--pgo" => {
                return Err("PGO is the default now; `--nopgo` turns it off".to_string());
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    let host = host_triple()?;
    let target = target.unwrap_or_else(|| host.clone());
    let request = request.unwrap_or(Request::Best);
    let arch = resolve_arch(request, &target, &host)?;
    if arch == Arch::Universal && command == CommandKind::Build && (!pgo || !features.is_empty()) {
        return Err(
            "the universal binary is the release build: PGO, with the default features".to_string(),
        );
    }
    let config = Config {
        command,
        exe,
        default_cpu,
        arch,
        native: request == Request::Native,
        target,
        pgo,
        bench_depth,
        features,
    };
    if config.command == CommandKind::Build && config.pgo {
        ensure_host_trains(&config, &host)?;
    }
    Ok(config)
}

/// What the command line asks for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Request {
    /// The host's best tier, or arm64 on an aarch64 target.
    Best,
    /// One x86-64 tier.
    Tier(cpu_tier::Tier),
    /// The host's best tier, tuned for this exact CPU.
    Native,
    /// The universal x86-64 binary.
    Universal,
}

/// The build a request names on `target`.
fn resolve_arch(request: Request, target: &str, host: &str) -> Result<Arch> {
    if request == Request::Native && target != host {
        return Err(format!(
            "`--native` builds for this host, so it cannot target `{target}` from `{host}`"
        ));
    }
    if target.starts_with("aarch64-") {
        return match request {
            Request::Tier(tier) => Err(format!(
                "`--tier {}` is an x86-64 tier; an aarch64 target has none",
                tier.name()
            )),
            Request::Universal => {
                Err("the universal binary is x86-64; an aarch64 target has no tiers".to_string())
            }
            Request::Best | Request::Native => Ok(Arch::Arm64),
        };
    }
    if !target.starts_with("x86_64-") {
        return Err(format!("unsupported target `{target}`"));
    }
    match request {
        Request::Tier(tier) => Ok(Arch::from_tier(tier)),
        Request::Universal => Ok(Arch::Universal),
        Request::Best | Request::Native if target != host => Err(format!(
            "a build for `{target}` from `{host}` names its `--tier`: this host's CPU says \
             nothing about the target's"
        )),
        Request::Best | Request::Native => Ok(Arch::from_tier(host_best_tier()?)),
    }
}

/// The fastest tier this host runs.
fn host_best_tier() -> Result<cpu_tier::Tier> {
    let level = cpu_tier::host_level()
        .ok_or("this host is not x86-64, so it has no best x86-64 tier; name a `--tier`")?;
    Ok(cpu_tier::best_tier(level, cpu_tier::host_has_slow_pext()))
}

/// Refuse a PGO build this host cannot train: PGO runs the instrumented
/// build, so the target must be the host and the tier one the CPU runs.
fn ensure_host_trains(config: &Config, host: &str) -> Result<()> {
    if config.target != host {
        return Err(format!(
            "a PGO build trains by running the engine, so it cannot target `{}` from `{host}`; \
             `--nopgo` builds it without training",
            config.target
        ));
    }
    if host_runs(config)? {
        Ok(())
    } else {
        Err(format!(
            "this CPU cannot run the `{}` tier, and a PGO build trains by running it; \
             `--nopgo` builds it without training",
            arch_arg_name(config.arch)
        ))
    }
}

/// Can this host run the build? Only a host build can be run at all.
fn host_runs(config: &Config) -> Result<bool> {
    if config.target != host_triple()? {
        return Ok(false);
    }
    Ok(match config.arch.tier() {
        Some(tier) => cpu_tier::host_level().is_some_and(|level| tier.level() <= level),
        None => true,
    })
}

/// `release-check vX.Y.Z [--notes <path>] [--base <ref>]`.
fn parse_release_check(mut args: impl Iterator<Item = String>) -> Result<release::ReleaseCheck> {
    let tag = args.next().ok_or_else(|| {
        "`release-check` needs the tag: cargo xtask release-check vX.Y.Z".to_string()
    })?;
    let mut notes = None;
    let mut base = "origin/master".to_string();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--notes" => {
                let path = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "`--notes` requires a path".to_string())?,
                );
                release::ensure_parent(&path)?;
                notes = Some(path);
            }
            "--base" => {
                base = args
                    .next()
                    .ok_or_else(|| "`--base` requires a git ref".to_string())?;
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(release::ReleaseCheck { tag, notes, base })
}

fn print_usage() {
    println!(
        "Usage:
  cargo xtask build [--release] [--tier base|sse42|avx2|pext] [--native] [--nopgo]
                    [--features tune|diag|texel] [--target <triple>] [--bench-depth <n>]
  cargo xtask verify-isa [--tier <same>] [--target <triple>] [--exe <path>] [--native]
                         [--nopgo] [--features <same>] [--default-cpu]
  cargo xtask version
  cargo xtask release-check vX.Y.Z [--notes <path>] [--base <ref>]
  cargo xtask declared-fingerprint

`build` makes the best build this host runs: its fastest x86-64 tier (or
arm64), profile-guided, with that tier's portable code generation. `--release`
is the same. It writes target/dist/rarog-<version>-<os>-<x64|arm64>[-<tier>][-<kind>],
the version being what `id name` reports (`2.7.0`, or `dev-<date>-<hash>`
for a development build, `-m` before the hash when the tree differs from it),
and checks the built engine reports that name.
`--tier` picks another x86-64 tier; a PGO build trains by running the engine,
so it must be one this CPU runs. `--nopgo` skips the training and names the
file `-nopgo`; `--native` tunes for this exact CPU (LOCAL ONLY, named
`-native`, not faster on a Zen 3); `--features` builds a kind named in the file.
An aarch64 `--target` builds arm64; a cross build needs `--nopgo`, and an
x86-64 one its `--tier`.
`verify-isa` disassembles a finished artifact and holds it to its tier's
contract: no instruction the tier's `target-cpu` does not enable, and every
instruction class the tier exists to emit. Needs `rustup component add llvm-tools`.
`version` prints the version a build of this tree reports.
`release-check` refuses a tag that does not name Cargo.toml's version, a HEAD
not reachable from `--base` (default origin/master) unless it is a patch
descending from its line's `vX.Y.0` on `--base`, a CHANGELOG without a
dated section for the version, or a GUIDE whose Released baseline row does not
name the version at the Development head row's fingerprint; `--notes` writes
that section as the notes. `declared-fingerprint` prints the `bench 13` node
count GUIDE's Development head row declares, which every asset must reproduce.

Examples:
  cargo xtask build                               # this host's best tier, PGO
  cargo xtask build --tier avx2                   # another tier
  cargo xtask build --nopgo                       # no profile: rarog-...-x64-pext-nopgo
  cargo xtask build --features tune               # the SPSA build: rarog-...-x64-pext-tune
  cargo xtask build --native                      # tuned for this CPU
  cargo xtask build --target aarch64-apple-darwin --nopgo
  cargo xtask verify-isa --tier base              # prove the baseline build is baseline
  cargo xtask release-check v2.7.0 --notes notes.md   # what the release workflow checks first"
    );
}

fn parse_tier(value: &str) -> Result<cpu_tier::Tier> {
    cpu_tier::Tier::ALL
        .into_iter()
        .find(|tier| tier.name() == value)
        .ok_or_else(|| format!("unknown tier `{value}`; expected base, sse42, avx2 or pext"))
}

/// The features of a build, each one a kind its name carries. A name word is
/// lower-case letters, digits and `_`; cargo itself refuses an unknown one.
fn parse_features(value: &str) -> Result<Vec<String>> {
    value
        .split(',')
        .map(str::trim)
        .filter(|feature| !feature.is_empty())
        .map(|feature| {
            let plain = feature
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
            if plain && feature != "default" {
                Ok(feature.to_string())
            } else {
                Err(format!(
                    "feature `{feature}` cannot be named in a file: a build's features \
                     are lower-case letters, digits and `_`, and not `default`"
                ))
            }
        })
        .collect()
}

fn ensure_arch_target_pair(arch: Arch, target: &str) -> Result<()> {
    match arch {
        Arch::Arm64 if !target.starts_with("aarch64-") => Err(format!(
            "an arm64 build requires an aarch64 target, got `{target}`"
        )),
        Arch::Arm64 => Ok(()),
        _ if !target.starts_with("x86_64-") => Err(format!(
            "the `{}` build requires an x86_64 target, got `{target}`",
            arch_arg_name(arch)
        )),
        _ => Ok(()),
    }
}

fn arch_arg_name(arch: Arch) -> &'static str {
    match arch {
        Arch::Universal => "universal",
        Arch::Arm64 => "arm64",
        tier => tier.tier().map_or("arm64", cpu_tier::Tier::name),
    }
}

/// Directory tag for an (arch, native) pair.
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
        // The universal binary's own code is its dispatcher, at the baseline.
        Arch::Base | Arch::Universal => cpu("x86-64"),
        Arch::Sse42 => cpu("x86-64-v2"),
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

// ─── The ISA contract, as something that EXECUTES ────────────────────────────
//
// A tier is a promise about which instructions an asset may contain, and
// `rustflags` above state it but do not check it. Node agreement
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
    // The TT prefetch. `prfm` is ARMv8 BASELINE, so it can never be a
    // forbidden class; it is listed so the arm64 tier can REQUIRE it. A
    // `prefetch_ptr` compiled to nothing is invisible to every test,
    // fingerprint and node count — the engine plays
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
        // The whole image holds every tier, so it is held to their union:
        // the pext tier's contract. Each tier, the dispatcher and Fathom are
        // held to their own before the link (`universal::build`).
        Arch::Universal => IsaContract {
            classes: X86_CLASSES,
            also_forbidden: &[],
            required: &["avx", "popcnt", "bmi2", "pext"],
        },
        // The hardware popcount is most of what this tier is for.
        Arch::Sse42 => IsaContract {
            classes: X86_CLASSES,
            also_forbidden: &[],
            required: &["popcnt"],
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
/// This is the single source of truth for the whole command. The
/// `target-cpu`/`target-feature` contract is inspected in generated artifacts
/// rather than assumed, and asking rustc is how the inspection stays
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
        let flags = rustflags(
            if arch == Arch::Universal {
                Arch::Pext
            } else {
                arch
            },
            false,
        );
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
/// and cost a real failure: a PGO arm64 build followed by the obvious
/// `verify-isa` reported "artifact not found" while the artifact sat next to
/// the name it looked for. Try the requested flavour, then the other, and only
/// then fail — listing what IS there, because "not found" without the
/// directory contents is the least useful thing a tool can say.
fn resolve_dist_artifact(config: &Config) -> Result<PathBuf> {
    let dist = PathBuf::from("target").join("dist");
    let version = build_name::display_version(&package_version()?, Path::new("."))?;
    for pgo in [config.pgo, !config.pgo] {
        let candidate = dist.join(asset_name(config, pgo, &version));
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
        "no `{}` artifact for {version} in {}. Build it first, e.g. `cargo xtask build`.\n\
         Present:\n{listing}",
        arch_arg_name(config.arch),
        dist.display(),
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
    check_isa(&exe, config.arch, &config.target, config.default_cpu)
}

/// Hold `exe` (an executable, an object or an archive of objects) to the
/// contract of `arch`.
fn check_isa(exe: &Path, arch: Arch, target: &str, default_cpu: bool) -> Result<()> {
    let objdump = find_llvm_tool("llvm-objdump").ok_or_else(|| {
        "llvm-objdump not found. Install it with `rustup component add llvm-tools`.".to_string()
    })?;

    let features = tier_features(arch, target, default_cpu)?;
    println_flush(format_args!(
        "Verifying {} against the `{}` ISA contract",
        exe.display(),
        arch_arg_name(arch)
    ));
    println_flush(format_args!(
        "  rustc enables ({}): {}",
        if default_cpu {
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

    let contract = isa_contract(arch);
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
                arch_arg_name(arch),
                class.feature
            ));
        } else if banned_by_tier {
            failures.push(format!(
                "FORBIDDEN `{}` appears {count} times — the `{}` asset must not carry \
                 that source path, or the two assets are the same binary",
                class.name,
                arch_arg_name(arch)
            ));
        }
    }
    for class in contract.required {
        if count_of(class) == 0 {
            failures.push(format!(
                "REQUIRED `{class}` never appears — the `{}` tier is not actually \
                 emitting the path it is built for",
                arch_arg_name(arch)
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

/// The toolchain's own copy first: the raw profile format `llvm-profdata`
/// reads follows the LLVM that rustc was built with, so the sysroot copy
/// matches by construction and a PATH copy (an MSYS2 or system LLVM) only by
/// coincidence. An LLVM 22 `llvm-profdata` on PATH rejected rustc 1.99's
/// version-11 raw profiles outright.
fn find_llvm_tool(name: &str) -> Option<PathBuf> {
    find_rustup_llvm_tool(name).or_else(|| find_on_path(name))
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

/// Instrument, train on the bench suite, merge, and build with the profile;
/// returns the built binary. A universal tier also keeps its post-LTO object
/// and the link line, in [`link_line_path`] of its use build's directory.
fn build_with_pgo(config: &Config, purpose: Purpose) -> Result<PathBuf> {
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
        .join(format!(
            "{}{}",
            flavour_name(config.arch, config.native),
            purpose_tag(purpose)
        ));
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
    generate_flags.extend(purpose_flags(purpose));

    let gen_target_dir = target_dir(
        &format!("pgo-gen{}", purpose_tag(purpose)),
        config.arch,
        config.native,
        &config.target,
    );
    cargo_build(config, &gen_target_dir, &generate_flags, None)?;

    let instrumented = binary_path(&gen_target_dir, &config.target);
    run_training_bench(&instrumented, &raw_dir, config.bench_depth)?;

    let profdata = pgo_dir.join("rarog.profdata");
    merge_profiles(&llvm_profdata, &raw_dir, &profdata, &config.target)?;

    let mut use_flags = vec![
        "-C".to_string(),
        format!("profile-use={}", profdata.display()),
    ];
    use_flags.extend(rustflags(config.arch, config.native));
    use_flags.extend(purpose_flags(purpose));

    let use_target_dir = target_dir(
        &format!("pgo-use{}", purpose_tag(purpose)),
        config.arch,
        config.native,
        &config.target,
    );
    let keep = (purpose == Purpose::UniversalTier).then(|| link_line_path(&use_target_dir));
    cargo_build(config, &use_target_dir, &use_flags, keep.as_deref())?;
    Ok(binary_path(&use_target_dir, &config.target))
}

/// Keeps a universal tier's profile and build directories apart from the
/// ordinary build of the same tier, whose code is not aligned the same way.
fn purpose_tag(purpose: Purpose) -> &'static str {
    match purpose {
        Purpose::Binary => "",
        Purpose::UniversalTier => "-universal",
    }
}

/// Codegen flags a purpose adds to both PGO phases.
fn purpose_flags(purpose: Purpose) -> Vec<String> {
    match purpose {
        Purpose::Binary => Vec::new(),
        // Every function on a cache line, so the universal link's placement
        // cannot move a hot function off one (Basilisk's unaligned relink
        // read -0.58%). Applied to the training build too, so the profile is
        // of the same code.
        Purpose::UniversalTier => vec![
            "-C".to_string(),
            format!("llvm-args=-align-all-functions={}", universal::ALIGN_LOG2),
        ],
    }
}

/// Where a universal tier's use build writes rustc's printed link line.
fn link_line_path(use_target_dir: &Path) -> PathBuf {
    use_target_dir.join("link-line.txt")
}

/// `keep_link_line`: build the engine binary with `cargo rustc`, keeping
/// rustc's temporary files (the post-LTO object among them) and writing the
/// link line it prints to that path.
fn cargo_build(
    config: &Config,
    target_dir: &Path,
    override_flags: &[String],
    keep_link_line: Option<&Path>,
) -> Result<()> {
    let target = config.target.as_str();
    let mut flags = if override_flags.is_empty() {
        rustflags(config.arch, config.native)
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
        flavour_name(config.arch, config.native),
        target,
        if override_flags.is_empty() {
            ""
        } else {
            " with PGO flags"
        }
    ));

    let mut command = Command::new("cargo");
    command.arg(if keep_link_line.is_some() {
        "rustc"
    } else {
        "build"
    });
    command
        .arg("--release")
        .arg("--target")
        .arg(target)
        .arg("--target-dir")
        .arg(target_dir);
    if !config.features.is_empty() {
        command.arg("--features").arg(config.features.join(","));
    }
    if let Some(path) = keep_link_line {
        // Only the engine binary's own rustc links, so only it prints a line.
        command
            .args(["--bin", "rarog", "--", "-C", "save-temps", "--print"])
            .arg(format!("link-args={}", path.display()));
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
            "llvm-profdata was not found. Install it with `rustup component add llvm-tools` or add LLVM's bin directory to PATH."
                .to_string(),
        );
    }

    println_flush(format_args!("Installing llvm-tools for PGO support"));
    let status = Command::new("rustup")
        .arg("component")
        .arg("add")
        .arg("llvm-tools")
        .status()
        .map_err(|err| format!("failed to run rustup component add: {err}"))?;
    if !status.success() {
        return Err(
            "failed to install llvm-tools; run `rustup component add llvm-tools` manually"
                .to_string(),
        );
    }

    find_llvm_profdata()
        .ok_or_else(|| "llvm-profdata was still not found after installing llvm-tools".to_string())
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

/// Copy the build to `target/dist` under its name; returns the copy.
fn copy_dist_binary(binary: &Path, config: &Config, version: &str) -> Result<PathBuf> {
    if !binary.exists() {
        return Err(format!(
            "expected binary `{}` does not exist",
            binary.display()
        ));
    }
    let dist = PathBuf::from("target").join("dist");
    fs::create_dir_all(&dist)
        .map_err(|err| format!("failed to create `{}`: {err}", dist.display()))?;
    let asset = dist.join(asset_name(config, config.pgo, version));
    fs::copy(binary, &asset).map_err(|err| {
        format!(
            "failed to copy `{}` to `{}`: {err}",
            binary.display(),
            asset.display()
        )
    })?;
    println_flush(format_args!("Built {}", asset.display()));
    Ok(asset)
}

fn println_flush(args: std::fmt::Arguments<'_>) {
    println!("{args}");
    io::stdout().flush().expect("stdout flush failed");
}

/// `rarog-<version>-<os>-<x64|arm64>[-<tier>][-<kind>...]`: the version as
/// `id name` reports it, the tier word (`native` in its place), then the
/// kinds. A PGO build with the default features names no kind.
fn asset_name(config: &Config, pgo: bool, version: &str) -> String {
    let arch = if config.arch == Arch::Arm64 {
        "arm64"
    } else {
        "x64"
    };
    let mut name = format!("rarog-{version}-{}-{arch}", os_name(&config.target));
    let kinds = build_name::kinds(pgo, false, &config.feature_list());
    for word in config
        .tier_word()
        .into_iter()
        .chain(kinds.iter().map(String::as_str))
    {
        name.push('-');
        name.push_str(word);
    }
    name.push_str(exe_suffix(&config.target));
    name
}

/// Run `binary` with `input` on stdin, `RAROG_ISA` set to `forced` if given;
/// returns its exit status and stdout.
fn run_engine(binary: &Path, input: &str, forced: Option<&str>) -> Result<(i32, String)> {
    let mut command = Command::new(binary);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    match forced {
        Some(tier) => command.env(cpu_tier::FORCE_VARIABLE, tier),
        None => command.env_remove(cpu_tier::FORCE_VARIABLE),
    };
    let mut child = command
        .spawn()
        .map_err(|err| format!("failed to run `{}`: {err}", binary.display()))?;
    child
        .stdin
        .take()
        .ok_or("no stdin for the engine")?
        .write_all(input.as_bytes())
        .map_err(|err| format!("failed to talk to `{}`: {err}", binary.display()))?;
    let output = child
        .wait_with_output()
        .map_err(|err| format!("failed to read `{}`: {err}", binary.display()))?;
    Ok((
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    ))
}

/// Run the built engine and require it to report the name its file carries:
/// `id name Rarog <version>`, and `label` (the start-up line's tier and
/// kinds). A stale binary, or a `build.rs` that read the build differently,
/// fails here. `forced` sets the universal binary's tier.
fn check_reported_name(
    binary: &Path,
    version: &str,
    label: &str,
    forced: Option<&str>,
) -> Result<()> {
    let (_, stdout) = run_engine(binary, "uci\nquit\n", forced)?;
    let banner = stdout.lines().next().unwrap_or_default();
    let banner_prefix = format!("Rarog {version} by ");
    let banner_ok = banner.starts_with(&banner_prefix)
        && if label.is_empty() {
            !banner.ends_with(')')
        } else {
            banner.ends_with(&format!(" ({label})"))
        };
    let id_line = format!("id name Rarog {version}");
    let id_ok = stdout.lines().any(|line| line == id_line);
    if banner_ok && id_ok {
        println_flush(format_args!(
            "  reports `{id_line}`, start-up line `{banner}`"
        ));
        Ok(())
    } else {
        Err(format!(
            "`{}` does not report the name its file carries: expected `{id_line}` and a \
             start-up line `{banner_prefix}...{}`, got:\n{stdout}",
            binary.display(),
            if label.is_empty() {
                String::new()
            } else {
                format!(" ({label})")
            }
        ))
    }
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

    const ALL_ARCHES: [Arch; 6] = [
        Arch::Base,
        Arch::Sse42,
        Arch::Avx2,
        Arch::Pext,
        Arch::Universal,
        Arch::Arm64,
    ];
    const WINDOWS: &str = "x86_64-pc-windows-msvc";
    const LINUX: &str = "x86_64-unknown-linux-gnu";

    fn flags(arch: Arch, native: bool) -> String {
        rustflags(arch, native).join(" ")
    }

    fn config(arch: Arch, target: &str, native: bool, pgo: bool, features: &[&str]) -> Config {
        Config {
            command: CommandKind::Build,
            exe: None,
            default_cpu: false,
            arch,
            native,
            target: target.to_string(),
            pgo,
            bench_depth: 13,
            features: features.iter().map(|f| (*f).to_string()).collect(),
        }
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

    /// A TT prefetch compiled to nothing is invisible — the engine plays
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
        assert!(isa_contract(Arch::Sse42).also_forbidden.is_empty());
        assert!(isa_contract(Arch::Pext).also_forbidden.is_empty());
        assert!(isa_contract(Arch::Sse42).required.contains(&"popcnt"));
        assert!(isa_contract(Arch::Pext).required.contains(&"pext"));
    }

    /// `--arch` and `--native` are INDEPENDENT: a `native` arch that hardcoded
    /// the PEXT path would give a pre-BMI2 x86_64 host `_pext_u64` against a
    /// `target-cpu` that does not enable BMI2.
    #[test]
    fn native_is_orthogonal_to_arch() {
        // Non-PEXT archs must NEVER pull in the PEXT path or BMI2, native or not.
        for arch in [Arch::Base, Arch::Sse42, Arch::Avx2, Arch::Arm64] {
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
        for arch in ALL_ARCHES {
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
        assert_eq!(flags(Arch::Sse42, false), "-C target-cpu=x86-64-v2");
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
        for arch in ALL_ARCHES {
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

    /// The names the maintainer agreed (PLAN E.3.2): the version as `id name`
    /// gives it, `x64` or `arm64`, the tier word, then the kinds in order.
    #[test]
    fn file_names_follow_the_scheme() {
        let dev = "dev-20261104-209b555a";
        let name = |c: &Config, pgo: bool, v: &str| asset_name(c, pgo, v);
        assert_eq!(
            name(
                &config(Arch::Pext, WINDOWS, false, true, &[]),
                true,
                "2.7.0"
            ),
            "rarog-2.7.0-windows-x64-pext.exe"
        );
        assert_eq!(
            name(&config(Arch::Pext, WINDOWS, false, true, &[]), true, dev),
            "rarog-dev-20261104-209b555a-windows-x64-pext.exe"
        );
        assert_eq!(
            name(&config(Arch::Sse42, LINUX, false, true, &[]), true, "2.7.0"),
            "rarog-2.7.0-linux-x64-sse42"
        );
        assert_eq!(
            name(
                &config(Arch::Base, LINUX, false, false, &[]),
                false,
                "2.7.0"
            ),
            "rarog-2.7.0-linux-x64-base-nopgo"
        );
        assert_eq!(
            name(&config(Arch::Pext, WINDOWS, true, true, &[]), true, dev),
            "rarog-dev-20261104-209b555a-windows-x64-native.exe"
        );
        // Kinds keep their order whatever order they were asked in.
        assert_eq!(
            name(
                &config(Arch::Pext, WINDOWS, false, false, &["diag", "tune"]),
                false,
                dev
            ),
            "rarog-dev-20261104-209b555a-windows-x64-pext-nopgo-tune-diag.exe"
        );
        assert_eq!(
            name(
                &config(Arch::Arm64, "aarch64-apple-darwin", false, true, &[]),
                true,
                "2.7.0"
            ),
            "rarog-2.7.0-macos-arm64"
        );
        assert_eq!(
            name(
                &config(Arch::Arm64, "aarch64-pc-windows-msvc", true, true, &[]),
                true,
                "2.7.0"
            ),
            "rarog-2.7.0-windows-arm64-native.exe"
        );
        assert_eq!(
            name(
                &config(Arch::Universal, WINDOWS, false, true, &[]),
                true,
                "2.7.0"
            ),
            "rarog-2.7.0-windows-x64.exe"
        );
        assert_eq!(
            name(&config(Arch::Universal, LINUX, false, true, &[]), true, dev),
            "rarog-dev-20261104-209b555a-linux-x64"
        );
        // No file is named after a guessed release, and none carries a `v`.
        let release = name(
            &config(Arch::Avx2, WINDOWS, false, true, &[]),
            true,
            "2.7.0",
        );
        assert!(
            !release.contains("-v2") && !release.contains("pgo"),
            "{release}"
        );
    }

    /// The start-up line says what the file name says, in the same order.
    #[test]
    fn the_label_matches_the_file_name() {
        let c = config(Arch::Pext, WINDOWS, false, false, &["tune"]);
        assert_eq!(
            build_name::label(c.tier_word(), &c.kinds()),
            "pext, nopgo, tune"
        );
        let c = config(Arch::Arm64, "aarch64-apple-darwin", false, true, &[]);
        assert_eq!(build_name::label(c.tier_word(), &c.kinds()), "");
        let c = config(Arch::Avx2, WINDOWS, true, true, &[]);
        assert_eq!(build_name::label(c.tier_word(), &c.kinds()), "native");
    }

    #[test]
    fn a_tier_is_named_by_its_word_only() {
        assert_eq!(parse_tier("sse42").unwrap(), cpu_tier::Tier::Sse42);
        assert_eq!(parse_tier("pext").unwrap(), cpu_tier::Tier::Pext);
        for retired in ["x86-64", "bmi2", "native", "arm64", "PEXT"] {
            assert!(parse_tier(retired).is_err(), "{retired}");
        }
    }

    #[test]
    fn features_are_build_kinds() {
        assert_eq!(parse_features("tune").unwrap(), ["tune"]);
        assert_eq!(parse_features("diag, tune").unwrap(), ["diag", "tune"]);
        assert!(parse_features("").unwrap().is_empty());
        // A candidate arm's own feature is named after the engine's kinds.
        assert_eq!(parse_features("b2core").unwrap(), ["b2core"]);
        assert!(parse_features("B2-core").is_err());
        assert!(parse_features("default").is_err());
    }

    #[test]
    fn a_request_resolves_to_one_build() {
        let tier = |t| Request::Tier(t);
        // A named tier on an x86-64 target, from any host.
        assert_eq!(
            resolve_arch(tier(cpu_tier::Tier::Avx2), LINUX, WINDOWS).unwrap(),
            Arch::Avx2
        );
        assert_eq!(
            resolve_arch(Request::Universal, WINDOWS, WINDOWS).unwrap(),
            Arch::Universal
        );
        // An aarch64 target is arm64 and takes no tier, nor the universal binary.
        assert_eq!(
            resolve_arch(
                Request::Best,
                "aarch64-apple-darwin",
                "aarch64-apple-darwin"
            )
            .unwrap(),
            Arch::Arm64
        );
        assert!(resolve_arch(tier(cpu_tier::Tier::Pext), "aarch64-apple-darwin", WINDOWS).is_err());
        assert!(resolve_arch(Request::Universal, "aarch64-apple-darwin", WINDOWS).is_err());
        // A cross build names its tier; this host's CPU says nothing about it.
        assert!(resolve_arch(Request::Best, LINUX, WINDOWS).is_err());
        // Native builds for this host only.
        assert!(resolve_arch(Request::Native, LINUX, WINDOWS).is_err());
        assert!(resolve_arch(Request::Best, "riscv64gc-unknown-linux-gnu", WINDOWS).is_err());
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn the_default_build_is_this_hosts_best_tier() {
        let host = host_triple().unwrap();
        let best = host_best_tier().unwrap();
        assert_eq!(
            resolve_arch(Request::Best, &host, &host).unwrap(),
            Arch::from_tier(best)
        );
        assert_eq!(
            resolve_arch(Request::Native, &host, &host).unwrap(),
            Arch::from_tier(best)
        );
        assert!(host_runs(&config(Arch::from_tier(best), &host, false, true, &[])).unwrap());
        assert!(host_runs(&config(Arch::Base, &host, false, true, &[])).unwrap());
    }

    #[test]
    fn arch_and_target_must_agree() {
        assert!(ensure_arch_target_pair(Arch::Pext, WINDOWS).is_ok());
        assert!(ensure_arch_target_pair(Arch::Sse42, LINUX).is_ok());
        assert!(ensure_arch_target_pair(Arch::Pext, "aarch64-apple-darwin").is_err());
        assert!(ensure_arch_target_pair(Arch::Arm64, "aarch64-apple-darwin").is_ok());
        assert!(ensure_arch_target_pair(Arch::Arm64, LINUX).is_err());
    }

    /// Native and portable builds must not share PGO or intermediate dirs.
    #[test]
    fn native_builds_keep_their_own_directories() {
        assert_eq!(flavour_name(Arch::Pext, false), "pext");
        assert_eq!(flavour_name(Arch::Pext, true), "pext-native");
        assert_ne!(
            target_dir("pgo-gen", Arch::Pext, false, WINDOWS),
            target_dir("pgo-gen", Arch::Pext, true, WINDOWS),
            "native and portable PGO builds must not share an intermediate dir"
        );
    }

    // `src/build_name.rs` has no test target of its own in the engine, which
    // compiles it only into `build.rs`; its rules are tested here.

    #[test]
    fn a_release_reports_its_version_whatever_the_commit() {
        let dirty = build_name::DevId::Commit {
            date: "20261104".into(),
            sha: "209b555a".into(),
            modified: true,
        };
        assert_eq!(build_name::format_version("2.7.0", &dirty), "2.7.0");
        assert!(!build_name::is_development("2.7.0"));
        assert!(build_name::is_development("2.7.0-dev"));
    }

    #[test]
    fn a_development_build_reports_its_commit() {
        let commit = |modified| build_name::DevId::Commit {
            date: "20261104".into(),
            sha: "209b555a".into(),
            modified,
        };
        assert_eq!(
            build_name::format_version("2.7.0-dev", &commit(false)),
            "dev-20261104-209b555a"
        );
        assert_eq!(
            build_name::format_version("2.7.0-dev", &commit(true)),
            "dev-20261104-m-209b555a"
        );
        assert_eq!(
            build_name::format_version(
                "2.7.0-dev",
                &build_name::DevId::NoGit {
                    date: "20261104".into()
                }
            ),
            "dev-20261104-nogit"
        );
        let fixed = build_name::DevId::fixed("20261104-209b555a").unwrap();
        assert_eq!(
            build_name::format_version("2.7.0-dev", &fixed),
            "dev-20261104-209b555a"
        );
    }

    #[test]
    fn a_fixed_id_is_a_plain_word() {
        for bad in ["", "Bad", "has space", "20261104_209b555a", "v2.7.0"] {
            assert!(build_name::DevId::fixed(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn this_tree_reports_a_well_formed_name() {
        // Tests run in `xtask/`; the repository is its parent.
        let version = build_name::display_version("2.7.0-dev", Path::new("..")).unwrap();
        {
            let rest = version.strip_prefix("dev-").expect(&version);
            let (date, sha) = rest.split_once('-').expect(&version);
            assert!(
                date.len() == 8 && date.bytes().all(|b| b.is_ascii_digit()),
                "{version}"
            );
            let sha = sha.strip_prefix("m-").unwrap_or(sha);
            assert!(
                sha == "nogit" || (sha.len() == 8 && sha.bytes().all(|b| b.is_ascii_hexdigit())),
                "{version}"
            );
        }
    }

    #[test]
    fn kinds_keep_their_order() {
        assert!(build_name::kinds(true, false, &[]).is_empty());
        assert_eq!(build_name::kinds(false, false, &[]), ["nopgo"]);
        assert_eq!(
            build_name::kinds(false, true, &["texel", "diag", "tune"]),
            ["nopgo", "debug", "tune", "diag", "texel"]
        );
        // Other features follow, sorted, as a build script sees them.
        assert_eq!(
            build_name::kinds(true, false, &["zeta", "tune", "b2-core", "default"]),
            ["tune", "b2_core", "zeta"]
        );
        assert_eq!(
            build_name::label(Some("pext"), &["nopgo".to_string()]),
            "pext, nopgo"
        );
        assert_eq!(build_name::label(None, &[]), "");
    }

    #[test]
    fn the_calendar_is_right_at_its_edges() {
        assert_eq!(build_name::civil_from_days(0), (1970, 1, 1));
        assert_eq!(build_name::civil_from_days(59), (1970, 3, 1));
        assert_eq!(build_name::civil_from_days(11_016), (2000, 2, 29));
        assert_eq!(build_name::civil_from_days(20_761), (2026, 11, 4));
        assert_eq!(build_name::civil_from_days(20_819), (2027, 1, 1));
    }
}
