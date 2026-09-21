//! `cargo run -p e2e -- android [filter]` (964): the scenarios' `android` arm
//! against the fixtures APK in an emulator, booted headless when no device is up.

use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use super::{Guard, dx, free_port, own_target_dir, stop, workspace_root};

const PACKAGE: &str = "dev.libero.fixtures";
const ACTIVITY: &str = "dev.libero.fixtures/dev.dioxus.main.MainActivity";

pub fn run(filters: Vec<String>) -> Result<()> {
    let root = workspace_root()?;
    let dx = dx(&root)?;
    let target_dir = own_target_dir()?;
    let artifacts = std::env::temp_dir()
        .join("e2e-artifacts")
        .join(format!("android-pid{}", std::process::id()));
    std::fs::create_dir_all(&artifacts).context("create the artifacts directory")?;
    eprintln!("e2e android: artifacts go to {}", artifacts.display());
    let sdk = std::env::var_os("ANDROID_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join("Android/Sdk")
        });

    let mut guard = Guard::spawn()?;
    let mut emulator = None;
    let serial = match attached_device()? {
        Some(serial) => serial,
        None => boot(&sdk, &artifacts, &mut guard, &mut emulator)?,
    };
    eprintln!("e2e android: device {serial}");

    let outcome = run_on(
        &serial,
        &sdk,
        &root,
        &dx,
        &target_dir,
        &artifacts,
        &filters,
        &mut guard,
        &mut emulator,
    );

    let _ = adb(&serial, &["forward", "--remove-all"]);
    let _ = adb(&serial, &["shell", "am", "force-stop", PACKAGE]);
    if let Some(mut emulator) = emulator {
        let _ = adb(&serial, &["emu", "kill"]);
        stop(&mut emulator);
        eprintln!("e2e android: emulator stopped");
    }
    guard.done();
    outcome
}

/// Boots the emulator headless and waits for it; its serial.
fn boot(
    sdk: &Path,
    artifacts: &Path,
    guard: &mut Guard,
    emulator: &mut Option<Child>,
) -> Result<String> {
    let child = boot_emulator(sdk, artifacts)?;
    guard.tell(&format!("group {}", child.id()));
    *emulator = Some(child);
    wait_for_boot()
}

#[allow(clippy::too_many_arguments)]
fn run_on(
    serial: &str,
    sdk: &Path,
    root: &Path,
    dx: &std::ffi::OsStr,
    target_dir: &Path,
    artifacts: &Path,
    filters: &[String],
    guard: &mut Guard,
    emulator: &mut Option<Child>,
) -> Result<()> {
    let arch = match adb(serial, &["shell", "getprop", "ro.product.cpu.abi"])?.trim() {
        "x86_64" => "x86_64",
        "arm64-v8a" => "aarch64",
        other => bail!("no Rust target for the device's ABI {other:?}"),
    };
    eprintln!("e2e android: building the fixtures APK ({arch}), log in dx.log");
    let ndk = std::fs::read_dir(sdk.join("ndk"))
        .context("no NDK under the SDK")?
        .flatten()
        .map(|entry| entry.path())
        .max()
        .context("no NDK under the SDK")?;
    let log = std::fs::File::create(artifacts.join("dx.log"))?;
    let status = Command::new(dx)
        .current_dir(root.join("e2e/fixtures"))
        .args([
            "build",
            "--platform",
            "android",
            "--features",
            "mobile",
            "--target",
        ])
        .arg(format!("{arch}-linux-android"))
        .env("CARGO_TARGET_DIR", target_dir)
        .env("ANDROID_HOME", sdk)
        .env("ANDROID_SDK_ROOT", sdk)
        .env("ANDROID_NDK_HOME", &ndk)
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
    let apk = target_dir
        .join("dx/e2e-fixtures/debug/android/app/app/build/outputs/apk/debug/app-debug.apk");
    adb(serial, &["install", "-r", &apk.to_string_lossy()])?;

    // libtest ORs its filters, so the `android` tests matching any of them are
    // named one by one, `--exact`. An `android: skip(...)` is left out.
    let skipped = list(root, target_dir, filters, true)?;
    let names: Vec<String> = list(root, target_dir, filters, false)?
        .into_iter()
        .filter(|name| !skipped.contains(name))
        .collect();
    if names.is_empty() && skipped.is_empty() {
        bail!("no android scenario matches {filters:?}");
    }
    eprintln!(
        "e2e android: running {} scenario(s), {} skipped",
        names.len(),
        skipped.len()
    );

    // One app per unit, and a unit whose emulator died is run again on a fresh
    // one: its qemu dies of SIGSEGV every 30-40 scenarios (964).
    let mut red = Vec::new();
    for unit in names.chunk_by(|a, b| a.split("::").next() == b.split("::").next()) {
        let name = unit[0].split("::").next().unwrap_or_default();
        let mut passed = run_unit(serial, root, target_dir, artifacts, unit, guard)?;
        if attached_device()?.is_none() {
            eprintln!("e2e android: the emulator died during {name}; rebooting, running it again");
            if let Some(mut dead) = emulator.take() {
                stop(&mut dead);
            }
            boot(sdk, artifacts, guard, emulator)?;
            passed = run_unit(serial, root, target_dir, artifacts, unit, guard)?;
        }
        if !passed {
            red.push(name);
        }
    }
    if !red.is_empty() {
        bail!("android scenarios failed in: {}", red.join(", "));
    }
    Ok(())
}

