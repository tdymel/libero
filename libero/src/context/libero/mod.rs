use std::rc::Rc;

use dioxus::prelude::*;

use stylesheet_registry::StylesheetRegistry;

pub(crate) use stylesheet_registry::{SheetRank, StylesheetKey};

use super::{ModalHost, PortalHost, PortalOutlet, WindowHost};
use crate::{css::Stylesheet, platform::backend, theme::Theme};

mod css_layer;
mod stylesheet_registry;

pub(crate) use css_layer::CssLayer;

#[derive(Clone)]
pub struct LiberoContext {
    pub theme: &'static Theme,
    pub(crate) layer_order_css: &'static str,
    /// `Rc<str>`, not a `Stylesheet`: every `use_context::<LiberoContext>()`
    /// clones this struct, and only `LiberoProvider` ever reads this field.
    pub(crate) theme_css: Rc<str>,
    pub(crate) stylesheet_registry: StylesheetRegistry,
    pub(crate) stylesheet_registry_version: Signal<u64>,
}

impl LiberoContext {
    pub fn new(theme: &'static Theme, stylesheet_registry_version: Signal<u64>) -> Self {
        Self {
            theme,
            layer_order_css: CssLayer::order_css(),
            theme_css: Rc::from(Stylesheet::from(theme).as_str()),
            stylesheet_registry: StylesheetRegistry::new(),
            stylesheet_registry_version,
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
    #[props(default = &Theme::DEFAULT)] theme: &'static Theme,
    children: Element,
) -> Element {
    let stylesheet_registry_version = use_signal(|| 0u64);
    let context = use_context_provider(|| LiberoContext::new(theme, stylesheet_registry_version));

    let portal_entries = use_signal(Vec::new);
    use_context_provider(|| PortalHost::new(portal_entries));

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
        style {
            dangerous_inner_html: "{context.theme_css}"
        }
        {children}
        PortalOutlet {}
        StyleOutlet {}
        backend::Outlet {}
    }
}
