//! The universal x86-64 binary (PLAN E.3.3, `analysis/universal_binary_2026-09.md`).
//!
//! One executable holds four complete engines, one per tier, behind a
//! dispatcher compiled for the baseline (`universal/dispatch.rs`):
//!
//! 1. Each tier is its ordinary PGO build, with every function aligned to 64
//!    bytes, and rustc keeps its post-LTO object and prints its link line.
//!    The object is the measured build's own code: relinked alone it gives
//!    the same text byte for byte.
//! 2. After PGO, every global symbol a tier defines is renamed
//!    `rarog_<tier>_<name>`, so no tier can resolve to another's copy and no
//!    profile is ever matched against a renamed symbol. Only COMDAT constants
//!    named by their own value stay shared; their bytes are their name.
//! 3. rustc links the dispatcher with the four renamed objects, one Fathom
//!    and the libraries of the base tier's own link line, all at the
//!    baseline.
//!
//! Each tier object, Fathom and the dispatcher are held to their own ISA
//! contract before the link, and the image to the union after it; every
//! forced tier is benched for one fingerprint (`check`).

use super::{
    Arch, Config, Purpose, Result, binary_path, build_with_pgo, check_isa, check_reported_name,
    cpu_tier, exe_suffix, find_llvm_tool, link_line_path, println_flush, run_engine, sanitize,
    target_dir,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every tier function starts on a `1 << ALIGN_LOG2`-byte line.
pub(super) const ALIGN_LOG2: u32 = 6;

/// The engine's hot path, which must be aligned whatever else is not.
const HOT_FUNCTIONS: [&str; 10] = [
    "negamax",
    "quiescence",
    "search_root",
    "pick_next",
    "make_move",
    "unmake_move",
    "is_legal",
    "see_ge",
    "evaluate",
    "see",
];

/// Code sections a tier object may leave below the line: the odd cold
/// helper LLVM places in the object's plain `.text` (one, 132 bytes, the
/// drop of a panic payload, on 2026-11-04).
const UNALIGNED_ALLOWANCE: usize = 2;

/// Build the four tiers, rename them and link them behind the dispatcher;
/// returns the universal executable.
pub(super) fn build(config: &Config, version: &str) -> Result<PathBuf> {
    let level = cpu_tier::host_level().ok_or("the universal binary is built on x86-64")?;
    if level < cpu_tier::Level::V3 {
        return Err(
            "the universal binary trains every tier by running it, so it is built on an \
             x86-64-v3 CPU"
                .to_string(),
        );
    }
    let dir = PathBuf::from("target")
        .join("universal")
        .join(sanitize(&config.target));
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|err| format!("failed to clear `{}`: {err}", dir.display()))?;
    }
    fs::create_dir_all(&dir)
        .map_err(|err| format!("failed to create `{}`: {err}", dir.display()))?;

    let mut objects = Vec::new();
    let mut base_line = None;
    for tier in cpu_tier::Tier::ALL {
        println_flush(format_args!(
            "== universal binary: the {} tier",
            tier.name()
        ));
        let tier_config = Config {
            arch: Arch::from_tier(tier),
            native: false,
            ..config.clone()
        };
        let binary = build_with_pgo(&tier_config, Purpose::UniversalTier)?;
        check_reported_name(&binary, version, tier.name(), None)?;
        let use_dir = target_dir("pgo-use-universal", tier_config.arch, false, &config.target);
        if binary != binary_path(&use_dir, &config.target) {
            return Err(format!(
                "the {} tier was built at `{}`, not in its universal use directory",
                tier.name(),
                binary.display()
            ));
        }
        let line = read_link_line(&link_line_path(&use_dir))?;
        let object = post_lto_object(&line)?;
        check_alignment(&object)?;
        check_initialisers(&object)?;
        check_isa(&object, tier_config.arch, &config.target, false)?;
        objects.push(rename_tier(&object, tier, &dir)?);
        if tier == cpu_tier::Tier::Base {
            base_line = Some(line);
        }
    }
    let base_line = base_line.ok_or("the base tier left no link line")?;
    let shared = SharedInputs::from_base_line(&base_line)?;
    // Fathom is the base tier's, built at the baseline; it is linked once.
    check_isa(&shared.engine_archive, Arch::Base, &config.target, false)?;
    link(config, &dir, &objects, &shared)
}

