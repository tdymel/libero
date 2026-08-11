use dioxus::prelude::*;

use crate::{
    components::{Box, util::classes},
    css::SizeCssVar,
    sx::{Sx, sx},
    theme::Size,
};

const STACK_BASE_SX: Sx = sx()
    .display("flex")
    .flex_direction("column")
    .align_items("var(--lsx-stack-align, stretch)")
    .justify_content("var(--lsx-stack-justify, flex-start)")
    .gap("var(--lsx-stack-gap, var(--lsx-spacing-md))")
    .build();

#[derive(Props, Clone, PartialEq)]
pub struct StackProps {
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
    children: Element,
}

#[component]
pub fn Stack(props: StackProps) -> Element {
    crate::context::use_sx(&STACK_BASE_SX);

    let mut variables = Vec::new();

    if let Some(align) = props.align {
        if align != "stretch" {
            variables.push(("stack-align", align));
        }
    }

    if let Some(justify) = props.justify {
        if justify != "flex-start" {
            variables.push(("stack-justify", justify));
        }
    }

    if let Some(gap) = props.gap {
        let gap = match Size::parse_str(gap.as_str()) {
            Some(size) => SizeCssVar::SPACING.value(size),
            None => gap,
        };

        if gap != SizeCssVar::SPACING.value(Size::Md) {
            variables.push(("stack-gap", gap));
        }
    }

    let class = classes(props.class, STACK_BASE_SX.class_name());

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