/// The `android` tests matching `filters`; with `ignored`, only the skipped ones.
fn list(root: &Path, target_dir: &Path, filters: &[String], ignored: bool) -> Result<Vec<String>> {
    let listed = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "test",
            "-q",
            "-p",
            "e2e",
            "--features",
            "android",
            "--test",
            "all",
            "--target-dir",
        ])
        .arg(target_dir)
        .args(["--", "--list"])
        .args(ignored.then_some("--ignored"))
        .stderr(Stdio::inherit())
        .output()
        .context("list the tests")?;
    Ok(String::from_utf8_lossy(&listed.stdout)
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .filter(|name| name.ends_with("::android"))
        .filter(|name| filters.is_empty() || filters.iter().any(|f| name.contains(f.as_str())))
        .map(str::to_string)
        .collect())
}

/// One unit's scenarios on a freshly launched app; whether they passed. A
/// dead emulator hangs CDP calls, hence the deadline.
fn run_unit(
    serial: &str,
    root: &Path,
    target_dir: &Path,
    artifacts: &Path,
    unit: &[String],
    guard: &mut Guard,
) -> Result<bool> {
    let port = launch(serial)?;
    let mut tests = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "test",
            "-q",
            "-p",
            "e2e",
            "--features",
            "android",
            "--test",
            "all",
            "--target-dir",
        ])
        .arg(target_dir)
        .args(["--", "--exact", "--test-threads=1", "--format", "pretty"])
        .args(unit)
        .env("E2E_ANDROID_CDP", format!("127.0.0.1:{port}"))
        .env("E2E_ANDROID_SERIAL", serial)
        .env("E2E_ARTIFACTS", artifacts)
        .process_group(0)
        .spawn()
        .context("run the tests")?;
    guard.tell(&format!("group {}", tests.id()));
    let deadline = Instant::now() + Duration::from_secs(60 + 30 * unit.len() as u64);
    let passed = loop {
        if let Some(status) = tests.try_wait()? {
            break status.success();
        }
        if Instant::now() > deadline || attached_device()?.is_none() {
            eprintln!(
                "e2e android: {} overran its deadline or lost the device; stopped",
                unit[0]
            );
            stop(&mut tests);
            break false;
        }
        std::thread::sleep(Duration::from_secs(1));
    };
    let _ = adb(serial, &["forward", "--remove", &format!("tcp:{port}")]);
    Ok(passed)
}

