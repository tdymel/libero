//! A cache in front of dx's `wasm-bindgen` (todo 1617): dx reruns it on every build, 12-16 s,
//! even when the fixture wasm did not change. The runner's own executable stands in for it.

use std::ffi::OsString;
use std::hash::Hasher;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};

use crate::dx::locked_version;
use crate::process::pid_alive;

/// The name dx runs: the runner acts as the cache when started under it.
pub(crate) const NAME: &str = "wasm-bindgen";
const REAL_ENV: &str = "E2E_BINDGEN_REAL";
const CACHE_ENV: &str = "E2E_BINDGEN_CACHE";
const REPORT_ENV: &str = "E2E_BINDGEN_REPORT";
/// Outputs kept per target dir, newest first: ~35 MB each.
const KEEP: usize = 3;
/// A copy in progress, suffixed with its runner's pid.
const PARTIAL: &str = "partial-";

/// Whether this process was started as `wasm-bindgen`.
pub(crate) fn is_wrapper() -> bool {
    std::env::args_os()
        .next()
        .is_some_and(|arg0| Path::new(&arg0).file_name() == Some(NAME.as_ref()))
}

/// Gives `dx` a home whose locked bindgen is the cache, and returns the file the cache
/// reports to. None when dx has not installed that bindgen yet: dx then downloads it as before.
pub(crate) fn wrap(dx: &mut Command, root: &Path, target_dir: &Path) -> Result<Option<PathBuf>> {
    // Windows: a symlink needs developer mode, and dx's tool is `wasm-bindgen.exe`.
    if cfg!(not(unix)) {
        return Ok(None);
    }
    let tool = format!("{NAME}-{}", locked_version(root, NAME)?);
    let real_home = dx_home();
    let real = real_home.join("tools").join(&tool).join(NAME);
    if !real.is_file() {
        return Ok(None);
    }
    // dx's home mirrored by links (settings, other tools), but for this one bindgen.
    let dir = target_dir.join("e2e-bindgen");
    let home = dir.join("home");
    let ours = home.join("tools").join(&tool);
    std::fs::create_dir_all(&ours).context("create the bindgen cache")?;
    link_entries(&real_home, &home, "tools");
    link_entries(&real_home.join("tools"), &home.join("tools"), &tool);
    // Renamed into place, so a run starting beside this one never sees no link.
    let exe = std::env::current_exe().context("locate the runner's executable")?;
    let link = ours.join(format!(".{NAME}-{}", std::process::id()));
    let _ = std::fs::remove_file(&link);
    symlink(&exe, &link).context("link the bindgen cache")?;
    std::fs::rename(&link, ours.join(NAME)).context("link the bindgen cache")?;

    let report = dir.join(format!("report-{}", std::process::id()));
    let _ = std::fs::remove_file(&report);
    dx.env("DX_HOME", &home)
        .env(REAL_ENV, &real)
        .env(CACHE_ENV, dir.join("out"))
        .env(REPORT_ENV, &report);
    Ok(Some(report))
}

/// Links every entry of `from` but `except` into `to`; links already there stay.
fn link_entries(from: &Path, to: &Path, except: &str) {
    let Ok(entries) = std::fs::read_dir(from) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name() != except {
            let _ = symlink(&entry.path(), &to.join(entry.file_name()));
        }
    }
}

#[cfg(unix)]
fn symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(not(unix))]
fn symlink(_: &Path, _: &Path) -> std::io::Result<()> {
    Err(std::io::ErrorKind::Unsupported.into())
}

/// Replaces this process with `command`; returns only on failure.
#[cfg(unix)]
fn exec(command: &mut Command) -> anyhow::Error {
    std::os::unix::process::CommandExt::exec(command).into()
}

/// No `exec` here: runs `command` and exits with its code.
#[cfg(not(unix))]
fn exec(command: &mut Command) -> anyhow::Error {
    match command.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => error.into(),
    }
}

/// What the cache did for this run's build, for the runner's log.
pub(crate) fn outcome(report: Option<&Path>) -> String {
    let said = report
        .and_then(|report| std::fs::read_to_string(report).ok())
        .map(|said| said.trim().to_string())
        .unwrap_or_else(|| "not cached".into());
    if let Some(report) = report {
        let _ = std::fs::remove_file(report);
    }
    said
}

