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
//! 5. Kills the server and exits with the tests' status.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

/// Generous, because a cold cargo build happens inside this wait. The fork's
/// playwright config allows 50 minutes for the same reason; a stock 30 second
/// timeout fails every cold run and reads as a broken test rather than a slow
/// one.
const BUILD_TIMEOUT: Duration = Duration::from_secs(45 * 60);

fn main() -> Result<()> {
    let root = workspace_root()?;
    let port = free_port()?;
    let base_url = format!("http://127.0.0.1:{port}");

    let dx = root.join(".dioxus-active/target/release/dx");
    if !dx.exists() {
        bail!(
            "no dx at {}. Build it, or point `.dioxus-active` at a tree that has one.",
            dx.display()
        );
    }

    // Every build the runner starts goes where the runner itself was built.
    // Neither `dx run` nor the nested `cargo test` sees `--target-dir` on the
    // outer `cargo run`, so without this both fell back to the root config's
    // shared `target/main` (todo 328).
    let target_dir = own_target_dir()?;
    eprintln!("e2e: building into {}", target_dir.display());

    // Artifacts (failure screenshots, dx.log) go in a directory of this run's
    // own, named by its port. One shared directory, emptied at start, let a
    // second agent's run delete the first one's evidence, and a `dx.log` read
    // afterwards could belong to the other worktree (todo 329).
    let artifacts = std::env::temp_dir()
        .join("e2e-artifacts")
        .join(format!("port-{port}"));
    prune_old_runs(
        artifacts
            .parent()
            .context("the artifacts directory has a parent")?,
    );
    let _ = std::fs::remove_dir_all(&artifacts);
    std::fs::create_dir_all(&artifacts).context("create the artifacts directory")?;
    eprintln!("e2e: artifacts go to {}", artifacts.display());

    // dx's own output, kept rather than discarded: when the fixture crate fails
    // to compile, this file is the only place that says why.
    let dx_log = artifacts.join("dx.log");
    let log_handle = std::fs::File::create(&dx_log).context("create the dx log")?;

    // Eagerly, so a broken archive fails here rather than inside the first
    // contrast assertion, with the browser and server already running.
    let axe = e2e::vendor::ensure_axe().context("prepare the vendored axe archive")?;
    eprintln!("e2e: axe ready at {}", axe.display());

    eprintln!("e2e: starting the fixture server on {base_url}");
    let mut server = Command::new(&dx)
        .current_dir(root.join("e2e/fixtures"))
        .args([
            "run",
            "--web",
            "--addr",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        // Line tables only, for the same reason every other build in this repo
        // uses them: full debug info is most of what a build writes, and the
        // write load stalls the machine.
        .env("CARGO_PROFILE_DEV_DEBUG", "line-tables-only")
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

    let ready = wait_for_app(&base_url, &mut server, &dx_log);
    if let Err(error) = ready {
        stop(&mut server);
        return Err(error);
    }

    // A Chrome profile unique to this run. Chrome will not start against a
    // profile another process still holds, so sharing one means an interrupted
    // run poisons every run after it.
    let profile = std::env::temp_dir().join(format!("e2e-chrome-{}", std::process::id()));

    eprintln!("e2e: server is up, running the suite");
    // Everything after the runner's own name goes to the **test harness**, not
    // to cargo. Without the `--`, `cargo run -p e2e -- --nocapture` hands
    // `--nocapture` to cargo, which rejects it and prints a usage message -
    // and the run dies having told you nothing about your tests. libtest takes
    // a name filter after `--` just as happily, so both
    // `-- focus_contrast` and `-- --nocapture` work.
    let passthrough: Vec<String> = std::env::args().skip(1).collect();
    let status = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["test", "-p", "e2e", "--test", "all", "--target-dir"])
        .arg(&target_dir)
        // Before the `--`: after it, `--target-dir` would go to the test
        // binary and cargo would build into `target/main` without a word.
        .arg("--")
        .args(&passthrough)
        .env("E2E_BASE_URL", &base_url)
        .env("E2E_CHROME_PROFILE", &profile)
        .env("E2E_ARTIFACTS", &artifacts)
        .env("CARGO_PROFILE_DEV_DEBUG", "line-tables-only")
        .status()
        .context("run the tests");

    // The browser lives in a `static` inside the test binary, and a static is
    // never dropped - so nothing kills Chrome when the tests end. Reap it here,
    // matched on this run's own profile path, which no other process can carry.
    let needle = profile.to_string_lossy().into_owned();
    let reaped = kill_by_cmdline(&needle);
    if reaped > 0 {
        eprintln!("e2e: reaped {reaped} browser process(es)");
    }

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

    if let Err(error) = std::fs::remove_dir_all(&profile)
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

    if !status?.success() {
        std::process::exit(1);
    }
    Ok(())
}

/// Bind port 0, read what the OS gave us, release it. There is a race between
/// releasing and `dx` binding, which is why this is preferable to a fixed port
/// only in a shared checkout - here it is what keeps the suite off the eight
/// dev servers running beside it.
fn free_port() -> Result<u16> {
    // `E2E_PORT` pins it, which is what a shared checkout wants when several
    // agents each own a port and have to prove theirs is free afterwards.
    if let Ok(port) = std::env::var("E2E_PORT") {
        return port.parse().context("E2E_PORT is not a port number");
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
fn wait_for_app(base_url: &str, server: &mut Child, dx_log: &std::path::Path) -> Result<()> {
    let deadline = Instant::now() + BUILD_TIMEOUT;
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

        if let Some(body) = http_get(base_url) {
            // The placeholder is served with a success status, so the check is
            // on what came back, not on whether anything came back.
            let placeholder = body.contains("dx is not serving a web app");
            if !placeholder && body.contains("wasm") {
                return Ok(());
            }
        }

        if Instant::now() >= deadline {
            bail!(
                "the fixture server never finished building (waited {BUILD_TIMEOUT:?}); see {}",
                dx_log.display()
            );
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

/// A one-shot HTTP GET.
///
/// Hand-rolled rather than shelling out to `curl`: a test harness that fails
/// when a system binary is missing fails for a reason that has nothing to do
/// with the code under test, and the failure would read as "the server never
/// came up".
fn http_get(base_url: &str) -> Option<String> {
    let address = base_url.strip_prefix("http://")?;
    let mut stream = TcpStream::connect(address).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .ok()?;
    write!(
        stream,
        "GET / HTTP/1.0\r\nHost: {address}\r\nConnection: close\r\n\r\n"
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

/// Delete other runs' artifact directories once they are a day old.
///
/// Never a younger one: a parallel run may still be writing to it, or its owner
/// may not have read it yet. A day keeps /tmp from growing without bound.
fn prune_old_runs(parent: &Path) {
    const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let old = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|at| at.elapsed().ok())
            .is_some_and(|age| age > MAX_AGE);
        if old && entry.path().is_dir() {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
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
