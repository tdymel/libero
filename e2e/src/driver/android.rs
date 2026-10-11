use std::time::Duration;

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetCpuThrottlingRateParams;

use super::web::{element, evaluate, json};
use super::{Driver, Platform, Rect};
use crate::android::{
    ALT, CTRL, SHIFT, clear_log, disable_ime, download, harness, input, input_text, keycode,
    log_lines, posted_notifications, restore_ime, screen_text, soft_keyboard_shown,
    tap_any_node_within, tap_node, tap_notification, tap_notification_action, webview_view_focused,
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
        restore_ime().await?;
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
        self.settled_centre(selector).await
    }

    /// The first match's centre once the page stopped moving under it: a fling from the
    /// last swipe keeps it moving under a press (1104).
    async fn settled_centre(&self, selector: &str) -> Result<pointer::Point> {
        // A page wider than the screen pans the visual viewport, which `getBoundingClientRect`
        // ignores and a touch does not (2255); the pan animates, so it settles too (3278).
        let pan = "[visualViewport.offsetLeft, visualViewport.offsetTop]";
        let mut at = pointer::centre_of(&self.page, selector).await?;
        let mut offset: [f64; 2] = json(&self.page, pan).await?;
        for _ in 0..40 {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let next = pointer::centre_of(&self.page, selector).await?;
            let panned: [f64; 2] = json(&self.page, pan).await?;
            let settled = (next.x - at.x).abs() < 0.5
                && (next.y - at.y).abs() < 0.5
                && (panned[0] - offset[0]).abs() < 0.5
                && (panned[1] - offset[1]).abs() < 0.5;
            (at, offset) = (next, panned);
            if settled {
                break;
            }
        }
        let [left, top] = offset;
        Ok(pointer::Point {
            x: at.x - left,
            y: at.y - top,
        })
    }

    /// A fast swipe, so the page flings on after the lift.
    pub async fn fling_from(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        input(&self.fling_chain(pointer::Point { x, y }, dx, dy)).await
    }

    /// `input swipe` over 100 ms: its own moves carry the speed. Stepped `motionevent`s are
    /// one process each, up to 190 ms apart on the FP4, too slow to fling (2598).
    fn fling_chain(&self, from: pointer::Point, dx: f64, dy: f64) -> Vec<String> {
        let [x, y] = self.device(from);
        let [x2, y2] = self.device(pointer::Point {
            x: from.x + dx,
            y: from.y + dy,
        });
        vec!["swipe".into(), x, y, x2, y2, "100".into()]
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
        self.paced_chain(from, (dx, dy), steps, step_ms, hold_ms, true)
            .await
    }

    /// [`Self::paced_drag`] from `from`; without `down` the touch is already down.
    async fn paced_chain(
        &self,
        from: pointer::Point,
        (dx, dy): (f64, f64),
        steps: usize,
        step_ms: u64,
        hold_ms: u64,
        down: bool,
    ) -> Result<()> {
        let at = |t: f64| {
            self.device(pointer::Point {
                x: from.x + dx * t,
                y: from.y + dy * t,
            })
        };
        let motion = |event: &str, t: f64| {
            let [x, y] = at(t);
            vec!["motionevent".to_string(), event.into(), x, y]
        };
        let pause = |ms: u64| vec!["sleep".to_string(), format!("{:.3}", ms as f64 / 1000.0)];
        let mut commands = Vec::new();
        if down {
            commands.push(motion("DOWN", 0.0));
        }
        for step in 1..=steps {
            if !commands.is_empty() {
                commands.push(pause(step_ms));
            }
            // Eased out: Chrome extrapolates a fast last move past the drop point (3278, 3313).
            let left = 1.0 - step as f64 / steps as f64;
            commands.push(motion("MOVE", 1.0 - left * left));
        }
        commands.push(pause(hold_ms));
        commands.push(motion("UP", 1.0));
        // `input` leads the first command; the rest are `; input ...` or `; sleep ...`.
        let mut chain = Vec::new();
        for (i, command) in commands.into_iter().enumerate() {
            if i > 0 {
                chain.push(";".into());
                if command[0] != "sleep" {
                    chain.push("input".into());
                }
            }
            chain.extend(command);
        }
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
        // The moves wait for the press's own measure, so a slow host cannot put it past the slop (3267).
        let from = self.centre(selector).await?;
        let before: u64 = json(&self.page, "window.__rectReads || 0").await?;
        let [x, y] = self.device(from);
        input(&["motionevent".into(), "DOWN".into(), x.clone(), y.clone()]).await?;
        let measured = crate::wait::for_js_true(
            &self.page,
            &format!("(window.__rectReads || 0) > {before}"),
            "the press measured the board",
        )
        .await;
        if measured.is_err() {
            input(&["motionevent".into(), "UP".into(), x, y]).await?;
        }
        measured?;
        self.paced_chain(from, (dx, dy), 12, 16, 600, false).await?;
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

    /// The Tab stops of the page, after a D-pad key turned spatial navigation on, that
    /// no focusable element of its own owns: a `cursor: pointer` wrapper (3156).
    pub async fn stray_tab_stops(&mut self) -> Result<Vec<String>> {
        self.tab_walk(true).await
    }

    /// [`Self::stray_tab_stops`] from the focused element on, so an open list stays open.
    pub async fn stray_tab_stops_open(&mut self) -> Result<Vec<String>> {
        self.tab_walk(false).await
    }

    /// The drawn topmost elements of a `cursor: pointer` subtree that no focusable element
    /// owns: the Tab stops spatial navigation would add, found without the Tab walk that a
    /// list closing on Tab cuts short.
    pub async fn pointer_wrappers(&mut self) -> Result<Vec<String>> {
        pointer::move_to(&self.page, pointer::Point { x: -5.0, y: -5.0 }).await?;
        json(
            &self.page,
            "[...document.querySelectorAll('body *')].filter(e => {
                const cursor = el => el ? getComputedStyle(el).cursor : '';
                if (cursor(e) !== 'pointer' || cursor(e.parentElement) === 'pointer') return false;
                // Dioxus' toast host is not libero's.
                if (!e.getClientRects().length || e.matches(':hover') || e.closest('#__dx-toast-inner')) return false;
                return !e.matches('a[href], area[href], button, input, select, textarea, summary, iframe, [tabindex], [contenteditable]');
            }).map(e => e.outerHTML.slice(0, 240))",
        )
        .await
    }

    async fn tab_walk(&mut self, from_top: bool) -> Result<Vec<String>> {
        self.leave_touch_mode().await?;
        input(&["keyevent".into(), keycode("ArrowDown")?.to_string()]).await?;
        // The emulator's pointer rests over the page: nothing hovered, as on a phone.
        pointer::move_to(&self.page, pointer::Point { x: -5.0, y: -5.0 }).await?;
        if from_top {
            self.page
                .evaluate("document.activeElement?.blur(); scrollTo(0, 0)")
                .await?;
        }
        let describe = "(() => {
            const e = document.activeElement;
            if (!e || e === document.body) return null;
            // The app root and dioxus' toast host: scrollers Chromium focuses, not libero's.
            if (e.matches('#main, #__dx-toast-inner')) return [true, '#' + e.id];
            const own = e.matches('a[href], area[href], button, input, select, textarea, summary, iframe, [tabindex], [contenteditable], audio[controls], video[controls]');
            const cursor = el => el ? getComputedStyle(el).cursor : '';
            return [own, `${cursor(e)}/${cursor(e.parentElement)} hover=${e.matches(':hover')}${e.outerHTML.slice(0, 240)}`];
        })()";
        let mut strays = Vec::new();
        let mut seen = Vec::new();
        for _ in 0..150 {
            input(&["keyevent".into(), keycode("Tab")?.to_string()]).await?;
            let Some((own, html)) = json::<Option<(bool, String)>>(&self.page, describe).await?
            else {
                break;
            };
            if seen.contains(&html) {
                break;
            }
            if !own {
                strays.push(html.clone());
            }
            seen.push(html);
        }
        Ok(strays)
    }

    /// A [`Self::fling_from`] and a [`Self::swipe_from`] in one `input` chain: no host
    /// round trip between them, so the swipe lands while the page still flings (3139).
    pub async fn fling_then_swipe(
        &mut self,
        (x, y, dx, dy): (f64, f64, f64, f64),
        (sx, sy, sdx, sdy): (f64, f64, f64, f64),
    ) -> Result<()> {
        let mut chain = self.fling_chain(pointer::Point { x, y }, dx, dy);
        chain.extend([";".into(), "input".into()]);
        chain.extend(
            self.stroke_chain(pointer::Point { x: sx, y: sy }, sdx, sdy, 1.0, false)
                .await?,
        );
        input(&chain).await
    }

    /// Four moves to `from + d`, lifted at `from + d * lift`.
    async fn stroke(&mut self, from: pointer::Point, dx: f64, dy: f64, lift: f64) -> Result<()> {
        let chain = self.stroke_chain(from, dx, dy, lift, false).await?;
        input(&chain).await
    }

    /// [`Self::stroke`] slowing into its end point: Chrome extrapolates a fast last move by
    /// up to 8 ms of its velocity, 30px past the point a drop is read at (3278).
    async fn drag_stroke(&mut self, from: pointer::Point, dx: f64, dy: f64) -> Result<()> {
        let chain = self.stroke_chain(from, dx, dy, 1.0, true).await?;
        input(&chain).await
    }

    /// [`Self::stroke`]'s `input` arguments; `eased` slows the last moves.
    async fn stroke_chain(
        &mut self,
        from: pointer::Point,
        dx: f64,
        dy: f64,
        lift: f64,
        eased: bool,
    ) -> Result<Vec<String>> {
        // `input swipe` lifts 1-2px short of its last move, and a drag follows moves
        // only (1002): step the touch by hand, its last move on the end point.
        // A touch lifted off the screen leaves the WebView a stuck touch that eats later drags.
        let [vw, vh]: [f64; 2] =
            json(&self.page, "[visualViewport.width, visualViewport.height]").await?;
        let mut events = vec!["DOWN".to_string()];
        events.extend(std::iter::repeat_n("MOVE".to_string(), 4));
        events.push("UP".into());
        let mut chain = Vec::new();
        for (i, event) in events.into_iter().enumerate() {
            let t = match (i, eased) {
                (5.., _) => lift,
                (_, true) => [0.0, 0.5, 0.85, 0.97, 1.0][i],
                _ => i as f64 / 4.0,
            };
            let [x, y] = self.device(pointer::Point {
                x: (from.x + dx * t).clamp(0.0, vw - 1.0),
                y: (from.y + dy * t).clamp(0.0, vh - 1.0),
            });
            if i > 0 {
                chain.extend([";".into(), "input".into()]);
            }
            chain.extend(["motionevent".into(), event, x, y]);
        }
        Ok(chain)
    }

    /// Returns once the page saw the tap: `input tap` returns before it lands, and a late
    /// tap's focus would take the next `focus()` back (3268).
    async fn tap(&self, at: pointer::Point) -> Result<()> {
        self.page
            .evaluate(
                "(() => {
                    window.__tap?.abort();
                    const ac = window.__tap = new AbortController();
                    window.__tapped = false;
                    const done = () => { window.__tapped = true; };
                    addEventListener('click', done, { capture: true, signal: ac.signal });
                    addEventListener('pointercancel', done, { capture: true, signal: ac.signal });
                    // A prevented touchend raises no click.
                    addEventListener('touchend', e => e.defaultPrevented && done(), { signal: ac.signal });
                })()",
            )
            .await?;
        let [x, y] = self.device(at);
        input(&["tap".into(), x, y]).await?;
        crate::wait::for_js_true(&self.page, "window.__tapped === true", "the tap landed").await
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

    async fn clear_log(&mut self) -> Result<()> {
        clear_log().await
    }

    async fn log_lines(&mut self, needle: &str) -> Result<String> {
        log_lines(needle).await
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
        self.centre(selector).await?;
        // A drag's end past the screen's edge is clamped short (2255): pan it into view, sideways.
        self.page
            .evaluate(format!(
                "(e => {{
                    const r = e.getBoundingClientRect();
                    const end = document.createElement('div');
                    end.style.cssText = `position:absolute;pointer-events:none;width:48px;height:48px;\
                        left:${{r.x + r.width / 2 + {dx} + scrollX - 24}}px;top:${{r.y + r.height / 2 + scrollY - 24}}px`;
                    document.body.append(end);
                    end.scrollIntoView({{ block: 'nearest', inline: 'nearest' }});
                    end.remove();
                }})({})",
                element(selector)
            ))
            .await?;
        let from = self.settled_centre(selector).await?;
        self.drag_stroke(from, dx, dy).await
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

    async fn plain_keys(&mut self) -> Result<()> {
        disable_ime().await
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

    async fn system_text(&mut self) -> Result<String> {
        screen_text().await
    }

    async fn posted_notifications(&mut self) -> Result<String> {
        let package = std::env::var(crate::android::PACKAGE_ENV)
            .unwrap_or_else(|_| "dev.libero.fixtures".into());
        posted_notifications(&package).await
    }

    async fn tap_notification(&mut self, title: &str) -> Result<()> {
        tap_notification(title).await
    }

    async fn tap_notification_action(&mut self, label: &str) -> Result<()> {
        tap_notification_action(label).await
    }

    async fn drag_at(&mut self, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
        self.drag_stroke(pointer::Point { x, y }, dx, dy).await
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
