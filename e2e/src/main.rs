//! The runner, `cargo run -p e2e`: serves the fixtures with `dx run` (no watcher) on a free
//! port, runs the tests, then stops everything; a guard process cleans up if it dies (todo 313).

use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

mod android_runner;
mod bindgen;
mod desktop_runner;
mod dx;
mod http;
mod ios_runner;
mod process;
mod units;

use dx::{dx, workspace_root};
use http::{free_port, wait_for_app, wait_for_log_line};
use process::{
    GUARD_ENV, Guard, OwnGroup, Signal, count_by_cmdline, describe_pid, guard, kill_by_cmdline,
    pids_by_cmdline, signal_pid, stop, wait_until,
};

/// The fixture app's `<title>` (`e2e/fixtures/Dioxus.toml`): dx's build splash cannot fake it.
const FIXTURE_TITLE: &str = "libero e2e fixtures";

/// The docs site's `<title>`, from `docs/Dioxus.toml`, for `sweep`.
const DOCS_TITLE: &str = "Libero";

/// What `dx run` logs once its bundle is built, optimised and pre-compressed.
const BUILD_DONE: &str = "Build completed successfully";

/// What a run serves, and the test binary that runs against it.
struct Target {
    app_dir: &'static str,
    app_package: &'static str,
    title: &'static str,
    test: &'static str,
    /// The docs site's own tests, which build every page rather than a unit's fixtures.
    docs: bool,
}

impl Target {
    fn new(docs_test: Option<&'static str>) -> Self {
        match docs_test {
            Some(test) => Self {
                app_dir: "docs",
                app_package: "docs",
                title: DOCS_TITLE,
                test,
                docs: true,
            },
            None => Self {
                app_dir: "e2e/fixtures",
                app_package: "e2e-fixtures",
                title: FIXTURE_TITLE,
                test: "all",
                docs: false,
            },
        }
    }
}

/// One run's addresses and directories.
struct Session {
    base_url: String,
    artifacts: PathBuf,
    /// Chrome's profile under the target dir: a held profile blocks every later launch.
    profile: PathBuf,
}

/// What the test process left behind.
struct Outcome {
    status: ExitStatus,
    tally: e2e::journal::Tally,
    /// Browser tests only: the verdict is about pages (todo 364, `NON_BROWSER`).
    browser: e2e::journal::Tally,
    set_aside: u64,
}

fn main() -> Result<()> {
    if std::env::var_os(GUARD_ENV).is_some() {
        return guard();
    }
    if bindgen::is_wrapper() {
        return bindgen::run();
    }
    let mut passthrough: Vec<String> = std::env::args().skip(1).collect();
    match passthrough.first().map(String::as_str) {
        Some("android") => return android_runner::run(passthrough.split_off(1)),
        Some("desktop") => return desktop_runner::run(passthrough.split_off(1)),
        Some("ios") => return ios_runner::run(passthrough.split_off(1)),
        _ => {}
    }
    let target = Target::new(docs_test(&mut passthrough));
    run(&target, passthrough)
}

/// `sweep` (todo 760) serves the docs site instead of the fixtures and runs `tests/sweep.rs`
/// alone: a report over every page, too slow for the suite. `docs-perf` (todo 2104) likewise
/// runs the docs timing survey of `tests/docs_perf.rs`.
fn docs_test(args: &mut Vec<String>) -> Option<&'static str> {
    let test = match args.first().map(String::as_str) {
        Some("sweep") => "sweep",
        Some("docs-perf") => "docs_perf",
        _ => return None,
    };
    args.remove(0);
    args.push("--nocapture".into());
    if test == "docs_perf" {
        args.push("--ignored".into());
    }
    Some(test)
}

