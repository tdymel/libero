//! Finds the workspace and the `dx` that matches its dioxus version.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

/// The `dx` built from the lockfile's dioxus version and git rev: another CLI
/// speaks another protocol to the app it builds, and hot-patching breaks.
/// `$DX` when set (it must match), else the first match of the main checkout's
/// `target/tools/dx`, cargo's `bin/dx` and `dx` on `PATH`.
pub(crate) fn dx(root: &Path) -> Result<OsString> {
    let version = locked_version(root, "dioxus")?;
    let rev = locked_rev(root, "dioxus")?;
    let wanted = match &rev {
        Some(rev) => format!("dioxus {version} ({rev})"),
        None => format!("dioxus {version}"),
    };
    let explicit = std::env::var_os("DX");
    let candidates = match &explicit {
        Some(dx) => vec![dx.clone()],
        None => candidates(root),
    };
    let mut seen = Vec::new();
    for dx in candidates {
        let Ok(output) = Command::new(&dx).arg("--version").output() else {
            seen.push(format!("{}: not found", dx.display()));
            continue;
        };
        let reported = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if matches(&reported, &version, rev.as_deref()) {
            return Ok(dx);
        }
        seen.push(format!("{}: `{reported}`", dx.display()));
    }
    let fix = match explicit {
        Some(_) => "unset DX or point it at a matching dx",
        None => "run `just install-dx` in docs/ or set DX",
    };
    bail!("no dx is {wanted}: {}. {fix}", seen.join(", "))
}

/// `$DX`'s fallbacks, the locked build first: a seat has no `target/tools`.
fn candidates(root: &Path) -> Vec<OsString> {
    let mut roots = vec![root.to_path_buf()];
    if let Some(main) = main_checkout(root)
        && main != root
    {
        roots.push(main);
    }
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| Path::new(&home).join(".cargo")));
    let mut out: Vec<OsString> = roots
        .iter()
        .map(|root| root.join("target/tools/dx").into_os_string())
        .collect();
    out.extend(cargo_home.map(|home| home.join("bin/dx").into_os_string()));
    out.push("dx".into());
    out
}

/// The main worktree of the repository `root` belongs to.
fn main_checkout(root: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()
        .ok()?;
    let common = PathBuf::from(String::from_utf8(output.stdout).ok()?.trim());
    common.parent().map(Path::to_path_buf)
}

/// Whether `dx --version` output is `version` at `rev`. With a locked rev, a dx that
/// reports none (built outside a git checkout) may be stale and does not match.
fn matches(reported: &str, version: &str, rev: Option<&str>) -> bool {
    let Some(rest) = reported.strip_prefix("dioxus ") else {
        return false;
    };
    let (reported_version, reported_rev) = match rest.split_once(" (") {
        Some((v, r)) => (v, r.strip_suffix(')')),
        None => (rest, None),
    };
    if reported_version != version {
        return false;
    }
    match (rev, reported_rev) {
        (None, _) => true,
        (Some(rev), Some(reported)) => {
            !reported.is_empty()
                && reported.chars().all(|c| c.is_ascii_hexdigit())
                && rev.starts_with(reported)
        }
        (Some(_), None) => false,
    }
}

/// The short git rev of the package `name` in `Cargo.lock`, `None` for a registry source.
fn locked_rev(root: &Path, name: &str) -> Result<Option<String>> {
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).context("read Cargo.lock")?;
    Ok(lock
        .split("\n\n")
        .find(|entry| entry.contains(&format!("\nname = \"{name}\"\n")))
        .and_then(|entry| {
            entry
                .lines()
                .find_map(|line| line.strip_prefix("source = \"git+"))
        })
        .and_then(|source| source.trim_end_matches('"').rsplit_once('#'))
        .map(|(_, rev)| rev.chars().take(7).collect()))
}

/// The version of the package `name` in the workspace's `Cargo.lock`.
pub(crate) fn locked_version(root: &Path, name: &str) -> Result<String> {
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).context("read Cargo.lock")?;
    lock.split("\n\n")
        .find(|entry| entry.contains(&format!("\nname = \"{name}\"\n")))
        .and_then(|entry| {
            entry
                .lines()
                .find_map(|line| line.strip_prefix("version = "))
        })
        .map(|version| version.trim_matches('"').to_string())
        .with_context(|| format!("no {name} in Cargo.lock"))
}

pub(crate) fn workspace_root() -> Result<std::path::PathBuf> {
    // `CARGO_MANIFEST_DIR` is `<root>/e2e`.
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(Path::to_path_buf)
        .context("no workspace root above the e2e crate")
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn a_dx_matches_only_the_locked_version_and_rev() {
        let (version, rev) = ("0.8.0-alpha.1", Some("f951996"));
        assert!(matches("dioxus 0.8.0-alpha.1 (f951996)", version, rev));
        assert!(!matches("dioxus 0.8.0-alpha.1 (c607e4e)", version, rev));
        assert!(!matches(
            "dioxus 0.7.10 (was built without git repository)",
            version,
            rev
        ));
        // No rev to compare: a crates.io or tarball build of the same version.
        assert!(!matches("dioxus 0.8.0-alpha.1", version, rev));
        assert!(!matches(
            "dioxus 0.8.0-alpha.1 (was built without git repository)",
            version,
            rev
        ));
        assert!(matches("dioxus 0.8.0-alpha.1 (c607e4e)", version, None));
        assert!(!matches("dioxus 0.8.0-alpha.10", version, None));
    }
}