/// The universal binary's checks, run on the file that ships: the tier it
/// picks here and the name it reports, every tier this CPU runs forced and
/// benched for one fingerprint, a tier name it does not know refused.
pub(super) fn check(binary: &Path, version: &str) -> Result<()> {
    let level = cpu_tier::host_level().ok_or("the universal binary runs on x86-64")?;
    let best = cpu_tier::universal_tier(Some(level), cpu_tier::host_has_slow_pext(), None)?;
    check_reported_name(binary, version, best.name(), None)?;
    let mut fingerprints = Vec::new();
    for tier in cpu_tier::Tier::ALL {
        if tier.level() > level {
            println_flush(format_args!(
                "  note: this CPU cannot run the {} tier, so it is not forced here",
                tier.name()
            ));
            continue;
        }
        check_reported_name(binary, version, tier.name(), Some(tier.name()))?;
        let (status, out) = run_engine_args(binary, &["bench", "13"], tier.name())?;
        let nodes = out
            .lines()
            .find_map(|line| line.strip_prefix("Nodes searched"))
            .and_then(|rest| {
                rest.trim_start_matches([' ', ':'])
                    .trim()
                    .parse::<u64>()
                    .ok()
            })
            .ok_or_else(|| format!("the forced {} tier benched nothing:\n{out}", tier.name()))?;
        if status != 0 {
            return Err(format!(
                "the forced {} tier exited {status} in bench",
                tier.name()
            ));
        }
        println_flush(format_args!("  {} tier: bench 13 {nodes}", tier.name()));
        fingerprints.push((tier.name(), nodes));
    }
    if fingerprints.windows(2).any(|pair| pair[0].1 != pair[1].1) {
        return Err(format!("the tiers disagree on bench 13: {fingerprints:?}"));
    }
    let (status, out) = run_engine(binary, "uci\nquit\n", Some("none"))?;
    if status != 1 || !out.contains("info string") {
        return Err(format!(
            "an unknown forced tier was not refused (exit {status}):\n{out}"
        ));
    }
    println_flush(format_args!(
        "  an unknown forced tier is refused with exit 1"
    ));
    Ok(())
}

/// `run_engine` with arguments instead of stdin input.
fn run_engine_args(binary: &Path, args: &[&str], forced: &str) -> Result<(i32, String)> {
    let output = Command::new(binary)
        .args(args)
        .env(cpu_tier::FORCE_VARIABLE, forced)
        .output()
        .map_err(|err| format!("failed to run `{}`: {err}", binary.display()))?;
    Ok((
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    ))
}

/// rustc's printed link line, one argument per element. rustc writes each
/// argument quoted with Rust's string escapes.
fn read_link_line(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("no link line at `{}`: {err}", path.display()))?;
    let args = parse_link_line(&text);
    if args.is_empty() {
        return Err(format!("`{}` holds no link line", path.display()));
    }
    Ok(args)
}

fn parse_link_line(text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }
        let mut arg = String::new();
        while let Some(ch) = chars.next() {
            match ch {
                '"' => break,
                '\\' => match chars.next() {
                    Some('n') => arg.push('\n'),
                    Some('t') => arg.push('\t'),
                    Some(other) => arg.push(other),
                    None => break,
                },
                other => arg.push(other),
            }
        }
        args.push(arg);
    }
    args
}

