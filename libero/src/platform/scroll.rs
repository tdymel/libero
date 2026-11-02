use std::{rc::Rc, time::Duration};

use dioxus::html::geometry::WheelDelta;
pub(crate) use dioxus::prelude::ScrollData;
use dioxus::prelude::{MountedData, WheelData};

use super::{Dimensions, Read, backend};

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

/// How [`unpinned_scroll`] finds what a scroller pins by its scroll offset.
#[derive(Clone, Copy)]
#[cfg_attr(all(not(target_arch = "wasm32"), feature = "native"), allow(dead_code))]
pub(crate) struct PinnedLayers<'a> {
    /// From the scroller, the layer riding the block offset.
    pub layer: &'a str,
    /// From `layer`, the one riding the inline offset; absent while nothing is drawn.
    pub inline_layer: &'a str,
    pub range_y: &'a str,
    pub range_x: &'a str,
}

/// Content size and `(left, top)` measured in one task with the layers out of the overflow they
/// hold (5306), then restored at the new range; `None` while the content reaches the range anyway.
pub(crate) fn unpinned_scroll(
    root: &Rc<MountedData>,
    tag: Option<u64>,
    layers: PinnedLayers<'_>,
    view: Dimensions,
) -> Read<Option<(Dimensions, (f64, f64))>> {
    backend::unpinned_scroll(root, tag, layers, view)
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

/// Whether the wheel reports its travel in pages, as a page-mode wheel setting does.
pub(crate) fn wheel_is_paged(data: &WheelData) -> bool {
    matches!(data.delta(), WheelDelta::Pages(_))
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
/// (`scrollIntoView` `nearest`). Blitz jumps, whatever `smooth` says.
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
/// until the subscription drops. Blitz reports no window resize: the size is polled. The web
/// calls back on every event, a WebView once per frame: keep the callback cheap.
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

/// A band press on an app's `touch-action: none` box under the swiped element, by pointer id.
pub(crate) type RefusedPress = Box<dyn Fn(i32)>;

/// The custom property [`hold_edge_pan`] finds the swiped element by, inherited.
pub(crate) const EDGE_SWIPE_MARK: &str = "--lsx-edge-swipe";

/// `(inset, width, left, distance, swiped, refused) => remove`: holds every move of a band
/// touch going inward, mostly sideways, from inner scrollers. One whose pointer was cancelled
/// anyway (its first move uncancelable in a fling) goes to `swiped` past `distance` (2190).
/// A band press on a `touch-action: none` box inside goes to `refused` and is never held (4800).
#[cfg_attr(all(not(target_arch = "wasm32"), feature = "native"), allow(dead_code))]
pub(crate) const EDGE_PAN_JS: &str = "(inset, width, left, distance, swiped, refused) => {
    let start = null;
    let origin = null;
    let holding = false;
    let lost = false;
    let under = [];
    let pinned = [];
    const reset = () => {
        start = null;
        origin = null;
        holding = false;
        lost = false;
        under = [];
    };
    const release = () => {
        for (const [box, at, overflow] of pinned) {
            box.scrollLeft = at;
            box.style.overflowX = overflow;
        }
        pinned = [];
    };
    const inBand = (x) => {
        const from = left ? x : innerWidth - x;
        return from >= inset && from <= inset + width;
    };
    const marked = (box) => getComputedStyle(box).getPropertyValue('--lsx-edge-swipe').trim() === '1';
    // The mark is inherited: a box without it lies past the swiped element.
    const refuses = (target) => {
        for (let box = target; box instanceof Element && marked(box); box = box.parentElement) {
            if (getComputedStyle(box).touchAction === 'none') return true;
        }
        return false;
    };
    const press = (event) => {
        if (event.pointerType === 'mouse' || !event.isPrimary || !inBand(event.clientX)) return;
        if (refuses(event.target)) refused(event.pointerId);
    };
    const down = (event) => {
        reset();
        const touch = event.touches[0];
        if (event.touches.length !== 1 || !(event.target instanceof Element)) return;
        if (!inBand(touch.clientX)) return;
        if (!marked(event.target) || refuses(event.target)) return;
        start = { x: touch.clientX, y: touch.clientY };
        origin = start;
        for (let box = event.target; box; box = box.parentElement) {
            if (box.scrollWidth > box.clientWidth && /auto|scroll/.test(getComputedStyle(box).overflowX)) {
                under.push([box, box.scrollLeft, box.style.overflowX]);
            }
        }
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
            // In a fling the touch cannot be held: freeze the scrollers under it instead (2586).
            if (holding && !event.cancelable) {
                release();
                pinned = under;
                for (const [box, at] of pinned) {
                    box.style.overflowX = 'hidden';
                    box.scrollLeft = at;
                }
            }
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
    // Two frames on, past the fling the lift would start.
    const end = () => {
        reset();
        if (pinned.length) requestAnimationFrame(() => requestAnimationFrame(release));
    };
    const capture = { capture: true };
    const passive = { capture: true, passive: true };
    addEventListener('pointerdown', press, passive);
    addEventListener('touchstart', down, passive);
    addEventListener('touchmove', move, { capture: true, passive: false });
    addEventListener('pointercancel', cancel, passive);
    addEventListener('touchend', end, passive);
    addEventListener('touchcancel', end, passive);
    return () => {
        removeEventListener('pointerdown', press, capture);
        removeEventListener('touchstart', down, capture);
        removeEventListener('touchmove', move, capture);
        removeEventListener('pointercancel', cancel, capture);
        removeEventListener('touchend', end, capture);
        removeEventListener('touchcancel', end, capture);
        release();
    };
}";

/// Holds a sideways pan from `band` for an edge swipe until the subscription drops: an
/// inner scroller's `touch-action` would else take the touch and cancel the pointer
/// (2170). Only inside an element carrying [`EDGE_SWIPE_MARK`]; `None` on Blitz. A
/// swipe the pointer lost all the same goes to `swiped`, a press it must leave to `refused`.
pub(crate) fn hold_edge_pan(
    band: EdgeBand,
    swiped: LostSwipe,
    refused: RefusedPress,
) -> Option<Box<dyn ScrollSubscription>> {
    backend::hold_edge_pan(band, swiped, refused)
}

/// Whether `press` lands on a `touch-action: none` box inside the element carrying
/// [`EDGE_SWIPE_MARK`]: Blitz hit-tests it (4814), elsewhere [`hold_edge_pan`]'s script reports it.
pub(crate) fn refuses_edge_swipe(press: &dioxus::prelude::PointerEvent) -> bool {
    backend::refuses_edge_swipe(press)
}

/// Whether an edge swipe holds the pan by cancelling its own `pointermove`s: Blitz runs
/// no script for [`hold_edge_pan`], but skips the touch pan of a cancelled move (2792).
pub(crate) fn holds_edge_pan_by_pointer() -> bool {
    NATIVE
}

/// The attribute a swiped-closed box carries, valued with its [`hold_sheet_pan`] id.
pub(crate) const SWIPE_CLOSE_ATTR: &str = "data-swipe-close";
/// The attribute of a swiped box's strip: a swipe from it is always the box's, and the
/// box swipes only while its strip is drawn. The script names it literally.
#[cfg_attr(
    not(all(not(target_arch = "wasm32"), feature = "native")),
    allow(dead_code)
)]
pub(crate) const SWIPE_HANDLE_ATTR: &str = "data-swipe-handle";

