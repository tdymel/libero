use dioxus::prelude::*;

use crate::{context::LiberoContext, localization::Formats};

/// The active [`Formats`]: how dates, times and numbers are written.
/// Reactive. Panics outside a `LiberoProvider`.
///
/// Docs: <https://libero-ui.dev/about/localization>
pub fn use_formats() -> &'static Formats {
    *use_context::<LiberoContext>().formats.read()
}

/// [`use_formats`] without the hook, for a plain render function that may run
/// conditionally or in a loop. Still subscribes the rendering scope.
pub(crate) fn current_formats() -> &'static Formats {
    *consume_context::<LiberoContext>().formats.read()
}

/// The active [`Formats`], and how to swap them: a region picker's base.
/// Independent of the language.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{localization::Formats, use_formats_handle};
/// # fn app() -> Element {
/// let formats = use_formats_handle();
///
/// rsx! {
///     button { onclick: move |_| formats.set(&Formats::GERMAN), "24-hour clock" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/about/localization>
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
    /// The active formats. Reactive.
    pub fn get(&self) -> &'static Formats {
        *self.formats.read()
    }

    /// Swaps the formats; a no-op for the active ones, so nothing re-renders.
    pub fn set(&self, formats: &'static Formats) {
        let mut current = self.formats;
        if !std::ptr::eq(*current.peek(), formats) {
            current.set(formats);
        }
    }
}
