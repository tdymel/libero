//! The desktop WebView (wry/WebKitGTK, 1126) has no DevTools socket: a
//! [`Desktop`] launches the fixture app once per unit, reads the page through the
//! app's `E2E_BRIDGE` (an `eval` loop over loopback TCP) and drives it with real
//! X input from `xdotool`. Run by `cargo run -p e2e -- desktop`, under Xvfb.
//! On macOS (2782) the app builds NSEvents for its own WKWebView from ops on the bridge;
//! on Windows (2783) input goes over WebView2's DevTools port ([`webview2`]).

use std::fs::File;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use crate::clock::{app_clock, app_count};
#[cfg(target_os = "macos")]
use crate::driver::bridge::note;
use crate::driver::bridge::{Bridge, element};
use crate::driver::{Driver, Platform, Rect};
use crate::passes::keyboard::Key;

#[cfg(windows)]
mod webview2;

/// The fixture binary, built with `--features desktop`; set by the runner.
pub const APP_ENV: &str = "E2E_DESKTOP_APP";

/// A cold WebKitGTK start under Xvfb took 5.5 s on a seat.
const LAUNCH: Duration = Duration::from_secs(60);

/// What one WebKitGTK wheel notch scrolls, in CSS px: measured 68 per notch in a 120 px tall
/// scroller of the harness's 573 px tall window (todo 2197).
#[cfg(not(any(target_os = "macos", windows)))]
const WHEEL_NOTCH: f64 = 68.0;

/// The app the last scenario finished cleanly in; the next scenario of its unit reuses it.
static IDLE: Mutex<Option<Desktop>> = Mutex::new(None);

