use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    hooks::use_theme,
    sx::{StaticSx, Sx},
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
    let size = props.size.as_ref().copied().unwrap_or(theme.list.size);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(size.state_name(), true);

    rsx! {
        Box {
            component: "ul",
            class: props.class,
            sx: props.sx,
            states,
            framework_sx: &LIST_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
