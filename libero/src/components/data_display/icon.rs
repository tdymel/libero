use dioxus::prelude::*;

use crate::{
    components::{Box, HtmlTag, Input, States, common::class_list},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, Size, SizeCss},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconVariant {
    Filled,
    Outlined,
    Transparent,
}

impl Default for IconVariant {
    fn default() -> Self {
        Self::Filled
    }
}

impl From<&str> for IconVariant {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "outlined" | "outline" => Self::Outlined,
            "transparent" => Self::Transparent,
            _ => Self::Filled,
        }
    }
}

impl From<String> for IconVariant {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<IconVariant> {
    fn from(value: &str) -> Self {
        Input::Value(IconVariant::from(value))
    }
}

impl From<String> for Input<IconVariant> {
    fn from(value: String) -> Self {
        Input::Value(IconVariant::from(value))
    }
}

// Matches Button's own default shade - bold enough to read as a filled badge
// rather than the library-wide default (5).
const ICON_DEFAULT_SHADE: ColorShade = ColorShade::S6;

fn icon_color_parts(value: Option<&ThemeAwareValue>) -> (Color, ColorShade) {
    match value {
        Some(ThemeAwareValue::Color(color)) => (*color, ICON_DEFAULT_SHADE),
        Some(ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade))) => (*color, *shade),
        Some(ThemeAwareValue::ColorValue(ColorValue::Contrast(color, shade))) => (*color, *shade),
        _ => (Color::Primary, ICON_DEFAULT_SHADE),
    }
}

fn icon_size(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Size(size) => SizeCss::ICON_SIZE.value(*size).into(),
        other => other.clone(),
    }
}

fn icon_variant_sx(variant: IconVariant, color: Color, shade: ColorShade) -> Sx {
    let base = ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade));

    match variant {
        IconVariant::Filled => {
            let contrast = ThemeAwareValue::ColorValue(ColorValue::Contrast(color, shade));
            sx().background(base).color(contrast)
        }
        IconVariant::Outlined => sx()
            .background("transparent")
            .border("1px solid")
            .border_color(base.clone())
            .color(base),
        IconVariant::Transparent => sx().background("transparent").color(base),
    }
}

static ICON_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .width(SizeCss::ICON_SIZE.value(Size::Md))
        .height(SizeCss::ICON_SIZE.value(Size::Md))
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .selector("& svg", sx().width("100%").height("100%"))
});

fn icon_dynamic_sx(props: &IconProps) -> Sx {
    let variant = props.variant.as_ref().copied().unwrap_or_default();
    let (color, shade) = icon_color_parts(props.color.as_ref());

    icon_variant_sx(variant, color, shade)
        .apply_if(props.size.as_ref().map(icon_size), |sx, size| {
            sx.width(size.clone()).height(size)
        })
        .apply_if(props.radius.as_ref(), |sx, radius| {
            sx.border_radius(radius.clone())
        })
}

#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    /// Which element to render as - `span` by default.
    #[props(default, into)]
    component: Input<HtmlTag>,
    #[props(default, into)]
    variant: Input<IconVariant>,
    #[props(default, into)]
    color: Input<ThemeAwareValue>,
    #[props(default, into)]
    size: Input<ThemeAwareValue>,
    #[props(default, into)]
    radius: Input<ThemeAwareValue>,
    children: Element,
}

/// Wraps an svg child in a sized, colored badge. `color` sets the
/// container's CSS `color`, which any child svg using `currentColor` for its
/// fill/stroke - the convention most icon sets follow - then inherits.
#[component]
pub fn Icon(props: IconProps) -> Element {
    let component = props.component.as_ref().copied().unwrap_or(HtmlTag::Span);
    let dynamic_class =
        crate::hooks::use_css(&icon_dynamic_sx(&props), crate::CssLayer::UserDynamic);

    rsx! {
        Box {
            component,
            class: class_list([props.class, dynamic_class]),
            sx: props.sx,
            states: props.states,
            framework_sx: &ICON_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
