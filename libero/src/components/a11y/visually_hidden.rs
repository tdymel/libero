use dioxus::prelude::*;

use crate::{
    SxLayer,
    components::{Input, States, common::class_list},
    context::use_sx,
    sx::{StaticSx, Sx, sx},
};

static VISUALLY_HIDDEN_SX: StaticSx = StaticSx::new(|| {
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

#[derive(Props, Clone, PartialEq)]
pub struct VisuallyHiddenProps {
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
pub fn VisuallyHidden(props: VisuallyHiddenProps) -> Element {
    let framework_class = use_sx(&VISUALLY_HIDDEN_SX, SxLayer::Framework);

    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, SxLayer::UserStatic));

    let data_state = props.states.as_ref().and_then(States::data_state);

    let class = class_list([props.class, framework_class, static_class]);

    rsx! {
        span {
            class: class,
            "data-state": data_state,
            ..props.attributes,
            {props.children}
        }
    }
}