/// Restarts the fixtures app and forwards its DevTools socket to a free port.
fn launch(serial: &str) -> Result<u16> {
    // A fresh boot sits on the lock screen, which takes the taps and keys.
    adb(serial, &["shell", "input", "keyevent", "KEYCODE_WAKEUP"])?;
    adb(serial, &["shell", "wm", "dismiss-keyguard"])?;
    let mut attempts = 0;
    let pid = loop {
        attempts += 1;
        match start_app(serial) {
            Ok(pid) => break pid,
            Err(error) if attempts < 3 => eprintln!("e2e android: {error:#}; starting it again"),
            Err(error) => return Err(error),
        }
    };
    let port = free_port()?;
    adb(
        serial,
        &[
            "forward",
            &format!("tcp:{port}"),
            &format!("localabstract:webview_devtools_remote_{pid}"),
        ],
    )?;
    Ok(port)
}

/// Starts the app afresh; its pid once its WebView's DevTools socket is open.
fn start_app(serial: &str) -> Result<String> {
    // A pause after `force-stop`: an immediate start raced into tao's
    // `NoAvailableActivity` panic ([[codebase/platform/android-build]]).
    adb(serial, &["shell", "am", "force-stop", PACKAGE])?;
    std::thread::sleep(Duration::from_secs(1));
    adb(serial, &["shell", "am", "start", "-W", "-n", ACTIVITY])?;
    let pid = adb(serial, &["shell", "pidof", PACKAGE])?;
    let pid = pid
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    if pid.is_empty() {
        bail!("{PACKAGE} is not running after `am start`");
    }
    // The WebView opens it after `am start -W` returns.
    let socket = format!("@webview_devtools_remote_{pid}");
    let deadline = Instant::now() + Duration::from_secs(20);
    while !adb(serial, &["shell", "cat", "/proc/net/unix"])?.contains(&socket) {
        if Instant::now() > deadline {
            bail!("{PACKAGE} opened no {socket} within 20 s");
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    Ok(pid)
}

/// The first device `adb devices` lists as ready.
fn attached_device() -> Result<Option<String>> {
    let output = Command::new("adb")
        .arg("devices")
        .output()
        .context("run adb")?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .skip(1)
        .find_map(|line| line.strip_suffix("\tdevice"))
        .map(str::to_string))
}

/// Headless, and never on the desktop's display.
fn boot_emulator(sdk: &Path, artifacts: &Path) -> Result<Child> {
    let avd = std::env::var("E2E_ANDROID_AVD").unwrap_or_else(|_| "libero-docs".into());
    eprintln!("e2e android: booting the {avd} emulator headless");
    let log = std::fs::File::create(artifacts.join("emulator.log"))?;
    Command::new(sdk.join("emulator/emulator"))
        .args(["-avd", &avd, "-no-window", "-gpu", "swiftshader_indirect"])
        .args(["-no-audio", "-no-snapshot-save", "-no-boot-anim"])
        .env_remove("DISPLAY")
        .env("ANDROID_HOME", sdk)
        .env("ANDROID_SDK_ROOT", sdk)
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .process_group(0)
        .spawn()
        .context("start the emulator")
}

fn wait_for_boot() -> Result<String> {
    let deadline = Instant::now() + Duration::from_secs(300);
    while Instant::now() < deadline {
        if let Some(serial) = attached_device()?
            && adb(&serial, &["shell", "getprop", "sys.boot_completed"])
                .is_ok_and(|booted| booted.trim() == "1")
        {
            return Ok(serial);
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    bail!("the emulator did not boot within 5 minutes")
}

/// `adb -s <serial> <args>`, its stdout, or an error with its stderr.
fn adb(serial: &str, args: &[&str]) -> Result<String> {
    let output = Command::new("adb")
        .args(["-s", serial])
        .args(args)
        .output()
        .context("run adb")?;
    if !output.status.success() {
        bail!(
            "adb {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
