//! One scenario, several platforms (todo 822): a scenario is an `async fn`
//! over any [`Driver`], and [`scenario!`](crate::scenario) runs it on the web
//! backend (Chromium over CDP) and, with the `native` feature, on Blitz.
//!
//! ```ignore
//! async fn pages<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
//!     d.click("[aria-label=\"Go to page 3\"]").await?;
//!     eventually(d, "page 3", async |d| Ok(d.text("[aria-current=page]").await? == "3")).await
//! }
//! e2e::scenario!(paging, "/pagination", pages);
//! ```

use std::time::{Duration, Instant};

use anyhow::{Result, bail};

use crate::passes::keyboard::Key;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Web,
    Native,
}

/// A `getBoundingClientRect()` in CSS px.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
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
    async fn type_text(&mut self, text: &str) -> Result<()>;
    /// Presses at the first match's centre, moves by `(dx, dy)` in steps, releases.
    async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()>;
    async fn focus(&mut self, selector: &str) -> Result<()>;
    async fn text(&mut self, selector: &str) -> Result<String>;
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

/// `selector`'s text reads `expected`, or fails naming what it reads.
pub async fn eventually_text<D: Driver>(
    d: &mut D,
    selector: &str,
    expected: &str,
    during: &str,
) -> Result<()> {
    let settled = eventually(
        d,
        &format!("{selector} to read {expected:?} after {during}"),
        async |d| Ok(d.text(selector).await? == expected),
    )
    .await;
    if settled.is_err() {
        bail!(
            "{:?}: after {during}, {selector} reads {:?}, not {expected:?}",
            d.platform(),
            d.text(selector).await?
        );
    }
    Ok(())
}