fn run(target: &Target, passthrough: Vec<String>) -> Result<()> {
    let root = workspace_root()?;
    let port = free_port()?;
    let dx = dx(&root)?;
    eprintln!("e2e: dx is {}", dx.display());

    // Nested builds go where the runner was built: they don't see the outer `--target-dir` (todo 328).
    let target_dir = own_target_dir()?;
    eprintln!("e2e: building into {}", target_dir.display());

    let started = SystemTime::now();
    let artifacts = start_artifacts(port, started)?;
    let session = Session {
        base_url: format!("http://127.0.0.1:{port}"),
        profile: target_dir.join(format!("e2e-chrome-{}", std::process::id())),
        artifacts,
    };
    // dx's own output, kept rather than discarded: when the fixture crate fails
    // to compile, this file is the only place that says why.
    let dx_log = session.artifacts.join("dx.log");
    let log_handle = std::fs::File::create(&dx_log).context("create the dx log")?;

    // Eagerly, so a broken archive fails here rather than inside the first
    // contrast assertion, with the browser and server already running.
    let axe = e2e::vendor::ensure_axe().context("prepare the vendored axe archive")?;
    eprintln!("e2e: axe ready at {}", axe.display());

    let mut guard = Guard::spawn()?;

    eprintln!(
        "e2e: starting the {} server on {}",
        target.app_dir, session.base_url
    );
    let (mut command, bindgen_report) =
        server_command(target, &root, &dx, &target_dir, port, &passthrough)?;
    let mut server = command
        .stdout(Stdio::from(
            log_handle.try_clone().context("clone the log handle")?,
        ))
        .stderr(Stdio::from(log_handle))
        // Own process group, so a kill also takes its cargo and rustc children.
        .own_group()
        .spawn()
        .context("start dx run")?;
    guard.tell(&format!("group {}", server.id()));

    let mut prebuild = spawn_prebuild(&root, target.test, &target_dir)?;
    guard.tell(&format!("group {}", prebuild.id()));

    // A unit filter is expanded once the test binary is built, before the wasm is: a
    // filter that matches nothing stops here.
    let mut test_args = passthrough.clone();
    if units::wanted(&passthrough) {
        let _ = prebuild.wait();
        let exact = list_tests(&root, target.test, &target_dir)
            .map(|listed| units::exact(&listed, &passthrough));
        match exact {
            Ok(exact) if !exact.is_empty() => test_args = exact,
            outcome => {
                stop(&mut server);
                guard.done();
                outcome?;
                bail!("no test ran: the name filter {passthrough:?} matched no test");
            }
        }
        let names = test_args.iter().skip_while(|arg| *arg != "--exact").count() - 1;
        eprintln!("e2e: the filter selects {names} test(s), run --exact");
    }

    let mut ready = wait_for_app(&session.base_url, target.title, &mut server, &dx_log);
    if ready.is_ok() && std::env::var_os("E2E_RELEASE").is_some() {
        ready = wait_for_log_line(BUILD_DONE, &mut server, &dx_log);
    }
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
        mark_red(&session.artifacts, &format!("served-body: {error}"));
        return Err(error);
    }

    guard.tell(&format!("profile {}", session.profile.display()));
    eprintln!(
        "e2e: server is up after {} s (wasm-bindgen {}), running the suite",
        started.elapsed().unwrap_or_default().as_secs(),
        bindgen::outcome(bindgen_report.as_deref())
    );
    let outcome = run_suite(
        &root,
        target.test,
        &target_dir,
        &test_args,
        &session,
        &mut guard,
    );

    let red = !matches!(&outcome, Ok(outcome) if outcome.status.success());
    report_slow_tests(&session.artifacts);
    let survivors = reap_browser(&session, red);
    drop_profile(&session.profile, red || !survivors.is_empty());

    stop(&mut server);
    eprintln!("e2e: server stopped");
    guard.done();

    if red {
        let (browser, set_aside) = outcome
            .as_ref()
            .map(|outcome| (outcome.browser, outcome.set_aside))
            .unwrap_or_default();
        report_red(&session, &passthrough, browser, set_aside, survivors.len());
    }
    finish(outcome?, &passthrough)
}

