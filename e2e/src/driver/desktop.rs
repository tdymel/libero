//! The desktop WebView (wry/WebKitGTK, 1126) has no DevTools socket: a
//! [`Desktop`] launches the fixture app once per unit, reads the page through the
//! app's `E2E_BRIDGE` (an `eval` loop over loopback TCP) and drives it with real
//! X input from `xdotool`. Run by `cargo run -p e2e -- desktop`, under Xvfb.

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;

use crate::driver::{Driver, Platform, Rect};
use crate::passes::keyboard::Key;

/// The fixture binary, built with `--features desktop`; set by the runner.
pub const APP_ENV: &str = "E2E_DESKTOP_APP";

/// A cold WebKitGTK start under Xvfb took 5.5 s on a seat.
const LAUNCH: Duration = Duration::from_secs(60);

/// Records console errors from the moment the bridge answers.
const CONSOLE_HOOK: &str = "window.__e2eErrors = [];
    const error = console.error.bind(console);
    console.error = (...args) => { __e2eErrors.push(args.map(String).join(' ')); error(...args); };
    addEventListener('error', e => __e2eErrors.push(String(e.message)));
    addEventListener('unhandledrejection', e => __e2eErrors.push(String(e.reason)));";

/// The app the last scenario finished cleanly in; the next scenario of its unit reuses it.
static IDLE: Mutex<Option<Desktop>> = Mutex::new(None);

/// One fixture app in its own window; dropped, it is killed, so a failed
/// scenario's app is never reused.
pub struct Desktop {
    app: Child,
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    window: String,
    /// The viewport's origin in window px: the menu bar sits above the WebView.
    origin: (f64, f64),
    scale: f64,
    /// The test module path minus the scenario: one app per unit (~5 s a launch).
    unit: String,
}

impl Desktop {
    /// `route` in the app of `module`'s unit, launched if the last scenario
    /// ran in another unit or failed; `window.__route` remounts the fixture.
    pub fn open(module: &str, route: &str) -> Result<Self> {
        let unit = module.rsplit_once("::").map_or(module, |(unit, _)| unit);
        let idle = IDLE.lock().unwrap_or_else(PoisonError::into_inner).take();
        let mut desktop = match idle {
            Some(desktop) if desktop.unit == unit => {
                // Past GTK's double-click time: the last scenario's clicks would count
                // towards the first one here (a splitter drag read as a double-click).
                std::thread::sleep(Duration::from_millis(500));
                desktop
            }
            stale => {
                drop(stale);
                Self::launch(unit)?
            }
        };
        // Pointer back to the calibration corner: no hover left from the last scenario.
        desktop.run("document.activeElement?.blur(); scrollTo(0, 0); __e2eErrors = []")?;
        desktop.park_pointer()?;
        let generation: u64 = desktop.json(&format!("window.__route({route:?})"))?;
        desktop.wait_for(&format!("[data-fixture-generation=\"{generation}\"]"))?;
        Ok(desktop)
    }

