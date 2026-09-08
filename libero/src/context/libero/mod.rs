use std::rc::Rc;

use dioxus::prelude::*;

use stylesheet_registry::StylesheetRegistry;

pub(crate) use stylesheet_registry::{SheetRank, StylesheetKey};

use super::{ModalHost, PortalHost, PortalOutlet, WindowHost};
use crate::{
    css::Stylesheet,
    platform::{backend, document},
    theme::{THEME_ATTRIBUTE, Theme, ThemeSet},
    utils::warn,
};

mod css_layer;
mod stylesheet_registry;

pub(crate) use css_layer::CssLayer;

#[derive(Clone)]
pub struct LiberoContext {
    /// Every theme the app ships, and which of them is its light and its dark.
    pub themes: ThemeSet,
    /// The active one. A `Signal` so [`use_theme`](crate::hooks::use_theme) is
    /// reactive: a component that reads a spacing number, a label or a
    /// `HexColor` off the theme re-renders when the theme changes, and so
    /// stays in step with the CSS vars that changed without it.
    pub theme: Signal<&'static Theme>,
    /// The active theme's name, so a switch can tell a repeat from a change
    /// and the pair from a rebuild.
    pub(crate) active: Signal<&'static str>,
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
    pub(crate) fn new(
        themes: ThemeSet,
        theme: Signal<&'static Theme>,
        active: Signal<&'static str>,
        theme_css: Signal<Rc<str>>,
        stylesheet_registry_version: Signal<u64>,
    ) -> Self {
        Self {
            themes,
            theme,
            active,
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
        let Some(theme) = self.themes.get(name) else {
            warn(&format!(
                "unknown theme `{name}`; the set has no such theme, so nothing changed"
            ));
            return;
        };

        let (mut active, mut current, mut css) = (self.active, self.theme, self.theme_css);
        if *active.peek() == name {
            return;
        }

        let by_attribute = self.themes.is_in_pair(name)
            && document()
                .is_some_and(|document| document.set_root_attribute(THEME_ATTRIBUTE, Some(name)));
        if !by_attribute {
            css.set(Rc::from(Stylesheet::from(theme).as_str()));
        }

        active.set(name);
        current.set(theme);
    }
}

/// The theme's own `<style>`. Its own component so that a rebuilt sheet - a
/// named theme beyond the pair, or a platform that cannot carry the attribute
/// - re-renders this leaf instead of everything under `{children}`.
#[component]
fn ThemeStyle() -> Element {
    let context = use_context::<LiberoContext>();
    let css = context.theme_css.read().clone();

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
    /// One theme, and sugar for a [`ThemeSet`] holding only it. Ignored when
    /// `themes` is given.
    #[props(default = &Theme::DEFAULT)]
    theme: &'static Theme,
    /// Every theme the app ships. Its light and dark halves are emitted into
    /// the sheet together, so switching between them costs no re-render.
    #[props(default)]
    themes: Option<ThemeSet>,
    children: Element,
) -> Element {
    let themes = use_hook(|| themes.clone().unwrap_or_else(|| ThemeSet::of(theme)));
    let active = use_signal(|| ThemeSet::LIGHT);
    let theme_signal = use_signal({
        let themes = themes.clone();
        move || themes.light_theme()
    });
    let theme_css = use_signal({
        let themes = themes.clone();
        move || Rc::<str>::from(Stylesheet::from(&themes).as_str())
    });

    let stylesheet_registry_version = use_signal(|| 0u64);
    let context = use_context_provider(|| {
        LiberoContext::new(
            themes,
            theme_signal,
            active,
            theme_css,
            stylesheet_registry_version,
        )
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
        {children}
        PortalOutlet {}
        StyleOutlet {}
        backend::Outlet {}
    }
}