/// The `dx run` command for `target`, with its environment; and the file the bindgen cache
/// reports to. Stdio and spawning are the caller's.
fn server_command(
    target: &Target,
    root: &Path,
    dx: &OsStr,
    target_dir: &Path,
    port: u16,
    passthrough: &[String],
) -> Result<(Command, Option<PathBuf>)> {
    let mut server = Command::new(dx);
    server.current_dir(root.join(target.app_dir)).args([
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
        // dx serves the last release bundle while it rebuilds, and readiness took that
        // one; the first navigation then timed out (todo 1956).
        let stale = target_dir
            .join("dx")
            .join(target.app_package)
            .join("release/web");
        if let Err(error) = std::fs::remove_dir_all(&stale)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            bail!("remove the stale bundle {}: {error}", stale.display());
        }
    }
    let bindgen_report = bindgen::wrap(&mut server, root, target_dir)?;
    // A unit filter builds only that unit's fixture modules: 2 s instead of 25 (1618).
    let fixtures = match target.docs {
        true => None,
        false => units::fixtures(passthrough, &fixture_modules(root)),
    };
    match &fixtures {
        Some(modules) => {
            eprintln!("e2e: building only the fixtures {}", modules.join(", "));
            server.env("E2E_FIXTURES", modules.join(","));
        }
        None => {
            server.env_remove("E2E_FIXTURES");
        }
    }
    // The env var, since `dx` takes no `--target-dir`.
    server.env("CARGO_TARGET_DIR", target_dir);
    Ok((server, bindgen_report))
}

/// The test binary builds while `dx` builds the wasm: cargo locks per profile dir (823).
/// Its errors show again in the real run.
fn spawn_prebuild(root: &Path, test: &str, target_dir: &Path) -> Result<Child> {
    Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "test",
            "-p",
            "e2e",
            "--test",
            test,
            "--no-run",
            "--target-dir",
        ])
        .arg(target_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .own_group()
        .spawn()
        .context("prebuild the tests")
}

/// One artifacts directory per run, never per port: a reused one lost red runs' evidence
/// (todos 329, 364). `prune_old_runs` keeps a red run a week. Exports what the tests read.
fn start_artifacts(port: u16, started: SystemTime) -> Result<PathBuf> {
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
        // The storage fixture's files off the web: fresh each run, never the user's data dir.
        std::env::set_var("E2E_STORAGE_DIR", artifacts.join("storage"));
        std::env::set_var(
            e2e::journal::RUN_STARTED,
            started
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .to_string(),
        );
        // sccache for `dx` and the test build too, unless the caller chose a wrapper: a cold
        // fixture wasm took 204 s, 133 s from a warm cache; cargo's fingerprints ignore it.
        if std::env::var_os("RUSTC_WRAPPER").is_none() && has_sccache() {
            std::env::set_var("RUSTC_WRAPPER", "sccache");
        }
    }
    Ok(artifacts)
}

/// Runs the tests against the served app and echoes their output line by line.
fn run_suite(
    root: &Path,
    test: &str,
    target_dir: &Path,
    test_args: &[String],
    session: &Session,
    guard: &mut Guard,
) -> Result<Outcome> {
    // The runner's arguments go to libtest after `--`, so filters and `--nocapture` both work.
    let mut tests = Command::new(env!("CARGO"))
        .current_dir(root)
        .args(["test", "-p", "e2e", "--test", test, "--target-dir"])
        .arg(target_dir)
        // Before the `--`, or cargo silently builds into `target/main`.
        .arg("--")
        .args(test_args)
        .env("E2E_BASE_URL", &session.base_url)
        .env("E2E_SWEEP_REPORT", root.join("target/a11y-sweep/report.md"))
        .env("E2E_CHROME_PROFILE", &session.profile)
        .env("E2E_ARTIFACTS", &session.artifacts)
        // Read here, to see whether anything ran. Echoed line by line.
        .stdout(Stdio::piped())
        // Its own group, so the guard can stop it and the Chrome it launched
        // without signalling whatever group the runner was started in.
        .own_group()
        .spawn()
        .context("run the tests")?;
    guard.tell(&format!("group {}", tests.id()));

    let mut tally = e2e::journal::Tally::default();
    let mut browser = e2e::journal::Tally::default();
    let mut set_aside = 0u64;
    let stdout = tests.stdout.take().context("the tests' stdout")?;
    // Teed to a file: a caller's `| tail` cut the per-test detail twice (todo 364).
    let mut log = std::fs::File::create(session.artifacts.join("test-output.log")).ok();
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
    Ok(Outcome {
        status: tests.wait().context("wait for the tests")?,
        tally,
        browser,
        set_aside,
    })
}

