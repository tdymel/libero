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
    .align_items(crate::sx_var!("stack-align"))
    .justify_content(crate::sx_var!("stack-justify"))
    .gap(crate::sx_var!("stack-gap"))
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

    let gap = props.gap.unwrap_or_else(|| "md".to_string());
    let variables = vec![
        (
            "stack-align",
            props.align.unwrap_or_else(|| "stretch".to_string()),
        ),
        (
            "stack-justify",
            props.justify.unwrap_or_else(|| "flex-start".to_string()),
        ),
        (
            "stack-gap",
            match Size::parse_str(gap.as_str()) {
                Some(size) => SizeCssVar::SPACING.value(size),
                None => gap,
            },
        ),
    ];

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
