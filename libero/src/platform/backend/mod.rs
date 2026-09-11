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

use super::{ColorSchemeApi, DocumentApi, ElementApi, KeyboardApi, ScrollApi, TimerApi};

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

/// The portal outlet's `style`; only Blitz sets one (see `blitz::PORTAL_ROOT_STYLE`).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub(crate) const PORTAL_ROOT_STYLE: Option<&str> = Some(blitz::PORTAL_ROOT_STYLE);
#[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
pub(crate) const PORTAL_ROOT_STYLE: Option<&str> = None;

/// Wraps one portaled entry; only Blitz needs a box (see `blitz::PortalEntry`).
#[allow(non_snake_case)]
pub(crate) fn PortalEntry(children: Element) -> Element {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::PortalEntry { {children} } };
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return children;
}

/// Blitz opens no picker for a `<select>`; see [`select_picker`](crate::platform::select_picker).
pub(crate) const SELECT_PICKER: bool = !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

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
