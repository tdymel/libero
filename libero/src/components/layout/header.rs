use dioxus::prelude::*;

use crate::{
    components::{
        Box, HtmlTag, Input, States,
        common::{base_props, class_list},
    },
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

fn header_size(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Size(size) => SizeCss::HEADER_HEIGHT.value(*size).into(),
        other => other.clone(),
    }
}

fn header_position_sx(position: HeaderPosition) -> Sx {
    match position {
        HeaderPosition::Static => sx().position("static"),
        HeaderPosition::Sticky => sx().position("sticky").top("0"),
        HeaderPosition::Fixed => sx().position("fixed").top("0"),
    }
}

static HEADER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .width("100%")
        .height(SizeCss::HEADER_HEIGHT.value(Size::Md))
        .padding_left("md")
        .padding_right("md")
        .background("white")
        .border_bottom("1px solid")
        .border_bottom_color("grey.3")
});

fn header_dynamic_sx(props: &HeaderProps) -> Sx {
    let position = props.position.as_ref().copied().unwrap_or_default();

    header_position_sx(position)
        .apply_if(props.size.as_ref().map(header_size), |sx, size| {
            sx.height(size)
        })
        .apply_if(header_base_color(props.color.as_ref()), |sx, base| {
            let contrast = header_contrast_color(&base);
            let sx = sx.background(base);
            match contrast {
                Some(contrast) => sx.color(contrast),
                None => sx,
            }
        })
        .apply_if(props.z_index.as_ref(), |sx, z_index| {
            sx.z_index(z_index.clone())
        })
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
    let dynamic_class =
        crate::hooks::use_css(&header_dynamic_sx(&props), crate::CssLayer::UserDynamic);

    rsx! {
        Box {
            component: HtmlTag::Header,
            class: class_list([props.class, dynamic_class]),
            sx: props.sx,
            states: props.states,
            framework_sx: &HEADER_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
