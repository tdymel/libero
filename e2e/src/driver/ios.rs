//! The iOS simulator's WKWebView (2784) has no DevTools socket the harness reaches: an [`Ios`]
//! launches the fixture app with `xcrun simctl launch` once per unit, reads the page through
//! the app's `E2E_BRIDGE` as the desktop driver does, and drives it with HID touches and keys
//! from AXe (idb with `E2E_IOS_INPUT=idb`). Run by `cargo run -p e2e -- ios`, on macOS only.

use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use crate::clock::{app_clock, app_count};
use crate::driver::bridge::{Bridge, element, note};
use crate::driver::{Driver, Platform, Rect};
use crate::passes::keyboard::Key;
use crate::wait::output_within;

/// The booted simulator; set by the runner.
pub const UDID_ENV: &str = "E2E_IOS_UDID";
/// The installed fixture app's bundle id; set by the runner.
pub const BUNDLE_ENV: &str = "E2E_IOS_BUNDLE";
/// `axe` (the default) or `idb`: the tool that injects touches and keys.
pub const INPUT_ENV: &str = "E2E_IOS_INPUT";
/// Written into `E2E_ARTIFACTS` when the app never dialled the bridge; holds the app log's path.
pub const NO_BRIDGE: &str = "ios-no-bridge";

/// Written into `E2E_ARTIFACTS`: every injector and simctl call, begun and ended; the runner
/// prints its tail when a unit overruns.
pub const TRACE: &str = "ios-trace.log";

/// A cold app start in a simulator on a 3-core runner: a guess until the first CI run.
const LAUNCH: Duration = Duration::from_secs(90);

/// An injector or simctl call that takes longer is wedged; the longest real one is a swipe.
const TOOL_LIMIT: Duration = Duration::from_secs(45);

/// The status bar's height in screen points on the iPhones CI picks; `env(safe-area-inset-top)`
/// wins when the page reports more.
const STATUS_BAR: f64 = 62.0;

/// HID usages (keyboard page 0x07) of the left modifiers.
const SHIFT: u8 = 225;
const CTRL: u8 = 224;
const ALT: u8 = 226;
const CMD: u8 = 227;

/// The page's last keys, touches, presses and focus moves, printed with a red scenario: what the
/// simulator's taps and Option+Tab reached (3455, 3456). The bubbling keydown runs after the app's;
/// a run of moves or scrolls is one line with its count (3603).
const EVENT_LOG: &str = "window.__e2eLog = []; \
    const name = e => e === document.body ? 'body' : e === document ? 'document' : !e?.tagName ? String(e) \
      : e.tagName.toLowerCase() + (e.id ? '#' + e.id : '') \
      + (e.dataset?.slot ? '[' + e.dataset.slot + ']' : ''); \
    const log = line => { const ms = Math.round(performance.now()); \
      const last = [__e2eLog.at(-1), __e2eLog.at(-2)].find((e, i) => e?.line === line \
        && (i === 0 || __e2eLog.at(-1).line.includes('move'))); \
      if (last) { last.n++; last.to = ms; return; } \
      __e2eLog.push({ line, n: 1, at: ms, to: ms }); if (__e2eLog.length > 120) __e2eLog.shift(); }; \
    window.__e2eMark = log; \
    addEventListener('keydown', e => log(`keydown ${e.key} alt=${+e.altKey} shift=${+e.shiftKey} on ${name(e.target)}`), true); \
    addEventListener('keydown', e => log(`  prevented=${+e.defaultPrevented}`)); \
    for (const type of ['pointerdown', 'pointerup', 'pointercancel', 'gotpointercapture', 'lostpointercapture']) { \
      addEventListener(type, e => log(`${type} ${e.pointerType} ${name(e.target)} at ${Math.round(e.clientX)},${Math.round(e.clientY)}` \
        + (type === 'pointerdown' ? ` touch-action ${getComputedStyle(e.target).touchAction}` : '')), true); } \
    addEventListener('pointermove', e => log(`pointermove ${e.pointerType} ${name(e.target)}`), { capture: true, passive: true }); \
    for (const type of ['touchstart', 'touchmove', 'touchend', 'touchcancel']) { \
      addEventListener(type, e => log(`${type} ${name(e.target)} touches=${e.touches.length}`), { capture: true, passive: true }); } \
    for (const type of ['click', 'dblclick', 'focusin']) { \
      addEventListener(type, e => log(`${type} ${name(e.target)}`), true); } \
    addEventListener('scroll', e => log(`scroll ${name(e.target)}`), { capture: true, passive: true }); \
    visualViewport?.addEventListener('resize', () => log(`zoom ${visualViewport.scale.toFixed(2)}`));";

