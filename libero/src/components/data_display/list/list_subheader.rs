use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    tokens::NamedColorCss,
};

// Block, not `list-item`, so an ordered list's numbering skips it. The smaller text is
// the inner span's: the `li` keeps the list's `em`, which pulls it out of the number gutter.
static LIST_SUBHEADER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .padding("0")
        .color(NamedColorCss::TEXT_DIMMED.value())
        .font_weight("600")
        .selector("& > span", sx().font_size("sm"))
});

base_props! {
    pub struct ListSubheaderProps {
        children: Element,
    }
}

/// A dimmed caption `<li>` that opens a run of items in a [`List`](super::List). It is
/// read as one more item; for groups a screen reader names, nest one `List` per group,
/// each with an `aria-label`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{List, ListItem, ListSubheader};
/// # fn app() -> Element {
/// rsx! {
///     List {
///         ListSubheader { "Fruit" }
///         ListItem { "Apple" }
///         ListSubheader { "Vegetables" }
///         ListItem { "Carrot" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/list>
#[component]
pub fn ListSubheader(props: ListSubheaderProps) -> Element {
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("subheader", true)
        .into();

    use_box()
        .framework_sx(&LIST_SUBHEADER_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(
            HtmlTag::Li,
            props.attributes,
            rsx! {
                span { {props.children} }
            },
        )
}