/// The browser is a never-dropped static, so reaps Chrome by this run's profile path. On a
/// red run, lists the survivors first: they are evidence (todo 364). Returns who survived.
fn reap_browser(session: &Session, red: bool) -> Vec<u32> {
    let needle = session.profile.to_string_lossy().into_owned();
    if red {
        let before = pids_by_cmdline(&needle);
        let census = before
            .iter()
            .map(|&pid| describe_pid(pid))
            .collect::<Vec<_>>()
            .join("\n");
        let _ = std::fs::write(
            session.artifacts.join("browser-processes.txt"),
            format!(
                "{} browser process(es) alive on {needle} when the tests ended\n\
                 pid ppid state cmdline\n{census}\n",
                before.len()
            ),
        );
    }
    let signalled = kill_by_cmdline(&needle);

    // Wait for them to exit: deleting a profile Chrome still writes leaves it half-deleted.
    wait_until(Duration::from_secs(10), || count_by_cmdline(&needle) == 0);

    // Verify, escalate, and name any surviving pids rather than a count of SIGTERMs (todo 364).
    let mut survivors = pids_by_cmdline(&needle);
    if !survivors.is_empty() {
        for &pid in &survivors {
            signal_pid(pid, Signal::Kill);
        }
        wait_until(Duration::from_secs(5), || count_by_cmdline(&needle) == 0);
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
            session.profile.display(),
        );
        eprintln!("e2e: {message}");
        e2e::journal::note(&message);
    }
    survivors
}

/// A red run's Chrome profile is part of its evidence, so `keep` leaves it (todo 364). A green
/// one is removed: leaving every run's profile behind is how /tmp filled up in the first place.
fn drop_profile(profile: &Path, keep: bool) {
    if keep {
        eprintln!("e2e: keeping the Chrome profile {}", profile.display());
    } else if let Err(error) = std::fs::remove_dir_all(profile)
        && profile.exists()
    {
        // Said out loud rather than swallowed: a cleanup that silently fails
        // is how a leak survives being looked for.
        eprintln!("e2e: could not remove {} ({error})", profile.display());
    }
}

/// Prints the verdict and marks the run red. A name filter makes "everything failed" mean
/// something much smaller, so the verdict is told about it; flags are not a filter.
fn report_red(
    session: &Session,
    passthrough: &[String],
    browser: e2e::journal::Tally,
    set_aside: u64,
    survivors: usize,
) {
    let filter = passthrough
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .map(String::as_str);
    let verdict = e2e::journal::verdict(
        browser,
        set_aside,
        &session.artifacts,
        &session.profile,
        survivors,
        filter,
    );
    eprintln!("{verdict}");
    mark_red(&session.artifacts, &verdict);
}

/// The process exit: red when the tests failed, and an error when nothing ran.
fn finish(outcome: Outcome, passthrough: &[String]) -> Result<()> {
    if !outcome.status.success() {
        std::process::exit(1);
    }
    // libtest exits 0 when its filter matched nothing, so a mistyped name reads
    // as a green run. `--list` runs nothing on purpose.
    if outcome.tally.ran() == 0 && !passthrough.iter().any(|arg| arg == "--list") {
        bail!("no test ran: the name filter {passthrough:?} matched no test");
    }
    Ok(())
}

fn has_sccache() -> bool {
    Command::new("sccache")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// The fixture crate's module names, one per `src/*.rs`.
fn fixture_modules(root: &Path) -> Vec<String> {
    std::fs::read_dir(root.join("e2e/fixtures/src"))
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            Some(name.strip_suffix(".rs")?.to_string())
        })
        .collect()
}

