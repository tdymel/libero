use dioxus::prelude::*;

use crate::{context::LiberoContext, localization::Localization};

/// The active [`Localization`]: every string libero shows a reader.
/// Reactive. Panics outside a `LiberoProvider`.
///
/// Docs: <https://libero-ui.dev/about/localization>
pub fn use_localization() -> &'static Localization {
    *use_context::<LiberoContext>().localization.read()
}

/// [`use_localization`] without the hook, for a plain render function that may
/// run conditionally or in a loop. Still subscribes the rendering scope.
pub(crate) fn current_localization() -> &'static Localization {
    *consume_context::<LiberoContext>().localization.read()
}

/// The active [`Localization`], and how to swap it: a language picker's base.
/// Readers re-render; the stylesheet is untouched.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{localization::Localization, use_localization_handle};
/// static GERMAN: Localization = Localization { ..Localization::ENGLISH };
///
/// # fn app() -> Element {
/// let localization = use_localization_handle();
///
/// rsx! {
///     button { onclick: move |_| localization.set(&GERMAN), "Deutsch" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/about/localization>
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
    /// The active localization. Reactive.
    pub fn get(&self) -> &'static Localization {
        *self.localization.read()
    }

    /// Swaps the localization; a no-op for the active one, so nothing re-renders.
    pub fn set(&self, localization: &'static Localization) {
        let mut current = self.localization;
        if !std::ptr::eq(*current.peek(), localization) {
            current.set(localization);
        }
    }
}
