//! WebView2 is Chromium (2783): the app starts with a DevTools port through
//! `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, and input goes in over CDP's `Input` domain, as on
//! the web tier. Reads stay on the bridge. No window lookup, focus or screen size involved.

use std::net::TcpListener;
use std::process::Command;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chromiumoxide::cdp::browser_protocol::emulation::SetFocusEmulationEnabledParams;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use chromiumoxide::handler::HandlerConfig;
use chromiumoxide::{Browser, Page};
use futures::StreamExt;
use tokio::runtime::Runtime;

use super::{Desktop, LAUNCH};
use crate::passes::keyboard::{self, ALT, CTRL, Key, SHIFT};

/// Runs the CDP handler between calls: the scenario's own executor is not tokio.
static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("start the CDP runtime"));

fn block<T>(work: impl Future<Output = Result<T>>) -> Result<T> {
    RUNTIME.block_on(work)
}

/// A DevTools port and a fresh profile for the app about to start; the port.
pub(super) fn prepare(command: &mut Command, launch: &str) -> Result<u16> {
    // Released for the app to take: a free loopback port is enough on a CI VM.
    let port = TcpListener::bind("127.0.0.1:0")
        .context("find a DevTools port")?
        .local_addr()?
        .port();
    // Appended to wry's own arguments; the folder replaces the default profile.
    command
        .env(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            format!("--remote-debugging-port={port}"),
        )
        .env(
            "WEBVIEW2_USER_DATA_FOLDER",
            std::env::temp_dir().join(format!("e2e-webview2-{launch}")),
        );
    Ok(port)
}

/// The app's page over CDP.
pub(super) struct Cdp {
    page: Page,
    _browser: Browser,
}

impl Cdp {
    /// Called once the bridge answered, so the page is loaded.
    pub(super) fn connect(port: u16) -> Result<Self> {
        let url = format!("http://127.0.0.1:{port}");
        block(async move {
            let started = Instant::now();
            let (mut browser, mut handler) = loop {
                // No viewport: an override would lay the page out at another size than the window.
                let config = HandlerConfig {
                    viewport: None,
                    ignore_invalid_messages: true,
                    ..HandlerConfig::default()
                };
                match Browser::connect_with_config(&url, config).await {
                    Ok(connected) => break connected,
                    Err(error) if started.elapsed() > LAUNCH => {
                        return Err(error).with_context(|| {
                            format!(
                                "connect to WebView2's DevTools at {url}: is the runtime installed?"
                            )
                        });
                    }
                    Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
                }
            };
            tokio::spawn(async move { while handler.next().await.is_some() {} });
            browser.fetch_targets().await?;
            crate::wait::until("the WebView's page", || async {
                Ok(!browser.pages().await?.is_empty())
            })
            .await?;
            let page = browser.pages().await?.remove(0);
            // Focus and blur as in a foreground window, whatever the desktop's own focus.
            page.execute(SetFocusEmulationEnabledParams::new(true))
                .await?;
            Ok(Self {
                page,
                _browser: browser,
            })
        })
    }

    async fn mouse(
        &self,
        kind: DispatchMouseEventType,
        (x, y): (f64, f64),
        buttons: i64,
        clicks: i64,
    ) -> Result<()> {
        self.page
            .execute(
                DispatchMouseEventParams::builder()
                    .r#type(kind)
                    .x(x)
                    .y(y)
                    .button(MouseButton::Left)
                    .buttons(buttons)
                    .click_count(clicks)
                    .build()
                    .map_err(anyhow::Error::msg)?,
            )
            .await?;
        Ok(())
    }
}

/// The input verbs of the other backends, over CDP.
impl Desktop {
    fn cdp(&self) -> Result<&Cdp> {
        self.cdp
            .as_ref()
            .context("WebView2's DevTools are not connected")
    }

    /// No window to find: CDP addresses the page.
    pub(super) fn find_window(&mut self) -> Result<String> {
        Ok(String::new())
    }

