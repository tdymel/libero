use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
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
    let framework_class = crate::context::use_sx(&LIST_ITEM_BASE_SX, crate::SxLayer::Framework);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));

    let class = class_list([props.class, framework_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    rsx! {
        li {
            class: class,
            "data-state": data_state,
            onclick: move |event| props.onclick.call(event),
            ..props.attributes,
            {props.children}
        }
    }
}
