use dioxus::prelude::*;

use crate::{
    components::{
        Box, HtmlTag, Input, States, Variables,
        common::{base_props, focus_ring_sx, variables},
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CONTAINER_GUTTERS, CONTAINER_SIZE, SizeCss},
};

const CONTAINER_SIZE_VAR: &str = "--lsx-container-size-override";
const CONTAINER_GUTTERS_VAR: &str = "--lsx-container-gutters-override";

static CONTAINER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .height("100%")
        .margin_left("auto")
        .margin_right("auto")
        .max_width(format!(
            "var({CONTAINER_SIZE_VAR}, {})",
            CONTAINER_SIZE.value()
        ))
        .padding_left(format!(
            "var({CONTAINER_GUTTERS_VAR}, {})",
            CONTAINER_GUTTERS.value()
        ))
        .padding_right(format!(
            "var({CONTAINER_GUTTERS_VAR}, {})",
            CONTAINER_GUTTERS.value()
        ))
        .focus_visible(focus_ring_sx())
});

// `xs`-`xxl` resolve through the same breakpoint/spacing scales the theme
// default itself uses, just a different step; anything else passes through.
fn container_size(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Size(size) => SizeCss::BREAKPOINT.value(*size).into(),
        other => other.clone(),
    }
}

fn container_gutters(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Size(size) => SizeCss::SPACING.value(*size).into(),
        other => other.clone(),
    }
}

fn container_variables(props: &ContainerProps) -> Variables {
    variables()
        .with(
            CONTAINER_SIZE_VAR,
            props
                .size
                .as_ref()
                .map(container_size)
                .and_then(|v| v.raw()),
        )
        .with(
            CONTAINER_GUTTERS_VAR,
            props
                .gutters
                .as_ref()
                .map(container_gutters)
                .and_then(|v| v.raw()),
        )
}

base_props! {
    pub struct ContainerProps {
        /// Which element to render as - `div` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        gutters: Input<ThemeAwareValue>,
        children: Element,
    }
}

#[component]
pub fn Container(props: ContainerProps) -> Element {
    let variables = container_variables(&props);

    rsx! {
        Box {
            component: props.component,
            class: props.class,
            sx: props.sx,
            states: props.states,
            variables,
            framework_sx: &CONTAINER_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
