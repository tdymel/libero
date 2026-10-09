//! The iOS simulator's WKWebView (2784) has no DevTools socket the harness reaches: an [`Ios`]
//! launches the fixture app with `xcrun simctl launch` once per unit, reads the page through
//! the app's `E2E_BRIDGE` as the desktop driver does, and drives it with HID touches and keys
//! from AXe (idb with `E2E_IOS_INPUT=idb`). Run by `cargo run -p e2e -- ios`, on macOS only.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;

use crate::clock::{app_clock, app_count};
use crate::driver::{Driver, Platform, Rect};
use crate::passes::keyboard::Key;

/// The booted simulator; set by the runner.
pub const UDID_ENV: &str = "E2E_IOS_UDID";
/// The installed fixture app's bundle id; set by the runner.
pub const BUNDLE_ENV: &str = "E2E_IOS_BUNDLE";
/// `axe` (the default) or `idb`: the tool that injects touches and keys.
pub const INPUT_ENV: &str = "E2E_IOS_INPUT";

/// A cold app start in a simulator on a 3-core runner: a guess until the first CI run.
const LAUNCH: Duration = Duration::from_secs(90);

/// HID usages (keyboard page 0x07) of the left modifiers.
const SHIFT: u8 = 225;
const CTRL: u8 = 224;
const ALT: u8 = 226;

/// The app the last scenario finished cleanly in; the next scenario of its unit reuses it.
static IDLE: Mutex<Option<Ios>> = Mutex::new(None);

/// The touch and key injector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Injector {
    Axe,
    Idb,
}

/// One fixture app in the simulator; dropped, it is terminated, so a failed scenario's app is
/// never reused.
pub struct Ios {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    udid: String,
    bundle: String,
    input: Injector,
    /// The viewport's origin in screen points: the status bar may sit above the WebView.
    origin: (f64, f64),
    /// Screen points per CSS px: 1 under a `width=device-width` viewport.
    scale: f64,
    /// The test module path minus the scenario: one app per unit.
    unit: String,
}

impl Ios {
    /// `route` in the app of `module`'s unit, launched if the last scenario ran in another
    /// unit or failed; `window.__route` remounts the fixture.
    pub fn open(module: &str, route: &str) -> Result<Self> {
        let unit = module.rsplit_once("::").map_or(module, |(unit, _)| unit);
        let idle = IDLE.lock().unwrap_or_else(PoisonError::into_inner).take();
        let mut ios = match idle {
            Some(mut ios) if ios.unit == unit => {
                ios.run("__e2eErrors = []")?;
                ios
            }
            stale => {
                drop(stale);
                Self::launch(unit)?
            }
        };
        ios.run("document.activeElement?.blur(); scrollTo(0, 0)")?;
        let generation: u64 = ios.json(&format!("window.__route({route:?})"))?;
        ios.wait_for(&format!("[data-fixture-generation=\"{generation}\"]"))?;
        Ok(ios)
    }

