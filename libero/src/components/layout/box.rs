use dioxus::prelude::*;

use crate::{
    SxLayer,
    components::{Input, States, common::class_list},
    context::use_sx,
    sx::Sx,
};

#[derive(Props, Clone, PartialEq)]
pub struct BoxProps {
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
pub fn Box(props: BoxProps) -> Element {
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, SxLayer::UserStatic));

    let data_state = props.states.as_ref().and_then(States::data_state);

    let class = class_list([props.class, static_class]);

    rsx! {
        div {
            class: class,
            "data-state": data_state,
            ..props.attributes,
            {props.children}
        }
    }
}
