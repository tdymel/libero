use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props, variables},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{OVERLAY_BLUR, OVERLAY_OPACITY, Z_INDEX_OVERLAY},
};

static OVERLAY_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .z_index(Z_INDEX_OVERLAY.overridable())
        .background(format!("rgba(0, 0, 0, {})", OVERLAY_OPACITY.overridable()))
        .backdrop_filter(OVERLAY_BLUR.overridable())
});

base_props! {
    pub struct OverlayProps {
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        #[props(default, into)]
        opacity: Input<ThemeAwareValue>,
        #[props(default, into)]
        blur: Input<ThemeAwareValue>,
        children: Option<Element>,
    }
}

/// A CSS pixel length for values with no unit of their own (numbers), passed
/// through unchanged otherwise (strings, CSS vars).
fn px_value(value: &ThemeAwareValue) -> Option<String> {
    match value {
        ThemeAwareValue::Number(number) => Some(format!("{number}px")),
        _ => value.resolve(None),
    }
}

/// Dims/blurs whatever is behind it. Callers control whether it exists by
/// conditionally rendering it, not by passing an `open` flag.
#[component]
pub fn Overlay(props: OverlayProps) -> Element {
    let variables = variables()
        .with(OVERLAY_OPACITY.override_var(), props.opacity.resolve(None))
        .with(Z_INDEX_OVERLAY.override_var(), props.z_index.resolve(None))
        .with(
            OVERLAY_BLUR.override_var(),
            props
                .blur
                .as_ref()
                .and_then(px_value)
                .map(|blur| format!("blur({blur})")),
        );

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states: props.states,
            variables,
            framework_sx: &OVERLAY_BASE_SX,
            attributes: props.attributes,
            {props.children.unwrap_or_else(|| rsx! {})}
        }
    }
}
