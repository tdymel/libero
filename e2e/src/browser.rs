//! One browser, many pages.
//!
//! The browser and the tokio runtime are process-wide and built once. Spawning
//! a browser per test would cost a process launch and its memory for every
//! case; a page is cheap by comparison, and each test gets a fresh one so
//! nothing leaks between them.

use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Context, Result};
use chromiumoxide::cdp::browser_protocol::emulation::{
    MediaFeature, SetDeviceMetricsOverrideParams, SetEmulatedMediaParams,
    SetFocusEmulationEnabledParams,
};
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams,
};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use tokio::runtime::Runtime;

/// The two viewports every pass runs at.
///
/// Mobile is 390px because that is the width the docs reviews use, so a finding
/// here and a finding there describe the same screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Viewport {
    Desktop,
    Mobile,
}

impl Viewport {
    pub const ALL: [Viewport; 2] = [Viewport::Desktop, Viewport::Mobile];

    pub fn size(self) -> (i64, i64) {
        match self {
            Viewport::Desktop => (1280, 800),
            Viewport::Mobile => (390, 844),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Viewport::Desktop => "desktop",
            Viewport::Mobile => "mobile",
        }
    }
}

/// The colour scheme a page is opened under, through the system case:
/// `prefers-color-scheme`, which is what an app naming no theme follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    Light,
    Dark,
}

impl Scheme {
    pub fn name(self) -> &'static str {
        match self {
            Scheme::Light => "light",
            Scheme::Dark => "dark",
        }
    }
}

/// Emulate the colour scheme, and `prefers-reduced-motion` when given.
///
/// One call, because `Emulation.setEmulatedMedia` replaces the whole feature
/// list: setting reduced motion alone would drop a dark scheme set earlier.
pub async fn emulate_media(
    page: &Page,
    scheme: Scheme,
    reduced_motion: Option<bool>,
) -> Result<()> {
    let mut features = vec![MediaFeature::new("prefers-color-scheme", scheme.name())];
    if let Some(reduced) = reduced_motion {
        let value = if reduced { "reduce" } else { "no-preference" };
        features.push(MediaFeature::new("prefers-reduced-motion", value));
    }
    page.execute(SetEmulatedMediaParams::builder().features(features).build())
        .await?;
    Ok(())
}

/// Where this run's Chrome profile lives. The runner sets it and cleans it up;
/// a bare `cargo test` gets a pid-scoped fallback so it is still unique.
fn chrome_profile() -> std::path::PathBuf {
    std::env::var("E2E_CHROME_PROFILE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join(format!("e2e-chrome-{}", std::process::id())))
}

struct Harness {
    runtime: Runtime,
    browser: Browser,
    /// Caps how many fixtures navigate at once.
    ///
    /// Every run gets a fresh Chrome profile, so the HTTP cache is empty and
    /// each page load pulls the whole wasm bundle again. Eight tests opening
    /// pages simultaneously against one dev server timed out the lot of them -
    /// a failure that reads like every component being broken. Three at a time
    /// keeps the server honest and costs nothing: the tests are dominated by
    /// the browser, not by this.
    navigations: tokio::sync::Semaphore,
}

static HARNESS: OnceLock<Harness> = OnceLock::new();

fn harness() -> &'static Harness {
    HARNESS.get_or_init(|| {
        let runtime = Runtime::new().expect("tokio runtime");
        let browser = runtime.block_on(async {
            let config = BrowserConfig::builder()
                // Headless, and with a window big enough that the desktop
                // viewport override is never clamped by the outer window.
                .window_size(1280, 900)
                // A profile directory of our own, per run.
                //
                // chromiumoxide's default is a fixed `/tmp/chromiumoxide-runner`,
                // and Chrome refuses to start a second instance against a
                // profile another process still holds: "Failed to create
                // SingletonLock: File exists". So one interrupted run leaves a
                // browser behind and *every subsequent run* dies at launch,
                // with an error that says nothing about the real cause. It also
                // means two agents running the suite at once break each other.
                .user_data_dir(chrome_profile())
                // The default is short enough that a cold wasm bundle can miss
                // it. A timeout here should mean "the app is broken", never
                // "the bundle was still downloading".
                .request_timeout(Duration::from_secs(120))
                .build()
                .expect("browser config");
            let (browser, mut handler) = Browser::launch(config).await.expect("launch chromium");
            // The handler stream drives every CDP message. Nothing works if it
            // is not polled, and the failure looks like every call hanging.
            tokio::spawn(async move { while handler.next().await.is_some() {} });
            browser
        });
        Harness {
            runtime,
            browser,
            navigations: tokio::sync::Semaphore::new(3),
        }
    })
}