/// One fixture app in its own window; dropped, it is killed, so a failed
/// scenario's app is never reused.
pub struct Desktop {
    app: Child,
    bridge: Bridge,
    #[cfg_attr(any(target_os = "macos", windows), allow(dead_code))]
    window: String,
    /// The viewport's origin in window px: the menu bar sits above the WebView.
    #[cfg_attr(any(target_os = "macos", windows), allow(dead_code))]
    origin: (f64, f64),
    scale: f64,
    #[cfg(target_os = "macos")]
    routes: MacRoutes,
    #[cfg(windows)]
    cdp: Option<webview2::Cdp>,
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
            Some(mut desktop) if desktop.unit == unit => {
                // Past GTK's double-click time: the last scenario's clicks would count
                // towards the first one here (a splitter drag read as a double-click).
                // CDP sends each press's click count, so WebView2 needs no wait.
                #[cfg(not(windows))]
                std::thread::sleep(Duration::from_millis(500));
                desktop.bridge.run("__e2eErrors = []")?;
                desktop
            }
            stale => {
                drop(stale);
                Self::launch(unit)?
            }
        };
        // Pointer back to the calibration corner: no hover left from the last scenario.
        desktop
            .bridge
            .run("document.activeElement?.blur(); scrollTo(0, 0)")?;
        desktop.park_pointer()?;
        let generation: u64 = desktop.bridge.json(&format!("window.__route({route:?})"))?;
        desktop
            .bridge
            .wait_for(&format!("[data-fixture-generation=\"{generation}\"]"))?;
        Ok(desktop)
    }

    fn launch(unit: &str) -> Result<Self> {
        let binary = std::env::var_os(APP_ENV).with_context(|| {
            format!("{APP_ENV} is unset: run `cargo run -p e2e -- desktop`, which starts Xvfb")
        })?;
        let listener = TcpListener::bind("127.0.0.1:0").context("bind the bridge")?;
        let artifacts = std::env::var_os("E2E_ARTIFACTS")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let log = artifacts.join("desktop-app.log");
        let log = File::options().create(true).append(true).open(&log)?;
        // A store per launch: a kept scheme or direction never reaches the next unit.
        static LAUNCHES: AtomicU64 = AtomicU64::new(0);
        let launch = format!(
            "{}-{}",
            std::process::id(),
            LAUNCHES.fetch_add(1, Ordering::Relaxed)
        );
        let mut command = Command::new(binary);
        command
            .env("E2E_BRIDGE", listener.local_addr()?.to_string())
            .env(
                "E2E_STORAGE_DIR",
                artifacts.join(format!("desktop-storage-{launch}")),
            )
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log));
        #[cfg(windows)]
        let devtools = webview2::prepare(&mut command, &launch)?;
        let mut app = command.spawn().context("launch the fixture app")?;

        let bridge = Bridge::accept(&listener, "desktop", LAUNCH, || match app.try_wait()? {
            Some(status) => bail!("the fixture app exited ({status}) before its bridge connected"),
            None => Ok(()),
        })
        .inspect_err(|_| {
            let _ = app.kill();
        })?;
        let mut desktop = Self {
            app,
            bridge,
            window: String::new(),
            origin: (0.0, 0.0),
            scale: 1.0,
            #[cfg(target_os = "macos")]
            routes: MacRoutes::default(),
            #[cfg(windows)]
            cdp: None,
            unit: unit.to_string(),
        };
        desktop.bridge.wait_for("[data-fixture-ready]")?;
        // `Shell` installs the route hook in an effect, after the marker mounts.
        desktop
            .bridge
            .wait_until("typeof window.__route === 'function'")?;
        #[cfg(windows)]
        {
            desktop.cdp = Some(webview2::Cdp::connect(devtools)?);
        }
        desktop.window = desktop.find_window()?;
        desktop.scale = desktop.bridge.json("devicePixelRatio")?;
        desktop.calibrate()?;
        desktop.focus_window()?;
        Ok(desktop)
    }

    /// The console stayed clean since the page loaded or the unit's last scenario
    /// (the fixture's head hook records it); keeps the app for the
    /// unit's next scenario.
    pub fn finish(mut self, what: &str) -> Result<()> {
        let errors: Vec<String> = self.bridge.json("__e2eErrors")?;
        if !errors.is_empty() {
            bail!("{what}: console errors in the desktop WebView: {errors:?}");
        }
        *IDLE.lock().unwrap_or_else(PoisonError::into_inner) = Some(self);
        Ok(())
    }

    /// Where viewport 0,0 sits in the window: a pointer move at a known window
    /// point, read back as `clientX/Y`. Bottom-left, away from the fixture.
    fn calibrate(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        self.find_routes()?;
        self.bridge.run("window.__e2eMove = null; addEventListener('mousemove', e => { __e2eMove = [e.clientX, e.clientY]; })")?;
        let at = self.park_pointer()?;
        let started = Instant::now();
        let seen = loop {
            if let Some((x, y)) = self.bridge.json::<Option<(f64, f64)>>("__e2eMove")? {
                break (x, y);
            }
            if started.elapsed() > Duration::from_secs(5) {
                bail!("the WebView saw no pointer move");
            }
            std::thread::sleep(Duration::from_millis(25));
        };
        // macOS ops and CDP are in viewport px already: the move must land where it was sent.
        #[cfg(any(target_os = "macos", windows))]
        if (seen.0 - at.0).abs() > 1.0 || (seen.1 - at.1).abs() > 1.0 {
            bail!("an NSEvent move to {at:?} reached the page at {seen:?}");
        }
        #[cfg(not(any(target_os = "macos", windows)))]
        {
            self.origin = (at.0 - seen.0 * self.scale, at.1 - seen.1 * self.scale);
        }
        Ok(())
    }

    fn centre(&mut self, selector: &str) -> Result<(f64, f64)> {
        self.bridge.run(&format!(
            "{}.scrollIntoView({{ block: 'nearest', inline: 'nearest' }})",
            element(selector)
        ))?;
        // A popup is hidden until placed, a frame or more after it mounts: aim where it rests.
        let rested = format!(
            "(async () => {{ const e = {}; const a = e.getBoundingClientRect(); \
             await new Promise((r) => {{ requestAnimationFrame(() => r()); setTimeout(r, 100); }}); \
             const b = e.getBoundingClientRect(); \
             return {{ settled: getComputedStyle(e).visibility !== 'hidden' && a.x === b.x && a.y === b.y \
             && a.width === b.width && a.height === b.height, \
             rect: {{ x: b.x, y: b.y, width: b.width, height: b.height }} }}; }})()",
            element(selector)
        );
        let started = Instant::now();
        let rect = loop {
            let Rested { settled, rect } = self.bridge.json(&format!("await {rested}"))?;
            if settled || started.elapsed() > Duration::from_secs(2) {
                break rect;
            }
        };
        Ok((rect.x + rect.width / 2.0, rect.y + rect.height / 2.0))
    }
}

/// An element's rect and whether it is shown and held still across a frame.
#[derive(serde::Deserialize)]
struct Rested {
    settled: bool,
    rect: Rect,
}

