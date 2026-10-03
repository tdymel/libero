use std::rc::Rc;

use dioxus::prelude::*;

use stylesheet_registry::StylesheetRegistry;

pub(crate) use stylesheet_registry::{SheetRank, StylesheetKey};

use super::{ModalHost, PortalHost, PortalOutlet, WindowHost, window::ZLayers};
use crate::{
    css::Stylesheet,
    localization::{Formats, Localization},
    platform::{
        self, A11yAnswers, a11y_media, answer_a11y_media, answers_a11y_media, apply_direction,
        clear_root_direction, color_scheme, document, focus_selectors, forget_direction,
        set_current_a11y_answers, set_root_direction, store_direction, stored_direction,
    },
    theme::{THEME_ATTRIBUTE, Theme, ThemeSet, physical_text_align, themed_form_controls},
    tokens::{AccessibilityPreferences, ColorScheme, ColorSchemeSetting, Direction},
    utils::warn,
};

mod css_layer;
mod stylesheet_registry;

pub(crate) use css_layer::CssLayer;

/// The app-wide state [`LiberoProvider`] provides: themes, localization, formats.
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
            stylesheet_registry: StylesheetRegistry::new(),
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
        css.set(Rc::from(Stylesheet::from(theme).as_str()));
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
            css.set(Rc::from(Stylesheet::from(theme).as_str()));
        } else if stale {
            // A named theme's sheet has no attribute blocks to select (todo 1837).
            css.set(Rc::from(Stylesheet::from(&*self.themes.peek()).as_str()));
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
            css.set(Rc::from(Stylesheet::from(&themes).as_str()));
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

    fn force_reduced_motion(&self, reduced: Option<bool>) {
        let mut stored = self.forced_reduced_motion;
        stored.set(reduced);
        self.publish_accessibility();
    }

    fn set_accessibility_system(&self, system: AccessibilityPreferences) {
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

/// The theme's own `<style>`, its own component so a rebuilt sheet re-renders
/// this leaf instead of everything under `{children}`.
#[component]
fn ThemeStyle() -> Element {
    let context = use_context::<LiberoContext>();
    let sheet = context.theme_css.read().clone();
    let css = focus_selectors(&sheet);
    // Bumped per rebuilt sheet, for `SheetWatch` (todo 871).
    let version = use_hook(|| Rc::new(std::cell::Cell::new((0u64, sheet.clone()))));
    let (mut count, last) = version.take();
    if !Rc::ptr_eq(&last, &sheet) {
        count += 1;
    }
    version.set((count, sheet.clone()));

    // Every rebuilt sheet after the first, once it is in the document.
    let theme_css = context.theme_css;
    let mounted = use_hook(|| Rc::new(std::cell::Cell::new(false)));
    use_effect(move || {
        theme_css.read();
        if mounted.replace(true)
            && let Some(document) = document()
        {
            document.colors_changed();
        }
    });

    let physical = use_hook(|| (!platform::aligns_logical_text()).then(physical_text_align));
    let form_controls = use_hook(|| (!platform::colors_form_controls()).then(themed_form_controls));

    rsx! {
        style {
            dangerous_inner_html: "{css}"
        }
        if let Some(physical) = physical {
            style { dangerous_inner_html: "{physical}" }
        }
        if let Some(form_controls) = form_controls {
            style { dangerous_inner_html: "{form_controls}" }
        }
        {platform::SheetWatch(count)}
    }
}

/// The `<style>` nodes for everything registered so far. A leaf, so registering
/// re-renders only it; after `{children}`, so the first pass has every sheet.
#[component]
fn StyleOutlet() -> Element {
    let context = use_context::<LiberoContext>();
    let registry_version = *context.stylesheet_registry_version.read();
    // Blitz's stylo and a forced reduced motion need libero's answers in the text (todo 954).
    let answers = context.a11y_answers();

    rsx! {
        if let Some(properties) = platform::scroll_padding_properties() {
            style { dangerous_inner_html: properties }
        }
        for (node_key, stylesheet) in context.stylesheet_registry.stylesheets() {
            style {
                key: "{node_key}",
                dangerous_inner_html: "{renderer_css(&stylesheet, &answers)}"
            }
        }
        {platform::SheetWatch(registry_version)}
    }
}

/// `css` as this renderer matches it.
fn renderer_css(css: &str, answers: &A11yAnswers) -> String {
    answer_a11y_media(&focus_selectors(css), answers).into_owned()
}

/// The root every libero app renders once: themes, localization, stylesheets,
/// portals and the modal stack.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{LiberoProvider, localization::{Formats, Localization}};
/// # fn app() -> Element {
/// // English words, German dates: `14. September 2026`, `15:30`.
/// rsx! {
///     LiberoProvider {
///         localization: &Localization::ENGLISH,
///         formats: &Formats::GERMAN,
///         "Hello"
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/about/getting-started>
#[component]
pub fn LiberoProvider(
    /// Every theme the app ships; the light and dark halves share one sheet.
    /// A lone `&'static Theme` is a set of one. Default [`ThemeSet::DEFAULT`].
    #[props(default, into)]
    themes: ThemeSet,
    /// Every string libero shows a reader. Read at mount; switch it later with
    /// [`use_localization_handle`](crate::hooks::use_localization_handle).
    #[props(default = &Localization::ENGLISH)]
    localization: &'static Localization,
    /// How dates, times and numbers are written, whatever the language. Read at
    /// mount; switch it later with [`use_formats_handle`](crate::hooks::use_formats_handle).
    #[props(default = &Formats::AMERICAN)]
    formats: &'static Formats,
    /// The start text direction, set as the root's `dir`. A choice made through
    /// [`use_direction`](crate::hooks::use_direction) is kept on the web and wins.
    #[props(default, into)]
    direction: Option<Direction>,
    children: Element,
) -> Element {
    let themes = use_hook(|| themes.clone());
    // Only the outermost provider owns the root's `lang`: two would race for it.
    let outermost = use_hook(|| try_consume_context::<LiberoContext>().is_none());
    let localization = use_signal(|| localization);
    let formats = use_signal(|| formats);
    // Read at mount, so the first render paints the kept scheme, not a light flash.
    let setting = use_hook(|| {
        color_scheme()
            .and_then(|platform| platform.stored())
            .unwrap_or_default()
    });
    let system =
        use_hook(|| color_scheme().map_or(ColorScheme::Light, |platform| platform.system()));
    let scheme = setting.resolve(system);

    let scheme_setting = use_signal(|| setting);
    let system_scheme = use_signal(|| system);
    let active = use_signal(|| scheme.as_str());
    let theme_signal = use_signal({
        let themes = themes.clone();
        move || {
            themes
                .get(scheme.as_str())
                .unwrap_or_else(|| themes.light_theme())
        }
    });
    let theme_css = use_signal({
        let themes = themes.clone();
        move || Rc::<str>::from(Stylesheet::from(&themes).as_str())
    });
    let theme_set = use_signal(|| themes);
    // Set before the first render where the root is in reach (the web).
    let kept_direction = use_signal(stored_direction);
    let start_direction = use_hook(|| direction);
    let chosen_direction = use_hook(|| kept_direction.peek().or(start_direction));
    let direction_set = use_hook(|| chosen_direction.is_some_and(set_root_direction));
    let direction_signal = use_signal(|| chosen_direction.unwrap_or_default());

    // Natively at once, so the first frame already answers; on the web in an
    // effect below, as a server cannot know it.
    let accessibility_system = use_signal(|| {
        a11y_media()
            .filter(|_| answers_a11y_media())
            .map(|platform| platform.system())
            .unwrap_or_default()
    });
    // A kept choice. The web reads it after mount: it rewrites the sheets' text,
    // which a hydrating client must render as the server did.
    let forced_reduced_motion = use_signal(|| {
        a11y_media()
            .filter(|_| !cfg!(target_arch = "wasm32"))
            .and_then(|platform| platform.stored_reduced_motion())
    });

    let stylesheet_registry_version = use_signal(|| 0u64);
    let context = use_context_provider(|| {
        LiberoContext::new(
            theme_set,
            theme_signal,
            localization,
            formats,
            active,
            scheme_setting,
            system_scheme,
            direction_signal,
            kept_direction,
            start_direction,
            accessibility_system,
            forced_reduced_motion,
            theme_css,
            stylesheet_registry_version,
        )
    });
    use_hook(|| {
        set_current_a11y_answers(A11yAnswers::new(
            *accessibility_system.peek(),
            *forced_reduced_motion.peek(),
        ))
    });
    // Every later change of the platform's settings, held for the provider's lifetime.
    let _accessibility_subscription = use_hook(|| {
        let context = context.clone();
        Rc::new(a11y_media().map(|platform| {
            platform.on_change(Box::new(move |system| {
                context.set_accessibility_system(system)
            }))
        }))
    });
    use_effect({
        let context = context.clone();
        move || {
            if let Some(platform) = a11y_media().filter(|_| !answers_a11y_media()) {
                context.set_accessibility_system(platform.system());
                // A choice made during the first render wins over the kept one.
                if cfg!(target_arch = "wasm32")
                    && context.forced_reduced_motion.peek().is_none()
                    && let Some(kept) = platform.stored_reduced_motion()
                {
                    context.force_reduced_motion(Some(kept));
                }
            }
        }
    });
    // Elsewhere after mount: Blitz reaches its root through an element the provider renders.
    // The latest choice, not the mount's: a `set` during the first render found no root (956).
    let kept = context.kept_direction;
    use_effect(move || {
        let current = kept.peek().or(start_direction);
        if let Some(direction) = current.filter(|_| !direction_set || current != chosen_direction) {
            apply_direction(direction);
        }
    });

    // After mount and on every switch, so a screen reader speaks the words in their language.
    use_effect(move || {
        if outermost {
            platform::apply_lang(localization().lang);
        }
    });

    // A platform scheme change reaches the Rust side too; the CSS follows on its own.
    let _scheme_subscription = use_hook(|| {
        let context = context.clone();
        // `Rc`: a hook value must be `Clone`, and a subscription is not.
        Rc::new(color_scheme().map(|platform| {
            platform.on_change(Box::new(move |scheme| {
                let mut system = context.system_scheme;
                system.set(scheme);
                if *context.scheme_setting.peek() == ColorSchemeSetting::System {
                    context.follow_system(scheme);
                }
            }))
        }))
    });

    // A stored choice must reach the root, or the media block keeps answering.
    // The inline script does it before paint; this covers a client-only build.
    use_hook(|| {
        if let Some(pinned) = setting.fixed() {
            context.set_color_scheme(pinned.into());
        }
    });

    let portal_entries = use_signal(Vec::new);
    use_context_provider(|| PortalHost::new(portal_entries));

    // From the active theme, so a switch to other `z_index` values moves the stacks too.
    let modal_layers = use_memo(move || {
        let z = theme_signal().z_index;
        ZLayers {
            base: z.modal,
            step: z.modal_step,
            ceiling: z.popover,
        }
    });
    let modal_stack = use_signal(Vec::new);
    use_context_provider(|| ModalHost::new(modal_stack, modal_layers.into()));

    let window_layers = use_memo(move || {
        let z = theme_signal().z_index;
        ZLayers {
            base: z.window,
            step: z.window_step,
            ceiling: z.overlay,
        }
    });
    let window_stack = use_signal(Vec::new);
    use_context_provider(|| WindowHost::new(window_stack, window_layers.into()));

    rsx! {
        style {
            dangerous_inner_html: "{context.layer_order_css}"
        }
        ThemeStyle {}
        {platform::Listener(rsx! {
            {children}
            PortalOutlet {}
        })}
        StyleOutlet {}
        platform::Outlet {}
    }
}
