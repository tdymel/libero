use dioxus::prelude::*;

use crate::{LiberoContext, Theme};

#[component]
pub fn LiberoProvider(children: Element) -> Element {
    let context = use_context_provider(|| {
        let theme = Theme::default();

        LiberoContext {
            theme,
            ..LiberoContext::default()
        }
    });
    let theme_stylesheet = context.theme.css_variables();
    let stylesheet = context.sx.stylesheet();

    rsx! {
        style { {theme_stylesheet} }
        style { {stylesheet} }
        {children}
    }
}