#[cfg(not(any(target_os = "macos", windows)))]
impl Desktop {
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

    fn pointer(&self, x: f64, y: f64, then: &[&str]) -> Result<()> {
        let [x, y] = self.window_point(x, y);
        let mut args = vec![
            "mousemove",
            "--window",
            &self.window,
            // A drag past the window's top or left edge: `-102` would read as an option.
            "--",
            x.as_str(),
            y.as_str(),
        ];
        args.extend_from_slice(then);
        xdotool(&args).map(drop)
    }

    fn key(&self, chord: &str) -> Result<()> {
        xdotool(&["key", "--window", &self.window, chord]).map(drop)
    }

    fn focus_window(&self) -> Result<()> {
        xdotool(&["windowfocus", "--sync", &self.window]).map(drop)
    }

    fn move_to(&mut self, x: f64, y: f64) -> Result<()> {
        self.pointer(x, y, &[])
    }

    fn click_point(&mut self, x: f64, y: f64, count: u32) -> Result<()> {
        if count == 2 {
            return self.pointer(x, y, &["click", "--repeat", "2", "--delay", "80", "1"]);
        }
        self.pointer(x, y, &["click", "1"])
    }

    fn drag_by(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.pointer(x, y, &["mousedown", "1"])?;
        for step in 1..=8 {
            let t = step as f64 / 8.0;
            self.pointer(x + dx * t, y + dy * t, &[])?;
            std::thread::sleep(Duration::from_millis(16));
        }
        xdotool(&["mouseup", "1"]).map(drop)
    }

    /// X buttons 4 and 5, one notch per [`WHEEL_NOTCH`] px of `dy`, at least one.
    fn wheel_at(&mut self, x: f64, y: f64, dy: f64) -> Result<()> {
        let notches = (dy.abs() / WHEEL_NOTCH).round().max(1.0).to_string();
        let button = if dy < 0.0 { "4" } else { "5" };
        self.pointer(
            x,
            y,
            &["click", "--repeat", &notches, "--delay", "16", button],
        )
    }

    /// `key` with `mods` among `shift`, `ctrl` and `alt`, as one X chord.
    fn chord(&mut self, mods: &[&str], key: Key) -> Result<()> {
        let mut chord: Vec<&str> = mods.to_vec();
        chord.push(keysym(key.key));
        self.key(&chord.join("+"))
    }

    /// Real X key events at `ms` a key, through WebKitGTK's own input path.
    fn type_keys(&mut self, text: &str, ms: u64) -> Result<()> {
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
}

/// Counts what input the page saw, so an op's effects are in before the next eval (2782).
#[cfg(target_os = "macos")]
const SEEN: &str = "window.__e2eSeen = { up: 0, key: 0, wheel: 0, move: null };
    addEventListener('mouseup', () => { __e2eSeen.up += 1; }, true);
    addEventListener('keyup', () => { __e2eSeen.key += 1; }, true);
    addEventListener('wheel', () => { __e2eSeen.wheel += 1; }, true);
    addEventListener('mousemove', (e) => { __e2eSeen.move = [e.clientX, e.clientY]; }, true)";

/// Each pointer route moves to its own x, so a late event never counts for the next route.
#[cfg(target_os = "macos")]
fn pointer_index(route: &str) -> f64 {
    ["target", "view", "window", "post", "cg"]
        .iter()
        .position(|name| *name == route)
        .unwrap_or_default() as f64
}

/// How the app hands each kind of NSEvent to its WebView (`mac_input.rs`), found per launch.
#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct MacRoutes {
    pointer: &'static str,
    button: &'static str,
    key: &'static str,
}

#[cfg(target_os = "macos")]
impl Default for MacRoutes {
    fn default() -> Self {
        Self {
            pointer: "target",
            button: "target",
            key: "window",
        }
    }
}

/// macOS input (2782): ops on the bridge that the app turns into NSEvents for its WKWebView
/// (`e2e/fixtures/src/mac_input.rs`). No TCC grant, no shared cursor, no window lookup.
#[cfg(target_os = "macos")]
impl Desktop {
    fn input(&mut self, op: serde_json::Value) -> Result<()> {
        let answer = self.bridge.request(&op.to_string())?;
        if let Some(error) = answer.get("err") {
            bail!("desktop input {op} failed: {error}");
        }
        Ok(())
    }