/// Run an async body on the shared runtime. Tests are ordinary `#[test]` fns.
pub fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    harness().runtime.block_on(fut)
}

/// A fixture page, open at one route and one viewport.
pub struct Fixture {
    pub page: Page,
    pub viewport: Viewport,
    pub scheme: Scheme,
    /// Attached **before** navigation, so it catches errors raised during the
    /// app's first mount. Attaching after `goto` - which is what this harness
    /// did at first - silently misses exactly the errors most worth having,
    /// since a component that throws while mounting throws once and never
    /// again.
    pub console: crate::passes::console::Recorder,
    closes_on_drop: ClosesOnDrop,
}

/// Closes a page that was never closed by hand: a failed `open` or a test that
/// panicked or returned early. Leaked pages were the amplifier in todo 465.
struct ClosesOnDrop(Option<Page>);

impl Drop for ClosesOnDrop {
    fn drop(&mut self) {
        if let Some(page) = self.0.take() {
            // Spawned: `Drop` cannot await, and it may run inside `block_on`.
            harness().runtime.spawn(async move {
                let _ = page.close().await;
            });
        }
    }
}

impl Fixture {
    /// Open a fixture route and wait until the app has actually rendered.
    ///
    /// The wait is on `data-fixture-ready`, not on the navigation resolving:
    /// while its first build runs, `dx` answers every path with a 404
    /// placeholder *at a success status*, so a page that loaded proves nothing
    /// (`codebase/testing`). The marker is rendered by the app itself, which a
    /// placeholder cannot fake.
    pub async fn open(route: &str, viewport: Viewport) -> Result<Self> {
        Self::open_in(route, viewport, Scheme::Light).await
    }

    /// [`Fixture::open`] under a colour scheme. The emulation is in place
    /// before navigation, so the first paint is already in that scheme, and
    /// the page is checked to have resolved it (see [`Fixture::assert_scheme`]).
    pub async fn open_in(route: &str, viewport: Viewport, scheme: Scheme) -> Result<Self> {
        // Resolved first, deliberately: it panics with an explanation when the
        // runner did not set it, and doing that *before* launching Chrome is
        // what stops a bare `cargo test` leaving a browser and a profile
        // directory behind on every invocation.
        let url = format!("{}{}", crate::base_url(), route);

        let _permit = harness()
            .navigations
            .acquire()
            .await
            .expect("navigation semaphore");

        // Everything from here to the ready marker is journalled as
        // `navigation` (todo 364). It is the half of the run that shares the
        // browser connection with every other test, so it is where a red run
        // that failed *everything* has to be distinguished from one page's own
        // trouble. `navigated` measures it; nothing is written on success.
        let navigation_started = std::time::Instant::now();
        let stage = |stage: &'static str, started: std::time::Instant, error: &anyhow::Error| {
            crate::journal::gave_up(&crate::journal::GaveUp {
                kind: "navigation",
                how: "failed",
                what: &format!("{stage} for {url} ({error})"),
                // chromiumoxide's own request timeout bounds these, not
                // `E2E_TIMEOUT_MS`; recorded so the line is readable without
                // knowing that.
                budget: Duration::from_secs(120),
                elapsed: started.elapsed(),
                slowest_poll: started.elapsed(),
                polls: 1,
            });
        };

        // In the background, so a new page never takes activation from the
        // pages other tests are driving. Focus emulation alone (below) stopped
        // the blur, but runs with it and foreground pages timed out on
        // navigation 1-2 times per full run (the 30s CDP navigation limit, or
        // the 15s ready wait), against none in five runs with both. Measured
        // 2026-09-19; why foreground creation slows loads is not known.
        // Background alone passed too, but took 118s a run against 21s.
        let page = harness()
            .browser
            .new_page(
                chromiumoxide::cdp::browser_protocol::target::CreateTargetParams::builder()
                    .url("about:blank")
                    .background(true)
                    .build()
                    .map_err(anyhow::Error::msg)?,
            )
            .await
            .context("open a page")
            .inspect_err(|error| stage("creating the page", navigation_started, error))?;
        // Armed at once, so every `?` below closes the page too.
        let closes_on_drop = ClosesOnDrop(Some(page.clone()));

