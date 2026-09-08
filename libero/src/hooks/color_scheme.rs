use dioxus::prelude::*;

use crate::{
    context::LiberoContext,
    tokens::{ColorScheme, ColorSchemeSetting},
};

/// The app's colour scheme: what it is set to, what that resolves to, and how
/// to change it.
///
/// [`ColorSchemeButton`](crate::components::ColorSchemeButton) is the
/// ready-made switch built on it. Anything else - a `Switch`, or a
/// `SegmentedControl` that offers "follow the system" - is built here.
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

    /// Flips to the other scheme. It pins that scheme only while it differs
    /// from the platform's: flipping back to what the platform says hands
    /// the choice back to it, so a toggle can never strand the app on a pin
    /// that ignores the system - or a devtools emulation of it - for good.
    pub fn toggle(&self) {
        let target = self.resolved().flipped();
        if target == *self.context.system_scheme.peek() {
            self.set(ColorSchemeSetting::System);
        } else {
            self.set(target);
        }
    }

    /// Steps through all three settings: from following the platform to
    /// the scheme it is *not* showing, then to the one it is, then back to
    /// following it. Every press changes what is on screen except the last,
    /// which changes what the app listens to - so a reader can always get
    /// back to the platform's choice, and to a devtools emulation of it.
    pub fn cycle(&self) {
        self.set(self.next_in_cycle());
    }

    /// The setting [`cycle`](Self::cycle) would move to next.
    pub fn next_in_cycle(&self) -> ColorSchemeSetting {
        let system = *self.context.system_scheme.read();
        match self.setting().fixed() {
            None => system.flipped().into(),
            Some(pinned) if pinned != system => system.into(),
            Some(_) => ColorSchemeSetting::System,
        }
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
