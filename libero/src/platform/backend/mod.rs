//! One module per renderer, plus two portable ones: [`mounted`], the floor
//! every renderer shares for elements, and [`thread`], the non-wasm timer that
//! needs no renderer at all.
//!
//! Every renderer gives a mounted element the same `MountedData`, and that
//! covers measuring, scrolling and focus everywhere. The rest of
//! [`ElementApi`] needs the renderer's own handle, which `MountedData` carries
//! as an `Any`: downcasting it yields a `web_sys::Element` on the web and a
//! Blitz `NodeHandle` natively, either of which can answer the whole trait.
//!
//! So there is no id, no selector and no second way in - [`element`] hands
//! back the richest implementation this build can produce, and callers never
//! learn which one they got.

use std::rc::Rc;

use dioxus::prelude::{
    Callback, Element, Event, FocusData, KeyboardData, MountedData, MouseData, PointerData,
    TransitionData,
};

use super::{
    ColorSchemeApi, ContentSubscription, DocumentApi, ElementApi, KeyboardApi, ScrollApi,
    SilentFocusApi, TimerApi,
};

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
mod blitz;
mod mounted;
#[cfg(not(target_arch = "wasm32"))]
mod thread;
#[cfg(target_arch = "wasm32")]
mod web;

/// What HTML counts as interactive content, plus anything a caller made
/// focusable. A label does not forward a click on any of these.
#[cfg(any(target_arch = "wasm32", feature = "native"))]
const INTERACTIVE: &str = "a[href], button, input, select, textarea, summary, \
    [tabindex], [contenteditable]:not([contenteditable=\"false\"])";

/// The richest [`ElementApi`] this renderer can back `mounted` with.
pub(crate) fn element(mounted: &Rc<MountedData>) -> Box<dyn ElementApi> {
    #[cfg(target_arch = "wasm32")]
    if let Some(element) = web::element(mounted) {
        return element;
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    if let Some(element) = blitz::element(mounted) {
        return element;
    }

    Box::new(mounted::MountedElement(mounted.clone()))
}

/// Only the web has a `MutationObserver`. Blitz reports no mutation to user
/// code, and a per-frame subtree walk would cost more than the re-checks it
/// saves - see [`on_content_change`](crate::platform::on_content_change).
pub(crate) fn on_content_change(
    mounted: &Rc<MountedData>,
    callback: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (mounted, callback);
    #[cfg(target_arch = "wasm32")]
    return web::on_content_change(mounted, callback);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// Only the web: Blitz fires no native `submit` or `reset` for a raw `<form>`,
/// so an owner found there would promise what nothing delivers.
pub(crate) fn on_form_reset(
    mounted: &Rc<MountedData>,
    on_reset: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (mounted, on_reset);
    #[cfg(target_arch = "wasm32")]
    return web::on_form_reset(mounted, on_reset);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// The web and Blitz compute `direction`; the WebView floor answers LTR.
pub(crate) fn is_rtl(mounted: &Rc<MountedData>) -> bool {
    #[cfg(not(any(target_arch = "wasm32", feature = "native")))]
    let _ = mounted;
    #[cfg(target_arch = "wasm32")]
    return web::is_rtl(mounted);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::is_rtl(mounted);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return false;
}

/// Whatever the renderer needs mounted at the root, rendered once by
/// `LiberoProvider`. Only Blitz needs anything: an element to reach its
/// document through from the first frame, and one to run the commands it had
/// to defer (see `blitz::Outlet`). Everywhere else this renders nothing.
#[allow(non_snake_case)]
pub(crate) fn Outlet() -> Element {
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::Outlet {} };
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return rsx! {};
}

/// Wraps the app on a renderer that has to watch input from above it; only
/// Blitz does (see `blitz::Listener`). Everywhere else this is `children`.
#[allow(non_snake_case)]
pub(crate) fn Listener(children: Element) -> Element {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::Listener { {children} } };
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return children;
}

/// The portal outlet's box; only Blitz places it (see `blitz::PortalRoot`).
#[allow(non_snake_case)]
pub(crate) fn PortalRoot(children: Element) -> Element {
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::PortalRoot { {children} } };
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return rsx! { div { {children} } };
}

/// Wraps one portaled entry; only Blitz needs a box (see `blitz::PortalEntry`).
/// `idle`: mounted but drawing nothing.
#[allow(non_snake_case)]
pub(crate) fn PortalEntry(children: Element, idle: bool) -> Element {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::PortalEntry { idle, {children} } };
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    {
        let _ = idle;
        children
    }
}

