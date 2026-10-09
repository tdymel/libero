//! One scenario, several platforms (todo 822): a scenario is an `async fn`
//! over any [`Driver`], and [`scenario!`](crate::scenario) runs it on the web
//! backend (Chromium over CDP), with the `native` feature on Blitz, and with
//! `android` in the emulator's WebView (964), with `desktop` in wry's (1126).
//!
//! ```ignore
//! async fn pages<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
//!     d.click("[aria-label=\"Go to page 3\"]").await?;
//!     eventually(d, "page 3", async |d| Ok(d.text("[aria-current=page]").await? == "3")).await
//! }
//! e2e::scenario!(paging, "/pagination", pages);
//! ```

use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use crate::passes::keyboard::Key;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Web,
    Native,
    Android,
    Desktop,
}

/// A `getBoundingClientRect()` in CSS px.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The JS expression for `selector`'s first match.
fn query(selector: &str) -> String {
    format!(
        "document.querySelector({})",
        serde_json::to_string(selector).expect("a string serialises")
    )
}

/// What a scenario may do and read. Every read is of the settled page on
/// Blitz; on the web it is a snapshot, so assert through [`eventually`].
// Driven by one thread's `block_on`, so the futures need no `Send` bound.
#[allow(async_fn_in_trait)]
pub trait Driver {
    fn platform(&self) -> Platform;
    async fn click(&mut self, selector: &str) -> Result<()>;
    /// Moves the pointer onto the first match's centre, pressing nothing.
    async fn hover(&mut self, selector: &str) -> Result<()>;
    /// Two clicks at the first match's centre, the second a `dblclick`.
    async fn double_click(&mut self, selector: &str) -> Result<()>;
    /// A click at a viewport point, e.g. on a backdrop.
    async fn click_at(&mut self, x: f64, y: f64) -> Result<()>;
    /// The viewport's `(width, height)` in CSS px.
    async fn viewport(&mut self) -> Result<(f64, f64)>;
    async fn press(&mut self, key: Key) -> Result<()>;
    async fn press_shift(&mut self, key: Key) -> Result<()>;
    /// A chord with Ctrl held; give `key` no `text`.
    async fn press_ctrl(&mut self, key: Key) -> Result<()>;
    /// A chord with Alt held; give `key` no `text`.
    async fn press_alt(&mut self, key: Key) -> Result<()> {
        let _ = key;
        bail!("{:?}: no Alt chord", self.platform())
    }
    async fn type_text(&mut self, text: &str) -> Result<()>;
    /// Types `text` one key per `ms`, faster than a WebView's IPC round trip (1026).
    async fn type_burst(&mut self, text: &str, ms: u64) -> Result<()> {
        let _ = ms;
        self.type_text(text).await
    }
    /// Text committed with no key event, as a soft keyboard's IME sends it.
    async fn insert_text(&mut self, text: &str) -> Result<()> {
        let _ = text;
        bail!("{:?}: no text without a key", self.platform())
    }
    /// Presses at the first match's centre, moves by `(dx, dy)` in steps, releases.
    async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()>;
    /// Scrolls the page by `dy` CSS px (positive: down), by a swipe on touch.
    async fn scroll_by(&mut self, dy: f64) -> Result<()> {
        let _ = dy;
        bail!("{:?}: no page scroll", self.platform())
    }
    /// Turns the wheel over the first match's centre by about `dy` CSS px (positive: down).
    /// Desktop turns whole notches, so `dy` is rounded there: assert movement, not exact px.
    async fn wheel(&mut self, selector: &str, dy: f64) -> Result<()> {
        let _ = (selector, dy);
        bail!("{:?}: no wheel", self.platform())
    }
    /// The escape hatch for a measurement no typed read covers: `expression`'s JSON value,
    /// a promise awaited. Prefer a typed method; name the probe a const in the test file.
    async fn evaluate(&mut self, expression: &str) -> Result<serde_json::Value> {
        let _ = expression;
        bail!("{:?}: no script reads", self.platform())
    }
    /// The first match's `(scrollLeft, scrollTop)`.
    async fn scroll_pos(&mut self, selector: &str) -> Result<(f64, f64)> {
        let value = self
            .evaluate(&format!(
                "(e => [e.scrollLeft, e.scrollTop])({})",
                query(selector)
            ))
            .await?;
        let read = |at: usize| value[at].as_f64().context("no scroll offset read");
        Ok((read(0)?, read(1)?))
    }
    /// The first match's `scrollHeight`.
    async fn scroll_height(&mut self, selector: &str) -> Result<f64> {
        let value = self
            .evaluate(&format!("{}.scrollHeight", query(selector)))
            .await?;
        value.as_f64().context("no scroll height read")
    }
    /// Scrolls the first match to `(left, top)`, as `scrollTo` does; on Blitz by wheel, so a
    /// target past the end is clamped and the offset lands within a px.
    async fn scroll_to(&mut self, selector: &str, left: f64, top: f64) -> Result<()> {
        self.evaluate(&format!("{}.scrollTo({left}, {top})", query(selector)))
            .await?;
        Ok(())
    }
    /// Sets one inline style property of the first match.
    async fn set_style(&mut self, selector: &str, property: &str, value: &str) -> Result<()> {
        self.evaluate(&format!(
            "{}.style.setProperty({}, {})",
            query(selector),
            serde_json::to_string(property)?,
            serde_json::to_string(value)?
        ))
        .await?;
        Ok(())
    }
    /// Every match's bounding rect, in document order.
    async fn rects(&mut self, selector: &str) -> Result<Vec<Rect>> {
        let value = self
            .evaluate(&format!(
                "[...document.querySelectorAll({})].map(e => {{ const r = e.getBoundingClientRect(); \
                 return {{x: r.x, y: r.y, width: r.width, height: r.height}}; }})",
                serde_json::to_string(selector)?
            ))
            .await?;
        Ok(serde_json::from_value(value)?)
    }
    /// Every match's `name` attribute (empty where unset), in document order.
    async fn attrs(&mut self, selector: &str, name: &str) -> Result<Vec<String>> {
        let value = self
            .evaluate(&format!(
                "[...document.querySelectorAll({})].map(e => e.getAttribute({}) ?? '')",
                serde_json::to_string(selector)?,
                serde_json::to_string(name)?
            ))
            .await?;
        Ok(serde_json::from_value(value)?)
    }
    /// A touch held still at the first match's centre for `ms`, then lifted.
    async fn long_press(&mut self, selector: &str, ms: u64) -> Result<()> {
        let _ = (selector, ms);
        bail!("{:?}: no touch input", self.platform())
    }
    /// A touch pressed at viewport point `(x, y)`, moved by `(dx, dy)` in steps, lifted.
    async fn swipe_from(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        let _ = (x, y, dx, dy);
        bail!("{:?}: no touch input", self.platform())
    }
    /// A touch pressed at the first match's centre and held until [`Driver::touch_up`].
    async fn touch_down(&mut self, selector: &str) -> Result<()> {
        let _ = selector;
        bail!("{:?}: no held touch", self.platform())
    }
    /// Lifts the held [`Driver::touch_down`] at the first match's centre.
    async fn touch_up(&mut self, selector: &str) -> Result<()> {
        let _ = selector;
        bail!("{:?}: no held touch", self.platform())
    }
    /// Two touches either side of the first match's centre, `from` px apart,
    /// spread to `to` px apart, then lifted.
    async fn pinch(&mut self, selector: &str, from: f64, to: f64) -> Result<()> {
        let _ = (selector, from, to);
        bail!("{:?}: no multi-touch input", self.platform())
    }
    async fn focus(&mut self, selector: &str) -> Result<()>;
    async fn text(&mut self, selector: &str) -> Result<String>;
    /// The first match's live `value`, which its attribute does not follow.
    async fn value(&mut self, selector: &str) -> Result<String> {
        let _ = selector;
        bail!("{:?}: no live value read", self.platform())
    }
    async fn attr(&mut self, selector: &str, name: &str) -> Result<Option<String>>;
    async fn exists(&mut self, selector: &str) -> Result<bool>;
    /// The first match's bounding rect.
    async fn rect(&mut self, selector: &str) -> Result<Rect>;
    /// The first match's computed `property`, as `getComputedStyle` reads it.
    async fn style(&mut self, selector: &str, property: &str) -> Result<String>;
    /// Focus is on any match of `selector`.
    async fn is_focused(&mut self, selector: &str) -> Result<bool>;
    /// The focused element's `id`, empty when it has none.
    async fn focused_id(&mut self) -> Result<String>;
    /// What focus is on, for a failure message.
    async fn focus_owner(&mut self) -> Result<String>;
    /// Presses `trigger`, then answers the file chooser it opens with `files`,
    /// each a name and its text.
    async fn choose_files(&mut self, trigger: &str, files: &[(&str, &str)]) -> Result<()> {
        let _ = (trigger, files);
        bail!("{:?}: no file chooser to answer", self.platform())
    }
    /// Presses `trigger`, then allows the system permission dialog it opens.
    async fn allow_permission(&mut self, trigger: &str) -> Result<()> {
        let _ = trigger;
        bail!("{:?}: no permission dialog to answer", self.platform())
    }
    /// The app's posted system notifications, one record per line; only Android reads them.
    async fn posted_notifications(&mut self) -> Result<String> {
        bail!("{:?}: no system notifications to read", self.platform())
    }
    /// Opens the notification shade and taps the one titled `title`.
    async fn tap_notification(&mut self, title: &str) -> Result<()> {
        let _ = title;
        bail!("{:?}: no notification shade", self.platform())
    }
    /// Opens the notification shade and presses the action button labelled `label`.
    async fn tap_notification_action(&mut self, label: &str) -> Result<()> {
        let _ = label;
        bail!("{:?}: no notification shade", self.platform())
    }
    /// A finger drag from a viewport point by `(dx, dy)`, for a press beside any element.
    async fn drag_at(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        let _ = (x, y, dx, dy);
        bail!("{:?}: no drag from a point", self.platform())
    }
    /// The system Back key, which only Android has.
    async fn press_back(&mut self) -> Result<()> {
        bail!("{:?}: no Back key", self.platform())
    }
    /// Whether a soft keyboard is up; `false` where there is none.
    async fn soft_keyboard_shown(&mut self) -> Result<bool> {
        Ok(false)
    }
    /// Resizes the viewport to `(width, height)` CSS px, as a soft keyboard shrinks it.
    async fn resize_viewport(&mut self, width: f64, height: f64) -> Result<()> {
        let _ = (width, height);
        bail!("{:?}: no viewport resize", self.platform())
    }
    /// Holds the app's timers of `delays` ([`crate::clock`]; a WebView's in the app, 2144);
    /// `false` where none can be held (Blitz), so the caller waits them out instead.
    async fn hold_timers(&mut self, delays: &[u32]) -> Result<bool> {
        let _ = delays;
        Ok(false)
    }
    /// Pending held timers of `ms`.
    async fn armed(&mut self, ms: u32) -> Result<usize> {
        let _ = ms;
        bail!("{:?}: no held clock", self.platform())
    }
    /// Fires every pending held timer of `ms`; returns how many. A WebView's run on the
    /// app's next poll: wait for their effect, never read it straight after.
    async fn fire_timers(&mut self, ms: u32) -> Result<usize> {
        let _ = ms;
        bail!("{:?}: no held clock", self.platform())
    }
    /// Returns once the page has handled what the last action queued: on the web two
    /// macrotask turns, past dioxus's microtask-driven render; elsewhere a few idle rounds.
    async fn settle(&mut self) -> Result<()> {
        for _ in 0..4 {
            self.idle().await;
        }
        Ok(())
    }
    /// Waits until the page drew a frame and settled: what a frame delivers (an observer's
    /// report, a scroll event) has landed.
    async fn frame(&mut self) -> Result<()>;
    /// Lets time pass: timers fire and the page settles.
    async fn idle(&mut self);
    /// How long [`eventually`] polls.
    fn budget(&self) -> Duration;
}

