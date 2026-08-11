use dioxus::prelude::*;

use crate::{css::Stylesheet, sx_registry::SxRegistry, theme::Theme};

#[derive(Clone)]
pub struct LiberoContext {
    pub theme: &'static Theme,
    pub(crate) theme_css: Stylesheet,
    pub(crate) sx_registry: SxRegistry,
    pub(crate) sx_registry_version: Signal<u64>,
}

impl LiberoContext {
    pub fn new(theme: &'static Theme, sx_registry_version: Signal<u64>) -> Self {
        Self {
            theme,
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

    rsx! {
        style {
            dangerous_inner_html: "{context.theme_css.as_str()}"
        }
        for stylesheet in active_stylesheets {
            style {
                dangerous_inner_html: "{stylesheet}"
            }
        }
        {children}
    }
}

pub fn use_theme() -> &'static Theme {
    use_context::<LiberoContext>().theme
}

pub fn use_sx(sx: &crate::sx::Sx) {
    let mut context = use_context::<LiberoContext>();
    let _registration = use_hook(|| {
        *context.sx_registry_version.write() += 1;
        context
            .sx_registry
            .register(sx, context.sx_registry_version)
    });
}
