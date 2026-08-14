use dioxus::prelude::*;

use crate::{
    SxLayer,
    components::{Input, States, common::class_list},
    context::use_sx,
    sx::{StaticSx, Sx, sx},
};

// Only ever shows up for a Box that received a tabindex (e.g. because it was
// made clickable via `onclick`), since a plain div isn't keyboard-focusable
// on its own.
static BOX_FOCUS_SX: StaticSx = StaticSx::new(|| {
    sx().focus_visible(
        sx().outline("2px solid var(--lsx-primary-6)")
            .outline_offset("2px"),
    )
});

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
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    children: Element,
}

#[component]
pub fn Box(props: BoxProps) -> Element {
    let focus_class = use_sx(&BOX_FOCUS_SX, SxLayer::Framework);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, SxLayer::UserStatic));

    let data_state = props.states.as_ref().and_then(States::data_state);

    let class = class_list([props.class, focus_class, static_class]);

    rsx! {
        div {
            class: class,
            "data-state": data_state,
            onclick: move |event| props.onclick.call(event),
            ..props.attributes,
            {props.children}
        }
    }
}
