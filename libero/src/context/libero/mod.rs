use dioxus::prelude::*;

use sx_registry::SxRegistry;

use super::{MODAL_BASE_Z_INDEX, ModalHost, PortalHost, PortalOutlet};
use crate::{css::Stylesheet, theme::Theme};

mod sx_layer;
mod sx_registry;

pub(crate) use sx_layer::SxLayer;

const SCROLL_LOCK_CSS: &str = "body:has([data-lsx-scroll-lock]) { overflow: hidden; }";

#[derive(Clone)]
pub struct LiberoContext {
    pub theme: &'static Theme,
    pub(crate) layer_order_css: &'static str,
    pub(crate) theme_css: Stylesheet,
    pub(crate) sx_registry: SxRegistry,
    pub(crate) sx_registry_version: Signal<u64>,
}

impl LiberoContext {
    pub fn new(theme: &'static Theme, sx_registry_version: Signal<u64>) -> Self {
        Self {
            theme,
            layer_order_css: SxLayer::order_css(),
            theme_css: Stylesheet::from(theme),
            sx_registry: SxRegistry::new(),
            sx_registry_version,
        }
    }
}

#[component]
pub fn LiberoProvider(
    #[props(default = &Theme::DEFAULT)] theme: &'static Theme,
    children: Element,
) -> Element {
    let sx_registry_version = use_signal(|| 0u64);
    let context = use_context_provider(|| LiberoContext::new(theme, sx_registry_version));
    let _registry_version = context.sx_registry_version.read();
    let active_stylesheets = context.sx_registry.stylesheets();

    let portal_entries = use_signal(Vec::new);
    use_context_provider(|| PortalHost::new(portal_entries));

    let modal_z_index = use_signal(|| MODAL_BASE_Z_INDEX);
    use_context_provider(|| ModalHost::new(modal_z_index));

    rsx! {
        style {
            dangerous_inner_html: "{context.layer_order_css}"
        }
        style {
            dangerous_inner_html: "{context.theme_css.as_str()}"
        }
        style {
            dangerous_inner_html: SCROLL_LOCK_CSS
        }
        for stylesheet in active_stylesheets {
            style {
                dangerous_inner_html: "{stylesheet}"
            }
        }
        {children}
        PortalOutlet {}
    }
}
