//! One process-wide browser and tokio runtime; each test gets a fresh page.

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

/// The two viewports every pass runs at. Mobile is 390px, the docs reviews' width.
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

/// Emulates the colour scheme and optional reduced motion in one call:
/// `setEmulatedMedia` replaces the whole feature list.
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

/// Headless Chrome reports `(pointer: coarse)` and CDP cannot emulate `pointer`: fakes the
/// `matchMedia` lists of `(pointer: coarse)` and `(pointer: fine)`, fine at first. Reloads.
pub async fn fake_pointer(page: &Page) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;
    const FAKE: &str = "{
        const list = coarse => Object.assign(new EventTarget(), { coarse, matches: !coarse });
        const lists = { '(pointer: coarse)': list(true), '(pointer: fine)': list(false) };
        const real = window.matchMedia.bind(window);
        window.matchMedia = q => lists[q.trim()] ?? real(q);
        window.__libero_set_coarse = on => Object.values(lists).forEach(l => {
            const matches = l.coarse === on;
            if (l.matches !== matches) {
                l.matches = matches;
                l.dispatchEvent(Object.assign(new Event('change'), { matches }));
            }
        });
    }";
    page.execute(AddScriptToEvaluateOnNewDocumentParams::new(FAKE))
        .await
        .context("install the fake pointer")?;
    page.reload()
        .await
        .context("reload with the fake pointer")?;
    Ok(())
}

/// Switches the [`fake_pointer`] lists and fires their `change` events.
pub async fn set_coarse_pointer(page: &Page, coarse: bool) -> Result<()> {
    page.evaluate(format!("window.__libero_set_coarse({coarse})"))
        .await
        .context("switch the fake pointer")?;
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
    /// Caps concurrent navigations: the first runs alone to fill the cold HTTP cache,
    /// then [`NAVIGATIONS`] at once (todo 823).
    navigations: tokio::sync::Semaphore,
    primed: std::sync::Once,
}

/// Navigations at once after the first, `E2E_NAVIGATIONS` to override.
const NAVIGATIONS: usize = 16;

/// Opens the navigation cap once the first navigation ended, however it ended.
struct PrimesOnDrop;

impl Drop for PrimesOnDrop {
    fn drop(&mut self) {
        let harness = harness();
        harness.primed.call_once(|| {
            let limit = std::env::var("E2E_NAVIGATIONS")
                .ok()
                .and_then(|n| n.parse().ok())
                .unwrap_or(NAVIGATIONS);
            harness.navigations.add_permits(limit.max(1) - 1);
        });
    }
}

static HARNESS: OnceLock<Harness> = OnceLock::new();

/// Held by a test that sets CDP permissions, from the first set until its page
/// closed: closing a page that set one resets every override, another test's too.
pub static PERMISSIONS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn harness() -> &'static Harness {
    HARNESS.get_or_init(|| {
        let runtime = Runtime::new().expect("tokio runtime");
        let browser = runtime.block_on(async {
            let config = BrowserConfig::builder()
                // Headless, and with a window big enough that the desktop
                // viewport override is never clamped by the outer window.
                .window_size(1280, 900)
                // Own profile per run: the fixed default hits "SingletonLock: File exists"
                // after one interrupted run, or with two runs at once.
                .user_data_dir(chrome_profile())
                // Long enough for a cold wasm bundle.
                .request_timeout(Duration::from_secs(120))
                // Instant scrolls: a background page stalls smooth ones (todo 687).
                .arg("disable-smooth-scrolling")
                // A fake camera and microphone for `use_user_media`; the grant stays a CDP call.
                .arg("use-fake-device-for-media-stream")
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
            navigations: tokio::sync::Semaphore::new(1),
            primed: std::sync::Once::new(),
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
    /// Attached before navigation, to catch errors thrown once during the first mount.
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
    /// Opens a fixture route and waits for `data-fixture-ready`: during its first build
    /// `dx` serves a placeholder at a success status (`codebase/testing`).
    pub async fn open(route: &str, viewport: Viewport) -> Result<Self> {
        Self::open_in(route, viewport, Scheme::Light).await
    }

    /// [`Fixture::open`] under a colour scheme, emulated before navigation.
    pub async fn open_in(route: &str, viewport: Viewport, scheme: Scheme) -> Result<Self> {
        Self::open_until(route, viewport, scheme, "[data-fixture-ready]").await
    }

    /// [`Fixture::open_in`] for an app without the fixture marker, the docs
    /// site in the sweep: `ready` is what the app renders once it is up.
    pub async fn open_until(
        route: &str,
        viewport: Viewport,
        scheme: Scheme,
        ready: &str,
    ) -> Result<Self> {
        // First: panicking before Chrome launches keeps a bare `cargo test` from leaking a browser.
        let url = format!("{}{}", crate::base_url(), route);

        let _permit = harness()
            .navigations
            .acquire()
            .await
            .expect("navigation semaphore");
        let _primes = PrimesOnDrop;

        // Failures up to the ready marker are journalled as `navigation` (todo 364).
        let navigation_started = std::time::Instant::now();
        let stage = |stage: &'static str, started: std::time::Instant, error: &anyhow::Error| {
            crate::journal::gave_up(&crate::journal::GaveUp {
                kind: "navigation",
                how: "failed",
                what: &format!("{stage} for {url} ({error})"),
                // chromiumoxide's request timeout bounds these, not `E2E_TIMEOUT_MS`.
                budget: Duration::from_secs(120),
                elapsed: started.elapsed(),
                slowest_poll: started.elapsed(),
                polls: 1,
            });
        };

        // Background plus focus emulation: foreground pages timed out on navigation
        // 1-2 times a run, background alone was 5x slower (measured 2026-09-19).
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

        // Every page acts focused: another test's new page would blur it and close
        // open popovers (todo 356, `isolation::a_page_keeps_focus_while_another_opens`).
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
        // An unhandled Alt+ArrowLeft goes Back in the active tab, any test's page
        // once `frames::start` brought one to front: no `about:blank` to go back to.
        page.execute(
            chromiumoxide::cdp::browser_protocol::page::ResetNavigationHistoryParams::default(),
        )
        .await
        .context("drop the about:blank history entry")?;

        let fixture = Fixture {
            page,
            viewport,
            scheme,
            console,
            closes_on_drop,
        };
        crate::wait::for_selector_kind("fixture-ready", &fixture.page, ready)
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

    /// The page is drawn in `self.scheme`: media query, no pinning `data-lsx-theme`, body
    /// paint. Else a no-op emulation lets every dark pass measure the light page.
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

    /// Writes a PNG of the current page and returns its path.
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

    /// Closes the page and waits. Fails on a logged `dioxus_signals` warning (todo 719).
    pub async fn close(mut self) -> Result<()> {
        self.closes_on_drop.0 = None;
        self.page.close().await.context("close the page")?;
        let messages = self.console.peek();
        let warned = crate::passes::console::signal_warnings(&messages);
        if !warned.is_empty() {
            anyhow::bail!("dioxus_signals warned:\n  {}", warned.join("\n  "));
        }
        Ok(())
    }
}
