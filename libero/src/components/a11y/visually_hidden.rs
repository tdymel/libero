use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    sx::{StaticSx, Sx, sx},
};

pub(crate) static VISUALLY_HIDDEN_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .width("1px")
        .height("1px")
        .padding("0")
        .margin("-1px")
        .overflow("hidden")
        .clip("rect(0, 0, 0, 0)")
        .white_space("nowrap")
        .border_width("0")
});

base_props! {
    pub struct VisuallyHiddenProps {
        children: Element,
    }
}

#[component]
pub fn VisuallyHidden(props: VisuallyHiddenProps) -> Element {
    rsx! {
        Box {
            component: "span",
            class: props.class,
            sx: props.sx,
            states: props.states,
            framework_sx: &VISUALLY_HIDDEN_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
