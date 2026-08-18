use dioxus::prelude::*;

use crate::{
    components::{Box, HtmlTag, Input, States, Variables, common::base_props, variables},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ColorShade, ColorValue, Size, SizeCss},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderPosition {
    Static,
    Sticky,
    Fixed,
}

impl Default for HeaderPosition {
    fn default() -> Self {
        Self::Sticky
    }
}

impl From<&str> for HeaderPosition {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "static" => Self::Static,
            "fixed" => Self::Fixed,
            _ => Self::Sticky,
        }
    }
}

impl From<String> for HeaderPosition {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<HeaderPosition> {
    fn from(value: &str) -> Self {
        Input::Value(HeaderPosition::from(value))
    }
}

impl From<String> for Input<HeaderPosition> {
    fn from(value: String) -> Self {
        Input::Value(HeaderPosition::from(value))
    }
}

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

const HEADER_HEIGHT_VAR: &str = "--lsx-header-height-override";
const HEADER_BACKGROUND_VAR: &str = "--lsx-header-background";
const HEADER_COLOR_VAR: &str = "--lsx-header-color";
const HEADER_Z_INDEX_VAR: &str = "--lsx-header-z-index";

static HEADER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .width("100%")
        .height(format!(
            "var({HEADER_HEIGHT_VAR}, {})",
            SizeCss::HEADER_HEIGHT.value(Size::Md)
        ))
        .padding_left("md")
        .padding_right("md")
        .background(format!("var({HEADER_BACKGROUND_VAR}, white)"))
        .color(format!("var({HEADER_COLOR_VAR}, inherit)"))
        .border_bottom("1px solid")
        .border_bottom_color("grey.4")
        .z_index(format!("var({HEADER_Z_INDEX_VAR}, auto)"))
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
            HEADER_HEIGHT_VAR,
            props
                .size
                .as_ref()
                .and_then(|v| v.resolve(Some(SizeCss::HEADER_HEIGHT))),
        )
        .with(
            HEADER_BACKGROUND_VAR,
            base.as_ref().and_then(|v| v.resolve(None)),
        )
        .with(
            HEADER_COLOR_VAR,
            contrast.as_ref().and_then(|v| v.resolve(None)),
        )
        .with(
            HEADER_Z_INDEX_VAR,
            props.z_index.as_ref().and_then(|v| v.resolve(None)),
        )
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
    let position = props.position.as_ref().copied().unwrap_or_default();
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