/// Blitz opens no picker for a `<select>`; see [`opens_select_picker`](crate::platform::opens_select_picker).
pub(crate) const OPENS_SELECT_PICKER: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

pub(crate) fn document() -> Option<&'static dyn DocumentApi> {
    #[cfg(target_arch = "wasm32")]
    return web::document();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::document();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return None;
}

/// The web and Blitz can tell Rust what the platform is set to; only the web
/// has somewhere to persist an override - see
/// [`color_scheme`](crate::platform::color_scheme).
pub(crate) fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    #[cfg(target_arch = "wasm32")]
    return web::color_scheme();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::color_scheme();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return None;
}

/// The web and Blitz can report a scroll. Elsewhere this is `None`, which is
/// the house rule for an absent capability - never an `Unsupported` stub.
pub(crate) fn scroll() -> Option<&'static dyn ScrollApi> {
    #[cfg(target_arch = "wasm32")]
    return web::scroll();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::scroll();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return None;
}

/// `scrollIntoView`'s `nearest`, vertically: how far a view spanning
/// `view_top..view_bottom` has to scroll to show `top..bottom`, `None` if it
/// already shows it. A box taller than the view aligns its top.
#[cfg_attr(not(any(target_arch = "wasm32", feature = "native")), allow(dead_code))]
fn nearest_scroll(top: f64, bottom: f64, view_top: f64, view_bottom: f64) -> Option<f64> {
    if top < view_top || bottom - top > view_bottom - view_top {
        Some(top - view_top)
    } else if bottom > view_bottom {
        Some(bottom - view_bottom)
    } else {
        None
    }
}

/// A timer everywhere: the browser's own `setTimeout` on the web, and
/// [`thread`]'s sleeping thread on every other renderer - it needs nothing
/// from the renderer beyond dioxus's own task queue, so Blitz and the WebView
/// floor are both covered by the one arm.
pub(crate) fn timer() -> Option<&'static dyn TimerApi> {
    #[cfg(target_arch = "wasm32")]
    return web::timer();
    #[cfg(not(target_arch = "wasm32"))]
    return thread::timer();
}

/// The web listens on the window; Blitz hears a press bubble out of the app to
/// `blitz::Listener`. The WebView floor and a server have neither.
pub(crate) fn keyboard() -> Option<&'static dyn KeyboardApi> {
    #[cfg(target_arch = "wasm32")]
    return web::keyboard();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::keyboard();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return None;
}

/// Only Blitz moves focus without a focus event - see
/// [`silent_focus`](crate::platform::silent_focus).
pub(crate) fn silent_focus() -> Option<&'static dyn SilentFocusApi> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::silent_focus();
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return None;
}

/// Only Blitz blurs for a press that cancelled its `mousedown` - see
/// [`blur_counts`](crate::platform::blur_counts).
pub(crate) fn press_kept_focus() -> bool {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::press_kept_focus();
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return false;
}

/// Only Blitz rewrites them - see [`focus_selectors`](crate::platform::focus_selectors).
pub(crate) fn focus_selectors(css: &str) -> std::borrow::Cow<'_, str> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::focus_selectors(css);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return std::borrow::Cow::Borrowed(css);
}

/// The web reads the `web_sys` event; every other renderer is asked for the
/// payload the desktop and Android WebView deliver, which carries the property
/// across the IPC. Blitz and a server hand over something else and answer
/// `None` - see [`transition_property`](crate::platform::transition_property)
/// for why this has to be asked of the platform at all.
pub(crate) fn transition_property(event: &Event<TransitionData>) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    return web::transition_property(event);
    #[cfg(not(target_arch = "wasm32"))]
    return {
        use dioxus::html::{HasTransitionData, SerializedTransitionData};
        event
            .downcast::<SerializedTransitionData>()
            .map(HasTransitionData::property_name)
    };
}

/// Only the web can say - see [`focus_visible`](crate::platform::focus_visible).
pub(crate) fn focus_visible(event: &Event<FocusData>) -> Option<bool> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::focus_visible(event);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// Only the web has a document key listener that can take a press ahead of
/// the element handlers - see [`key_taken`](crate::platform::key_taken).
pub(crate) fn key_taken(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::key_taken(event);
    #[cfg(not(target_arch = "wasm32"))]
    return false;
}

/// The web reads the event's target; Blitz the focused node, where it sends a
/// key press - see [`typing_target`](crate::platform::typing_target).
pub(crate) fn typing_target(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::typing_target(event);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::typing_target();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return false;
}

