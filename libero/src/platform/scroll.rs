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

/// Whether a scroll container clips its z-indexed descendants. Blitz paints
/// them with the nearest stacking context, past any clip in between, so a
/// scroller has to be a stacking context itself there.
pub(crate) fn clips_z_indexed() -> bool {
    !NATIVE
}

/// Whether a released scroll comes to rest on its `scroll-snap` points. Blitz
/// has no scroll snap.
pub(crate) fn snaps_scroll() -> bool {
    !NATIVE
}

/// Whether a scroll container fires `scrollend`. Blitz does not (todo 949).
pub(crate) fn fires_scroll_end() -> bool {
    !NATIVE
}

/// Where `scrollend` does not fire, how long a scroll stays quiet before it
/// counts as ended: longer than the gap between two wheel ticks.
pub(crate) const SCROLL_QUIET: Duration = Duration::from_millis(150);

/// A wheel's vertical travel in the web's sign, positive down the page, with
/// lines and pages counted as `line` pixels and `page` lines. Blitz reports
/// the finger's sign (todo 844).
pub(crate) fn wheel_travel_y(data: &WheelData, line: f64, page: f64) -> f64 {
    let travel = match data.delta() {
        WheelDelta::Pixels(delta) => delta.y,
        WheelDelta::Lines(delta) => delta.y * line,
        WheelDelta::Pages(delta) => delta.y * line * page,
    };
    if NATIVE { -travel } else { travel }
}

/// A live scroll subscription. **Dropping it unsubscribes** - that is the whole
/// contract, which is why the trait has no methods.
///
/// Held by whatever needs the callback to stop: a popover keeps one while it is
/// open and drops it on close, so a closed dropdown costs nothing.
pub trait ScrollSubscription {}

/// Being told that something scrolled.
///
/// The first callback-shaped capability here - everything else is a command or
/// a [`Read`](super::Read), because everything else is a question with an
/// answer. This one is the platform talking back.
///
/// **Anything that scrolls counts, not just the page.** A scroll event does not
/// bubble, so a listener on the window alone misses an element scrolling - and
/// an anchor inside a `ScrollArea` is the common case, not the exotic one. An
/// implementation has to catch both.
pub trait ScrollApi {
    /// Calls `callback` whenever anything scrolls, until the returned
    /// subscription is dropped. No coordinates: the callback's job is to
    /// re-measure, and it has the handles to do that with.
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription>;
}

/// `None` where the renderer cannot report a scroll - a webview and a
/// headless build, where an open popover drifts off its anchor. Blitz reports
/// a wheel inside `LiberoProvider` and libero's own scroll commands.
pub fn scroll() -> Option<&'static dyn ScrollApi> {
    backend::scroll()
}
