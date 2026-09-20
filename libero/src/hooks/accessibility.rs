use dioxus::prelude::*;

use crate::{
    context::LiberoContext,
    tokens::{AccessibilityOverrides, AccessibilityPreferences},
};

/// The reader's accessibility preferences (`prefers-reduced-motion`,
/// `forced-colors`, `prefers-contrast`, `prefers-reduced-transparency`), and the
/// app's own answers over them, for a settings page.
///
/// ```ignore
/// let accessibility = use_accessibility();
///
/// rsx! {
///     Switch {
///         label: "Reduce motion",
///         checked: accessibility.get().reduced_motion,
///         onchange: move |reduce: bool| accessibility.set_overrides(AccessibilityOverrides {
///             reduced_motion: Some(reduce),
///             ..accessibility.overrides()
///         }),
///     }
/// }
/// ```
///
/// The overrides act on native renderers only: on the web the browser's media
/// queries decide, and [`set_overrides`](AccessibilityHandle::set_overrides)
/// changes nothing on screen.
pub fn use_accessibility() -> AccessibilityHandle {
    AccessibilityHandle {
        context: use_context::<LiberoContext>(),
    }
}

/// What [`use_accessibility`] hands back.
#[derive(Clone)]
pub struct AccessibilityHandle {
    context: LiberoContext,
}

impl AccessibilityHandle {
    /// What the page answers: the system's settings with the overrides on top.
    /// Reactive. On the web, the browser's answers.
    pub fn get(&self) -> AccessibilityPreferences {
        if crate::platform::answers_a11y_media() {
            self.context.accessibility()
        } else {
            *self.context.accessibility_system.read()
        }
    }

    /// The platform's own settings, without the overrides. Reactive.
    pub fn system(&self) -> AccessibilityPreferences {
        *self.context.accessibility_system.read()
    }

    /// The app's answers, as set on `LiberoProvider` or through
    /// [`set_overrides`](Self::set_overrides). Reactive.
    pub fn overrides(&self) -> AccessibilityOverrides {
        *self.context.accessibility_overrides.read()
    }

    /// Replaces the app's answers; a `None` field follows the system again.
    /// Lives for the session. Native renderers only.
    pub fn set_overrides(&self, overrides: AccessibilityOverrides) {
        self.context.set_accessibility_overrides(overrides);
    }
}