    fn launch(unit: &str) -> Result<Self> {
        let binary = std::env::var_os(APP_ENV).with_context(|| {
            format!("{APP_ENV} is unset: run `cargo run -p e2e -- desktop`, which starts Xvfb")
        })?;
        let listener = TcpListener::bind("127.0.0.1:0").context("bind the bridge")?;
        listener.set_nonblocking(true)?;
        let log = std::env::var_os("E2E_ARTIFACTS")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("desktop-app.log");
        let log = File::options().create(true).append(true).open(&log)?;
        let mut app = Command::new(binary)
            .env("E2E_BRIDGE", listener.local_addr()?.to_string())
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .spawn()
            .context("launch the fixture app")?;

        // The bridge connects from `Shell`'s first effect: it is the ready signal.
        let started = Instant::now();
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.into()),
            }
            if let Some(status) = app.try_wait()? {
                bail!("the fixture app exited ({status}) before its bridge connected");
            }
            if started.elapsed() > LAUNCH {
                let _ = app.kill();
                bail!("the fixture app's bridge did not connect within {LAUNCH:?}");
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(20)))?;
        let mut desktop = Self {
            app,
            reader: BufReader::new(stream.try_clone()?),
            writer: stream,
            window: String::new(),
            origin: (0.0, 0.0),
            scale: 1.0,
            unit: unit.to_string(),
        };
        desktop.wait_for("[data-fixture-ready]")?;
        // `Shell` installs the route hook in an effect, after the marker mounts.
        desktop.wait_until("typeof window.__route === 'function'")?;
        desktop.run(CONSOLE_HOOK)?;
        desktop.window = desktop.find_window()?;
        desktop.scale = desktop.json("devicePixelRatio")?;
        desktop.calibrate()?;
        xdotool(&["windowfocus", "--sync", &desktop.window])?;
        Ok(desktop)
    }

    /// The console stayed clean since [`Desktop::open`]; keeps the app for the
    /// unit's next scenario.
    pub fn finish(mut self, what: &str) -> Result<()> {
        let errors: Vec<String> = self.json("__e2eErrors")?;
        if !errors.is_empty() {
            bail!("{what}: console errors in the desktop WebView: {errors:?}");
        }
        *IDLE.lock().unwrap_or_else(PoisonError::into_inner) = Some(self);
        Ok(())
    }

    /// Runs a JS body (`return` for a value) in the page; its JSON answer.
    pub fn eval(&mut self, body: &str) -> Result<serde_json::Value> {
        // The bridge's own error names no cause: catch and carry the message.
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
            bail!("desktop eval failed: {error}, running {running}");
        }
        let value = answer["ok"].take();
        if let Some(error) = value.get("__e2eError") {
            bail!("desktop eval threw: {error}");
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

    fn find_window(&self) -> Result<String> {
        let pid = self.app.id().to_string();
        let started = Instant::now();
        loop {
            let found = Command::new("xdotool")
                .args(["search", "--onlyvisible", "--pid", &pid])
                .output()
                .context("run xdotool")?;
            if let Some(window) = String::from_utf8_lossy(&found.stdout).lines().next() {
                return Ok(window.to_string());
            }
            if started.elapsed() > LAUNCH {
                bail!("no visible window for the fixture app (pid {pid})");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Where viewport 0,0 sits in the window: a pointer move at a known window
    /// point, read back as `clientX/Y`. Bottom-left, away from the fixture.
    fn calibrate(&mut self) -> Result<()> {
        self.run("window.__e2eMove = null; addEventListener('mousemove', e => { __e2eMove = [e.clientX, e.clientY]; })")?;
        let at = self.park_pointer()?;
        let started = Instant::now();
        let seen = loop {
            if let Some((x, y)) = self.json::<Option<(f64, f64)>>("__e2eMove")? {
                break (x, y);
            }
            if started.elapsed() > Duration::from_secs(5) {
                bail!("the WebView saw no pointer move");
            }
            std::thread::sleep(Duration::from_millis(25));
        };
        self.origin = (at.0 - seen.0 * self.scale, at.1 - seen.1 * self.scale);
        Ok(())
    }

    /// Moves the pointer to the window's bottom-left corner; that point in window px.
    fn park_pointer(&self) -> Result<(f64, f64)> {
        let geometry = xdotool(&["getwindowgeometry", "--shell", &self.window])?;
        let height: f64 = geometry
            .lines()
            .find_map(|line| line.strip_prefix("HEIGHT="))
            .context("xdotool gave no window height")?
            .parse()?;
        let at = (2.0, height - 2.0);
        xdotool(&[
            "mousemove",
            "--window",
            &self.window,
            &at.0.to_string(),
            &at.1.to_string(),
        ])?;
        Ok(at)
    }

    /// A CSS-px viewport point in window px, as `xdotool --window` takes it.
    fn window_point(&self, x: f64, y: f64) -> [String; 2] {
        [
            (x * self.scale + self.origin.0).round().to_string(),
            (y * self.scale + self.origin.1).round().to_string(),
        ]
    }

    fn centre(&mut self, selector: &str) -> Result<(f64, f64)> {
        self.run(&format!(
            "{}.scrollIntoView({{ block: 'nearest', inline: 'nearest' }})",
            element(selector)
        ))?;
        let rect: Rect = self.json(&format!("{}.getBoundingClientRect()", element(selector)))?;
        Ok((rect.x + rect.width / 2.0, rect.y + rect.height / 2.0))
    }

    fn pointer(&self, x: f64, y: f64, then: &[&str]) -> Result<()> {
        let [x, y] = self.window_point(x, y);
        let mut args = vec![
            "mousemove",
            "--window",
            &self.window,
            x.as_str(),
            y.as_str(),
        ];
        args.extend_from_slice(then);
        xdotool(&args).map(drop)
    }

    fn key(&self, chord: &str) -> Result<()> {
        xdotool(&["key", "--window", &self.window, chord]).map(drop)
    }
}

impl Drop for Desktop {
    fn drop(&mut self) {
        let _ = self.app.kill();
        let _ = self.app.wait();
    }
}

fn element(selector: &str) -> String {
    format!("document.querySelector({selector:?})")
}

/// `xdotool` with `args`; its stdout.
fn xdotool(args: &[&str]) -> Result<String> {
    let out = Command::new("xdotool")
        .args(args)
        .output()
        .context("run xdotool")?;
    if !out.status.success() {
        bail!(
            "xdotool {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The X keysym for a DOM `key`.
fn keysym(key: &str) -> &str {
    match key {
        " " => "space",
        "Enter" => "Return",
        "Backspace" => "BackSpace",
        "PageUp" => "Prior",
        "PageDown" => "Next",
        "ArrowLeft" => "Left",
        "ArrowRight" => "Right",
        "ArrowUp" => "Up",
        "ArrowDown" => "Down",
        other => other,
    }
}

impl Driver for Desktop {
    fn platform(&self) -> Platform {
        Platform::Desktop
    }

    async fn click(&mut self, selector: &str) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.pointer(x, y, &["click", "1"])
    }

    async fn hover(&mut self, selector: &str) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.pointer(x, y, &[])
    }

    async fn double_click(&mut self, selector: &str) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.pointer(x, y, &["click", "--repeat", "2", "--delay", "80", "1"])
    }

    async fn click_at(&mut self, x: f64, y: f64) -> Result<()> {
        self.pointer(x, y, &["click", "1"])
    }

    async fn viewport(&mut self) -> Result<(f64, f64)> {
        self.json("[innerWidth, innerHeight]")
    }

    async fn press(&mut self, key: Key) -> Result<()> {
        self.key(keysym(key.key))
    }

    async fn press_shift(&mut self, key: Key) -> Result<()> {
        self.key(&format!("shift+{}", keysym(key.key)))
    }

    async fn press_ctrl(&mut self, key: Key) -> Result<()> {
        self.key(&format!("ctrl+{}", keysym(key.key)))
    }

    async fn type_text(&mut self, text: &str) -> Result<()> {
        self.type_burst(text, 30).await
    }

    /// Real X key events at `ms` a key, through WebKitGTK's own input path.
    async fn type_burst(&mut self, text: &str, ms: u64) -> Result<()> {
        xdotool(&[
            "type",
            "--window",
            &self.window,
            "--delay",
            &ms.to_string(),
            text,
        ])
        .map(drop)
    }

    async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.pointer(x, y, &["mousedown", "1"])?;
        for step in 1..=8 {
            let t = step as f64 / 8.0;
            self.pointer(x + dx * t, y + dy * t, &[])?;
            std::thread::sleep(Duration::from_millis(16));
        }
        xdotool(&["mouseup", "1"]).map(drop)
    }

    async fn scroll_by(&mut self, dy: f64) -> Result<()> {
        self.run(&format!("scrollBy({{ top: {dy}, behavior: 'instant' }})"))
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

    async fn idle(&mut self) {
        std::thread::sleep(Duration::from_millis(25));
    }

    fn budget(&self) -> Duration {
        crate::wait::timeout()
    }
}