/// The hardware keyboard capitalizes a field's first letter; WebKit reads the attribute when a
/// field takes focus, so every field gets one when it mounts (3630).
const NO_CAPITALS: &str = "const off = () => document.querySelectorAll( \
      'input:not([autocapitalize]), textarea:not([autocapitalize])') \
      .forEach(field => field.setAttribute('autocapitalize', 'off')); \
    new MutationObserver(off).observe(document, { childList: true, subtree: true }); \
    addEventListener('focusin', off, true); off();";

/// The [`EVENT_LOG`] lines, a repeated one with its count and time span in ms.
const EVENT_LINES: &str = "(window.__e2eLog ?? []).map(({ line, n, at, to }) => \
    `${at}${n > 1 ? '-' + to : ''} ${line}${n > 1 ? ' x' + n : ''}`)";

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
    bridge: Bridge,
    udid: String,
    bundle: String,
    input: Injector,
    /// The viewport's origin in screen points: the status bar may sit above the WebView.
    origin: (f64, f64),
    /// Screen points per CSS px: 1 under a `width=device-width` viewport.
    scale: f64,
    /// The test module path minus the scenario: one app per unit.
    unit: String,
    /// The scenario's route, which a relaunch opens again.
    route: String,
    /// A system sheet may still cover the app, so the next scenario launches a fresh one.
    overlaid: bool,
}

impl Ios {
    /// `route` in the app of `module`'s unit, launched if the last scenario ran in another
    /// unit or failed; `window.__route` remounts the fixture.
    pub fn open(module: &str, route: &str) -> Result<Self> {
        let unit = module.rsplit_once("::").map_or(module, |(unit, _)| unit);
        trace(&format!("= {module} {route}"));
        let idle = IDLE.lock().unwrap_or_else(PoisonError::into_inner).take();
        let mut ios = match idle {
            Some(mut ios) if ios.unit == unit => {
                ios.bridge.run("__e2eErrors = []")?;
                ios
            }
            stale => {
                drop(stale);
                Self::launch(unit)?
            }
        };
        ios.route = route.to_string();
        ios.mount()?;
        Ok(ios)
    }

    /// Remounts the fixture on `self.route`, focus and scroll reset.
    fn mount(&mut self) -> Result<()> {
        self.bridge
            .run("document.activeElement?.blur(); scrollTo(0, 0); window.__e2eLog = []")?;
        let generation: u64 = self
            .bridge
            .json(&format!("window.__route({:?})", self.route))?;
        self.bridge
            .wait_for(&format!("[data-fixture-generation=\"{generation}\"]"))
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

        let bridge = Bridge::accept(&listener, "iOS", LAUNCH, || Ok(())).map_err(|error| {
            let _ = simctl(&["terminate", &udid, &bundle], &[]);
            // The runner stops the run on this marker: the next unit's app would not dial either.
            let _ = std::fs::write(artifacts.join(NO_BRIDGE), log.display().to_string());
            error.context(format!("see {}", log.display()))
        })?;
        let mut ios = Self {
            bridge,
            udid,
            bundle,
            input,
            origin: (0.0, 0.0),
            scale: 1.0,
            unit: unit.to_string(),
            route: String::new(),
            overlaid: false,
        };
        ios.bridge.wait_for("[data-fixture-ready]")?;
        // `Shell` installs the route hook in an effect, after the marker mounts.
        ios.bridge
            .wait_until("typeof window.__route === 'function'")?;
        ios.bridge.run(EVENT_LOG)?;
        ios.bridge.run(NO_CAPITALS)?;
        if let Err(error) = ios.calibrate() {
            ios.keep_failure();
            return Err(error);
        }
        Ok(ios)
    }

    /// The console stayed clean since the page loaded or the unit's last scenario; keeps the
    /// app for the unit's next scenario.
    pub fn finish(mut self, what: &str) -> Result<()> {
        let errors: Vec<String> = self.bridge.json("__e2eErrors")?;
        if !errors.is_empty() {
            bail!("{what}: console errors in the iOS WebView: {errors:?}");
        }
        if !self.overlaid {
            *IDLE.lock().unwrap_or_else(PoisonError::into_inner) = Some(self);
        }
        Ok(())
    }

