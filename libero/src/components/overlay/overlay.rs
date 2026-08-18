use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props, variables},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
};

const OVERLAY_Z_INDEX: &str = "100";
const OVERLAY_OPACITY: f32 = 0.6;

const OVERLAY_OPACITY_VAR: &str = "--lsx-overlay-opacity";
const OVERLAY_Z_INDEX_VAR: &str = "--lsx-overlay-z-index";
const OVERLAY_BLUR_VAR: &str = "--lsx-overlay-blur";

static OVERLAY_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .z_index(format!("var({OVERLAY_Z_INDEX_VAR}, {OVERLAY_Z_INDEX})"))
        .background(format!(
            "rgba(0, 0, 0, var({OVERLAY_OPACITY_VAR}, {OVERLAY_OPACITY}))"
        ))
        .backdrop_filter(format!("var({OVERLAY_BLUR_VAR}, none)"))
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
        .with(
            OVERLAY_OPACITY_VAR,
            props.opacity.as_ref().and_then(|v| v.resolve(None)),
        )
        .with(
            OVERLAY_Z_INDEX_VAR,
            props.z_index.as_ref().and_then(|v| v.resolve(None)),
        )
        .with(
            OVERLAY_BLUR_VAR,
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
