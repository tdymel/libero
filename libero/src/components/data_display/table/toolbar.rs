use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, attr},
        layout::use_box,
    },
    sx::{StaticSx, sx},
};

static TOOLBAR_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_wrap("wrap")
        .align_items("center")
        .gap("8px")
        .margin("0 0 8px")
        // The quick filter keeps to the inline end, and drops its own margin.
        .selector("& > [data-toolbar-end]", sx().margin_inline_start("auto"))
        .selector("& > [data-toolbar-end] > *", sx().margin("0"))
});

/// The row above a table: the caller's `toolbar`, then the quick filter at
/// the end. No `role=toolbar`, which would promise arrow-key focus.
#[component]
pub(super) fn TableToolbar(content: Element, search: Option<Element>) -> Element {
    use_box().framework_sx(&TOOLBAR_SX).prepare().render(
        HtmlTag::Div,
        vec![attr("data-toolbar", "true")],
        rsx! {
            {content}
            if let Some(search) = search {
                div { "data-toolbar-end": true, {search} }
            }
        },
    )
}
