use std::time::Duration;

use dioxus::html::geometry::WheelDelta;
pub(crate) use dioxus::prelude::ScrollData;
use dioxus::prelude::WheelData;

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

/// Whether a windowed table reserves its skipped rows in its body: a browser
/// scrolls ahead of the rows. Blitz keeps the padding it was verified with.
pub(crate) fn reserves_rows_in_tables() -> bool {
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

/// One element's scroll position, as a `scroll` event carries it.
#[cfg_attr(any(target_arch = "wasm32", feature = "native"), allow(dead_code))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ScrollReport {
    pub top: f64,
    pub left: f64,
    pub scroll_width: i32,
    pub scroll_height: i32,
    pub client_width: i32,
    pub client_height: i32,
}

impl dioxus::html::HasScrollData for ScrollReport {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn scroll_top(&self) -> f64 {
        self.top
    }
    fn scroll_left(&self) -> f64 {
        self.left
    }
    fn scroll_width(&self) -> i32 {
        self.scroll_width
    }
    fn scroll_height(&self) -> i32 {
        self.scroll_height
    }
    fn client_width(&self) -> i32 {
        self.client_width
    }
    fn client_height(&self) -> i32 {
        self.client_height
    }
}

/// The scrolls of the element carrying `tag`, about one a frame and the last
/// one always, flagged `true` on `scrollend`, without holding up the page.
/// `None` where its `scroll` event is cheap: a WebView's is a synchronous round
/// trip to Rust (todo 2131).
pub(crate) fn on_element_scroll(
    tag: u64,
    callback: Box<dyn Fn(ScrollData, bool)>,
) -> Option<Box<dyn ScrollSubscription>> {
    backend::on_element_scroll(tag, callback)
}

/// Brings the element into view through every scroller around it, on both axes
/// (`scrollIntoView` `nearest`). Blitz moves its nearest scroller only.
pub(crate) fn scroll_chain_into_view(
    mounted: &std::rc::Rc<dioxus::prelude::MountedData>,
    smooth: bool,
) -> Result<(), super::PlatformError> {
    backend::scroll_chain_into_view(mounted, smooth)
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

/// Calls `callback` when the viewport resizes, the visual one included (a soft keyboard),
/// until the subscription drops. `None` on Blitz: no window resize is reported.
pub(crate) fn on_viewport_resize(callback: Box<dyn Fn()>) -> Option<Box<dyn ScrollSubscription>> {
    backend::on_viewport_resize(callback)
}

/// Where an edge swipe starts: `inset..inset + width` px in from the left edge, or
/// from the right one; `distance` is how far it travels before it counts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EdgeBand {
    pub(crate) inset: f64,
    pub(crate) width: f64,
    pub(crate) left: bool,
    pub(crate) distance: f64,
}

/// A swipe the browser took from the pointer: where it started and how far it
/// went, `[x, y, dx, dy]` in client px.
pub(crate) type LostSwipe = Box<dyn Fn([f64; 4])>;

/// The custom property [`hold_edge_pan`] finds the swiped element by, inherited.
pub(crate) const EDGE_SWIPE_MARK: &str = "--lsx-edge-swipe";

/// `(inset, width, left, distance, swiped) => remove`: holds every move of a band touch
/// going inward, mostly sideways, from inner scrollers. One whose pointer was cancelled
/// anyway (its first move uncancelable in a fling) goes to `swiped` past `distance` (2190).
#[cfg_attr(all(not(target_arch = "wasm32"), feature = "native"), allow(dead_code))]
pub(crate) const EDGE_PAN_JS: &str = "(inset, width, left, distance, swiped) => {
    let start = null;
    let origin = null;
    let holding = false;
    let lost = false;
    const reset = () => {
        start = null;
        origin = null;
        holding = false;
        lost = false;
    };
    const down = (event) => {
        reset();
        const touch = event.touches[0];
        if (event.touches.length !== 1 || !(event.target instanceof Element)) return;
        const from = left ? touch.clientX : innerWidth - touch.clientX;
        if (from < inset || from > inset + width) return;
        if (getComputedStyle(event.target).getPropertyValue('--lsx-edge-swipe').trim() !== '1') return;
        start = { x: touch.clientX, y: touch.clientY };
        origin = start;
    };
    const move = (event) => {
        if (event.touches.length !== 1) reset();
        const touch = event.touches[0];
        if (start) {
            const dx = (touch.clientX - start.x) * (left ? 1 : -1);
            const dy = Math.abs(touch.clientY - start.y);
            if (Math.max(Math.abs(dx), dy) < 4) return;
            start = null;
            holding = dx > dy;
        }
        if (!holding) return;
        if (event.cancelable) event.preventDefault();
        if (!lost) return;
        const dx = touch.clientX - origin.x;
        const dy = touch.clientY - origin.y;
        if (Math.max(Math.abs(dx), Math.abs(dy)) < distance) return;
        swiped(origin.x, origin.y, dx, dy);
        reset();
    };
    const cancel = (event) => {
        if (event.pointerType === 'touch' && (start || holding)) lost = true;
    };
    const capture = { capture: true };
    const passive = { capture: true, passive: true };
    addEventListener('touchstart', down, passive);
    addEventListener('touchmove', move, { capture: true, passive: false });
    addEventListener('pointercancel', cancel, passive);
    addEventListener('touchend', reset, passive);
    addEventListener('touchcancel', reset, passive);
    return () => {
        removeEventListener('touchstart', down, capture);
        removeEventListener('touchmove', move, capture);
        removeEventListener('pointercancel', cancel, capture);
        removeEventListener('touchend', reset, capture);
        removeEventListener('touchcancel', reset, capture);
    };
}";

/// Holds a sideways pan from `band` for an edge swipe until the subscription drops: an
/// inner scroller's `touch-action` would else take the touch and cancel the pointer
/// (2170). Only inside an element carrying [`EDGE_SWIPE_MARK`]; `None` on Blitz. A
/// swipe the pointer lost all the same goes to `swiped`.
pub(crate) fn hold_edge_pan(
    band: EdgeBand,
    swiped: LostSwipe,
) -> Option<Box<dyn ScrollSubscription>> {
    backend::hold_edge_pan(band, swiped)
}