/// How far, in CSS px, a touch moves before [`hold_sheet_pan`] decides whose it is.
#[cfg_attr(all(not(target_arch = "wasm32"), feature = "native"), allow(dead_code))]
pub(crate) const SHEET_PAN_SLOP: f64 = 4.0;

/// A touch [`hold_sheet_pan`] took for its box: moved `dx`, `dy` from where it took it, then let go.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SheetPan {
    Move { dx: f64, dy: f64 },
    End,
    Cancel,
}

impl SheetPan {
    /// From the script's `[kind, dx, dy]`.
    #[cfg_attr(all(not(target_arch = "wasm32"), feature = "native"), allow(dead_code))]
    pub(crate) fn from_report(kind: f64, dx: f64, dy: f64) -> Self {
        match kind as i32 {
            0 => SheetPan::Move { dx, dy },
            1 => SheetPan::End,
            _ => SheetPan::Cancel,
        }
    }
}

/// `(id, slop, report) => remove`: a touch in the box marked `id` going down first is the
/// box's, unless it presses an inner control owning a vertical drag (an input, a `touch-action`
/// without `pan-y`) or a scroller on its way is scrolled down. That one scrolls, and hands
/// over once it is back at the top and still moving down (Android's nested scroll, 4994).
/// Taking the touch cancels the pointer at the press, as the browser's own pan would.
/// Reports `[0, dx, dy]` per held move, `[1]` on the lift, `[2]` on a cancel.
#[cfg_attr(all(not(target_arch = "wasm32"), feature = "native"), allow(dead_code))]
pub(crate) const SHEET_PAN_JS: &str = "(id, slop, report) => {
    let press = null;
    let pointer = null;
    const owner = (target) => target instanceof Element ? target.closest('[data-swipe-close]') : null;
    const shown = (box) => {
        const strip = box.querySelector('[data-swipe-handle]');
        return !!strip && strip.closest('[data-swipe-close]') === box && strip.getClientRects().length > 0;
    };
    const refuses = (target, box) => {
        for (let el = target; el && el !== box; el = el.parentElement) {
            if (el.matches('input, textarea, select, [contenteditable]:not([contenteditable=\"false\"])')) return true;
            const action = getComputedStyle(el).touchAction;
            if (action !== 'auto' && action !== 'manipulation' && !action.split(' ').includes('pan-y')) return true;
        }
        return false;
    };
    const scrolled = (target, box) => {
        for (let el = target; el; el = el.parentElement) {
            if (el.scrollTop > 0.5 && el.scrollHeight > el.clientHeight && /auto|scroll/.test(getComputedStyle(el).overflowY)) return true;
            if (el === box) return false;
        }
        return false;
    };
    const drop = () => {
        if (press?.state === 'drag') report(2, 0, 0);
        press = null;
    };
    const down = (event) => {
        drop();
        const box = owner(event.target);
        if (event.touches.length !== 1 || !box || box.getAttribute('data-swipe-close') !== id || !shown(box)) return;
        const strip = event.target.closest('[data-swipe-handle]');
        const grip = !!strip && strip.closest('[data-swipe-close]') === box;
        const selecting = !(getSelection()?.isCollapsed ?? true);
        if (!grip && (selecting || refuses(event.target, box))) return;
        const touch = event.touches[0];
        press = { target: event.target, box, grip, pointer, x: touch.clientX, y: touch.clientY, last: touch.clientY, state: 'wait' };
    };
    // A stylus fires touch events too in Chromium on Android and in Safari (5054).
    const pressed = (event) => {
        if ((event.pointerType === 'touch' || event.pointerType === 'pen') && event.isPrimary) pointer = { id: event.pointerId, type: event.pointerType };
    };
    // An inner control's pending press (a sideways slider, a month swipe) drops, as on a pan.
    const take = () => {
        press.state = 'drag';
        if (press.pointer === null) return;
        press.target.dispatchEvent(new PointerEvent('pointercancel', { pointerId: press.pointer.id, pointerType: press.pointer.type, isPrimary: true, bubbles: true }));
    };
    const move = (event) => {
        if (!press) return;
        if (event.touches.length !== 1) return drop();
        const touch = event.touches[0];
        if (press.state === 'wait') {
            const dx = touch.clientX - press.x;
            const dy = touch.clientY - press.y;
            if (Math.max(Math.abs(dx), Math.abs(dy)) < slop) return;
            // Sideways is a horizontal scroller's or a control's.
            if (Math.abs(dx) >= Math.abs(dy)) return drop();
            if (dy > 0 && (press.grip || !scrolled(press.target, press.box))) take();
            else press.state = 'scroll';
        } else if (press.state === 'scroll' && touch.clientY > press.last && !scrolled(press.target, press.box)) {
            take();
            press.x = touch.clientX;
            press.y = press.last;
        }
        press.last = touch.clientY;
        if (press.state !== 'drag') return;
        if (event.cancelable) event.preventDefault();
        report(0, touch.clientX - press.x, touch.clientY - press.y);
    };
    const end = (event) => {
        if (press?.state === 'drag') report(event.type === 'touchend' ? 1 : 2, 0, 0);
        press = null;
    };
    const passive = { capture: true, passive: true };
    const capture = { capture: true };
    addEventListener('pointerdown', pressed, passive);
    addEventListener('touchstart', down, passive);
    addEventListener('touchmove', move, { capture: true, passive: false });
    addEventListener('touchend', end, passive);
    addEventListener('touchcancel', end, passive);
    return () => {
        drop();
        removeEventListener('pointerdown', pressed, capture);
        removeEventListener('touchstart', down, capture);
        removeEventListener('touchmove', move, capture);
        removeEventListener('touchend', end, capture);
        removeEventListener('touchcancel', end, capture);
    };
}";

/// Holds a touch pan of the box marked `id` (see [`SHEET_PAN_JS`]) until the subscription
/// drops, reporting it to `report`. `None` on Blitz, which runs no script: see [`swipe_press`].
pub(crate) fn hold_sheet_pan(
    id: u64,
    report: Box<dyn Fn(SheetPan)>,
) -> Option<Box<dyn ScrollSubscription>> {
    backend::hold_sheet_pan(id, report)
}

/// What a press on a swiped box lands on, read at once: Blitz only, elsewhere the script decides.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SwipePress {
    /// The box's strip is drawn: it swipes at all.
    pub(crate) shown: bool,
    /// An inner control owning a drag keeps it.
    pub(crate) refused: bool,
    /// A scroller between the press and the box is scrolled down.
    pub(crate) scrolled: bool,
}

/// [`SwipePress`] for a press at `event`'s point in the box marked `id`; `None` off Blitz.
pub(crate) fn swipe_press(event: &dioxus::prelude::PointerEvent, id: u64) -> Option<SwipePress> {
    backend::swipe_press(event, id)
}
