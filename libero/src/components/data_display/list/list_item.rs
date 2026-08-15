use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States},
    sx::{StaticSx, Sx, sx},
};

static LIST_ITEM_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .padding("0")
        .focus_visible(
            sx().outline("2px solid var(--lsx-primary-6)")
                .outline_offset("2px"),
        )
});

#[derive(Props, Clone, PartialEq)]
pub struct ListItemProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    children: Element,
}

#[component]
pub fn ListItem(props: ListItemProps) -> Element {
    rsx! {
        Box {
            component: "li",
            class: props.class,
            sx: props.sx,
            states: props.states,
            framework_sx: &LIST_ITEM_BASE_SX,
            onclick: props.onclick,
            attributes: props.attributes,
            {props.children}
        }
    }
}
