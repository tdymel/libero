use dioxus::prelude::*;

use crate::{
    context::LiberoContext,
    tokens::{ColorScheme, ColorSchemeSetting},
};

/// The app's colour scheme: what it is set to, what that resolves to, and how
/// to change it. Reactive, the platform's own changes included.
///
/// [`ThemeSwitcher`](crate::components::ThemeSwitcher) is the ready-made switch built on it.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::use_color_scheme;
/// # fn app() -> Element {
/// let scheme = use_color_scheme();
///
/// rsx! {
///     button { onclick: move |_| scheme.toggle(), "Toggle the colour scheme" }
/// }
/// # }
/// ```
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

    /// Pins a scheme, or follows the platform with [`ColorSchemeSetting::System`].
    /// Persisted where the platform can, so a reload keeps it.
    pub fn set(&self, setting: impl Into<ColorSchemeSetting>) {
        self.context.set_color_scheme(setting.into());
    }

    /// Flips to the other scheme. Flipping back to the platform's scheme
    /// follows the platform again, so a toggle never strands the app on a pin.
    pub fn toggle(&self) {
        let target = self.resolved().flipped();
        if target == *self.context.system_scheme.peek() {
            self.set(ColorSchemeSetting::System);
        } else {
            self.set(target);
        }
    }

    /// Steps through all three settings: the scheme the platform is *not*
    /// showing, the one it is, then following the platform again.
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

    /// Selects a theme beyond the pair, by its [`ThemeSet::named`](crate::theme::ThemeSet::named)
    /// name. Rebuilds the sheet, which holds only the pair.
    pub fn set_theme(&self, name: &'static str) {
        self.context.set_active_theme(name);
    }
}
