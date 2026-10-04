//! `cargo xtask release-check vX.Y.Z [--notes <path>] [--base <ref>]`
//!
//! The checks a release tag must pass before anything is built or published,
//! runnable locally before the tag is pushed and run first by the release
//! workflow: the tag names `Cargo.toml`'s version, the tagged commit is on
//! the default branch, and `CHANGELOG.md` has a dated section for that
//! version. With `--notes`, that section's body is written out as the
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
        "release-check {}: version {version} in Cargo.toml, CHANGELOG section present, HEAD on {}",
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
