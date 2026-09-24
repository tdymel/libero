//! Runner processes: the guard, the server's process group, and processes found by command line.

use std::io::{BufRead, Write};
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

/// Set on the guard process, which is this same binary.
pub(crate) const GUARD_ENV: &str = "E2E_GUARD";

/// `pid ppid state cmdline`; a ppid of 1 means it outlived its run.
pub(crate) fn describe_pid(pid: u32) -> String {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
    // `comm` may hold spaces and brackets: split after the last `)`.
    let after = stat.rsplit_once(')').map(|(_, rest)| rest).unwrap_or("");
    let mut fields = after.split_whitespace();
    let state = fields.next().unwrap_or("?");
    let ppid = fields.next().unwrap_or("?");
    let cmdline = std::fs::read(format!("/proc/{pid}/cmdline"))
        .map(|raw| String::from_utf8_lossy(&raw).replace('\0', " "))
        .unwrap_or_else(|_| "(gone)".into());
    format!("{pid} {ppid} {state} {}", cmdline.trim())
}

/// The runner's pipe to the guard. Closed without `done`, the guard cleans up.
pub(crate) struct Guard {
    child: Child,
    pipe: Option<ChildStdin>,
}

impl Guard {
    pub(crate) fn spawn() -> Result<Self> {
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
    pub(crate) fn tell(&mut self, line: &str) {
        if let Some(pipe) = &mut self.pipe {
            let _ = writeln!(pipe, "{line}");
        }
    }

    /// The runner cleaned up itself; the guard exits without doing anything.
    pub(crate) fn done(&mut self) {
        self.tell("done");
        self.pipe = None;
        let _ = self.child.wait();
    }
}

/// The guard process: on end-of-file without `done`, stops the process groups, then
/// Chrome on the profile, then deletes the profile.
pub(crate) fn guard() -> Result<()> {
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
        // A frozen Chromium ignores SIGTERM (todo 364).
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

/// Terminates the server's whole process group, cargo and rustc included.
pub(crate) fn stop(server: &mut Child) {
    let pid = server.id() as i32;
    unsafe { libc_kill(-pid, 15) };

    // Grace period, so dx and cargo leave no half-written target dir.
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

/// SIGTERMs every process whose command line contains `needle`. Reads `/proc`:
/// `pkill -f` also kills the shell carrying the pattern (exit 144).
pub(crate) fn kill_by_cmdline(needle: &str) -> usize {
    each_matching_pid(needle, |pid| unsafe {
        // SIGTERM, so Chrome flushes and removes its own lock file.
        libc_kill(pid as i32, 15);
    })
}

/// How many processes still match, for waiting on a kill to take effect.
pub(crate) fn count_by_cmdline(needle: &str) -> usize {
    each_matching_pid(needle, |_| {})
}

/// Which processes still match.
pub(crate) fn pids_by_cmdline(needle: &str) -> Vec<u32> {
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
    pub(crate) fn libc_kill(pid: i32, sig: i32) -> i32;
}
