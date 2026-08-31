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

use dioxus::prelude::{Event, MountedData, TransitionData};

use super::{DocumentApi, ElementApi, KeyboardApi, ScrollApi, TimerApi};

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

pub(crate) fn document() -> Option<Box<dyn DocumentApi>> {
    #[cfg(target_arch = "wasm32")]
    return web::document();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::document();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return None;
}

/// Only the web can report a scroll today. Off it this is `None`, which is the
/// house rule for an absent capability - never an `Unsupported` stub.
pub(crate) fn scroll() -> Option<Box<dyn ScrollApi>> {
    #[cfg(target_arch = "wasm32")]
    return web::scroll();
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

<<<<<<< HEAD
/// A timer everywhere: the browser's own `setTimeout` on the web, and
/// [`thread`]'s sleeping thread on every other renderer - it needs nothing
/// from the renderer beyond dioxus's own task queue, so Blitz and the WebView
/// floor are both covered by the one arm.
pub(crate) fn timer() -> Option<Box<dyn TimerApi>> {
    #[cfg(target_arch = "wasm32")]
    return web::timer();
    #[cfg(not(target_arch = "wasm32"))]
    return thread::timer();
}

/// Only the web can report a document-level key press today, for the same
/// reason [`scroll`] cannot: the notification has to come from the renderer,
/// and Blitz's would be fork work.
pub(crate) fn keyboard() -> Option<Box<dyn KeyboardApi>> {
    #[cfg(target_arch = "wasm32")]
    return web::keyboard();
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// Only the web can name a finished transition's property today - see
/// [`transition_property`](crate::platform::transition_property) for why this
/// has to be asked of the platform at all.
=======
>>>>>>> 9739a68 (fix(hooks): correct the focus-return rationale and tighten the presence API)
pub(crate) fn transition_property(event: &Event<TransitionData>) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::transition_property(event);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}
