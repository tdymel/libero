use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
    hooks::use_theme,
    sx::StaticSx,
    theme::{ListDefaults, Size},
};

static LIST_BASE_SX: StaticSx = StaticSx::new(|| {
    ListDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .list_style("none")
        .margin("0")
        .padding("0")
});

base_props! {
    pub struct ListProps {
        /// Item gap and nested-list indent - `theme.list.size` (`Md`) by default.
        #[props(default, into)]
        size: Input<Size>,
        children: Element,
    }
}

#[component]
pub fn List(props: ListProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.list.size);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .into();

    use_box()
        .framework_sx(&LIST_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        // Not redundant: Safari with VoiceOver drops list semantics from a
        // `list-style: none` list. A caller's own `role` still wins.
        .attr_default("role", "list")
        .render(HtmlTag::Ul, props.attributes, props.children)
}
