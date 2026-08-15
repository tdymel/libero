use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States},
    sx::{StaticSx, Sx, sx},
};

static LIST_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .list_style("none")
        .margin("0")
        .padding("0")
        // Nested Lists (rendered inside a ListItem) are indented relative to
        // their own content.
        .selector("& ul", sx().padding_left("16px"))
});

#[derive(Props, Clone, PartialEq)]
pub struct ListProps {
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
pub fn List(props: ListProps) -> Element {
    rsx! {
        Box {
            component: "ul",
            class: props.class,
            sx: props.sx,
            states: props.states,
            framework_sx: &LIST_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
