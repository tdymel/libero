use std::rc::Rc;

use dioxus::prelude::spawn;
use serde_json::json;

use super::{Slot, WebViewListener, eval, eval_with, runs_scripts};
use crate::platform::scroll::{ScrollData, ScrollReport};
use crate::platform::{OBSERVE_ATTR, PlatformError, Read, ScrollApi, ScrollSubscription};

pub(crate) fn scroll() -> Option<&'static dyn ScrollApi> {
    runs_scripts().then_some(&SCROLL as &'static dyn ScrollApi)
}

struct WebViewScroll;

static SCROLL: WebViewScroll = WebViewScroll;

/// Capture phase on the window, as on the web: a scroll does not bubble. One
/// message per frame at most, since each crosses the IPC.
const ON_SCROLL: &str = "let queued = false;
    const tick = () => {
        if (queued) return;
        queued = true;
        requestAnimationFrame(() => { queued = false; dioxus.send(null); });
    };
    window.addEventListener('scroll', tick, { capture: true, passive: true });";

impl ScrollApi for WebViewScroll {
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription> {
        let slot = Slot::new();
        let script = eval(&format!(
            "{ON_SCROLL}
            {}
            window.removeEventListener('scroll', tick, {{ capture: true }});",
            slot.park("")
        ));
        let task = spawn(async move {
            let mut script = script;
            while script.recv::<()>().await.is_ok() {
                callback();
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

/// The visual viewport's edges in layout coordinates, the whole viewport where none is panned.
pub(crate) fn visible_band() -> Read<crate::platform::VisibleBand> {
    let read = eval(
        "const visual = window.visualViewport;
        const left = Math.max(0, visual?.offsetLeft ?? 0);
        return [left, Math.max(0, visual?.offsetTop ?? 0), visual ? left + visual.width : window.innerWidth];",
    );
    Box::pin(async move {
        let (left, top, right) = read
            .join::<(f64, f64, f64)>()
            .await
            .map_err(|_| PlatformError::Unsupported)?;
        Ok(crate::platform::VisibleBand { left, top, right })
    })
}

/// The window and the visual viewport (a soft keyboard), throttled as [`ON_SCROLL`].
pub(crate) fn on_viewport_resize(callback: Box<dyn Fn()>) -> Option<Box<dyn ScrollSubscription>> {
    if !runs_scripts() {
        return None;
    }
    let slot = Slot::new();
    let script = eval(&format!(
        "let queued = false;
        const tick = () => {{
            if (queued) return;
            queued = true;
            requestAnimationFrame(() => {{ queued = false; dioxus.send(null); }});
        }};
        const visual = window.visualViewport;
        window.addEventListener('resize', tick);
        visual?.addEventListener('resize', tick);
        visual?.addEventListener('scroll', tick);
        {}
        window.removeEventListener('resize', tick);
        visual?.removeEventListener('resize', tick);
        visual?.removeEventListener('scroll', tick);",
        slot.park("")
    ));
    let task = spawn(async move {
        let mut script = script;
        while script.recv::<()>().await.is_ok() {
            callback();
        }
    });
    Some(Box::new(WebViewListener {
        task,
        _slot: Rc::new(slot),
    }))
}

/// Delegated from the window, so a remounted element under the same tag still
/// reports. The first step of a frame goes at once, the rest as one at the next
/// frame: a starved frame would hold the first back too. `scrollend` carries the
/// last position itself, so it never lands before a queued step. A step under a
/// px is dropped: Android rocks a device px at an edge for seconds (todo 2178).
const ON_ELEMENT_SCROLL: &str = "const selector = '[' + data[0] + '=\"' + data[1] + '\"]';
    let queued = null, armed = false, seq = 0, top = NaN, left = NaN;
    const send = (s, ended) => {
        if (!s.isConnected) return;
        if (!ended && Math.abs(s.scrollTop - top) < 1 && Math.abs(s.scrollLeft - left) < 1) return;
        top = s.scrollTop;
        left = s.scrollLeft;
        seq += 1;
        dioxus.send([seq, s.scrollTop, s.scrollLeft, s.scrollWidth, s.scrollHeight, s.clientWidth, s.clientHeight, ended]);
    };
    const flush = () => {
        armed = false;
        if (queued !== null) send(queued, false);
        queued = null;
    };
    const ours = (event) => event.target instanceof Element && event.target.matches(selector);
    const tick = (event) => {
        if (!ours(event)) return;
        if (armed) {
            queued = event.target;
            return;
        }
        armed = true;
        send(event.target, false);
        requestAnimationFrame(flush);
    };
    const end = (event) => {
        if (!ours(event)) return;
        queued = null;
        send(event.target, true);
    };
    window.addEventListener('scroll', tick, { capture: true, passive: true });
    window.addEventListener('scrollend', end, { capture: true, passive: true });";

pub(crate) fn on_element_scroll(
    tag: u64,
    callback: Box<dyn Fn(ScrollData, bool)>,
) -> Option<Box<dyn ScrollSubscription>> {
    if !runs_scripts() {
        return None;
    }
    let slot = Slot::new();
    let script = eval_with(
        json!([OBSERVE_ATTR, tag.to_string()]),
        &format!(
            "{ON_ELEMENT_SCROLL}
            {}
            window.removeEventListener('scroll', tick, {{ capture: true }});
            window.removeEventListener('scrollend', end, {{ capture: true }});",
            slot.park("")
        ),
    );
    let task = spawn(async move {
        let mut script = script;
        let mut last = 0;
        while let Ok((
            seq,
            top,
            left,
            scroll_width,
            scroll_height,
            client_width,
            client_height,
            ended,
        )) = script
            .recv::<(u64, f64, f64, i32, i32, i32, i32, bool)>()
            .await
        {
            // Never back to an older position, should one arrive late.
            if seq <= last {
                continue;
            }
            last = seq;
            let report = ScrollReport {
                top,
                left,
                scroll_width,
                scroll_height,
                client_width,
                client_height,
            };
            callback(ScrollData::new(report), ended);
        }
    });
    Some(Box::new(WebViewListener {
        task,
        _slot: Rc::new(slot),
    }))
}

/// Injected: `preventDefault` has to answer the `touchmove` synchronously.
pub(crate) fn hold_edge_pan(
    band: crate::platform::scroll::EdgeBand,
    swiped: crate::platform::scroll::LostSwipe,
) -> Option<Box<dyn ScrollSubscription>> {
    if !runs_scripts() {
        return None;
    }
    let slot = Slot::new();
    let script = eval(&format!(
        "const remove = ({})({}, {}, {}, {}, (...swipe) => dioxus.send(swipe));
        {}
        remove();",
        crate::platform::scroll::EDGE_PAN_JS,
        band.inset,
        band.width,
        band.left,
        band.distance,
        slot.park("")
    ));
    let task = spawn(async move {
        let mut script = script;
        while let Ok(swipe) = script.recv::<[f64; 4]>().await {
            swiped(swipe);
        }
    });
    Some(Box::new(WebViewListener {
        task,
        _slot: Rc::new(slot),
    }))
}

impl ScrollSubscription for WebViewListener {}
