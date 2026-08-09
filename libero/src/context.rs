use dioxus::prelude::*;

use crate::theme::Theme;

#[derive(Clone, Copy)]
pub struct LiberoContext {
    pub theme: &'static Theme,
}

#[component]
pub fn LiberoProvider(theme: &'static Theme, children: Element) -> Element {
    let css = theme.to_css_vars();
    use_context_provider(|| LiberoContext { theme });

    rsx! {
        style {
            dangerous_inner_html: "{css.as_str()}"
        }
        {children}
    }
}

pub fn use_theme() -> &'static Theme {
    use_context::<LiberoContext>().theme
}
