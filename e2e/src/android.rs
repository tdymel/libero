//! The Android WebView the `android` scenario arm drives (964): the runner's
//! forwarded DevTools socket, one shared page, input through `adb shell input`.

use std::io::{BufRead, BufReader, Read, Write};
use std::sync::OnceLock;

use anyhow::{Context, Result, bail};
use chromiumoxide::handler::HandlerConfig;
use chromiumoxide::{Browser, Page};
use futures::StreamExt;
use tokio::runtime::Runtime;

use crate::passes::console::Recorder;

/// The forwarded DevTools address, `127.0.0.1:<port>`. Set by the runner.
pub const CDP_ENV: &str = "E2E_ANDROID_CDP";
/// The `adb -s` serial of the device the runner set up.
pub const SERIAL_ENV: &str = "E2E_ANDROID_SERIAL";

pub(crate) struct Harness {
    pub runtime: Runtime,
    pub page: Page,
    pub console: Recorder,
    /// Device px per CSS px, and the WebView's top-left on the screen.
    pub scale: f64,
    pub origin: (f64, f64),
    _browser: Browser,
}

static HARNESS: OnceLock<Harness> = OnceLock::new();

pub(crate) fn harness() -> &'static Harness {
    HARNESS.get_or_init(|| {
        // Retried: a just-launched app lists no page yet, or swaps its first
        // target for the app's ("Session with given id not found").
        let mut attempt = 0;
        loop {
            match connect() {
                Ok(harness) => return harness,
                Err(_) if attempt < 20 => {
                    attempt += 1;
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
                Err(error) => panic!("connect to the Android WebView: {error:#}"),
            }
        }
    })
}

/// Runs an async body on the Android harness's runtime.
pub fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    harness().runtime.block_on(fut)
}

fn connect() -> Result<Harness> {
    let address = std::env::var(CDP_ENV).with_context(|| {
        format!("{CDP_ENV} is unset: run `cargo run -p e2e -- android`, which sets up the device")
    })?;
    let origin = webview_origin(&address)?;
    let runtime = Runtime::new().context("tokio runtime")?;
    let (browser, page, console, scale) = runtime.block_on(async {
        // No viewport: an override would lay the WebView out at another size
        // than the screen it is tapped on.
        let config = HandlerConfig {
            viewport: None,
            ignore_invalid_messages: true,
            ..HandlerConfig::default()
        };
        // The websocket straight away: the DevTools server never closes a
        // `/json/version` connection, and the WebView refuses one with an `Origin`.
        let (mut browser, mut handler) =
            Browser::connect_with_config(format!("ws://{address}/devtools/browser"), config)
                .await
                .context("connect over the forwarded DevTools socket")?;
        tokio::spawn(async move { while handler.next().await.is_some() {} });
        browser.fetch_targets().await?;
        crate::wait::until("the WebView's page", || async {
            Ok(!browser.pages().await?.is_empty())
        })
        .await?;
        let page = browser.pages().await?.remove(0);
        crate::wait::for_js_true(
            &page,
            "typeof window.__route === 'function'",
            "the fixture app's route hook",
        )
        .await?;
        let console = Recorder::attach(&page).await?;
        let scale: f64 = page.evaluate("devicePixelRatio").await?.into_value()?;
        anyhow::Ok((browser, page, console, scale))
    })?;
    Ok(Harness {
        runtime,
        page,
        console,
        scale,
        origin,
        _browser: browser,
    })
}

/// The WebView's `screenX`/`screenY` in device px, which only `/json` reports.
fn webview_origin(address: &str) -> Result<(f64, f64)> {
    let mut stream = std::net::TcpStream::connect(address).context("connect to /json")?;
    write!(stream, "GET /json HTTP/1.1\r\nHost: {address}\r\n\r\n")?;
    // The DevTools server keeps the connection open: read `Content-Length` bytes.
    let mut reader = BufReader::new(stream);
    let mut length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse()?;
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    let targets: Vec<serde_json::Value> = serde_json::from_slice(&body)?;
    let description = targets
        .iter()
        .find(|t| t["type"] == "page")
        .and_then(|t| t["description"].as_str())
        .context("no page target on /json")?;
    let geometry: serde_json::Value = serde_json::from_str(description)?;
    Ok((
        geometry["screenX"].as_f64().unwrap_or(0.0),
        geometry["screenY"].as_f64().unwrap_or(0.0),
    ))
}

/// `adb shell input <args>` on the runner's device.
pub(crate) async fn input(args: &[String]) -> Result<()> {
    let mut command = vec!["input".to_string()];
    command.extend_from_slice(args);
    shell(&command).await.map(drop)
}

/// Whether the soft keyboard is up: it takes an Escape's `keydown` to hide
/// itself, and the page sees only the `keyup` (994).
pub(crate) async fn soft_keyboard_shown() -> Result<bool> {
    let state = shell(&["dumpsys".into(), "input_method".into()]).await?;
    Ok(state.contains("mInputShown=true"))
}

/// `adb shell <args>` on the runner's device; its stdout.
async fn shell(args: &[String]) -> Result<String> {
    let mut command = tokio::process::Command::new("adb");
    if let Ok(serial) = std::env::var(SERIAL_ENV) {
        command.args(["-s", &serial]);
    }
    command.arg("shell").args(args).kill_on_drop(true);
    // Bounded: the emulator has exited mid-run a few times (964), and a
    // hung `adb` would stall the whole serial run.
    let output = tokio::time::timeout(std::time::Duration::from_secs(20), command.output())
        .await
        .with_context(|| format!("adb shell {} hung for 20 s", args.join(" ")))?
        .context("run adb")?;
    if !output.status.success() {
        bail!(
            "adb shell {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The Android keycode for a DOM `key`, for `input keyevent`.
pub(crate) fn keycode(key: &str) -> Result<u32> {
    Ok(match key {
        "Tab" => 61,
        " " => 62,
        "Enter" => 66,
        "Escape" => 111,
        "Home" => 122,
        "End" => 123,
        "PageUp" => 92,
        "PageDown" => 93,
        "ArrowLeft" => 21,
        "ArrowUp" => 19,
        "ArrowRight" => 22,
        "ArrowDown" => 20,
        "Backspace" => 67,
        "Delete" => 112,
        f if f.len() > 1
            && f.starts_with('F')
            && f[1..].parse::<u32>().is_ok_and(|n| (1..=12).contains(&n)) =>
        {
            130 + f[1..].parse::<u32>()?
        }
        c if c.len() == 1 && c.as_bytes()[0].is_ascii_lowercase() => {
            29 + u32::from(c.as_bytes()[0] - b'a')
        }
        c if c.len() == 1 && c.as_bytes()[0].is_ascii_digit() => {
            7 + u32::from(c.as_bytes()[0] - b'0')
        }
        other => bail!("no Android keycode for {other:?}"),
    })
}

/// `KEYCODE_SHIFT_LEFT`, `KEYCODE_CTRL_LEFT`, for `input keycombination`.
pub(crate) const SHIFT: u32 = 59;
pub(crate) const CTRL: u32 = 113;

/// `input text` takes `%s` for a space and a shell-quoted rest.
pub(crate) fn input_text(text: &str) -> String {
    let escaped = text
        .replace('%', "\\%")
        .replace(' ', "%s")
        .replace('\'', "'\\''");
    format!("'{escaped}'")
}
