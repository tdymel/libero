use std::rc::Rc;

use dioxus::prelude::*;

use super::{CssLayer, stylesheet_registry::StylesheetRegistry};
use crate::{
    localization::{Formats, Localization},
    platform::{
        A11yAnswers, a11y_media, apply_direction, clear_root_direction, color_scheme, document,
        forget_direction, set_current_a11y_answers, store_direction,
    },
    theme::{THEME_ATTRIBUTE, Theme, ThemeSet, theme_sheet_css},
    tokens::{AccessibilityPreferences, ColorScheme, ColorSchemeSetting, Direction},
    utils::warn,
};

/// The app-wide state [`LiberoProvider`](crate::LiberoProvider) provides: themes, localization, formats.
/// Most apps reach it through the hooks (`use_theme`, `use_localization`, ...).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::LiberoContext;
/// # fn app() -> Element {
/// let context = use_context::<LiberoContext>();
///
/// rsx! {
///     // A pair name pins the colour scheme, as `use_color_scheme().set(ColorScheme::Dark)` does.
///     button { onclick: move |_| context.set_active_theme("dark"), "Dark" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/about/theming>
#[derive(Clone)]
pub struct LiberoContext {
    /// Every theme the app ships, with its light and dark pair. Swappable at runtime.
    pub themes: Signal<ThemeSet>,
    /// The active theme. Reactive, so values read off it stay in step with its CSS vars.
    pub theme: Signal<&'static Theme>,
    /// Every string libero shows a reader. A switch leaves the theme's sheet alone.
    pub localization: Signal<&'static Localization>,
    /// How dates, times and numbers are written, apart from the language.
    pub formats: Signal<&'static Formats>,
    /// The active theme's name: tells a repeat from a change, the pair from a rebuild.
    pub(crate) active: Signal<&'static str>,
    /// What the app asked for, before it is resolved against `system_scheme`.
    pub(crate) scheme_setting: Signal<ColorSchemeSetting>,
    /// What the platform is set to; `Light` where this build cannot tell.
    pub(crate) system_scheme: Signal<ColorScheme>,
    /// Which way the text runs, as the document root's `dir` says.
    pub(crate) direction: Signal<Direction>,
    /// The direction chosen and kept, `None` when nothing was chosen.
    pub(crate) kept_direction: Signal<Option<Direction>>,
    /// The provider's `direction` prop, what clearing the choice goes back to.
    pub(crate) start_direction: Option<Direction>,
    /// What the platform's accessibility settings say, and the app's forced reduced motion.
    pub(crate) accessibility_system: Signal<AccessibilityPreferences>,
    pub(crate) forced_reduced_motion: Signal<Option<bool>>,
    pub(crate) layer_order_css: &'static str,
    /// `Rc<str>`, not a `Stylesheet`: every `use_context` clones this struct,
    /// and only `ThemeStyle` reads this field.
    pub(crate) theme_css: Signal<Rc<str>>,
    pub(crate) stylesheet_registry: StylesheetRegistry,
    pub(crate) stylesheet_registry_version: Signal<u64>,
}

