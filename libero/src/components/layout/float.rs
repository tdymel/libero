use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, input_from_str},
        layout::use_box,
        variables,
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, FLOAT_OFFSET_X, FLOAT_OFFSET_Y, SizeCss, Z_INDEX_FLOAT},
};

pub use crate::theme::Placement;

input_from_str!(Placement);

const FLOAT_OFFSET_X_VAR: CssVar = CssVar::new("--lsx-float-offset-x");
const FLOAT_OFFSET_Y_VAR: CssVar = CssVar::new("--lsx-float-offset-y");
const FLOAT_TRANSLATE_X_VAR: CssVar = CssVar::new("--lsx-float-translate-x");
const FLOAT_TRANSLATE_Y_VAR: CssVar = CssVar::new("--lsx-float-translate-y");

static FLOAT_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .z_index(Z_INDEX_FLOAT.overridable())
        .transform(format!(
            "translate(calc({} + {}), calc({} + {}))",
            FLOAT_TRANSLATE_X_VAR.value_or("0%"),
            FLOAT_OFFSET_X_VAR.value_or(FLOAT_OFFSET_X.value()),
            FLOAT_TRANSLATE_Y_VAR.value_or("0%"),
            FLOAT_OFFSET_Y_VAR.value_or(FLOAT_OFFSET_Y.value()),
        ))
        .when("vertical-top", sx().top("0"))
        .when(
            "vertical-center",
            sx().top("50%").var(FLOAT_TRANSLATE_Y_VAR, "-50%"),
        )
        .when("vertical-bottom", sx().bottom("0"))
        .when("horizontal-start", sx().left("0"))
        .when(
            "horizontal-center",
            sx().left("50%").var(FLOAT_TRANSLATE_X_VAR, "-50%"),
        )
        .when("horizontal-end", sx().right("0"))
});

fn float_variables(props: &FloatProps) -> Variables {
    variables()
        // Through the spacing scale, so a size token is an offset like
        // anywhere else - `resolve(None)` dropped one silently.
        .with(
            FLOAT_OFFSET_X_VAR,
            props.offset_x.resolve(Some(SizeCss::SPACING)),
        )
        .with(
            FLOAT_OFFSET_Y_VAR,
            props.offset_y.resolve(Some(SizeCss::SPACING)),
        )
        .with(Z_INDEX_FLOAT.override_var(), props.z_index.resolve(None))
}

base_props! {
    pub struct FloatProps {
        /// Anchor corner/edge, e.g. `"top-start"`.
        #[props(default, into)]
        placement: Input<Placement>,
        /// Shift along the horizontal axis.
        #[props(default, into)]
        offset_x: Input<ThemeAwareValue>,
        /// Shift along the vertical axis.
        #[props(default, into)]
        offset_y: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        children: Element,
    }
}

/// Anchors `children` to a corner/edge of the nearest `position: relative`
/// ancestor. The parent must set `position: relative` itself.
#[component]
pub fn Float(props: FloatProps) -> Element {
    let theme = use_theme();
    let placement = props.placement.copied_or(theme.float.placement);
    let variables: Input<Variables> = float_variables(&props).into();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(
            "vertical-top",
            matches!(
                placement,
                Placement::TopStart | Placement::TopCenter | Placement::TopEnd
            ),
        )
        .with(
            "vertical-center",
            matches!(
                placement,
                Placement::CenterStart | Placement::CenterCenter | Placement::CenterEnd
            ),
        )
        .with(
            "vertical-bottom",
            matches!(
                placement,
                Placement::BottomStart | Placement::BottomCenter | Placement::BottomEnd
            ),
        )
        .with(
            "horizontal-start",
            matches!(
                placement,
                Placement::TopStart | Placement::CenterStart | Placement::BottomStart
            ),
        )
        .with(
            "horizontal-center",
            matches!(
                placement,
                Placement::TopCenter | Placement::CenterCenter | Placement::BottomCenter
            ),
        )
        .with(
            "horizontal-end",
            matches!(
                placement,
                Placement::TopEnd | Placement::CenterEnd | Placement::BottomEnd
            ),
        )
        .into();

    use_box()
        .framework_sx(&FLOAT_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Div, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::Size;

    fn props(offset_x: Input<ThemeAwareValue>) -> FloatProps {
        FloatProps {
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            placement: Input::None,
            offset_x,
            offset_y: Input::None,
            z_index: Input::None,
            children: rsx! {},
        }
    }

    /// A size token has no meaning without a scale, and `resolve(None)` used
    /// to drop it - the offset silently did nothing.
    #[test]
    fn a_size_offset_resolves_through_the_spacing_scale() {
        let variables = float_variables(&props(Size::Md.into()));

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};",
                FLOAT_OFFSET_X_VAR.name(),
                SizeCss::SPACING.value(Size::Md)
            )
        );
    }

    #[test]
    fn a_css_length_offset_passes_through() {
        let variables = float_variables(&props(Input::from("-8px")));

        assert_eq!(
            variables.to_string(),
            format!("{}:-8px;", FLOAT_OFFSET_X_VAR.name())
        );
    }
}
