//! `cargo run -p e2e -- android [filter]` (964): the scenarios' `android` arm
//! against the fixtures APK in an emulator of its own, booted headless.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use super::{
    dx::{dx, workspace_root},
    http::free_port,
    own_target_dir,
    process::{Guard, OwnGroup, stop},
};

const PACKAGE: &str = "dev.libero.fixtures";
const ACTIVITY: &str = "dev.libero.fixtures/dev.dioxus.main.MainActivity";
/// Host GLES renderer, `E2E_ANDROID_GPU` overrides it. `swiftshader_indirect`
/// (legacy SwiftShader 4.0) segfaulted qemu every ~10 min (995).
const GPU: &str = "swangle_indirect";

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
    // Each run boots its own emulator on a free console port, so seats run in
    // parallel (997); `E2E_ANDROID_SERIAL` picks a running device instead.
    let port = match std::env::var("E2E_ANDROID_SERIAL") {
        Ok(_) => None,
        Err(_) => Some(free_console_port()?),
    };
    let serial = match port {
        Some(port) => boot(&sdk, &artifacts, port, &mut guard, &mut emulator)?,
        None => std::env::var("E2E_ANDROID_SERIAL")?,
    };
    eprintln!("e2e android: device {serial}");

    let outcome = run_on(
        &serial,
        port,
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

/// Boots the emulator headless on `port` and waits for it; its serial.
fn boot(
    sdk: &Path,
    artifacts: &Path,
    port: u16,
    guard: &mut Guard,
    emulator: &mut Option<Child>,
) -> Result<String> {
    let child = boot_emulator(sdk, artifacts, port)?;
    guard.tell(&format!("group {}", child.id()));
    let serial = format!("emulator-{port}");
    wait_for_boot(&serial, emulator.insert(child))?;
    Ok(serial)
}

/// The first even console port from 5554 whose pair (console, adb) is free
/// and no device claims.
fn free_console_port() -> Result<u16> {
    let devices = Command::new("adb")
        .arg("devices")
        .output()
        .context("run adb")?;
    let devices = String::from_utf8_lossy(&devices.stdout).into_owned();
    let free = |port: u16| std::net::TcpListener::bind(("127.0.0.1", port)).is_ok();
    (5554..=5682)
        .step_by(2)
        .find(|&port| {
            !devices.contains(&format!("emulator-{port}")) && free(port) && free(port + 1)
        })
        .context("no free emulator console port in 5554..5682")
}

#[allow(clippy::too_many_arguments)]
fn run_on(
    serial: &str,
    port: Option<u16>,
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
    // `E2E_RELEASE=1`: optimised fixtures, for the frame-time report (1086).
    let release = std::env::var_os("E2E_RELEASE").is_some();
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
        .args(release.then_some("--release"))
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
    let profile = if release { "release" } else { "debug" };
    let apk = target_dir.join(format!(
        "dx/e2e-fixtures/{profile}/android/app/app/build/outputs/apk/debug/app-debug.apk"
    ));
    adb(serial, &["install", "-r", &apk.to_string_lossy()])?;
    // `use_geolocation`: no driver answers its dialog, so grant up front.
    for permission in ["ACCESS_FINE_LOCATION", "ACCESS_COARSE_LOCATION"] {
        let permission = format!("android.permission.{permission}");
        adb(serial, &["shell", "pm", "grant", PACKAGE, &permission])?;
    }
    // `use_local_storage` and `use_indexed_db`: `install -r` keeps the app's files, so a past run's values would show.
    // Best effort: the scenario also starts by removing its keys.
    let _ = adb(
        serial,
        &["shell", "run-as", PACKAGE, "rm", "-rf", "files/storage"],
    );
    // `use_user_media` answers wry's dialog itself (1346), so it must show even on a reused device.
    // `use_system_notification` too (1348).
    for permission in ["CAMERA", "RECORD_AUDIO", "POST_NOTIFICATIONS"] {
        let permission = format!("android.permission.{permission}");
        adb(serial, &["shell", "pm", "revoke", PACKAGE, &permission])?;
        adb(
            serial,
            &[
                "shell",
                "pm",
                "clear-permission-flags",
                PACKAGE,
                &permission,
                "user-set",
                "user-fixed",
            ],
        )?;
    }

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
    // one: its qemu died of SIGSEGV every 30-40 scenarios before swangle (995).
    let mut red = Vec::new();
    for unit in names.chunk_by(|a, b| a.split("::").next() == b.split("::").next()) {
        let name = unit[0].split("::").next().unwrap_or_default();
        let mut passed = run_unit(serial, root, target_dir, artifacts, unit, guard)?;
        if !is_up(serial)? {
            let Some(port) = port else {
                bail!("{serial} went away during {name}");
            };
            eprintln!("e2e android: the emulator died during {name}; rebooting, running it again");
            if let Some(mut dead) = emulator.take() {
                stop(&mut dead);
            }
            boot(sdk, artifacts, port, guard, emulator)?;
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
    list_arm("android", root, target_dir, filters, ignored)
}

/// The tests of the scenario arm `arm` (also its feature) matching `filters`.
pub(super) fn list_arm(
    arm: &str,
    root: &Path,
    target_dir: &Path,
    filters: &[String],
    ignored: bool,
) -> Result<Vec<String>> {
    let suffix = format!("::{arm}");
    let listed = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "test",
            "-q",
            "-p",
            "e2e",
            "--features",
            arm,
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
        .filter(|name| name.ends_with(&suffix))
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
        // `E2E_FRAMES`: the frame-time rows and the swipe's first row are printed (1153).
        .args(
            std::env::var_os("E2E_FRAMES")
                .is_some()
                .then_some("--nocapture"),
        )
        .args(unit)
        .env("E2E_ANDROID_CDP", format!("127.0.0.1:{port}"))
        .env("E2E_ANDROID_SERIAL", serial)
        .env("E2E_ARTIFACTS", artifacts)
        .own_group()
        .spawn()
        .context("run the tests")?;
    guard.tell(&format!("group {}", tests.id()));
    let deadline = Instant::now() + Duration::from_secs(60 + 30 * unit.len() as u64);
    let passed = loop {
        if let Some(status) = tests.try_wait()? {
            break status.success();
        }
        if Instant::now() > deadline || !is_up(serial)? {
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
    // Gesture navigation takes a swipe from a screen edge as Back and closes the app (992).
    const BUTTONS: &str = "com.android.internal.systemui.navbar.threebutton";
    const GESTURES: &str = "com.android.internal.systemui.navbar.gestural";
    // `E2E_ANDROID_NAV=gestural` keeps gestures, for the back zone's own scenarios (2133).
    let (nav, other) = match std::env::var("E2E_ANDROID_NAV").as_deref() {
        Ok("gestural") => (GESTURES, BUTTONS),
        _ => (BUTTONS, GESTURES),
    };
    let overlays = adb(serial, &["shell", "cmd", "overlay", "list"])?;
    // Exclusive: beside a still enabled `gestural` the window keeps its 24dp insets under
    // the 48dp bar, which takes taps on the WebView's bottom 24px (1149).
    let enabled = |name: &str| overlays.lines().any(|l| l.trim() == format!("[x] {name}"));
    if !enabled(nav) || enabled(other) {
        adb(
            serial,
            &[
                "shell",
                "cmd",
                "overlay",
                "enable-exclusive",
                "--category",
                nav,
            ],
        )?;
        // System UI swaps the bar asynchronously; a swipe before it lands still goes Back.
        std::thread::sleep(Duration::from_secs(3));
    }
    let mut attempts = 0;
    let pid = loop {
        attempts += 1;
        match start_app(serial) {
            Ok(pid) => break pid,
            Err(error) if attempts < 5 => eprintln!("e2e android: {error:#}; starting it again"),
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
    // Launched in touch mode (a fresh boot, an earlier tap), the WebView holds no
    // view focus and drops key presses; a key to the launcher leaves it (1006).
    adb(serial, &["shell", "input", "keyevent", "KEYCODE_DPAD_UP"])?;
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
    // The WebView opens it after `am start -W` returns. A start still races
    // into wry's or tao's no-activity panic about one launch in five, and the
    // process lives on without a WebView.
    let socket = format!("@webview_devtools_remote_{pid}");
    let deadline = Instant::now() + Duration::from_secs(20);
    while !adb(serial, &["shell", "cat", "/proc/net/unix"])?.contains(&socket) {
        let log = adb(
            serial,
            &["logcat", "-d", "--pid", &pid, "-s", "RustStdoutStderr"],
        )?;
        if let Some(panic) = log.lines().find(|line| line.contains("panicked at")) {
            bail!("{PACKAGE} panicked at start: {}", panic.trim());
        }
        if Instant::now() > deadline {
            bail!("{PACKAGE} opened no {socket} within 20 s");
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    Ok(pid)
}

/// Whether `adb devices` lists `serial` as ready.
fn is_up(serial: &str) -> Result<bool> {
    let output = Command::new("adb")
        .arg("devices")
        .output()
        .context("run adb")?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line == format!("{serial}\tdevice")))
}

/// Headless, and never on the desktop's display. `-read-only` lets several
/// runs boot the same AVD at once; installs vanish with the instance.
fn boot_emulator(sdk: &Path, artifacts: &Path, port: u16) -> Result<Child> {
    let avd = std::env::var("E2E_ANDROID_AVD").unwrap_or_else(|_| "libero-docs".into());
    let gpu = std::env::var("E2E_ANDROID_GPU").unwrap_or_else(|_| GPU.into());
    eprintln!("e2e android: booting the {avd} emulator headless on {port}, -gpu {gpu}");
    let log = std::fs::File::create(artifacts.join("emulator.log"))?;
    Command::new(sdk.join("emulator/emulator"))
        .args(["-avd", &avd, "-no-window", "-gpu", &gpu, "-read-only"])
        .args(["-port", &port.to_string()])
        .args(["-no-audio", "-no-snapshot-save", "-no-boot-anim"])
        // A second camera, as on a phone: `use_user_media` switches between them (1347).
        .args(["-camera-front", "emulated"])
        .env_remove("DISPLAY")
        .env("ANDROID_HOME", sdk)
        .env("ANDROID_SDK_ROOT", sdk)
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .own_group()
        .spawn()
        .context("start the emulator")
}

/// Until `serial` finished booting; fails early when the emulator exits (its
/// port taken by a run that raced this one).
fn wait_for_boot(serial: &str, emulator: &mut Child) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(300);
    while Instant::now() < deadline {
        if let Some(status) = emulator.try_wait()? {
            bail!("the emulator exited while booting ({status}), see emulator.log");
        }
        if is_up(serial)?
            && adb(serial, &["shell", "getprop", "sys.boot_completed"])
                .is_ok_and(|booted| booted.trim() == "1")
        {
            return Ok(());
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