/// The test names of the `test` binary, from libtest's `--list`.
fn list_tests(root: &Path, test: &str, target_dir: &Path) -> Result<Vec<String>> {
    let output = Command::new(env!("CARGO"))
        .current_dir(root)
        .args(["test", "-q", "-p", "e2e", "--test", test, "--target-dir"])
        .arg(target_dir)
        .args(["--", "--list"])
        .stderr(Stdio::inherit())
        .output()
        .context("list the tests")?;
    if !output.status.success() {
        bail!("listing the tests failed: {}", output.status);
    }
    Ok(units::listed(&String::from_utf8_lossy(&output.stdout)))
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
fn own_target_dir() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("locate the runner's executable")?;
    exe.ancestors()
        .skip(1)
        .find(|dir| dir.join("CACHEDIR.TAG").is_file())
        .map(Path::to_path_buf)
        .with_context(|| format!("no cargo target directory above {}", exe.display()))
}

/// Lists the browser tests that ran past `E2E_SLOW_TEST_S`, slowest first; never fails the run.
fn report_slow_tests(artifacts: &Path) {
    let Ok(log) = std::fs::read_to_string(artifacts.join(e2e::journal::SLOW_TESTS)) else {
        return;
    };
    let slow = slowest_first(&log);
    eprintln!(
        "e2e: {} test(s) over the {} s budget (E2E_SLOW_TEST_S):",
        slow.len(),
        e2e::journal::slow_budget().as_secs()
    );
    for (seconds, test) in slow {
        eprintln!("  {seconds:>6.1} s  {test}");
    }
}

/// `<seconds> <test>` per line, longest first; a line that is not one is skipped.
fn slowest_first(log: &str) -> Vec<(f64, &str)> {
    let mut slow: Vec<(f64, &str)> = log
        .lines()
        .filter_map(|line| line.split_once(' '))
        .filter_map(|(seconds, test)| Some((seconds.parse().ok()?, test)))
        .collect();
    slow.sort_by(|a, b| b.0.total_cmp(&a.0));
    slow
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    #[test]
    fn sweep_and_docs_perf_serve_the_docs_site() {
        let mut sweep = args(&["sweep", "a_page"]);
        assert_eq!(docs_test(&mut sweep), Some("sweep"));
        assert_eq!(sweep, args(&["a_page", "--nocapture"]));

        let mut perf = args(&["docs-perf"]);
        assert_eq!(docs_test(&mut perf), Some("docs_perf"));
        assert_eq!(perf, args(&["--nocapture", "--ignored"]));
    }

    #[test]
    fn any_other_argument_runs_the_fixture_suite() {
        let mut filter = args(&["button::", "--nocapture"]);
        assert_eq!(docs_test(&mut filter), None);
        assert_eq!(filter, args(&["button::", "--nocapture"]));
        assert_eq!(docs_test(&mut Vec::new()), None);
    }

    #[test]
    fn the_target_names_the_app_and_its_test_binary() {
        let fixtures = Target::new(None);
        assert_eq!((fixtures.app_dir, fixtures.test), ("e2e/fixtures", "all"));
        assert!(!fixtures.docs);
        let docs = Target::new(Some("sweep"));
        assert_eq!(
            (docs.app_dir, docs.test, docs.title),
            ("docs", "sweep", DOCS_TITLE)
        );
    }

    #[test]
    fn old_runs_go_and_red_ones_stay_longer() {
        let parent = std::env::temp_dir().join(format!("e2e-prune-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        let (green, red) = (parent.join("green"), parent.join("red"));
        std::fs::create_dir_all(&green).unwrap();
        std::fs::create_dir_all(&red).unwrap();
        std::fs::write(red.join("RED"), "").unwrap();
        prune_old_runs(&parent);
        assert!(green.exists() && red.exists(), "a fresh run is kept");

        let two_days = std::time::SystemTime::now() - Duration::from_secs(2 * 24 * 60 * 60);
        for dir in [&green, &red] {
            std::fs::File::open(dir)
                .unwrap()
                .set_modified(two_days)
                .unwrap();
        }
        prune_old_runs(&parent);
        assert!(!green.exists(), "a green run past a day goes");
        assert!(red.exists(), "a red run stays a week");
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn the_slow_tests_come_longest_first_and_junk_is_skipped() {
        let log = "21.5 a::slow\nbroken\n40.25 b::slower\nx c::not_seconds\n";
        assert_eq!(
            slowest_first(log),
            vec![(40.25, "b::slower"), (21.5, "a::slow")]
        );
        assert!(slowest_first("").is_empty());
    }
}
