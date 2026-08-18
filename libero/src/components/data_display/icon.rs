use dioxus::prelude::*;

use crate::{
    components::{
        Box, HtmlTag, Input, States, Variables,
        common::{base_color, base_props, contrast_color},
        variables,
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, ICON_SIZE, Size, SizeCss},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IconVariant {
    #[default]
    Filled,
    Outlined,
    Transparent,
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

// A bare theme color name (e.g. "primary") has no shade of its own, so it's
// resolved to our own default shade here rather than the sx pipeline's
// generic default (5). Anything else - an explicit shade/contrast, or a
// literal value like "red"/#123456/rgb(...) - passes through unchanged and
// is resolved by the normal sx-to-css pipeline. Only a genuinely unset
// `color` falls back to the library's default color.
/// Structural chrome for `variant`, referencing `color_var`/`contrast_var`
/// (a `var()` name each, not a resolved value) - shared with `ActionIcon`,
/// which reuses this exact shape under its own var names since it builds
/// directly on `Icon`'s own variant styling.
pub(crate) fn icon_variant_sx(
    variant: IconVariant,
    color_var: &CssVar,
    contrast_var: &CssVar,
) -> Sx {
    match variant {
        IconVariant::Filled => sx()
            .background(color_var.value())
            .color(contrast_var.value_or("inherit")),
        IconVariant::Outlined => sx()
            .background("transparent")
            .border("1px solid")
            .border_color(color_var.value())
            .color(color_var.value()),
        IconVariant::Transparent => sx().background("transparent").color(color_var.value()),
    }
}

pub(crate) const ICON_COLOR_VAR: CssVar = CssVar::new("--lsx-icon-color");
pub(crate) const ICON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-icon-contrast");
const ICON_RADIUS_VAR: CssVar = CssVar::new("--lsx-icon-radius");

static ICON_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .width(ICON_SIZE.overridable(Size::Md))
        .height(ICON_SIZE.overridable(Size::Md))
        .border_radius(ICON_RADIUS_VAR.value_or(SizeCss::RADIUS.value(Size::Sm)))
        .selector("& svg", sx().width("100%").height("100%"))
        .when(
            "filled",
            icon_variant_sx(IconVariant::Filled, &ICON_COLOR_VAR, &ICON_CONTRAST_VAR),
        )
        .when(
            "outlined",
            icon_variant_sx(IconVariant::Outlined, &ICON_COLOR_VAR, &ICON_CONTRAST_VAR),
        )
        .when(
            "transparent",
            icon_variant_sx(
                IconVariant::Transparent,
                &ICON_COLOR_VAR,
                &ICON_CONTRAST_VAR,
            ),
        )
});

pub(crate) fn variant_token(variant: IconVariant) -> &'static str {
    match variant {
        IconVariant::Filled => "filled",
        IconVariant::Outlined => "outlined",
        IconVariant::Transparent => "transparent",
    }
}

fn icon_variables(props: &IconProps) -> Variables {
    let base = base_color(props.color.as_ref());
    let contrast = contrast_color(&base);

    variables()
        .with(ICON_COLOR_VAR, base.resolve(None))
        .with(ICON_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(
            ICON_SIZE.override_var(),
            props.size.as_ref().and_then(|v| v.resolve(Some(ICON_SIZE))),
        )
        .with(
            ICON_RADIUS_VAR,
            props
                .radius
                .as_ref()
                .and_then(|v| v.resolve(Some(SizeCss::RADIUS))),
        )
}

base_props! {
    pub struct IconProps {
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
}

/// Wraps an svg child in a sized, colored badge. `color` sets the
/// container's CSS `color`, which any child svg using `currentColor` for its
/// fill/stroke - the convention most icon sets follow - then inherits.
#[component]
pub fn Icon(props: IconProps) -> Element {
    let component = props.component.as_ref().copied().unwrap_or(HtmlTag::Span);
    let variant = props.variant.as_ref().copied().unwrap_or_default();
    let variables = icon_variables(&props);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(variant_token(variant), true);

    rsx! {
        Box {
            component,
            class: props.class,
            sx: props.sx,
            states,
            variables,
            framework_sx: &ICON_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
