use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, Variables, common::base_props, variables},
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{FLOAT_OFFSET_X, FLOAT_OFFSET_Y, FLOAT_Z_INDEX},
};

pub use crate::theme::Placement;

impl From<&str> for Input<Placement> {
    fn from(value: &str) -> Self {
        Input::Value(Placement::from(value))
    }
}

impl From<String> for Input<Placement> {
    fn from(value: String) -> Self {
        Input::Value(Placement::from(value))
    }
}

const FLOAT_OFFSET_X_VAR: &str = "--lsx-float-offset-x";
const FLOAT_OFFSET_Y_VAR: &str = "--lsx-float-offset-y";
const FLOAT_Z_INDEX_VAR: &str = "--lsx-float-z-index-override";
const FLOAT_TRANSLATE_X_VAR: &str = "--lsx-float-translate-x";
const FLOAT_TRANSLATE_Y_VAR: &str = "--lsx-float-translate-y";

static FLOAT_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .z_index(format!(
            "var({FLOAT_Z_INDEX_VAR}, {})",
            FLOAT_Z_INDEX.value()
        ))
        .transform(format!(
            "translate(calc(var({FLOAT_TRANSLATE_X_VAR}, 0%) + var({FLOAT_OFFSET_X_VAR}, {})), calc(var({FLOAT_TRANSLATE_Y_VAR}, 0%) + var({FLOAT_OFFSET_Y_VAR}, {})))",
            FLOAT_OFFSET_X.value(),
            FLOAT_OFFSET_Y.value(),
        ))
        .when("vertical-top", sx().top("0"))
        .when(
            "vertical-center",
            sx().top("50%").with(FLOAT_TRANSLATE_Y_VAR, "-50%"),
        )
        .when("vertical-bottom", sx().bottom("0"))
        .when("horizontal-start", sx().left("0"))
        .when(
            "horizontal-center",
            sx().left("50%").with(FLOAT_TRANSLATE_X_VAR, "-50%"),
        )
        .when("horizontal-end", sx().right("0"))
});

fn float_variables(props: &FloatProps) -> Variables {
    variables()
        .with(
            FLOAT_OFFSET_X_VAR,
            props.offset_x.as_ref().and_then(ThemeAwareValue::raw),
        )
        .with(
            FLOAT_OFFSET_Y_VAR,
            props.offset_y.as_ref().and_then(ThemeAwareValue::raw),
        )
        .with(
            FLOAT_Z_INDEX_VAR,
            props.z_index.as_ref().and_then(ThemeAwareValue::raw),
        )
}

base_props! {
    pub struct FloatProps {
        /// Anchor corner/edge, e.g. `"top-start"` - defaults to the theme's
        /// `float.placement` setting.
        #[props(default, into)]
        placement: Input<Placement>,
        /// Shift along the horizontal axis - defaults to the theme's
        /// `float.offset_x` setting.
        #[props(default, into)]
        offset_x: Input<ThemeAwareValue>,
        /// Shift along the vertical axis - defaults to the theme's
        /// `float.offset_y` setting.
        #[props(default, into)]
        offset_y: Input<ThemeAwareValue>,
        /// Defaults to the theme's `float.z_index` setting.
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
    let placement = props
        .placement
        .as_ref()
        .copied()
        .unwrap_or(theme.float.placement);
    let variables = float_variables(&props);

    let states = props
        .states
        .as_ref()
        .cloned()
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
        );

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states,
            variables,
            framework_sx: &FLOAT_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
