use std::time::Duration;

use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::{ScrollData, WheelData};

use super::backend;

const NATIVE: bool = cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// A scroll event's range per axis, `(max_x, max_y)` in px. Blitz reports the
/// range itself as `scroll_width`/`scroll_height`, the web the content's size.
pub(crate) fn scroll_range(data: &ScrollData) -> (f64, f64) {
    let (width, height) = (data.scroll_width() as f64, data.scroll_height() as f64);
    let (max_x, max_y) = match NATIVE {
        true => (width, height),
        false => (
            width - data.client_width() as f64,
            height - data.client_height() as f64,
        ),
    };
    (max_x.max(0.0), max_y.max(0.0))
}

/// Whether a scroll container clips its z-indexed descendants. Blitz paints them
/// past any clip, so a scroller must be a stacking context there.
pub(crate) fn clips_z_indexed() -> bool {
    !NATIVE
}

/// Whether an always-visible `ScrollArea` draws its own track and thumb. A
/// browser may overlay and fade its bars; Blitz keeps painting its own.
pub(crate) fn draws_own_scrollbars() -> bool {
    !NATIVE
}

/// Whether CSS scroll timelines move the drawn bars, so a scroll writes nothing to the
/// DOM: a write per scroll cost Chromium a frame per wheel step (todo 1954).
pub(crate) fn scroll_timelines() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        thread_local! {
            static SUPPORTED: bool = web::css_supports("animation-timeline: scroll()");
        }
        SUPPORTED.with(|supported| *supported)
    }
    // A WebView may have them, but answers only asynchronously.
    #[cfg(not(target_arch = "wasm32"))]
    false
}

#[cfg(target_arch = "wasm32")]
mod web {
    use js_sys::{Function, Reflect};
    use wasm_bindgen::{JsCast, JsValue};

    /// `CSS.supports(condition)`, by name: web-sys's `Css` feature is not enabled.
    pub(super) fn css_supports(condition: &str) -> bool {
        let css = Reflect::get(&js_sys::global(), &JsValue::from_str("CSS")).ok();
        let supports = css.as_ref().and_then(|css| {
            Reflect::get(css, &JsValue::from_str("supports"))
                .ok()?
                .dyn_into::<Function>()
                .ok()
        });
        match (css, supports) {
            (Some(css), Some(supports)) => supports
                .call1(&css, &JsValue::from_str(condition))
                .is_ok_and(|answer| answer.is_truthy()),
            _ => false,
        }
    }
}

/// Whether a released scroll comes to rest on its `scroll-snap` points. Blitz
/// has no scroll snap.
pub(crate) fn snaps_scroll() -> bool {
    !NATIVE
}

/// Whether a focused scroll container scrolls on arrows, pages, Home, End and
/// Space by itself. Blitz does not (todo 1267).
pub(crate) fn scrolls_on_keys() -> bool {
    !NATIVE
}

/// Whether a scroll container fires `scrollend`. Blitz does not (todo 949).
pub(crate) fn fires_scroll_end() -> bool {
    !NATIVE
}

/// Whether `scroll_to` and scroll-into-view fire a `scroll` event. Blitz's do
/// not; there only [`scroll`] reports them.
pub(crate) fn fires_scroll_on_scroll_to() -> bool {
    !NATIVE
}

/// Where `scrollend` does not fire, how long a scroll stays quiet before it
/// counts as ended: longer than the gap between two wheel ticks.
pub(crate) const SCROLL_QUIET: Duration = Duration::from_millis(150);

/// A wheel's vertical travel in px, positive down the page; a line is `line` px,
/// a page `page` lines. Blitz reports the finger's sign (844).
pub(crate) fn wheel_travel_y(data: &WheelData, line: f64, page: f64) -> f64 {
    let travel = match data.delta() {
        WheelDelta::Pixels(delta) => delta.y,
        WheelDelta::Lines(delta) => delta.y * line,
        WheelDelta::Pages(delta) => delta.y * line * page,
    };
    if NATIVE { -travel } else { travel }
}

/// A live scroll subscription. Dropping it unsubscribes.
pub trait ScrollSubscription {}

/// Being told that anything scrolled, not just the page: scroll does not bubble,
/// so an implementation catches element scrolls too.
pub trait ScrollApi {
    /// Calls `callback` whenever anything scrolls, until the returned
    /// subscription is dropped. No coordinates: the callback re-measures.
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription>;
}

/// `None` where no scroll is reported: a WebView, a headless build. Blitz reports
/// a wheel inside `LiberoProvider` and libero's own scroll commands.
///
/// ```no_run
/// let _subscription = libero::platform::scroll()
///     .map(|scroll| scroll.on_scroll(Box::new(|| println!("scrolled"))));
/// ```
pub fn scroll() -> Option<&'static dyn ScrollApi> {
    backend::scroll()
}