/// The one post-LTO object a fat-LTO, one-unit build links.
fn post_lto_object(line: &[String]) -> Result<PathBuf> {
    let objects: Vec<&String> = line
        .iter()
        .filter(|arg| arg.ends_with("-cgu.0.rcgu.o"))
        .collect();
    match objects.as_slice() {
        [object] => {
            let path = PathBuf::from(object.as_str());
            if path.is_file() {
                Ok(path)
            } else {
                Err(format!("the post-LTO object `{object}` was not kept"))
            }
        }
        _ => Err(format!(
            "expected one post-LTO object in the link line, found {}",
            objects.len()
        )),
    }
}

/// What the universal link takes from the base tier's own link line: its
/// engine archive (which carries Fathom, built at the baseline), the
/// compiler builtins, the system libraries, and whether cargo had the linker
/// strip debug information.
struct SharedInputs {
    engine_archive: PathBuf,
    archives: Vec<String>,
    libraries: Vec<String>,
    /// Cargo's release default strips the debug information std brings
    /// (`-Wl,--strip-debug` on Linux; nothing on MSVC, whose PDB is
    /// separate). Unstripped, std's copy in each of five objects made the
    /// Linux binary 14 MB.
    strip_debuginfo: bool,
}

impl SharedInputs {
    fn from_base_line(line: &[String]) -> Result<SharedInputs> {
        let file_name = |arg: &str| {
            Path::new(arg)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        let engine: Vec<&String> = line
            .iter()
            .filter(|arg| {
                let name = file_name(arg);
                name.starts_with("librarog-") && name.ends_with(".rlib")
            })
            .collect();
        let [engine_archive] = engine.as_slice() else {
            return Err(format!(
                "expected the engine's archive once in the base link line, found {}",
                engine.len()
            ));
        };
        // The builtins, and std's archive of import stubs (Windows'
        // `raw-dylib` imports), which the tiers' std copies call through.
        let archives = line
            .iter()
            .filter(|arg| {
                let name = file_name(arg);
                name.starts_with("libcompiler_builtins-")
                    || (name.starts_with("libstd-") && name.ends_with(".rlib"))
            })
            .cloned()
            .collect();
        let mut libraries: Vec<String> = line
            .iter()
            .filter(|arg| {
                arg.to_ascii_lowercase().ends_with(".lib")
                    || (arg.starts_with("-l") && arg.len() > 2)
            })
            .cloned()
            .collect();
        libraries.dedup();
        Ok(SharedInputs {
            engine_archive: PathBuf::from(engine_archive.as_str()),
            archives,
            libraries,
            strip_debuginfo: line.iter().any(|arg| arg == "-Wl,--strip-debug"),
        })
    }
}

/// The value-named COMDAT constants a tier may share: `__real@3ff0…` holds
/// exactly the bytes its name spells, in every tier.
fn is_value_named_constant(name: &str) -> bool {
    ["__real@", "__xmm@", "__ymm@", "__zmm@"]
        .iter()
        .any(|prefix| {
            name.strip_prefix(prefix)
                .is_some_and(|hex| !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        })
}

/// The tier's name for one of its symbols.
fn tier_symbol(tier: cpu_tier::Tier, name: &str) -> String {
    format!("rarog_{}_{name}", tier.name())
}

fn llvm_tool(name: &str) -> Result<PathBuf> {
    find_llvm_tool(name).ok_or_else(|| {
        format!("{name} not found. Install it with `rustup component add llvm-tools`.")
    })
}

fn tool_output(tool: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(tool)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run `{}`: {err}", tool.display()))?;
    if !output.status.success() {
        return Err(format!(
            "`{}` failed: {}",
            tool.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Every symbol `object` defines globally.
fn defined_globals(object: &Path) -> Result<Vec<String>> {
    let nm = llvm_tool("llvm-nm")?;
    let path = object.to_str().ok_or("object path is not UTF-8")?;
    Ok(tool_output(
        &nm,
        &[
            "--defined-only",
            "--extern-only",
            "--format=just-symbols",
            path,
        ],
    )?
    .lines()
    .map(str::trim)
    .filter(|name| !name.is_empty())
    .map(str::to_string)
    .collect())
}

/// Rename every global a tier object defines to the tier's own name, then
/// require that nothing else stayed global and that its entry is there.
fn rename_tier(object: &Path, tier: cpu_tier::Tier, dir: &Path) -> Result<PathBuf> {
    let names = defined_globals(object)?;
    let private: Vec<&String> = names
        .iter()
        .filter(|name| !is_value_named_constant(name))
        .collect();
    let map = dir.join(format!("{}.symbols", tier.name()));
    fs::write(
        &map,
        private
            .iter()
            .map(|name| format!("{name} {}\n", tier_symbol(tier, name)))
            .collect::<String>(),
    )
    .map_err(|err| format!("failed to write `{}`: {err}", map.display()))?;
    let renamed = dir.join(format!("{}.o", tier.name()));
    let objcopy = llvm_tool("llvm-objcopy")?;
    tool_output(
        &objcopy,
        &[
            &format!("--redefine-syms={}", map.display()),
            object.to_str().ok_or("object path is not UTF-8")?,
            renamed.to_str().ok_or("object path is not UTF-8")?,
        ],
    )?;
    let after = defined_globals(&renamed)?;
    let prefix = tier_symbol(tier, "");
    let leftover: Vec<&String> = after
        .iter()
        .filter(|name| !is_value_named_constant(name) && !name.starts_with(&prefix))
        .collect();
    if !leftover.is_empty() {
        return Err(format!(
            "the {} tier still defines {leftover:?} under a name another tier can take",
            tier.name()
        ));
    }
    let entry = tier_symbol(tier, "main");
    if !after.contains(&entry) {
        return Err(format!("the {} tier defines no `{entry}`", tier.name()));
    }
    println_flush(format_args!(
        "  {} tier: {} globals renamed ({}), {} value-named constants shared",
        tier.name(),
        private.len(),
        private
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        after.len() - private.len()
    ));
    Ok(renamed)
}

/// One section of an object, as `llvm-readobj --sections` describes it.
#[derive(Debug, Default)]
struct Section {
    number: u32,
    name: String,
    size: u64,
    code: bool,
    alignment: u64,
}

fn sections(object: &Path) -> Result<Vec<Section>> {
    let readobj = llvm_tool("llvm-readobj")?;
    let text = tool_output(
        &readobj,
        &[
            "--sections",
            object.to_str().ok_or("object path is not UTF-8")?,
        ],
    )?;
    let mut sections = Vec::new();
    let mut current: Option<Section> = None;
    for line in text.lines().map(str::trim) {
        if line == "Section {" {
            sections.extend(current.take());
            current = Some(Section::default());
            continue;
        }
        let Some(section) = current.as_mut() else {
            continue;
        };
        let value = |key: &str| line.strip_prefix(key).map(str::trim);
        if let Some(number) = value("Number:").or_else(|| value("Index:")) {
            section.number = number.parse().unwrap_or(0);
        } else if let Some(name) = value("Name:") {
            section.name = name.split(" (").next().unwrap_or(name).to_string();
        } else if let Some(size) = value("RawDataSize:").or_else(|| value("Size:")) {
            section.size = size.parse().unwrap_or(0);
        } else if let Some(align) = value("AddressAlignment:") {
            section.alignment = align.parse().unwrap_or(0);
        } else if line.starts_with("IMAGE_SCN_CNT_CODE") || line.starts_with("SHF_EXECINSTR") {
            section.code = true;
        } else if let Some(rest) = line.strip_prefix("IMAGE_SCN_ALIGN_") {
            section.alignment = rest
                .split("BYTES")
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
        }
    }
    sections.extend(current);
    Ok(sections)
}

/// Every function of the tier on a cache line: at most
/// [`UNALIGNED_ALLOWANCE`] code sections below it, none of them hot.
fn check_alignment(object: &Path) -> Result<()> {
    let line = 1u64 << ALIGN_LOG2;
    let code: Vec<Section> = sections(object)?
        .into_iter()
        .filter(|section| section.code && section.size > 0)
        .collect();
    let below: Vec<&Section> = code
        .iter()
        .filter(|section| section.alignment < line)
        .collect();
    let objdump = llvm_tool("llvm-objdump")?;
    let table = tool_output(
        &objdump,
        &["-t", object.to_str().ok_or("object path is not UTF-8")?],
    )?;
    let mut hot_below = Vec::new();
    for section in &below {
        let marker = format!("(sec{:>3})", section.number);
        for symbol in table.lines().filter(|row| row.contains(&marker)) {
            let name = symbol.split_whitespace().last().unwrap_or_default();
            if HOT_FUNCTIONS
                .iter()
                .any(|hot| name.contains(&format!("{}{hot}", hot.len())))
            {
                hot_below.push(name.to_string());
            }
        }
    }
    if !hot_below.is_empty() || below.len() > UNALIGNED_ALLOWANCE {
        return Err(format!(
            "`{}`: {} of {} code sections are below {line}-byte alignment (allowed \
             {UNALIGNED_ALLOWANCE}, none hot); hot ones: {hot_below:?}",
            object.display(),
            below.len(),
            code.len()
        ));
    }
    println_flush(format_args!(
        "  {} of {} code sections on a {line}-byte line",
        code.len() - below.len(),
        code.len()
    ));
    Ok(())
}

/// A tier's initialisers run at load, before the dispatcher has chosen, in
/// every tier. Windows: none may exist. ELF: only `.init_array`, std's
/// capture of `argc`/`argv` into the tier's own copy, which is harmless run
/// four times.
fn check_initialisers(object: &Path) -> Result<()> {
    let found: Vec<String> = sections(object)?
        .into_iter()
        .map(|section| section.name)
        .filter(|name| {
            name.starts_with(".CRT$X")
                || name.starts_with(".init_array")
                || name.starts_with(".preinit_array")
                || name.starts_with(".ctors")
                || name.starts_with(".fini_array")
                || name.starts_with(".dtors")
        })
        .collect();
    let refused: Vec<&String> = found
        .iter()
        .filter(|name| !name.starts_with(".init_array"))
        .collect();
    if !refused.is_empty() {
        return Err(format!(
            "`{}` carries initialisers that would run in every tier: {refused:?}",
            object.display()
        ));
    }
    if !found.is_empty() {
        println_flush(format_args!(
            "  initialisers kept (std's argument capture): {found:?}"
        ));
    }
    Ok(())
}

/// Link the dispatcher with the renamed tiers. rustc does the link, so the
/// platform's linker and its environment are the ones every build uses.
fn link(
    config: &Config,
    dir: &Path,
    objects: &[PathBuf],
    shared: &SharedInputs,
) -> Result<PathBuf> {
    let exe = dir.join(format!("rarog{}", exe_suffix(&config.target)));
    let dispatcher_object = dir.join("dispatch.o");
    let mut command = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()));
    command
        .arg("universal/dispatch.rs")
        .args([
            "--edition",
            "2024",
            "--crate-name",
            "rarog",
            "--crate-type",
            "bin",
        ])
        .args(["--target", &config.target])
        .args([
            "-C",
            "opt-level=3",
            "-C",
            "lto=fat",
            "-C",
            "codegen-units=1",
        ])
        .args(["-C", "panic=abort", "-D", "warnings"])
        .args(super::rustflags(Arch::Universal, false))
        .args(if shared.strip_debuginfo {
            &["-C", "strip=debuginfo"][..]
        } else {
            &[]
        })
        .arg(format!(
            "--emit=obj={},link={}",
            dispatcher_object.display(),
            exe.display()
        ));
    for input in objects
        .iter()
        .map(|object| object.display().to_string())
        .chain([shared.engine_archive.display().to_string()])
        .chain(shared.archives.iter().cloned())
        .chain(shared.libraries.iter().cloned())
    {
        command.arg("-C").arg(format!("link-arg={input}"));
    }
    println_flush(format_args!(
        "Linking the universal binary: the dispatcher, {} tiers, Fathom and {} libraries",
        objects.len(),
        shared.libraries.len()
    ));
    let status = command
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .status()
        .map_err(|err| format!("failed to run rustc: {err}"))?;
    if !status.success() {
        return Err(format!("the universal link failed with status {status}"));
    }
    // The dispatcher is the code that runs before any tier is chosen.
    check_isa(&dispatcher_object, Arch::Base, &config.target, false)?;
    check_isa(&exe, Arch::Universal, &config.target, false)?;
    Ok(exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_line_is_read_with_rusts_escapes() {
        let line = r#""C:\\VC\\link.exe" "/NOLOGO" "D:\\t\\deps\\rarog.rarog.ab-cgu.0.rcgu.o" "kernel32.lib" "/OUT:D:\\t\\rarog.exe""#;
        let args = parse_link_line(line);
        assert_eq!(args[0], r"C:\VC\link.exe");
        assert_eq!(args[2], r"D:\t\deps\rarog.rarog.ab-cgu.0.rcgu.o");
        assert_eq!(args.len(), 5);
        let unix = r#""cc" "-m64" "/t/deps/rarog.rarog.ab-cgu.0.rcgu.o" "-lgcc_s" "-o" "/t/rarog""#;
        assert_eq!(parse_link_line(unix)[3], "-lgcc_s");
    }

    #[test]
    fn the_shared_inputs_come_from_the_base_line() {
        let line: Vec<String> = [
            r"C:\VC\link.exe",
            r"D:\t\deps\rarog.rarog.ab-cgu.0.rcgu.o",
            r"D:\t\deps\rustcAb\librarog-6a3bb779a354820a.rlib",
            r"D:\t\deps\rustcAb\libstd-38.rlib",
            r"C:\sysroot\lib\libcompiler_builtins-99.rlib",
            "kernel32.lib",
            "advapi32.lib",
            "/OPT:REF,ICF",
        ]
        .map(String::from)
        .to_vec();
        let shared = SharedInputs::from_base_line(&line).unwrap();
        assert!(
            shared
                .engine_archive
                .ends_with("librarog-6a3bb779a354820a.rlib")
        );
        assert_eq!(shared.archives.len(), 2);
        assert_eq!(shared.libraries, ["kernel32.lib", "advapi32.lib"]);
        // MSVC's line strips nothing, so neither does the universal link.
        assert!(!shared.strip_debuginfo);
        let mut unix = line.clone();
        unix.push("-Wl,--strip-debug".to_string());
        assert!(SharedInputs::from_base_line(&unix).unwrap().strip_debuginfo);
        let two_engines = [line.clone(), line].concat();
        assert!(SharedInputs::from_base_line(&two_engines).is_err());
    }

    #[test]
    fn only_value_named_constants_stay_shared() {
        assert!(is_value_named_constant("__real@3ff0000000000000"));
        assert!(is_value_named_constant(
            "__xmm@0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f"
        ));
        for private in [
            "main",
            "rust_eh_personality",
            "__real@",
            "__real@xyz",
            "__realloc",
        ] {
            assert!(!is_value_named_constant(private), "{private}");
        }
        assert_eq!(
            tier_symbol(cpu_tier::Tier::Sse42, "main"),
            "rarog_sse42_main"
        );
    }

    #[test]
    fn exactly_one_post_lto_object_is_taken() {
        let one = vec!["a.o".to_string(), "x-cgu.0.rcgu.o".to_string()];
        assert!(post_lto_object(&one).is_err(), "it does not exist on disk");
        let none = vec!["a.o".to_string()];
        assert!(post_lto_object(&none).is_err());
    }
}
