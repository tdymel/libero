use dioxus::prelude::*;

use crate::{context::LiberoContext, localization::Localization};

/// The active [`Localization`]: every string libero shows a reader.
///
/// Reactive: a component reading a label re-renders when the localization is
/// swapped. Panics outside a `LiberoProvider`, as [`use_theme`](super::use_theme).
pub fn use_localization() -> &'static Localization {
    *use_context::<LiberoContext>().localization.read()
}

/// [`use_localization`] without the hook, for a plain render function that may
/// run conditionally or in a loop. Still subscribes the rendering scope.
pub(crate) fn current_localization() -> &'static Localization {
    *consume_context::<LiberoContext>().localization.read()
}

/// The active [`Localization`], and how to swap it - what a language picker is
/// built on. Readers re-render; the theme's stylesheet is untouched.
///
/// ```ignore
/// static GERMAN: Localization = Localization { ..Localization::ENGLISH };
///
/// let localization = use_localization_handle();
/// rsx! {
///     Button { onclick: move |_| localization.set(&GERMAN), "Deutsch" }
/// }
/// ```
pub fn use_localization_handle() -> LocalizationHandle {
    LocalizationHandle {
        localization: use_context::<LiberoContext>().localization,
    }
}

/// What [`use_localization_handle`] hands back.
#[derive(Clone, Copy)]
pub struct LocalizationHandle {
    localization: Signal<&'static Localization>,
}

impl LocalizationHandle {
    pub fn get(&self) -> &'static Localization {
        *self.localization.read()
    }

    /// A no-op when `localization` is the active one, so readers do not
    /// re-render for nothing.
    pub fn set(&self, localization: &'static Localization) {
        let mut current = self.localization;
        if !std::ptr::eq(*current.peek(), localization) {
            current.set(localization);
        }
    }
}