/// As [`typing_target`] - see [`arrow_target`](crate::platform::arrow_target).
pub(crate) fn arrow_target(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::arrow_target(event);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::arrow_target();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return false;
}

/// As [`typing_target`] - see [`rtl_target`](crate::platform::rtl_target).
pub(crate) fn rtl_target(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::rtl_target(event);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::rtl_target();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return false;
}

/// The web reads the click's target, Blitz the press `blitz::Listener` hit -
/// see [`nested_interactive`](crate::platform::nested_interactive).
pub(crate) fn nested_interactive(event: &Event<MouseData>, boundary: &str) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::nested_interactive(event, boundary);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::nested_interactive(boundary);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return {
        let _ = boundary;
        false
    };
}

/// Only Blitz needs it: it has no pointer capture, so `blitz::Listener` hands
/// `capture` the moves and the release that land outside it - see
/// [`follow_pointer`](crate::platform::follow_pointer).
pub(crate) fn follow_pointer(
    event: &Event<PointerData>,
    capture: &Rc<MountedData>,
    onmove: Callback<Event<PointerData>>,
    onup: Callback<Event<PointerData>>,
) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    blitz::follow_pointer(event, capture, onmove, onup);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    let _ = (event, capture, onmove, onup);
}

/// Only the web can see a click's target today - see
/// [`padding_press`](crate::platform::padding_press).
pub(crate) fn padding_press(
    event: &Event<MouseData>,
    boundary: &str,
) -> Option<Box<dyn ElementApi>> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (event, boundary);
    #[cfg(target_arch = "wasm32")]
    return web::padding_press(event, boundary);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// Only the web can see a focus event's `relatedTarget` - see
/// [`focus_entered_from`](crate::platform::focus_entered_from).
pub(crate) fn focus_entered_from(
    event: &Event<FocusData>,
    boundary: &str,
) -> Option<Option<Box<dyn ElementApi>>> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (event, boundary);
    #[cfg(target_arch = "wasm32")]
    return web::focus_entered_from(event, boundary);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// Only the web can see a press's target - see
/// [`focus_pressed`](crate::platform::focus_pressed).
pub(crate) fn focus_pressed(event: &Event<PointerData>, within: &Rc<MountedData>) {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (event, within);
    #[cfg(target_arch = "wasm32")]
    web::focus_pressed(event, within);
}

/// Only Blitz fires no `submit` - see
/// [`emulates_submit`](crate::platform::emulates_submit).
pub(crate) fn emulates_submit() -> bool {
    cfg!(all(not(target_arch = "wasm32"), feature = "native"))
}

pub(crate) fn activated_submitter(form: &Rc<MountedData>) -> bool {
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    let _ = form;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::activated_submitter(form);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return false;
}

pub(crate) fn implicit_submission(form: &Rc<MountedData>) -> bool {
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    let _ = form;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::implicit_submission(form);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return false;
}

pub(crate) fn form_values(form: &Rc<MountedData>) -> Vec<(String, dioxus::html::FormValue)> {
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    let _ = form;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::form_values(form);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return Vec::new();
}

/// Only Blitz renders with its document borrowed - see
/// [`when_free`](crate::platform::when_free).
pub(crate) fn when_free(run: Box<dyn FnOnce()>) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    blitz::when_free(run);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    run();
}

/// Only Blitz lays out after effects run - see
/// [`when_laid_out`](crate::platform::when_laid_out).
pub(crate) fn when_laid_out(run: Box<dyn FnOnce()>) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    blitz::when_laid_out(run);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    run();
}

/// Only the web can read a media query today - see
/// [`prefers_reduced_motion`](crate::platform::prefers_reduced_motion).
pub(crate) fn prefers_reduced_motion() -> bool {
    #[cfg(target_arch = "wasm32")]
    return web::prefers_reduced_motion();
    #[cfg(not(target_arch = "wasm32"))]
    return false;
}

#[cfg(test)]
mod tests {
    use super::nearest_scroll;

    #[test]
    fn nearest_scroll_moves_the_out_of_view_edge_only() {
        assert_eq!(nearest_scroll(20.0, 40.0, 0.0, 100.0), None);
        assert_eq!(nearest_scroll(120.0, 140.0, 0.0, 100.0), Some(40.0));
        assert_eq!(nearest_scroll(-30.0, -10.0, 0.0, 100.0), Some(-30.0));
        // Taller than the view: its top.
        assert_eq!(nearest_scroll(50.0, 250.0, 0.0, 100.0), Some(50.0));
    }
}
