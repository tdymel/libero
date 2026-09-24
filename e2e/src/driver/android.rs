use std::time::Duration;

use anyhow::Result;
use chromiumoxide::Page;

use super::web::{element, json};
use super::{Driver, Platform, Rect};
use crate::android::{
    CTRL, SHIFT, download, harness, input, input_text, keycode, soft_keyboard_shown, tap_node,
    webview_view_focused,
};
use crate::passes::{focus, keyboard, pointer};

/// A fixture route in the emulator's WebView, reached through the app's
/// `window.__route` hook; each open remounts the fixture.
pub struct Android {
    page: Page,
}

impl Android {
    pub async fn open(route: &str) -> Result<Self> {
        let harness = harness();
        let page = harness.page.clone();
        // Launched in touch mode, the WebView lacks view focus and eats the first key;
        // take it before the fixture mounts, as gaining it moves DOM focus (1052).
        if !webview_view_focused().await? {
            input(&["keyevent".into(), keycode("ArrowUp")?.to_string()]).await?;
        }
        page.evaluate("document.activeElement?.blur(); scrollTo(0, 0)")
            .await?;
        harness.console.drain();
        let generation: u64 = json(&page, &format!("window.__route({route:?})")).await?;
        crate::wait::for_selector_kind(
            "fixture-ready",
            &page,
            &format!("[data-fixture-generation=\"{generation}\"]"),
        )
        .await?;
        Ok(Self { page })
    }

    /// The WebView's page, for CDP calls the driver has no verb for.
    pub fn page(&self) -> &Page {
        &self.page
    }

    /// The console stayed clean since [`Android::open`].
    pub async fn finish(self, what: &str) -> Result<()> {
        harness().console.assert_clean(what)
    }

    /// A CSS-px viewport point as the device px `input tap` takes.
    fn device(&self, at: pointer::Point) -> [String; 2] {
        let harness = harness();
        [
            (at.x * harness.scale + harness.origin.0)
                .round()
                .to_string(),
            (at.y * harness.scale + harness.origin.1)
                .round()
                .to_string(),
        ]
    }

