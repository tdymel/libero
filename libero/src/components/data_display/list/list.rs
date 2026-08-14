use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
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
    let framework_class = crate::context::use_sx(&LIST_BASE_SX, crate::SxLayer::Framework);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));

    let class = class_list([props.class, framework_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    rsx! {
        ul {
            class: class,
            "data-state": data_state,
            ..props.attributes,
            {props.children}
        }
    }
}

