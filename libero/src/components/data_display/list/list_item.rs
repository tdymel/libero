use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States},
    sx::{StaticSx, Sx, sx},
};

// Block-level, no flex - a nested `List` placed among this item's own
// content must stack below it, not sit beside it in a row. Row-alignment
// (e.g. an icon next to a label) is the job of whatever's placed inside,
// same as `NavLink` already does for itself.
static LIST_ITEM_BASE_SX: StaticSx = StaticSx::new(|| sx().padding("0"));

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
            attributes: props.attributes,
            {props.children}
        }
    }
}
