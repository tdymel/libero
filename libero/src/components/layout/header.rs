use dioxus::prelude::*;

use crate::{
    components::{
        Box, HtmlTag, Input, States, Variables,
        common::{base_props, input_from_str},
        variables,
    },
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorShade, ColorValue, CssVar, HEADER_HEIGHT, Size, Z_INDEX_HEADER},
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

// Matches Button's own default shade - bold enough for a solid brand-color
// banner rather than the library-wide default (5).
const HEADER_DEFAULT_SHADE: ColorShade = ColorShade::S6;

// Unlike `Icon`/`Button`, an unset `color` means "leave the neutral default
// alone" rather than falling back to a theme color - a bare theme color name
// has no shade of its own, so it's resolved to our own default shade here;
// anything else (an explicit shade/contrast, or a literal value like
// "red"/#123456/rgb(...)) passes through unchanged for the sx-to-css
// pipeline to resolve normally.
fn header_base_color(value: Option<&ThemeAwareValue>) -> Option<ThemeAwareValue> {
    match value {
        None => None,
        Some(ThemeAwareValue::Color(color)) => Some(ThemeAwareValue::ColorValue(
            ColorValue::Shade(*color, HEADER_DEFAULT_SHADE),
        )),
        Some(other) => Some(other.clone()),
    }
}

// Only a resolved theme shade has a precomputed contrast CSS var to pair
// with; a literal color has no such pairing available.
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
        .background(HEADER_BACKGROUND_VAR.value_or("white"))
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
    let contrast = base.as_ref().and_then(header_contrast_color);

    variables()
        .with(
            HEADER_HEIGHT.override_var(),
            props.size.resolve(Some(HEADER_HEIGHT)),
        )
        .with(
            HEADER_BACKGROUND_VAR,
            base.as_ref().and_then(|v| v.resolve(None)),
        )
        .with(
            HEADER_COLOR_VAR,
            contrast.as_ref().and_then(|v| v.resolve(None)),
        )
        .with(Z_INDEX_HEADER.override_var(), props.z_index.resolve(None))
}

base_props! {
    pub struct HeaderProps {
        /// `Sticky` (default) stays visible while scrolling with no offset
        /// needed; `Fixed` is viewport-relative but requires you to offset your
        /// own content, same caveat as `Drawer`'s `anchor`.
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

/// The page's `banner` landmark - always renders `<header>`. Hosts a nav and
/// actions as children rather than being scoped to either itself.
#[component]
pub fn Header(props: HeaderProps) -> Element {
    let position = props.position.copied_or_default();
    let variables = header_variables(&props);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("static", position == HeaderPosition::Static)
        .with("fixed", position == HeaderPosition::Fixed);

    rsx! {
        Box {
            component: HtmlTag::Header,
            class: props.class,
            sx: props.sx,
            states,
            variables,
            framework_sx: &HEADER_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