impl LiberoContext {
    /// Only `LiberoProvider` builds one; callers reach it through `use_context`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        themes: Signal<ThemeSet>,
        theme: Signal<&'static Theme>,
        localization: Signal<&'static Localization>,
        formats: Signal<&'static Formats>,
        active: Signal<&'static str>,
        scheme_setting: Signal<ColorSchemeSetting>,
        system_scheme: Signal<ColorScheme>,
        direction: Signal<Direction>,
        kept_direction: Signal<Option<Direction>>,
        start_direction: Option<Direction>,
        accessibility_system: Signal<AccessibilityPreferences>,
        forced_reduced_motion: Signal<Option<bool>>,
        theme_css: Signal<Rc<str>>,
        stylesheet_registry: StylesheetRegistry,
        stylesheet_registry_version: Signal<u64>,
    ) -> Self {
        Self {
            themes,
            theme,
            localization,
            formats,
            active,
            scheme_setting,
            system_scheme,
            direction,
            kept_direction,
            start_direction,
            accessibility_system,
            forced_reduced_motion,
            layer_order_css: CssLayer::order_css(),
            theme_css,
            stylesheet_registry,
            stylesheet_registry_version,
        }
    }

    /// Switches to the theme `name` selects. An unknown name warns and changes nothing.
    ///
    /// A name of the light/dark pair pins that colour scheme, as
    /// `use_color_scheme().set` does, kept across a reload; anything else rebuilds the sheet.
    pub fn set_active_theme(&self, name: &'static str) {
        let Some(theme) = self.themes.peek().get(name) else {
            warn(&format!(
                "unknown theme `{name}`; the set has no such theme, so nothing changed"
            ));
            return;
        };

        if self.themes.peek().is_in_pair(name) {
            let scheme = match name == ColorScheme::Dark.as_str() {
                true => ColorScheme::Dark,
                false => ColorScheme::Light,
            };
            self.set_color_scheme(scheme.into());
            return;
        }

        let (mut active, mut current, mut css) = (self.active, self.theme, self.theme_css);
        if *active.peek() == name {
            return;
        }
        css.set(theme_sheet_css(theme, None));
        active.set(name);
        current.set(theme);
    }

    /// Pins one scheme of the pair, or hands the choice back to the platform.
    ///
    /// Writes the attribute even when that theme already shows: else "dark"
    /// clicked on a dark platform would leave the page following the platform.
    pub(crate) fn set_color_scheme(&self, setting: ColorSchemeSetting) {
        self.apply_scheme(setting, !self.shows_pair());
    }

    /// `stale`: the sheet shows no pair of this set, so a carried scheme rebuilds it too.
    fn apply_scheme(&self, setting: ColorSchemeSetting, stale: bool) {
        // No dark half: follow the system rather than refuse, since a picker
        // swapping sets while dark is pinned also lands here.
        let setting = match setting.fixed() {
            Some(scheme) if self.themes.peek().get(scheme.as_str()).is_none() => {
                warn(&format!(
                    "no `{}` theme in this set, so the colour scheme follows the platform",
                    scheme.as_str()
                ));
                ColorSchemeSetting::System
            }
            _ => setting,
        };

        let mut stored = self.scheme_setting;
        stored.set(setting);
        if let Some(platform) = color_scheme() {
            platform.store(setting);
        }

        match setting.fixed() {
            Some(scheme) => self.pin_scheme(scheme, stale),
            None => self.show_system(*self.system_scheme.peek(), stale),
        }
    }

    fn pin_scheme(&self, scheme: ColorScheme, stale: bool) {
        let name = scheme.as_str();
        let Some(theme) = self.themes.peek().get(name) else {
            return;
        };

        let carried = document()
            .is_some_and(|document| document.set_root_attribute(THEME_ATTRIBUTE, Some(name)));
        let mut css = self.theme_css;
        if !carried {
            css.set(theme_sheet_css(theme, None));
        } else if stale {
            // A named theme's sheet has no attribute blocks to select (todo 1837).
            css.set(self.themes.peek().sheet_css());
        }

        let (mut active, mut current) = (self.active, self.theme);
        active.set(name);
        current.set(theme);
    }

    /// Hands the choice back to the platform: no root attribute, so the sheet's
    /// media block decides. The Rust-side theme still follows `scheme`.
    pub(crate) fn follow_system(&self, scheme: ColorScheme) {
        self.show_system(scheme, !self.shows_pair());
    }

    fn show_system(&self, scheme: ColorScheme, stale: bool) {
        let name = scheme.as_str();
        let themes = self.themes.peek().clone();
        let theme = themes.get(name).unwrap_or_else(|| themes.light_theme());

        let cleared =
            document().is_some_and(|document| document.set_root_attribute(THEME_ATTRIBUTE, None));
        if !cleared || stale {
            // No attribute to clear: rebuild with the whole set, whose media
            // block follows the system on its own.
            let mut css = self.theme_css;
            css.set(themes.sheet_css());
        }

        let (mut active, mut current) = (self.active, self.theme);
        active.set(name);
        current.set(theme);
    }

    /// Whether the sheet is the pair's, not one a theme beyond it rebuilt.
    fn shows_pair(&self) -> bool {
        self.themes.peek().is_in_pair(*self.active.peek())
    }

    /// Turns the app's text, on the document root, and keeps the choice.
    pub(crate) fn set_direction(&self, direction: Direction) {
        let mut stored = self.direction;
        stored.set(direction);
        apply_direction(direction);
        store_direction(direction);
        let mut kept = self.kept_direction;
        kept.set(Some(direction));
    }

    /// Drops the kept choice: back to the provider's `direction`, or no root `dir`.
    pub(crate) fn clear_direction(&self) {
        let mut stored = self.direction;
        stored.set(self.start_direction.unwrap_or_default());
        match self.start_direction {
            Some(direction) => apply_direction(direction),
            None => clear_root_direction(),
        }
        forget_direction();
        let mut kept = self.kept_direction;
        kept.set(None);
    }

    /// What libero writes into its sheets for the accessibility media features. Reactive.
    pub(crate) fn a11y_answers(&self) -> A11yAnswers {
        A11yAnswers::new(
            *self.accessibility_system.read(),
            *self.forced_reduced_motion.read(),
        )
    }

    /// Forces reduced motion, or with `None` follows the system, and keeps the choice.
    pub(crate) fn set_forced_reduced_motion(&self, reduced: Option<bool>) {
        if let Some(platform) = a11y_media() {
            platform.store_reduced_motion(reduced);
        }
        self.force_reduced_motion(reduced);
    }

    pub(super) fn force_reduced_motion(&self, reduced: Option<bool>) {
        let mut stored = self.forced_reduced_motion;
        stored.set(reduced);
        self.publish_accessibility();
    }

    pub(super) fn set_accessibility_system(&self, system: AccessibilityPreferences) {
        let mut stored = self.accessibility_system;
        stored.set(system);
        self.publish_accessibility();
    }

    /// Hands the answers to motion started from Rust and bumps the sheets'
    /// version, so `SheetWatch` sees the rewritten text.
    fn publish_accessibility(&self) {
        set_current_a11y_answers(A11yAnswers::new(
            *self.accessibility_system.peek(),
            *self.forced_reduced_motion.peek(),
        ));
        let mut version = self.stylesheet_registry_version;
        version += 1;
    }

    /// Swaps the whole set, as a theme picker does. Rebuilds the sheet and keeps
    /// the colour-scheme setting, so a pinned dark stays dark.
    pub fn set_theme_set(&self, themes: ThemeSet) {
        let mut current_set = self.themes;
        current_set.set(themes);

        // Read out first: a `peek()` guard held across `apply_scheme` is a borrow panic.
        let setting = *self.scheme_setting.peek();
        // Re-applied, as the new set may have no dark half; it builds the one sheet (todo 2087).
        self.apply_scheme(setting, true);
    }
}
