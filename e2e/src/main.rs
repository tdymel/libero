//! The runner, `cargo run -p e2e`: serves the fixtures with `dx run` (no watcher) on a free
//! port, runs the tests, then stops everything; a guard process cleans up if it dies (todo 313).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

mod android_runner;
mod desktop_runner;
mod dx;
mod http;
mod process;

use dx::{dx, workspace_root};
use http::{free_port, wait_for_app};
use process::{
    GUARD_ENV, Guard, count_by_cmdline, describe_pid, guard, kill_by_cmdline, libc_kill,
    pids_by_cmdline, stop,
};

/// The fixture app's `<title>` (`e2e/fixtures/Dioxus.toml`): dx's build splash cannot fake it.
const FIXTURE_TITLE: &str = "libero e2e fixtures";

/// The docs site's `<title>`, from `docs/Dioxus.toml`, for `sweep`.
const DOCS_TITLE: &str = "libero";

fn main() -> Result<()> {
    if std::env::var_os(GUARD_ENV).is_some() {
        return guard();
    }
    // `sweep` (todo 760) serves the docs site instead of the fixtures and runs
    // `tests/sweep.rs` alone: a report over every page, too slow for the suite.
    let mut passthrough: Vec<String> = std::env::args().skip(1).collect();
    if passthrough.first().is_some_and(|arg| arg == "android") {
        return android_runner::run(passthrough.split_off(1));
    }
    if passthrough.first().is_some_and(|arg| arg == "desktop") {
        return desktop_runner::run(passthrough.split_off(1));
    }
    let sweep = passthrough.first().is_some_and(|arg| arg == "sweep");
    if sweep {
        passthrough.remove(0);
        passthrough.push("--nocapture".into());
    }
    let (app_dir, title, test) = match sweep {
        true => ("docs", DOCS_TITLE, "sweep"),
        false => ("e2e/fixtures", FIXTURE_TITLE, "all"),
    };
    let root = workspace_root()?;
    let port = free_port()?;
    let base_url = format!("http://127.0.0.1:{port}");

    let dx = dx(&root)?;

    // Nested builds go where the runner was built: they don't see the outer `--target-dir` (todo 328).
    let target_dir = own_target_dir()?;
    eprintln!("e2e: building into {}", target_dir.display());

    // One artifacts directory per run, never per port: a reused one lost red runs' evidence
    // (todos 329, 364). `prune_old_runs` keeps a red run a week.
    let started = SystemTime::now();
    let artifacts = std::env::temp_dir().join("e2e-artifacts").join(format!(
        "run-{}-pid{}-port{port}",
        e2e::journal::utc_now().replace([':', '.'], "-"),
        std::process::id(),
    ));
    prune_old_runs(
        artifacts
            .parent()
            .context("the artifacts directory has a parent")?,
    );
    std::fs::create_dir_all(&artifacts).context("create the artifacts directory")?;
    eprintln!("e2e: artifacts go to {}", artifacts.display());

    // Inherited by the test process, which journals too.
    // SAFETY: single-threaded, before anything is spawned.
    unsafe {
        std::env::set_var("E2E_ARTIFACTS", &artifacts);
        std::env::set_var(
            e2e::journal::RUN_STARTED,
            started
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .to_string(),
        );
    }

    // dx's own output, kept rather than discarded: when the fixture crate fails
    // to compile, this file is the only place that says why.
    let dx_log = artifacts.join("dx.log");
    let log_handle = std::fs::File::create(&dx_log).context("create the dx log")?;

    // Eagerly, so a broken archive fails here rather than inside the first
    // contrast assertion, with the browser and server already running.
    let axe = e2e::vendor::ensure_axe().context("prepare the vendored axe archive")?;
    eprintln!("e2e: axe ready at {}", axe.display());

    let mut guard = Guard::spawn()?;

    eprintln!("e2e: starting the {app_dir} server on {base_url}");
    let mut server = Command::new(&dx);
    server.current_dir(root.join(app_dir)).args([
        "run",
        "--web",
        "--addr",
        "127.0.0.1",
        "--port",
        &port.to_string(),
    ]);
    // `E2E_RELEASE=1`: optimised fixtures, for the frame-time report (1086).
    if std::env::var_os("E2E_RELEASE").is_some() {
        server.arg("--release");
    }
    let mut server = server
        // The env var, since `dx` takes no `--target-dir`. `RUSTC_WRAPPER` is
        // left alone: `dx` drives it itself for hot-patching.
        .env("CARGO_TARGET_DIR", &target_dir)
        .stdout(Stdio::from(
            log_handle.try_clone().context("clone the log handle")?,
        ))
        .stderr(Stdio::from(log_handle))
        // Own process group, so a kill also takes its cargo and rustc children.
        .process_group(0)
        .spawn()
        .context("start dx run")?;
    guard.tell(&format!("group {}", server.id()));

    // The test binary builds while `dx` builds the wasm: cargo locks per profile dir (823).
    // Its errors show again in the real run below.
    let mut prebuild = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["test", "-p", "e2e", "--test", test, "--no-run", "--target-dir"])
        .arg(&target_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .context("prebuild the tests")?;
    guard.tell(&format!("group {}", prebuild.id()));

    let ready = wait_for_app(&base_url, title, &mut server, &dx_log);
    if ready.is_ok() {
        let _ = prebuild.wait();
    } else {
        stop(&mut prebuild);
    }
    if let Err(error) = ready {
        stop(&mut server);
        guard.done();
        // Red before a single test ran. Marked all the same, so the directory
        // survives the prune and so the next reader sees which wait it was.
        mark_red(&artifacts, &format!("served-body: {error}"));
        return Err(error);
    }

    // Per-run Chrome profile under the target dir: a held profile blocks every later launch.
    let profile = target_dir.join(format!("e2e-chrome-{}", std::process::id()));
    guard.tell(&format!("profile {}", profile.display()));

    eprintln!("e2e: server is up, running the suite");
    // The runner's arguments go to libtest after `--`, so filters and `--nocapture` both work.
    let status = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["test", "-p", "e2e", "--test", test, "--target-dir"])
        .arg(&target_dir)
        // Before the `--`, or cargo silently builds into `target/main`.
        .arg("--")
        .args(&passthrough)
        .env("E2E_BASE_URL", &base_url)
        .env("E2E_SWEEP_REPORT", root.join("target/a11y-sweep/report.md"))
        .env("E2E_CHROME_PROFILE", &profile)
        .env("E2E_ARTIFACTS", &artifacts)
        // Read here, to see whether anything ran. Echoed line by line.
        .stdout(Stdio::piped())
        // Its own group, so the guard can stop it and the Chrome it launched
        // without signalling whatever group the runner was started in.
        .process_group(0)
        .spawn()
        .context("run the tests")
        .and_then(|mut tests| {
            guard.tell(&format!("group {}", tests.id()));
            let mut tally = e2e::journal::Tally::default();
            // Browser tests only: the verdict is about pages (todo 364, `NON_BROWSER`).
            let mut browser = e2e::journal::Tally::default();
            let mut set_aside = 0u64;
            let stdout = tests.stdout.take().context("the tests' stdout")?;
            // Teed to a file: a caller's `| tail` cut the per-test detail twice (todo 364).
            let mut log = std::fs::File::create(artifacts.join("test-output.log")).ok();
            for line in BufReader::new(stdout).lines() {
                let line = line.context("read the tests' output")?;
                tally += e2e::journal::Tally::from_summary(&line);
                if let Some((test, passed)) = e2e::journal::test_outcome(&line) {
                    match (e2e::journal::drives_a_browser(test), passed) {
                        (true, true) => browser.passed += 1,
                        (true, false) => browser.failed += 1,
                        (false, true) => set_aside += 1,
                        // Says nothing about pages: neither counted nor set aside.
                        (false, false) => {}
                    }
                }
                if let Some(log) = log.as_mut() {
                    let _ = writeln!(log, "{line}");
                }
                println!("{line}");
            }
            Ok((
                tests.wait().context("wait for the tests")?,
                tally,
                browser,
                set_aside,
            ))
        });

    let red = !matches!(&status, Ok((status, ..)) if status.success());
    let (browser, set_aside) = status
        .as_ref()
        .map(|(_, _, browser, set_aside)| (*browser, *set_aside))
        .unwrap_or_default();

    // The browser is a never-dropped static, so reap Chrome by this run's profile path.
    // On a red run, list the survivors first: they are evidence (todo 364).
    let needle = profile.to_string_lossy().into_owned();
    if red {
        let before = pids_by_cmdline(&needle);
        let census = before
            .iter()
            .map(|&pid| describe_pid(pid))
            .collect::<Vec<_>>()
            .join("\n");
        let _ = std::fs::write(
            artifacts.join("browser-processes.txt"),
            format!(
                "{} browser process(es) alive on {needle} when the tests ended\n\
                 pid ppid state cmdline\n{census}\n",
                before.len()
            ),
        );
    }
    let signalled = kill_by_cmdline(&needle);

    // Wait for them to exit: deleting a profile Chrome still writes leaves it half-deleted.
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && count_by_cmdline(&needle) > 0 {
        std::thread::sleep(Duration::from_millis(100));
    }

    // Verify, escalate, and name any surviving pids rather than a count of SIGTERMs (todo 364).
    let mut survivors = pids_by_cmdline(&needle);
    if !survivors.is_empty() {
        for &pid in &survivors {
            unsafe { libc_kill(pid as i32, 9) };
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && count_by_cmdline(&needle) > 0 {
            std::thread::sleep(Duration::from_millis(100));
        }
        survivors = pids_by_cmdline(&needle);
    }
    if survivors.is_empty() {
        if signalled > 0 {
            eprintln!("e2e: reaped {signalled} browser process(es), none left");
        }
    } else {
        let message = format!(
            "signalled {signalled} browser process(es) and {} survived SIGTERM then SIGKILL: {}. \
             The profile {} is left in place; kill them by pid and remove it.",
            survivors.len(),
            survivors
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            profile.display(),
        );
        eprintln!("e2e: {message}");
        e2e::journal::note(&message);
    }

    // A red run's Chrome profile is part of its evidence, so it stays
    // (todo 364). A green one is removed as before; leaving every run's
    // profile behind is how /tmp filled up in the first place.
    if red || !survivors.is_empty() {
        eprintln!("e2e: keeping the Chrome profile {}", profile.display());
    } else if let Err(error) = std::fs::remove_dir_all(&profile)
        && profile.exists()
    {
        // Said out loud rather than swallowed: a cleanup that silently fails
        // is how a leak survives being looked for.
        eprintln!("e2e: could not remove {} ({error})", profile.display());
    }

    stop(&mut server);
    eprintln!("e2e: server stopped");
    guard.done();

    if red {
        // A name filter makes "everything failed" mean something much
        // smaller, so the verdict is told about it. Flags are not a filter.
        let filter = passthrough
            .iter()
            .find(|arg| !arg.starts_with('-'))
            .map(String::as_str);
        let verdict = e2e::journal::verdict(
            browser,
            set_aside,
            &artifacts,
            &profile,
            survivors.len(),
            filter,
        );
        eprintln!("{verdict}");
        mark_red(&artifacts, &verdict);
    }

    let (status, tally, ..) = status?;
    if !status.success() {
        std::process::exit(1);
    }
    // libtest exits 0 when its filter matched nothing, so a mistyped name reads
    // as a green run. `--list` runs nothing on purpose.
    if tally.ran() == 0 && !passthrough.iter().any(|arg| arg == "--list") {
        bail!("no test ran: the name filter {passthrough:?} matched no test");
    }
    Ok(())
}

