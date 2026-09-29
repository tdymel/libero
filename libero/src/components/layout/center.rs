use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, Variables, base_props, variables},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::CENTER_DISPLAY,
};

static CENTER_BASE_SX: StaticSx = StaticSx::new(|| {
    // `safe`: an oversized child spills toward the end, where scrolling reaches it.
    sx().display(CENTER_DISPLAY.overridable())
        .align_items("safe center")
        .justify_content("safe center")
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
        /// `inline-flex`, so it doesn't stretch to the parent's width.
        #[props(default)]
        inline: Option<bool>,
        children: Element,
    }
}

/// Centers its children on both axes.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Center;
/// # fn app() -> Element {
/// rsx! {
///     Center { "Nothing here yet" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/center>
#[component]
pub fn Center(props: CenterProps) -> Element {
    let variables: Input<Variables> = center_variables(&props).into();

    use_box()
        .framework_sx(&CENTER_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Div, props.attributes, props.children)
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

    /// Unset leaves it to the theme: no override var pinned to `flex`.
    #[test]
    fn unset_emits_no_override() {
        assert_eq!(center_variables(&center_props(None)).to_string(), "");
    }
}
