use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, Variables, common::base_props, variables},
    sx::{StaticSx, Sx, sx},
    theme::CENTER_DISPLAY,
};

const CENTER_DISPLAY_VAR: &str = "--lsx-center-display-override";

static CENTER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display(format!(
        "var({CENTER_DISPLAY_VAR}, {})",
        CENTER_DISPLAY.value()
    ))
    .align_items("center")
    .justify_content("center")
});

fn center_variables(props: &CenterProps) -> Variables {
    variables().with(
        CENTER_DISPLAY_VAR,
        props
            .inline
            .map(|inline| if inline { "inline-flex" } else { "flex" }.to_string()),
    )
}

base_props! {
    pub struct CenterProps {
        /// Uses `inline-flex` instead of `flex` - for centering inline
        /// content without stretching to fill the parent's width. Defaults
        /// to the theme's `center.inline` setting.
        #[props(default)]
        inline: Option<bool>,
        children: Element,
    }
}

#[component]
pub fn Center(props: CenterProps) -> Element {
    let variables = center_variables(&props);

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states: props.states,
            variables,
            framework_sx: &CENTER_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