/// Runs `$body(&mut driver, $route)` as `$name::web` and, with the `native`
/// feature, `$name::native`. The web run also fails on a console error.
///
/// A backend gap is a named skip, listed by `--ignored`, never silent:
/// `scenario!(name, "/route", body, native: skip("Blitz has no <summary> stop"));`
#[macro_export]
macro_rules! scenario {
    ($name:ident, $route:expr, $body:ident) => {
        $crate::scenario!(@web $name, $route, $body, #[test]);
    };
    ($name:ident, $route:expr, $body:ident, native: skip($reason:literal)) => {
        $crate::scenario!(@web $name, $route, $body, #[test] #[ignore = $reason]);
    };
    (@web $name:ident, $route:expr, $body:ident, $(#[$native:meta])*) => {
        mod $name {
            #[test]
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
        }
    };
}

pub use web::Web;

mod web {
    use std::time::Duration;

    use anyhow::Result;

    use super::{Driver, Platform, Rect};
    use crate::passes::{focus, keyboard, pointer};
    use crate::{Fixture, Viewport};

    pub struct Web {
        pub fixture: Fixture,
    }

    impl Web {
        pub async fn open(route: &str) -> Result<Self> {
            Ok(Self {
                fixture: Fixture::open(route, Viewport::Desktop).await?,
            })
        }

        /// The console stayed clean; closes the page.
        pub async fn finish(self, what: &str) -> Result<()> {
            self.fixture.console.assert_clean(what)?;
            self.fixture.close().await
        }

        /// A JS expression's value, through JSON: a bare `null` does not
        /// deserialise into an `Option`.
        async fn json<T: serde::de::DeserializeOwned>(&self, expression: &str) -> Result<T> {
            let json: String = self
                .fixture
                .page
                .evaluate(format!("JSON.stringify({expression})"))
                .await?
                .into_value()?;
            Ok(serde_json::from_str(&json)?)
        }
    }

    fn element(selector: &str) -> String {
        format!("document.querySelector({selector:?})")
    }

    impl Driver for Web {
        fn platform(&self) -> Platform {
            Platform::Web
        }

        async fn click(&mut self, selector: &str) -> Result<()> {
            pointer::click(&self.fixture.page, selector).await
        }

        async fn hover(&mut self, selector: &str) -> Result<()> {
            pointer::hover(&self.fixture.page, selector).await
        }

        async fn double_click(&mut self, selector: &str) -> Result<()> {
            pointer::double_click(&self.fixture.page, selector).await
        }

        async fn click_at(&mut self, x: f64, y: f64) -> Result<()> {
            pointer::click_at(&self.fixture.page, pointer::Point { x, y }).await
        }

        async fn viewport(&mut self) -> Result<(f64, f64)> {
            self.json("[innerWidth, innerHeight]").await
        }

        async fn press(&mut self, key: keyboard::Key) -> Result<()> {
            keyboard::press(&self.fixture.page, key).await
        }

        async fn press_shift(&mut self, key: keyboard::Key) -> Result<()> {
            keyboard::press_shift(&self.fixture.page, key).await
        }

        async fn type_text(&mut self, text: &str) -> Result<()> {
            keyboard::type_text(&self.fixture.page, text).await
        }

        async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()> {
            let page = &self.fixture.page;
            let from = pointer::centre_of(page, selector).await?;
            let to = pointer::Point {
                x: from.x + dx,
                y: from.y + dy,
            };
            pointer::drag(page, from, to, 8).await
        }

        async fn focus(&mut self, selector: &str) -> Result<()> {
            self.fixture
                .page
                .evaluate(format!("{}.focus()", element(selector)))
                .await?;
            Ok(())
        }

        async fn text(&mut self, selector: &str) -> Result<String> {
            self.json(&format!("{}.textContent", element(selector)))
                .await
        }

        async fn attr(&mut self, selector: &str, name: &str) -> Result<Option<String>> {
            self.json(&format!("{}.getAttribute({name:?})", element(selector)))
                .await
        }

        async fn exists(&mut self, selector: &str) -> Result<bool> {
            self.json(&format!("{} !== null", element(selector))).await
        }

        async fn rect(&mut self, selector: &str) -> Result<Rect> {
            self.json(&format!("{}.getBoundingClientRect()", element(selector)))
                .await
        }

        async fn style(&mut self, selector: &str, property: &str) -> Result<String> {
            self.json(&format!(
                "getComputedStyle({}).getPropertyValue({property:?})",
                element(selector)
            ))
            .await
        }

        async fn is_focused(&mut self, selector: &str) -> Result<bool> {
            // Any match, as Blitz's reads it.
            self.json(&format!(
                "document.activeElement?.matches({selector:?}) ?? false"
            ))
            .await
        }

        async fn focused_id(&mut self) -> Result<String> {
            self.json("document.activeElement?.id ?? ''").await
        }

        async fn focus_owner(&mut self) -> Result<String> {
            Ok(format!(
                "{:?}",
                focus::active_element(&self.fixture.page).await?
            ))
        }

        async fn idle(&mut self) {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }

        fn budget(&self) -> Duration {
            crate::wait::timeout()
        }
    }
}

#[cfg(feature = "native")]
pub use native::Native;

#[cfg(feature = "native")]
mod native {
    use std::{str::FromStr, time::Duration};

    use anyhow::{Result, anyhow};

    use super::{Driver, Platform, Rect};
    use crate::passes::keyboard;

    /// A fixture route mounted in a windowless Blitz document.
    pub struct Native {
        pub page: native_tests::Page,
    }

    impl Native {
        pub fn open(route: &str) -> Self {
            let page =
                e2e_fixtures::route(route).unwrap_or_else(|| panic!("no fixture at {route}"));
            Self {
                page: native_tests::mount(page),
            }
        }
    }

    fn native_key(key: keyboard::Key) -> Result<native_tests::Key> {
        native_tests::Key::from_str(key.key)
            .map_err(|_| anyhow!("no dioxus key named {:?}", key.key))
    }

    impl Driver for Native {
        fn platform(&self) -> Platform {
            Platform::Native
        }

        async fn click(&mut self, selector: &str) -> Result<()> {
            self.page.click(selector);
            Ok(())
        }

        async fn hover(&mut self, selector: &str) -> Result<()> {
            self.page.hover(selector);
            Ok(())
        }

        async fn double_click(&mut self, selector: &str) -> Result<()> {
            self.page.click(selector);
            self.page.click(selector);
            Ok(())
        }

        async fn click_at(&mut self, x: f64, y: f64) -> Result<()> {
            self.page.click_at(x as f32, y as f32);
            Ok(())
        }

        async fn viewport(&mut self) -> Result<(f64, f64)> {
            let (width, height) = native_tests::VIEWPORT;
            Ok((f64::from(width), f64::from(height)))
        }

        async fn press(&mut self, key: keyboard::Key) -> Result<()> {
            let key = native_key(key)?;
            self.page.press(key);
            Ok(())
        }

        async fn press_shift(&mut self, key: keyboard::Key) -> Result<()> {
            let key = native_key(key)?;
            self.page.press_with(key, native_tests::Modifiers::SHIFT);
            Ok(())
        }

        async fn type_text(&mut self, text: &str) -> Result<()> {
            for ch in text.chars() {
                self.page
                    .press(native_tests::Key::Character(ch.to_string()));
            }
            Ok(())
        }

        async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()> {
            self.page.drag(selector, dx as f32, dy as f32);
            Ok(())
        }

        async fn focus(&mut self, selector: &str) -> Result<()> {
            self.page.focus(selector);
            Ok(())
        }

        async fn text(&mut self, selector: &str) -> Result<String> {
            Ok(self.page.text(selector))
        }

        async fn attr(&mut self, selector: &str, name: &str) -> Result<Option<String>> {
            Ok(self.page.attr(selector, name))
        }

        async fn exists(&mut self, selector: &str) -> Result<bool> {
            Ok(self.page.exists(selector))
        }

        async fn rect(&mut self, selector: &str) -> Result<Rect> {
            let (x, y, width, height) = self.page.rect(selector);
            Ok(Rect {
                x,
                y,
                width,
                height,
            })
        }

        async fn style(&mut self, selector: &str, property: &str) -> Result<String> {
            Ok(self.page.computed(selector, property))
        }

        async fn is_focused(&mut self, selector: &str) -> Result<bool> {
            Ok(self.page.is_focused(selector))
        }

        async fn focused_id(&mut self) -> Result<String> {
            let page = &self.page;
            Ok(page
                .focused()
                .and_then(|node| page.attr_of(node, "id"))
                .unwrap_or_default())
        }

        async fn focus_owner(&mut self) -> Result<String> {
            Ok(self.page.focus_owner())
        }

        /// Real time for libero's timers, the same span on the animation clock.
        async fn idle(&mut self) {
            self.page.wait(Duration::from_millis(20));
            self.page.advance(0.02);
        }

        fn budget(&self) -> Duration {
            Duration::from_secs(3)
        }
    }
}