    /// The first match's centre, scrolled into view first: the screen is
    /// narrower than the desktop viewport the fixtures are written for.
    async fn centre(&self, selector: &str) -> Result<pointer::Point> {
        // A popover mounts hidden at 0,0 a round trip before it is placed (1002).
        let hidden = format!(
            "(e => !!e && getComputedStyle(e).visibility === 'hidden')({})",
            element(selector)
        );
        for _ in 0..40 {
            if !json::<bool>(&self.page, &hidden).await? {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        self.page
            .evaluate(format!(
                "{}?.scrollIntoView({{ block: 'center', inline: 'center' }})",
                element(selector)
            ))
            .await?;
        // A fling from the last swipe keeps the page moving under a press (1104).
        let mut at = pointer::centre_of(&self.page, selector).await?;
        for _ in 0..40 {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let next = pointer::centre_of(&self.page, selector).await?;
            let settled = (next.x - at.x).abs() < 0.5 && (next.y - at.y).abs() < 0.5;
            at = next;
            if settled {
                break;
            }
        }
        Ok(at)
    }

    async fn swipe(&mut self, from: pointer::Point, dx: f64, dy: f64) -> Result<()> {
        // `input swipe` lifts 1-2px short of its last move, and a drag follows moves
        // only (1002): step the touch by hand, its last move on the end point.
        // A touch lifted off the screen leaves the WebView a stuck touch that eats later drags.
        let (vw, vh) = self.viewport().await?;
        let mut events = vec!["DOWN".to_string()];
        events.extend(std::iter::repeat_n("MOVE".to_string(), 4));
        events.push("UP".into());
        let mut chain = Vec::new();
        for (i, event) in events.into_iter().enumerate() {
            let t = (i as f64 / 4.0).min(1.0);
            let [x, y] = self.device(pointer::Point {
                x: (from.x + dx * t).clamp(0.0, vw - 1.0),
                y: (from.y + dy * t).clamp(0.0, vh - 1.0),
            });
            if i > 0 {
                chain.extend([";".into(), "input".into()]);
            }
            chain.extend(["motionevent".into(), event, x, y]);
        }
        input(&chain).await
    }

    async fn tap(&self, at: pointer::Point) -> Result<()> {
        let [x, y] = self.device(at);
        input(&["tap".into(), x, y]).await
    }

    /// In touch mode (after an earlier tap opened the soft keyboard) the WebView
    /// holds no view focus, and Android eats the first key to leave touch mode (998).
    async fn leave_touch_mode(&self) -> Result<()> {
        if !json::<bool>(&self.page, "document.hasFocus()").await? {
            input(&["keyevent".into(), keycode("ArrowUp")?.to_string()]).await?;
        }
        Ok(())
    }

    async fn chord(&self, modifier: u32, key: keyboard::Key) -> Result<()> {
        self.leave_touch_mode().await?;
        let code = keycode(key.key)?;
        input(&[
            "keycombination".into(),
            modifier.to_string(),
            code.to_string(),
        ])
        .await
    }
}

impl Driver for Android {
    fn platform(&self) -> Platform {
        Platform::Android
    }

    async fn click(&mut self, selector: &str) -> Result<()> {
        let at = self.centre(selector).await?;
        self.tap(at).await
    }

    /// No hover on a touch screen: CDP's mouse move instead.
    async fn hover(&mut self, selector: &str) -> Result<()> {
        pointer::hover(&self.page, selector).await
    }

    /// CDP's, since two `input tap` processes can miss the double-tap window.
    async fn double_click(&mut self, selector: &str) -> Result<()> {
        self.centre(selector).await?;
        pointer::double_click(&self.page, selector).await
    }

    async fn click_at(&mut self, x: f64, y: f64) -> Result<()> {
        self.tap(pointer::Point { x, y }).await
    }

    async fn viewport(&mut self) -> Result<(f64, f64)> {
        json(&self.page, "[innerWidth, innerHeight]").await
    }

    /// A hardware key's: an Escape the soft keyboard would eat goes twice.
    async fn press(&mut self, key: keyboard::Key) -> Result<()> {
        let code = keycode(key.key)?.to_string();
        self.leave_touch_mode().await?;
        if key.key == "Escape" && soft_keyboard_shown().await? {
            input(&["keyevent".into(), code.clone()]).await?;
        }
        input(&["keyevent".into(), code]).await
    }

    async fn press_shift(&mut self, key: keyboard::Key) -> Result<()> {
        self.chord(SHIFT, key).await
    }

    async fn press_ctrl(&mut self, key: keyboard::Key) -> Result<()> {
        self.chord(CTRL, key).await
    }

    /// One `input text` per character, at about a typist's pace: one call
    /// for the lot outran PinField's move to the next cell.
    async fn type_text(&mut self, text: &str) -> Result<()> {
        for ch in text.chars() {
            input(&["text".into(), input_text(&ch.to_string())]).await?;
        }
        Ok(())
    }

    /// CDP's key events: each `adb input` is a process, far slower than `ms`.
    async fn type_burst(&mut self, text: &str, ms: u64) -> Result<()> {
        for ch in text.chars() {
            keyboard::type_text(&self.page, &ch.to_string()).await?;
            tokio::time::sleep(Duration::from_millis(ms)).await;
        }
        Ok(())
    }

    /// CDP's: `adb` has no IME, and `input text` names a key per character.
    async fn insert_text(&mut self, text: &str) -> Result<()> {
        keyboard::insert_text(&self.page, text).await
    }

    async fn drag(&mut self, selector: &str, dx: f64, dy: f64) -> Result<()> {
        let from = self.centre(selector).await?;
        self.swipe(from, dx, dy).await
    }

    async fn scroll_by(&mut self, dy: f64) -> Result<()> {
        let (vw, vh) = self.viewport().await?;
        let from = pointer::Point {
            x: vw / 2.0,
            y: vh / 2.0 + dy / 2.0,
        };
        self.swipe(from, 0.0, -dy).await
    }

    /// A swipe that goes nowhere.
    async fn long_press(&mut self, selector: &str, ms: u64) -> Result<()> {
        let [x, y] = self.device(self.centre(selector).await?);
        input(&["swipe".into(), x.clone(), y.clone(), x, y, ms.to_string()]).await
    }

    /// CDP's: `adb input` has one finger.
    async fn pinch(&mut self, selector: &str, from: f64, to: f64) -> Result<()> {
        let at = self.centre(selector).await?;
        pointer::pinch(&self.page, at, from, to, 8).await
    }

    async fn focus(&mut self, selector: &str) -> Result<()> {
        self.page
            .evaluate(format!("{}.focus()", element(selector)))
            .await?;
        Ok(())
    }

    async fn text(&mut self, selector: &str) -> Result<String> {
        json(&self.page, &format!("{}.textContent", element(selector))).await
    }

    async fn value(&mut self, selector: &str) -> Result<String> {
        json(&self.page, &format!("{}.value", element(selector))).await
    }

    async fn attr(&mut self, selector: &str, name: &str) -> Result<Option<String>> {
        json(
            &self.page,
            &format!("{}.getAttribute({name:?})", element(selector)),
        )
        .await
    }

    async fn exists(&mut self, selector: &str) -> Result<bool> {
        json(&self.page, &format!("{} !== null", element(selector))).await
    }

    async fn rect(&mut self, selector: &str) -> Result<Rect> {
        json(
            &self.page,
            &format!("{}.getBoundingClientRect()", element(selector)),
        )
        .await
    }

    async fn style(&mut self, selector: &str, property: &str) -> Result<String> {
        json(
            &self.page,
            &format!(
                "getComputedStyle({}).getPropertyValue({property:?})",
                element(selector)
            ),
        )
        .await
    }

    async fn is_focused(&mut self, selector: &str) -> Result<bool> {
        json(
            &self.page,
            &format!("document.activeElement?.matches({selector:?}) ?? false"),
        )
        .await
    }

    async fn focused_id(&mut self) -> Result<String> {
        json(&self.page, "document.activeElement?.id ?? ''").await
    }

    async fn focus_owner(&mut self) -> Result<String> {
        Ok(format!("{:?}", focus::active_element(&self.page).await?))
    }

    async fn soft_keyboard_shown(&mut self) -> Result<bool> {
        soft_keyboard_shown().await
    }

    /// The files go to Downloads, and the system picker's row is tapped
    /// there: one file, since a tap on a row picks it and closes the picker.
    async fn choose_files(&mut self, trigger: &str, files: &[(&str, &str)]) -> Result<()> {
        let [(name, text)] = files else {
            anyhow::bail!("Android: one file per pick");
        };
        download(name, text).await?;
        self.click(trigger).await?;
        // It opens on Recent, which lists no file a shell wrote.
        tap_node("content-desc=\"Show roots\"").await?;
        tap_node("text=\"Downloads\"").await?;
        tap_node(&format!("text=\"{name}\"")).await
    }

    async fn idle(&mut self) {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    fn budget(&self) -> Duration {
        crate::wait::timeout()
    }
}
