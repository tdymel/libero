use dioxus::prelude::*;

use crate::{
    context::LiberoContext,
    tokens::{ColorScheme, ColorSchemeSetting},
};

/// The app's colour scheme: what it is set to, what that resolves to, and how
/// to change it.
///
/// A hook rather than a component, deliberately: the library ships no toggle.
/// An app builds its own out of a `Switch`, an `ActionIcon` or a
/// `SegmentedControl`, because only the app knows where that control belongs
/// and what it should look like.
///
/// ```ignore
/// let scheme = use_color_scheme();
///
/// rsx! {
///     ActionIcon {
///         onclick: move |_| scheme.toggle(),
///         aria_label: match scheme.resolved() {
///             ColorScheme::Dark => "Switch to the light theme",
///             ColorScheme::Light => "Switch to the dark theme",
///         },
///     }
/// }
/// ```
///
/// Reactive: every read below subscribes, so a component that shows the
/// scheme re-renders when it changes - including when the *platform* changes
/// its mind while the app is following it.
pub fn use_color_scheme() -> ColorSchemeHandle {
    ColorSchemeHandle {
        context: use_context::<LiberoContext>(),
    }
}

/// What [`use_color_scheme`] hands back.
#[derive(Clone)]
pub struct ColorSchemeHandle {
    context: LiberoContext,
}

impl ColorSchemeHandle {
    /// What the app asked for, which is [`ColorSchemeSetting::System`] until
    /// something pins it.
    pub fn setting(&self) -> ColorSchemeSetting {
        *self.context.scheme_setting.read()
    }

    /// Which scheme is actually on screen.
    pub fn resolved(&self) -> ColorScheme {
        self.setting().resolve(*self.context.system_scheme.read())
    }

    /// Pins a scheme, or hands the choice back to the platform with
    /// [`ColorSchemeSetting::System`]. Persisted where the platform has
    /// somewhere to persist it, so a reload comes back to the same scheme.
    pub fn set(&self, setting: impl Into<ColorSchemeSetting>) {
        self.context.set_color_scheme(setting.into());
    }

    /// Flips to the other scheme, and pins it. A toggle is an explicit
    /// choice - following the platform again is [`set`](Self::set) with
    /// [`ColorSchemeSetting::System`].
    pub fn toggle(&self) {
        self.set(self.resolved().flipped());
    }

    /// Selects a theme beyond the pair, by the name it was given to
    /// [`ThemeSet::named`](crate::theme::ThemeSet::named).
    ///
    /// Such a theme is not in the emitted sheet, so this rebuilds it - the
    /// accepted cost of not growing every app's sheet with every theme it
    /// happens to own.
    pub fn set_theme(&self, name: &'static str) {
        self.context.set_active_theme(name);
    }
}
