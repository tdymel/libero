use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::ContainerDefaults,
};

static CONTAINER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .height("100%")
        .margin_left("auto")
        .margin_right("auto")
        .and(ContainerDefaults::default_sx())
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
    children: Element,
}

#[component]
pub fn Container(props: ContainerProps) -> Element {
    let base_class = crate::context::use_sx(&CONTAINER_BASE_SX, crate::SxLayer::Framework);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));
    let dynamic_class =
        crate::context::use_sx(&container_dynamic_sx(&props), crate::SxLayer::UserDynamic);

    let class = class_list([props.class, base_class, dynamic_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    rsx! {
        div {
            class: class,
            "data-state": data_state,
            ..props.attributes,
            {props.children}
        }
    }
}
