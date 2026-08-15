use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States},
    sx::{StaticSx, Sx, sx},
};

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
pub struct CodeBlockProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    children: Element,
}

#[component]
pub fn CodeBlock(props: CodeBlockProps) -> Element {
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
}
