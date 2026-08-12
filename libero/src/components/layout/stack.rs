use dioxus::prelude::*;

use crate::{
    components::{Box, util::classes},
    sx::{StaticSx, Sx, SxInput, sx},
    theme::Size,
};

static STACK_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("var(--lsx-stack-direction, column)")
        .flex_wrap("var(--lsx-stack-wrap, var(--lsx-stack-column-wrap))")
        .align_items("var(--lsx-stack-align, var(--lsx-stack-column-align))")
        .justify_content("var(--lsx-stack-justify, var(--lsx-stack-column-justify))")
        .gap("var(--lsx-stack-spacing, var(--lsx-stack-column-spacing))")
});

fn stack_dynamic_sx(props: &StackProps) -> Sx {
    let is_row = matches!(props.direction.as_deref(), Some("row")) || props.wrap.is_some();
    let direction = if is_row { "row" } else { "column" };
    let align = props.align.as_deref().unwrap_or(if is_row {
        "var(--lsx-stack-row-align)"
    } else {
        "var(--lsx-stack-column-align)"
    });
    let justify = props.justify.as_deref().unwrap_or(if is_row {
        "var(--lsx-stack-row-justify)"
    } else {
        "var(--lsx-stack-column-justify)"
    });
    let spacing = props.spacing.as_deref().map_or_else(
        || {
            if is_row {
                "var(--lsx-stack-row-spacing)".to_string()
            } else {
                "var(--lsx-stack-column-spacing)".to_string()
            }
        },
        |spacing| match Size::parse_dynamic(spacing) {
            Some(size) => format!("var(--lsx-spacing-{})", size.as_str()),
            None => spacing.to_string(),
        },
    );
    let wrap = props.wrap.map_or_else(
        || {
            if is_row {
                "var(--lsx-stack-row-wrap)"
            } else {
                "var(--lsx-stack-column-wrap)"
            }
        },
        |wrap| if wrap { "wrap" } else { "nowrap" },
    );

    sx().flex_direction(direction)
        .align_items(align)
        .justify_content(justify)
        .gap(spacing)
        .flex_wrap(wrap)
}

#[derive(Props, Clone, PartialEq)]
pub struct StackProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: SxInput,
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
    let stack_class = crate::context::use_sx(&STACK_BASE_SX, crate::SxLayer::Framework);
    let dynamic_class =
        crate::context::use_sx(&stack_dynamic_sx(&props), crate::SxLayer::UserDynamic);

    let class = classes(props.class, stack_class);
    let class = classes(class, dynamic_class);

    rsx! {
        Box {
            attributes: props.attributes,
            class: class,
            sx: props.sx,
            states: props.states,
            variables: Vec::new(),
            {props.children}
        }
    }
}
