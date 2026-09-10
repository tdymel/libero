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
    Element, Event, FocusData, KeyboardData, MountedData, MouseData, PointerData, TransitionData,
};

use super::{ColorSchemeApi, DocumentApi, ElementApi, KeyboardApi, ScrollApi, TimerApi};

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
mod blitz;
mod mounted;
#[cfg(not(target_arch = "wasm32"))]
mod thread;
#[cfg(target_arch = "wasm32")]
mod web;

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

/// Only the web can report a scroll today. Off it this is `None`, which is the
/// house rule for an absent capability - never an `Unsupported` stub.
pub(crate) fn scroll() -> Option<&'static dyn ScrollApi> {
    #[cfg(target_arch = "wasm32")]
    return web::scroll();
    #[cfg(not(target_arch = "wasm32"))]
    return None;
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

/// Only the web can report a document-level key press today, for the same
/// reason [`scroll`] cannot: the notification has to come from the renderer,
/// and Blitz's would be fork work.
pub(crate) fn keyboard() -> Option<&'static dyn KeyboardApi> {
    #[cfg(target_arch = "wasm32")]
    return web::keyboard();
    #[cfg(not(target_arch = "wasm32"))]
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

/// Only the web can see a key press's target today - see
/// [`typing_target`](crate::platform::typing_target).
pub(crate) fn typing_target(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::typing_target(event);
    #[cfg(not(target_arch = "wasm32"))]
    return false;
}

/// Only the web can see a key press's target today - see
/// [`arrow_target`](crate::platform::arrow_target).
pub(crate) fn arrow_target(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::arrow_target(event);
    #[cfg(not(target_arch = "wasm32"))]
    return false;
}

/// Only the web can see a click's target today - see
/// [`nested_interactive`](crate::platform::nested_interactive).
pub(crate) fn nested_interactive(event: &Event<MouseData>, boundary: &str) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (event, boundary);
    #[cfg(target_arch = "wasm32")]
    return web::nested_interactive(event, boundary);
    #[cfg(not(target_arch = "wasm32"))]
    return false;
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

/// Only the web can read a media query today - see
/// [`prefers_reduced_motion`](crate::platform::prefers_reduced_motion).
pub(crate) fn prefers_reduced_motion() -> bool {
    #[cfg(target_arch = "wasm32")]
    return web::prefers_reduced_motion();
    #[cfg(not(target_arch = "wasm32"))]
    return false;
}