        let (width, height) = viewport.size();
        let at = std::time::Instant::now();
        page.execute(SetDeviceMetricsOverrideParams::new(
            width,
            height,
            1.0,
            viewport == Viewport::Mobile,
        ))
        .await
        .context("set the viewport")
        .inspect_err(|error| stage("setting the viewport", at, error))?;

        // Every page behaves as the focused one. The tests share one browser,
        // and a page another test opens takes window focus: the first page
        // gets `blur`, and an open combobox, menu or popover closes by itself
        // mid-test. `isolation::a_page_keeps_focus_while_another_opens` pins
        // it; without this it fails every time (todo 356).
        page.execute(SetFocusEmulationEnabledParams::new(true))
            .await
            .context("emulate a focused page")?;

        if scheme == Scheme::Dark {
            emulate_media(&page, scheme, None)
                .await
                .context("emulate the dark scheme")?;
        }

        // Before `goto`, deliberately. See the field's doc comment.
        let console = crate::passes::console::Recorder::attach(&page).await?;

        let at = std::time::Instant::now();
        page.goto(&url)
            .await
            .with_context(|| format!("go to {url}"))
            .inspect_err(|error| stage("navigating", at, error))?;

        let fixture = Fixture {
            page,
            viewport,
            scheme,
            console,
            closes_on_drop,
        };
        crate::wait::for_selector_kind("fixture-ready", &fixture.page, "[data-fixture-ready]")
            .await
            .with_context(|| {
                format!(
                    "the fixture at {url} never rendered. If the page is dx's \
                     \"not serving a web app\" placeholder, the build was still running."
                )
            })?;
        if scheme == Scheme::Dark {
            fixture.assert_scheme().await?;
        }
        Ok(fixture)
    }

    /// The page must be drawn in `self.scheme`: the media query matches, no
    /// `data-lsx-theme` pins another one, and the body is painted dark.
    ///
    /// Without it an emulation that did nothing, or a stored
    /// `lsx-color-scheme` pinning the light theme, leaves every dark pass
    /// measuring the light page and reporting it clean.
    pub async fn assert_scheme(&self) -> Result<()> {
        let (matches, pinned, luminance): (bool, Option<String>, f64) = self
            .page
            .evaluate(
                "(() => { \
                   const [r, g, b, a = 1] = getComputedStyle(document.body).backgroundColor \
                     .match(/[\\d.]+/g).map(Number); \
                   return [matchMedia('(prefers-color-scheme: dark)').matches, \
                     document.documentElement.getAttribute('data-lsx-theme'), \
                     a === 0 ? -1 : (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255]; })()",
            )
            .await?
            .into_value()?;
        let dark = self.scheme == Scheme::Dark;
        let pinned_other = pinned.as_deref().is_some_and(|p| p != self.scheme.name());
        // A transparent body (-1) is unreadable, never "dark".
        if matches != dark || pinned_other || luminance < 0.0 || (luminance < 0.5) != dark {
            anyhow::bail!(
                "the page is not drawn in the {} scheme: prefers-color-scheme: dark {matches}, \
                 data-lsx-theme {pinned:?}, body luma {luminance:.2}",
                self.scheme.name()
            );
        }
        Ok(())
    }

    /// Write a PNG of the current page and return where it went.
    ///
    /// This is the trace viewer we gave up with Playwright, in its cheapest
    /// useful form. A focus or layout failure described only in prose costs the
    /// next person a re-run to see; a picture costs them nothing.
    pub async fn screenshot(&self, name: &str) -> Result<std::path::PathBuf> {
        let dir = std::env::var("E2E_ARTIFACTS")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("e2e-artifacts"));
        std::fs::create_dir_all(&dir).context("create the artifacts directory")?;
        let path = dir.join(format!("{name}-{}.png", self.viewport.name()));

        let png = self
            .page
            .screenshot(
                CaptureScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .capture_beyond_viewport(true)
                    .build(),
            )
            .await
            .context("capture a screenshot")?;
        std::fs::write(&path, png).context("write the screenshot")?;
        Ok(path)
    }

    /// Close the page and wait for it. A fixture dropped without this closes
    /// its page in the background instead.
    pub async fn close(mut self) -> Result<()> {
        self.closes_on_drop.0 = None;
        self.page.close().await.context("close the page")
    }
}
