use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, fill_color, input_from_str},
        layout::use_box,
        variables,
    },
    str_enum::str_enum,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        ColorShade, ColorValue, CssVar, HEADER_HEIGHT, NamedColorCss, PAPER_BACKGROUND, Size,
        Z_INDEX_HEADER,
    },
};

str_enum! {
    pub enum HeaderPosition {
        Static = "static",
        #[default]
        Sticky = "sticky",
        Fixed = "fixed",
    }
}

input_from_str!(HeaderPosition);

// Matches Button's shade: bold enough for a solid brand-color banner.
const HEADER_DEFAULT_SHADE: ColorShade = ColorShade::S6;

// Unlike `Icon`/`Button`, an unset `color` keeps the neutral default rather
// than falling back to a theme color. A bare color name takes the shade
// above; everything else passes through.
fn header_base_color(value: Option<&ThemeAwareValue>) -> Option<ThemeAwareValue> {
    match value {
        None => None,
        Some(ThemeAwareValue::Color(color)) => Some(ThemeAwareValue::ColorValue(
            ColorValue::Shade(*color, HEADER_DEFAULT_SHADE),
        )),
        Some(other) => Some(other.clone()),
    }
}

// Only a resolved theme shade has a precomputed contrast var.
fn header_contrast_color(base: &ThemeAwareValue) -> Option<ThemeAwareValue> {
    match base {
        ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) => Some(
            ThemeAwareValue::ColorValue(ColorValue::Contrast(*color, *shade)),
        ),
        _ => None,
    }
}

const HEADER_BACKGROUND_VAR: CssVar = CssVar::new("--lsx-header-background");
const HEADER_COLOR_VAR: CssVar = CssVar::new("--lsx-header-color");

static HEADER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .width("100%")
        .height(HEADER_HEIGHT.overridable(Size::Md))
        .padding_left("md")
        .padding_right("md")
        .background(HEADER_BACKGROUND_VAR.value_or(PAPER_BACKGROUND.value()))
        .color(HEADER_COLOR_VAR.value_or("inherit"))
        .border_bottom("1px solid")
        .border_bottom_color("grey.4")
        .z_index(Z_INDEX_HEADER.overridable())
        .position("sticky")
        .top("0")
        .when("static", sx().position("static"))
        .when("fixed", sx().position("fixed").top("0"))
});

fn header_variables(props: &HeaderProps) -> Variables {
    let base = header_base_color(props.color.as_ref());
    let contrast = base
        .as_ref()
        .and_then(header_contrast_color)
        .and_then(|v| v.resolve(None));

    variables()
        .with(
            HEADER_HEIGHT.override_var(),
            props.size.resolve(Some(HEADER_HEIGHT)),
        )
        // The banner is a fill under `HEADER_COLOR_VAR`, so it resolves
        // through the fill ramp: a `primary` header used to be `blue.6` with
        // white text on it, 3.56:1 (todo 239).
        .with(HEADER_BACKGROUND_VAR, base.as_ref().and_then(fill_color))
        .with(HEADER_COLOR_VAR, contrast.clone())
        // The background comes through a var, so `sx` cannot publish the
        // focus contrast from it (`codebase/sx`): without this every ring in a
        // coloured header is the primary shade on a primary banner. Only with
        // a `color`, so an uncoloured header inherits the page's.
        .with(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            contrast,
        )
        .with(Z_INDEX_HEADER.override_var(), props.z_index.resolve(None))
}

base_props! {
    pub struct HeaderProps {
        /// `Sticky` (default) needs no offset; `Fixed` is viewport-relative
        /// and you offset your own content, as with `Drawer`'s `anchor`.
        #[props(default, into)]
        position: Input<HeaderPosition>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        children: Element,
    }
}

/// The page's `banner` landmark, always a `<header>`. Hosts nav and actions
/// as children rather than being scoped to either.
#[component]
pub fn Header(props: HeaderProps) -> Element {
    let position = props.position.copied_or_default();
    let variables: Input<Variables> = header_variables(&props).into();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("static", position == HeaderPosition::Static)
        .with("fixed", position == HeaderPosition::Fixed)
        .into();

    use_box()
        .framework_sx(&HEADER_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Header, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::Color;

    fn header_props(color: Input<ThemeAwareValue>) -> HeaderProps {
        HeaderProps {
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            position: Input::None,
            size: Input::None,
            color,
            z_index: Input::None,
            children: rsx! {},
        }
    }

    #[test]
    fn a_theme_color_brings_its_own_contrast_along() {
        let variables = header_variables(&header_props(Color::Primary.into())).to_string();

        assert!(variables.contains(HEADER_BACKGROUND_VAR.name()));
        assert!(variables.contains(HEADER_COLOR_VAR.name()));
        assert!(variables.contains(NamedColorCss::FOCUS_CONTRAST.name()));
    }

    /// Unset means the themed default applies, so neither var is pinned.
    #[test]
    fn no_color_emits_neither_variable() {
        let variables = header_variables(&header_props(Input::None)).to_string();

        assert!(!variables.contains(HEADER_BACKGROUND_VAR.name()));
        assert!(!variables.contains(HEADER_COLOR_VAR.name()));
        assert!(!variables.contains(NamedColorCss::FOCUS_CONTRAST.name()));
    }
}
