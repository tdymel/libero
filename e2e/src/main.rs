//! The runner: `cargo run -p e2e`.
//!
//! Owns the fixture server's lifecycle so the tests do not have to, which is
//! what Playwright's `webServer` block does for the fork's suite. A Rust test
//! binary has no global teardown, so a server started from inside a test would
//! outlive the run - and this repo's brief is strict that a browser pass is not
//! done until its processes are gone.
//!
//! What it does, in order:
//!
//! 1. Picks a free port, so the suite never collides with a dev server.
//! 2. Starts `dx run` on the fixture crate. `run`, not `serve`: it builds once
//!    and starts no watcher, which is what makes the whole trap class in
//!    `codebase/testing` unreachable rather than merely avoidable.
//! 3. Waits for the app, not the socket. `dx` answers with a 404 placeholder at
//!    a success status while the first build runs, so a reachable port proves
//!    nothing.
//! 4. Runs the tests with `E2E_BASE_URL` set.
//! 5. Kills the server and exits with the tests' status, or with a failure
//!    when the name filter matched no test.
//!
//! A guard process covers the runner's own death (todo 313). SIGKILL cannot
//! be caught, so no signal handler in the runner can clean up after it. The
//! runner starts a copy of itself instead, in its own process group, and tells
//! it over a pipe what to stop. The kernel closes the pipe however the runner
//! dies (SIGKILL, SIGTERM, Ctrl-C, a panic, the OOM killer). A guard that
//! reads end-of-file without `done` stops the server, the tests and Chrome.
//! The guard does not help when it is killed as well: a kill of every
//! descendant, `kill -9 -1`, or a shutdown. Neither does a SIGKILL in the few
//! microseconds between spawning a process and telling the guard about it.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

mod android_runner;

/// Generous, because a cold cargo build happens inside this wait. The fork's
/// playwright config allows 50 minutes for the same reason; a stock 30 second
/// timeout fails every cold run and reads as a broken test rather than a slow
/// one.
const BUILD_TIMEOUT: Duration = Duration::from_secs(45 * 60);

/// Set on the guard process, which is this same binary.
const GUARD_ENV: &str = "E2E_GUARD";

