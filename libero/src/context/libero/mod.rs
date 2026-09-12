use std::rc::Rc;

use dioxus::prelude::*;

use stylesheet_registry::StylesheetRegistry;

pub(crate) use stylesheet_registry::{SheetRank, StylesheetKey};

use super::{ModalHost, PortalHost, PortalOutlet, WindowHost};
use crate::{
    css::Stylesheet,
    platform::{backend, color_scheme, document},
    theme::{THEME_ATTRIBUTE, Theme, ThemeSet},
    tokens::{ColorScheme, ColorSchemeSetting},
    utils::warn,
};

mod css_layer;
mod stylesheet_registry;

pub(crate) use css_layer::CssLayer;

#[derive(Clone)]
pub struct LiberoContext {
    /// Every theme the app ships, and which of them is its light and its
    /// dark. A `Signal`, because a whole set can be swapped at runtime -
    /// which is what a theme picker does.
    pub themes: Signal<ThemeSet>,
    /// The active one. A `Signal` so [`use_theme`](crate::hooks::use_theme) is
    /// reactive: a component that reads a spacing number, a label or a
    /// `HexColor` off the theme re-renders when the theme changes, and so
    /// stays in step with the CSS vars that changed without it.
    pub theme: Signal<&'static Theme>,
    /// The active theme's name, so a switch can tell a repeat from a change
    /// and the pair from a rebuild.
    pub(crate) active: Signal<&'static str>,
    /// What the app asked for, which is not what is on screen until it is
    /// resolved against `system_scheme`.
    pub(crate) scheme_setting: Signal<ColorSchemeSetting>,
    /// What the platform is set to. `Light` where this build cannot tell -
    /// see [`color_scheme`](crate::platform::color_scheme).
    pub(crate) system_scheme: Signal<ColorScheme>,
    pub(crate) layer_order_css: &'static str,
    /// `Rc<str>`, not a `Stylesheet`: every `use_context::<LiberoContext>()`
    /// clones this struct, and only `ThemeStyle` ever reads this field.
    pub(crate) theme_css: Signal<Rc<str>>,
    pub(crate) stylesheet_registry: StylesheetRegistry,
    pub(crate) stylesheet_registry_version: Signal<u64>,
}

impl LiberoContext {
    /// `pub(crate)`: the signals are internal, and `LiberoProvider` is the
    /// only thing that may own them. A caller reaches the context through
    /// `use_context`, never by building it.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        themes: Signal<ThemeSet>,
        theme: Signal<&'static Theme>,
        active: Signal<&'static str>,
        scheme_setting: Signal<ColorSchemeSetting>,
        system_scheme: Signal<ColorScheme>,
        theme_css: Signal<Rc<str>>,
        stylesheet_registry_version: Signal<u64>,
    ) -> Self {
        Self {
            themes,
            theme,
            active,
            scheme_setting,
            system_scheme,
            layer_order_css: CssLayer::order_css(),
            theme_css,
            stylesheet_registry: StylesheetRegistry::new(),
            stylesheet_registry_version,
        }
    }

    /// Switches to the theme `name` selects. An unknown name warns and
    /// changes nothing, so a typo cannot blank the app's colours.
    ///
    /// The light/dark pair is already in the sheet, so that switch is the
    /// attribute on the document root plus a signal write - no re-mount, and
    /// the colours change before the re-render that follows. Anything else,
    /// or any platform with no reachable root, rebuilds the sheet.
    pub fn set_active_theme(&self, name: &'static str) {
        let Some(theme) = self.themes.peek().get(name) else {
            warn(&format!(
                "unknown theme `{name}`; the set has no such theme, so nothing changed"
            ));
            return;
        };

        let (mut active, mut current, mut css) = (self.active, self.theme, self.theme_css);
        if *active.peek() == name {
            return;
        }

        let by_attribute = self.themes.peek().is_in_pair(name)
            && document()
                .is_some_and(|document| document.set_root_attribute(THEME_ATTRIBUTE, Some(name)));
        if !by_attribute {
            css.set(Rc::from(Stylesheet::from(theme).as_str()));
        }

        active.set(name);
        current.set(theme);
    }

    /// Pins one scheme of the pair, or hands the choice back to the platform.
    ///
    /// Not [`set_active_theme`](Self::set_active_theme): that one is about
    /// *which theme*, and cannot express "whichever the platform says". The
    /// two differ in the attribute - a pinned scheme writes it, the system
    /// case removes it so the sheet's own media block decides - and this one
    /// writes it even when the resolved theme is the one already showing,
    /// because otherwise a click on "dark" while the platform is dark would
    /// leave the page following the platform.
    pub(crate) fn set_color_scheme(&self, setting: ColorSchemeSetting) {
        // A set with no dark half cannot be pinned to dark. Falling back to
        // the system setting rather than refusing outright, because this also
        // runs when a *picker* swaps to such a set while dark was pinned:
        // refusing would leave the pin describing a theme that is not there.
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
            Some(scheme) => self.pin_scheme(scheme),
            None => self.follow_system(*self.system_scheme.peek()),
        }
    }

    fn pin_scheme(&self, scheme: ColorScheme) {
        let name = scheme.as_str();
        let Some(theme) = self.themes.peek().get(name) else {
            return;
        };

        let carried = document()
            .is_some_and(|document| document.set_root_attribute(THEME_ATTRIBUTE, Some(name)));
        if !carried {
            let mut css = self.theme_css;
            css.set(Rc::from(Stylesheet::from(theme).as_str()));
        }

        let (mut active, mut current) = (self.active, self.theme);
        active.set(name);
        current.set(theme);
    }

    /// Hands the choice back to the platform: the root carries no theme
    /// attribute, so the sheet's `@media (prefers-color-scheme: dark)` block
    /// decides. The Rust-side theme is set to `scheme`'s anyway, or a
    /// component reading a `HexColor` off the theme would disagree with the
    /// page it is painted on.
    pub(crate) fn follow_system(&self, scheme: ColorScheme) {
        let name = scheme.as_str();
        let themes = self.themes.peek().clone();
        let theme = themes.get(name).unwrap_or_else(|| themes.light_theme());

        let cleared =
            document().is_some_and(|document| document.set_root_attribute(THEME_ATTRIBUTE, None));
        if !cleared {
            // No attribute to clear, so the sheet has to carry the choice
            // itself. The whole set, not this one theme: the media block is
            // what makes the system case work without us.
            let mut css = self.theme_css;
            css.set(Rc::from(Stylesheet::from(&themes).as_str()));
        }

        let (mut active, mut current) = (self.active, self.theme);
        active.set(name);
        current.set(theme);
    }

    /// Swaps the whole set - what a theme picker does.
    ///
    /// The sheet is rebuilt, because the pair it carries is this set's pair:
    /// nothing in the old sheet describes the new palette. The colour-scheme
    /// *setting* is kept and re-applied on top, so a reader who pinned dark
    /// stays in dark through the swap.
    pub fn set_theme_set(&self, themes: ThemeSet) {
        let (mut current_set, mut css) = (self.themes, self.theme_css);
        current_set.set(themes.clone());
        css.set(Rc::from(Stylesheet::from(&themes).as_str()));

        // Read out first: `set_color_scheme` writes this same signal, and a
        // `peek()` guard held across the call is a borrow panic, not a
        // compile error.
        let setting = *self.scheme_setting.peek();
        // Re-applied rather than assumed: the new set may have no dark half,
        // in which case this falls back to following the platform.
        self.set_color_scheme(setting);
    }
}