/// Polls `probe` until it holds, or fails naming `what`.
pub async fn eventually<D: Driver>(
    d: &mut D,
    what: &str,
    mut probe: impl AsyncFnMut(&mut D) -> Result<bool>,
) -> Result<()> {
    let started = Instant::now();
    loop {
        if probe(d).await? {
            return Ok(());
        }
        if started.elapsed() > d.budget() {
            bail!("{:?}: gave up waiting for {what}", d.platform());
        }
        d.idle().await;
    }
}

/// Lets `rounds` idle steps pass, for a check that something did *not* happen.
pub async fn linger<D: Driver>(d: &mut D, rounds: usize) {
    for _ in 0..rounds {
        d.idle().await;
    }
}

/// Focus lands on `selector`, or fails naming what holds it.
pub async fn eventually_focused<D: Driver>(d: &mut D, selector: &str, during: &str) -> Result<()> {
    let settled = eventually(
        d,
        &format!("focus on {selector} after {during}"),
        async |d| d.is_focused(selector).await,
    )
    .await;
    if settled.is_err() {
        bail!(
            "{:?}: after {during}, focus is on {}, not {selector}",
            d.platform(),
            d.focus_owner().await?
        );
    }
    Ok(())
}

/// `selector`'s text reads `expected`, or fails naming what it reads. No match yet
/// is not yet: an Android tap renders after it returns.
pub async fn eventually_text<D: Driver>(
    d: &mut D,
    selector: &str,
    expected: &str,
    during: &str,
) -> Result<()> {
    let settled = eventually(
        d,
        &format!("{selector} to read {expected:?} after {during}"),
        async |d| Ok(d.exists(selector).await? && d.text(selector).await? == expected),
    )
    .await;
    if settled.is_err() {
        if !d.exists(selector).await? {
            bail!(
                "{:?}: after {during}, no element matches {selector}",
                d.platform()
            );
        }
        bail!(
            "{:?}: after {during}, {selector} reads {:?}, not {expected:?}",
            d.platform(),
            d.text(selector).await?
        );
    }
    Ok(())
}

