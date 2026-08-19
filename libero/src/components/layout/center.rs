use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, Variables, common::base_props, variables},
    sx::{StaticSx, Sx, sx},
    theme::CENTER_DISPLAY,
};

static CENTER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display(CENTER_DISPLAY.overridable())
        .align_items("center")
        .justify_content("center")
});

fn center_variables(props: &CenterProps) -> Variables {
    variables().with(
        CENTER_DISPLAY.override_var(),
        props
            .inline
            .map(|inline| if inline { "inline-flex" } else { "flex" }.to_string()),
    )
}

base_props! {
    pub struct CenterProps {
        /// `inline-flex` instead of `flex`, so it doesn't stretch to the
        /// parent's width.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn center_props(inline: Option<bool>) -> CenterProps {
        CenterProps {
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            inline,
            children: rsx! {},
        }
    }

    #[test]
    fn inline_picks_the_display_mode() {
        assert_eq!(
            center_variables(&center_props(Some(true))).to_string(),
            format!("{}:inline-flex;", CENTER_DISPLAY.override_var().name())
        );
        assert_eq!(
            center_variables(&center_props(Some(false))).to_string(),
            format!("{}:flex;", CENTER_DISPLAY.override_var().name())
        );
    }

    /// Unset means "leave it to the theme", so the override var must stay
    /// absent rather than being pinned to `flex`.
    #[test]
    fn unset_emits_no_override() {
        assert_eq!(center_variables(&center_props(None)).to_string(), "");
    }
}
