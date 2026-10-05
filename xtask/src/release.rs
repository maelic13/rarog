//! `cargo xtask release-check vX.Y.Z [--notes <path>] [--base <ref>]`
//!
//! The checks a release tag must pass before anything is built or published,
//! runnable locally before the tag is pushed and run first by the release
//! workflow: the tag names `Cargo.toml`'s version, the tagged commit is on
//! the default branch, `CHANGELOG.md` has a dated section for that version,
//! and GUIDE's checkpoint marks that version released at the fingerprint it
//! declares. With `--notes`, that section's body is written out as the
//! release notes, so the GitHub form is never typed into.
use std::fs;
use std::path::Path;
use std::process::Command;

type Result<T> = std::result::Result<T, String>;

pub struct ReleaseCheck {
    pub tag: String,
    pub notes: Option<std::path::PathBuf>,
    /// The branch a release must be reachable from, as a git ref.
    pub base: String,
}

pub fn run(check: &ReleaseCheck) -> Result<()> {
    let version = version_of_tag(&check.tag)?;
    let manifest = fs::read_to_string("Cargo.toml").map_err(|e| format!("Cargo.toml: {e}"))?;
    let declared = manifest_version(&manifest)?;
    if declared != version {
        return Err(format!(
            "tag {} names version {version}, but Cargo.toml declares {declared}",
            check.tag
        ));
    }
    let changelog = fs::read_to_string("CHANGELOG.md").map_err(|e| format!("CHANGELOG.md: {e}"))?;
    let body = changelog_section(&changelog, &version)?;
    let guide = fs::read_to_string("GUIDE.md").map_err(|e| format!("GUIDE.md: {e}"))?;
    let fingerprint = release_marked_in_guide(&guide, &version)?;
    ensure_on_base(&check.base)?;
    if let Some(path) = &check.notes {
        fs::write(path, &body).map_err(|e| format!("{}: {e}", path.display()))?;
        println!(
            "release notes for {version}: {} bytes to {}",
            body.len(),
            path.display()
        );
    }
    println!(
        "release-check {}: version {version} in Cargo.toml, CHANGELOG section present, \
         GUIDE marks it released at bench {fingerprint}, HEAD on {}",
        check.tag, check.base
    );
    Ok(())
}

/// `vX.Y.Z` and nothing else: no pre-release suffix, no `arm/` or `oracle/`
/// marker, no bare version.
pub fn version_of_tag(tag: &str) -> Result<String> {
    let version = tag
        .strip_prefix('v')
        .ok_or_else(|| format!("tag `{tag}` must read vX.Y.Z"))?;
    let parts: Vec<&str> = version.split('.').collect();
    let numeric = parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
    if !numeric {
        return Err(format!("tag `{tag}` must read vX.Y.Z with three numbers"));
    }
    Ok(version.to_string())
}

/// The `version = "..."` of the first `[package]` in a manifest.
pub fn manifest_version(manifest: &str) -> Result<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if in_package && let Some(rest) = line.strip_prefix("version") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                return Ok(rest.trim().trim_matches('"').to_string());
            }
        }
    }
    Err("Cargo.toml has no [package] version".to_string())
}

/// The body of `## [version] - YYYY-MM-DD`, up to the next `## ` heading,
/// trimmed, with one trailing newline. An undated or missing section is an
/// error: the changelog is closed before the tag, never after.
pub fn changelog_section(changelog: &str, version: &str) -> Result<String> {
    let heading = format!("## [{version}] - ");
    let lines: Vec<&str> = changelog.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.starts_with(&heading) && is_date(&l[heading.len()..]))
        .ok_or_else(|| {
            format!("CHANGELOG.md has no `## [{version}] - YYYY-MM-DD` section; close [Unreleased] first")
        })?;
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.starts_with("## "))
        .map_or(lines.len(), |n| start + 1 + n);
    let body = lines[start + 1..end].join("\n").trim().to_string();
    if body.is_empty() {
        return Err(format!("the CHANGELOG section for {version} is empty"));
    }
    Ok(body + "\n")
}

fn is_date(s: &str) -> bool {
    let s = s.trim();
    s.len() == 10
        && s.bytes().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        })
}

/// The bench fingerprint GUIDE's checkpoint declares for the source in this
/// commit: the `fingerprint **N` of its Development head row. The Released
/// baseline row above it names the last release, which a candidate carrying
/// a new fingerprint does not match.
pub fn declared_fingerprint(guide: &str) -> Result<u64> {
    fingerprint_in(guide_row(guide, "Development head")?, "Development head")
}

/// A release commit marks its own version released: GUIDE's Released
/// baseline row names that version, at the fingerprint the Development head
/// row declares. Returns that fingerprint.
pub fn release_marked_in_guide(guide: &str, version: &str) -> Result<u64> {
    let declared = declared_fingerprint(guide)?;
    let row = guide_row(guide, "Released baseline")?;
    let released = bold_versions(row)
        .next()
        .ok_or_else(|| "GUIDE's Released baseline row names no **X.Y.Z** version".to_string())?;
    if released != version {
        return Err(format!(
            "GUIDE's Released baseline row names {released}; the release commit must mark {version} released"
        ));
    }
    let released_nodes = fingerprint_in(row, "Released baseline")?;
    if released_nodes != declared {
        return Err(format!(
            "GUIDE's Released baseline fingerprint {released_nodes} differs from the Development head's {declared}"
        ));
    }
    Ok(declared)
}

