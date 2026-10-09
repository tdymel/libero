//! `cargo run -p e2e -- desktop [filter]` (1126): the scenarios' `desktop` arm
//! against the fixture app in wry's WebView, all under one Xvfb display.

use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use super::android_runner::list_arm;
use super::{Guard, OwnGroup, own_target_dir, workspace_root};

/// `--ignored` runs only the `desktop: skip` scenarios, e.g. to see 1051 still red.
pub fn run(mut filters: Vec<String>) -> Result<()> {
    let before = filters.len();
    filters.retain(|filter| filter != "--ignored");
    let ignored = filters.len() != before;
    for tool in ["xvfb-run", "xdotool"] {
        let found = Command::new("which").arg(tool).output()?;
        if !found.status.success() {
            bail!("the desktop arm needs {tool} on PATH");
        }
    }
    let root = workspace_root()?;
    let target_dir = own_target_dir()?;
    let artifacts = std::env::temp_dir()
        .join("e2e-artifacts")
        .join(format!("desktop-pid{}", std::process::id()));
    std::fs::create_dir_all(&artifacts).context("create the artifacts directory")?;
    eprintln!("e2e desktop: artifacts go to {}", artifacts.display());

    // Before the listing: its `cargo test` relinks this binary, which the guard re-execs.
    let mut guard = Guard::spawn()?;
    eprintln!("e2e desktop: building the fixture app");
    let built = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args([
            "build",
            "-q",
            "-p",
            "e2e-fixtures",
            "--features",
            "desktop",
            "--target-dir",
        ])
        .arg(&target_dir)
        .status()
        .context("build the fixture app")?;
    if !built.success() {
        bail!("the desktop fixture app failed to build");
    }

    let names = list_arm("desktop", &root, &target_dir, &filters, ignored)?;
    if names.is_empty() {
        bail!("no desktop scenario matches {filters:?}");
    }
    eprintln!("e2e desktop: running {} scenario(s)", names.len());

    // One display for the run; never the Maintainer's (`DISPLAY=:0`).
    let mut tests = Command::new("xvfb-run")
        .args(["-a", "-s", "-screen 0 1280x800x24", env!("CARGO")])
        .current_dir(&root)
        .args([
            "test",
            "-q",
            "-p",
            "e2e",
            "--features",
            "desktop",
            "--test",
            "all",
            "--target-dir",
        ])
        .arg(&target_dir)
        .args(["--", "--exact", "--test-threads=1", "--format", "pretty"])
        .args(ignored.then_some("--ignored"))
        .args(&names)
        // `e2e::driver::desktop::APP_ENV`: the runner is built without the feature.
        .env("E2E_DESKTOP_APP", target_dir.join("debug/e2e-fixtures"))
        .env("E2E_ARTIFACTS", &artifacts)
        .stdin(Stdio::null())
        .own_group()
        .spawn()
        .context("run the tests under xvfb-run")?;
    guard.tell(&format!("group {}", tests.id()));
    let status = tests.wait()?;
    guard.done();
    if !status.success() {
        bail!(
            "desktop scenarios failed, app log in {}",
            artifacts.display()
        );
    }
    Ok(())
}
