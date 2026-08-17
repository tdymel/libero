use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, Variables, common::base_props, variables},
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue},
};

// A light shade keeps the highlight a background tint rather than a solid
// fill, so the surrounding text still reads clearly over it. Which color is
// tinted by default lives in `Theme::mark` (see `MarkDefaults`); only the
// shade itself is fixed here.
const MARK_TINT_SHADE: ColorShade = ColorShade::S1;

const MARK_BACKGROUND_VAR: &str = "--lsx-mark-background";

// A bare theme color name (e.g. "primary") has no shade of its own, so it's
// resolved to our own tint shade here rather than the sx pipeline's generic
// default (5). Anything else - an explicit shade/contrast, or a literal
// value like "yellow"/#123456/rgb(...) - passes through unchanged and is
// resolved by the normal sx-to-css pipeline. Only a genuinely unset `color`
// falls back to the theme's default.
fn mark_background_color(value: Option<&ThemeAwareValue>, default_color: Color) -> ThemeAwareValue {
    match value {
        None => ThemeAwareValue::ColorValue(ColorValue::Shade(default_color, MARK_TINT_SHADE)),
        Some(ThemeAwareValue::Color(color)) => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, MARK_TINT_SHADE))
        }
        Some(other) => other.clone(),
    }
}

// The browser's own UA stylesheet forces `<mark>` to black text, which would
// stay illegible against a dark caller-supplied background - `color:inherit`
// hands text color back to the surrounding context, same as Mantine's Mark
// leaves it untouched and only ever sets the background. The background
// itself always resolves to *some* value (default or override), so no
// fallback is needed on the `var()` reference here - `mark_variables` always
// sets it.
static MARK_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().color("inherit")
        .background(format!("var({MARK_BACKGROUND_VAR})"))
});

fn mark_variables(color: Option<&ThemeAwareValue>, default_color: Color) -> Variables {
    variables().with(
        MARK_BACKGROUND_VAR,
        mark_background_color(color, default_color).resolved(),
    )
}

base_props! {
    pub struct MarkProps {
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        children: Element,
    }
}

/// Highlights `children` with a themed background tint, rendered as a real
/// `<mark>`. `color` picks the tint (any theme color or literal value);
/// unset falls back to a light shade of the theme's default (`Theme::mark`,
/// `warning` out of the box).
#[component]
pub fn Mark(props: MarkProps) -> Element {
    let theme = use_theme();
    let variables = mark_variables(props.color.as_ref(), theme.mark.color);

    rsx! {
        Box {
            component: "mark",
            class: props.class,
            sx: props.sx,
            states: props.states,
            variables,
            framework_sx: &MARK_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
