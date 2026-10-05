use std::time::Duration;

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetCpuThrottlingRateParams;

use super::web::{element, evaluate, json};
use super::{Driver, Platform, Rect};
use crate::android::{
    ALT, CTRL, SHIFT, download, harness, input, input_text, keycode, posted_notifications,
    soft_keyboard_shown, tap_any_node_within, tap_node, tap_notification, tap_notification_action,
    webview_view_focused,
};
use crate::clock::{app_clock, app_count};
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

    /// A swipe lifted its travel again further on, so the page flings on: lifted on
    /// its last move, a touch comes to rest.
    pub async fn fling_from(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.stroke(pointer::Point { x, y }, dx, dy, 2.0).await
    }

    /// A finger's drag of `selector` by `dx`, `dy` in `steps` moves `step_ms` apart,
    /// held still `hold_ms` before it lifts: a drag with a person's pace.
    pub async fn paced_drag(
        &mut self,
        selector: &str,
        (dx, dy): (f64, f64),
        steps: usize,
        step_ms: u64,
        hold_ms: u64,
    ) -> Result<()> {
        let from = self.centre(selector).await?;
        let at = |t: f64| {
            self.device(pointer::Point {
                x: from.x + dx * t,
                y: from.y + dy * t,
            })
        };
        let pause = |ms: u64| {
            [
                ";".into(),
                "sleep".into(),
                format!("{:.3}", ms as f64 / 1000.0),
            ]
        };
        let [x, y] = at(0.0);
        let mut chain: Vec<String> = vec!["motionevent".into(), "DOWN".into(), x, y];
        for step in 1..=steps {
            chain.extend(pause(step_ms));
            let [x, y] = at(step as f64 / steps as f64);
            chain.extend([
                ";".into(),
                "input".into(),
                "motionevent".into(),
                "MOVE".into(),
                x,
                y,
            ]);
        }
        chain.extend(pause(hold_ms));
        let [x, y] = at(1.0);
        chain.extend([
            ";".into(),
            "input".into(),
            "motionevent".into(),
            "UP".into(),
            x,
            y,
        ]);
        input(&chain).await
    }

    /// Fails when the median of three [`Self::drag_start`]s, each on a fresh
    /// `route` under an 8x slower WebView CPU, reads a rect between the move past
    /// the slop and the first moved frame: each read is a round trip, and
    /// measuring there took 180-280 ms (todo 2018). The time varies with the
    /// host's load, the reads do not.
    pub async fn drag_starts_without_a_read(
        route: &str,
        selector: &str,
        delta: (f64, f64),
    ) -> Result<()> {
        let mut runs = Vec::new();
        for _ in 0..3 {
            let mut driver = Self::open(route).await?;
            driver
                .page
                .execute(SetCpuThrottlingRateParams::new(8.0))
                .await?;
            let start = driver.drag_start(selector, delta).await;
            driver
                .page
                .execute(SetCpuThrottlingRateParams::new(1.0))
                .await?;
            runs.push(start?);
            driver.finish("a slow drag start").await?;
        }
        runs.sort_by_key(|&(_, reads)| reads);
        if runs[1].1 > 0 {
            bail!("{selector}'s drag reads rects past its slop: (ms, reads) {runs:?}");
        }
        Ok(())
    }

    /// From the move that takes `selector`'s paced drag past 8px to the first
    /// paint of a non-zero `translate` on a node holding it: the ms, and the
    /// rect reads in between (todo 2018).
    pub async fn drag_start(&mut self, selector: &str, (dx, dy): (f64, f64)) -> Result<(f64, u32)> {
        let probe = format!(
            "(() => {{
                const handle = {handle};
                const moved = s => /translate[^(]*\\((?!0px(, 0px)?\\))/.test(s || '');
                const lag = window.__dragLag = {{ down: null, moves: [], moved: null }};
                const I = window.interpreter;
                if (I && !I.__readsCounted) {{
                    I.__readsCounted = true;
                    const read = I.getClientRect.bind(I);
                    I.getClientRect = (...a) => {{ window.__rectReads = (window.__rectReads || 0) + 1; return read(...a); }};
                }}
                const reads = () => window.__rectReads || 0;
                const opts = {{ capture: true, passive: true }};
                const ac = new AbortController();
                addEventListener('pointerdown', e => {{ lag.down = [e.clientX, e.clientY, performance.now()]; }}, {{ ...opts, signal: ac.signal }});
                addEventListener('pointermove', e => {{ lag.moves.push([performance.now(), e.clientX, e.clientY, reads()]); }}, {{ ...opts, signal: ac.signal }});
                const seen = new MutationObserver(ms => {{
                    if (lag.moved || !lag.down) return;
                    if (ms.some(m => m.target.contains(handle) && moved(m.target.getAttribute('style'))))
                        requestAnimationFrame(t => {{ lag.moved = [t, reads()]; seen.disconnect(); ac.abort(); }});
                }});
                seen.observe(document.body, {{ subtree: true, attributes: true, attributeFilter: ['style'] }});
                return !!handle;
            }})()",
            handle = element(selector)
        );
        if !json::<bool>(&self.page, &probe).await? {
            bail!("no {selector} to drag");
        }
        self.paced_drag(selector, (dx, dy), 12, 16, 600).await?;
        let lag: serde_json::Value = json(&self.page, "window.__dragLag").await?;
        let (Some([x, y, down]), Some([moved, reads])) = (
            serde_json::from_value::<[f64; 3]>(lag["down"].clone()).ok(),
            serde_json::from_value::<[f64; 2]>(lag["moved"].clone()).ok(),
        ) else {
            bail!("no drag start seen: {lag}");
        };
        let moves: Vec<[f64; 4]> = serde_json::from_value(lag["moves"].clone())?;
        let Some(past) = moves.iter().find(|m| (m[1] - x).hypot(m[2] - y) >= 8.0) else {
            bail!("the drag never passed 8px: {lag}");
        };
        let (ms, reads) = (moved - past[0], (reads - past[3]) as u32);
        if std::env::var_os("E2E_FRAMES").is_some() {
            println!(
                "drag start: down->slop {:.0} ms, slop->moved {ms:.0} ms, {reads} reads",
                past[0] - down
            );
        }
        Ok((ms, reads))
    }

    async fn swipe(&mut self, from: pointer::Point, dx: f64, dy: f64) -> Result<()> {
        self.stroke(from, dx, dy, 1.0).await
    }

    /// Four moves to `from + d`, lifted at `from + d * lift`.
    async fn stroke(&mut self, from: pointer::Point, dx: f64, dy: f64, lift: f64) -> Result<()> {
        // `input swipe` lifts 1-2px short of its last move, and a drag follows moves
        // only (1002): step the touch by hand, its last move on the end point.
        // A touch lifted off the screen leaves the WebView a stuck touch that eats later drags.
        let (vw, vh) = self.viewport().await?;
        let mut events = vec!["DOWN".to_string()];
        events.extend(std::iter::repeat_n("MOVE".to_string(), 4));
        events.push("UP".into());
        let mut chain = Vec::new();
        for (i, event) in events.into_iter().enumerate() {
            let t = if i > 4 { lift } else { i as f64 / 4.0 };
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

    /// `KEYCODE_BACK`: wry's activity hands it to the WebView's history, else finishes.
    async fn press_back(&mut self) -> Result<()> {
        input(&["keyevent".into(), "4".into()]).await
    }

    async fn press_shift(&mut self, key: keyboard::Key) -> Result<()> {
        self.chord(SHIFT, key).await
    }

    async fn press_ctrl(&mut self, key: keyboard::Key) -> Result<()> {
        self.chord(CTRL, key).await
    }

    async fn press_alt(&mut self, key: keyboard::Key) -> Result<()> {
        self.chord(ALT, key).await
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

    async fn swipe_from(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.swipe(pointer::Point { x, y }, dx, dy).await
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

    async fn touch_down(&mut self, selector: &str) -> Result<()> {
        let [x, y] = self.device(self.centre(selector).await?);
        input(&["motionevent".into(), "DOWN".into(), x, y]).await
    }

    async fn touch_up(&mut self, selector: &str) -> Result<()> {
        let [x, y] = self.device(pointer::centre_of(&self.page, selector).await?);
        input(&["motionevent".into(), "UP".into(), x, y]).await
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

    /// CDP's: `adb input` has one finger.
    async fn pinch(&mut self, selector: &str, from: f64, to: f64) -> Result<()> {
        let at = self.centre(selector).await?;
        pointer::pinch(&self.page, at, from, to, 8).await
    }

    async fn evaluate(&mut self, expression: &str) -> Result<serde_json::Value> {
        evaluate(&self.page, expression).await
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

    /// The runtime permission dialogs, each answered "While using the app" (camera,
    /// then microphone when both are asked) or "Allow" (notifications).
    async fn allow_permission(&mut self, trigger: &str) -> Result<()> {
        const ALLOW: [&str; 2] = [
            "permission_allow_foreground_only_button",
            "permission_allow_button",
        ];
        self.click(trigger).await?;
        tap_any_node_within(&ALLOW, crate::wait::timeout()).await?;
        let _ = tap_any_node_within(&ALLOW, Duration::from_secs(3)).await;
        Ok(())
    }

    async fn posted_notifications(&mut self) -> Result<String> {
        // The fixtures app's identifier, as in the runner's `PACKAGE`.
        posted_notifications("dev.libero.fixtures").await
    }

    async fn tap_notification(&mut self, title: &str) -> Result<()> {
        tap_notification(title).await
    }

    async fn tap_notification_action(&mut self, label: &str) -> Result<()> {
        tap_notification_action(label).await
    }

    async fn drag_at(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.swipe(pointer::Point { x, y }, dx, dy).await
    }

    /// The app runs outside the WebView: the idle rounds of `settle` carry the frame's report.
    async fn frame(&mut self) -> Result<()> {
        crate::clock::next_frame(&self.page).await?;
        self.settle().await
    }

    async fn idle(&mut self) {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    fn budget(&self) -> Duration {
        crate::wait::timeout()
    }
}
