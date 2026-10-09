//! Runner processes: the guard, the server's process group, and processes found by command line.
//! Unix signals process groups; Windows ends process trees with `taskkill` (todo 2781).

use std::io::{BufRead, Write};
#[cfg(unix)]
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

/// Set on the guard process, which is this same binary.
pub(crate) const GUARD_ENV: &str = "E2E_GUARD";

/// `pid ppid state cmdline`; a ppid of 1 means it outlived its run.
#[cfg(unix)]
pub(crate) fn describe_pid(pid: u32) -> String {
    match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(stat) => {
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
        Err(_) => describe_with_ps(pid),
    }
}

/// `pid ppid ? cmdline`: Windows has no process state to show.
#[cfg(windows)]
pub(crate) fn describe_pid(pid: u32) -> String {
    match windows::processes(Some(pid)).into_iter().next() {
        Some((pid, ppid, cmdline)) => format!("{pid} {ppid} ? {}", cmdline.trim()),
        None => format!("{pid} ? ? (gone)"),
    }
}

/// Where there is no `/proc`: the same four fields from `ps`.
#[cfg(unix)]
fn describe_with_ps(pid: u32) -> String {
    let listing = Command::new("ps")
        .args(["-o", "pid=,ppid=,state=,command=", "-p", &pid.to_string()])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    let line = listing.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.is_empty() {
        format!("{pid} ? ? (gone)")
    } else {
        line
    }
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
            .own_group()
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
    guard_from(std::io::stdin().lock())
}

