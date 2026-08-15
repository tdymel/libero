use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::class_list},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::ContainerDefaults,
};

static CONTAINER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .height("100%")
        .margin_left("auto")
        .margin_right("auto")
        .and(ContainerDefaults::default_sx())
        .focus_visible(
            sx().outline("2px solid var(--lsx-primary-6)")
                .outline_offset("2px"),
        )
});

fn container_dynamic_sx(props: &ContainerProps) -> Sx {
    sx().apply_if(props.size.as_ref(), |sx, size| sx.max_width(size.clone()))
        .apply_if(props.gutters.as_ref(), |sx, gutters| {
            sx.padding_left(gutters.clone())
                .padding_right(gutters.clone())
        })
}

#[derive(Props, Clone, PartialEq)]
pub struct ContainerProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default, into)]
    size: Input<ThemeAwareValue>,
    #[props(default, into)]
    gutters: Input<ThemeAwareValue>,
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    children: Element,
}

#[component]
pub fn Container(props: ContainerProps) -> Element {
    let dynamic_class =
        crate::hooks::use_sx(&container_dynamic_sx(&props), crate::SxLayer::UserDynamic);

    rsx! {
        Box {
            class: class_list([props.class, dynamic_class]),
            sx: props.sx,
            states: props.states,
            framework_sx: &CONTAINER_BASE_SX,
            onclick: props.onclick,
            attributes: props.attributes,
            {props.children}
        }
    }
}