/// Writes the verdict to `RED`, so `prune_old_runs` keeps the directory a week.
fn mark_red(artifacts: &Path, verdict: &str) {
    let _ = std::fs::write(
        artifacts.join("RED"),
        format!("{}\n{verdict}\n", e2e::journal::utc_now()),
    );
}

/// The runner's own target directory: the nearest ancestor of the executable with
/// cargo's `CACHEDIR.TAG`, whichever way it was chosen.
fn own_target_dir() -> Result<std::path::PathBuf> {
    let exe = std::env::current_exe().context("locate the runner's executable")?;
    exe.ancestors()
        .skip(1)
        .find(|dir| dir.join("CACHEDIR.TAG").is_file())
        .map(Path::to_path_buf)
        .with_context(|| format!("no cargo target directory above {}", exe.display()))
}

/// Deletes artifact directories older than a day, or a week when marked `RED` (todo 364).
fn prune_old_runs(parent: &Path) {
    const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
    const MAX_AGE_RED: Duration = Duration::from_secs(7 * 24 * 60 * 60);
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let limit = if path.join("RED").exists() {
            MAX_AGE_RED
        } else {
            MAX_AGE
        };
        let old = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|at| at.elapsed().ok())
            .is_some_and(|age| age > limit);
        if old {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

use std::path::Path;
