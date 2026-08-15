use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States},
    sx::{StaticSx, Sx, sx},
};

static CODE_INLINE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline")
        .background("grey.1")
        .border_radius("4px")
        .padding("2px 6px")
        .font_family("ui-monospace, SFMono-Regular, Menlo, Consolas, monospace")
        .font_size("0.875em")
});

static CODE_BLOCK_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .margin("0")
        .background("grey.1")
        .border("1px solid")
        .border_color("grey.3")
        .border_radius("6px")
        .padding("12px 16px")
        .overflow("auto")
        .font_family("ui-monospace, SFMono-Regular, Menlo, Consolas, monospace")
        .font_size("0.875rem")
});

#[derive(Props, Clone, PartialEq)]
pub struct CodeProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    /// Renders as a `pre`-wrapped, multi-line block instead of inline `code`.
    #[props(default)]
    block: bool,
    children: Element,
}

#[component]
pub fn Code(props: CodeProps) -> Element {
    if props.block {
        rsx! {
            Box {
                component: "pre",
                class: props.class,
                sx: props.sx,
                states: props.states,
                framework_sx: &CODE_BLOCK_SX,
                attributes: props.attributes,
                Box { component: "code", {props.children} }
            }
        }
    } else {
        rsx! {
            Box {
                component: "code",
                class: props.class,
                sx: props.sx,
                states: props.states,
                framework_sx: &CODE_INLINE_SX,
                attributes: props.attributes,
                {props.children}
            }
        }
    }
}
