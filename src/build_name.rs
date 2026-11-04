//! The name a build reports, and the words that say what kind of build it is.
//!
//! Shared, by `#[path]`, between `build.rs`, which bakes the name into the
//! engine, and `xtask`, which names the file after it and checks the built
//! engine reports the same; its tests run with `xtask`'s. Standard library
//! only.
//!
//! A release reports its version, `2.7.0`. While `Cargo.toml` holds a `-dev`
//! version the next number is not known, so a development build reports its
//! commit instead, as Stockfish does: `dev-<commit date>-<8 hash characters>`,
//! with `-m` before the hash when the working tree differs from the commit.

use std::path::Path;
use std::process::{Command, Stdio};

/// Environment variable that fixes a development build's id, for comparing
/// two builds byte for byte: the name is part of the binary.
pub const DEV_ID_VARIABLE: &str = "RAROG_DEV_ID";

/// The engine's own feature kinds, in the order a name lists them. Any other
/// feature (a candidate arm compiled behind a flag) follows them, sorted.
pub const FEATURE_KINDS: [&str; 3] = ["tune", "diag", "texel"];

/// Where a development build's id comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevId {
    /// Given by [`DEV_ID_VARIABLE`].
    Fixed(String),
    /// The checked-out commit: its date (`yyyymmdd`), the first eight
    /// characters of its hash, and whether the tree differs from it.
    Commit {
        date: String,
        sha: String,
        modified: bool,
    },
    /// No git, or no repository: the build date.
    NoGit { date: String },
}

/// Is `package_version` a development version?
pub fn is_development(package_version: &str) -> bool {
    package_version.ends_with("-dev")
}

/// The version a build in `repo` reports: what `id name` and file names use.
pub fn display_version(package_version: &str, repo: &Path) -> Result<String, String> {
    if !is_development(package_version) {
        return Ok(package_version.to_string());
    }
    Ok(format_version(package_version, &DevId::read(repo)?))
}

/// The version for `package_version`, given a development build's id.
pub fn format_version(package_version: &str, dev: &DevId) -> String {
    if !is_development(package_version) {
        return package_version.to_string();
    }
    match dev {
        DevId::Fixed(id) => format!("dev-{id}"),
        DevId::Commit {
            date,
            sha,
            modified: false,
        } => format!("dev-{date}-{sha}"),
        DevId::Commit {
            date,
            sha,
            modified: true,
        } => format!("dev-{date}-m-{sha}"),
        DevId::NoGit { date } => format!("dev-{date}-nogit"),
    }
}

impl DevId {
    /// The id from [`DEV_ID_VARIABLE`] if set, else from git in `repo`.
    pub fn read(repo: &Path) -> Result<DevId, String> {
        if let Ok(fixed) = std::env::var(DEV_ID_VARIABLE) {
            return DevId::fixed(&fixed);
        }
        let git = |args: &[&str]| -> Option<String> {
            let output = Command::new("git")
                .args(args)
                .current_dir(repo)
                .stderr(Stdio::null())
                .output()
                .ok()?;
            output
                .status
                .success()
                .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        };
        let sha = git(&["rev-parse", "HEAD"])
            .filter(|sha| sha.len() >= 8 && sha.bytes().all(|byte| byte.is_ascii_hexdigit()));
        let date = git(&["show", "-s", "--date=format:%Y%m%d", "--format=%cd", "HEAD"])
            .filter(|date| date.len() == 8 && date.bytes().all(|byte| byte.is_ascii_digit()));
        let (Some(sha), Some(date)) = (sha, date) else {
            return Ok(DevId::NoGit { date: today_utc() });
        };
        // Refresh the index's file stamps first, so a file whose bytes match
        // the commit but whose stamp moved (a checkout, a CRLF rewrite) does
        // not read as modified.
        let _ = git(&["update-index", "-q", "--refresh"]);
        let modified = Command::new("git")
            .args(["diff-index", "--quiet", "HEAD", "--"])
            .current_dir(repo)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.code() == Some(1));
        Ok(DevId::Commit {
            date,
            sha: sha[..8].to_string(),
            modified,
        })
    }

    /// A fixed id: lower-case letters, digits and dashes.
    pub fn fixed(id: &str) -> Result<DevId, String> {
        let valid = !id.is_empty()
            && id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if valid {
            Ok(DevId::Fixed(id.to_string()))
        } else {
            Err(format!(
                "{DEV_ID_VARIABLE}=`{id}`: a development id is lower-case letters, digits and dashes, \
                 such as 20261104-209b555a"
            ))
        }
    }
}

/// The kinds a build carries besides its tier, in the order a name lists
/// them: `nopgo` for anything but a PGO build, `debug` for an unoptimised
/// one, the engine's feature kinds that `features` turns on, then its other
/// features, sorted. A feature is written as cargo shows it to a build
/// script: lower case, `-` as `_`.
pub fn kinds(pgo: bool, debug: bool, features: &[&str]) -> Vec<String> {
    let mut kinds = Vec::new();
    if !pgo {
        kinds.push("nopgo".to_string());
    }
    if debug {
        kinds.push("debug".to_string());
    }
    let mut features: Vec<String> = features
        .iter()
        .map(|feature| feature.to_lowercase().replace('-', "_"))
        .filter(|feature| feature != "default")
        .collect();
    features.sort();
    features.dedup();
    for kind in FEATURE_KINDS {
        if let Some(at) = features.iter().position(|feature| feature == kind) {
            kinds.push(features.remove(at));
        }
    }
    kinds.extend(features);
    kinds
}

/// What the start-up line says in parentheses: the tier, then the kinds.
/// Empty for a PGO build with the default features and no tier (arm64).
pub fn label(tier: Option<&str>, kinds: &[String]) -> String {
    tier.into_iter()
        .chain(kinds.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Today's date in UTC, `yyyymmdd`.
fn today_utc() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let (year, month, day) = civil_from_days(seconds / 86_400);
    format!("{year:04}{month:02}{day:02}")
}

/// The proleptic Gregorian date `days` after 1970-01-01 (Howard Hinnant's
/// `civil_from_days`, for non-negative days).
pub fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let day_of_era = z % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month, day)
}