/// The one line of GUIDE's checkpoint table that starts `| <name> |`.
fn guide_row<'a>(guide: &'a str, name: &str) -> Result<&'a str> {
    let prefix = format!("| {name} |");
    let mut rows = guide.lines().filter(|l| l.starts_with(&prefix));
    let row = rows
        .next()
        .ok_or_else(|| format!("GUIDE.md has no `{prefix}` checkpoint row"))?;
    if rows.next().is_some() {
        return Err(format!("GUIDE.md has more than one `{prefix}` row"));
    }
    Ok(row)
}

/// The number after the row's first `fingerprint **`, thousands commas removed.
fn fingerprint_in(row: &str, name: &str) -> Result<u64> {
    let marker = "fingerprint **";
    let start = row
        .find(marker)
        .ok_or_else(|| format!("GUIDE's {name} row carries no `fingerprint **N`"))?
        + marker.len();
    let digits: String = row[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(char::is_ascii_digit)
        .collect();
    digits
        .parse()
        .map_err(|_| format!("GUIDE's {name} row has no number after `fingerprint **`"))
}

/// The row's bold `**X.Y.Z**` spans, in order.
fn bold_versions(row: &str) -> impl Iterator<Item = &str> {
    row.split("**")
        .skip(1)
        .step_by(2)
        .filter(|span| version_of_tag(&format!("v{span}")).is_ok())
}

fn ensure_on_base(base: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["merge-base", "--is-ancestor", "HEAD", base])
        .status()
        .map_err(|e| format!("git: {e}"))?;
    if !status.success() {
        return Err(format!(
            "HEAD is not reachable from {base}; a release is cut from the default branch"
        ));
    }
    Ok(())
}

/// `--notes` may name a directory that does not exist yet.
pub fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_plain_three_number_tag_is_a_release() {
        assert_eq!(version_of_tag("v2.5.0").unwrap(), "2.5.0");
        for bad in [
            "2.5.0",
            "v2.5",
            "v2.5.0-dev",
            "arm/p46-root-relief",
            "oracle/hybrid",
            "v2.5.0.1",
        ] {
            assert!(version_of_tag(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_package_version_is_read_not_a_dependency_version() {
        let manifest = "[package]\nname = \"rarog\"\nversion = \"2.5.0\"\n\n[dependencies]\nfoo = { version = \"9.9.9\" }\n";
        assert_eq!(manifest_version(manifest).unwrap(), "2.5.0");
        assert!(manifest_version("[dependencies]\nfoo = { version = \"1\" }\n").is_err());
    }

    const GUIDE: &str = "## Current checkpoint\n\n| Item | Value |\n|---|---|\n\
        | Released baseline | **2.4.0** on `master`; fingerprint **7,601,220 / EBF 2.474** |\n\
        | Development head | `dev`, version **2.5.0**; fingerprint **11,171,726 / EBF 2.512** |\n";

    #[test]
    fn the_declared_fingerprint_is_the_development_heads_not_the_first_in_the_file() {
        assert_eq!(declared_fingerprint(GUIDE).unwrap(), 11_171_726);
        assert!(
            declared_fingerprint("| Released baseline | fingerprint **7,601,220** |\n").is_err()
        );
        let twice = format!("{GUIDE}| Development head | fingerprint **1** |\n");
        assert!(declared_fingerprint(&twice).is_err());
        assert!(declared_fingerprint(include_str!("../../GUIDE.md")).is_ok());
    }

    #[test]
    fn a_release_commit_marks_its_version_released_at_the_declared_fingerprint() {
        // The previous release still standing as the baseline is refused.
        assert!(release_marked_in_guide(GUIDE, "2.5.0").is_err());
        let marked = GUIDE.replace(
            "**2.4.0** on `master`; fingerprint **7,601,220 / EBF 2.474**",
            "**2.5.0** on `master`; fingerprint **11,171,726 / EBF 2.512**",
        );
        assert_eq!(
            release_marked_in_guide(&marked, "2.5.0").unwrap(),
            11_171_726
        );
        assert!(release_marked_in_guide(&marked, "2.5.1").is_err());
        let stale_fingerprint = GUIDE.replace("**2.4.0** on", "**2.5.0** on");
        assert!(release_marked_in_guide(&stale_fingerprint, "2.5.0").is_err());
    }

    #[test]
    fn the_dated_section_body_is_extracted_and_an_undated_or_missing_one_refused() {
        let changelog = "# Changelog\n\n## [Unreleased]\n\n## [2.5.0] - 2026-10-04\n\nA new search.\n\n### Added\n\n- `MultiPV`.\n\n## [2.4.0] - 2026-09-11\n\nOlder.\n";
        assert_eq!(
            changelog_section(changelog, "2.5.0").unwrap(),
            "A new search.\n\n### Added\n\n- `MultiPV`.\n"
        );
        assert_eq!(changelog_section(changelog, "2.4.0").unwrap(), "Older.\n");
        assert!(changelog_section(changelog, "2.6.0").is_err());
        assert!(changelog_section("## [2.5.0]\n\nundated\n", "2.5.0").is_err());
        assert!(
            changelog_section(
                "## [2.5.0] - 2026-10-04\n\n## [2.4.0] - 2026-09-11\nx\n",
                "2.5.0"
            )
            .is_err()
        );
    }
}
