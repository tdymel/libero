//! `cargo run -p e2e -- ios [filter]` (2784): the scenarios' `ios` arm against the fixture
//! app in an iOS simulator, on macOS. The driver launches the app per unit; AXe (idb with
//! `E2E_IOS_INPUT=idb`) injects the touches and keys.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use super::{
    android_runner::list_arm,
    dx::{dx, workspace_root},
    own_target_dir,
    process::{Guard, OwnGroup, stop},
};

/// `--ignored` runs only the `ios: skip` scenarios.
pub fn run(mut filters: Vec<String>) -> Result<()> {
    let before = filters.len();
    filters.retain(|filter| filter != "--ignored");
    let ignored = filters.len() != before;
    // `e2e::driver::ios::INPUT_ENV`: the runner is built without the feature.
    let input = std::env::var("E2E_IOS_INPUT").unwrap_or_else(|_| "axe".into());
    for tool in ["xcrun", input.as_str()] {
        let found = Command::new("which").arg(tool).output()?;
        if !found.status.success() {
            bail!("the ios arm needs {tool} on PATH");
        }
    }
    let root = workspace_root()?;
    let dx = dx(&root)?;
    let target_dir = own_target_dir()?;
    let artifacts = std::env::temp_dir()
        .join("e2e-artifacts")
        .join(format!("ios-pid{}", std::process::id()));
    std::fs::create_dir_all(&artifacts).context("create the artifacts directory")?;
    eprintln!("e2e ios: artifacts go to {}", artifacts.display());

    // Before the listing: its `cargo test` relinks this binary, which the guard re-execs.
    let mut guard = Guard::spawn()?;
    // `E2E_IOS_UDID` picks a booted simulator; else the newest runtime's first iPhone boots.
    let (udid, booted_here) = match std::env::var("E2E_IOS_UDID") {
        Ok(udid) => (udid, false),
        Err(_) => {
            let devices = simctl(&["list", "-j", "devices", "available"])?;
            let wanted = std::env::var("E2E_IOS_DEVICE").ok();
            let (udid, name, runtime) = pick_device(&devices, wanted.as_deref())?;
            eprintln!("e2e ios: booting {name} ({runtime}), {udid}");
            // Already booted fails with a message, not a broken device.
            let _ = simctl(&["boot", &udid]);
            simctl(&["bootstatus", &udid, "-b"])?;
            (udid, true)
        }
    };

    let outcome = run_on(
        &udid,
        &root,
        &dx,
        &target_dir,
        &artifacts,
        &filters,
        ignored,
        &mut guard,
    );

    if booted_here {
        let _ = simctl(&["shutdown", &udid]);
        eprintln!("e2e ios: simulator shut down");
    }
    guard.done();
    outcome
}

#[allow(clippy::too_many_arguments)]
fn run_on(
    udid: &str,
    root: &Path,
    dx: &std::ffi::OsStr,
    target_dir: &Path,
    artifacts: &Path,
    filters: &[String],
    ignored: bool,
    guard: &mut Guard,
) -> Result<()> {
    let triple = if cfg!(target_arch = "aarch64") {
        "aarch64-apple-ios-sim"
    } else {
        "x86_64-apple-ios"
    };
    eprintln!("e2e ios: building the fixture app ({triple}), log in dx.log");
    let log = std::fs::File::create(artifacts.join("dx.log"))?;
    let status = Command::new(dx)
        .current_dir(root.join("e2e/fixtures"))
        .args([
            "build",
            "--platform",
            "ios",
            "--features",
            "ios",
            "--target",
        ])
        .arg(triple)
        .env("CARGO_TARGET_DIR", target_dir)
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .status()
        .context("run dx build")?;
    if !status.success() {
        bail!(
            "dx build failed, see {}",
            artifacts.join("dx.log").display()
        );
    }
    let app = find_app(&target_dir.join("dx/e2e-fixtures/debug/ios"))?;
    let plist = app.join("Info.plist");
    let bundle = Command::new("plutil")
        .args(["-extract", "CFBundleIdentifier", "raw", "-o", "-"])
        .arg(&plist)
        .output()
        .context("run plutil")?;
    let bundle = String::from_utf8_lossy(&bundle.stdout).trim().to_string();
    if bundle.is_empty() {
        bail!("no CFBundleIdentifier in {}", plist.display());
    }
    eprintln!("e2e ios: installing {} ({bundle})", app.display());
    simctl(&["install", udid, &app.to_string_lossy()])?;
    // `use_geolocation`: no driver answers its dialog, so grant up front, as on Android.
    let _ = simctl(&["privacy", udid, "grant", "location", &bundle]);

    let names = list_arm("ios", root, target_dir, filters, ignored)?;
    let names: Vec<String> = if ignored {
        names
    } else {
        let skipped = list_arm("ios", root, target_dir, filters, true)?;
        names
            .into_iter()
            .filter(|name| !skipped.contains(name))
            .collect()
    };
    if names.is_empty() {
        bail!("no ios scenario matches {filters:?}");
    }
    eprintln!("e2e ios: running {} scenario(s)", names.len());

    // One test process per unit, under a deadline: a wedged injector or bridge hangs a read.
    let mut red = Vec::new();
    for unit in names.chunk_by(|a, b| a.split("::").next() == b.split("::").next()) {
        let name = unit[0].split("::").next().unwrap_or_default();
        let passed = run_unit(
            udid, &bundle, root, target_dir, artifacts, unit, ignored, guard,
        )?;
        if !passed {
            let shot = artifacts.join(format!("{name}.png"));
            let _ = simctl(&["io", udid, "screenshot", &shot.to_string_lossy()]);
            red.push(name);
        }
    }
    let _ = simctl(&["terminate", udid, &bundle]);
    if !red.is_empty() {
        bail!("ios scenarios failed in: {}", red.join(", "));
    }
    Ok(())
}

