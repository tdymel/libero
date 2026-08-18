use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States, Variables,
        common::{base_props, variables},
    },
    sx::{StaticSx, Sx, sx},
    theme::ASPECT_RATIO,
};

const ASPECT_RATIO_VAR: &str = "--lsx-aspect-ratio-override";

static ASPECT_RATIO_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().aspect_ratio(format!("var({ASPECT_RATIO_VAR}, {})", ASPECT_RATIO.value()))
        .overflow("hidden")
        .selector("& > *", sx().width("100%").height("100%"))
});

fn aspect_ratio_variables(ratio: Option<&f32>) -> Variables {
    variables().with(ASPECT_RATIO_VAR, ratio.map(f32::to_string))
}

base_props! {
    pub struct AspectRatioProps {
        /// Width-to-height ratio, e.g. `16.0 / 9.0` - `1` by default.
        #[props(default, into)]
        ratio: Input<f32>,
        children: Element,
    }
}

#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    let variables = aspect_ratio_variables(props.ratio.as_ref());

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states: props.states,
            variables,
            framework_sx: &ASPECT_RATIO_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
