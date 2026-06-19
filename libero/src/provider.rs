use dioxus::prelude::*;

use crate::LiberoContext;

#[component]
pub fn LiberoProvider(children: Element) -> Element {
    let context = use_context_provider(LiberoContext::default);
    let stylesheet = context.sx.stylesheet();

    rsx! {
        style { {stylesheet} }
        {children}
    }
}