/// Runs `$body(&mut driver, $route)` as `$name::web`, with the `native`
/// feature as `$name::native`, with `android` as `$name::android`, with
/// `desktop` as `$name::desktop`. All but native also fail on a console error.
///
/// A backend gap is a named skip, listed by `--ignored`, never silent:
/// `scenario!(name, "/route", body, native: skip("Blitz has no <summary> stop"));`
/// `android: skip("958 ...")`, `desktop: skip("1126 ...")` and `web: skip(..)` do the same
/// for Android, the desktop WebView and Chromium; the skips may follow each other in any
/// order. `android_only("1275 ...")` skips every arm but Android's.
#[macro_export]
macro_rules! scenario {
    (@skip $name:ident, $route:expr, $body:ident, $w:tt $n:tt $a:tt $d:tt; web: skip($reason:literal) $(, $($rest:tt)*)?) => {
        $crate::scenario!(@skip $name, $route, $body, [#[test] #[ignore = $reason]] $n $a $d; $($($rest)*)?);
    };
    (@skip $name:ident, $route:expr, $body:ident, $w:tt $n:tt $a:tt $d:tt; native: skip($reason:literal) $(, $($rest:tt)*)?) => {
        $crate::scenario!(@skip $name, $route, $body, $w [#[test] #[ignore = $reason]] $a $d; $($($rest)*)?);
    };
    (@skip $name:ident, $route:expr, $body:ident, $w:tt $n:tt $a:tt $d:tt; android: skip($reason:literal) $(, $($rest:tt)*)?) => {
        $crate::scenario!(@skip $name, $route, $body, $w $n [#[test] #[ignore = $reason]] $d; $($($rest)*)?);
    };
    (@skip $name:ident, $route:expr, $body:ident, $w:tt $n:tt $a:tt $d:tt; desktop: skip($reason:literal) $(, $($rest:tt)*)?) => {
        $crate::scenario!(@skip $name, $route, $body, $w $n $a [#[test] #[ignore = $reason]]; $($($rest)*)?);
    };
    (@skip $name:ident, $route:expr, $body:ident, $w:tt $n:tt $a:tt $d:tt; android_only($reason:literal) $(, $($rest:tt)*)?) => {
        $crate::scenario!(@skip $name, $route, $body, [#[test] #[ignore = $reason]] [#[test] #[ignore = $reason]] $a [#[test] #[ignore = $reason]]; $($($rest)*)?);
    };
    (@skip $name:ident, $route:expr, $body:ident, $w:tt $n:tt $a:tt $d:tt;) => {
        $crate::scenario!(@all $name, $route, $body, $w, $n, $a, $d);
    };
    (@all $name:ident, $route:expr, $body:ident, [$(#[$web:meta])*], [$(#[$native:meta])*], [$(#[$android:meta])*], [$(#[$desktop:meta])*]) => {
        mod $name {
            $(#[$web])*
            fn web() {
                $crate::browser::block_on(async {
                    let mut driver = $crate::driver::Web::open($route).await.unwrap();
                    super::$body(&mut driver, $route).await.unwrap();
                    driver.finish(stringify!($name)).await.unwrap();
                });
            }

            #[cfg(feature = "native")]
            $(#[$native])*
            fn native() {
                let mut driver = $crate::driver::Native::open($route);
                $crate::futures::executor::block_on(super::$body(&mut driver, $route)).unwrap();
            }

            #[cfg(feature = "android")]
            $(#[$android])*
            fn android() {
                $crate::android::block_on(async {
                    let mut driver = $crate::driver::Android::open($route).await.unwrap();
                    super::$body(&mut driver, $route).await.unwrap();
                    driver.finish(stringify!($name)).await.unwrap();
                });
            }

            #[cfg(feature = "desktop")]
            $(#[$desktop])*
            fn desktop() {
                let mut driver = $crate::driver::Desktop::open(module_path!(), $route).unwrap();
                $crate::futures::executor::block_on(super::$body(&mut driver, $route)).unwrap();
                driver.finish(stringify!($name)).unwrap();
            }
        }
    };
    ($name:ident, $route:expr, $body:ident $(, $($skips:tt)*)?) => {
        $crate::scenario!(@skip $name, $route, $body, [#[test]] [#[test]] [#[test]] [#[test]]; $($($skips)*)?);
    };
}

pub use web::Web;

mod web;

#[cfg(feature = "native")]
pub use native::Native;

#[cfg(feature = "native")]
mod native;

#[cfg(feature = "android")]
pub use android::Android;

#[cfg(feature = "android")]
mod android;

#[cfg(feature = "desktop")]
pub use desktop::Desktop;

#[cfg(feature = "desktop")]
pub mod desktop;
