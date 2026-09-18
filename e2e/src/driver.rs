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

/// What a scenario may do and read. Every read is of the settled page on
/// Blitz; on the web it is a snapshot, so assert through [`eventually`].
// Driven by one thread's `block_on`, so the futures need no `Send` bound.
#[allow(async_fn_in_trait)]
pub trait Driver {
    fn platform(&self) -> Platform;
    async fn click(&mut self, selector: &str) -> Result<()>;
    async fn press(&mut self, key: Key) -> Result<()>;
    async fn type_text(&mut self, text: &str) -> Result<()>;
    async fn focus(&mut self, selector: &str) -> Result<()>;
    async fn text(&mut self, selector: &str) -> Result<String>;
    async fn attr(&mut self, selector: &str, name: &str) -> Result<Option<String>>;
    async fn is_focused(&mut self, selector: &str) -> Result<bool>;
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

/// Runs `$body(&mut driver, $route)` as `$name::web` and, with the `native`
/// feature, `$name::native`. The web run also fails on a console error.
#[macro_export]
macro_rules! scenario {
    ($name:ident, $route:expr, $body:ident) => {
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
            #[test]
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

    use super::{Driver, Platform};
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

        async fn press(&mut self, key: keyboard::Key) -> Result<()> {
            keyboard::press(&self.fixture.page, key).await
        }

        async fn type_text(&mut self, text: &str) -> Result<()> {
            keyboard::type_text(&self.fixture.page, text).await
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

        async fn is_focused(&mut self, selector: &str) -> Result<bool> {
            self.json(&format!("document.activeElement === {}", element(selector)))
                .await
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

    use super::{Driver, Platform};
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

    impl Driver for Native {
        fn platform(&self) -> Platform {
            Platform::Native
        }

        async fn click(&mut self, selector: &str) -> Result<()> {
            self.page.click(selector);
            Ok(())
        }

        async fn press(&mut self, key: keyboard::Key) -> Result<()> {
            let key = native_tests::Key::from_str(key.key)
                .map_err(|_| anyhow!("no dioxus key named {:?}", key.key))?;
            self.page.press(key);
            Ok(())
        }

        async fn type_text(&mut self, text: &str) -> Result<()> {
            for ch in text.chars() {
                self.page
                    .press(native_tests::Key::Character(ch.to_string()));
            }
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

        async fn is_focused(&mut self, selector: &str) -> Result<bool> {
            Ok(self.page.is_focused(selector))
        }

        async fn focus_owner(&mut self) -> Result<String> {
            Ok(self.page.focus_owner())
        }

        async fn idle(&mut self) {
            self.page.wait(Duration::from_millis(20));
        }

        fn budget(&self) -> Duration {
            Duration::from_secs(3)
        }
    }
}
