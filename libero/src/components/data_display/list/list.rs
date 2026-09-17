use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{ListDefaults, Size},
};

static LIST_BASE_SX: StaticSx = StaticSx::new(|| {
    ListDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .list_style("none")
        .margin("0")
        .padding("0")
        // Outside markers, in em so two-digit numbers still fit the gutter.
        .when(
            "ordered",
            sx().list_style_type("decimal").padding_inline_start("2em"),
        )
});

base_props! {
    pub struct ListProps {
        /// Item gap and nested-list indent - `theme.list.size` (`Md`) by default.
        #[props(default, into)]
        size: Input<Size>,
        /// An `ol` with visible numbers, for items whose order is the point.
        #[props(default)]
        ordered: bool,
        children: Element,
    }
}

/// A vertical list of [`ListItem`](super::ListItem)s: a bare `ul`, or with
/// `ordered` a numbered `ol`.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{List, ListItem};
/// # fn app() -> Element { rsx! {
/// List { ordered: true,
///     ListItem { "Install" }
///     ListItem { "Configure" }
/// }
/// # } }
/// ```
#[component]
pub fn List(props: ListProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.list.size);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("ordered", props.ordered)
        .into();
    let tag = if props.ordered {
        HtmlTag::Ol
    } else {
        HtmlTag::Ul
    };

    use_box()
        .framework_sx(&LIST_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        // Not redundant: Safari with VoiceOver drops list semantics from a
        // `list-style: none` list. A caller's own `role` still wins.
        .attr_default("role", "list")
        .render(tag, props.attributes, props.children)
}
