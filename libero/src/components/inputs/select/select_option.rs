use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    sx::Sx,
};

base_props! {
    extends(option);
    pub struct OptionProps {
        #[props(into)]
        value: String,
        children: Element,
    }
}

#[component]
pub fn Option(props: OptionProps) -> Element {
    rsx! {
        Box {
            component: "option",
            class: props.class,
            sx: props.sx,
            states: props.states,
            value: props.value,
            attributes: props.attributes,
            {props.children}
        }
    }
}
