//! One module per renderer, plus [`mounted`] - the portable floor.
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

use dioxus::prelude::MountedData;

use super::{DocumentApi, ElementApi, ScrollApi};

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
mod blitz;
mod mounted;
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