/// One unit's scenarios in one test process; whether they passed.
#[allow(clippy::too_many_arguments)]
fn run_unit(
    udid: &str,
    bundle: &str,
    root: &Path,
    target_dir: &Path,
    artifacts: &Path,
    unit: &[String],
    ignored: bool,
    guard: &mut Guard,
) -> Result<bool> {
    let mut tests = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "test",
            "-q",
            "-p",
            "e2e",
            "--features",
            "ios",
            "--test",
            "all",
            "--target-dir",
        ])
        .arg(target_dir)
        .args(["--", "--exact", "--test-threads=1", "--format", "pretty"])
        .args(ignored.then_some("--ignored"))
        .args(unit)
        // `e2e::driver::ios::{UDID_ENV, BUNDLE_ENV}`.
        .env("E2E_IOS_UDID", udid)
        .env("E2E_IOS_BUNDLE", bundle)
        .env("E2E_ARTIFACTS", artifacts)
        .stdin(Stdio::null())
        .own_group()
        .spawn()
        .context("run the tests")?;
    guard.tell(&format!("group {}", tests.id()));
    let deadline = Instant::now() + Duration::from_secs(120 + 30 * unit.len() as u64);
    loop {
        if let Some(status) = tests.try_wait()? {
            return Ok(status.success());
        }
        if Instant::now() > deadline {
            eprintln!("e2e ios: {} overran its deadline; stopped", unit[0]);
            stop(&mut tests);
            return Ok(false);
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// The `.app` dx assembled under `dir`; its name follows the product name.
fn find_app(dir: &Path) -> Result<PathBuf> {
    let entries = std::fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "app") {
            return Ok(path);
        }
        // One level deeper, should dx nest it.
        if path.is_dir()
            && let Ok(inner) = find_app(&path)
        {
            return Ok(inner);
        }
    }
    bail!("no .app under {}", dir.display())
}

/// From `simctl list -j devices available`: the UDID, name and runtime of the device named
/// `wanted`, else of the first iPhone on the newest iOS runtime.
fn pick_device(json: &str, wanted: Option<&str>) -> Result<(String, String, String)> {
    let listed: serde_json::Value = serde_json::from_str(json).context("parse simctl's list")?;
    let runtimes = listed["devices"]
        .as_object()
        .context("simctl listed no devices")?;
    // `com.apple.CoreSimulator.SimRuntime.iOS-26-2`: the version is the tail.
    let version = |runtime: &str| -> Option<Vec<u32>> {
        let tail = runtime.rsplit_once(".iOS-")?.1;
        tail.split('-').map(|part| part.parse().ok()).collect()
    };
    let mut ios: Vec<(&String, Vec<u32>)> = runtimes
        .keys()
        .filter_map(|runtime| Some((runtime, version(runtime)?)))
        .collect();
    ios.sort_by(|a, b| b.1.cmp(&a.1));
    for (runtime, _) in ios {
        let devices = runtimes[runtime.as_str()].as_array().into_iter().flatten();
        for device in devices {
            let name = device["name"].as_str().unwrap_or_default();
            let fits = match wanted {
                Some(wanted) => name == wanted,
                None => name.starts_with("iPhone"),
            };
            if fits && let Some(udid) = device["udid"].as_str() {
                let short = runtime.rsplit('.').next().unwrap_or(runtime);
                return Ok((udid.into(), name.into(), short.into()));
            }
        }
    }
    bail!("no available iOS simulator matches {wanted:?}; `xcrun simctl list devices` lists them")
}

/// `xcrun simctl` with `args`; its stdout, or an error with its stderr.
fn simctl(args: &[&str]) -> Result<String> {
    let output = Command::new("xcrun")
        .arg("simctl")
        .args(args)
        .output()
        .context("run xcrun simctl")?;
    if !output.status.success() {
        bail!(
            "simctl {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = r#"{"devices": {
        "com.apple.CoreSimulator.SimRuntime.iOS-18-6": [
            {"name": "iPhone 16", "udid": "OLD-16", "isAvailable": true}
        ],
        "com.apple.CoreSimulator.SimRuntime.iOS-26-2": [
            {"name": "iPad Air 11-inch (M3)", "udid": "NEW-IPAD", "isAvailable": true},
            {"name": "iPhone 17 Pro", "udid": "NEW-17", "isAvailable": true}
        ],
        "com.apple.CoreSimulator.SimRuntime.watchOS-26-0": [
            {"name": "Apple Watch Series 11", "udid": "WATCH", "isAvailable": true}
        ]
    }}"#;

    #[test]
    fn the_newest_runtimes_first_iphone_is_picked() {
        let (udid, name, runtime) = pick_device(LIST, None).unwrap();
        assert_eq!((udid.as_str(), name.as_str()), ("NEW-17", "iPhone 17 Pro"));
        assert_eq!(runtime, "iOS-26-2");
    }

    #[test]
    fn a_named_device_wins_on_any_runtime() {
        let (udid, ..) = pick_device(LIST, Some("iPhone 16")).unwrap();
        assert_eq!(udid, "OLD-16");
        assert!(pick_device(LIST, Some("iPhone 3G")).is_err());
    }
}
