use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    SxLayer, css::Stylesheet,
    hooks::{PortalHost, PortalOutlet},
    sx_registry::SxRegistry,
    theme::Theme,
};

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

    rsx! {
        style {
            dangerous_inner_html: "{context.layer_order_css}"
        }
        style {
            dangerous_inner_html: "{context.theme_css.as_str()}"
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

pub fn use_theme() -> &'static Theme {
    use_context::<LiberoContext>().theme
}

/// Registers an [`Sx`](crate::sx::Sx) value and returns the class name to put
/// on any element - including ones libero doesn't provide a component for
/// (e.g. a raw `select`). Uses the `UserCustom` layer, which has priority
/// over every other layer (including a component's own dynamic prop-driven
/// styles), so it always wins.
pub fn use_class(sx: &crate::sx::Sx) -> Option<String> {
    use_sx(sx, SxLayer::UserCustom)
}

pub(crate) fn use_sx(sx: &crate::sx::Sx, layer: SxLayer) -> Option<String> {
    let mut context = use_context::<LiberoContext>();
    let key = (layer, sx.hash());
    let active_registration = use_hook(|| Rc::new(RefCell::new(None::<(SxLayer, u64)>)));

    {
        let mut active_registration = active_registration.borrow_mut();

        if sx.is_empty() {
            if let Some(active_key) = active_registration.take() {
                context.sx_registry.release(active_key);
                *context.sx_registry_version.write() += 1;
            }
        } else {
            match *active_registration {
                Some(active_key) if active_key == key => {}
                Some(active_key) => {
                    context.sx_registry.release(active_key);
                    context.sx_registry.acquire(sx, layer);
                    *active_registration = Some(key);
                    *context.sx_registry_version.write() += 1;
                }
                None => {
                    context.sx_registry.acquire(sx, layer);
                    *active_registration = Some(key);
                    *context.sx_registry_version.write() += 1;
                }
            }
        }
    }

    {
        let active_registration = active_registration.clone();
        let sx_registry = context.sx_registry.clone();
        let mut sx_registry_version = context.sx_registry_version;
        use_drop(move || {
            if let Some(active_key) = active_registration.borrow_mut().take() {
                sx_registry.release(active_key);
                *sx_registry_version.write() += 1;
            }
        });
    }

    if sx.is_empty() {
        None
    } else {
        Some(sx.class_name())
    }
}
