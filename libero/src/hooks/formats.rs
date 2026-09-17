use dioxus::prelude::*;

use crate::{context::LiberoContext, localization::Formats};

/// The active [`Formats`]: how dates, times and numbers are written.
///
/// Reactive: a reader re-renders when the formats are swapped. Panics outside
/// a `LiberoProvider`, as [`use_theme`](super::use_theme).
pub fn use_formats() -> &'static Formats {
    *use_context::<LiberoContext>().formats.read()
}

/// [`use_formats`] without the hook, for a plain render function that may run
/// conditionally or in a loop. Still subscribes the rendering scope.
pub(crate) fn current_formats() -> &'static Formats {
    *consume_context::<LiberoContext>().formats.read()
}

/// The active [`Formats`], and how to swap them - what a region picker is
/// built on. Independent of the language.
///
/// ```ignore
/// let formats = use_formats_handle();
/// rsx! {
///     Button { onclick: move |_| formats.set(&Formats::GERMAN), "24-hour clock" }
/// }
/// ```
pub fn use_formats_handle() -> FormatsHandle {
    FormatsHandle {
        formats: use_context::<LiberoContext>().formats,
    }
}

/// What [`use_formats_handle`] hands back.
#[derive(Clone, Copy)]
pub struct FormatsHandle {
    formats: Signal<&'static Formats>,
}

impl FormatsHandle {
    pub fn get(&self) -> &'static Formats {
        *self.formats.read()
    }

    /// A no-op when `formats` are the active ones, so readers do not
    /// re-render for nothing.
    pub fn set(&self, formats: &'static Formats) {
        let mut current = self.formats;
        if !std::ptr::eq(*current.peek(), formats) {
            current.set(formats);
        }
    }
}
