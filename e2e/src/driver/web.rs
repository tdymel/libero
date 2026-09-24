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

    async fn json<T: serde::de::DeserializeOwned>(&self, expression: &str) -> Result<T> {
        json(&self.fixture.page, expression).await
    }
}

/// A JS expression's value, through JSON: a bare `null` does not
/// deserialise into an `Option`.
pub(super) async fn json<T: serde::de::DeserializeOwned>(
    page: &chromiumoxide::Page,
    expression: &str,
) -> Result<T> {
    let json: String = page
        .evaluate(format!("JSON.stringify({expression})"))
        .await?
        .into_value()?;
    Ok(serde_json::from_str(&json)?)
}

pub(super) fn element(selector: &str) -> String {
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

    async fn press_ctrl(&mut self, key: keyboard::Key) -> Result<()> {
        keyboard::press_with(&self.fixture.page, key, keyboard::CTRL).await
    }

    async fn type_text(&mut self, text: &str) -> Result<()> {
        keyboard::type_text(&self.fixture.page, text).await
    }

    async fn insert_text(&mut self, text: &str) -> Result<()> {
        keyboard::insert_text(&self.fixture.page, text).await
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

    async fn long_press(&mut self, selector: &str, ms: u64) -> Result<()> {
        pointer::long_press(&self.fixture.page, selector, ms).await
    }

    async fn scroll_by(&mut self, dy: f64) -> Result<()> {
        // Instant: a smooth scroll restarts on every poll and never arrives.
        let scroll = format!("(window.scrollBy({{ top: {dy}, behavior: 'instant' }}), true)");
        let _: bool = self.json(&scroll).await?;
        Ok(())
    }

    async fn pinch(&mut self, selector: &str, from: f64, to: f64) -> Result<()> {
        let at = pointer::centre_of(&self.fixture.page, selector).await?;
        pointer::pinch(&self.fixture.page, at, from, to, 8).await
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

    async fn value(&mut self, selector: &str) -> Result<String> {
        self.json(&format!("{}.value", element(selector))).await
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

    /// CDP intercepts the chooser and names the input it opened for.
    async fn choose_files(&mut self, trigger: &str, files: &[(&str, &str)]) -> Result<()> {
        use anyhow::Context;
        use chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams;
        use chromiumoxide::cdp::browser_protocol::page::{
            EventFileChooserOpened, SetInterceptFileChooserDialogParams,
        };
        use futures::StreamExt;

        let dir = std::env::temp_dir().join(format!("e2e-chooser-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let mut paths = Vec::new();
        for (name, text) in files {
            let path = dir.join(name);
            std::fs::write(&path, text)?;
            paths.push(path.to_string_lossy().into_owned());
        }
        let page = &self.fixture.page;
        let mut opened = page.event_listener::<EventFileChooserOpened>().await?;
        page.execute(SetInterceptFileChooserDialogParams::new(true))
            .await?;
        pointer::click(page, trigger).await?;
        let event = tokio::time::timeout(crate::wait::timeout(), opened.next())
            .await
            .context("no file chooser opened")?
            .context("the chooser events ended")?;
        let input = event
            .backend_node_id
            .context("the chooser names no input")?;
        page.execute(
            SetFileInputFilesParams::builder()
                .files(paths)
                .backend_node_id(input)
                .build()
                .map_err(anyhow::Error::msg)?,
        )
        .await?;
        page.execute(SetInterceptFileChooserDialogParams::new(false))
            .await?;
        Ok(())
    }
}
