use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    context::use_sx,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
};

const OVERLAY_Z_INDEX: &str = "100";
const OVERLAY_OPACITY: f32 = 0.6;

static OVERLAY_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .z_index(OVERLAY_Z_INDEX)
});

#[derive(Props, Clone, PartialEq)]
pub struct OverlayProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default, into)]
    z_index: Input<ThemeAwareValue>,
    #[props(default)]
    opacity: Option<f32>,
    #[props(default)]
    blur: Option<f32>,
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    children: Option<Element>,
}

/// Dims/blurs whatever is behind it. Callers control whether it exists by
/// conditionally rendering it, not by passing an `open` flag.
#[component]
pub fn Overlay(props: OverlayProps) -> Element {
    let framework_class = use_sx(&OVERLAY_BASE_SX, crate::SxLayer::Framework);

    let opacity = props.opacity.unwrap_or(OVERLAY_OPACITY);
    let dynamic_sx = sx()
        .background(format!("rgba(0, 0, 0, {opacity})"))
        .apply_if(props.z_index.as_ref(), |sx, z_index| {
            sx.z_index(z_index.clone())
        })
        .apply_if(props.blur, |sx, blur| {
            sx.backdrop_filter(format!("blur({blur}px)"))
        });
    let dynamic_class = use_sx(&dynamic_sx, crate::SxLayer::UserDynamic);

    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, crate::SxLayer::UserStatic));

    let class = class_list([props.class, framework_class, dynamic_class, static_class]);

    let data_state = props.states.as_ref().and_then(States::data_state);

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