/// The fixture app's `<title>`, from `e2e/fixtures/Dioxus.toml`. It is the one
/// string in the served body that neither dx's build splash nor somebody
/// else's app can produce, which is why `wait_for_app` waits for it.
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

    // Every build the runner starts goes where the runner itself was built.
    // Neither `dx run` nor the nested `cargo test` sees `--target-dir` on the
    // outer `cargo run`, so without this both fell back to the root config's
    // shared `target/main` (todo 328).
    let target_dir = own_target_dir()?;
    eprintln!("e2e: building into {}", target_dir.display());

    // Artifacts (failure screenshots, dx.log, the wait journal, the tests'
    // own output) go in a directory of this run's own. One shared directory,
    // emptied at start, let a second agent's run delete the first one's
    // evidence, and a `dx.log` read afterwards could belong to the other
    // worktree (todo 329).
    //
    // Keyed by **run**, not by port, since todo 364: a port comes round again
    // and the next run deleted the evidence of the red one before anybody had
    // read it. Both all-fail events lost their artifacts exactly this way.
    // Nothing is deleted at start any more - the name is new every time - and
    // `prune_old_runs` keeps a red run a week instead of a day.
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

    // The journal and the offsets in it need to be readable from the test
    // process too, which is a child of this one.
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
    let mut server = Command::new(&dx)
        .current_dir(root.join(app_dir))
        .args([
            "run",
            "--web",
            "--addr",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        // The env var, since `dx` takes no `--target-dir`. `RUSTC_WRAPPER` is
        // left alone: `dx` drives it itself for hot-patching.
        .env("CARGO_TARGET_DIR", &target_dir)
        .stdout(Stdio::from(
            log_handle.try_clone().context("clone the log handle")?,
        ))
        .stderr(Stdio::from(log_handle))
        // Its own process group, so killing it takes the cargo and rustc
        // children it spawned. Killing the `dx` pid alone leaves a multi-minute
        // wasm build running against the shared target directory, long after
        // the run that wanted it has gone.
        .process_group(0)
        .spawn()
        .context("start dx run")?;
    guard.tell(&format!("group {}", server.id()));

    let ready = wait_for_app(&base_url, title, &mut server, &dx_log);
    if let Err(error) = ready {
        stop(&mut server);
        guard.done();
        // Red before a single test ran. Marked all the same, so the directory
        // survives the prune and so the next reader sees which wait it was.
        mark_red(&artifacts, &format!("served-body: {error}"));
        return Err(error);
    }

    // A Chrome profile unique to this run. Chrome will not start against a
    // profile another process still holds, so sharing one means an interrupted
    // run poisons every run after it. Under the run's target dir, so seats never share one.
    let profile = target_dir.join(format!("e2e-chrome-{}", std::process::id()));
    guard.tell(&format!("profile {}", profile.display()));

    eprintln!("e2e: server is up, running the suite");
    // Everything after the runner's own name goes to the **test harness**, not
    // to cargo. Without the `--`, `cargo run -p e2e -- --nocapture` hands
    // `--nocapture` to cargo, which rejects it and prints a usage message -
    // and the run dies having told you nothing about your tests. libtest takes
    // a name filter after `--` just as happily, so both
    // `-- focus_contrast` and `-- --nocapture` work.
    let status = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["test", "-p", "e2e", "--test", test, "--target-dir"])
        .arg(&target_dir)
        // Before the `--`: after it, `--target-dir` would go to the test
        // binary and cargo would build into `target/main` without a word.
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
            // Counted separately from the summary line, because the verdict's
            // question is about **pages**: a unit that drives no browser
            // cannot be a sibling that stayed up (todo 364, `NON_BROWSER`).
            let mut browser = e2e::journal::Tally::default();
            let mut set_aside = 0u64;
            let stdout = tests.stdout.take().context("the tests' stdout")?;
            // Teed to a file as well as echoed. Both all-fail events were run
            // under a pipe through `tail`, so the per-test detail - which is
            // the only thing that says *how* they failed - was cut before
            // anybody read it, and the todo has said "no per-test output
            // survived" twice (todo 364).
            let mut log = std::fs::File::create(artifacts.join("test-output.log")).ok();
            for line in BufReader::new(stdout).lines() {
                let line = line.context("read the tests' output")?;
                tally += e2e::journal::Tally::from_summary(&line);
                if let Some((test, passed)) = e2e::journal::test_outcome(&line) {
                    match (e2e::journal::drives_a_browser(test), passed) {
                        (true, true) => browser.passed += 1,
                        (true, false) => browser.failed += 1,
                        (false, true) => set_aside += 1,
                        // A non-browser unit that failed is an ordinary
                        // failure and says nothing about the pages either,
                        // so it is neither counted nor set aside.
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

    // The browser lives in a `static` inside the test binary, and a static is
    // never dropped - so nothing kills Chrome when the tests end. Reap it here,
    // matched on this run's own profile path, which no other process can carry.
    //
    // On a red run, what is alive at this moment is evidence: the second
    // all-fail left eleven Chromiums with a dead parent and the runner still
    // printed `reaped 88` and exited 0 (todo 364). So the list is written down
    // before anything is signalled.
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

    // Wait for them to actually go before deleting the directory.
    //
    // `kill` only *asks*. Removing the profile while Chrome is still shutting
    // down and writing to it leaves a partially deleted directory behind, and
    // because the result was discarded nothing ever said so - three leaked
    // profiles were sitting in /tmp before this check was written. They are
    // small individually and unbounded in number.
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && count_by_cmdline(&needle) > 0 {
        std::thread::sleep(Duration::from_millis(100));
    }

    // `reaped N` used to be printed straight after the SIGTERM, so it said how
    // many processes were *asked* to go, not how many went - and it was being
    // read as proof the run tidied up. Verify, escalate, and if anything is
    // still there say which pids rather than claiming a number (todo 364).
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

    // Kill the server whatever the tests did, including a panic in the
    // command itself.
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

/// Mark the artifacts directory so `prune_old_runs` keeps it a week, and so
/// the next reader finds the verdict without rebuilding it from the log.
fn mark_red(artifacts: &Path, verdict: &str) {
    let _ = std::fs::write(
        artifacts.join("RED"),
        format!("{}\n{verdict}\n", e2e::journal::utc_now()),
    );
}

/// `pid ppid state cmdline` for one process, or a note that it is already
/// gone. The parent is the interesting column: a Chromium whose parent is 1
/// outlived the run that started it.
fn describe_pid(pid: u32) -> String {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
    // `comm` can contain spaces and brackets, so the fields after it are found
    // from the last `)`, not by splitting the whole line.
    let after = stat.rsplit_once(')').map(|(_, rest)| rest).unwrap_or("");
    let mut fields = after.split_whitespace();
    let state = fields.next().unwrap_or("?");
    let ppid = fields.next().unwrap_or("?");
    let cmdline = std::fs::read(format!("/proc/{pid}/cmdline"))
        .map(|raw| String::from_utf8_lossy(&raw).replace('\0', " "))
        .unwrap_or_else(|_| "(gone)".into());
    format!("{pid} {ppid} {state} {}", cmdline.trim())
}

/// The runner's side of the guard: the pipe it tells the guard things over.
/// Dropped without `done`, as when the runner dies, it tells the guard to
/// clean up.
struct Guard {
    child: Child,
    pipe: Option<ChildStdin>,
}

impl Guard {
    fn spawn() -> Result<Self> {
        let mut child = Command::new(std::env::current_exe().context("locate the runner")?)
            .env(GUARD_ENV, "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            // Out of the runner's group, so a Ctrl-C or a group kill that
            // takes the runner leaves the guard to clean up after it.
            .process_group(0)
            .spawn()
            .context("start the guard")?;
        let pipe = child.stdin.take();
        Ok(Self { child, pipe })
    }

    /// `group <pgid>` or `profile <path>`: one more thing to stop.
    fn tell(&mut self, line: &str) {
        if let Some(pipe) = &mut self.pipe {
            let _ = writeln!(pipe, "{line}");
        }
    }

    /// The runner cleaned up itself; the guard exits without doing anything.
    fn done(&mut self) {
        self.tell("done");
        self.pipe = None;
        let _ = self.child.wait();
    }
}

/// The guard process. Reads what to stop until `done` or end-of-file, and on
/// end-of-file stops it: the process groups first, then any Chrome left on
/// the profile, then the profile itself.
fn guard() -> Result<()> {
    let (mut groups, mut profiles) = (Vec::new(), Vec::new());
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        match line.split_once(' ') {
            Some(("group", pgid)) => groups.extend(pgid.parse::<i32>().ok()),
            Some(("profile", path)) => profiles.push(path.to_string()),
            _ if line == "done" => return Ok(()),
            _ => {}
        }
    }
    // The runner failed before it started anything.
    if groups.is_empty() && profiles.is_empty() {
        return Ok(());
    }
    // The terminal may be gone with the runner, so nothing here may panic on a
    // failed write, as `eprintln!` would.
    let say = |message: &str| {
        let _ = writeln!(std::io::stderr(), "e2e guard: {message}");
    };
    say("the runner died without cleaning up; stopping what it started");
    for &group in &groups {
        unsafe { libc_kill(-group, 15) };
    }
    let alive = |groups: &[i32]| groups.iter().any(|&g| unsafe { libc_kill(-g, 0) } == 0);
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && alive(&groups) {
        std::thread::sleep(Duration::from_millis(100));
    }
    for &group in &groups {
        unsafe { libc_kill(-group, 9) };
    }
    for profile in &profiles {
        kill_by_cmdline(profile);
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline && count_by_cmdline(profile) > 0 {
            std::thread::sleep(Duration::from_millis(100));
        }
        // SIGTERM only asks, and a Chromium whose main thread is frozen never
        // gets round to answering. The runner's own reap escalates for the
        // same reason (todo 364).
        for pid in pids_by_cmdline(profile) {
            unsafe { libc_kill(pid as i32, 9) };
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && count_by_cmdline(profile) > 0 {
            std::thread::sleep(Duration::from_millis(100));
        }
        let left = count_by_cmdline(profile);
        if left > 0 {
            say(&format!(
                "{left} browser process(es) on {profile} survived SIGKILL; leaving the profile"
            ));
        } else {
            let _ = std::fs::remove_dir_all(profile);
        }
    }
    say(&format!(
        "stopped {} process group(s) and the browser",
        groups.len()
    ));
    Ok(())
}

/// Bind port 0, read what the OS gave us, release it. There is a race between
/// releasing and `dx` binding, which is why this is preferable to a fixed port
/// only in a shared checkout - here it is what keeps the suite off the eight
/// dev servers running beside it.
fn free_port() -> Result<u16> {
    // `E2E_PORT` pins it, which is what a shared checkout wants when several
    // agents each own a port and have to prove theirs is free afterwards.
    //
    // Bound here rather than taken on trust (todo 364, 2026-09-20). A pinned
    // port that somebody else already held was accepted, `dx run` could not
    // have it, and the suite ran its whole filter against **that** server -
    // another agent's docs site - failing every test with nothing anywhere
    // saying the port was not ours.
    if let Ok(port) = std::env::var("E2E_PORT") {
        let port: u16 = port.parse().context("E2E_PORT is not a port number")?;
        TcpListener::bind(("127.0.0.1", port)).with_context(|| {
            format!("E2E_PORT={port} is already in use; that port belongs to something else")
        })?;
        return Ok(port);
    }
    let listener = TcpListener::bind("127.0.0.1:0").context("bind a free port")?;
    Ok(listener.local_addr()?.port())
}

/// Terminate the server and everything it spawned.
///
/// Negative pid is the process group, which is why the child was given one:
/// `dx run` drives cargo and rustc, and killing only `dx` leaves a wasm build
/// grinding away against the shared target directory.
fn stop(server: &mut Child) {
    let pid = server.id() as i32;
    unsafe { libc_kill(-pid, 15) };

    // A grace period before SIGKILL. Sending both back to back makes the
    // SIGTERM pointless: dx never gets to remove its own lock files or let
    // cargo finish writing, and a half-written target directory is a problem
    // for whoever builds next, not for this run.
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if matches!(server.try_wait(), Ok(Some(_))) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    unsafe { libc_kill(-pid, 9) };
    let _ = server.kill();
    let _ = server.wait();
}

/// Wait until the served page is the fixture app and not dx's placeholder.
///
/// Three ways this ends, and the second is the one worth having:
///
/// * the app answers, and we go;
/// * **`dx` exits**, which is what a compile error in the fixture crate looks
///   like. Without this check the runner sat out the full build timeout and
///   then reported a timeout, hiding a compile error behind a 45-minute wait;
/// * the deadline passes.
fn wait_for_app(
    base_url: &str,
    title: &str,
    server: &mut Child,
    dx_log: &std::path::Path,
) -> Result<()> {
    let started = Instant::now();
    let deadline = started + BUILD_TIMEOUT;
    let (mut polls, mut slowest) = (0u32, Duration::ZERO);
    // Named in the timeout so a failure says which link of the chain never
    // appeared, rather than only that something did not.
    let mut reached;
    loop {
        if let Ok(Some(status)) = server.try_wait() {
            let tail = std::fs::read_to_string(dx_log)
                .map(|log| log.lines().rev().take(30).collect::<Vec<_>>().join("\n"))
                .unwrap_or_default();
            bail!(
                "dx exited before the app was served ({status}). Last of {}:\n{tail}",
                dx_log.display()
            );
        }

        let at = Instant::now();
        let stage = app_readiness(base_url, title);
        polls += 1;
        slowest = slowest.max(at.elapsed());
        if stage == Readiness::Ready {
            return Ok(());
        }
        reached = stage;

        if Instant::now() >= deadline {
            // The first of the three waits todo 364 could not tell apart.
            e2e::journal::gave_up(&e2e::journal::GaveUp {
                kind: "served-body",
                how: "expired",
                what: &format!("the fixture app to be servable (got as far as {reached:?})"),
                budget: BUILD_TIMEOUT,
                elapsed: started.elapsed(),
                slowest_poll: slowest,
                polls,
            });
            bail!(
                "timed out after {:.1?} (budget {BUILD_TIMEOUT:?}) waiting for the fixture \
                 app to be servable; got as far as {reached:?} [wait=served-body, {polls} \
                 poll(s), slowest {slowest:.2?}]; see {}",
                started.elapsed(),
                dx_log.display()
            );
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

/// How far the served app has got. Ordered: each variant means every earlier
/// one already held.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Readiness {
    /// Nothing answered on the port.
    NoServer,
    /// Something answered, but not this app - dx's build splash, or an
    /// entirely different application on a port we do not own.
    NotOurApp,
    /// Our index.html, but its script is not being served yet.
    NoScript,
    /// The script, but the wasm bundle it names is not being served yet.
    NoBundle,
    /// The index, the script and the bundle are all servable.
    Ready,
}

/// Follow what the index references until something that only exists once the
/// build has finished.
///
/// Three checks were tried here and the first two were both wrong in the same
/// way (todo 364, 2026-09-20):
///
/// 1. *"The body does not say `dx is not serving a web app` and does contain
///    `wasm`."* A double negative that dx's **build splash** satisfies, and so
///    does any other app with a wasm bundle - another agent's docs site
///    satisfied it when `E2E_PORT` named a port this run did not own.
/// 2. *"The body contains the fixture app's `<title>`."* A positive assertion,
///    and still wrong: **dx serves the real `index.html` from the moment it
///    binds the port**, so the title is there about 44 s before the bundle is.
///    Measured: with this check the first navigation went out at +2.6 s and
///    the build finished at +46.5 s, and the suite failed all 87 browser
///    tests exactly as before.
///
/// The lesson is the one this whole todo keeps teaching: a marker the app
/// *emits* is not a marker that the app is *ready*. So this follows the chain
/// to the wasm bundle, which cannot exist before the build has produced it,
/// and it reads each link out of what was actually served rather than
/// hardcoding a path - a hardcoded one that quietly stopped matching would
/// make the check vacuous again, which is the failure mode being fixed.
fn app_readiness(base_url: &str, title: &str) -> Readiness {
    let Some(index) = http_get(base_url) else {
        return Readiness::NoServer;
    };
    if !index.contains(title) {
        return Readiness::NotOurApp;
    }
    // `.js`, not merely the first `src="`: an icon or an image would send the
    // rest of the chain looking at the wrong file, and the symptom would be a
    // 45-minute timeout rather than anything that names the cause.
    let Some(script) = quoted_ending_in(&index, ".js") else {
        return Readiness::NoScript;
    };
    let Some(js) = http_get(&join(base_url, "", &script)) else {
        return Readiness::NoScript;
    };
    if !is_ok(&js) {
        return Readiness::NoScript;
    }
    // wasm-bindgen's glue names the bundle. It appears twice and in different
    // shapes - `'e2e-fixtures_bg.wasm'` bare, and
    // `"/./wasm/e2e-fixtures_bg.wasm"` rooted - so `join` resolves either
    // against the script's own directory.
    let Some(bundle) = quoted_ending_in(&js, ".wasm") else {
        return Readiness::NoBundle;
    };
    let dir = script
        .trim_start_matches(['.', '/'])
        .rsplit_once('/')
        .map(|(dir, _)| dir)
        .unwrap_or("");
    match http_get(&join(base_url, dir, &bundle)) {
        Some(response) if is_ok(&response) => Readiness::Ready,
        _ => Readiness::NoBundle,
    }
}

/// A URL for `reference`, which is either rooted (`/./wasm/x.wasm`) or
/// relative to `dir`.
fn join(base_url: &str, dir: &str, reference: &str) -> String {
    let path = reference.trim_start_matches(['.', '/']);
    if reference.contains('/') || dir.is_empty() {
        format!("{base_url}/{path}")
    } else {
        format!("{base_url}/{dir}/{path}")
    }
}

/// The first single- or double-quoted run in `text` that ends with `suffix`.
fn quoted_ending_in(text: &str, suffix: &str) -> Option<String> {
    text.split(['"', '\''])
        .find(|part| part.ends_with(suffix) && !part.contains(['<', '>', ' ']))
        .map(str::to_string)
}

/// Whether a raw HTTP response carries a 2xx status.
fn is_ok(response: &str) -> bool {
    response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .is_some_and(|code| code.starts_with('2'))
}

/// A one-shot HTTP GET.
///
/// Hand-rolled rather than shelling out to `curl`: a test harness that fails
/// when a system binary is missing fails for a reason that has nothing to do
/// with the code under test, and the failure would read as "the server never
/// came up".
fn http_get(url: &str) -> Option<String> {
    // Host and path split apart. This used to hardcode `GET /` and hand the
    // whole remainder to `TcpStream::connect`, which was fine while `/` was
    // the only thing ever fetched - and silently returned `None` for every
    // URL with a path the moment the readiness check needed one, so the
    // runner waited out its 45-minute budget saying nothing (todo 364).
    let rest = url.strip_prefix("http://")?;
    let (address, path) = match rest.find('/') {
        Some(at) => (&rest[..at], &rest[at..]),
        None => (rest, "/"),
    };
    let mut stream = TcpStream::connect(address).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .ok()?;
    write!(
        stream,
        "GET {path} HTTP/1.0\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .ok()?;
    let mut body = Vec::new();
    stream.read_to_end(&mut body).ok()?;
    Some(String::from_utf8_lossy(&body).into_owned())
}

/// Kill every process whose command line contains `needle`.
///
/// Reads `/proc` rather than shelling out to `pkill -f`. `-f` matches the whole
/// command line, and the shell running the pkill carries the pattern in its own,
/// so pkill SIGTERMs that shell before the kill is reported (exit 144, measured
/// three times in this repo, most recently while building this harness).
/// Reading `/proc` and skipping our own pid has no such failure mode.
fn kill_by_cmdline(needle: &str) -> usize {
    each_matching_pid(needle, |pid| unsafe {
        // SIGTERM, so Chrome flushes and removes its own lock file.
        libc_kill(pid as i32, 15);
    })
}

/// How many processes still match, for waiting on a kill to take effect.
fn count_by_cmdline(needle: &str) -> usize {
    each_matching_pid(needle, |_| {})
}

/// Which processes still match. A count says a leak happened; the pids say
/// what leaked, which is what the second all-fail needed and did not have.
fn pids_by_cmdline(needle: &str) -> Vec<u32> {
    let mut pids = Vec::new();
    each_matching_pid(needle, |pid| pids.push(pid));
    pids
}

fn each_matching_pid(needle: &str, mut act: impl FnMut(u32)) -> usize {
    let mut matched = 0;
    let me = std::process::id();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return 0;
    };
    for entry in entries.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        if pid == me {
            continue;
        }
        let Ok(cmdline) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        if String::from_utf8_lossy(&cmdline).contains(needle) {
            act(pid);
            matched += 1;
        }
    }
    matched
}

unsafe extern "C" {
    #[link_name = "kill"]
    fn libc_kill(pid: i32, sig: i32) -> i32;
}

/// The target directory this runner was built into.
///
/// Cargo passes a `--target-dir` flag to nothing it runs, so the runner reads
/// it off its own path: the nearest ancestor of the executable holding the
/// `CACHEDIR.TAG` cargo writes at every target directory's root. That covers
/// the flag, `CARGO_TARGET_DIR`, the root config's default and a `--target`
/// triple alike.
fn own_target_dir() -> Result<std::path::PathBuf> {
    let exe = std::env::current_exe().context("locate the runner's executable")?;
    exe.ancestors()
        .skip(1)
        .find(|dir| dir.join("CACHEDIR.TAG").is_file())
        .map(Path::to_path_buf)
        .with_context(|| format!("no cargo target directory above {}", exe.display()))
}

/// Delete other runs' artifact directories once they are old enough.
///
/// Never a younger one: a parallel run may still be writing to it, or its owner
/// may not have read it yet. A day keeps /tmp from growing without bound.
///
/// A run that ended red carries a `RED` marker and is kept a week instead
/// (todo 364). The evidence from a red run is the scarce thing here; a green
/// run's artifacts have never been worth reading.
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

/// `$DX`, else `dx` on `PATH`, when it is the lockfile's dioxus version: an
/// older CLI speaks another protocol to the app it builds.
fn dx(root: &Path) -> Result<std::ffi::OsString> {
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

fn workspace_root() -> Result<std::path::PathBuf> {
    // `CARGO_MANIFEST_DIR` is `<root>/e2e`.
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(Path::to_path_buf)
        .context("no workspace root above the e2e crate")
}

use std::path::Path;
