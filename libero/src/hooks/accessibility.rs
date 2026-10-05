use dioxus::prelude::*;

use crate::{
    context::LiberoContext,
    tokens::{AccessibilityPreferences, Contrast},
};

/// The reader's accessibility settings (reduced motion, forced colours,
/// contrast, reduced transparency), and an app's own reduced-motion switch.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Switch;
/// # use libero::hooks::use_accessibility;
/// # fn app() -> Element {
/// let accessibility = use_accessibility();
/// let reduced = accessibility.reduced_motion();
///
/// rsx! {
///     Switch {
///         label: "Reduce motion",
///         checked: reduced,
///         onchange: move |reduce: bool| accessibility.set_reduced_motion(Some(reduce)),
///     }
/// }
/// # }
/// ```
///
/// A forced reduced motion reaches libero's own CSS, not a `<style>` the app adds itself.
/// It applies page-wide, set under a nested `LiberoProvider` too. Panics outside a `LiberoProvider`.
///
/// Docs: <https://libero-ui.dev/accessibility/use-accessibility>
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
    /// All four settings at once, a forced reduced motion included. Reactive.
    pub fn get(&self) -> AccessibilityPreferences {
        AccessibilityPreferences {
            reduced_motion: self.reduced_motion(),
            ..*self.context.accessibility_system.read()
        }
    }

    /// Whether motion is reduced: the forced answer, else the system's. Reactive.
    pub fn reduced_motion(&self) -> bool {
        self.context
            .forced_reduced_motion
            .read()
            .unwrap_or(self.context.accessibility_system.read().reduced_motion)
    }

    /// Forces reduced motion on or off; `None` follows the system again. Kept in local
    /// storage where it persists (see `set_storage_dir`), so a restart keeps it.
    pub fn set_reduced_motion(&self, reduced: Option<bool>) {
        self.context.set_forced_reduced_motion(reduced);
    }

    /// Whether the system forces its own colours (`forced-colors: active`). Reactive.
    pub fn forced_colors(&self) -> bool {
        self.context.accessibility_system.read().forced_colors
    }

    /// Which way the system asks the contrast to go. Reactive.
    pub fn contrast(&self) -> Contrast {
        self.context.accessibility_system.read().contrast
    }

    /// Whether the system asks for less transparency. Reactive.
    pub fn reduced_transparency(&self) -> bool {
        self.context
            .accessibility_system
            .read()
            .reduced_transparency
    }
}
