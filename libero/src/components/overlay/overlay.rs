use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::class_list},
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
    #[props(default, into)]
    opacity: Input<ThemeAwareValue>,
    #[props(default, into)]
    blur: Input<ThemeAwareValue>,
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    children: Option<Element>,
}

/// A CSS pixel length for values with no unit of their own (numbers), passed
/// through unchanged otherwise (strings, CSS vars).
fn px_value(value: &ThemeAwareValue) -> Option<String> {
    match value {
        ThemeAwareValue::Number(number) => Some(format!("{number}px")),
        _ => value.raw(),
    }
}

/// Dims/blurs whatever is behind it. Callers control whether it exists by
/// conditionally rendering it, not by passing an `open` flag.
#[component]
pub fn Overlay(props: OverlayProps) -> Element {
    let opacity = props
        .opacity
        .as_ref()
        .and_then(ThemeAwareValue::raw)
        .unwrap_or_else(|| OVERLAY_OPACITY.to_string());
    let dynamic_sx = sx()
        .background(format!("rgba(0, 0, 0, {opacity})"))
        .apply_if(props.z_index.as_ref(), |sx, z_index| {
            sx.z_index(z_index.clone())
        })
        .apply_if(props.blur.as_ref().and_then(px_value), |sx, blur| {
            sx.backdrop_filter(format!("blur({blur})"))
        });
    let dynamic_class = use_sx(&dynamic_sx, crate::SxLayer::UserDynamic);
    let class = class_list([props.class, dynamic_class]);

    rsx! {
        Box {
            class: class,
            sx: props.sx,
            states: props.states,
            framework_sx: &OVERLAY_BASE_SX,
            onclick: props.onclick,
            attributes: props.attributes,
            {props.children.unwrap_or_else(|| rsx! {})}
        }
    }
}
