use dioxus::prelude::*;

use crate::{
    components::{Box, util::classes},
    css::SizeCssVar,
    sx::{Sx, sx},
    theme::Size,
};

const STACK_BASE_SX: Sx = sx()
    .display("flex")
    .flex_direction("var(--lsx-stack-direction, column)")
    .flex_wrap("var(--lsx-stack-wrap, var(--lsx-stack-column-wrap))")
    .align_items("var(--lsx-stack-align, var(--lsx-stack-column-align))")
    .justify_content("var(--lsx-stack-justify, var(--lsx-stack-column-justify))")
    .gap("var(--lsx-stack-spacing, var(--lsx-stack-column-spacing))");

#[derive(Props, Clone, PartialEq)]
pub struct StackProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default)]
    sx: Option<Sx>,
    #[props(default)]
    states: Vec<(&'static str, bool)>,
    #[props(default)]
    align: Option<String>,
    #[props(default)]
    justify: Option<String>,
    #[props(default)]
    spacing: Option<String>,
    #[props(default)]
    direction: Option<String>,
    #[props(default)]
    wrap: Option<bool>,
    children: Element,
}

#[component]
pub fn Stack(props: StackProps) -> Element {
    let stack_class = crate::context::use_sx(&STACK_BASE_SX);

    let mut variables = Vec::new();
    let is_row = matches!(props.direction.as_deref(), Some("row"));

    if let Some(direction) = props.direction {
        variables.push(("stack-direction", direction));
        variables.push((
            "stack-align",
            if is_row {
                "var(--lsx-stack-row-align)"
            } else {
                "var(--lsx-stack-column-align)"
            }
            .to_string(),
        ));
        variables.push((
            "stack-justify",
            if is_row {
                "var(--lsx-stack-row-justify)"
            } else {
                "var(--lsx-stack-column-justify)"
            }
            .to_string(),
        ));
        variables.push((
            "stack-spacing",
            if is_row {
                "var(--lsx-stack-row-spacing)"
            } else {
                "var(--lsx-stack-column-spacing)"
            }
            .to_string(),
        ));
        variables.push((
            "stack-wrap",
            if is_row {
                "var(--lsx-stack-row-wrap)"
            } else {
                "var(--lsx-stack-column-wrap)"
            }
            .to_string(),
        ));
    }

    if let Some(align) = props.align {
        variables.push(("stack-align", align));
    }

    if let Some(justify) = props.justify {
        variables.push(("stack-justify", justify));
    }

    if let Some(spacing) = props.spacing {
        let spacing = match Size::parse_str(spacing.as_str()) {
            Some(size) => SizeCssVar::SPACING.value(size),
            None => spacing,
        };

        variables.push(("stack-spacing", spacing));
    }

    if let Some(wrap) = props.wrap {
        variables.push((
            "stack-wrap",
            if wrap { "wrap" } else { "nowrap" }.to_string(),
        ));
    }

    let class = classes(props.class, stack_class);

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