    /// Where viewport 0,0 sits on the screen: a tap at the screen's centre on the bare index
    /// page, read back as a `touchstart`'s `clientX/Y`. Printed, for the first CI runs.
    fn calibrate(&mut self) -> Result<()> {
        self.bridge.run(
            "window.__e2eTouch = null; addEventListener('touchstart', e => { \
             __e2eTouch = [e.touches[0].clientX, e.touches[0].clientY]; }, { capture: true, passive: true })",
        )?;
        let (screen_w, screen_h, inner_w): (f64, f64, f64) = self
            .bridge
            .json("[screen.width, screen.height, innerWidth]")?;
        self.scale = screen_w / inner_w;
        let at = (screen_w / 2.0, screen_h / 2.0);
        // The first axe call after a boot can find no accessibility element to tap yet.
        let first = Instant::now();
        while let Err(error) = self.tap_screen(at) {
            if first.elapsed() > Duration::from_secs(20) {
                return Err(error);
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        let started = Instant::now();
        let seen = loop {
            if let Some((x, y)) = self.bridge.json::<Option<(f64, f64)>>("__e2eTouch")? {
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
        static KEYS_NOTED: AtomicBool = AtomicBool::new(false);
        if !KEYS_NOTED.swap(true, Ordering::Relaxed) {
            self.note_keys()?;
        }
        Ok(())
    }

    /// What the page sees of some HID keys, noted once per run: the first CI runs' answer to
    /// which keys the simulator's WebView takes from the injector.
    fn note_keys(&mut self) -> Result<()> {
        self.bridge.run(
            "window.__e2eKeys = []; for (const type of ['keydown', 'keyup', 'beforeinput']) { \
             addEventListener(type, e => { __e2eKeys.push(type + ':' + (e.key ?? e.data) + ':' + (e.code ?? '')); }, true); }",
        )?;
        for (name, usage) in [("a", 4), ("Tab", 43), ("ArrowDown", 81), ("Enter", 40)] {
            self.key(usage)?;
            std::thread::sleep(Duration::from_millis(400));
            let seen: Vec<String> = self.bridge.json("__e2eKeys.splice(0)")?;
            note(&format!(
                "ios key {name} (usage {usage}) reached the page as {seen:?}"
            ));
        }
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
            if !self.bridge.json::<bool>(&hidden)? {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        self.bridge.run(&format!(
            "{}.scrollIntoView({{ block: 'center', inline: 'center' }})",
            element(selector)
        ))?;
        // Momentum from the last swipe keeps the page moving under a tap, as on Android (1104).
        let read = format!(
            "(r => [r.x + r.width / 2, r.y + r.height / 2])({}.getBoundingClientRect())",
            element(selector)
        );
        let mut at: (f64, f64) = self.bridge.json(&read)?;
        for _ in 0..40 {
            std::thread::sleep(Duration::from_millis(50));
            let next: (f64, f64) = self.bridge.json(&read)?;
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

    /// Returns once the page saw the tap end, as Android's (3268); a tap the page never felt
    /// is sent again (3532). A touch that raised no click (a disabled target) counts once lifted.
    fn tap(&mut self, x: f64, y: f64) -> Result<()> {
        self.bridge.run(
            "window.__tap?.abort(); const ac = window.__tap = new AbortController(); \
             window.__tapped = 0; \
             const saw = n => () => { __tapped = Math.max(__tapped, n); }; \
             addEventListener('touchstart', saw(1), { capture: true, passive: true, signal: ac.signal }); \
             addEventListener('touchend', saw(2), { capture: true, passive: true, signal: ac.signal }); \
             for (const type of ['click', 'pointercancel']) { \
               addEventListener(type, saw(3), { capture: true, signal: ac.signal }); }",
        )?;
        self.close_picker()?;
        for _ in 0..3 {
            self.mark(&format!("> tap {x},{y}"))?;
            self.tap_screen(self.screen(x, y))?;
            let started = Instant::now();
            let mut lifted = None;
            while started.elapsed() < Duration::from_secs(3) {
                match self.bridge.json::<u8>("__tapped")? {
                    3.. => return Ok(()),
                    // WebKit raises the click a moment after the lift.
                    2 if lifted.get_or_insert_with(Instant::now).elapsed()
                        > Duration::from_millis(500) =>
                    {
                        return Ok(());
                    }
                    _ => {}
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            // Felt but not yet lifted: a busy page; the scenario's own wait decides.
            if self.bridge.json::<u8>("__tapped")? > 0 {
                return Ok(());
            }
            trace(&format!(
                "! the page felt no tap at {x}, {y}: tapping again"
            ));
        }
        let under: String = self
            .bridge
            .json(&format!(
                "(e => e ? e.outerHTML.slice(0, 120) : 'nothing')(document.elementFromPoint({x}, {y}))"
            ))
            .unwrap_or_default();
        bail!("the page felt none of three taps at {x}, {y}, over {under}")
    }

    /// A focused select may show iOS's picker, which takes the next tap and key (3700, 3701): an
    /// Escape closes it, swallowed at the window when no picker took it.
    fn close_picker(&mut self) -> Result<()> {
        if !self
            .bridge
            .json::<bool>("document.activeElement?.tagName === 'SELECT'")?
        {
            return Ok(());
        }
        self.bridge.run(
            "window.__picker?.abort(); const ac = window.__picker = new AbortController(); \
             window.__pickerFelt = 0; \
             for (const type of ['keydown', 'keyup']) addEventListener(type, e => { \
               if (e.key !== 'Escape') return; \
               e.stopImmediatePropagation(); e.preventDefault(); __pickerFelt++; \
             }, { capture: true, signal: ac.signal });",
        )?;
        self.mark("> close a select picker")?;
        self.key(41)?;
        // The page felt the keydown, so no picker was up: hold the swallow for its keyup too.
        let started = Instant::now();
        let felt = loop {
            let felt: u8 = self.bridge.json("__pickerFelt")?;
            if felt >= 2 || started.elapsed() > Duration::from_millis(300) {
                break felt;
            }
            std::thread::sleep(Duration::from_millis(25));
        };
        self.bridge.run("window.__picker.abort()")?;
        self.mark(match felt {
            0 => "  a select picker took the Escape",
            _ => "  no select picker was up",
        })
    }

    /// A line in the page's [`EVENT_LOG`], to place the injector's calls among the events.
    fn mark(&mut self, line: &str) -> Result<()> {
        self.bridge.run(&format!("window.__e2eMark?.({line:?})"))
    }

    /// A touch from viewport point `(x, y)` by `(dx, dy)` over `secs`, clamped to the viewport:
    /// a touch lifted off the WebView would leave it a stuck touch. AXe's `swipe` flicks a
    /// page; its `drag` emits every move, which pointer-event drags need.
    fn swipe(&mut self, from: (f64, f64), dx: f64, dy: f64, secs: f64) -> Result<()> {
        self.stroke("swipe", from, dx, dy, secs)
    }

    fn drag_by(&mut self, from: (f64, f64), dx: f64, dy: f64, secs: f64) -> Result<()> {
        self.stroke("drag", from, dx, dy, secs)
    }

    fn stroke(
        &mut self,
        verb: &str,
        (x, y): (f64, f64),
        dx: f64,
        dy: f64,
        secs: f64,
    ) -> Result<()> {
        let (vw, vh): (f64, f64) = self.bridge.json("[innerWidth, innerHeight]")?;
        let clamp = |x: f64, y: f64| self.screen(x.clamp(0.0, vw - 1.0), y.clamp(0.0, vh - 1.0));
        let (from, to) = (clamp(x, y), clamp(x + dx, y + dy));
        let [x1, y1, x2, y2] = [point(from.0), point(from.1), point(to.0), point(to.1)];
        let secs = format!("{secs:.2}");
        self.mark(&format!("> {verb} {x1},{y1} to {x2},{y2} pt"))?;
        match self.input {
            Injector::Axe => self.inject(&[
                verb,
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

    fn chord(&self, modifiers: &[u8], key: Key) -> Result<()> {
        let (usage, _) = hid_usage(key)?;
        let modifiers: Vec<String> = modifiers.iter().map(u8::to_string).collect();
        match self.input {
            Injector::Axe => self.inject(&[
                "key-combo",
                "--modifiers",
                &modifiers.join(","),
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
        self.inject_output(args).map(drop)
    }

    /// As [`Ios::inject`]; the tool's stdout.
    fn inject_output(&self, args: &[&str]) -> Result<String> {
        let tool = match self.input {
            Injector::Axe => "axe",
            Injector::Idb => "idb",
        };
        // AXe's accessibility translation is not up for the first seconds of a fresh simulator.
        let mut tries = 0;
        loop {
            let out = traced(
                Command::new(tool).args(args).args(["--udid", &self.udid]),
                &format!("{tool} {args:?}"),
            )?;
            if out.status.success() {
                return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
            }
            let error = String::from_utf8_lossy(&out.stderr);
            tries += 1;
            // A tap that dies without a word is the tool under load: once or twice more.
            let starting = tries < 20 && error.contains("No translation object");
            if starting || tries < 3 && error.trim().is_empty() {
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            bail!("{tool} {args:?} failed: {}", error.trim());
        }
    }
}

impl Ios {
    /// Keeps the screen and page text of a red scenario before the app stops; the runner's own
    /// shot comes after.
    fn keep_failure(&mut self) {
        let dir = std::env::var_os("E2E_ARTIFACTS")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let name = self.unit.replace("::", "-");
        let shot = dir.join(format!("{name}-failure.png"));
        let _ = simctl(
            &["io", &self.udid, "screenshot", &shot.to_string_lossy()],
            &[],
        );
        if let Ok(text) = self.bridge.json::<String>("document.body.innerText") {
            let _ = std::fs::write(dir.join(format!("{name}-failure.txt")), text);
        }
        if let Ok(log) = self.bridge.json::<Vec<String>>(EVENT_LINES) {
            eprintln!(
                "e2e ios: the page's last events at {}, focus now on {}:\n  {}",
                self.bridge
                    .json::<String>(
                        "`${Math.round(performance.now())} ms, zoom ${visualViewport?.scale}, \
                         scroll ${scrollX},${scrollY}, page focused ${document.hasFocus()}`"
                    )
                    .unwrap_or_default(),
                self.bridge
                    .json::<String>("document.activeElement?.outerHTML.slice(0, 120) ?? 'nothing'")
                    .unwrap_or_default(),
                log.join("\n  ")
            );
        }
        if let Ok(tree) = self.inject_output(&["describe-ui"]) {
            let _ = std::fs::write(dir.join(format!("{name}-failure.json")), tree);
        }
    }
}

impl Drop for Ios {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.keep_failure();
        }
        let _ = simctl(&["terminate", &self.udid, &self.bundle], &[]);
    }
}

/// `command` under [`TOOL_LIMIT`], each call noted in [`TRACE`]: a call that never returns is
/// the last line without its `<`.
fn traced(command: &mut Command, label: &str) -> Result<Output> {
    trace(&format!("> {label}"));
    let started = Instant::now();
    let out = output_within(command, TOOL_LIMIT).with_context(|| format!("{label} hung"));
    trace(&format!("< {label} {:?}", started.elapsed()));
    out
}

fn trace(line: &str) {
    let dir = std::env::var_os("E2E_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let file = std::fs::File::options()
        .create(true)
        .append(true)
        .open(dir.join(TRACE));
    if let Ok(mut file) = file {
        let _ = writeln!(file, "{line}");
    }
}

/// A screen point as the injectors take it: whole points.
fn point(at: f64) -> String {
    (at.round() as i64).to_string()
}

/// `xcrun simctl` with `args` and `env`; its stdout.
fn simctl(args: &[&str], env: &[(&str, &str)]) -> Result<String> {
    let out = traced(
        Command::new("xcrun")
            .arg("simctl")
            .args(args)
            .envs(env.iter().copied()),
        &format!("simctl {args:?}"),
    )?;
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

    /// Both taps as four raw touches in one AXe `batch` (one HID event). The CI simulator still
    /// lands them 500 ms apart and raises no `dblclick`; a call per tap took 1.6 s (3702).
    async fn double_click(&mut self, selector: &str) -> Result<()> {
        let (x, y) = self.centre(selector)?;
        if self.input != Injector::Axe {
            self.tap(x, y)?;
            return self.tap(x, y);
        }
        self.close_picker()?;
        let (sx, sy) = self.screen(x, y);
        let touch = |edge| format!("touch -x {} -y {} --{edge}", point(sx), point(sy));
        let (down, up) = (touch("down"), touch("up"));
        self.mark(&format!("> double tap {x},{y}"))?;
        let mut args = vec!["batch"];
        for step in [&down, &up, &down, &up] {
            args.extend(["--step", step.as_str()]);
        }
        self.inject(&args)
    }

    /// A point under the status bar moves just below it: the system takes taps there, so a
    /// backdrop tap at 4,4 never reached the page (3416).
    async fn click_at(&mut self, x: f64, y: f64) -> Result<()> {
        let inset: f64 = self.bridge.json(
            "(() => { const e = document.createElement('div'); \
             e.style.cssText = 'position: fixed; padding-top: env(safe-area-inset-top)'; \
             document.body.append(e); const top = parseFloat(getComputedStyle(e).paddingTop); \
             e.remove(); return top || 0; })()",
        )?;
        let bar = STATUS_BAR.max(inset * self.scale + self.origin.1);
        let below = (bar - self.origin.1) / self.scale + 1.0;
        self.tap(x, y.max(below))
    }

    async fn viewport(&mut self) -> Result<(f64, f64)> {
        self.bridge.json("[innerWidth, innerHeight]")
    }

    /// A Tab is Option+Tab: WebKit's own skips buttons and links on iOS, Option+Tab visits
    /// every control.
    async fn press(&mut self, key: Key) -> Result<()> {
        self.close_picker()?;
        if key.code == "Tab" && self.input == Injector::Axe {
            return self.chord(&[ALT], key);
        }
        match hid_usage(key)? {
            (usage, false) => self.key(usage),
            (_, true) => self.chord(&[SHIFT], key),
        }
    }

    async fn press_shift(&mut self, key: Key) -> Result<()> {
        self.close_picker()?;
        if key.code == "Tab" && self.input == Injector::Axe {
            return self.chord(&[ALT, SHIFT], key);
        }
        self.chord(&[SHIFT], key)
    }

    async fn press_ctrl(&mut self, key: Key) -> Result<()> {
        self.close_picker()?;
        self.chord(&[CTRL], key)
    }

    /// Command, as on the Mac: libero reads `mod` as Meta on Apple platforms.
    async fn press_mod(&mut self, key: Key) -> Result<()> {
        self.close_picker()?;
        self.chord(&[CMD], key)
    }

    async fn press_alt(&mut self, key: Key) -> Result<()> {
        self.close_picker()?;
        self.chord(&[ALT], key)
    }

    /// One injector call per character, at about a typist's pace: one call for the lot
    /// outran PinField's move to the next cell on Android.
    async fn type_text(&mut self, text: &str) -> Result<()> {
        self.close_picker()?;
        for ch in text.chars() {
            self.type_all(&ch.to_string())?;
        }
        Ok(())
    }

    /// The whole text in one call, as fast as the injector types; `ms` is not kept.
    async fn type_burst(&mut self, text: &str, ms: u64) -> Result<()> {
        let _ = ms;
        self.close_picker()?;
        self.type_all(text)
    }

    async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()> {
        let from = self.centre(selector)?;
        self.drag_by(from, dx, dy, 0.5)
    }

    async fn swipe_from(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.swipe((x, y), dx, dy, 0.5)
    }

    async fn drag_at(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.drag_by((x, y), dx, dy, 0.5)
    }

    /// By script, as the desktop driver: a swipe flicks the page well past `dy`, and a slow
    /// drag takes 10 s a call under load (3631).
    async fn scroll_by(&mut self, dy: f64) -> Result<()> {
        self.bridge
            .run(&format!("scrollBy({{ top: {dy}, behavior: 'instant' }})"))
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
        self.bridge.json(&format!("await ({expression})"))
    }

    /// The injector's accessibility dump of the whole screen, as JSON text.
    async fn system_text(&mut self) -> Result<String> {
        self.overlaid = true;
        match self.input {
            Injector::Axe => self.inject_output(&["describe-ui"]),
            Injector::Idb => self.inject_output(&["ui", "describe-all"]),
        }
    }

    /// Presses `trigger`, then taps the dialog's Allow button, found by its label once it shows.
    async fn allow_permission(&mut self, trigger: &str) -> Result<()> {
        if self.input != Injector::Axe {
            bail!("Ios: only axe taps a dialog by its label");
        }
        self.click(trigger).await?;
        self.overlaid = true;
        let started = Instant::now();
        loop {
            match self.inject(&["tap", "--label", "Allow"]) {
                Ok(()) => return Ok(()),
                Err(error) if started.elapsed() > Duration::from_secs(15) => {
                    return Err(error.context("no Allow button after the press"));
                }
                Err(_) => std::thread::sleep(Duration::from_millis(100)),
            }
        }
    }

    /// `simctl launch --terminate-running-process` ends the running app. The old handle is
    /// forgotten: its drop would terminate the new one.
    async fn relaunch(&mut self) -> Result<()> {
        let fresh = Self::launch(&self.unit)?;
        let route = std::mem::take(&mut self.route);
        std::mem::forget(std::mem::replace(self, fresh));
        self.route = route;
        self.mount()
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

    /// As the desktop's: a flag the frame sets, polled, then `settle`'s idle rounds.
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