    fn launch(unit: &str) -> Result<Self> {
        let unset = |name: &str| {
            format!("{name} is unset: run `cargo run -p e2e -- ios`, which boots the simulator")
        };
        let udid = std::env::var(UDID_ENV).with_context(|| unset(UDID_ENV))?;
        let bundle = std::env::var(BUNDLE_ENV).with_context(|| unset(BUNDLE_ENV))?;
        let input = match std::env::var(INPUT_ENV).as_deref() {
            Err(_) | Ok("axe") => Injector::Axe,
            Ok("idb") => Injector::Idb,
            Ok(other) => bail!("{INPUT_ENV}={other:?}: expected axe or idb"),
        };
        // The simulator shares the Mac's loopback, so the app dials this listener.
        let listener = TcpListener::bind("127.0.0.1:0").context("bind the bridge")?;
        listener.set_nonblocking(true)?;
        let artifacts = std::env::var_os("E2E_ARTIFACTS")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let log = artifacts.join(format!("ios-app-{}.log", unit.replace("::", "-")));
        // `SIMCTL_CHILD_*` reaches the app's environment without the prefix.
        simctl(
            &[
                "launch",
                "--terminate-running-process",
                &format!("--stdout={}", log.display()),
                &format!("--stderr={}", log.display()),
                &udid,
                &bundle,
            ],
            &[(
                "SIMCTL_CHILD_E2E_BRIDGE",
                &listener.local_addr()?.to_string(),
            )],
        )?;

        // The bridge connects from `Shell`'s first effect: it is the ready signal.
        let started = Instant::now();
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.into()),
            }
            if started.elapsed() > LAUNCH {
                let _ = simctl(&["terminate", &udid, &bundle], &[]);
                bail!(
                    "the fixture app's bridge did not connect within {LAUNCH:?}, see {}",
                    log.display()
                );
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(20)))?;
        let mut ios = Self {
            reader: BufReader::new(stream.try_clone()?),
            writer: stream,
            udid,
            bundle,
            input,
            origin: (0.0, 0.0),
            scale: 1.0,
            unit: unit.to_string(),
        };
        ios.wait_for("[data-fixture-ready]")?;
        // `Shell` installs the route hook in an effect, after the marker mounts.
        ios.wait_until("typeof window.__route === 'function'")?;
        ios.calibrate()?;
        Ok(ios)
    }

    /// The console stayed clean since the page loaded or the unit's last scenario; keeps the
    /// app for the unit's next scenario.
    pub fn finish(mut self, what: &str) -> Result<()> {
        let errors: Vec<String> = self.json("__e2eErrors")?;
        if !errors.is_empty() {
            bail!("{what}: console errors in the iOS WebView: {errors:?}");
        }
        *IDLE.lock().unwrap_or_else(PoisonError::into_inner) = Some(self);
        Ok(())
    }

    /// Runs a JS body (`return` for a value) in the page; its JSON answer.
    pub fn eval(&mut self, body: &str) -> Result<serde_json::Value> {
        let body = format!("try {{ {body} }} catch (e) {{ return {{ __e2eError: String(e) }}; }}");
        writeln!(self.writer, "{}", serde_json::to_string(&body)?)?;
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .context("read the bridge's answer")?;
        let mut answer: serde_json::Value =
            serde_json::from_str(&line).context("the bridge closed")?;
        if let Some(error) = answer.get("err") {
            let running: String = body.chars().skip(6).take(120).collect();
            bail!("iOS eval failed: {error}, running {running}");
        }
        let value = answer["ok"].take();
        if let Some(error) = value.get("__e2eError") {
            bail!("iOS eval threw: {error}");
        }
        Ok(value)
    }

    fn run(&mut self, body: &str) -> Result<()> {
        self.eval(&format!("{body}; return null;")).map(drop)
    }

    /// A JS expression's value; `undefined` reads as `null`.
    fn json<T: DeserializeOwned>(&mut self, expression: &str) -> Result<T> {
        let text = self.eval(&format!("return JSON.stringify({expression}) ?? 'null';"))?;
        let text = text.as_str().context("JSON.stringify gave no string")?;
        Ok(serde_json::from_str(text)?)
    }

    fn wait_for(&mut self, selector: &str) -> Result<()> {
        self.wait_until(&format!("{} !== null", element(selector)))
    }

    fn wait_until(&mut self, expression: &str) -> Result<()> {
        let started = Instant::now();
        while !self.json::<bool>(expression)? {
            if started.elapsed() > LAUNCH {
                bail!("{expression} did not hold within {LAUNCH:?}");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }

    /// Where viewport 0,0 sits on the screen: a tap at the screen's centre on the bare index
    /// page, read back as a `touchstart`'s `clientX/Y`. Printed, for the first CI runs.
    fn calibrate(&mut self) -> Result<()> {
        self.run(
            "window.__e2eTouch = null; addEventListener('touchstart', e => { \
             __e2eTouch = [e.touches[0].clientX, e.touches[0].clientY]; }, { capture: true, passive: true })",
        )?;
        let (screen_w, screen_h, inner_w): (f64, f64, f64) =
            self.json("[screen.width, screen.height, innerWidth]")?;
        self.scale = screen_w / inner_w;
        let at = (screen_w / 2.0, screen_h / 2.0);
        self.tap_screen(at)?;
        let started = Instant::now();
        let seen = loop {
            if let Some((x, y)) = self.json::<Option<(f64, f64)>>("__e2eTouch")? {
                break (x, y);
            }
            if started.elapsed() > Duration::from_secs(10) {
                bail!(
                    "the WebView saw no touch from {:?} at {at:?} pt",
                    self.input
                );
            }
            std::thread::sleep(Duration::from_millis(25));
        };
        self.origin = (at.0 - seen.0 * self.scale, at.1 - seen.1 * self.scale);
        eprintln!(
            "e2e ios: screen {screen_w}x{screen_h} pt, innerWidth {inner_w}, viewport origin {:?} pt, via {:?}",
            self.origin, self.input
        );
        Ok(())
    }

    /// A CSS-px viewport point in screen points, as the injector takes it.
    fn screen(&self, x: f64, y: f64) -> (f64, f64) {
        (
            x * self.scale + self.origin.0,
            y * self.scale + self.origin.1,
        )
    }

    /// The first match's centre, scrolled into view first: the phone is narrower than the
    /// desktop viewport the fixtures are written for.
    fn centre(&mut self, selector: &str) -> Result<(f64, f64)> {
        // A popover mounts hidden at 0,0 a round trip before it is placed (1002).
        let hidden = format!(
            "(e => !!e && getComputedStyle(e).visibility === 'hidden')({})",
            element(selector)
        );
        for _ in 0..40 {
            if !self.json::<bool>(&hidden)? {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        self.run(&format!(
            "{}.scrollIntoView({{ block: 'center', inline: 'center' }})",
            element(selector)
        ))?;
        // Momentum from the last swipe keeps the page moving under a tap, as on Android (1104).
        let read = format!(
            "(r => [r.x + r.width / 2, r.y + r.height / 2])({}.getBoundingClientRect())",
            element(selector)
        );
        let mut at: (f64, f64) = self.json(&read)?;
        for _ in 0..40 {
            std::thread::sleep(Duration::from_millis(50));
            let next: (f64, f64) = self.json(&read)?;
            let settled = (next.0 - at.0).abs() < 0.5 && (next.1 - at.1).abs() < 0.5;
            at = next;
            if settled {
                break;
            }
        }
        Ok(at)
    }

    fn tap_screen(&self, (x, y): (f64, f64)) -> Result<()> {
        let [x, y] = [point(x), point(y)];
        match self.input {
            Injector::Axe => self.inject(&["tap", "-x", &x, "-y", &y]),
            Injector::Idb => self.inject(&["ui", "tap", &x, &y]),
        }
    }

    fn tap(&self, x: f64, y: f64) -> Result<()> {
        self.tap_screen(self.screen(x, y))
    }

    /// A touch from viewport point `(x, y)` by `(dx, dy)` over `secs`, clamped to the viewport:
    /// a touch lifted off the WebView would leave it a stuck touch.
    fn swipe(&mut self, (x, y): (f64, f64), dx: f64, dy: f64, secs: f64) -> Result<()> {
        let (vw, vh): (f64, f64) = self.json("[innerWidth, innerHeight]")?;
        let clamp = |x: f64, y: f64| self.screen(x.clamp(0.0, vw - 1.0), y.clamp(0.0, vh - 1.0));
        let (from, to) = (clamp(x, y), clamp(x + dx, y + dy));
        let [x1, y1, x2, y2] = [point(from.0), point(from.1), point(to.0), point(to.1)];
        let secs = format!("{secs:.2}");
        match self.input {
            Injector::Axe => self.inject(&[
                "swipe",
                "--start-x",
                &x1,
                "--start-y",
                &y1,
                "--end-x",
                &x2,
                "--end-y",
                &y2,
                "--duration",
                &secs,
            ]),
            Injector::Idb => self.inject(&["ui", "swipe", &x1, &y1, &x2, &y2, "--duration", &secs]),
        }
    }

    /// A touch held at the first match's centre for `secs`; with `None`, only pressed or
    /// only lifted (`down`).
    fn touch(&mut self, selector: &str, hold: Option<f64>, down: bool) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        let (x, y) = self.screen(x, y);
        let [x, y] = [point(x), point(y)];
        match (self.input, hold) {
            (Injector::Axe, Some(secs)) => self.inject(&[
                "touch",
                "-x",
                &x,
                "-y",
                &y,
                "--down",
                "--up",
                "--delay",
                &format!("{secs:.2}"),
            ]),
            (Injector::Axe, None) => self.inject(&[
                "touch",
                "-x",
                &x,
                "-y",
                &y,
                if down { "--down" } else { "--up" },
            ]),
            (Injector::Idb, Some(secs)) => {
                self.inject(&["ui", "tap", &x, &y, "--duration", &format!("{secs:.2}")])
            }
            (Injector::Idb, None) => bail!("Ios: idb has no held touch"),
        }
    }

    fn key(&self, usage: u8) -> Result<()> {
        let usage = usage.to_string();
        match self.input {
            Injector::Axe => self.inject(&["key", &usage]),
            Injector::Idb => self.inject(&["ui", "key", &usage]),
        }
    }

    fn chord(&self, modifier: u8, key: Key) -> Result<()> {
        let (usage, _) = hid_usage(key)?;
        match self.input {
            Injector::Axe => self.inject(&[
                "key-combo",
                "--modifiers",
                &modifier.to_string(),
                "--key",
                &usage.to_string(),
            ]),
            Injector::Idb => bail!("Ios: idb holds no modifier"),
        }
    }

    /// `text` in one injector call, as a HID key sequence.
    fn type_all(&self, text: &str) -> Result<()> {
        match self.input {
            Injector::Axe => self.inject(&["type", text]),
            Injector::Idb => self.inject(&["ui", "text", text]),
        }
    }

    /// The injector with `args` against this simulator.
    fn inject(&self, args: &[&str]) -> Result<()> {
        let tool = match self.input {
            Injector::Axe => "axe",
            Injector::Idb => "idb",
        };
        let out = Command::new(tool)
            .args(args)
            .args(["--udid", &self.udid])
            .output()
            .with_context(|| format!("run {tool}"))?;
        if !out.status.success() {
            bail!(
                "{tool} {args:?} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(())
    }
}

impl Drop for Ios {
    fn drop(&mut self) {
        let _ = simctl(&["terminate", &self.udid, &self.bundle], &[]);
    }
}

fn element(selector: &str) -> String {
    format!("document.querySelector({selector:?})")
}

/// A screen point as the injectors take it: whole points.
fn point(at: f64) -> String {
    (at.round() as i64).to_string()
}

/// `xcrun simctl` with `args` and `env`; its stdout.
fn simctl(args: &[&str], env: &[(&str, &str)]) -> Result<String> {
    let out = Command::new("xcrun")
        .arg("simctl")
        .args(args)
        .envs(env.iter().copied())
        .output()
        .context("run xcrun simctl")?;
    if !out.status.success() {
        bail!(
            "simctl {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The HID usage (keyboard page 0x07) of a DOM key's `code`, and whether its `key` needs Shift.
fn hid_usage(key: Key) -> Result<(u8, bool)> {
    let code = key.code;
    let usage = if let Some(letter) = code.strip_prefix("Key").filter(|l| l.len() == 1) {
        4 + (letter.as_bytes()[0] - b'A')
    } else if let Some(digit) = code.strip_prefix("Digit").filter(|d| d.len() == 1) {
        match digit.as_bytes()[0] {
            b'0' => 39,
            d => 30 + (d - b'1'),
        }
    } else if let Some(n) = code.strip_prefix('F').and_then(|n| n.parse::<u8>().ok()) {
        57 + n
    } else {
        match code {
            "Enter" => 40,
            "Escape" => 41,
            "Backspace" => 42,
            "Tab" => 43,
            "Space" => 44,
            "Minus" => 45,
            "Equal" => 46,
            "BracketLeft" => 47,
            "BracketRight" => 48,
            "Backslash" => 49,
            "Semicolon" => 51,
            "Quote" => 52,
            "Backquote" => 53,
            "Comma" => 54,
            "Period" => 55,
            "Slash" => 56,
            "Home" => 74,
            "PageUp" => 75,
            "Delete" => 76,
            "End" => 77,
            "PageDown" => 78,
            "ArrowRight" => 79,
            "ArrowLeft" => 80,
            "ArrowDown" => 81,
            "ArrowUp" => 82,
            other => bail!("Ios: no HID usage for the key code {other:?}"),
        }
    };
    let shifted = key.key.len() == 1 && key.key.chars().all(|c| c.is_ascii_uppercase());
    Ok((usage, shifted))
}

impl Driver for Ios {
    fn platform(&self) -> Platform {
        Platform::Ios
    }

    async fn click(&mut self, selector: &str) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.tap(x, y)
    }

    async fn hover(&mut self, selector: &str) -> Result<()> {
        let _ = selector;
        bail!("Ios: no hover on a touch screen")
    }

    /// Two HID taps; whether WebKit reads them as a `dblclick` is unverified.
    async fn double_click(&mut self, selector: &str) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.tap(x, y)?;
        self.tap(x, y)
    }

    async fn click_at(&mut self, x: f64, y: f64) -> Result<()> {
        self.tap(x, y)
    }

    async fn viewport(&mut self) -> Result<(f64, f64)> {
        self.json("[innerWidth, innerHeight]")
    }

    async fn press(&mut self, key: Key) -> Result<()> {
        match hid_usage(key)? {
            (usage, false) => self.key(usage),
            (_, true) => self.chord(SHIFT, key),
        }
    }

    async fn press_shift(&mut self, key: Key) -> Result<()> {
        self.chord(SHIFT, key)
    }

    async fn press_ctrl(&mut self, key: Key) -> Result<()> {
        self.chord(CTRL, key)
    }

    async fn press_alt(&mut self, key: Key) -> Result<()> {
        self.chord(ALT, key)
    }

    /// One injector call per character, at about a typist's pace: one call for the lot
    /// outran PinField's move to the next cell on Android.
    async fn type_text(&mut self, text: &str) -> Result<()> {
        for ch in text.chars() {
            self.type_all(&ch.to_string())?;
        }
        Ok(())
    }

    /// The whole text in one call, as fast as the injector types; `ms` is not kept.
    async fn type_burst(&mut self, text: &str, ms: u64) -> Result<()> {
        let _ = ms;
        self.type_all(text)
    }

    async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()> {
        let from = self.centre(selector)?;
        self.swipe(from, dx, dy, 0.5)
    }

    async fn swipe_from(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.swipe((x, y), dx, dy, 0.5)
    }

    async fn drag_at(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.swipe((x, y), dx, dy, 0.5)
    }

    /// A slow swipe through the viewport's middle, so the page comes to rest.
    async fn scroll_by(&mut self, dy: f64) -> Result<()> {
        let (vw, vh) = self.viewport().await?;
        self.swipe((vw / 2.0, vh / 2.0 + dy / 2.0), 0.0, -dy, 0.5)
    }

    async fn long_press(&mut self, selector: &str, ms: u64) -> Result<()> {
        self.touch(selector, Some(ms as f64 / 1000.0), true)
    }

    async fn touch_down(&mut self, selector: &str) -> Result<()> {
        self.touch(selector, None, true)
    }

    async fn touch_up(&mut self, selector: &str) -> Result<()> {
        self.touch(selector, None, false)
    }

    async fn evaluate(&mut self, expression: &str) -> Result<serde_json::Value> {
        self.json(&format!("await ({expression})"))
    }

    /// libero's thread timers, held in the app (2144).
    async fn hold_timers(&mut self, delays: &[u32]) -> Result<bool> {
        self.evaluate(&app_clock("hold", delays)).await?;
        Ok(true)
    }

    async fn armed(&mut self, ms: u32) -> Result<usize> {
        app_count(self.evaluate(&app_clock("armed", &[ms])).await?)
    }

    async fn fire_timers(&mut self, ms: u32) -> Result<usize> {
        app_count(self.evaluate(&app_clock("fire", &[ms])).await?)
    }

    async fn focus(&mut self, selector: &str) -> Result<()> {
        self.run(&format!("{}.focus()", element(selector)))
    }

    async fn text(&mut self, selector: &str) -> Result<String> {
        self.json(&format!("{}.textContent", element(selector)))
    }

    async fn value(&mut self, selector: &str) -> Result<String> {
        self.json(&format!("{}.value", element(selector)))
    }

    async fn attr(&mut self, selector: &str, name: &str) -> Result<Option<String>> {
        self.json(&format!("{}.getAttribute({name:?})", element(selector)))
    }

    async fn exists(&mut self, selector: &str) -> Result<bool> {
        self.json(&format!("{} !== null", element(selector)))
    }

    async fn rect(&mut self, selector: &str) -> Result<Rect> {
        self.json(&format!("{}.getBoundingClientRect()", element(selector)))
    }

    async fn style(&mut self, selector: &str, property: &str) -> Result<String> {
        self.json(&format!(
            "getComputedStyle({}).getPropertyValue({property:?})",
            element(selector)
        ))
    }

    async fn is_focused(&mut self, selector: &str) -> Result<bool> {
        self.json(&format!(
            "document.activeElement?.matches({selector:?}) ?? false"
        ))
    }

    async fn focused_id(&mut self) -> Result<String> {
        self.json("document.activeElement?.id ?? ''")
    }

    async fn focus_owner(&mut self) -> Result<String> {
        self.json("document.activeElement?.outerHTML.slice(0, 200) ?? 'nothing'")
    }

    /// As the desktop's: a flag the frame sets, polled, then `settle`'s idle rounds.
    async fn frame(&mut self) -> Result<()> {
        self.run(
            "window.__e2eFrame = false; requestAnimationFrame(() => { window.__e2eFrame = true; })",
        )?;
        let started = Instant::now();
        while !self.json::<bool>("window.__e2eFrame")? {
            if started.elapsed() > self.budget() {
                bail!("the WebView drew no frame within {:?}", self.budget());
            }
            self.idle().await;
        }
        self.settle().await
    }

    async fn idle(&mut self) {
        std::thread::sleep(Duration::from_millis(25));
    }

    fn budget(&self) -> Duration {
        crate::wait::timeout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passes::keyboard::{ARROW_UP, ENTER, SPACE, TAB};

    fn key(key: &'static str, code: &'static str) -> Key {
        Key {
            key,
            code,
            vk: 0,
            text: None,
        }
    }

    #[test]
    fn hid_usages_follow_the_keyboard_page() {
        assert_eq!(hid_usage(key("a", "KeyA")).unwrap(), (4, false));
        assert_eq!(hid_usage(key("Z", "KeyZ")).unwrap(), (29, true));
        assert_eq!(hid_usage(key("1", "Digit1")).unwrap(), (30, false));
        assert_eq!(hid_usage(key("0", "Digit0")).unwrap(), (39, false));
        assert_eq!(hid_usage(key("F1", "F1")).unwrap(), (58, false));
        assert_eq!(hid_usage(key("F12", "F12")).unwrap(), (69, false));
        assert_eq!(hid_usage(ENTER).unwrap().0, 40);
        assert_eq!(hid_usage(TAB).unwrap().0, 43);
        assert_eq!(hid_usage(SPACE).unwrap().0, 44);
        assert_eq!(hid_usage(ARROW_UP).unwrap().0, 82);
        assert!(hid_usage(key("Unidentified", "Fn")).is_err());
    }
}