    pub(super) fn focus_window(&self) -> Result<()> {
        Ok(())
    }

    /// Moves the pointer to the viewport's bottom-left corner; that point.
    pub(super) fn park_pointer(&mut self) -> Result<(f64, f64)> {
        let height: f64 = self.json("innerHeight")?;
        let at = (2.0, height - 2.0);
        self.move_to(at.0, at.1)?;
        Ok(at)
    }

    pub(super) fn move_to(&mut self, x: f64, y: f64) -> Result<()> {
        let cdp = self.cdp()?;
        block(cdp.mouse(DispatchMouseEventType::MouseMoved, (x, y), 0, 0))
    }

    /// A move onto the point, then `count` presses, the second a `dblclick`.
    pub(super) fn click_point(&mut self, x: f64, y: f64, count: u32) -> Result<()> {
        let cdp = self.cdp()?;
        block(async {
            cdp.mouse(DispatchMouseEventType::MouseMoved, (x, y), 0, 0)
                .await?;
            for clicks in 1..=i64::from(count) {
                cdp.mouse(DispatchMouseEventType::MousePressed, (x, y), 1, clicks)
                    .await?;
                cdp.mouse(DispatchMouseEventType::MouseReleased, (x, y), 0, clicks)
                    .await?;
            }
            Ok(())
        })
    }

    /// Press, eight moves a frame apart, release, as the X backend drags.
    pub(super) fn drag_by(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        let cdp = self.cdp()?;
        block(async {
            cdp.mouse(DispatchMouseEventType::MouseMoved, (x, y), 0, 0)
                .await?;
            cdp.mouse(DispatchMouseEventType::MousePressed, (x, y), 1, 1)
                .await?;
            for step in 1..=8 {
                let t = step as f64 / 8.0;
                let at = (x + dx * t, y + dy * t);
                cdp.mouse(DispatchMouseEventType::MouseMoved, at, 1, 0)
                    .await?;
                tokio::time::sleep(Duration::from_millis(16)).await;
            }
            cdp.mouse(
                DispatchMouseEventType::MouseReleased,
                (x + dx, y + dy),
                0,
                1,
            )
            .await
        })
    }

    /// One wheel event of exactly `dy` CSS px, not whole notches as under X.
    pub(super) fn wheel_at(&mut self, x: f64, y: f64, dy: f64) -> Result<()> {
        let wheel = DispatchMouseEventParams::builder()
            .r#type(DispatchMouseEventType::MouseWheel)
            .x(x)
            .y(y)
            .delta_x(0.0)
            .delta_y(dy)
            .build()
            .map_err(anyhow::Error::msg)?;
        let cdp = self.cdp()?;
        block(async {
            cdp.mouse(DispatchMouseEventType::MouseMoved, (x, y), 0, 0)
                .await?;
            cdp.page.execute(wheel).await?;
            Ok(())
        })
    }

    /// `key` with `mods` among `shift`, `ctrl` and `alt` held.
    pub(super) fn chord(&mut self, mods: &[&str], key: Key) -> Result<()> {
        let modifiers = mods
            .iter()
            .map(|name| match *name {
                "shift" => SHIFT,
                "ctrl" => CTRL,
                "alt" => ALT,
                _ => 0,
            })
            .fold(0, |all, bit| all | bit);
        let cdp = self.cdp()?;
        block(keyboard::press_with(&cdp.page, key, modifiers))
    }

    /// One key per character, `ms` apart.
    pub(super) fn type_keys(&mut self, text: &str, ms: u64) -> Result<()> {
        let cdp = self.cdp()?;
        block(async {
            for ch in text.chars() {
                keyboard::type_text(&cdp.page, &ch.to_string()).await?;
                tokio::time::sleep(Duration::from_millis(ms)).await;
            }
            Ok(())
        })
    }
}
