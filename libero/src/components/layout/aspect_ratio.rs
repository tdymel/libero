use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, Variables,
        common::{base_props, variables},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::ASPECT_RATIO,
};

static ASPECT_RATIO_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().aspect_ratio(ASPECT_RATIO.overridable())
        .overflow("hidden")
        .selector("& > *", sx().width("100%").height("100%"))
});

fn aspect_ratio_variables(ratio: Option<&f32>) -> Variables {
    variables().with(ASPECT_RATIO.override_var(), ratio.map(f32::to_string))
}

base_props! {
    pub struct AspectRatioProps {
        /// Width-to-height ratio, e.g. `16.0 / 9.0`.
        #[props(default, into)]
        ratio: Input<f32>,
        children: Element,
    }
}

#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    let variables: Input<Variables> = aspect_ratio_variables(props.ratio.as_ref()).into();

    use_box()
        .framework_sx(&ASPECT_RATIO_BASE_SX)
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

    /// Sets the `-override` twin: the base var is the theme default this has
    /// to beat.
    #[test]
    fn a_ratio_sets_the_override_variable() {
        let variables = aspect_ratio_variables(Some(&1.5));

        assert_eq!(
            variables.to_string(),
            format!("{}:1.5;", ASPECT_RATIO.override_var().name())
        );
    }

    #[test]
    fn no_ratio_emits_no_variable() {
        assert_eq!(aspect_ratio_variables(None).to_string(), "");
    }
}
