use dioxus::prelude::*;

use crate::{
    components::{Box, util::classes},
    css::SizeCssVar,
    sx::{Sx, sx},
    theme::Size,
};

const GROUP_BASE_SX: Sx = sx()
    .display("flex")
    .flex_direction("row")
    .flex_wrap("var(--lsx-group-wrap)")
    .align_items("var(--lsx-group-align)")
    .justify_content("var(--lsx-group-justify)")
    .gap("var(--lsx-group-gap)")
    .build();

#[derive(Props, Clone, PartialEq)]
pub struct GroupProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default)]
    sx: Option<&'static Sx>,
    #[props(default)]
    states: Vec<(&'static str, bool)>,
    #[props(default)]
    align: Option<String>,
    #[props(default)]
    justify: Option<String>,
    #[props(default)]
    gap: Option<String>,
    #[props(default)]
    wrap: Option<bool>,
    children: Element,
}

#[component]
pub fn Group(props: GroupProps) -> Element {
    crate::context::use_sx(&GROUP_BASE_SX);

    let mut variables = Vec::new();

    if let Some(align) = props.align {
        variables.push(("group-align", align));
    }

    if let Some(justify) = props.justify {
        variables.push(("group-justify", justify));
    }

    if let Some(gap) = props.gap {
        let gap = match Size::parse_str(gap.as_str()) {
            Some(size) => SizeCssVar::SPACING.value(size),
            None => gap,
        };

        variables.push(("group-gap", gap));
    }

    if let Some(wrap) = props.wrap {
        variables.push((
            "group-wrap",
            if wrap { "wrap" } else { "nowrap" }.to_string(),
        ));
    }

    let class = classes(props.class, GROUP_BASE_SX.class_name());

    rsx! {
        Box {
            attributes: props.attributes,
            class: class,
            sx: props.sx,
            states: props.states,
            variables: variables,
            {props.children}
        }
    }
}
