//! `cargo run -p e2e -- desktop [filter]` (1126): the scenarios' `desktop` arm
//! against the fixture app in wry's WebView, all under one Xvfb display (macOS and Windows:
//! the session's).

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use super::android_runner::list_arm;
use super::process::{Signal, pids_by_cmdline, stop};
use super::{Guard, OwnGroup, own_target_dir, workspace_root};

/// Past this many bytes of names a Windows command line (32 767 characters) fails (os error 206).
const NAMES_BUDGET: usize = 20_000;

/// `--ignored` runs only the `desktop: skip` scenarios, e.g. to see 1051 still red.
pub fn run(mut filters: Vec<String>) -> Result<()> {
    let before = filters.len();
    filters.retain(|filter| filter != "--ignored");
    let ignored = filters.len() != before;
    // macOS and Windows need neither: the runner's own GUI session, and the app's NSEvents
    // (2782) or WebView2's DevTools (2783).
    let tools: &[&str] = if cfg!(any(target_os = "macos", windows)) {
        &[]
    } else {
        &["xvfb-run", "xdotool"]
    };
    for tool in tools {
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

    // Windows cannot replace the running `e2e.exe`, which `--features desktop` rebuilds, unless
    // the runner was built with it (CI): then no second target dir compiles everything twice.
    let test_dir = match cfg!(windows) && !cfg!(feature = "desktop") {
        true => target_dir.join("desktop-tests"),
        false => target_dir.clone(),
    };
    let names = list_arm("desktop", &root, &test_dir, &filters, ignored)?;
    if names.is_empty() {
        bail!("no desktop scenario matches {filters:?}");
    }
    eprintln!("e2e desktop: running {} scenario(s)", names.len());
    let app = target_dir.join(format!(
        "debug/e2e-fixtures{}",
        std::env::consts::EXE_SUFFIX
    ));

    // A deadline for the whole run: a wedged window or bridge hangs a read;
    // `E2E_DESKTOP_DEADLINE` (seconds) overrides.
    let deadline = std::env::var("E2E_DESKTOP_DEADLINE")
        .ok()
        .and_then(|secs| secs.parse().ok())
        .unwrap_or(120 + 6 * names.len() as u64);
    let deadline = Instant::now() + Duration::from_secs(deadline);
    let mut passed = true;
    let budget = if cfg!(windows) {
        NAMES_BUDGET
    } else {
        usize::MAX
    };
    for batch in batches(&names, budget) {
        // Never the Maintainer's display (`DISPLAY=:0`).
        let mut command = if cfg!(any(target_os = "macos", windows)) {
            Command::new(env!("CARGO"))
        } else {
            let mut xvfb = Command::new("xvfb-run");
            xvfb.args(["-a", "-s", "-screen 0 1280x800x24", env!("CARGO")]);
            xvfb
        };
        let mut tests = command
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
            .arg(&test_dir)
            .args(["--", "--exact", "--test-threads=1", "--format", "pretty"])
            .args(ignored.then_some("--ignored"))
            .args(batch)
            // `e2e::driver::desktop::APP_ENV`: the runner is built without the feature.
            .env("E2E_DESKTOP_APP", &app)
            .env("E2E_ARTIFACTS", &artifacts)
            .stdin(Stdio::null())
            .own_group()
            .spawn()
            .context("run the desktop tests")?;
        guard.tell(&format!("group {}", tests.id()));
        let status = loop {
            if let Some(status) = tests.try_wait()? {
                break status;
            }
            if Instant::now() > deadline {
                stop(&mut tests);
                stop_apps(&app);
                guard.done();
                bail!(
                    "e2e desktop: the run overran its deadline; stopped. App log in {}",
                    artifacts.display()
                );
            }
            std::thread::sleep(Duration::from_secs(1));
        };
        passed &= status.success();
    }
    stop_apps(&app);
    guard.done();
    if !passed {
        bail!(
            "desktop scenarios failed, app log in {}",
            artifacts.display()
        );
    }
    Ok(())
}

/// The names in runs whose command line stays within the budget.
fn batches(names: &[String], budget: usize) -> Vec<&[String]> {
    let (mut batches, mut start, mut len) = (Vec::new(), 0, 0);
    for (at, name) in names.iter().enumerate() {
        if len + name.len() + 1 > budget {
            batches.push(&names[start..at]);
            (start, len) = (at, 0);
        }
        len += name.len() + 1;
    }
    batches.push(&names[start..]);
    batches
}

/// The last scenario's app idles for reuse and outlives the tests; on Windows it and its
/// WebView2 hold the runner's inherited stdout open, so a `| tee` never ended (3326).
fn stop_apps(app: &Path) {
    for pid in pids_by_cmdline(&app.to_string_lossy()) {
        #[cfg(windows)]
        super::process::signal_group(pid, Signal::Kill);
        #[cfg(unix)]
        super::process::signal_pid(pid, Signal::Term);
    }
}

#[cfg(test)]
mod tests {
    use super::batches;

    #[test]
    fn names_split_where_the_budget_ends_and_keep_their_order() {
        let names: Vec<String> = ["aaa", "bbb", "ccc", "dd"].map(String::from).to_vec();
        assert_eq!(batches(&names, 8), [&names[..2], &names[2..]]);
        assert_eq!(batches(&names, usize::MAX), [&names[..]]);
    }
}
