use dioxus::prelude::*;

use crate::{
    components::{
        Box, HtmlTag, Input, States, Variables,
        common::{base_color, base_props, contrast_color, input_from_str},
        variables,
    },
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, ICON_SIZE, Size, SizeCss},
};

str_enum! {
    pub enum IconVariant {
        #[default]
        Filled = "filled",
        Outlined = "outlined" | "outline",
        Transparent = "transparent",
    }
}

input_from_str!(IconVariant);

// Matches Button's shade: bold enough to read as a filled badge.

// A bare color name carries no shade, so it takes the default above rather
// than the sx pipeline's generic one. Everything else passes through.
/// Structural chrome for `variant`. The arguments are `var()` names, not
/// resolved values, so `ActionIcon` reuses this under its own.
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
    let base = sx()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .width(ICON_SIZE.overridable(Size::Md))
        .height(ICON_SIZE.overridable(Size::Md))
        .border_radius(ICON_RADIUS_VAR.value_or(SizeCss::RADIUS.value(Size::Sm)))
        .selector("& svg", sx().width("100%").height("100%"));

    IconVariant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            icon_variant_sx(variant, &ICON_COLOR_VAR, &ICON_CONTRAST_VAR),
        )
    })
});

fn icon_variables(props: &IconProps) -> Variables {
    let base = base_color(props.color.as_ref());
    let contrast = contrast_color(&base);

    variables()
        .with(ICON_COLOR_VAR, base.resolve(None))
        .with(ICON_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(
            ICON_SIZE.override_var(),
            props.size.resolve(Some(ICON_SIZE)),
        )
        .with(ICON_RADIUS_VAR, props.radius.resolve(Some(SizeCss::RADIUS)))
}

base_props! {
    pub struct IconProps {
        /// Element to render as; `span` by default.
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

/// Wraps an svg child in a sized, colored badge. `color` sets the container's
/// CSS `color`, which a `currentColor` svg then inherits.
#[component]
pub fn Icon(props: IconProps) -> Element {
    let component = props.component.copied_or(HtmlTag::Span);
    let variant = props.variant.copied_or_default();
    let variables = icon_variables(&props);

    let states = props
        .states
        .unwrap_or_default()
        .with(variant.state_name(), true);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Color, ColorShade, ColorValue};

    fn icon_props(color: Input<ThemeAwareValue>) -> IconProps {
        IconProps {
            component: Input::None,
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            variant: Input::None,
            color,
            size: Input::None,
            radius: Input::None,
            children: rsx! {},
        }
    }

    #[test]
    fn a_bare_theme_color_becomes_a_shade_plus_its_contrast() {
        let variables = icon_variables(&icon_props(Color::Error.into())).to_string();

        assert!(variables.contains(&format!(
            "{}:{};",
            ICON_COLOR_VAR.name(),
            ColorValue::Shade(Color::Error, ColorShade::S6).value()
        )));
        assert!(variables.contains(ICON_CONTRAST_VAR.name()));
    }

    /// A literal has no theme shade, so no contrast to pair with it.
    #[test]
    fn a_literal_color_emits_no_contrast() {
        let variables = icon_variables(&icon_props("#123456".into())).to_string();

        assert!(variables.contains(&format!("{}:#123456;", ICON_COLOR_VAR.name())));
        assert!(!variables.contains(ICON_CONTRAST_VAR.name()));
    }

    #[test]
    fn each_variant_renders_its_own_css() {
        let class_of =
            |variant| icon_variant_sx(variant, &ICON_COLOR_VAR, &ICON_CONTRAST_VAR).class_name();

        assert_ne!(
            class_of(IconVariant::Filled),
            class_of(IconVariant::Outlined)
        );
        assert_ne!(
            class_of(IconVariant::Outlined),
            class_of(IconVariant::Transparent)
        );
        assert_eq!(class_of(IconVariant::Filled), class_of(IconVariant::Filled));
    }
}