/// Runs as `wasm-bindgen`: copies the output kept for the same input and arguments, else runs
/// the real one and keeps its output.
pub(crate) fn run() -> Result<()> {
    let real = std::env::var_os(REAL_ENV).context("E2E_BINDGEN_REAL is not set")?;
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let out_dir = args
        .iter()
        .position(|arg| arg == "--out-dir")
        .and_then(|at| args.get(at + 1))
        .map(PathBuf::from);
    let (Some(cache), Some(out_dir), Some(input)) =
        (std::env::var_os(CACHE_ENV), out_dir, args.last())
    else {
        // `--version` and anything else that writes nothing.
        return Err(exec(Command::new(&real).args(&args)));
    };
    let cache = PathBuf::from(cache);
    if let Err(error) = std::fs::create_dir_all(&cache) {
        eprintln!("wasm-bindgen cache: {error}; running the real one uncached");
        return Err(exec(Command::new(&real).args(&args)));
    }
    let started = Instant::now();
    let entry = cache.join(key(&real, &args, Path::new(input))?);
    // Said before the output lands: the runner may find the app served at once.
    let report = |said: String| {
        if let Some(report) = std::env::var_os(REPORT_ENV) {
            let _ = std::fs::write(report, said);
        }
    };
    if entry.is_dir() {
        report(format!("cached, {} ms", started.elapsed().as_millis()));
        // Touched first: its age orders the prune, so a runner pruning beside this one keeps it.
        let _ = std::fs::File::open(&entry)
            .and_then(|dir| dir.set_modified(std::time::SystemTime::now()));
        match copy_tree(&entry, &out_dir) {
            Ok(()) => return Ok(()),
            Err(error) => eprintln!("wasm-bindgen cache: {error:#}; running the real one"),
        }
    }
    report("ran".into());
    let status = Command::new(&real).args(&args).status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    report(format!("ran, {} s", started.elapsed().as_secs()));
    // The real output is in place; a cache that cannot keep it only costs the next run.
    if let Err(error) = keep(&cache, &out_dir, &entry) {
        eprintln!("wasm-bindgen cache: output not kept: {error:#}");
    }
    prune(&cache);
    Ok(())
}

/// Copies `out_dir` to `entry` through a `partial-<pid>` dir, removed whatever happens.
fn keep(cache: &Path, out_dir: &Path, entry: &Path) -> Result<()> {
    let partial = cache.join(format!("{PARTIAL}{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&partial);
    let kept = copy_tree(out_dir, &partial).and_then(|()| {
        // Another runner may have kept the same entry first.
        match std::fs::rename(&partial, entry) {
            Err(_) if entry.is_dir() => Ok(()),
            renamed => renamed.context("move the output into the cache"),
        }
    });
    let _ = std::fs::remove_dir_all(&partial);
    kept
}

/// The input wasm's bytes, the binary, and the arguments but the two paths.
fn key(real: &OsString, args: &[OsString], input: &Path) -> Result<String> {
    let mut hasher = std::hash::DefaultHasher::new();
    hasher.write(real.as_encoded_bytes());
    let mut skip = false;
    for arg in &args[..args.len() - 1] {
        if !std::mem::take(&mut skip) {
            hasher.write(arg.as_encoded_bytes());
        }
        skip = arg == "--out-dir";
    }
    let mut file = std::fs::File::open(input).context("open the bindgen input")?;
    let mut buffer = vec![0; 1 << 20];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.write(&buffer[..read]);
    }
    Ok(format!("{:016x}", hasher.finish()))
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)
                .with_context(|| format!("copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}

/// Keeps the `KEEP` newest outputs and drops dead runners' partial copies.
fn prune(cache: &Path) {
    let Ok(entries) = std::fs::read_dir(cache) else {
        return;
    };
    let mut kept: Vec<_> = entries
        .flatten()
        .filter(|entry| {
            let name = entry.file_name();
            let Some(pid) = name
                .to_string_lossy()
                .strip_prefix(PARTIAL)
                .map(str::to_owned)
            else {
                return true;
            };
            // A killed runner's copy: nothing else removes it.
            if !pid.parse().is_ok_and(pid_alive) {
                let _ = std::fs::remove_dir_all(entry.path());
            }
            false
        })
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .collect();
    kept.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
    for (_, path) in kept.into_iter().skip(KEEP) {
        let _ = std::fs::remove_dir_all(path);
    }
}

/// dx's home: `$DX_HOME`, else `.dx` in the XDG data dir.
fn dx_home() -> PathBuf {
    std::env::var_os("DX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
                })
                .join(".dx")
        })
}
