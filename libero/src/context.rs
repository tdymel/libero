use dioxus::prelude::*;

use crate::{common::ConstStr, theme::Theme};

#[derive(Clone, Copy)]
pub struct LiberoContext {
    pub theme: &'static Theme,
    pub(crate) theme_css: ConstStr,
}

impl LiberoContext {
    pub const fn new(theme: &'static Theme) -> Self {
        Self {
            theme,
            theme_css: theme.to_css(),
        }
    }
}

#[component]
pub fn LiberoProvider(theme: &'static Theme, children: Element) -> Element {
    let context = LiberoContext::new(theme);
    use_context_provider(|| context);

    rsx! {
        style {
            dangerous_inner_html: "{context.theme_css.as_str()}"
        }
        {children}
    }
}

pub fn use_theme() -> &'static Theme {
    use_context::<LiberoContext>().theme
}