/// The theme's own `<style>`. Its own component so that a rebuilt sheet - a
/// named theme beyond the pair, or a platform that cannot carry the attribute
/// - re-renders this leaf instead of everything under `{children}`.
#[component]
fn ThemeStyle() -> Element {
    let context = use_context::<LiberoContext>();
    let css = context.theme_css.read().clone();

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

    rsx! {
        style {
            dangerous_inner_html: "{css}"
        }
    }
}

/// The `<style>` nodes for everything registered so far.
///
/// Its own component for two reasons. It subscribes to
/// `stylesheet_registry_version`, so registering CSS re-renders this leaf
/// rather than everything under `{children}`. And `LiberoProvider` renders it
/// *after* `{children}`: child scopes render eagerly in tree order, so by the
/// time this runs every descendant has registered and the first pass carries
/// the complete CSS.
#[component]
fn StyleOutlet() -> Element {
    let context = use_context::<LiberoContext>();
    let _registry_version = context.stylesheet_registry_version.read();

    rsx! {
        for (node_key, stylesheet) in context.stylesheet_registry.stylesheets() {
            style {
                key: "{node_key}",
                dangerous_inner_html: "{stylesheet}"
            }
        }
    }
}

#[component]
pub fn LiberoProvider(
    /// Every theme the app ships. Its light and dark halves are emitted into
    /// the sheet together, so switching between them costs no re-render.
    ///
    /// A lone `&'static Theme` converts to a set of that theme alone: a theme
    /// of the caller's own has no dark counterpart for us to pair it with.
    /// The default is [`ThemeSet::DEFAULT`], so an app that names no theme at
    /// all still follows `prefers-color-scheme`.
    #[props(default, into)]
    themes: ThemeSet,
    children: Element,
) -> Element {
    let themes = use_hook(|| themes.clone());
    // Read once, at mount, so the first render already paints the scheme the
    // app was last left in rather than flashing the light one.
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

    let stylesheet_registry_version = use_signal(|| 0u64);
    let context = use_context_provider(|| {
        LiberoContext::new(
            theme_set,
            theme_signal,
            active,
            scheme_setting,
            system_scheme,
            theme_css,
            stylesheet_registry_version,
        )
    });

    // The platform can change its mind while the app is running, and while
    // the app follows it that has to reach the Rust side too: the CSS already
    // follows on its own. Held for the provider's lifetime; dropping it
    // removes the listener.
    let _scheme_subscription = use_hook(|| {
        let context = context.clone();
        // `Rc`, because a hook's value has to be `Clone` and a subscription
        // is not - cloning one would be a second listener, not a second
        // handle to the same one.
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

    // A stored choice has to reach the document root, or the sheet's media
    // block would keep answering. The inline script below already did this
    // before first paint; this is the same write for a client-only build,
    // where there was no server-rendered document to run it in.
    use_hook(|| {
        if let Some(pinned) = setting.fixed() {
            context.set_color_scheme(pinned.into());
        }
    });

    let portal_entries = use_signal(Vec::new);
    use_context_provider(|| PortalHost::new(portal_entries));

    let theme = *theme_signal.peek();
    let modal_stack = use_signal(Vec::new);
    use_context_provider(|| {
        ModalHost::new(
            modal_stack,
            theme.z_index.modal,
            theme.z_index.modal_step,
            theme.z_index.popover,
        )
    });

    let window_stack = use_signal(Vec::new);
    use_context_provider(|| {
        WindowHost::new(
            window_stack,
            theme.z_index.window,
            theme.z_index.window_step,
            theme.z_index.overlay,
        )
    });

    rsx! {
        style {
            dangerous_inner_html: "{context.layer_order_css}"
        }
        ThemeStyle {}
        {backend::Listener(rsx! {
            {children}
            PortalOutlet {}
        })}
        StyleOutlet {}
        backend::Outlet {}
    }
}