    /// The app makes its window key and the WebView first responder; logs what the session gave it.
    fn find_window(&mut self) -> Result<String> {
        let state = self
            .bridge
            .request(&serde_json::json!({ "op": "prepare" }).to_string())?;
        eprintln!("e2e desktop: macOS window {state}");
        self.bridge.run(SEEN)?;
        Ok(String::new())
    }

    fn focus_window(&self) -> Result<()> {
        Ok(())
    }

    /// WebKit queues input behind the bridge's evals: waits until the page saw it, at most a
    /// second, as a disabled control gets no mouse events at all.
    fn seen(&mut self, condition: &str) -> Result<()> {
        let started = Instant::now();
        while !self
            .bridge
            .json::<bool>(&format!("window.__e2eSeen === undefined || {condition}"))?
        {
            if started.elapsed() > Duration::from_secs(1) {
                eprintln!("e2e desktop: the page did not see `{condition}` within 1 s");
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    /// Which route each kind of event reaches the page by: the first the page sees, as no one
    /// knows what a hosted runner's WKWebView answers. A route that stops the app answering is
    /// skipped by the next launch; every step is noted in `calibration.log` as it starts.
    fn find_routes(&mut self) -> Result<()> {
        static STATE: Mutex<(Option<MacRoutes>, Vec<String>)> = Mutex::new((None, Vec::new()));
        let (found, hung) = STATE.lock().unwrap_or_else(PoisonError::into_inner).clone();
        if let Some(found) = found {
            self.routes = found;
            return Ok(());
        }
        let height: f64 = self.bridge.json("innerHeight")?;
        let at = (2.0, height - 2.0);
        // The window's own events reach the page only once the real cursor is over it.
        self.input(serde_json::json!({ "op": "enter" }))?;
        let mut pointer = None;
        let mut button = None;
        let mut key = None;
        for (kind, route) in [
            ("pointer", "target"),
            ("pointer", "view"),
            ("pointer", "window"),
            ("pointer", "post"),
            ("pointer", "cg"),
            ("button", "target"),
            ("button", "view"),
            ("button", "post"),
            ("button", "cg"),
            ("key", "window"),
            ("key", "responder"),
            ("key", "post"),
        ] {
            let name = format!("{kind} {route}");
            if hung.contains(&name) || kind == "pointer" && pointer.is_some() && route == "cg" {
                continue;
            }
            note(&format!("trying {name}"));
            let tried = match kind {
                "pointer" => self.try_pointer(route, at.0 + 8.0 * pointer_index(route), at.1),
                "button" => self.try_button(route, at),
                _ => self.try_key(route),
            };
            let seen = tried.inspect_err(|error| {
                note(&format!("{name}: the app stopped answering ({error:#})"));
                STATE
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .1
                    .push(name.clone());
            })?;
            note(&format!("{name}: {seen}"));
            let slot = match kind {
                "pointer" => &mut pointer,
                "button" => &mut button,
                _ => &mut key,
            };
            if seen && slot.is_none() {
                *slot = Some(route);
            }
        }
        let Some(pointer) = pointer else {
            bail!("the WebView saw no pointer move by any route; see calibration.log");
        };
        self.routes = MacRoutes {
            pointer,
            button: button.unwrap_or(self.routes.button),
            key: key.unwrap_or(self.routes.key),
        };
        note(&format!(
            "picked pointer {} button {} key {}",
            self.routes.pointer, self.routes.button, self.routes.key
        ));
        STATE.lock().unwrap_or_else(PoisonError::into_inner).0 = Some(self.routes);
        Ok(())
    }

    fn try_pointer(&mut self, route: &str, x: f64, y: f64) -> Result<bool> {
        self.probe_route(
            serde_json::json!({ "op": "move", "x": x, "y": y, "via": route }),
            &format!("__e2eSeen.move?.[0] === {x}"),
        )
    }

    fn try_button(&mut self, route: &str, at: (f64, f64)) -> Result<bool> {
        let up = self.count("up")?;
        let mut seen = true;
        for op in ["down", "up"] {
            let condition = if op == "up" {
                format!("__e2eSeen.up > {up}")
            } else {
                "true".to_string()
            };
            seen &= self.probe_route(
                serde_json::json!({ "op": op, "x": at.0, "y": at.1, "count": 1, "via": route }),
                &condition,
            )?;
        }
        Ok(seen)
    }

    fn try_key(&mut self, route: &str) -> Result<bool> {
        let before = self.count("key")?;
        self.probe_route(
            serde_json::json!({ "op": "key", "key": "Escape", "via": route }),
            &format!("__e2eSeen.key > {before}"),
        )
    }

    /// Sends `op`; whether the page's `condition` held within 1.5 s. A refused op is `false`,
    /// an app that stops answering an error.
    fn probe_route(&mut self, op: serde_json::Value, condition: &str) -> Result<bool> {
        let answer = self.bridge.request(&op.to_string())?;
        if answer.get("err").is_some() {
            return Ok(false);
        }
        let started = Instant::now();
        while started.elapsed() < Duration::from_millis(1500) {
            if self
                .bridge
                .json::<bool>(&format!("window.__e2eSeen !== undefined && ({condition})"))?
            {
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        Ok(false)
    }

    fn count(&mut self, what: &str) -> Result<u64> {
        self.bridge.json(&format!("window.__e2eSeen?.{what} ?? 0"))
    }

    /// Moves the pointer to the viewport's bottom-left corner; that point.
    fn park_pointer(&mut self) -> Result<(f64, f64)> {
        let height: f64 = self.bridge.json("innerHeight")?;
        let at = (2.0, height - 2.0);
        self.move_to(at.0, at.1)?;
        Ok(at)
    }

    fn move_to(&mut self, x: f64, y: f64) -> Result<()> {
        let via = self.routes.pointer;
        self.input(serde_json::json!({ "op": "move", "x": x, "y": y, "via": via }))?;
        self.seen(&format!(
            "Math.abs(__e2eSeen.move?.[0] - {x}) <= 1 && Math.abs(__e2eSeen.move?.[1] - {y}) <= 1"
        ))
    }

    fn click_point(&mut self, x: f64, y: f64, count: u32) -> Result<()> {
        self.move_to(x, y)?;
        let before = self.count("up")?;
        let via = self.routes.button;
        for n in 1..=count {
            for op in ["down", "up"] {
                self.input(
                    serde_json::json!({ "op": op, "x": x, "y": y, "count": n, "via": via }),
                )?;
            }
        }
        self.seen(&format!("__e2eSeen.up >= {}", before + u64::from(count)))
    }

    fn drag_by(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.move_to(x, y)?;
        let via = self.routes.button;
        self.input(serde_json::json!({ "op": "down", "x": x, "y": y, "count": 1, "via": via }))?;
        for step in 1..=8 {
            let t = step as f64 / 8.0;
            let (x, y) = (x + dx * t, y + dy * t);
            self.input(
                serde_json::json!({ "op": "move", "x": x, "y": y, "held": true, "via": via }),
            )?;
            std::thread::sleep(Duration::from_millis(16));
        }
        let before = self.count("up")?;
        let (x, y) = (x + dx, y + dy);
        self.input(serde_json::json!({ "op": "up", "x": x, "y": y, "count": 1, "via": via }))?;
        self.seen(&format!("__e2eSeen.up > {before}"))
    }

    /// One pixel-precise wheel event of `dy` CSS px.
    fn wheel_at(&mut self, x: f64, y: f64, dy: f64) -> Result<()> {
        self.move_to(x, y)?;
        let before = self.count("wheel")?;
        self.input(serde_json::json!({ "op": "wheel", "x": x, "y": y, "dy": dy }))?;
        self.seen(&format!("__e2eSeen.wheel > {before}"))
    }

    fn chord(&mut self, mods: &[&str], key: Key) -> Result<()> {
        let before = self.count("key")?;
        let via = self.routes.key;
        self.input(serde_json::json!({ "op": "key", "key": key.key, "mods": mods, "via": via }))?;
        self.seen(&format!("__e2eSeen.key > {before}"))
    }

    /// One key op per character at `ms` a key; waits for the last only, so a burst stays one.
    fn type_keys(&mut self, text: &str, ms: u64) -> Result<()> {
        let before = self.count("key")?;
        let mut typed = 0;
        for ch in text.chars() {
            let key = match ch {
                '\n' => "Enter".to_string(),
                '\t' => "Tab".to_string(),
                ch => ch.to_string(),
            };
            let via = self.routes.key;
            self.input(serde_json::json!({ "op": "key", "key": key, "via": via }))?;
            typed += 1;
            std::thread::sleep(Duration::from_millis(ms));
        }
        self.seen(&format!("__e2eSeen.key >= {}", before + typed))
    }
}

impl Drop for Desktop {
    fn drop(&mut self) {
        let _ = self.app.kill();
        let _ = self.app.wait();
    }
}

/// `xdotool` with `args`; its stdout.
#[cfg(not(any(target_os = "macos", windows)))]
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
#[cfg(not(any(target_os = "macos", windows)))]
fn keysym(key: &str) -> &str {
    match key {
        " " => "space",
        "+" => "plus",
        "-" => "minus",
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
        self.click_point(x, y, 1)
    }

    async fn hover(&mut self, selector: &str) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.move_to(x, y)
    }

    async fn double_click(&mut self, selector: &str) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.click_point(x, y, 2)
    }

    async fn click_at(&mut self, x: f64, y: f64) -> Result<()> {
        self.click_point(x, y, 1)
    }

    async fn viewport(&mut self) -> Result<(f64, f64)> {
        self.bridge.json("[innerWidth, innerHeight]")
    }

    async fn press(&mut self, key: Key) -> Result<()> {
        self.chord(&[], key)
    }

    async fn press_shift(&mut self, key: Key) -> Result<()> {
        self.chord(&["shift"], key)
    }

    async fn press_ctrl(&mut self, key: Key) -> Result<()> {
        self.chord(&["ctrl"], key)
    }

    /// Cmd on macOS, where libero's `mod` is the Apple key.
    async fn press_mod(&mut self, key: Key) -> Result<()> {
        self.chord(
            &[if cfg!(target_os = "macos") {
                "meta"
            } else {
                "ctrl"
            }],
            key,
        )
    }

    async fn press_alt(&mut self, key: Key) -> Result<()> {
        self.chord(&["alt"], key)
    }

    async fn type_text(&mut self, text: &str) -> Result<()> {
        self.type_burst(text, 30).await
    }

    async fn type_burst(&mut self, text: &str, ms: u64) -> Result<()> {
        self.type_keys(text, ms)
    }

    async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.drag_by(x, y, dx, dy)
    }

    async fn wheel(&mut self, selector: &str, dy: f64) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        self.wheel_at(x, y, dy)
    }

    async fn evaluate(&mut self, expression: &str) -> Result<serde_json::Value> {
        self.bridge.json(&format!("await ({expression})"))
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

    async fn scroll_by(&mut self, dy: f64) -> Result<()> {
        self.bridge
            .run(&format!("scrollBy({{ top: {dy}, behavior: 'instant' }})"))
    }

    async fn focus(&mut self, selector: &str) -> Result<()> {
        self.bridge.run(&format!("{}.focus()", element(selector)))
    }

    async fn text(&mut self, selector: &str) -> Result<String> {
        self.bridge
            .json(&format!("{}.textContent", element(selector)))
    }

    async fn value(&mut self, selector: &str) -> Result<String> {
        self.bridge.json(&format!("{}.value", element(selector)))
    }

    async fn attr(&mut self, selector: &str, name: &str) -> Result<Option<String>> {
        self.bridge
            .json(&format!("{}.getAttribute({name:?})", element(selector)))
    }

    async fn exists(&mut self, selector: &str) -> Result<bool> {
        self.bridge.json(&format!("{} !== null", element(selector)))
    }

    async fn rect(&mut self, selector: &str) -> Result<Rect> {
        self.bridge
            .json(&format!("{}.getBoundingClientRect()", element(selector)))
    }

    async fn style(&mut self, selector: &str, property: &str) -> Result<String> {
        self.bridge.json(&format!(
            "getComputedStyle({}).getPropertyValue({property:?})",
            element(selector)
        ))
    }

    async fn is_focused(&mut self, selector: &str) -> Result<bool> {
        self.bridge.json(&format!(
            "document.activeElement?.matches({selector:?}) ?? false"
        ))
    }

    async fn focused_id(&mut self) -> Result<String> {
        self.bridge.json("document.activeElement?.id ?? ''")
    }

    async fn focus_owner(&mut self) -> Result<String> {
        self.bridge
            .json("document.activeElement?.outerHTML.slice(0, 200) ?? 'nothing'")
    }

    /// The bridge answers synchronously: a flag the frame sets, polled; then `settle`'s idle
    /// rounds, as the app runs outside the WebView.
    async fn frame(&mut self) -> Result<()> {
        self.bridge.run(
            "window.__e2eFrame = false; requestAnimationFrame(() => { window.__e2eFrame = true; })",
        )?;
        let started = Instant::now();
        while !self.bridge.json::<bool>("window.__e2eFrame")? {
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