fn guard_from(input: impl BufRead) -> Result<()> {
    let (mut groups, mut profiles) = (Vec::new(), Vec::new());
    for line in input.lines() {
        let Ok(line) = line else { break };
        match line.split_once(' ') {
            Some(("group", pgid)) => groups.extend(pgid.parse::<u32>().ok()),
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
        signal_group(group, Signal::Term);
    }
    wait_until(Duration::from_secs(5), || {
        !groups.iter().any(|&group| group_alive(group))
    });
    for &group in &groups {
        signal_group(group, Signal::Kill);
    }
    for profile in &profiles {
        kill_by_cmdline(profile);
        wait_until(Duration::from_secs(10), || count_by_cmdline(profile) == 0);
        // A frozen Chromium ignores SIGTERM (todo 364).
        for pid in pids_by_cmdline(profile) {
            signal_pid(pid, Signal::Kill);
        }
        wait_until(Duration::from_secs(5), || count_by_cmdline(profile) == 0);
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

/// Polls `done` every 100 ms for up to `within`; whether it held.
pub(crate) fn wait_until(within: Duration, mut done: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + within;
    while !done() {
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    true
}

/// Terminates the server's whole process group, cargo and rustc included.
pub(crate) fn stop(server: &mut Child) {
    let pid = server.id();
    signal_group(pid, Signal::Term);

    // Grace period, so dx and cargo leave no half-written target dir.
    if wait_until(Duration::from_secs(5), || {
        matches!(server.try_wait(), Ok(Some(_)))
    }) {
        return;
    }

    signal_group(pid, Signal::Kill);
    let _ = server.kill();
    let _ = server.wait();
}

/// SIGTERMs every process whose command line contains `needle`. Scans the process
/// table: `pkill -f` also kills the shell carrying the pattern (exit 144).
pub(crate) fn kill_by_cmdline(needle: &str) -> usize {
    // SIGTERM, so Chrome flushes and removes its own lock file.
    each_matching_pid(needle, |pid| signal_pid(pid, Signal::Term))
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
    let me = std::process::id();
    let matching: Vec<u32> = processes()
        .into_iter()
        .filter(|(pid, cmdline)| *pid != me && cmdline.contains(needle))
        .map(|(pid, _)| pid)
        .collect();
    matching.iter().for_each(|&pid| act(pid));
    matching.len()
}

/// Every process's pid and command line: `/proc` where there is one, else `ps`.
#[cfg(unix)]
fn processes() -> Vec<(u32, String)> {
    if Path::new("/proc/self").exists() {
        proc_processes()
    } else {
        ps_processes()
    }
}

#[cfg(windows)]
fn processes() -> Vec<(u32, String)> {
    windows::processes(None)
        .into_iter()
        .map(|(pid, _, cmdline)| (pid, cmdline))
        .collect()
}

#[cfg(unix)]
fn proc_processes() -> Vec<(u32, String)> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let pid = entry.file_name().to_string_lossy().parse::<u32>().ok()?;
            let cmdline = std::fs::read(entry.path().join("cmdline")).ok()?;
            Some((pid, String::from_utf8_lossy(&cmdline).into_owned()))
        })
        .collect()
}

/// macOS and the BSDs have no `/proc`; `-ww` keeps long command lines whole.
#[cfg(unix)]
fn ps_processes() -> Vec<(u32, String)> {
    Command::new("ps")
        .args(["-axww", "-o", "pid=,command="])
        .output()
        .map(|output| parse_ps(&String::from_utf8_lossy(&output.stdout)))
        .unwrap_or_default()
}

/// `  123 /usr/bin/x --flag`, one process per line.
#[cfg_attr(windows, allow(dead_code))]
fn parse_ps(listing: &str) -> Vec<(u32, String)> {
    listing
        .lines()
        .filter_map(|line| {
            let (pid, command) = line.trim_start().split_once(' ')?;
            Some((pid.parse().ok()?, command.trim().to_string()))
        })
        .collect()
}

/// Spawns into a process group of its own, so [`stop`] and the guard end its children too.
pub(crate) trait OwnGroup {
    fn own_group(&mut self) -> &mut Self;
}

impl OwnGroup for Command {
    #[cfg(unix)]
    fn own_group(&mut self) -> &mut Self {
        std::os::unix::process::CommandExt::process_group(self, 0)
    }

    /// `CREATE_NEW_PROCESS_GROUP`: the console's Ctrl-C does not reach it; `taskkill /T`
    /// follows the parent links, so the group needs no other bookkeeping.
    #[cfg(windows)]
    fn own_group(&mut self) -> &mut Self {
        std::os::windows::process::CommandExt::creation_flags(self, 0x0000_0200)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Signal {
    Term,
    Kill,
}

#[cfg(unix)]
impl Signal {
    fn number(self) -> i32 {
        match self {
            Signal::Term => 15,
            Signal::Kill => 9,
        }
    }
}

/// Signals the process group `pgid` leads.
#[cfg(unix)]
pub(crate) fn signal_group(pgid: u32, signal: Signal) {
    unsafe { libc_kill(-(pgid as i32), signal.number()) };
}

#[cfg(unix)]
pub(crate) fn group_alive(pgid: u32) -> bool {
    unsafe { libc_kill(-(pgid as i32), 0) == 0 }
}

#[cfg(unix)]
pub(crate) fn signal_pid(pid: u32, signal: Signal) {
    unsafe { libc_kill(pid as i32, signal.number()) };
}

#[cfg(unix)]
pub(crate) fn pid_alive(pid: u32) -> bool {
    unsafe { libc_kill(pid as i32, 0) == 0 }
}

#[cfg(unix)]
unsafe extern "C" {
    #[link_name = "kill"]
    fn libc_kill(pid: i32, sig: i32) -> i32;
}

/// A console process has no SIGTERM to catch, so both signals end the tree at once.
#[cfg(windows)]
pub(crate) fn signal_group(root: u32, _: Signal) {
    windows::taskkill(root, true);
}

/// The tree's root: `taskkill /T` ends its descendants with it.
#[cfg(windows)]
pub(crate) fn group_alive(root: u32) -> bool {
    pid_alive(root)
}

#[cfg(windows)]
pub(crate) fn signal_pid(pid: u32, _: Signal) {
    windows::taskkill(pid, false);
}

#[cfg(windows)]
pub(crate) fn pid_alive(pid: u32) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
        .output()
        .is_ok_and(|output| String::from_utf8_lossy(&output.stdout).contains(&format!("\"{pid}\"")))
}

#[cfg(windows)]
mod windows {
    use std::process::{Command, Stdio};

    /// `/F` always: a windowless process ignores the polite close `taskkill` sends otherwise.
    pub(super) fn taskkill(pid: u32, tree: bool) {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .args(tree.then_some("/T"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    /// `(pid, ppid, cmdline)` from CIM, of one process or all; `wmic` is gone on Server 2025.
    pub(super) fn processes(pid: Option<u32>) -> Vec<(u32, u32, String)> {
        let filter = pid
            .map(|pid| format!(" -Filter 'ProcessId={pid}'"))
            .unwrap_or_default();
        // `[Console]::Out`, not the pipeline: PowerShell wraps pipeline text at the console width.
        let script = format!(
            "Get-CimInstance Win32_Process{filter} | ForEach-Object {{ \
             [Console]::Out.WriteLine([string]$_.ProcessId + ' ' + $_.ParentProcessId + ' ' + $_.CommandLine) }}"
        );
        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .stderr(Stdio::null())
            .output()
            .map(|output| super::parse_cim(&String::from_utf8_lossy(&output.stdout)))
            .unwrap_or_default()
    }
}

/// `pid ppid cmdline` per line, as the CIM query writes it; a system process has no command line.
#[cfg_attr(unix, allow(dead_code))]
fn parse_cim(listing: &str) -> Vec<(u32, u32, String)> {
    listing
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, ' ');
            let pid = fields.next()?.trim().parse().ok()?;
            let ppid = fields.next()?.parse().ok()?;
            Some((pid, ppid, fields.next().unwrap_or("").trim().to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shell blocked on its stdin with `tag` on its command line; `tag` is unique per test.
    #[cfg(unix)]
    fn tagged(tag: &str) -> Child {
        Command::new("sh")
            .args(["-c", "read line", tag])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap()
    }

    #[cfg(unix)]
    fn needle(test: &str) -> String {
        format!("e2e-process-test-{test}-{}", std::process::id())
    }

    #[test]
    fn ps_lines_split_into_pid_and_command() {
        let listing = "    1 /sbin/init splash\n  4242 sh -c read line tag\n\nnot-a-pid x\n";
        assert_eq!(
            parse_ps(listing),
            vec![
                (1, "/sbin/init splash".to_string()),
                (4242, "sh -c read line tag".to_string()),
            ]
        );
    }

    #[test]
    fn cim_lines_split_into_pid_parent_and_command() {
        let listing = "4 0 \r\n812 640 C:\\chrome.exe --user-data-dir=C:\\t\\p\r\n\r\nx 1 y\r\n";
        assert_eq!(
            parse_cim(listing),
            vec![
                (4, 0, String::new()),
                (
                    812,
                    640,
                    "C:\\chrome.exe --user-data-dir=C:\\t\\p".to_string()
                ),
            ]
        );
    }

    #[test]
    #[cfg(unix)]
    fn a_tagged_process_is_found_and_ended_by_its_command_line() {
        let tag = needle("find");
        let mut child = tagged(&tag);
        // The command line shows a moment after the spawn returns.
        assert!(wait_until(Duration::from_secs(5), || count_by_cmdline(
            &tag
        ) == 1));
        assert_eq!(pids_by_cmdline(&tag), vec![child.id()]);
        assert_eq!(kill_by_cmdline(&tag), 1);
        child.wait().unwrap();
        assert_eq!(count_by_cmdline(&tag), 0);
    }

    #[test]
    #[cfg(unix)]
    fn a_process_is_described_with_its_parent_state_and_command() {
        let tag = needle("describe");
        let mut child = tagged(&tag);
        assert!(wait_until(Duration::from_secs(5), || {
            describe_pid(child.id()).contains(&tag)
        }));
        let line = describe_pid(child.id());
        let _ = child.kill();
        let _ = child.wait();
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(fields[0], child.id().to_string());
        assert_eq!(fields[1], std::process::id().to_string());
        assert!(line.contains(&tag), "{line}");
        assert!(describe_pid(u32::MAX - 1).ends_with("(gone)"));
    }

    #[test]
    #[cfg(unix)]
    fn stop_ends_the_whole_group() {
        let mut server = Command::new("sh")
            .args(["-c", "sleep 60 & wait"])
            .own_group()
            .spawn()
            .unwrap();
        let group = server.id();
        stop(&mut server);
        assert!(matches!(server.try_wait(), Ok(Some(_))));
        assert!(wait_until(Duration::from_secs(5), || !group_alive(group)));
    }

    #[test]
    fn wait_until_gives_up_at_its_deadline() {
        assert!(wait_until(Duration::ZERO, || true));
        let started = Instant::now();
        assert!(!wait_until(Duration::from_millis(250), || false));
        assert!(started.elapsed() >= Duration::from_millis(250));
    }

    #[test]
    fn a_guard_told_nothing_or_done_stops_nothing() {
        guard_from("".as_bytes()).unwrap();
        guard_from("group 1\ndone\n".as_bytes()).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn a_guard_whose_runner_died_stops_its_group_and_browser() {
        let tag = needle("guard");
        let mut group = Command::new("sh")
            .args(["-c", "sleep 60 & wait", &tag])
            .own_group()
            .spawn()
            .unwrap();
        let profile = std::env::temp_dir().join(&tag);
        std::fs::create_dir_all(&profile).unwrap();
        // Chrome carries its profile directory on the command line.
        let mut browser = tagged(profile.to_str().unwrap());
        let on_profile = profile.to_string_lossy().into_owned();
        assert!(wait_until(Duration::from_secs(5), || {
            count_by_cmdline(&on_profile) == 1
        }));
        let told = format!("group {}\nprofile {}\n", group.id(), profile.display());
        // Reaps what the guard kills, so none lingers as a zombie in its group.
        let reaper = std::thread::spawn(move || {
            let _ = group.wait();
            let _ = browser.wait();
        });
        guard_from(told.as_bytes()).unwrap();
        reaper.join().unwrap();
        assert!(!profile.exists());
    }
}
