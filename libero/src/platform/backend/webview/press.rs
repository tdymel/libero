use std::rc::Rc;

use dioxus::prelude::spawn;
use serde_json::json;

use super::{Slot, WebViewListener, eval, eval_with, page, runs_scripts};
use crate::platform::{OBSERVE_ATTR, OWNER_ATTR, PRESS_MARKER_ATTR, PressApi, PressSubscription};

/// Whether a press is down, and what waits for its `pointerup` or `pointercancel`; capture on the
/// window, as the web's: a control that stops its press still counts.
const WATCH_PRESSES: &str = "if (window.__lsxPress) return;
    const press = window.__lsxPress = { down: false, waiting: [] };
    window.addEventListener('pointerdown', () => { press.down = true; }, { capture: true });
    const up = () => {
        press.down = false;
        press.waiting.splice(0).forEach((done) => done());
    };
    window.addEventListener('pointerup', up, { capture: true });
    window.addEventListener('pointercancel', up, { capture: true });";

/// Starts tracking presses for [`press_ended`], once per page.
pub(crate) fn watch_presses() {
    if !runs_scripts() {
        return;
    }
    if let Some(page) = page()
        && !page.watching_presses.replace(true)
    {
        eval(WATCH_PRESSES);
    }
}

/// Answers once the press under way, if any, has ended: the answer leaves a task after its
/// `pointerup`, so the press's `click` reaches Rust first (4796).
pub(crate) async fn press_ended() {
    if runs_scripts() {
        let _ = eval(
            "const press = window.__lsxPress;
            if (press?.down) await new Promise((done) => press.waiting.push(done));
            await new Promise((done) => setTimeout(done));
            return 0;",
        )
        .join::<u8>()
        .await;
    }
}

pub(crate) fn press() -> Option<&'static dyn PressApi> {
    runs_scripts().then_some(&PRESS as &'static dyn PressApi)
}

struct WebViewPress;

static PRESS: WebViewPress = WebViewPress;

/// Capture phase, so no handler that stops the press hides it. Sends every
/// marker from the target up to the root.
const ON_PRESS: &str = "const onPress = (event) => {
        const markers = [];
        for (let el = event.target instanceof Element ? event.target : null; el; el = el.parentElement) {
            const value = el.getAttribute(data);
            if (value) markers.push(...value.split(' ').map(Number));
        }
        dioxus.send(markers);
    };
    window.addEventListener('pointerdown', onPress, { capture: true });";

impl PressApi for WebViewPress {
    fn on_press(&self, callback: Box<dyn Fn(Vec<u64>)>) -> Box<dyn PressSubscription> {
        let slot = Slot::new();
        let script = eval_with(
            json!(PRESS_MARKER_ATTR),
            &format!(
                "{ON_PRESS}
                {}
                window.removeEventListener('pointerdown', onPress, {{ capture: true }});",
                slot.park("")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok(markers) = script.recv::<Vec<u64>>().await {
                callback(markers);
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

impl PressSubscription for WebViewListener {}

/// On the WebView's portal outlet: a press inside lands in an open layer.
pub(crate) const PORTAL_ATTR: &str = "data-lsx-portal";

/// The web's swallow (4826), decided in the page: no `outside` reaches it (958). Once per page,
/// ahead of every subscription's press listener, a new press, a cancel or a key clears it.
/// As on the web, the swallowed press moves no focus and its click is sent to Rust (4915).
const SWALLOW: &str = "const [media, kept, observe, owner, portal] = data;
    if (!window.__lsxSwallow) {
        const state = window.__lsxSwallow = { on: false, told: null };
        const clear = () => { state.on = false; state.told = null; };
        for (const type of ['pointerdown', 'pointercancel', 'keydown']) {
            window.addEventListener(type, clear, { capture: true });
        }
        // A tap's mousedown follows its pointerup.
        window.addEventListener('mousedown', (event) => {
            if (event.isTrusted && state.on) event.preventDefault();
        }, { capture: true });
        window.addEventListener('click', (event) => {
            if (!event.isTrusted || !state.on) return;
            const told = state.told;
            clear();
            event.stopPropagation();
            event.preventDefault();
            told?.();
        }, { capture: true });
    }
    const query = window.matchMedia(media);
    // A layer, an open trigger, the anchor of a rendered layer, or the kept element.
    const keeps = (target) => {
        for (let at = target instanceof Element ? target : null; at; at = at.parentElement) {
            if (at.hasAttribute(portal) || at.getAttribute('aria-expanded') === 'true') return true;
            const tag = at.getAttribute(observe);
            if (tag === null) continue;
            if (kept !== null && Number(tag) === kept) return true;
            if (document.querySelector(`[${owner}=\"${CSS.escape(tag)}\"]`)) return true;
        }
        return false;
    };
    const onPress = (event) => {
        if (!query.matches || keeps(event.target)) return;
        window.__lsxSwallow.on = true;
        window.__lsxSwallow.told = () => dioxus.send(0);
    };
    window.addEventListener('pointerdown', onPress, { capture: true });";

pub(crate) fn swallow_outside_clicks(
    media: &str,
    onswallow: Rc<dyn Fn()>,
    kept: Option<u64>,
) -> Option<Box<dyn PressSubscription>> {
    if !runs_scripts() {
        return None;
    }
    let slot = Slot::new();
    let script = eval_with(
        json!([media, kept, OBSERVE_ATTR, OWNER_ATTR, PORTAL_ATTR]),
        &format!(
            "{SWALLOW}
            {}
            window.removeEventListener('pointerdown', onPress, {{ capture: true }});",
            slot.park("")
        ),
    );
    let task = spawn(async move {
        let mut script = script;
        while script.recv::<u8>().await.is_ok() {
            onswallow();
        }
    });
    Some(Box::new(WebViewListener {
        task,
        _slot: Rc::new(slot),
    }))
}
