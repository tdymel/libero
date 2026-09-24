//! Finds the workspace and the `dx` that matches its dioxus version.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

/// `$DX`, else `dx` on `PATH`, when it is the lockfile's dioxus version: an
/// older CLI speaks another protocol to the app it builds.
pub(crate) fn dx(root: &Path) -> Result<std::ffi::OsString> {
    let dx = std::env::var_os("DX").unwrap_or_else(|| "dx".into());
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).context("read Cargo.lock")?;
    let wanted = lock
        .split("\n\n")
        .find(|entry| entry.contains("\nname = \"dioxus\"\n"))
        .and_then(|entry| {
            entry
                .lines()
                .find_map(|line| line.strip_prefix("version = "))
        })
        .map(|version| version.trim_matches('"').to_string())
        .context("no dioxus in Cargo.lock")?;
    let output = Command::new(&dx)
        .arg("--version")
        .output()
        .with_context(|| {
            format!(
                "no {} on PATH: install dx {wanted} (see ci.yml's e2e job)",
                dx.display()
            )
        })?;
    let version = String::from_utf8_lossy(&output.stdout);
    if !version.contains(&wanted) {
        bail!(
            "{} is `{}`, not dioxus {wanted}: set DX or reinstall it",
            dx.display(),
            version.trim()
        );
    }
    Ok(dx)
}

pub(crate) fn workspace_root() -> Result<std::path::PathBuf> {
    // `CARGO_MANIFEST_DIR` is `<root>/e2e`.
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(Path::to_path_buf)
        .context("no workspace root above the e2e crate")
}
